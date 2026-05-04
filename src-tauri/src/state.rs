use std::sync::Arc;
use tokio::sync::Mutex;

use crate::downloader::manager::DownloadManager;
use crate::settings::Settings;

pub struct AppState {
    pub manager: Arc<Mutex<DownloadManager>>,
    pub settings: Arc<Mutex<Settings>>,
}
