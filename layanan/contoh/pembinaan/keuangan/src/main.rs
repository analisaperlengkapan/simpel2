mod auth;
mod config;
mod database;
mod error;
mod handlers;
mod models;

use crate::config::AppConfig;
use axum::{Router, routing::get};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
#[tokio::main]
async fn main() {
    // Shared Telemetry
    lib_common::telemetry::init_subscriber("info");

    // Configuration
    let config = AppConfig::from_env();

    // Standardized Database Pool
    let db_config = lib_common::db::DbConfig {
        url: config.database_url.clone(),
        max_size: 10,
        min_idle: None,
        connection_timeout: None,
        idle_timeout: None,
        max_lifetime: None,
    };
    let pool =
        lib_common::db::create_postgres_pool(db_config).expect("Failed to create database pool");

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

async fn health_check() -> impl axum::response::IntoResponse {
    let mut report = lib_common::health::HealthReport::new("keuangan", env!("CARGO_PKG_VERSION"));

    // In a real scenario, we'd check DB health here
    report.add_check(lib_common::health::ComponentCheck {
        component: "database".to_string(),
        status: lib_common::health::ServiceStatus::Healthy,
        message: None,
    });

    axum::Json(report)
}
