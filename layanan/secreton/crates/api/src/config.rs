//! Configuration management for the Secreton API server.
//!
//! Provides comprehensive configuration options for HTTP/gRPC servers,
//! authentication, authorization, rate limiting, and security features.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

// Re-export HSM configuration from secreton-hsm crate
pub use secreton_hsm::HsmConfig;

/// Main API configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiConfig {
    /// HTTP server configuration
    pub http: HttpConfig,

    /// gRPC server configuration
    pub grpc: GrpcConfig,

    /// Authentication configuration
    pub auth: AuthConfig,

    /// Rate limiting configuration
    pub rate_limit: RateLimitConfig,

    /// TLS configuration
    pub tls: Option<TlsConfig>,

    /// Monitoring configuration
    pub monitoring: MonitoringConfig,

    /// CORS configuration
    pub cors: CorsConfig,

    /// Logging configuration
    pub logging: LoggingConfig,

    /// HSM configuration
    #[serde(default)]
    pub hsm: HsmConfig,

    /// Database configuration
    #[serde(default)]
    pub database: DatabaseConfig,

    /// Storage backend configuration
    #[serde(default)]
    pub storage: StorageConfig,
}

impl ApiConfig {
    /// Load configuration from environment or default
    pub fn load() -> anyhow::Result<Self> {
        // Basic implementation: return default for now, as real loading is done in main via from_bootstrap_and_application
        Ok(Self::default())
    }

    /// Create ApiConfig from BootstrapConfig and ApplicationConfig
    pub fn from_bootstrap_and_application(
        bootstrap: &secreton_core::config::BootstrapConfig,
        _app: &secreton_core::config::ApplicationConfig,
    ) -> Result<Self, String> {
        let mut config = Self::default();

        // Map HTTP config
        config.http.bind_address = bootstrap
            .http
            .address
            .parse()
            .map_err(|e| format!("Invalid HTTP address '{}': {}", bootstrap.http.address, e))?;

        // Map gRPC config
        config.grpc.bind_address = bootstrap
            .grpc
            .address
            .parse()
            .map_err(|e| format!("Invalid gRPC address '{}': {}", bootstrap.grpc.address, e))?;
        config.grpc.enabled = bootstrap.grpc.enabled;

        // Map storage config
        if let secreton_core::config::StorageBackendConfig::Raft(raft_bootstrap) =
            &bootstrap.storage.config
        {
            let node_id = raft_bootstrap
                .node_id
                .parse::<u64>()
                .map_err(|e| format!("Invalid node_id '{}': {}", raft_bootstrap.node_id, e))?;

            config.storage.raft.node_id = node_id;

            if let Some(addr) = raft_bootstrap.listener_addr.as_ref() {
                config.storage.raft.bind_address = addr
                    .parse()
                    .map_err(|e| format!("Invalid Raft listener address '{}': {}", addr, e))?;
            } else {
                config.storage.raft.bind_address = format!("127.0.0.1:{}", 8200 + node_id)
                    .parse()
                    .expect("Failed to parse default Raft address");
            }
        }

        // Map other fields as needed (simplified for now)
        // In a real implementation, we would map all fields from app config

        Ok(config)
    }
}

/// HTTP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConfig {
    /// Address to bind HTTP server
    pub bind_address: SocketAddr,

    /// Request timeout
    pub timeout: Duration,

    /// Maximum request body size (bytes)
    pub max_body_size: usize,

    /// Keep-alive timeout
    pub keep_alive: Duration,

    /// Enable compression
    pub compression: bool,

    /// Enable static file serving
    pub static_files: Option<StaticFilesConfig>,
}

/// gRPC server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcConfig {
    /// Enable gRPC server
    pub enabled: bool,

    /// Address to bind gRPC server
    pub bind_address: SocketAddr,

    /// Request timeout
    pub timeout: Duration,

    /// Maximum message size (bytes)
    pub max_message_size: usize,

    /// Enable reflection
    pub reflection: bool,

    /// Enable health check service
    pub health_check: bool,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthConfig {
    /// JWT configuration
    pub jwt: JwtConfig,

    /// OAuth2 configuration
    pub oauth2: Option<OAuth2Config>,

    /// mTLS configuration
    pub mtls: Option<MtlsConfig>,

    /// Session configuration
    pub session: SessionConfig,

    /// Password policy configuration
    #[serde(default)]
    pub password_policy: PasswordPolicyConfig,

    /// Multi-factor authentication
    pub mfa: MfaConfig,
}

/// JWT configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    /// JWT signing secret
    pub secret: String,

    /// Token expiration time
    pub expiration: Duration,

    /// Refresh token expiration
    pub refresh_expiration: Duration,

    /// JWT algorithm
    pub algorithm: String,

    /// Issuer
    pub issuer: String,

    /// Audience
    pub audience: String,
}

/// OAuth2 configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Config {
    /// OAuth2 provider URLs
    pub providers: Vec<OAuth2Provider>,

    /// Redirect URL
    pub redirect_url: String,

    /// Scopes to request
    pub scopes: Vec<String>,
}

/// OAuth2 provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Provider {
    /// Provider name
    pub name: String,

    /// Client ID
    pub client_id: String,

    /// Client secret
    pub client_secret: String,

    /// Authorization URL
    pub auth_url: String,

    /// Token URL
    pub token_url: String,

    /// User info URL
    pub user_info_url: String,
}

/// mTLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MtlsConfig {
    /// Require client certificates
    pub required: bool,

    /// CA certificate path
    pub ca_cert: PathBuf,

    /// Allowed client certificate subjects
    pub allowed_subjects: Vec<String>,

    /// Certificate revocation list
    pub crl: Option<PathBuf>,
}

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Session timeout
    pub timeout: Duration,

    /// Session store type
    pub store: SessionStore,

    /// Cookie configuration
    pub cookie: CookieConfig,
}

/// Session store types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SessionStore {
    Memory,
    Redis { url: String },
    Database { table: String },
}

/// Password policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicyConfig {
    /// Minimum password length
    pub min_length: u8,

    /// Require uppercase characters
    pub require_uppercase: bool,

    /// Require lowercase characters
    pub require_lowercase: bool,

    /// Require numbers
    pub require_numbers: bool,

    /// Require special characters
    pub require_special: bool,
}

impl Default for PasswordPolicyConfig {
    fn default() -> Self {
        Self {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special: true,
        }
    }
}

/// Cookie configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieConfig {
    /// Cookie name
    pub name: String,

    /// Cookie domain
    pub domain: Option<String>,

    /// Cookie path
    pub path: String,

    /// Secure flag
    pub secure: bool,

    /// HttpOnly flag
    pub http_only: bool,

    /// SameSite policy
    pub same_site: String,
}

/// Multi-factor authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MfaConfig {
    /// Enable MFA
    pub enabled: bool,

    /// TOTP configuration
    pub totp: TotpConfig,

    /// SMS configuration
    pub sms: Option<SmsConfig>,

    /// Email configuration
    pub email: Option<EmailConfig>,

    /// WebAuthn configuration
    pub webauthn: Option<WebAuthnConfig>,

    /// Government-specific MFA policies
    pub policies: MfaPoliciesConfig,

    /// Recovery codes configuration
    pub recovery_codes: MfaRecoveryCodesConfig,

    /// Rate limiting for MFA operations
    pub rate_limiting: MfaRateLimitingConfig,

    /// Audit logging configuration
    pub audit: MfaAuditConfig,

    /// Security settings for MFA secrets
    pub security: MfaSecurityConfig,

    /// Backup and disaster recovery
    pub backup: MfaBackupConfig,

    /// Monitoring and alerting
    pub monitoring: MfaMonitoringConfig,

    /// Government compliance settings
    pub compliance: MfaComplianceConfig,

    /// Access control for MFA operations
    pub access_control: MfaAccessControlConfig,

    /// Network security settings
    pub network: MfaNetworkConfig,

    /// Session management for MFA
    pub session: MfaSessionConfig,
}

/// TOTP configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpConfig {
    /// Issuer name
    pub issuer: String,

    /// Secret length
    pub secret_length: usize,

    /// Time step (seconds)
    pub time_step: u64,

    /// Code length
    pub code_length: usize,

    /// Clock skew tolerance
    pub skew_tolerance: u64,
}

/// SMS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsConfig {
    /// SMS provider
    pub provider: String,

    /// API key
    pub api_key: String,

    /// From number
    pub from_number: String,
}

/// Email configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    /// SMTP server
    pub smtp_server: String,

    /// SMTP port
    pub smtp_port: u16,

    /// Username
    pub username: String,

    /// Password
    pub password: String,

    /// From address
    pub from_address: String,

    /// Use TLS
    pub use_tls: bool,
}

/// WebAuthn configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAuthnConfig {
    /// Relying party name
    pub rp_name: String,

    /// Relying party ID
    pub rp_id: String,

    /// Origin
    pub origin: String,
}

/// MFA enforcement policies for government employees
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaPoliciesConfig {
    /// Require MFA for all government employees
    pub enforce_for_all: bool,

    /// Grace period for MFA setup (in days)
    pub setup_grace_period: u32,

    /// Maximum failed MFA attempts before account lockout
    pub max_failed_attempts: u32,

    /// Account lockout duration in minutes
    pub lockout_duration: u32,

    /// Require MFA re-verification after this many hours
    pub reauth_interval: u32,

    /// Role-based MFA requirements
    pub roles: MfaRolePoliciesConfig,

    /// Organizational unit (Satker) specific MFA policies
    pub satker: MfaSatkerPoliciesConfig,
}

/// Role-based MFA requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaRolePoliciesConfig {
    /// Administrative roles require immediate MFA setup
    pub admin_immediate_setup: Vec<String>,

    /// High-privilege roles require stricter MFA policies
    pub high_privilege_roles: Vec<String>,

    /// Standard employee roles
    pub standard_roles: Vec<String>,
}

/// Satker-specific MFA policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaSatkerPoliciesConfig {
    /// High-security satkers require immediate MFA setup
    pub high_security_satkers: Vec<String>,

    /// Standard satkers have normal grace period
    pub standard_satkers: Vec<String>,
}

/// Recovery codes configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaRecoveryCodesConfig {
    /// Number of recovery codes to generate
    pub count: u32,

    /// Length of each recovery code
    pub length: u32,

    /// Format: "numeric", "alphanumeric", "hex"
    pub format: String,

    /// Allow recovery code reuse (not recommended for security)
    pub allow_reuse: bool,
}

/// Rate limiting for MFA operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaRateLimitingConfig {
    /// Maximum MFA verification attempts per minute
    pub max_attempts_per_minute: u32,

    /// Maximum MFA setup attempts per hour
    pub max_setup_attempts_per_hour: u32,

    /// Rate limit window in seconds
    pub window_seconds: u32,

    /// Enable progressive delays for failed attempts
    pub progressive_delays: bool,
}

/// Audit logging for MFA operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaAuditConfig {
    /// Enable comprehensive MFA audit logging
    pub enabled: bool,

    /// Log successful MFA verifications
    pub log_success: bool,

    /// Log failed MFA attempts
    pub log_failures: bool,

    /// Log MFA setup and configuration changes
    pub log_setup_changes: bool,

    /// Log recovery code usage
    pub log_recovery_usage: bool,

    /// Retention period for MFA audit logs (in days)
    pub retention_days: u32,
}

/// Security settings for MFA secrets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaSecurityConfig {
    /// Encryption algorithm for MFA secrets
    pub secret_encryption: String,

    /// Key derivation function for MFA secret encryption
    pub kdf: String,

    /// Number of KDF iterations
    pub kdf_iterations: u32,

    /// Salt length for KDF
    pub salt_length: u32,

    /// Enable hardware security module (HSM) for key storage
    pub use_hsm: bool,

    /// HSM configuration (if enabled)
    pub hsm_slot: u32,

    /// HSM PIN (should be provided via environment variable)
    pub hsm_pin: Option<String>,
}

/// Backup and disaster recovery for MFA data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaBackupConfig {
    /// Enable automatic backup of MFA configurations
    pub enabled: bool,

    /// Backup interval in hours
    pub interval_hours: u32,

    /// Backup retention period in days
    pub retention_days: u32,

    /// Backup encryption key (should be different from main encryption key)
    pub backup_encryption_key: Option<String>,

    /// Backup storage location
    pub backup_path: String,
}

/// Monitoring and alerting for MFA operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaMonitoringConfig {
    /// Enable MFA metrics collection
    pub metrics_enabled: bool,

    /// Alert on suspicious MFA patterns
    pub alert_on_anomalies: bool,

    /// Threshold for failed MFA attempts to trigger alert
    pub failed_attempts_threshold: u32,

    /// Time window for failed attempts monitoring (in minutes)
    pub monitoring_window_minutes: u32,

    /// Alert destinations
    pub alert_email: Option<String>,

    /// Alert webhook URL
    pub alert_webhook: Option<String>,
}

/// Government compliance settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaComplianceConfig {
    /// Enable FIPS 140-2 compliance mode
    pub fips_mode: bool,

    /// Enable audit trail integrity verification
    pub audit_integrity_check: bool,

    /// Require administrator approval for MFA policy changes
    pub require_admin_approval: bool,

    /// Enable automatic compliance reporting
    pub compliance_reporting: bool,

    /// Compliance report generation interval (in hours)
    pub report_interval_hours: u32,
}

/// Access control for MFA operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaAccessControlConfig {
    /// Require specific permissions for MFA operations
    pub setup_permission: String,

    /// Permission required for MFA verification
    pub verify_permission: String,

    /// Permission required for MFA administration
    pub admin_permission: String,

    /// Permission required for MFA audit access
    pub audit_permission: String,
}

/// Network security settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaNetworkConfig {
    /// Allowed IP ranges for MFA operations (government networks)
    pub allowed_ip_ranges: Vec<String>,

    /// Enable IP-based rate limiting
    pub ip_rate_limiting: bool,

    /// Maximum requests per IP per minute
    pub max_requests_per_ip: u32,
}

/// Session management for MFA
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaSessionConfig {
    /// MFA session timeout (in minutes)
    pub session_timeout: u32,

    /// Require MFA for session extension
    pub require_mfa_for_extension: bool,

    /// Maximum concurrent MFA sessions per user
    pub max_concurrent_sessions: u32,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Enable rate limiting
    pub enabled: bool,

    /// Global rate limits
    pub global: RateLimitRule,

    /// Per-endpoint rate limits
    pub endpoints: Vec<EndpointRateLimit>,

    /// Per-user rate limits
    pub per_user: Option<RateLimitRule>,

    /// Per-IP rate limits
    pub per_ip: Option<RateLimitRule>,
}

/// Rate limit rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitRule {
    /// Requests per time window
    pub requests: u32,

    /// Time window duration
    pub window: Duration,

    /// Burst size
    pub burst: Option<u32>,
}

/// Endpoint-specific rate limiting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointRateLimit {
    /// Endpoint pattern
    pub pattern: String,

    /// Rate limit rule
    pub rule: RateLimitRule,
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Certificate file path
    pub cert_file: PathBuf,

    /// Private key file path
    pub key_file: PathBuf,

    /// CA certificate file path
    pub ca_file: Option<PathBuf>,

    /// Minimum TLS version
    pub min_version: String,

    /// Cipher suites
    pub cipher_suites: Vec<String>,

    /// ALPN protocols
    pub alpn_protocols: Vec<String>,
}

/// Jaeger tracing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JaegerConfig {
    /// Jaeger endpoint URL
    pub endpoint: String,

    /// Service name for tracing
    pub service_name: String,

    /// Sampling rate (0.0 to 1.0)
    pub sample_rate: f64,
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable metrics
    pub metrics: bool,

    /// Metrics endpoint
    pub metrics_path: String,

    /// Health check endpoint
    pub health_path: String,

    /// Enable tracing
    pub tracing: bool,

    /// Jaeger configuration
    pub jaeger: Option<JaegerConfig>,
}

/// CORS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    /// Enable CORS
    pub enabled: bool,

    /// Allowed origins
    pub allowed_origins: Vec<String>,

    /// Allowed methods
    pub allowed_methods: Vec<String>,

    /// Allowed headers
    pub allowed_headers: Vec<String>,

    /// Exposed headers
    pub exposed_headers: Vec<String>,

    /// Max age
    pub max_age: Option<Duration>,

    /// Allow credentials
    pub allow_credentials: bool,
}

/// Static files configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticFilesConfig {
    /// Static files directory
    pub directory: PathBuf,

    /// URL path prefix
    pub path_prefix: String,

    /// Enable directory listing
    pub directory_listing: bool,

    /// Default index file
    pub index_file: Option<String>,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    pub level: String,

    /// Log format
    pub format: String,

    /// Enable JSON logging
    pub json: bool,

    /// Log file path
    pub file: Option<PathBuf>,

    /// Log rotation
    pub rotation: Option<LogRotationConfig>,
}

/// Log rotation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRotationConfig {
    /// Maximum file size
    pub max_size: u64,

    /// Maximum number of files
    pub max_files: u32,

    /// Rotation frequency
    pub frequency: String,
}

/// Storage backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Storage backend type: "raft", "memory", "postgres", "file", "redis", "consul"
    /// Default: "raft" (Raft consensus for HA)
    pub backend: String,

    /// Raft cluster configuration
    #[serde(default)]
    pub raft: RaftConfig,

    /// PostgreSQL backend URL (if backend = "postgres")
    pub postgres_url: Option<String>,

    /// File storage path (if backend = "file")
    pub file_path: Option<String>,

    /// Redis connection URL (if backend = "redis")
    pub redis_url: Option<String>,

    /// Consul address (if backend = "consul")
    pub consul_address: Option<String>,

    /// Consul path prefix (if backend = "consul")
    pub consul_path: Option<String>,
}

/// Raft cluster configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaftConfig {
    /// Node ID for this Raft node
    pub node_id: u64,

    /// Bind address for Raft cluster communication
    pub bind_address: SocketAddr,

    /// List of peer nodes: "node_id=http://address:port"
    pub peers: Vec<String>,

    /// Snapshot interval in seconds
    pub snapshot_interval: u64,

    /// Election timeout in milliseconds
    pub election_timeout_ms: u64,

    /// Heartbeat interval in milliseconds
    pub heartbeat_interval_ms: u64,

    /// Data directory for Raft logs and snapshots
    pub data_dir: PathBuf,
}

#[cfg(feature = "raft-consensus")]
impl From<RaftConfig> for secreton_storage::raft::RaftClusterConfig {
    fn from(api_config: RaftConfig) -> Self {
        let mut peers = std::collections::HashMap::new();
        for peer in &api_config.peers {
            if let Some((id_str, addr)) = peer.split_once('=') {
                if let Ok(id) = id_str.trim().parse::<u64>() {
                    peers.insert(id, addr.trim().to_string());
                }
            }
        }

        Self {
            node_id: api_config.node_id,
            bind_address: api_config.bind_address.to_string(),
            peers,
            election_timeout_ms: api_config.election_timeout_ms,
            heartbeat_interval_ms: api_config.heartbeat_interval_ms,
            max_payload_entries: 1000, // TODO: Make this configurable
            enable_tick: true,
            bootstrap: true, // TODO: Make this configurable
        }
    }
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database host
    pub host: String,

    /// Database port
    pub port: u16,

    /// Database name
    pub database: String,

    /// Database username
    pub username: String,

    /// Database password
    pub password: String,

    /// Maximum connections in pool
    pub max_connections: u32,

    /// Connection timeout in seconds
    pub connection_timeout: u64,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            // Default to Raft for HA and safety
            backend: "raft".to_string(),
            raft: RaftConfig::default(),
            postgres_url: None,
            file_path: None,
            redis_url: None,
            consul_address: None,
            consul_path: None,
        }
    }
}

impl Default for RaftConfig {
    fn default() -> Self {
        Self {
            node_id: 1,
            // Safe: hardcoded localhost address is always valid
            bind_address: "127.0.0.1:7000"
                .parse()
                .expect("hardcoded localhost address is valid"),
            peers: vec![],
            snapshot_interval: 3600, // 1 hour
            election_timeout_ms: 1500,
            heartbeat_interval_ms: 150,
            data_dir: PathBuf::from("/var/lib/secreton/raft"),
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5432,
            database: "secreton".to_string(),
            username: "secreton".to_string(),
            password: "secreton".to_string(),
            max_connections: 20,
            connection_timeout: 30,
        }
    }
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            // Safe: hardcoded localhost address is always valid
            bind_address: "127.0.0.1:8080"
                .parse()
                .expect("hardcoded localhost address is valid"),
            timeout: Duration::from_secs(30),
            max_body_size: 16 * 1024 * 1024, // 16MB
            keep_alive: Duration::from_secs(75),
            compression: true,
            static_files: None,
        }
    }
}

impl Default for GrpcConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            // Safe: hardcoded localhost address is always valid
            bind_address: "127.0.0.1:9090"
                .parse()
                .expect("hardcoded localhost address is valid"),
            timeout: Duration::from_secs(30),
            max_message_size: 4 * 1024 * 1024, // 4MB
            reflection: false,
            health_check: true,
        }
    }
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: "change-this-secret-in-production".to_string(),
            expiration: Duration::from_secs(3600), // 1 hour
            refresh_expiration: Duration::from_secs(86400 * 7), // 7 days
            algorithm: "HS256".to_string(),
            issuer: "Secreton".to_string(),
            audience: "secreton-api".to_string(),
        }
    }
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(3600), // 1 hour
            store: SessionStore::Memory,
            cookie: CookieConfig::default(),
        }
    }
}

impl Default for CookieConfig {
    fn default() -> Self {
        Self {
            name: "secreton-session".to_string(),
            domain: None,
            path: "/".to_string(),
            secure: false,
            http_only: true,
            same_site: "Strict".to_string(),
        }
    }
}

impl Default for TotpConfig {
    fn default() -> Self {
        Self {
            issuer: "SIMPEL Kejaksaan RI".to_string(),
            secret_length: 32,
            time_step: 30,
            code_length: 6,
            skew_tolerance: 1,
        }
    }
}

impl Default for MfaPoliciesConfig {
    fn default() -> Self {
        Self {
            enforce_for_all: true,
            setup_grace_period: 7,
            max_failed_attempts: 5,
            lockout_duration: 30,
            reauth_interval: 8,
            roles: MfaRolePoliciesConfig::default(),
            satker: MfaSatkerPoliciesConfig::default(),
        }
    }
}

impl Default for MfaRolePoliciesConfig {
    fn default() -> Self {
        Self {
            admin_immediate_setup: vec![
                "AdminPusat".to_string(),
                "AdminEselonI".to_string(),
                "AdminWilayah".to_string(),
                "AdminSatker".to_string(),
            ],
            high_privilege_roles: vec!["AdminPusat".to_string(), "AdminEselonI".to_string()],
            standard_roles: vec!["PegawaiNegeri".to_string(), "PegawaiKontrak".to_string()],
        }
    }
}

impl Default for MfaSatkerPoliciesConfig {
    fn default() -> Self {
        Self {
            high_security_satkers: vec![
                "KEJAKSAAN_AGUNG".to_string(),
                "KEJATI_DKI".to_string(),
                "KEJARI_JAKARTA_PUSAT".to_string(),
            ],
            standard_satkers: vec!["KEJATI_*".to_string(), "KEJARI_*".to_string()],
        }
    }
}

impl Default for MfaRecoveryCodesConfig {
    fn default() -> Self {
        Self {
            count: 10,
            length: 12,
            format: "alphanumeric".to_string(),
            allow_reuse: false,
        }
    }
}

impl Default for MfaRateLimitingConfig {
    fn default() -> Self {
        Self {
            max_attempts_per_minute: 10,
            max_setup_attempts_per_hour: 3,
            window_seconds: 60,
            progressive_delays: true,
        }
    }
}

impl Default for MfaAuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_success: true,
            log_failures: true,
            log_setup_changes: true,
            log_recovery_usage: true,
            retention_days: 365,
        }
    }
}

impl Default for MfaSecurityConfig {
    fn default() -> Self {
        Self {
            secret_encryption: "AES-256-GCM".to_string(),
            kdf: "PBKDF2".to_string(),
            kdf_iterations: 100000,
            salt_length: 32,
            use_hsm: false,
            hsm_slot: 0,
            hsm_pin: None,
        }
    }
}

impl Default for MfaBackupConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_hours: 24,
            retention_days: 90,
            backup_encryption_key: None,
            backup_path: "./backups/mfa".to_string(),
        }
    }
}

impl Default for MfaMonitoringConfig {
    fn default() -> Self {
        Self {
            metrics_enabled: true,
            alert_on_anomalies: true,
            failed_attempts_threshold: 20,
            monitoring_window_minutes: 15,
            alert_email: Some("security@kejaksaan.go.id".to_string()),
            alert_webhook: None,
        }
    }
}

impl Default for MfaComplianceConfig {
    fn default() -> Self {
        Self {
            fips_mode: false,
            audit_integrity_check: true,
            require_admin_approval: true,
            compliance_reporting: true,
            report_interval_hours: 168, // Weekly
        }
    }
}

impl Default for MfaAccessControlConfig {
    fn default() -> Self {
        Self {
            setup_permission: "mfa:setup".to_string(),
            verify_permission: "mfa:verify".to_string(),
            admin_permission: "mfa:admin".to_string(),
            audit_permission: "mfa:audit".to_string(),
        }
    }
}

impl Default for MfaNetworkConfig {
    fn default() -> Self {
        Self {
            allowed_ip_ranges: vec![
                "10.0.0.0/8".to_string(),
                "172.16.0.0/12".to_string(),
                "192.168.0.0/16".to_string(),
            ],
            ip_rate_limiting: true,
            max_requests_per_ip: 20,
        }
    }
}

impl Default for MfaSessionConfig {
    fn default() -> Self {
        Self {
            session_timeout: 15,
            require_mfa_for_extension: true,
            max_concurrent_sessions: 3,
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            global: RateLimitRule {
                requests: 1000,
                window: Duration::from_secs(60),
                burst: Some(100),
            },
            endpoints: vec![],
            per_user: Some(RateLimitRule {
                requests: 100,
                window: Duration::from_secs(60),
                burst: Some(10),
            }),
            per_ip: Some(RateLimitRule {
                requests: 200,
                window: Duration::from_secs(60),
                burst: Some(20),
            }),
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            metrics: true,
            metrics_path: "/metrics".to_string(),
            health_path: "/health".to_string(),
            tracing: true,
            jaeger: None,
        }
    }
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "PATCH".to_string(),
                "OPTIONS".to_string(),
            ],
            allowed_headers: vec![
                "Content-Type".to_string(),
                "Authorization".to_string(),
                "X-Requested-With".to_string(),
            ],
            exposed_headers: vec![],
            max_age: Some(Duration::from_secs(3600)),
            allow_credentials: true,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "pretty".to_string(),
            json: false,
            file: None,
            rotation: None,
        }
    }
}

/// ApiConfig methods for loading and managing configuration
impl ApiConfig {
    /// Apply environment variable overrides
    pub fn apply_env_overrides(&mut self) -> Result<(), String> {
        // HTTP server
        if let Ok(port) = std::env::var("HTTP_PORT") {
            let port: u16 = port.parse().map_err(|_| "Invalid HTTP_PORT".to_string())?;
            let ip = self.http.bind_address.ip();
            self.http.bind_address = SocketAddr::new(ip, port);
        }

        // gRPC server
        if let Ok(port) = std::env::var("GRPC_PORT") {
            let port: u16 = port.parse().map_err(|_| "Invalid GRPC_PORT".to_string())?;
            let ip = self.grpc.bind_address.ip();
            self.grpc.bind_address = SocketAddr::new(ip, port);
        }

        // Log level
        if let Ok(level) = std::env::var("LOG_LEVEL") {
            self.logging.level = level;
        }

        Ok(())
    }
}

#[cfg(all(test, feature = "enable-inline-tests"))]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::time::Duration;

    fn sample_api_config() -> ApiConfig {
        ApiConfig {
            http: HttpConfig {
                bind_address: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8200),
                timeout: Duration::from_secs(30),
                max_body_size: 5 * 1024 * 1024,
                keep_alive: Duration::from_secs(15),
                compression: true,
                static_files: None,
            },
            grpc: GrpcConfig {
                enabled: true,
                bind_address: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8201),
                timeout: Duration::from_secs(30),
                max_message_size: 16 * 1024 * 1024,
                reflection: true,
                health_check: true,
            },
            auth: AuthConfig {
                password_policy: Default::default(),
                jwt: JwtConfig {
                    secret: "super-secret-key".to_string(),
                    expiration: Duration::from_secs(3600),
                    refresh_expiration: Duration::from_secs(86400),
                    algorithm: "HS256".to_string(),
                    issuer: "secreton".to_string(),
                    audience: "engine-users".to_string(),
                },
                oauth2: None,
                mtls: Some(MtlsConfig {
                    required: true,
                    ca_cert: PathBuf::from("/etc/ssl/ca.pem"),
                    allowed_subjects: vec!["CN=trusted".to_string()],
                    crl: None,
                }),
                session: SessionConfig {
                    timeout: Duration::from_secs(1800),
                    store: SessionStore::Memory,
                    cookie: CookieConfig {
                        name: "secreton-session".to_string(),
                        domain: Some("example.com".to_string()),
                        path: "/".to_string(),
                        secure: true,
                        http_only: true,
                        same_site: "Strict".to_string(),
                    },
                },
                mfa: MfaConfig {
                    enabled: true,
                    totp: TotpConfig {
                        issuer: "Secreton Engine".to_string(),
                        secret_length: 32,
                        time_step: 30,
                        code_length: 6,
                        skew_tolerance: 1,
                    },
                    sms: None,
                    email: None,
                    webauthn: Some(WebAuthnConfig {
                        rp_name: "Secreton Engine".to_string(),
                        rp_id: "engine.example.com".to_string(),
                        origin: "https://engine.example.com".to_string(),
                    }),
                    ..Default::default()
                },
            },
            rate_limit: RateLimitConfig {
                enabled: true,
                global: RateLimitRule {
                    requests: 1000,
                    window: Duration::from_secs(60),
                    burst: Some(100),
                },
                endpoints: vec![EndpointRateLimit {
                    pattern: "/v1/auth/login".to_string(),
                    rule: RateLimitRule {
                        requests: 20,
                        window: Duration::from_secs(60),
                        burst: Some(10),
                    },
                }],
                per_user: Some(RateLimitRule {
                    requests: 200,
                    window: Duration::from_secs(60),
                    burst: None,
                }),
                per_ip: None,
            },
            tls: Some(TlsConfig {
                cert_file: PathBuf::from("/etc/tls/server.crt"),
                key_file: PathBuf::from("/etc/tls/server.key"),
                ca_file: None,
                min_version: "TLS1.3".to_string(),
                cipher_suites: vec!["TLS_AES_256_GCM_SHA384".to_string()],
                alpn_protocols: vec!["h2".to_string(), "http/1.1".to_string()],
            }),
            monitoring: MonitoringConfig {
                metrics: true,
                metrics_path: "/metrics".to_string(),
                health_path: "/health".to_string(),
                tracing: true,
                jaeger: Some(JaegerConfig {
                    endpoint: "http://jaeger:14268/api/traces".to_string(),
                    service_name: "secreton-api".to_string(),
                    sample_rate: 0.5,
                }),
            },
            cors: CorsConfig {
                enabled: true,
                allowed_origins: vec!["https://engine.example.com".to_string()],
                allowed_methods: vec!["GET".to_string(), "POST".to_string()],
                allowed_headers: vec!["Authorization".to_string()],
                exposed_headers: vec![],
                max_age: Some(Duration::from_secs(600)),
                allow_credentials: true,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
                json: true,
                file: None,
                rotation: None,
            },
            hsm: HsmConfig::default(),
            database: DatabaseConfig::default(),
            storage: StorageConfig::default(),
        }
    }

    #[test]
    fn test_api_config_structure() {
        let config = sample_api_config();
        assert_eq!(config.http.bind_address.port(), 8200);
        assert!(config.grpc.enabled);
        assert_eq!(config.auth.jwt.algorithm, "HS256");
        assert!(config.auth.mfa.enabled);
        assert!(config.rate_limit.enabled);
        assert!(config.tls.is_some());
        assert!(config.monitoring.metrics);
        assert_eq!(config.cors.allowed_origins.len(), 1);
        assert!(config.logging.json);
    }

    #[test]
    fn test_rate_limit_rule_burst_defaults() {
        let rule = RateLimitRule {
            requests: 50,
            window: Duration::from_secs(10),
            burst: None,
        };
        assert_eq!(rule.requests, 50);
        assert!(rule.burst.is_none());
    }

    #[test]
    fn test_cookie_config_flags() {
        let cookie = CookieConfig {
            name: "session".to_string(),
            domain: None,
            path: "/".to_string(),
            secure: true,
            http_only: true,
            same_site: "Lax".to_string(),
        };

        assert!(cookie.secure);
        assert!(cookie.http_only);
        assert_eq!(cookie.same_site, "Lax");
    }
}
