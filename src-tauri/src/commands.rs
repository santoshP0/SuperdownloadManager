use std::path::PathBuf;
use tauri::{AppHandle, State};

use crate::downloader::manager;
use crate::downloader::task::DownloadItem;
use crate::settings::Settings;
use crate::state::AppState;

fn map_err(e: anyhow::Error) -> String {
    e.to_string()
}

fn file_category(filename: &str) -> &'static str {
    let ext = std::path::Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match ext.as_str() {
        "mp4" | "mkv" | "avi" | "mov" | "wmv" | "webm" | "flv" | "m4v" => "Videos",
        "mp3" | "flac" | "wav" | "aac" | "ogg" | "m4a" | "opus"        => "Music",
        "pdf" | "epub" | "mobi" | "doc" | "docx" | "xls" | "xlsx"
        | "ppt" | "pptx" | "txt"                                        => "Documents",
        "exe" | "msi" | "dmg" | "pkg" | "deb" | "rpm" | "apk"          => "Programs",
        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "zst"    => "Archives",
        "iso" | "img"                                                    => "Disk Images",
        _                                                                => "Other",
    }
}

#[tauri::command]
pub async fn add_download(
    url: String,
    filename: String,
    save_path: String,
    chunk_count: u8,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<String, String> {
    let chunk_count = chunk_count.max(1).min(16);

    let effective_path = {
        let s = state.settings.lock().await;
        if s.category_folders {
            PathBuf::from(&save_path)
                .join(file_category(&filename))
                .to_string_lossy()
                .to_string()
        } else {
            save_path
        }
    };

    manager::add_download(state.manager.clone(), url, filename, effective_path, chunk_count, app)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn pause_download(id: String, state: State<'_, AppState>) -> Result<(), String> {
    manager::pause_download(state.manager.clone(), &id).await.map_err(map_err)
}

#[tauri::command]
pub async fn resume_download(
    id: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    manager::resume_download(state.manager.clone(), &id, app).await.map_err(map_err)
}

#[tauri::command]
pub async fn cancel_download(id: String, state: State<'_, AppState>) -> Result<(), String> {
    manager::cancel_download(state.manager.clone(), &id).await.map_err(map_err)
}

#[tauri::command]
pub async fn get_downloads(state: State<'_, AppState>) -> Result<Vec<DownloadItem>, String> {
    Ok(state.manager.lock().await.get_all())
}

#[tauri::command]
pub async fn set_max_concurrent(n: usize, state: State<'_, AppState>) -> Result<(), String> {
    state.manager.lock().await.max_concurrent = n.max(1).min(10);
    Ok(())
}

#[tauri::command]
pub async fn get_default_download_dir(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.settings.lock().await.default_save_path.clone())
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    Ok(state.settings.lock().await.clone())
}

#[tauri::command]
pub async fn save_settings(
    new_settings: Settings,
    state: State<'_, AppState>,
) -> Result<(), String> {
    crate::settings::save(&new_settings);
    state.manager.lock().await.apply_settings(&new_settings);
    *state.settings.lock().await = new_settings;
    Ok(())
}

#[tauri::command]
pub async fn open_file(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    std::process::Command::new("explorer").arg(&path).spawn().map_err(|e| e.to_string())?;
    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open").arg(&path).spawn().map_err(|e| e.to_string())?;
    #[cfg(target_os = "macos")]
    std::process::Command::new("open").arg(&path).spawn().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn open_folder(path: String) -> Result<(), String> {
    let folder = std::path::Path::new(&path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or(path);
    #[cfg(target_os = "windows")]
    std::process::Command::new("explorer").arg(&folder).spawn().map_err(|e| e.to_string())?;
    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open").arg(&folder).spawn().map_err(|e| e.to_string())?;
    #[cfg(target_os = "macos")]
    std::process::Command::new("open").arg(&folder).spawn().map_err(|e| e.to_string())?;
    Ok(())
}
