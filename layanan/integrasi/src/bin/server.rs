//! Combined Server Binary for layanan-integrasi
//!
//! This binary starts both:
//! 1. gRPC server for IntegrasiService (for backend-to-backend communication)
//! 2. HTTP server with one health endpoint (`monitoring.rs` says why there is nothing else)
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

use axum::Router;
use layanan_integrasi::{
    Config,
    db_handle::DbHandle,
    grpc::{server::GrpcServerConfig, start_grpc_server},
    monitoring::{MonitoringState, create_monitoring_router},
    scheduler::IntegrationScheduler,
};
use tokio::net::TcpListener;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,layanan_integrasi=debug".into()),
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

    // Connect to database. A handle, not a bare client: a bare client dies with its connection and
    // never recovers (see `db_handle`), which left this server unhealthy from the first idle hour.
    let db_client = DbHandle::connect(&database_url).await?;

    info!("Database connected successfully");

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

    // No CORS layer: no browser page calls this port. It used to allow any origin, for a
    // dashboard that was never written.
    let app = Router::new().merge(monitoring_router);

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
    info!(
        "  - GET  http://{}:{}/api/monitoring/health",
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

    // Start scheduler daemon if enabled
    let integrasi_config = Config::from_env()?;
    if integrasi_config.scheduler_enabled {
        info!("Starting Integration Scheduler daemon...");
        let mut scheduler = IntegrationScheduler::new(integrasi_config).await?;
        scheduler.start().await?;
        info!("Integration Scheduler started.");
    } else {
        info!("Integration Scheduler is disabled in configuration.");
    }

    // Start gRPC server (this blocks until shutdown)
    start_grpc_server(db_client, grpc_config).await?;

    info!("=== Server shutdown complete ===");

    Ok(())
}
