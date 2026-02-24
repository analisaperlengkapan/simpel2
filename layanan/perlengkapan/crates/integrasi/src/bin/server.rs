//! Combined Server Binary for layanan-integrasi
//!
//! This binary starts both:
//! 1. gRPC server for IntegrasiService (for backend-to-backend communication)
//! 2. HTTP server for monitoring dashboard (for admin/ops monitoring)
//!
//! Usage:
//!   cargo run --bin layanan-integrasi-server --features grpc
//!
//! Environment Variables:
//!   DATABASE_URL - PostgreSQL connection string
//!   GRPC_HOST    - gRPC host to bind to (default: 0.0.0.0)
//!   GRPC_PORT    - gRPC port to listen on (default: 50051)
//!   HTTP_HOST    - HTTP host to bind to (default: 0.0.0.0)
//!   HTTP_PORT    - HTTP port to listen on (default: 8080)

use std::sync::Arc;

use axum::Router;
use layanan_perlengkapan_integrasi::{
    grpc::{server::GrpcServerConfig, start_grpc_server},
    monitoring::{MonitoringState, create_monitoring_router},
};
use tokio::net::TcpListener;
use tokio_postgres::NoTls;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,layanan_perlengkapan_integrasi=debug".into()),
        )
        .init();

    info!("=== SIMPEL Layanan Integrasi Server ===");
    info!("Version: {}", env!("CARGO_PKG_VERSION"));

    // Load .env file if exists
    if let Err(e) = dotenvy::dotenv() {
        info!("No .env file found or error loading: {}", e);
    }

    // Get database URL
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set");

    info!("Connecting to database...");

    // Connect to database
    let (client, connection) = tokio_postgres::connect(&database_url, NoTls).await?;

    // Spawn the connection handler
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("Database connection error: {}", e);
        }
    });

    info!("Database connected successfully");

    let db_client = Arc::new(client);

    // Get HTTP server config
    let http_host = std::env::var("HTTP_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let http_port: u16 = std::env::var("HTTP_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    // Create monitoring state
    let monitoring_state = MonitoringState {
        db_client: db_client.clone(),
    };

    // Create monitoring router
    let monitoring_router = create_monitoring_router(monitoring_state);

    // Add CORS middleware for dashboard access
    let app = Router::new().merge(monitoring_router).layer(
        tower_http::cors::CorsLayer::new()
            .allow_origin(tower_http::cors::Any)
            .allow_methods(tower_http::cors::Any)
            .allow_headers(tower_http::cors::Any),
    );

    // Start HTTP server in background
    let http_addr = format!("{}:{}", http_host, http_port);
    info!("Starting HTTP monitoring server on {}", http_addr);

    let listener = TcpListener::bind(&http_addr).await?;
    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            error!("HTTP server error: {}", e);
        }
    });

    info!("HTTP monitoring server started successfully");
    info!("Monitoring endpoints:");
    info!(
        "  - GET  http://{}:{}/api/monitoring/health",
        http_host, http_port
    );
    info!(
        "  - GET  http://{}:{}/api/monitoring/sync-status",
        http_host, http_port
    );
    info!(
        "  - GET  http://{}:{}/api/monitoring/sync-history",
        http_host, http_port
    );
    info!(
        "  - GET  http://{}:{}/api/monitoring/alerts",
        http_host, http_port
    );
    info!(
        "  - GET  http://{}:{}/api/monitoring/dashboard",
        http_host, http_port
    );

    // Create gRPC config from environment
    let grpc_config = GrpcServerConfig::from_env();
    info!(
        "Starting gRPC server on {}:{}, max_message_size: {}MB",
        grpc_config.host,
        grpc_config.port,
        grpc_config.max_message_size / 1024 / 1024
    );

    // Start gRPC server (this blocks until shutdown)
    start_grpc_server(db_client, grpc_config).await?;

    info!("=== Server shutdown complete ===");

    Ok(())
}
