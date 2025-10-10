//! Local HTTP server for web app communication

use crate::protocol::{ApiResponse, DaemonStatus};
use crate::state::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

/// Start the local HTTP server
pub async fn start_server(port: u16, state: AppState) -> anyhow::Result<()> {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/active", post(handle_active))
        .route("/inactive", post(handle_inactive))
        .route("/heartbeat", post(handle_heartbeat))
        .route("/status", get(handle_status))
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::info!("Starting HTTP server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to bind to port {}. Is another instance running? Error: {}",
                port,
                e
            )
        })?;

    axum::serve(listener, app)
        .await
        .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;

    Ok(())
}

/// Handle POST /active - Web app is active
async fn handle_active(State(state): State<AppState>) -> Json<ApiResponse> {
    state.set_active().await;
    Json(ApiResponse::ok())
}

/// Handle POST /inactive - Web app is inactive
async fn handle_inactive(State(state): State<AppState>) -> Json<ApiResponse> {
    state.set_inactive().await;
    Json(ApiResponse::ok())
}

/// Handle POST /heartbeat - Keep-alive from web app
async fn handle_heartbeat(State(state): State<AppState>) -> Json<ApiResponse> {
    state.update_heartbeat().await;
    Json(ApiResponse::ok())
}

/// Handle GET /status - Health check
async fn handle_status(State(state): State<AppState>) -> (StatusCode, Json<DaemonStatus>) {
    let status = DaemonStatus {
        daemon: "running".to_string(),
        server_connected: state.is_server_connected().await,
        web_app_active: state.is_web_app_active().await,
    };

    (StatusCode::OK, Json(status))
}
