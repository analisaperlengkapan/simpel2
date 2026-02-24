//! Health check endpoint

use anyhow::Result;
use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::get};
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
struct AppState {
    token: Arc<RwLock<Option<String>>>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    token_valid: bool,
}

/// Start health server
pub async fn start_server(
    port: u16,
    token: Arc<RwLock<Option<String>>>,
    mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let app_state = AppState { token };

    let app = Router::new()
        .route("/healthz", get(health_handler))
        .route("/health", get(health_handler))
        .with_state(app_state);

    tracing::info!("Health server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            let _ = shutdown_rx.recv().await;
        })
        .await?;

    Ok(())
}

async fn health_handler(State(state): State<AppState>) -> impl IntoResponse {
    let token_valid = state.token.read().await.is_some();

    let response = HealthResponse {
        status: if token_valid { "ok" } else { "no_token" }.to_string(),
        token_valid,
    };

    (StatusCode::OK, Json(response))
}
