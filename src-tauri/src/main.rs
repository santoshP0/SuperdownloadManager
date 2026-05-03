// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    super_download_manager_lib::run();
}
