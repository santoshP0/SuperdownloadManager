use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DownloadStatus {
    Queued,
    Downloading,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadItem {
    pub id: String,
    pub url: String,
    pub filename: String,
    pub save_path: String,
    pub total_size: u64,
    pub downloaded: u64,
    pub status: DownloadStatus,
    pub speed: f64,
    pub eta_seconds: u64,
    pub created_at: u64,
    pub chunk_count: u8,
    pub error: Option<String>,
    pub supports_resume: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkState {
    pub id: u8,
    pub start: u64,
    pub end: u64,
    pub downloaded: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeState {
    pub item: DownloadItem,
    pub chunks: Vec<ChunkState>,
}
