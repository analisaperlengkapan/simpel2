mod audit;
mod config;
mod email;
mod error;
mod handlers;
mod models;
mod push;
mod queue;
mod security;
mod template;
mod websocket;
mod whatsapp;

use crate::config::AppConfig;
use crate::security::RateLimitState;
use axum::{Router, http::Method};
use deadpool_postgres::{Config, Runtime};
use prometheus::{Encoder, Registry, TextEncoder};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load config
    let config = AppConfig::from_env();
    // Logging & tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("info"))
        .with(tracing_subscriber::fmt::layer())
        .init();
    // Sentry (opsional)
    let _guard: Option<()> = None;
    // DB pool
    let mut cfg = Config::new();
    cfg.url = Some(config.database_url.clone());
    let pool = cfg.create_pool(Some(Runtime::Tokio1), tokio_postgres::NoTls)?;
    // Redis
    let _redis = redis::Client::open(config.redis_url.clone())?;
    // Prometheus registry
    let registry = Registry::new();
    // CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::header::HeaderName::from_static("x-api-key"),
        ]);
    // Rate limit state
    let rate_limit_state: RateLimitState = Arc::new(Mutex::new(HashMap::new()));
    // WebSocket state
    let ws_state = websocket::WsState::new(pool.clone());
    // Router
    let app = handlers::routes(
        config.clone(),
        pool.clone(),
        rate_limit_state.clone(),
        ws_state.clone(),
    )
    .layer(cors)
    .layer(TraceLayer::new_for_http());
    // Health & metrics
    let metrics_route = Router::new().route(
        "/metrics",
        axum::routing::get(|| async move {
            let encoder = TextEncoder::new();
            let mut buffer = Vec::new();
            let mf = registry.gather();
            encoder.encode(&mf, &mut buffer).unwrap();
            String::from_utf8(buffer).unwrap()
        }),
    );
    let app = app.merge(metrics_route);
    // Startup log
    tracing::info!(
        "Notifikasi service listening on {}:{}",
        config.server_host,
        config.server_port
    );
    // Run server
    let addr = SocketAddr::new(config.server_host.parse()?, config.server_port);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    use tokio::signal;
    let ctrl_c = signal::ctrl_c();
    #[cfg(unix)]
    let mut terminate_signal = signal::unix::signal(signal::unix::SignalKind::terminate()).unwrap();
    #[cfg(unix)]
    let terminate = terminate_signal.recv();
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::warn!("Shutdown signal received, shutting down gracefully...");
}
