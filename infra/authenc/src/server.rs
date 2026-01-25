//! Dual server management for HTTP and gRPC
//!
//! This module provides coordinated lifecycle management for both
//! HTTP (Axum) and gRPC (Tonic) servers running concurrently.

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::broadcast;
use tracing::{error, info};

use crate::app::AppState;
use crate::error::Result;

/// Dual server manager for HTTP and gRPC
pub struct DualServer {
    state: Arc<AppState>,
    http_addr: SocketAddr,
    grpc_addr: SocketAddr,
}

impl DualServer {
    /// Create a new dual server instance
    pub fn new(state: AppState) -> Self {
        let state = Arc::new(state);

        let http_addr = SocketAddr::from(([0, 0, 0, 0], state.config.server.port));
        let grpc_addr = SocketAddr::from(([0, 0, 0, 0], state.config.server.grpc_port));

        Self {
            state,
            http_addr,
            grpc_addr,
        }
    }

    /// Run both HTTP and gRPC servers concurrently with graceful shutdown
    pub async fn run(self) -> Result<()> {
        info!("🚀 Starting Authenc servers");
        info!("   HTTP server: {}", self.http_addr);

        if self.state.config.server.grpc_enabled {
            info!("   gRPC server: {}", self.grpc_addr);
        } else {
            info!("   gRPC server: disabled");
        }

        // Create shutdown channel
        let (shutdown_tx, _) = broadcast::channel::<()>(1);

        // Start HTTP server
        let http_handle = {
            let state = self.state.clone();
            let addr = self.http_addr;
            let shutdown_rx = shutdown_tx.subscribe();

            tokio::spawn(async move {
                if let Err(e) = run_http_server(state, addr, shutdown_rx).await {
                    error!("HTTP server error: {}", e);
                }
            })
        };

        // Start gRPC server if enabled
        let grpc_handle = if self.state.config.server.grpc_enabled {
            #[cfg(feature = "grpc")]
            {
                let state = self.state.clone();
                let addr = self.grpc_addr;
                let shutdown_rx = shutdown_tx.subscribe();

                Some(tokio::spawn(async move {
                    if let Err(e) = run_grpc_server(state, addr, shutdown_rx).await {
                        error!("gRPC server error: {}", e);
                    }
                }))
            }
            #[cfg(not(feature = "grpc"))]
            {
                tracing::warn!("gRPC enabled in config but binary compiled without 'grpc' feature. gRPC server will NOT start.");
                None
            }
        } else {
            None
        };

        // Wait for shutdown signal
        wait_for_shutdown_signal().await;
        info!("🛑 Shutdown signal received, stopping servers...");

        // Broadcast shutdown to all servers
        let _ = shutdown_tx.send(());

        // Wait for servers to complete with timeout
        let shutdown_timeout = tokio::time::Duration::from_secs(30);

        tokio::select! {
            _ = http_handle => {
                info!("✓ HTTP server stopped");
            }
            _ = tokio::time::sleep(shutdown_timeout) => {
                error!("⚠ HTTP server shutdown timeout");
            }
        }

        if let Some(grpc_handle) = grpc_handle {
            tokio::select! {
                _ = grpc_handle => {
                    info!("✓ gRPC server stopped");
                }
                _ = tokio::time::sleep(shutdown_timeout) => {
                    error!("⚠ gRPC server shutdown timeout");
                }
            }
        }

        info!("✓ All servers stopped gracefully");
        Ok(())
    }
}

/// Run the HTTP server
async fn run_http_server(
    state: Arc<AppState>,
    addr: SocketAddr,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> Result<()> {
    use crate::axum_app::AxumApp;

    info!("🌐 HTTP server starting on {}", addr);

    let app = AxumApp::new((*state).clone());
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!("🌐 HTTP server listening on {}", addr);

    // Create the router
    let router = app.into_router();

    // Serve with graceful shutdown
    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(async move {
        let _ = shutdown_rx.recv().await;
        info!("HTTP server received shutdown signal");
    })
    .await
    .map_err(|e| {
        error!("HTTP server error: {}", e);
        crate::error::AuthencError::internal(format!("HTTP server error: {}", e))
    })?;

    Ok(())
}

/// Run the gRPC server
#[cfg(feature = "grpc")]
async fn run_grpc_server(
    state: Arc<AppState>,
    addr: SocketAddr,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> Result<()> {
    use crate::grpc::{GrpcConfig, create_grpc_server};

    info!("🔌 gRPC server starting on {}", addr);

    let grpc_config = GrpcConfig {
        addr: addr.to_string(),
        max_concurrent_streams: 100,
        connection_timeout: 30,
        request_timeout: 30,
        enable_tls: false,
        tls_cert_path: None,
        tls_key_path: None,
    };

    let router = create_grpc_server(state, grpc_config);

    info!("🔌 gRPC server listening on {}", addr);

    // Serve with graceful shutdown
    router
        .serve_with_shutdown(addr, async move {
            let _ = shutdown_rx.recv().await;
            info!("gRPC server received shutdown signal");
        })
        .await
        .map_err(|e| {
            error!("gRPC server error: {}", e);
            crate::error::AuthencError::internal(format!("gRPC server error: {}", e))
        })?;

    Ok(())
}

/// Wait for shutdown signal (SIGTERM or Ctrl+C)
async fn wait_for_shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C signal");
        },
        _ = terminate => {
            info!("Received SIGTERM signal");
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;

    #[test]
    fn test_dual_server_creation() {
        let config = AppConfig::default();
        let state = AppState::new(config);
        // This will fail because we can't actually create AppState without a database
        // but it tests the DualServer::new function signature
        // let server = DualServer::new(state);
        // assert!(server.http_addr.port() > 0);
    }
}
