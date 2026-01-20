#![allow(dead_code)]
#![allow(unused_variables)]

mod config;
mod db;
mod handlers;
mod models;

use axum::{
    Router,
    routing::{get, put},
};
use deadpool_postgres::{Config, Runtime};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::AppConfig;
use crate::handlers::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("info"))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = AppConfig::from_env();

    // Setup Database Pool
    let mut cfg = Config::new();
    cfg.url = Some(config.database_url.clone());
    let pool = cfg.create_pool(Some(Runtime::Tokio1), tokio_postgres::NoTls)?;

    // Initialize Database (Schema)
    db::init_db(&pool).await?;

    // Create AppState
    let state = AppState { pool };

    // Define Routes
    let app = Router::new()
        .route("/", get(health_check))
        .route("/api/v1/perencanaan/health", get(health_check))
        .route(
            "/api/v1/perencanaan/pengadaan",
            get(handlers::list_rencana).post(handlers::create_rencana),
        )
        .route(
            "/api/v1/perencanaan/pengadaan/:id",
            put(handlers::update_rencana).delete(handlers::delete_rencana),
        )
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    tracing::info!("Perencanaan service listening on {}", addr);

    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "Perencanaan Service OK"
}
