use std::sync::Arc;
use tokio::sync::Mutex;

use crate::downloader::manager::DownloadManager;

pub struct AppState {
    pub manager: Arc<Mutex<DownloadManager>>,
}
