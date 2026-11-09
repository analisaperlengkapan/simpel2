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

/// Install rustls' process-wide default `CryptoProvider` (aws-lc-rs), exactly once.
///
/// rustls 0.23 picks a provider by itself ONLY when exactly one of its `aws_lc_rs` /
/// `ring` features is enabled in the whole dependency graph of the binary. Several of
/// our binaries have both (`lettre` turns on `rustls/ring` while `reqwest`/`tonic` use
/// the default `aws_lc_rs`), and then the first TLS config built in the process panics
/// with "Could not automatically determine the process-level CryptoProvider". That is
/// what took down `authenc` and `layanan-perlengkapan` the first time mTLS was switched
/// on in staging (rc39): the tests ran in a graph with one provider, the binaries did not.
///
/// Call this before building any TLS config. It is idempotent, and it leaves a provider
/// someone else already installed alone (`install_default` fails only in that case, which
/// is exactly the state we want).
pub fn ensure_crypto_provider() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        if rustls::crypto::CryptoProvider::get_default().is_none() {
            match rustls::crypto::aws_lc_rs::default_provider().install_default() {
                Ok(()) => {}
                Err(_already_installed_by_a_racing_caller) => {}
            }
        }
    });
}
