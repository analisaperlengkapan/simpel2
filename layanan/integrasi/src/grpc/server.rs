//! gRPC Server for IntegrasiService
//!
//! This module provides the gRPC server that exposes the IntegrasiService
//! for other backend services to consume integration data.

use std::net::SocketAddr;
use std::sync::Arc;

use std::path::PathBuf;

use tokio::signal;
use tokio_postgres::Client;
use tonic::transport::{Certificate, Identity, Server, ServerTlsConfig};
use tracing::{info, warn};

use crate::grpc::proto::integrasi_service_server::IntegrasiServiceServer;
use crate::grpc::service::IntegrasiServiceImpl;

/// Default gRPC server port
pub const DEFAULT_GRPC_PORT: u16 = 50051;

/// How the gRPC listener authenticates its callers.
///
/// This server used to call `Server::builder()` with no TLS and no gate, so
/// anything that could open a TCP connection to :50051 could read every
/// pegawai/satker/aset record integrasi holds (the master data of the whole
/// system) — while authenc and secreton, which already had the knobs, refuse to
/// start in production without mTLS. Same env contract as theirs, so one set of
/// operator instructions covers all three servers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListenerSecurity {
    /// Mutual TLS: the server presents `cert`/`key` and REQUIRES a client
    /// certificate signed by `ca`.
    Mtls {
        cert: PathBuf,
        key: PathBuf,
        ca: PathBuf,
    },
    /// Plaintext, anonymous callers accepted.
    Insecure { reason: String },
}

/// Decide the listener's security from the environment (via `get`, so the rules
/// are testable without mutating process state).
///
/// * `GRPC_TLS_CERT_PATH` + `GRPC_TLS_KEY_PATH` + `GRPC_TLS_CA_PATH` → mTLS;
/// * some but not all of them → ERROR (half-configured mTLS must not quietly
///   become plaintext);
/// * none, `GRPC_ALLOW_INSECURE=true` → plaintext, loudly (transitional);
/// * none, `APP_ENVIRONMENT=production` → ERROR: refuse to start;
/// * none, any other environment → plaintext (dev / CI compose), loudly.
pub fn resolve_listener_security(
    get: impl Fn(&str) -> Option<String>,
) -> Result<ListenerSecurity, String> {
    const NAMES: [&str; 3] = [
        "GRPC_TLS_CERT_PATH",
        "GRPC_TLS_KEY_PATH",
        "GRPC_TLS_CA_PATH",
    ];
    let read = |n: &str| get(n).filter(|v| !v.trim().is_empty());
    let (cert, key, ca) = (read(NAMES[0]), read(NAMES[1]), read(NAMES[2]));

    match (cert, key, ca) {
        (Some(cert), Some(key), Some(ca)) => Ok(ListenerSecurity::Mtls {
            cert: cert.into(),
            key: key.into(),
            ca: ca.into(),
        }),
        (None, None, None) => {
            if get("GRPC_ALLOW_INSECURE").as_deref() == Some("true") {
                return Ok(ListenerSecurity::Insecure {
                    reason: "GRPC_ALLOW_INSECURE=true".to_string(),
                });
            }
            let production = get("APP_ENVIRONMENT")
                .map(|v| v.eq_ignore_ascii_case("production"))
                .unwrap_or(false);
            if production {
                Err(format!(
                    "gRPC mTLS required in production: set {}, {} and {}, or set \
                     GRPC_ALLOW_INSECURE=true for a transitional deploy",
                    NAMES[0], NAMES[1], NAMES[2]
                ))
            } else {
                Ok(ListenerSecurity::Insecure {
                    reason: "no TLS material configured (non-production default)".to_string(),
                })
            }
        }
        (cert, key, ca) => {
            let missing: Vec<&str> = NAMES
                .iter()
                .zip([cert.is_some(), key.is_some(), ca.is_some()])
                .filter(|(_, set)| !set)
                .map(|(n, _)| *n)
                .collect();
            Err(format!(
                "gRPC mTLS is half-configured: {} not set — set all three or none",
                missing.join(", ")
            ))
        }
    }
}

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

    let mut builder = match resolve_listener_security(|k| std::env::var(k).ok())? {
        ListenerSecurity::Mtls { cert, key, ca } => {
            let read = |p: &PathBuf| {
                std::fs::read(p).map_err(|e| format!("cannot read {}: {e}", p.display()))
            };
            let tls = ServerTlsConfig::new()
                .identity(Identity::from_pem(read(&cert)?, read(&key)?))
                .client_ca_root(Certificate::from_pem(read(&ca)?));
            info!("gRPC listener requires client certificates (mTLS)");
            Server::builder().tls_config(tls)?
        }
        ListenerSecurity::Insecure { reason } => {
            warn!(%reason, "gRPC listener is PLAINTEXT and accepts anonymous callers");
            Server::builder()
        }
    };

    info!("gRPC server listening on {}", addr);

    // Start the server with graceful shutdown
    builder
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

#[cfg(test)]
mod listener_security_tests {
    use super::*;
    use std::collections::HashMap;

    fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let m: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |k| m.get(k).cloned()
    }

    const FULL: [(&str, &str); 3] = [
        ("GRPC_TLS_CERT_PATH", "/c"),
        ("GRPC_TLS_KEY_PATH", "/k"),
        ("GRPC_TLS_CA_PATH", "/ca"),
    ];

    #[test]
    fn full_material_selects_mtls_even_in_production() {
        let mut pairs = FULL.to_vec();
        pairs.push(("APP_ENVIRONMENT", "production"));
        assert!(matches!(
            resolve_listener_security(env(&pairs)),
            Ok(ListenerSecurity::Mtls { .. })
        ));
    }

    #[test]
    fn production_without_material_refuses_to_start() {
        let err = resolve_listener_security(env(&[("APP_ENVIRONMENT", "Production")])).unwrap_err();
        assert!(err.contains("mTLS required in production"), "{err}");
    }

    #[test]
    fn production_with_the_explicit_opt_out_serves_plaintext() {
        let r = resolve_listener_security(env(&[
            ("APP_ENVIRONMENT", "production"),
            ("GRPC_ALLOW_INSECURE", "true"),
        ]));
        assert!(matches!(r, Ok(ListenerSecurity::Insecure { .. })));
    }

    #[test]
    fn only_the_exact_string_true_opts_out() {
        for v in ["1", "yes", "TRUE", "false", ""] {
            let r = resolve_listener_security(env(&[
                ("APP_ENVIRONMENT", "production"),
                ("GRPC_ALLOW_INSECURE", v),
            ]));
            assert!(r.is_err(), "{v:?} must not count as the opt-out");
        }
    }

    #[test]
    fn non_production_without_material_stays_plaintext() {
        for e in [&[][..], &[("APP_ENVIRONMENT", "staging")][..]] {
            assert!(matches!(
                resolve_listener_security(env(e)),
                Ok(ListenerSecurity::Insecure { .. })
            ));
        }
    }

    #[test]
    fn every_partial_combination_is_an_error_in_every_environment() {
        for mask in 1u8..7 {
            let pairs: Vec<_> = FULL
                .iter()
                .enumerate()
                .filter(|(i, _)| mask & (1 << i) != 0)
                .map(|(_, p)| *p)
                .collect();
            for extra in [&[][..], &[("GRPC_ALLOW_INSECURE", "true")][..]] {
                let mut all = pairs.clone();
                all.extend_from_slice(extra);
                assert!(
                    resolve_listener_security(env(&all)).is_err(),
                    "mask {mask:03b} {extra:?}"
                );
            }
        }
    }
}
