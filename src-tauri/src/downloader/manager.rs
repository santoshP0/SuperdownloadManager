use anyhow::Result;
use serde_json::json;
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};
use tauri_plugin_notification::NotificationExt;
use tokio::sync::Mutex;
use uuid::Uuid;

use super::chunk::{
    self, check_disk_space, create_chunks, download_chunk, download_single, merge_chunks,
};
use super::speed::SpeedTracker;
use super::task::{ChunkState, DownloadItem, DownloadStatus, ResumeState};
use crate::persistence;
use crate::settings::Settings;

const MAX_RETRIES: u8 = 3;

struct ActiveHandle {
    cancelled: Arc<AtomicBool>,
}

pub struct DownloadManager {
    pub downloads: HashMap<String, DownloadItem>,
    active: HashMap<String, ActiveHandle>,
    pub queue: VecDeque<String>,
    pub max_concurrent: usize,
    pub speed_limit_bps: u64,
    pub notifications_enabled: bool,
}

impl DownloadManager {
    pub fn new() -> Self {
        Self {
            downloads: persistence::load_history(),
            active: HashMap::new(),
            queue: VecDeque::new(),
            max_concurrent: 3,
            speed_limit_bps: 0,
            notifications_enabled: true,
        }
    }

    pub fn apply_settings(&mut self, s: &Settings) {
        self.max_concurrent = s.max_concurrent;
        self.speed_limit_bps = s.speed_limit_kbps as u64 * 1024;
        self.notifications_enabled = s.notifications;
    }

    pub fn get_all(&self) -> Vec<DownloadItem> {
        let mut items: Vec<_> = self.downloads.values().cloned().collect();
        items.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        items
    }

    fn has_capacity(&self) -> bool {
        self.active.len() < self.max_concurrent
    }
}

// ─── Public async API ─────────────────────────────────────────────────────────

pub async fn add_download(
    mgr: Arc<Mutex<DownloadManager>>,
    url: String,
    filename: String,
    save_path: String,
    chunk_count: u8,
    app: AppHandle,
) -> Result<String> {
    let id = Uuid::new_v4().to_string();
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();

    let item = DownloadItem {
        id: id.clone(),
        url,
        filename,
        save_path,
        total_size: 0,
        downloaded: 0,
        status: DownloadStatus::Queued,
        speed: 0.0,
        eta_seconds: 0,
        created_at: now,
        chunk_count,
        error: None,
        supports_resume: false,
        retry_count: 0,
    };

    let should_start = {
        let mut m = mgr.lock().await;
        m.downloads.insert(id.clone(), item);
        persistence::save_history(&m.downloads);
        if m.has_capacity() { true } else { m.queue.push_back(id.clone()); false }
    };

    if should_start {
        launch(mgr, id.clone(), app);
    }
    Ok(id)
}

pub async fn pause_download(mgr: Arc<Mutex<DownloadManager>>, id: &str) -> Result<()> {
    let mut m = mgr.lock().await;
    if let Some(h) = m.active.get(id) {
        h.cancelled.store(true, Ordering::Relaxed);
    }
    m.queue.retain(|q| q != id);
    if let Some(item) = m.downloads.get_mut(id) {
        item.status = DownloadStatus::Paused;
        item.speed = 0.0;
        item.eta_seconds = 0;
    }
    persistence::save_history(&m.downloads);
    Ok(())
}

pub async fn resume_download(
    mgr: Arc<Mutex<DownloadManager>>,
    id: &str,
    app: AppHandle,
) -> Result<()> {
    let should_start = {
        let mut m = mgr.lock().await;
        if let Some(item) = m.downloads.get_mut(id) {
            item.status = DownloadStatus::Queued;
            item.error = None;
            item.retry_count = 0;
        }
        persistence::save_history(&m.downloads);
        if m.has_capacity() { true } else { m.queue.push_back(id.to_string()); false }
    };

    if should_start { launch(mgr, id.to_string(), app); }
    Ok(())
}

pub async fn cancel_download(mgr: Arc<Mutex<DownloadManager>>, id: &str) -> Result<()> {
    let (save_path, dl_id) = {
        let mut m = mgr.lock().await;
        m.queue.retain(|q| q != id);
        if let Some(h) = m.active.get(id) { h.cancelled.store(true, Ordering::Relaxed); }
        m.active.remove(id);
        let info = m.downloads.get(id).map(|i| (i.save_path.clone(), i.id.clone()));
        m.downloads.remove(id);
        persistence::save_history(&m.downloads);
        info.unzip()
    };

    if let (Some(sp), Some(did)) = (save_path, dl_id) {
        let _ = tokio::fs::remove_dir_all(PathBuf::from(sp).join(format!(".sdm_{}", did))).await;
    }
    Ok(())
}

// ─── Internal ────────────────────────────────────────────────────────────────

fn launch(mgr: Arc<Mutex<DownloadManager>>, id: String, app: AppHandle) {
    tokio::spawn(drive(mgr, id, app));
}

fn resolve_filename(dir: &Path, filename: &str) -> (PathBuf, String) {
    let path = dir.join(filename);
    if !path.exists() { return (path, filename.to_string()); }

    let stem = Path::new(filename).file_stem().unwrap_or_default().to_string_lossy().to_string();
    let ext = Path::new(filename)
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();

    let mut n = 1u32;
    loop {
        let name = format!("{} ({}){}", stem, n, ext);
        let p = dir.join(&name);
        if !p.exists() { return (p, name); }
        n += 1;
    }
}

// ─── Driver loop ──────────────────────────────────────────────────────────────

async fn drive(mgr: Arc<Mutex<DownloadManager>>, initial_id: String, app: AppHandle) {
    let mut current_id = initial_id;

    loop {
        let (url, filename, save_path, chunk_count, speed_limit_bps) = {
            let mut m = mgr.lock().await;
            let Some(item) = m.downloads.get_mut(&current_id) else { break };
            item.status = DownloadStatus::Downloading;
            item.retry_count = 0;
            (item.url.clone(), item.filename.clone(), item.save_path.clone(),
             item.chunk_count, m.speed_limit_bps)
        };

        let cancelled = Arc::new(AtomicBool::new(false));
        let progress = Arc::new(AtomicU64::new(0));

        {
            let mut m = mgr.lock().await;
            m.active.insert(current_id.clone(), ActiveHandle { cancelled: cancelled.clone() });
        }

        // ── Retry loop ────────────────────────────────────────────────────────
        let mut attempt = 0u8;
        let result = loop {
            progress.store(0, Ordering::Relaxed);

            let r = run_download(
                url.clone(), filename.clone(), save_path.clone(), chunk_count,
                speed_limit_bps, cancelled.clone(), progress.clone(),
                mgr.clone(), current_id.clone(), app.clone(),
            ).await;

            let was_cancelled = cancelled.load(Ordering::Relaxed);
            let is_paused = mgr.lock().await.downloads.get(&current_id)
                .map(|i| i.status == DownloadStatus::Paused).unwrap_or(false);

            if r.is_ok() || was_cancelled || is_paused || attempt >= MAX_RETRIES { break r; }

            attempt += 1;
            let delay = 2u64.pow(attempt as u32);

            {
                let mut m = mgr.lock().await;
                if let Some(item) = m.downloads.get_mut(&current_id) {
                    item.status = DownloadStatus::Retrying;
                    item.retry_count = attempt;
                    item.error = Some(format!("Retry {}/{} in {}s…", attempt, MAX_RETRIES, delay));
                    item.speed = 0.0;
                }
                persistence::save_history(&m.downloads);
            }

            let _ = app.emit("download://retrying", json!({ "id": current_id, "attempt": attempt }));
            tokio::time::sleep(Duration::from_secs(delay)).await;

            {
                let mut m = mgr.lock().await;
                if let Some(item) = m.downloads.get_mut(&current_id) {
                    item.status = DownloadStatus::Downloading;
                    item.error = None;
                }
            }
        };
        // ─────────────────────────────────────────────────────────────────────

        let was_paused;
        let (notifications_on, filename_for_notif) = {
            let m = mgr.lock().await;
            let notif = m.notifications_enabled;
            let fname = m.downloads.get(&current_id).map(|i| i.filename.clone()).unwrap_or_default();
            (notif, fname)
        };

        let next_id = {
            let mut m = mgr.lock().await;
            m.active.remove(&current_id);

            was_paused = m.downloads.get(&current_id)
                .map(|i| i.status == DownloadStatus::Paused).unwrap_or(false);

            match &result {
                Ok(()) if !was_paused => {
                    if let Some(item) = m.downloads.get_mut(&current_id) {
                        item.status = DownloadStatus::Completed;
                        item.downloaded = item.total_size;
                        item.speed = 0.0;
                        item.eta_seconds = 0;
                        item.error = None;
                    }
                    let _ = app.emit("download://completed", json!({ "id": current_id }));
                }
                Ok(()) => {}
                Err(e) => {
                    if let Some(item) = m.downloads.get_mut(&current_id) {
                        item.status = DownloadStatus::Failed;
                        item.error = Some(e.to_string());
                        item.speed = 0.0;
                        item.eta_seconds = 0;
                    }
                    let _ = app.emit("download://failed",
                        json!({ "id": current_id, "error": e.to_string() }));
                }
            }
            persistence::save_history(&m.downloads);
            m.queue.pop_front()
        };

        // Desktop notification on completion
        if result.is_ok() && !was_paused && notifications_on {
            let _ = app.notification()
                .builder()
                .title("Download Complete")
                .body(&format!("{} downloaded successfully", filename_for_notif))
                .show();
        }

        match next_id {
            Some(id) => current_id = id,
            None => break,
        }
    }
}

// ─── Core download logic ──────────────────────────────────────────────────────

async fn run_download(
    url: String,
    filename: String,
    save_path: String,
    chunk_count: u8,
    speed_limit_bps: u64,
    cancelled: Arc<AtomicBool>,
    progress: Arc<AtomicU64>,
    mgr: Arc<Mutex<DownloadManager>>,
    id: String,
    app: AppHandle,
) -> Result<()> {
    let (total_size, supports_ranges) = chunk::probe_url(&url).await?;

    // Disk space check
    check_disk_space(&save_path, total_size)?;

    // Resolve filename conflict before touching filesystem
    let (final_path, actual_filename) = resolve_filename(Path::new(&save_path), &filename);

    {
        let mut m = mgr.lock().await;
        if let Some(item) = m.downloads.get_mut(&id) {
            item.total_size = total_size;
            item.supports_resume = supports_ranges;
            item.filename = actual_filename.clone();
        }
    }

    let temp_dir = PathBuf::from(&save_path).join(format!(".sdm_{}", id));
    tokio::fs::create_dir_all(&temp_dir).await?;

    let resume_path = temp_dir.join("resume.json");
    let initial_chunks: Option<Vec<ChunkState>> = if resume_path.exists() {
        let data = tokio::fs::read_to_string(&resume_path).await.unwrap_or_default();
        serde_json::from_str::<ResumeState>(&data).ok().map(|rs| rs.chunks)
    } else {
        None
    };

    if supports_ranges && total_size > 2_000_000 && chunk_count > 1 {
        let chunks = initial_chunks.unwrap_or_else(|| create_chunks(total_size, chunk_count));
        let already: u64 = chunks.iter().map(|c| c.downloaded).sum();
        progress.store(already, Ordering::Relaxed);

        // Per-chunk speed limit
        let per_chunk_limit = if speed_limit_bps > 0 {
            (speed_limit_bps / chunk_count as u64).max(1)
        } else {
            0
        };

        let mon = start_monitor(
            progress.clone(), cancelled.clone(), total_size, mgr.clone(), id.clone(), app.clone(),
        );

        let tasks: Vec<_> = chunks.iter().map(|c| {
            let url = url.clone();
            let path = temp_dir.join(format!("chunk_{:03}", c.id));
            let chunk = c.clone();
            let prog = progress.clone();
            let cancel = cancelled.clone();
            tokio::spawn(async move {
                download_chunk(&url, &path, &chunk, prog, cancel, per_chunk_limit).await
            })
        }).collect();

        let results = futures::future::join_all(tasks).await;
        mon.abort();

        if cancelled.load(Ordering::Relaxed) {
            let updated: Vec<ChunkState> = chunks.iter().map(|c| {
                let part = temp_dir.join(format!("chunk_{:03}", c.id));
                let done = std::fs::metadata(&part).map(|m| m.len()).unwrap_or(c.downloaded);
                ChunkState { downloaded: done, ..c.clone() }
            }).collect();
            if let Some(item) = mgr.lock().await.downloads.get(&id).cloned() {
                let rs = ResumeState { item, chunks: updated };
                if let Ok(json) = serde_json::to_string(&rs) {
                    let _ = tokio::fs::write(&resume_path, json).await;
                }
            }
            return Ok(());
        }

        for r in results { r??; }
        merge_chunks(&temp_dir, &final_path, chunks.len()).await?;
        let _ = tokio::fs::remove_dir_all(&temp_dir).await;
    } else {
        let mon = start_monitor(
            progress.clone(), cancelled.clone(), total_size, mgr.clone(), id.clone(), app.clone(),
        );
        let res = download_single(&url, &final_path, progress, cancelled.clone(), speed_limit_bps).await;
        mon.abort();
        let _ = tokio::fs::remove_dir_all(&temp_dir).await;
        res?;
    }

    Ok(())
}

fn start_monitor(
    progress: Arc<AtomicU64>,
    cancelled: Arc<AtomicBool>,
    total_size: u64,
    mgr: Arc<Mutex<DownloadManager>>,
    id: String,
    app: AppHandle,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut tracker = SpeedTracker::new();
        let mut interval = tokio::time::interval(Duration::from_millis(500));
        loop {
            interval.tick().await;
            if cancelled.load(Ordering::Relaxed) { break; }

            let downloaded = progress.load(Ordering::Relaxed);
            let speed = tracker.update(downloaded);
            let eta = if speed > 0.0 && total_size > downloaded {
                ((total_size - downloaded) as f64 / speed) as u64
            } else { 0 };

            {
                let mut m = mgr.lock().await;
                if let Some(item) = m.downloads.get_mut(&id) {
                    item.downloaded = downloaded;
                    item.speed = speed;
                    item.eta_seconds = eta;
                }
            }
            let _ = app.emit("download://progress",
                json!({ "id": id, "downloaded": downloaded, "total": total_size,
                         "speed": speed, "eta_seconds": eta }));
        }
    })
}
