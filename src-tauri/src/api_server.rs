//! Local HTTP API on localhost:9876 so a browser extension can send download links.

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};

use crate::downloader::manager::DownloadManager;

#[derive(Debug, Deserialize)]
struct AddRequest {
    url: String,
    filename: Option<String>,
    save_path: Option<String>,
    chunk_count: Option<u8>,
}

#[derive(Debug, Serialize)]
struct AddResponse {
    id: String,
    status: &'static str,
}

#[derive(Debug, Serialize)]
struct StatusResponse {
    active: usize,
    queued: usize,
    downloads: Vec<crate::downloader::task::DownloadItem>,
}

async fn handle_add(
    State(mgr): State<Arc<Mutex<DownloadManager>>>,
    Json(req): Json<AddRequest>,
) -> impl IntoResponse {
    use crate::downloader::chunk::filename_from_url;

    let filename = req
        .filename
        .filter(|f| !f.is_empty())
        .unwrap_or_else(|| filename_from_url(&req.url));

    let save_path = req.save_path.unwrap_or_else(|| {
        if cfg!(target_os = "windows") {
            std::env::var("USERPROFILE")
                .map(|h| format!("{}\\Downloads", h))
                .unwrap_or_else(|_| ".".to_string())
        } else {
            std::env::var("HOME")
                .map(|h| format!("{}/Downloads", h))
                .unwrap_or_else(|_| ".".to_string())
        }
    });

    let chunk_count = req.chunk_count.unwrap_or(8).max(1).min(16);

    // We can't emit Tauri events from here (no AppHandle), so we add directly to the manager.
    // The frontend will pick it up on next poll or via a future WebSocket upgrade.
    let id = uuid::Uuid::new_v4().to_string();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let item = crate::downloader::task::DownloadItem {
        id: id.clone(),
        url: req.url,
        filename,
        save_path,
        total_size: 0,
        downloaded: 0,
        status: crate::downloader::task::DownloadStatus::Queued,
        speed: 0.0,
        eta_seconds: 0,
        created_at: now,
        chunk_count,
        error: None,
        supports_resume: false,
    };

    {
        let mut m = mgr.lock().await;
        m.downloads.insert(id.clone(), item);
        m.queue.push_back(id.clone()); // will be picked up by the manager
    }

    (
        StatusCode::OK,
        Json(AddResponse { id, status: "queued" }),
    )
}

async fn handle_status(
    State(mgr): State<Arc<Mutex<DownloadManager>>>,
) -> impl IntoResponse {
    let m = mgr.lock().await;
    Json(StatusResponse {
        active: m.downloads.values().filter(|i| {
            matches!(i.status, crate::downloader::task::DownloadStatus::Downloading)
        }).count(),
        queued: m.downloads.values().filter(|i| {
            matches!(i.status, crate::downloader::task::DownloadStatus::Queued)
        }).count(),
        downloads: m.get_all(),
    })
}

pub async fn start(mgr: Arc<Mutex<DownloadManager>>) {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/add", post(handle_add))
        .route("/status", get(handle_status))
        .with_state(mgr)
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 9876));
    if let Ok(listener) = tokio::net::TcpListener::bind(addr).await {
        let _ = axum::serve(listener, app).await;
    }
}
