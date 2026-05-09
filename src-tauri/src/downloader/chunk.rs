use anyhow::{anyhow, Result};
use bytes::Bytes;
use reqwest::Client;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::fs;
use tokio::io::{AsyncSeekExt, AsyncWriteExt, BufWriter};
use tokio::sync::mpsc;
use tokio::time::Instant;

use super::task::ChunkState;

const WRITE_BUF: usize = 512 * 1024; // 512 KB write-side buffer (reduces spawn_blocking calls)
const PIPE_CAP: usize = 64;           // channel slots ≈ 1 MB of in-flight bytes per chunk

pub fn build_client() -> Client {
    Client::builder()
        .user_agent("SuperDownloadManager/0.1 (Windows)")
        .tcp_keepalive(Duration::from_secs(30))
        // Disable Nagle so Range request headers are sent to the server immediately
        // rather than waiting for more data to batch.
        .tcp_nodelay(true)
        .build()
        .expect("failed to initialise HTTP client — TLS backend unavailable")
}

/// Probe the URL: returns (content_length, supports_byte_ranges).
pub async fn probe_url(url: &str) -> Result<(u64, bool)> {
    let client = build_client();
    let resp = client.head(url).send().await?;

    if !resp.status().is_success() {
        return Err(anyhow!("Server returned {}", resp.status()));
    }

    let supports_ranges = resp
        .headers()
        .get("accept-ranges")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.eq_ignore_ascii_case("bytes"))
        .unwrap_or(false);

    let size = resp
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);

    Ok((size, supports_ranges))
}

/// Try to extract a filename from a URL's path segment.
pub fn filename_from_url(url: &str) -> String {
    url.split('?')
        .next()
        .unwrap_or(url)
        .split('/')
        .last()
        .filter(|s| !s.is_empty())
        .unwrap_or("download")
        .to_string()
}

/// Divide total_size into chunk_count evenly-distributed ChunkStates.
pub fn create_chunks(total_size: u64, chunk_count: u8) -> Vec<ChunkState> {
    let n = chunk_count as u64;
    let base = total_size / n;
    (0..chunk_count)
        .map(|i| {
            let start = i as u64 * base;
            let end = if i == chunk_count - 1 {
                total_size - 1
            } else {
                start + base - 1
            };
            ChunkState { id: i, start, end, downloaded: 0 }
        })
        .collect()
}

/// Download one chunk using HTTP Range, writing directly into `out_path` at the
/// correct byte offset.
///
/// A dedicated reader task drains the TCP receive buffer as fast as possible and
/// forwards bytes through a channel. The caller's task writes those bytes to disk
/// concurrently. This keeps the TCP receive window open at all times, eliminating
/// the stutter that occurs when a serial read→write loop allows the socket buffer
/// to fill and causes the server to pause.
pub async fn download_chunk(
    url: &str,
    out_path: &Path,
    chunk: &ChunkState,
    progress: Arc<AtomicU64>,
    chunk_downloaded: Arc<AtomicU64>,
    cancelled: Arc<AtomicBool>,
    speed_limit_bps: u64,
    client: &Client,
) -> Result<()> {
    let resume_offset = chunk.start + chunk.downloaded;
    let end = chunk.end;

    if resume_offset > end {
        return Ok(());
    }

    let range_header = format!("bytes={}-{}", resume_offset, end);
    let resp = client.get(url).header("Range", &range_header).send().await?;

    if !resp.status().is_success() && resp.status().as_u16() != 206 {
        return Err(anyhow!("Chunk {} got status {}", chunk.id, resp.status()));
    }

    let file = fs::OpenOptions::new().write(true).open(out_path).await?;
    let mut file = BufWriter::with_capacity(WRITE_BUF, file);
    file.seek(std::io::SeekFrom::Start(resume_offset)).await?;

    // Reader task: forwards network bytes to the channel without waiting for disk.
    let (tx, mut rx) = mpsc::channel::<reqwest::Result<Bytes>>(PIPE_CAP);
    let read_task = tokio::spawn(async move {
        let mut resp = resp;
        loop {
            match resp.chunk().await {
                Ok(Some(b)) => { if tx.send(Ok(b)).await.is_err() { return; } }
                Ok(None)    => return,
                Err(e)      => { let _ = tx.send(Err(e)).await; return; }
            }
        }
    });

    // Writer: consumes the channel and writes to disk while the reader fills it.
    let mut window_bytes = 0u64;
    let mut window_start = Instant::now();

    while let Some(result) = rx.recv().await {
        let bytes = result?; // propagate any network error

        if cancelled.load(Ordering::Relaxed) {
            file.flush().await?;
            read_task.abort();
            return Ok(());
        }

        let n = bytes.len() as u64;
        file.write_all(&bytes).await?;
        progress.fetch_add(n, Ordering::Relaxed);
        chunk_downloaded.fetch_add(n, Ordering::Relaxed);

        if speed_limit_bps > 0 {
            window_bytes += n;
            let elapsed = window_start.elapsed();
            let allowed = Duration::from_secs_f64(window_bytes as f64 / speed_limit_bps as f64);
            if allowed > elapsed {
                tokio::time::sleep(allowed - elapsed).await;
            }
            if elapsed >= Duration::from_secs(5) {
                window_bytes = 0;
                window_start = Instant::now();
            }
        }
    }

    read_task.await.ok();
    file.flush().await?;
    Ok(())
}

/// Fallback single-stream download (no Range support). Same pipelined
/// reader/writer approach so the TCP window stays open during disk writes.
pub async fn download_single(
    url: &str,
    save_path: &Path,
    progress: Arc<AtomicU64>,
    cancelled: Arc<AtomicBool>,
    speed_limit_bps: u64,
    client: &Client,
) -> Result<()> {
    let resp = client.get(url).send().await?;

    if !resp.status().is_success() {
        return Err(anyhow!("Server returned {}", resp.status()));
    }

    let mut file = BufWriter::with_capacity(WRITE_BUF, fs::File::create(save_path).await?);

    let (tx, mut rx) = mpsc::channel::<reqwest::Result<Bytes>>(PIPE_CAP * 2);
    let read_task = tokio::spawn(async move {
        let mut resp = resp;
        loop {
            match resp.chunk().await {
                Ok(Some(b)) => { if tx.send(Ok(b)).await.is_err() { return; } }
                Ok(None)    => return,
                Err(e)      => { let _ = tx.send(Err(e)).await; return; }
            }
        }
    });

    let mut window_bytes = 0u64;
    let mut window_start = Instant::now();

    while let Some(result) = rx.recv().await {
        let bytes = result?;

        if cancelled.load(Ordering::Relaxed) {
            file.flush().await?;
            read_task.abort();
            return Ok(());
        }

        let n = bytes.len() as u64;
        file.write_all(&bytes).await?;
        progress.fetch_add(n, Ordering::Relaxed);

        if speed_limit_bps > 0 {
            window_bytes += n;
            let elapsed = window_start.elapsed();
            let allowed = Duration::from_secs_f64(window_bytes as f64 / speed_limit_bps as f64);
            if allowed > elapsed {
                tokio::time::sleep(allowed - elapsed).await;
            }
            if elapsed >= Duration::from_secs(5) {
                window_bytes = 0;
                window_start = Instant::now();
            }
        }
    }

    read_task.await.ok();
    file.flush().await?;
    Ok(())
}

/// Check available disk space; returns Err if not enough.
pub fn check_disk_space(save_path: &str, required: u64) -> Result<()> {
    if required == 0 {
        return Ok(());
    }
    let path = std::path::Path::new(save_path);
    let mut check = path;
    let existing = loop {
        if check.exists() {
            break check;
        }
        match check.parent() {
            Some(p) => check = p,
            None => return Ok(()),
        }
    };
    let available = fs2::available_space(existing)?;
    let needed = required + 10 * 1024 * 1024;
    if available < needed {
        anyhow::bail!(
            "Not enough disk space: need {:.0} MB, only {:.0} MB available",
            needed as f64 / 1_048_576.0,
            available as f64 / 1_048_576.0
        );
    }
    Ok(())
}
