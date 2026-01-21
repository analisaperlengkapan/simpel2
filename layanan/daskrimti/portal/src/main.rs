//! Unified Portal Backend for Daskrimti
//!
//! Combines Dashboard, Configuration, and Reports into a single microservice.

use axum::{Router, routing::get};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

mod config;
mod error;
mod handlers;
mod models;
mod modules;
mod services;
mod state;

use config::AppConfig;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    info!("Starting Daskrimti Portal Service");

    // Load configuration
    let config = AppConfig::from_env()?;
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;

    // Initialize state
    let state = Arc::new(AppState::new(config).await?);

    // Build router
    let app = build_router(state);

    info!("Portal service listening on {}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn build_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Health endpoints
        .route("/health", get(handlers::health::health_check))
        .route("/health/ready", get(handlers::health::readiness_check))
        .route("/health/live", get(handlers::health::liveness_check))
        // Dashboard module routes
        .nest("/dashboard", modules::dasbor::routes())
        // Configuration module routes
        .nest("/config", modules::konfigurasi::routes())
        // Reports module routes
        .nest("/reports", modules::laporan::routes())
        // Middleware
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}
