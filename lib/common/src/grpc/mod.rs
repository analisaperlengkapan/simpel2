//! gRPC Common Utilities
//!
//! Provides reusable gRPC interceptors, TLS configuration, and health check types.
//!
//! # Modules
//!
//! - `interceptors` - Auth, Logging, Metrics, RateLimit interceptors
//! - `tls` - TLS/mTLS configuration
//! - `health` - Health check types

#[cfg(feature = "grpc")]
pub mod interceptors;

#[cfg(feature = "grpc")]
pub mod tls;

#[cfg(feature = "grpc")]
pub mod health;

#[cfg(feature = "grpc")]
pub use interceptors::{
    AuthInterceptor, LoggingInterceptor, MetricsInterceptor, RateLimitInterceptor,
    extract_bearer_token,
};

#[cfg(feature = "grpc")]
pub use tls::{GrpcTlsConfig, TlsIdentity, GrpcTlsMetrics};

#[cfg(feature = "grpc")]
pub use health::{HealthStatus, ServingStatus, DependencyHealth, HealthInfo};
