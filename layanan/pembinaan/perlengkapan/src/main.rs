//! # Layanan Pembinaan Perlengkapan Backend Service
//!
//! Backend microservice for Perlengkapan (asset management) within SIMPelv2.
//! Integrates with Authenc (IAM) and Secreton (Secret Manager) via gRPC.

use axum::{Router, routing::get};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

mod database;
mod errors;
mod handlers;
mod middleware;
mod models;
mod routes;
mod services;

use database::Database;
use services::PerlengkapanService;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,layanan_perlengkapan=debug".into()),
        )
        .init();

    info!("Starting Layanan Pembinaan Perlengkapan Service");

    // Load configuration from environment
    let host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("SERVER_PORT")
        .unwrap_or_else(|_| "3020".to_string())
        .parse::<u16>()
        .expect("SERVER_PORT must be a valid port number");
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set");

    // Initialize database connection
    info!("Connecting to database...");
    let db = Database::new(&database_url).await?;

    // Run migrations
    info!("Running database migrations...");
    db.migrate().await?;

    // Create service
    let service = PerlengkapanService::new(db);

    // Build router
    let app = build_router(service);

    // Start server
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    info!("Perlengkapan service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn build_router(service: PerlengkapanService) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Health check routes (no auth required)
    let health_routes = Router::new()
        .route("/health", get(health_check))
        .route("/health/ready", get(readiness_check))
        .route("/health/live", get(liveness_check));

    // API routes with authentication
    let api_routes = routes::create_routes(service);

    // Combine all routes
    Router::new()
        .merge(health_routes)
        .nest("/api", api_routes)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}

// Health check handlers
async fn health_check() -> &'static str {
    "OK"
}

async fn readiness_check() -> &'static str {
    "Ready"
}

async fn liveness_check() -> &'static str {
    "Alive"
}
