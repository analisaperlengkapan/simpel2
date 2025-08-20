mod config;
mod error;
mod models;
mod storage;
mod security;
mod audit;
mod ocr;
mod classify;
mod archive;
mod handlers;

use crate::config::AppConfig;
use axum::{Router, http::Method};
use sqlx::postgres::PgPoolOptions;
use tower_http::{cors::{CorsLayer, Any}, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::net::SocketAddr;
use std::sync::Arc;
use prometheus::{Encoder, TextEncoder, Registry};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load config
    let config = AppConfig::from_env();
    // Logging & tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&config.log_level))
        .with(tracing_subscriber::fmt::layer())
        .init();
    // Sentry (opsional)
    let _guard = if let Some(ref dsn) = config.sentry_dsn {
        Some(sentry::init((dsn.as_str(), sentry::ClientOptions { release: sentry::release_name!(), ..Default::default() })))
    } else { None };
    // DB pool
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url).await?;
    // Prometheus registry
    let registry = Registry::new();
    // CORS
    let cors = CorsLayer::new()
        .allow_origin(config.cors_origins.iter().map(|s| s.parse().unwrap_or(Any)).collect::<Vec<_>>())
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([axum::http::header::CONTENT_TYPE, axum::http::header::AUTHORIZATION]);
    // Router
    let app = handlers::routes(config.clone(), pool.clone())
        .layer(cors)
        .layer(TraceLayer::new_for_http());
    // Health & metrics
    let metrics_route = Router::new().route("/metrics", axum::routing::get(|| async move {
        let encoder = TextEncoder::new();
        let mut buffer = Vec::new();
        let mf = registry.gather();
        encoder.encode(&mf, &mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    }));
    let app = app.merge(metrics_route);
    // Static file preview (opsional)
    let app = app.nest("/preview", ServeDir::new("./storage"));
    // Startup log
    tracing::info!("Dokumen service listening on {}:{}", config.server_host, config.server_port);
    // Run server
    let addr = SocketAddr::new(config.server_host.parse()?, config.server_port);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    use tokio::signal;
    let ctrl_c = signal::ctrl_c();
    #[cfg(unix)]
    let terminate = signal::unix::signal(signal::unix::SignalKind::terminate()).unwrap().recv();
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::warn!("Shutdown signal received, shutting down gracefully...");
} 