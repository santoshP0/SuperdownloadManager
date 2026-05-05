use anyhow::{anyhow, Result};
use reqwest::Client;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::fs;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::time::Instant;

use super::task::ChunkState;

pub const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36";

/// Probe the URL: returns (content_length, supports_byte_ranges).
pub async fn probe_url(client: &Client, url: &str) -> Result<(u64, bool)> {
    let resp = client.head(url).send().await?;

    if !resp.status().is_success() {
        // Fallback to GET if HEAD is denied (common on some CDNs)
        let resp = client.get(url).header("Range", "bytes=0-0").send().await?;
        if !resp.status().is_success() {
            return Err(anyhow!("Server returned {}", resp.status()));
        }
        let size = resp
            .headers()
            .get("content-range")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split('/').last())
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);
        return Ok((size, true));
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

use std::collections::HashMap;

/// Download one chunk using HTTP Range.
pub async fn download_chunk(
    client: &Client,
    url: &str,
    temp_path: &Path,
    chunk: &ChunkState,
    progress: Arc<AtomicU64>,
    cancelled: Arc<AtomicBool>,
    speed_limit_bps: u64,
    headers: Option<HashMap<String, String>>,
) -> Result<()> {
    let resume_offset = chunk.start + chunk.downloaded;
    let end = chunk.end;

    if resume_offset > end {
        return Ok(());
    }

    let range_header = format!("bytes={}-{}", resume_offset, end);
    let mut req = client.get(url).header("Range", &range_header);

    if let Some(h) = headers {
        for (k, v) in h {
            req = req.header(k, v);
        }
    }

    let resp = req.send().await?;

    if !resp.status().is_success() && resp.status().as_u16() != 206 {
        return Err(anyhow!("Chunk {} got status {}", chunk.id, resp.status()));
    }

    let file = if chunk.downloaded > 0 {
        fs::OpenOptions::new().create(true).append(true).open(temp_path).await?
    } else {
        fs::File::create(temp_path).await?
    };
    
    let mut writer = BufWriter::with_capacity(512 * 1024, file);
    let mut stream = resp.bytes_stream();

    use futures::StreamExt;
    let mut window_bytes = 0u64;
    let mut window_start = Instant::now();

    while let Some(item) = stream.next().await {
        if cancelled.load(Ordering::Relaxed) {
            writer.flush().await?;
            return Ok(());
        }
        let bytes = item?;
        let n = bytes.len() as u64;
        writer.write_all(&bytes).await?;
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

    writer.flush().await?;
    Ok(())
}

/// Fallback single-stream download (no Range support).
pub async fn download_single(
    client: &Client,
    url: &str,
    save_path: &Path,
    progress: Arc<AtomicU64>,
    cancelled: Arc<AtomicBool>,
    speed_limit_bps: u64,
    headers: Option<HashMap<String, String>>,
) -> Result<()> {
    let mut req = client.get(url);
    if let Some(h) = headers {
        for (k, v) in h {
            req = req.header(k, v);
        }
    }
    let resp = req.send().await?;

    if !resp.status().is_success() {
        return Err(anyhow!("Server returned {}", resp.status()));
    }

    let file = fs::File::create(save_path).await?;
    let mut writer = BufWriter::with_capacity(512 * 1024, file);
    let mut stream = resp.bytes_stream();

    use futures::StreamExt;
    let mut window_bytes = 0u64;
    let mut window_start = Instant::now();

    while let Some(item) = stream.next().await {
        if cancelled.load(Ordering::Relaxed) {
            writer.flush().await?;
            return Ok(());
        }
        let bytes = item?;
        let n = bytes.len() as u64;
        writer.write_all(&bytes).await?;
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

    writer.flush().await?;
    Ok(())
}


/// Concatenate chunk_{000..n-1} files from temp_dir into save_path.
/// This uses streaming to avoid loading large files into memory.
pub async fn merge_chunks(
    temp_dir: &Path,
    save_path: &Path,
    chunk_count: usize,
) -> Result<()> {
    let out_file = fs::File::create(save_path).await?;
    let mut writer = BufWriter::with_capacity(1024 * 1024, out_file); // 1MB buffer for merging

    for i in 0..chunk_count {
        let part_path = temp_dir.join(format!("chunk_{:03}", i));
        if part_path.exists() {
            let mut part_file = fs::File::open(&part_path).await?;
            tokio::io::copy(&mut part_file, &mut writer).await?;
            // Optional: delete chunk after merging to save space
            let _ = fs::remove_file(part_path).await;
        }
    }
    writer.flush().await?;
    Ok(())
}

/// Check available disk space; returns Err if not enough.
pub fn check_disk_space(save_path: &str, required: u64) -> Result<()> {
    if required == 0 {
        return Ok(());
    }
    let path = std::path::Path::new(save_path);
    // Walk up to the first existing directory
    let mut check = path;
    let existing = loop {
        if check.exists() {
            break check;
        }
        match check.parent() {
            Some(p) => check = p,
            None => return Ok(()), // can't determine, skip
        }
    };
    let available = fs2::available_space(existing)?;
    let needed = required + 10 * 1024 * 1024; // +10 MB buffer
    if available < needed {
        anyhow::bail!(
            "Not enough disk space: need {:.0} MB, only {:.0} MB available",
            needed as f64 / 1_048_576.0,
            available as f64 / 1_048_576.0
        );
    }
    Ok(())
}

