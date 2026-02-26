//! gRPC Server for IntegrasiService
//!
//! This module provides the gRPC server that exposes the IntegrasiService
//! for other backend services to consume integration data.

use std::net::SocketAddr;
use std::sync::Arc;

use tokio::signal;
use tokio_postgres::Client;
use tonic::transport::Server;
use tracing::info;

use crate::grpc::proto::integrasi_service_server::IntegrasiServiceServer;
use crate::grpc::service::IntegrasiServiceImpl;

/// Default gRPC server port
pub const DEFAULT_GRPC_PORT: u16 = 50051;

/// gRPC server configuration
#[derive(Debug, Clone)]
pub struct GrpcServerConfig {
    /// Host to bind to (default: 0.0.0.0)
    pub host: String,
    /// Port to listen on (default: 50051)
    pub port: u16,
    /// Maximum message size in bytes (default: 16MB)
    pub max_message_size: usize,
    /// Enable reflection for debugging (default: false in production)
    pub enable_reflection: bool,
}

impl Default for GrpcServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: DEFAULT_GRPC_PORT,
            max_message_size: 16 * 1024 * 1024, // 16MB
            enable_reflection: false,
        }
    }
}

impl GrpcServerConfig {
    /// Create from environment variables
    pub fn from_env() -> Self {
        Self {
            host: std::env::var("GRPC_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("GRPC_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(DEFAULT_GRPC_PORT),
            max_message_size: std::env::var("GRPC_MAX_MESSAGE_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(16 * 1024 * 1024),
            enable_reflection: std::env::var("GRPC_ENABLE_REFLECTION")
                .ok()
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
        }
    }

    /// Get socket address
    pub fn socket_addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .expect("Invalid socket address")
    }
}

/// Start the gRPC server with the given database client
pub async fn start_grpc_server(
    db_client: Arc<Client>,
    config: GrpcServerConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = config.socket_addr();

    info!("Starting gRPC server on {}", addr);

    // Create the service implementation
    let service = IntegrasiServiceImpl::new(db_client);

    // Build the server
    let service = IntegrasiServiceServer::new(service)
        .max_decoding_message_size(config.max_message_size)
        .max_encoding_message_size(config.max_message_size);

    info!("gRPC server listening on {}", addr);

    // Start the server with graceful shutdown
    Server::builder()
        .add_service(service)
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;

    info!("gRPC server shutdown complete");

    Ok(())
}

/// Start the gRPC server with default configuration
pub async fn start_grpc_server_default(
    db_client: Arc<Client>,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = GrpcServerConfig::from_env();
    start_grpc_server(db_client, config).await
}

/// Graceful shutdown signal handler
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, initiating graceful shutdown");
        }
        _ = terminate => {
            info!("Received SIGTERM, initiating graceful shutdown");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = GrpcServerConfig::default();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 50051);
        assert_eq!(config.max_message_size, 16 * 1024 * 1024);
        assert!(!config.enable_reflection);
    }

    #[test]
    fn test_socket_addr() {
        let config = GrpcServerConfig::default();
        let addr = config.socket_addr();
        assert_eq!(addr.port(), 50051);
    }
}
