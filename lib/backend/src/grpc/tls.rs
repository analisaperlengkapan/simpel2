//! TLS Configuration for gRPC
//!
//! Provides mTLS support for gRPC servers with client certificate verification.
//!
//! # Example
//!
//! ```rust,ignore
//! use lib_backend::grpc::tls::GrpcTlsConfig;
//! use std::path::PathBuf;
//!
//! let config = GrpcTlsConfig::new(
//!     PathBuf::from("/path/to/cert.pem"),
//!     PathBuf::from("/path/to/key.pem"),
//! )
//! .with_ca_cert(PathBuf::from("/path/to/ca.pem"))
//! .with_client_auth();
//!
//! let identity = config.load().await?;
//! ```

use std::path::PathBuf;
use tracing::{error, info};

/// TLS configuration for gRPC server
#[derive(Debug, Clone)]
pub struct GrpcTlsConfig {
    /// Path to TLS certificate
    pub cert_path: PathBuf,
    /// Path to TLS private key
    pub key_path: PathBuf,
    /// Path to CA certificate (optional)
    pub ca_cert_path: Option<PathBuf>,
    /// Require client authentication
    pub require_client_auth: bool,
}

/// TLS certificate and key data
#[derive(Debug, Clone)]
pub struct TlsIdentity {
    pub cert: Vec<u8>,
    pub key: Vec<u8>,
    pub ca_cert: Option<Vec<u8>>,
}

impl GrpcTlsConfig {
    /// Create a new TLS configuration
    pub fn new(cert_path: PathBuf, key_path: PathBuf) -> Self {
        Self {
            cert_path,
            key_path,
            ca_cert_path: None,
            require_client_auth: false,
        }
    }

    /// Set CA certificate path
    pub fn with_ca_cert(mut self, ca_cert_path: PathBuf) -> Self {
        self.ca_cert_path = Some(ca_cert_path);
        self
    }

    /// Enable client authentication
    pub fn with_client_auth(mut self) -> Self {
        self.require_client_auth = true;
        self
    }

    /// Load TLS configuration and return identity
    pub async fn load(&self) -> Result<TlsIdentity, Box<dyn std::error::Error + Send + Sync>> {
        info!("Loading gRPC TLS configuration");

        let cert = tokio::fs::read(&self.cert_path).await.map_err(|e| {
            format!(
                "Failed to read certificate from {:?}: {}",
                self.cert_path, e
            )
        })?;

        let key = tokio::fs::read(&self.key_path)
            .await
            .map_err(|e| format!("Failed to read private key from {:?}: {}", self.key_path, e))?;

        info!("Loaded server certificate and key");

        let ca_cert = if self.require_client_auth {
            if let Some(ca_cert_path) = &self.ca_cert_path {
                info!("Loading CA certificate for client authentication");
                let ca = tokio::fs::read(ca_cert_path).await.map_err(|e| {
                    format!(
                        "Failed to read CA certificate from {:?}: {}",
                        ca_cert_path, e
                    )
                })?;
                info!("Client certificate authentication enabled");
                Some(ca)
            } else {
                error!("Client authentication requested but no CA certificate provided");
                return Err("Client authentication requires CA certificate".into());
            }
        } else {
            None
        };

        info!("gRPC TLS configuration loaded successfully");
        Ok(TlsIdentity { cert, key, ca_cert })
    }

    /// Validate TLS configuration
    pub fn validate(&self) -> Result<(), String> {
        if !self.cert_path.exists() {
            return Err(format!("Certificate file not found: {:?}", self.cert_path));
        }

        if !self.key_path.exists() {
            return Err(format!("Private key file not found: {:?}", self.key_path));
        }

        if self.require_client_auth {
            if let Some(ca_cert_path) = &self.ca_cert_path {
                if !ca_cert_path.exists() {
                    return Err(format!("CA certificate file not found: {:?}", ca_cert_path));
                }
            } else {
                return Err("Client authentication requires CA certificate path".to_string());
            }
        }

        Ok(())
    }
}

/// TLS metrics for gRPC connections
#[derive(Debug, Default, Clone)]
pub struct GrpcTlsMetrics {
    /// Total TLS handshakes
    pub total_handshakes: u64,
    /// Successful handshakes
    pub successful_handshakes: u64,
    /// Failed handshakes
    pub failed_handshakes: u64,
    /// Client certificate verifications
    pub client_cert_verifications: u64,
    /// Failed client certificate verifications
    pub failed_client_cert_verifications: u64,
}

impl GrpcTlsMetrics {
    /// Create new metrics
    pub fn new() -> Self {
        Self::default()
    }

    /// Record successful handshake
    pub fn record_handshake_success(&mut self) {
        self.total_handshakes += 1;
        self.successful_handshakes += 1;
    }

    /// Record failed handshake
    pub fn record_handshake_failure(&mut self) {
        self.total_handshakes += 1;
        self.failed_handshakes += 1;
    }

    /// Record client certificate verification
    pub fn record_client_cert_verification(&mut self, success: bool) {
        self.client_cert_verifications += 1;
        if !success {
            self.failed_client_cert_verifications += 1;
        }
    }

    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_handshakes == 0 {
            0.0
        } else {
            (self.successful_handshakes as f64 / self.total_handshakes as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grpc_tls_config_creation() {
        let config = GrpcTlsConfig::new(
            PathBuf::from("/path/to/cert.pem"),
            PathBuf::from("/path/to/key.pem"),
        );

        assert_eq!(config.cert_path, PathBuf::from("/path/to/cert.pem"));
        assert!(!config.require_client_auth);
    }

    #[test]
    fn test_grpc_tls_config_with_mtls() {
        let config = GrpcTlsConfig::new(
            PathBuf::from("/path/to/cert.pem"),
            PathBuf::from("/path/to/key.pem"),
        )
        .with_ca_cert(PathBuf::from("/path/to/ca.pem"))
        .with_client_auth();

        assert!(config.require_client_auth);
        assert_eq!(config.ca_cert_path, Some(PathBuf::from("/path/to/ca.pem")));
    }

    #[test]
    fn test_tls_metrics() {
        let mut metrics = GrpcTlsMetrics::new();

        metrics.record_handshake_success();
        metrics.record_handshake_success();
        metrics.record_handshake_failure();

        assert_eq!(metrics.total_handshakes, 3);
        assert_eq!(metrics.successful_handshakes, 2);
        assert!((metrics.success_rate() - 66.67).abs() < 0.1);
    }
}
