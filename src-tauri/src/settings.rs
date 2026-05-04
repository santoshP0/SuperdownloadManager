use serde::{Deserialize, Serialize};

use crate::persistence::data_dir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub default_save_path: String,
    pub default_chunk_count: u8,
    pub max_concurrent: usize,
    /// 0 = unlimited, otherwise KB/s
    pub speed_limit_kbps: u32,
    pub clipboard_monitor: bool,
    pub notifications: bool,
    pub sound_on_completion: bool,
    /// Auto-sort downloads into Videos / Music / Documents / … subfolders
    pub category_folders: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_save_path: default_save_path(),
            default_chunk_count: 8,
            max_concurrent: 3,
            speed_limit_kbps: 0,
            clipboard_monitor: true,
            notifications: true,
            sound_on_completion: true,
            category_folders: false,
        }
    }
}

fn default_save_path() -> String {
    std::env::var("USERPROFILE")
        .map(|h| format!("{}\\Downloads", h))
        .or_else(|_| std::env::var("HOME").map(|h| format!("{}/Downloads", h)))
        .unwrap_or_else(|_| ".".to_string())
}

pub fn load() -> Settings {
    let path = data_dir().join("settings.json");
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(s: &Settings) {
    let path = data_dir().join("settings.json");
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(s) {
        let _ = std::fs::write(path, json);
    }
}
