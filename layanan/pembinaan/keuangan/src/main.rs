mod auth;
mod config;
mod database;
mod error;
mod handlers;
mod models;

use crate::config::AppConfig;
use axum::{Router, routing::get};
use deadpool_postgres::{Config, Runtime};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("info"))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Configuration
    let config = AppConfig::from_env();

    // Database Pool
    let mut db_cfg = Config::new();
    db_cfg.url = Some(config.database_url.clone());
    let pool = db_cfg
        .create_pool(Some(Runtime::Tokio1), tokio_postgres::NoTls)
        .expect("Failed to create pool");

    // Initialize Database
    if let Err(e) = database::init_db(&pool).await {
        tracing::error!("Failed to initialize database: {}", e);
    }

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(health_check))
        .route("/api/v1/keuangan/health", get(health_check))
        .route("/api/v1/keuangan/metrics", get(handlers::get_metrics))
        .route(
            "/api/v1/keuangan/budgets",
            get(handlers::list_budgets).post(handlers::create_budget),
        )
        .route(
            "/api/v1/keuangan/transactions",
            get(handlers::list_transactions).post(handlers::create_transaction),
        )
        .layer(cors)
        .with_state(pool);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    tracing::info!("Keuangan service listening on {}", addr);

    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "Keuangan Service OK"
}
