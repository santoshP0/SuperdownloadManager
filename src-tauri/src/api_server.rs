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
use crate::settings::Settings;

#[derive(Clone)]
struct ApiState {
    mgr:      Arc<Mutex<DownloadManager>>,
    settings: Arc<Mutex<Settings>>,
}

#[derive(Debug, Deserialize)]
struct AddRequest {
    url:         String,
    filename:    Option<String>,
    save_path:   Option<String>,
    chunk_count: Option<u8>,
}

#[derive(Debug, Serialize)]
struct AddResponse {
    id:     String,
    status: &'static str,
}

#[derive(Debug, Serialize)]
struct StatusResponse {
    active:    usize,
    queued:    usize,
    downloads: Vec<crate::downloader::task::DownloadItem>,
}

async fn handle_add(
    State(state): State<ApiState>,
    Json(req): Json<AddRequest>,
) -> impl IntoResponse {
    use crate::downloader::chunk::filename_from_url;

    let filename = req
        .filename
        .filter(|f| !f.is_empty())
        .unwrap_or_else(|| filename_from_url(&req.url));

    // Use user's configured defaults, not hardcoded ones
    let (default_path, default_chunks) = {
        let s = state.settings.lock().await;
        (s.default_save_path.clone(), s.default_chunk_count)
    };

    let save_path   = req.save_path.unwrap_or(default_path);
    let chunk_count = req.chunk_count.unwrap_or(default_chunks).max(1).min(16);

    let id  = uuid::Uuid::new_v4().to_string();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let item = crate::downloader::task::DownloadItem {
        id: id.clone(),
        url: req.url,
        filename,
        save_path,
        total_size:     0,
        downloaded:     0,
        status:         crate::downloader::task::DownloadStatus::Queued,
        speed:          0.0,
        eta_seconds:    0,
        created_at:     now,
        chunk_count,
        error:          None,
        supports_resume: false,
        retry_count:    0,
    };

    {
        let mut m = state.mgr.lock().await;
        m.downloads.insert(id.clone(), item);
        m.queue.push_back(id.clone());
    }

    (StatusCode::OK, Json(AddResponse { id, status: "queued" }))
}

async fn handle_status(State(state): State<ApiState>) -> impl IntoResponse {
    let m = state.mgr.lock().await;
    Json(StatusResponse {
        active: m.downloads.values()
            .filter(|i| matches!(i.status, crate::downloader::task::DownloadStatus::Downloading))
            .count(),
        queued: m.downloads.values()
            .filter(|i| matches!(i.status, crate::downloader::task::DownloadStatus::Queued))
            .count(),
        downloads: m.get_all(),
    })
}

pub async fn start(mgr: Arc<Mutex<DownloadManager>>, settings: Arc<Mutex<Settings>>) {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/add",    post(handle_add))
        .route("/status", get(handle_status))
        .with_state(ApiState { mgr, settings })
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 9876));
    if let Ok(listener) = tokio::net::TcpListener::bind(addr).await {
        let _ = axum::serve(listener, app).await;
    }
}
