use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::downloader::task::{DownloadItem, DownloadStatus};

pub fn data_dir() -> PathBuf {
    // Windows: %APPDATA%\com.superdownload.manager
    // Linux/Mac: ~/.config/com.superdownload.manager
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config"))
        })
        .unwrap_or_else(|| PathBuf::from("."));

    base.join("com.superdownload.manager")
}

fn history_path() -> PathBuf {
    data_dir().join("history.json")
}

pub fn save_history(downloads: &HashMap<String, DownloadItem>) {
    let path = history_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(downloads) {
        let _ = fs::write(&path, json);
    }
}

pub fn load_history() -> HashMap<String, DownloadItem> {
    let path = history_path();
    let Ok(data) = fs::read_to_string(&path) else {
        return HashMap::new();
    };
    let Ok(mut map) = serde_json::from_str::<HashMap<String, DownloadItem>>(&data) else {
        return HashMap::new();
    };

    // Items that were mid-flight when the app closed → reset to Paused
    for item in map.values_mut() {
        if matches!(
            item.status,
            DownloadStatus::Downloading | DownloadStatus::Retrying | DownloadStatus::Queued
        ) {
            item.status = DownloadStatus::Paused;
            item.speed = 0.0;
            item.eta_seconds = 0;
        }
    }

    map
}
