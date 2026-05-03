use anyhow::{anyhow, Result};
use reqwest::Client;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use super::task::ChunkState;

fn build_client() -> Client {
    Client::builder()
        .user_agent("SuperDownloadManager/0.1 (Windows)")
        .tcp_keepalive(std::time::Duration::from_secs(30))
        .build()
        .unwrap()
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

/// Download one chunk using HTTP Range, appending to temp_path.
/// progress is a global bytes-downloaded counter shared across all chunks.
pub async fn download_chunk(
    url: &str,
    temp_path: &Path,
    chunk: &ChunkState,
    progress: Arc<AtomicU64>,
    cancelled: Arc<AtomicBool>,
) -> Result<()> {
    let resume_offset = chunk.start + chunk.downloaded;
    let end = chunk.end;

    if resume_offset > end {
        return Ok(()); // already done
    }

    let client = build_client();
    let range_header = format!("bytes={}-{}", resume_offset, end);

    let resp = client
        .get(url)
        .header("Range", &range_header)
        .send()
        .await?;

    if !resp.status().is_success() && resp.status().as_u16() != 206 {
        return Err(anyhow!("Chunk {} got status {}", chunk.id, resp.status()));
    }

    let mut file = if chunk.downloaded > 0 {
        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(temp_path)
            .await?
    } else {
        fs::File::create(temp_path).await?
    };

    let mut resp = resp;
    while let Some(bytes) = resp.chunk().await? {
        if cancelled.load(Ordering::Relaxed) {
            file.flush().await?;
            return Ok(());
        }
        let n = bytes.len() as u64;
        file.write_all(&bytes).await?;
        progress.fetch_add(n, Ordering::Relaxed);
    }

    file.flush().await?;
    Ok(())
}

/// Fallback single-stream download (no Range support).
pub async fn download_single(
    url: &str,
    save_path: &Path,
    progress: Arc<AtomicU64>,
    cancelled: Arc<AtomicBool>,
) -> Result<()> {
    let client = build_client();
    let resp = client.get(url).send().await?;

    if !resp.status().is_success() {
        return Err(anyhow!("Server returned {}", resp.status()));
    }

    let mut file = fs::File::create(save_path).await?;
    let mut resp = resp;

    while let Some(bytes) = resp.chunk().await? {
        if cancelled.load(Ordering::Relaxed) {
            file.flush().await?;
            return Ok(());
        }
        let n = bytes.len() as u64;
        file.write_all(&bytes).await?;
        progress.fetch_add(n, Ordering::Relaxed);
    }

    file.flush().await?;
    Ok(())
}

/// Concatenate chunk_{000..n-1} files from temp_dir into save_path.
pub async fn merge_chunks(
    temp_dir: &Path,
    save_path: &Path,
    chunk_count: usize,
) -> Result<()> {
    let mut out = fs::File::create(save_path).await?;
    for i in 0..chunk_count {
        let part = temp_dir.join(format!("chunk_{:03}", i));
        if part.exists() {
            let data = fs::read(&part).await?;
            out.write_all(&data).await?;
        }
    }
    out.flush().await?;
    Ok(())
}
