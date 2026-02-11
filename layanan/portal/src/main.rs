//! Unified Portal Backend for Daskrimti
//!
//! Combines Dashboard, Configuration, and Reports into a single microservice.
//! Integrates with Authenc (IAM) and Secreton (Secret Manager) via gRPC.

use axum::{Router, middleware as axum_middleware, routing::get};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

mod config;
mod error;
mod handlers;
mod middleware;
mod models;
mod modules;
mod proto;
mod services;
mod state;

use config::AppConfig;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    info!("Starting Daskrimti Portal Service");

    // Initialize server start time for uptime tracking
    handlers::dashboard::init_server_start_time();

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

    // Protected routes that require authentication
    let protected_routes = Router::new()
        .nest("/dashboard", modules::dasbor::routes())
        .nest("/config", modules::konfigurasi::routes())
        .nest("/reports", modules::laporan::routes())
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::auth_middleware,
        ));

    // Combine all routes into one router
    Router::new()
        // Health endpoints (no auth)
        .route("/health", get(handlers::health::health_check))
        .route("/health/ready", get(handlers::health::readiness_check))
        .route("/health/live", get(handlers::health::liveness_check))
        // Auth endpoints (no auth)
        .nest("/api/auth", handlers::auth::routes())
        // CAPTCHA endpoints (no auth)
        .nest("/api/captcha", handlers::captcha::routes())
        // Merge in protected routes
        .merge(protected_routes)
        // Global middleware
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}
