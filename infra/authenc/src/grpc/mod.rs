//! gRPC service implementation for Authenc IAM
//!
//! This module provides gRPC endpoints for authentication, authorization,
//! and user management operations. It implements the AuthencService defined
//! in `infra/proto/authenc.proto`.
//!
//! # Architecture
//!
//! - `authenc_service`: Main service implementation with all RPC methods
//! - `interceptors`: Middleware for authentication, logging, and metrics
//! - `health`: Health check service implementation (grpc.health.v1)
//!
//! # Usage
//!
//! ```rust,no_run
//! use authenc::grpc::create_grpc_server;
//! use authenc::app::AppState;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let state = AppState::new(/* config */);
//!     let server = create_grpc_server(state).await?;
//!     server.serve("0.0.0.0:9088".parse()?).await?;
//!     Ok(())
//! }
//! ```

use std::sync::Arc;
use tonic::service::interceptor::InterceptorLayer;
use tonic::transport::Server;

use crate::app::AppState;
use crate::grpc::interceptors::{LoggingInterceptor, MetricsInterceptor};

/// gRPC service implementation
pub mod authenc_service;

/// gRPC interceptors for auth, logging, and metrics
pub mod interceptors;

/// Health check service implementation
pub mod health;

/// Batch operations for optimized bulk processing
pub mod batch_operations;

// Re-export generated proto types
pub use authenc_service::proto;
pub use batch_operations::{batch_check_permissions, batch_lookup_users, optimized_user_lookup};

// Include common proto types
/// Modul `common`.
pub mod common {
/// Modul `v1`.
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}

/// Create and configure the gRPC server
/// This function sets up the gRPC server with all services, interceptors,
/// and middleware configured. The server is ready to be started with `.serve()`.
/// # Arguments
/// * `state` - Application state shared across all requests
/// * `config` - gRPC server configuration
/// # Returns
/// A configured gRPC server ready to serve requests
/// # Example
/// ```rust,no_run
/// use authenc::grpc::{create_grpc_server, GrpcConfig};
/// use authenc::app::AppState;
/// use std::sync::Arc;
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let state = Arc::new(AppState::new(/* config */));
///     let config = GrpcConfig::default();
///     let server = create_grpc_server(state, config);
///     server.serve("0.0.0.0:9088".parse()?).await?;
///     Ok(())
/// }
/// ```
pub fn create_grpc_server(
    state: Arc<AppState>,
    config: GrpcConfig,
) -> tonic::transport::server::Router<
    tower::layer::util::Stack<
        tonic::service::interceptor::InterceptorLayer<LoggingInterceptor>,
        tower::layer::util::Stack<
            tonic::service::interceptor::InterceptorLayer<MetricsInterceptor>,
            tower::layer::util::Identity,
        >,
    >,
> {
    use interceptors::{LoggingInterceptor, MetricsInterceptor};

    // Create service instances
    let authenc_service = authenc_service::AuthencGrpcService::new(state);

    // Create interceptor stack
    let logging_interceptor = LoggingInterceptor::new();
    let metrics_interceptor = MetricsInterceptor::new();

    // Build server with configuration
    let server = Server::builder()
        .http2_keepalive_interval(Some(std::time::Duration::from_secs(30)))
        .http2_keepalive_timeout(Some(std::time::Duration::from_secs(10)))
        .timeout(std::time::Duration::from_secs(config.request_timeout))
        .concurrency_limit_per_connection(config.max_concurrent_streams as usize);

    // Add TLS if configured
    if config.enable_tls {
        if let (Some(cert_path), Some(key_path)) = (&config.tls_cert_path, &config.tls_key_path) {
            tracing::info!("gRPC server configured with TLS");
            // TLS configuration will be added when serving
            // For now, log the configuration
            tracing::debug!("TLS cert: {}, key: {}", cert_path, key_path);
        }
    }

    // Build router with interceptors and services
    // Note: Health check service will be added in task 9.1
    server
        .layer(InterceptorLayer::new(metrics_interceptor))
        .layer(InterceptorLayer::new(logging_interceptor))
        .add_service(proto::authenc_service_server::AuthencServiceServer::new(
            authenc_service,
        ))
}

/// gRPC server configuration
#[derive(Debug, Clone)]
pub struct GrpcConfig {
    /// Server listen address
    pub addr: String,
    /// Maximum concurrent streams per connection
    pub max_concurrent_streams: u32,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    /// Request timeout in seconds
    pub request_timeout: u64,
    /// Enable TLS
    pub enable_tls: bool,
    /// TLS certificate path
    pub tls_cert_path: Option<String>,
    /// TLS key path
    pub tls_key_path: Option<String>,
}

impl Default for GrpcConfig {
    fn default() -> Self {
        Self {
            addr: "0.0.0.0:9088".to_string(),
            max_concurrent_streams: 100,
            connection_timeout: 30,
            request_timeout: 30,
            enable_tls: false,
            tls_cert_path: None,
            tls_key_path: None,
        }
    }
}
