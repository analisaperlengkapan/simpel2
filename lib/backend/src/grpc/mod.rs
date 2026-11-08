//! gRPC Common Utilities
//!
//! Provides reusable gRPC interceptors, TLS configuration, and health check types.
//!
//! # Modules
//!
//! - `interceptors` - Auth, Logging, Metrics, RateLimit interceptors
//! - `tls` - TLS/mTLS configuration (server side)
//! - `client` - mTLS for gRPC clients (env-driven, fail-closed)
//! - `health` - Health check types

#[cfg(feature = "grpc")]
pub mod interceptors;

#[cfg(feature = "grpc")]
pub mod tls;

#[cfg(feature = "grpc")]
pub mod client;

#[cfg(feature = "grpc")]
pub mod health;

#[cfg(feature = "grpc")]
pub use interceptors::{
    AuthInterceptor, LoggingInterceptor, MetricsInterceptor, RateLimitInterceptor,
    extract_bearer_token,
};

#[cfg(feature = "grpc")]
pub use tls::{GrpcTlsConfig, GrpcTlsMetrics, TlsIdentity};

#[cfg(feature = "grpc")]
pub use health::{DependencyHealth, HealthInfo, HealthStatus, ServingStatus};
