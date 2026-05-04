pub mod api_server;
pub mod commands;
pub mod downloader;
pub mod persistence;
pub mod settings;
pub mod state;
pub mod tray;

use downloader::manager::DownloadManager;
use state::AppState;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let s = settings::load();

            let mut mgr = DownloadManager::new();
            mgr.apply_settings(&s);
            let manager = Arc::new(Mutex::new(mgr));
            let settings = Arc::new(Mutex::new(s));

            tokio::spawn(api_server::start(manager.clone()));
            tray::setup(app)?;

            app.manage(AppState { manager, settings });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::add_download,
            commands::pause_download,
            commands::resume_download,
            commands::cancel_download,
            commands::get_downloads,
            commands::set_max_concurrent,
            commands::get_default_download_dir,
            commands::open_file,
            commands::open_folder,
            commands::get_settings,
            commands::save_settings,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
