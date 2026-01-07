use axum::{Router, routing::{get, post}};
use deadpool_postgres::{Config, Runtime};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

mod api;
mod models;
mod repository;
mod services;

use crate::repository::PostgresRepository;
use crate::services::PidmilService;
use crate::api::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let _ = dotenvy::dotenv();

    // Setup DB Pool
    let mut cfg = Config::new();
    cfg.user = std::env::var("PG_USER").ok();
    cfg.password = std::env::var("PG_PASSWORD").ok();
    cfg.dbname = std::env::var("PG_DBNAME").ok();
    cfg.host = std::env::var("PG_HOST").ok();
    cfg.port = std::env::var("PG_PORT").ok().and_then(|p| p.parse().ok());

    // Fallback for development
    if cfg.host.is_none() {
         cfg.host = Some("localhost".to_string());
    }
    if cfg.dbname.is_none() {
         cfg.dbname = Some("simpel_pidmil".to_string());
    }

    let pool = cfg.create_pool(Some(Runtime::Tokio1), tokio_postgres::NoTls).unwrap();

    // Run Migrations (Embedded)
    let migration_sql = include_str!("../migrations.sql");
    println!("Running embedded migrations...");
    let client = pool.get().await.expect("Failed to connect to DB for migrations");
    if let Err(e) = client.batch_execute(migration_sql).await {
        eprintln!("Migration failed: {}", e);
    } else {
         println!("Migrations applied successfully.");
    }

    let repo = PostgresRepository::new(pool);
    let service = PidmilService::new(repo);
    let state = AppState { service };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/pidmil/health", get(health_check))
        .route("/api/v1/pidmil/status", get(status))
        // Case Routes
        .route("/api/v1/pidmil/cases", get(api::get_cases).post(api::create_case))
        .route("/api/v1/pidmil/cases/:id", get(api::get_case_by_id))
        // Suspect Routes
        .route("/api/v1/pidmil/cases/:id/suspects", get(api::get_suspects).post(api::create_suspect))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8086));
    println!("pidmil service listening on {}", addr);

    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "pidmil Service OK"
}

async fn status() -> &'static str {
    "pidmil service is running"
}
