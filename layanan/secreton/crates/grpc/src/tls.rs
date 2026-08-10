//! TLS Configuration for gRPC
//!
//! Provides mTLS support for gRPC server with client certificate verification.
//!
//! Environment variables, named to match authenc's gRPC boundary so the mesh
//! is configured one way:
//!
//! | Variable              | Meaning                                            |
//! |-----------------------|----------------------------------------------------|
//! | `GRPC_TLS_CERT_PATH`  | server certificate (PEM)                            |
//! | `GRPC_TLS_KEY_PATH`   | server private key (PEM)                            |
//! | `GRPC_TLS_CA_PATH`    | CA that signs client certs — presence enables mTLS  |
//! | `GRPC_ALLOW_INSECURE` | `true` opts out of all of the above, loudly         |

use std::path::PathBuf;
use tracing::{error, info};

/// Env var: server certificate path.
pub const TLS_CERT_ENV: &str = "GRPC_TLS_CERT_PATH";
/// Env var: server private key path.
pub const TLS_KEY_ENV: &str = "GRPC_TLS_KEY_PATH";
/// Env var: CA certificate used to verify client certificates.
pub const TLS_CA_ENV: &str = "GRPC_TLS_CA_PATH";
/// Env var: explicit opt-out from transport security and client auth.
pub const ALLOW_INSECURE_ENV: &str = "GRPC_ALLOW_INSECURE";

/// How the gRPC listener is allowed to come up.
#[derive(Debug, Clone)]
pub enum ListenerSecurity {
    /// mTLS with client-certificate authentication enforced.
    Mtls(Box<GrpcTlsConfig>),
    /// No transport security, no caller authentication. Carries the operator's
    /// stated reason so it can be logged at startup.
    Insecure { reason: String },
}

/// Decide how the listener must be secured — **fail-closed**.
///
/// Without TLS material the only way to serve is an explicit opt-out. This is
/// the rule the whole change turns on, so it lives in one pure function that
/// can be unit-tested rather than in the middle of `main`.
///
/// `tls` is what [`GrpcTlsConfig::from_env`] found; `allow_insecure` is the
/// operator's opt-out flag.
pub fn resolve_listener_security(
    tls: Option<GrpcTlsConfig>,
    allow_insecure: bool,
) -> Result<ListenerSecurity, String> {
    match (tls, allow_insecure) {
        // TLS material wins even if the opt-out is also set: a configured
        // operator who left a stale flag behind should get the secure path.
        (Some(cfg), _) if cfg.require_client_auth => Ok(ListenerSecurity::Mtls(Box::new(cfg))),

        // Server cert but no CA: one-way TLS authenticates the *server* to the
        // caller and leaves the caller anonymous. That is not client auth, so
        // it does not satisfy the requirement.
        (Some(_), true) => Ok(ListenerSecurity::Insecure {
            reason: format!(
                "{TLS_CA_ENV} is unset so callers cannot be authenticated, and \
                 {ALLOW_INSECURE_ENV}=true"
            ),
        }),
        (Some(_), false) => Err(format!(
            "gRPC client authentication is required but {TLS_CA_ENV} is unset. Point it at \
             the CA that signs client certificates (Secreton PKI), or set \
             {ALLOW_INSECURE_ENV}=true to serve anonymous callers deliberately."
        )),

        (None, true) => Ok(ListenerSecurity::Insecure {
            reason: format!("{ALLOW_INSECURE_ENV}=true"),
        }),
        (None, false) => Err(format!(
            "gRPC mTLS is required: set {TLS_CERT_ENV}, {TLS_KEY_ENV} and {TLS_CA_ENV} \
             (Secreton PKI), or set {ALLOW_INSECURE_ENV}=true for a transitional deploy. \
             Refusing to serve secrets to unauthenticated callers."
        )),
    }
}

/// Read [`ALLOW_INSECURE_ENV`] as a boolean. Only the exact string `true`
/// (any case) opts out — a typo must not silently disable authentication.
pub fn allow_insecure_from_env() -> bool {
    std::env::var(ALLOW_INSECURE_ENV)
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

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

    /// Build from the environment.
    ///
    /// Returns `None` when no server certificate is configured. A CA path
    /// implies client authentication — there is no way to load a CA and
    /// *not* verify clients against it, because a CA that verifies nobody is
    /// the configuration that produced this bug in the first place.
    pub fn from_env() -> Option<Self> {
        let cert = std::env::var(TLS_CERT_ENV).ok()?;
        let key = std::env::var(TLS_KEY_ENV).ok()?;
        let mut config = Self::new(PathBuf::from(cert), PathBuf::from(key));
        if let Ok(ca) = std::env::var(TLS_CA_ENV) {
            config = config.with_ca_cert(PathBuf::from(ca)).with_client_auth();
        }
        Some(config)
    }

    /// Load TLS configuration and return identity
    pub async fn load(&self) -> Result<TlsIdentity, Box<dyn std::error::Error>> {
        info!("Loading gRPC TLS configuration");

        // Load server certificate and key
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

        // Load CA certificate if client auth is required
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
        // Check if certificate file exists
        if !self.cert_path.exists() {
            return Err(format!("Certificate file not found: {:?}", self.cert_path));
        }

        // Check if key file exists
        if !self.key_path.exists() {
            return Err(format!("Private key file not found: {:?}", self.key_path));
        }

        // Check if CA cert exists when client auth is required
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

    /// Export metrics in Prometheus format
    pub fn to_prometheus(&self) -> String {
        format!(
            r#"# HELP grpc_tls_handshakes_total Total number of TLS handshakes
# TYPE grpc_tls_handshakes_total counter
grpc_tls_handshakes_total {}

# HELP grpc_tls_handshakes_successful Successful TLS handshakes
# TYPE grpc_tls_handshakes_successful counter
grpc_tls_handshakes_successful {}

# HELP grpc_tls_handshakes_failed Failed TLS handshakes
# TYPE grpc_tls_handshakes_failed counter
grpc_tls_handshakes_failed {}

# HELP grpc_tls_client_cert_verifications_total Total client certificate verifications
# TYPE grpc_tls_client_cert_verifications_total counter
grpc_tls_client_cert_verifications_total {}

# HELP grpc_tls_client_cert_verifications_failed Failed client certificate verifications
# TYPE grpc_tls_client_cert_verifications_failed counter
grpc_tls_client_cert_verifications_failed {}

# HELP grpc_tls_success_rate TLS handshake success rate percentage
# TYPE grpc_tls_success_rate gauge
grpc_tls_success_rate {}
"#,
            self.total_handshakes,
            self.successful_handshakes,
            self.failed_handshakes,
            self.client_cert_verifications,
            self.failed_client_cert_verifications,
            self.success_rate(),
        )
    }
}

/// Tests for the fail-closed rule. Deliberately NOT behind
/// `enable-inline-tests`: this is the decision that keeps an unauthenticated
/// secret store from booting, so it must run on a bare `cargo test`.
#[cfg(test)]
mod security_tests {
    use super::*;

    fn mtls_material() -> GrpcTlsConfig {
        GrpcTlsConfig::new(PathBuf::from("server.crt"), PathBuf::from("server.key"))
            .with_ca_cert(PathBuf::from("ca.crt"))
            .with_client_auth()
    }

    fn server_cert_only() -> GrpcTlsConfig {
        GrpcTlsConfig::new(PathBuf::from("server.crt"), PathBuf::from("server.key"))
    }

    #[test]
    fn no_tls_and_no_opt_out_refuses_to_serve() {
        let err = resolve_listener_security(None, false)
            .expect_err("an unauthenticated secret store must not be allowed to start");
        assert!(
            err.contains(ALLOW_INSECURE_ENV),
            "error must name the escape hatch: {err}"
        );
    }

    #[test]
    fn a_server_cert_without_a_ca_is_not_client_auth() {
        // One-way TLS proves who the *server* is. Callers stay anonymous, so
        // this must be refused exactly like plaintext.
        resolve_listener_security(Some(server_cert_only()), false)
            .expect_err("one-way TLS must not pass for client authentication");
    }

    #[test]
    fn full_mtls_material_selects_the_enforcing_path() {
        let resolved = resolve_listener_security(Some(mtls_material()), false).unwrap();
        assert!(matches!(resolved, ListenerSecurity::Mtls(_)));
    }

    #[test]
    fn a_stale_opt_out_does_not_downgrade_a_configured_listener() {
        // Operator configured mTLS and left GRPC_ALLOW_INSECURE=true behind.
        // The secure path must still win.
        let resolved = resolve_listener_security(Some(mtls_material()), true).unwrap();
        assert!(
            matches!(resolved, ListenerSecurity::Mtls(_)),
            "a leftover opt-out must never turn off configured mTLS"
        );
    }

    #[test]
    fn the_opt_out_records_why_it_was_taken() {
        let resolved = resolve_listener_security(None, true).unwrap();
        match resolved {
            ListenerSecurity::Insecure { reason } => assert!(reason.contains(ALLOW_INSECURE_ENV)),
            other => panic!("expected the insecure path, got {other:?}"),
        }
    }
}

#[cfg(all(test, feature = "enable-inline-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_grpc_tls_config_creation() {
        let config = GrpcTlsConfig::new(
            PathBuf::from("/path/to/cert.pem"),
            PathBuf::from("/path/to/key.pem"),
        );

        assert_eq!(config.cert_path, PathBuf::from("/path/to/cert.pem"));
        assert_eq!(config.key_path, PathBuf::from("/path/to/key.pem"));
        assert_eq!(config.ca_cert_path, None);
        assert!(!config.require_client_auth);
    }

    #[test]
    fn test_grpc_tls_config_with_ca() {
        let config = GrpcTlsConfig::new(
            PathBuf::from("/path/to/cert.pem"),
            PathBuf::from("/path/to/key.pem"),
        )
        .with_ca_cert(PathBuf::from("/path/to/ca.pem"))
        .with_client_auth();

        assert_eq!(config.ca_cert_path, Some(PathBuf::from("/path/to/ca.pem")));
        assert!(config.require_client_auth);
    }

    #[test]
    fn test_grpc_tls_metrics() {
        let mut metrics = GrpcTlsMetrics::new();

        metrics.record_handshake_success();
        metrics.record_handshake_success();
        metrics.record_handshake_failure();

        assert_eq!(metrics.total_handshakes, 3);
        assert_eq!(metrics.successful_handshakes, 2);
        assert_eq!(metrics.failed_handshakes, 1);
        assert!((metrics.success_rate() - 66.67).abs() < 0.1);
    }

    #[test]
    fn test_client_cert_verification_metrics() {
        let mut metrics = GrpcTlsMetrics::new();

        metrics.record_client_cert_verification(true);
        metrics.record_client_cert_verification(true);
        metrics.record_client_cert_verification(false);

        assert_eq!(metrics.client_cert_verifications, 3);
        assert_eq!(metrics.failed_client_cert_verifications, 1);
    }

    #[test]
    fn test_prometheus_export() {
        let mut metrics = GrpcTlsMetrics::new();
        metrics.record_handshake_success();
        metrics.record_handshake_failure();

        let prometheus = metrics.to_prometheus();

        assert!(prometheus.contains("grpc_tls_handshakes_total 2"));
        assert!(prometheus.contains("grpc_tls_handshakes_successful 1"));
        assert!(prometheus.contains("grpc_tls_handshakes_failed 1"));
    }
}
