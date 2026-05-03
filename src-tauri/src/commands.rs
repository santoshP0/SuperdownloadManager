use tauri::{AppHandle, State};

use crate::downloader::manager;
use crate::downloader::task::DownloadItem;
use crate::state::AppState;

fn map_err(e: anyhow::Error) -> String {
    e.to_string()
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
    manager::add_download(
        state.manager.clone(),
        url,
        filename,
        save_path,
        chunk_count,
        app,
    )
    .await
    .map_err(map_err)
}

#[tauri::command]
pub async fn pause_download(id: String, state: State<'_, AppState>) -> Result<(), String> {
    manager::pause_download(state.manager.clone(), &id)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn resume_download(
    id: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    manager::resume_download(state.manager.clone(), &id, app)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn cancel_download(id: String, state: State<'_, AppState>) -> Result<(), String> {
    manager::cancel_download(state.manager.clone(), &id)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn get_downloads(state: State<'_, AppState>) -> Result<Vec<DownloadItem>, String> {
    let m = state.manager.lock().await;
    Ok(m.get_all())
}

#[tauri::command]
pub async fn set_max_concurrent(n: usize, state: State<'_, AppState>) -> Result<(), String> {
    let mut m = state.manager.lock().await;
    m.max_concurrent = n.max(1).min(10);
    Ok(())
}

#[tauri::command]
pub fn get_default_download_dir() -> String {
    // On Windows: %USERPROFILE%\Downloads; fallback to home dir
    if let Some(home) = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")) {
        let p = std::path::PathBuf::from(home).join("Downloads");
        if p.exists() {
            return p.to_string_lossy().to_string();
        }
    }
    std::env::current_dir()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

#[tauri::command]
pub async fn open_file(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn open_folder(path: String) -> Result<(), String> {
    let folder = std::path::Path::new(&path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or(path);

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&folder)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&folder)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&folder)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
