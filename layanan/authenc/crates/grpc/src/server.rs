//! gRPC server setup and configuration

use crate::AuthencServiceServer;
use crate::error::GrpcResult;
use crate::service::AuthencGrpcService;
use crate::tls::TlsConfig;
use std::net::SocketAddr;
use tonic::transport::Server;
use tracing::info;

/// gRPC server configuration
#[derive(Debug, Clone)]
pub struct GrpcServerConfig {
    /// Server bind address
    pub bind_address: SocketAddr,
    /// TLS configuration (optional, but recommended for production)
    pub tls_config: Option<TlsConfig>,
    /// Enable request logging
    pub enable_logging: bool,
    /// Enable authentication interceptor
    pub enable_auth: bool,
}

impl Default for GrpcServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0:50051".parse().unwrap(),
            tls_config: None,
            enable_logging: true,
            enable_auth: true,
        }
    }
}

/// gRPC server builder
pub struct GrpcServerBuilder {
    config: GrpcServerConfig,
    service: Option<AuthencGrpcService>,
}

impl GrpcServerBuilder {
    /// Create new server builder
    pub fn new(config: GrpcServerConfig) -> Self {
        Self {
            config,
            service: None,
        }
    }

    /// Set the gRPC service implementation
    pub fn with_service(mut self, service: AuthencGrpcService) -> Self {
        self.service = Some(service);
        self
    }

    /// Build and start the gRPC server
    pub async fn serve(self) -> GrpcResult<()> {
        let service = self
            .service
            .expect("Service must be set before calling serve");

        info!("Starting gRPC server on {}", self.config.bind_address);

        // Create server builder
        let mut server = Server::builder();

        // Configure TLS if provided
        if let Some(tls_config) = &self.config.tls_config {
            info!("Configuring mTLS for gRPC server");
            let tls = tls_config.build_server_config().map_err(|e| {
                crate::error::GrpcError::Internal(format!("TLS config error: {}", e))
            })?;
            server = Server::builder().tls_config(tls).map_err(|e| {
                crate::error::GrpcError::Internal(format!("Failed to set TLS config: {}", e))
            })?;
        } else {
            tracing::warn!("Running gRPC server without TLS - not recommended for production!");
        }

        // Add service with interceptors
        let service_with_interceptors = AuthencServiceServer::new(service);

        // TODO: Add interceptors when tonic supports them properly
        // For now, interceptors are applied in the service layer

        // Start server
        server
            .add_service(service_with_interceptors)
            .serve(self.config.bind_address)
            .await
            .map_err(|e| crate::error::GrpcError::Internal(format!("Server error: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = GrpcServerConfig::default();
        assert_eq!(config.bind_address.port(), 50051);
        assert!(config.enable_logging);
        assert!(config.enable_auth);
    }

    #[test]
    fn test_server_builder() {
        let config = GrpcServerConfig::default();
        let _builder = GrpcServerBuilder::new(config);
        // Builder created successfully
    }
}
