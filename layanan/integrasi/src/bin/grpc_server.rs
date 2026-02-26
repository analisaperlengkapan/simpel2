//! gRPC Server Binary for layanan-integrasi
//!
//! This binary starts a gRPC server that exposes the IntegrasiService,
//! allowing other backend services (like layanan-perlengkapan) to access
//! MonSAKTI, MySIMKARI, and SIMAN data via gRPC.
//!
//! Usage:
//!   cargo run --bin layanan-integrasi-grpc --features grpc
//!
//! Environment Variables:
//!   DATABASE_URL - PostgreSQL connection string
//!   GRPC_HOST    - Host to bind to (default: 0.0.0.0)
//!   GRPC_PORT    - Port to listen on (default: 50051)

use std::sync::Arc;

use layanan_integrasi::grpc::{server::GrpcServerConfig, start_grpc_server};
use tokio_postgres::NoTls;
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

    info!("=== SIMPEL Layanan Integrasi gRPC Server ===");
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

    // Create gRPC config from environment
    let config = GrpcServerConfig::from_env();
    info!(
        "gRPC server config: {}:{}, max_message_size: {}MB",
        config.host,
        config.port,
        config.max_message_size / 1024 / 1024
    );

    // Start gRPC server
    let db_client = Arc::new(client);
    start_grpc_server(db_client, config).await?;

    info!("=== Server shutdown complete ===");

    Ok(())
}
