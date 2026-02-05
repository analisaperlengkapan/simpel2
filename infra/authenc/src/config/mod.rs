/// OpenID Connect configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OidcConfig {
    /// OIDC issuer URL
    pub issuer: String,
    /// OIDC client ID
    pub client_id: String,
    /// OIDC client secret
    pub client_secret: String,
    /// OIDC redirect URI
    pub redirect_uri: String,
}

/// SSO Cookie configuration for secure session management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsoCookieConfig {
    /// Cookie name (default: AUTHENC_SSO)
    #[serde(default = "default_cookie_name")]
    pub name: String,

    /// Cookie domain (e.g., simpel.kejaksaan.go.id)
    pub domain: Option<String>,

    /// Cookie path (default: /)
    #[serde(default = "default_cookie_path")]
    pub path: String,

    /// Cookie max age in seconds (default: 3600 = 1 hour)
    #[serde(default = "default_cookie_max_age")]
    pub max_age: i64,

    /// Enable Secure flag (HTTPS only)
    #[serde(default = "default_cookie_secure")]
    pub secure: bool,

    /// Enable HttpOnly flag (prevent JavaScript access)
    #[serde(default = "default_cookie_http_only")]
    pub http_only: bool,

    /// SameSite policy (Lax, Strict, None)
    #[serde(default = "default_cookie_same_site")]
    pub same_site: String,
}

impl Default for SsoCookieConfig {
    fn default() -> Self {
        Self {
            name: default_cookie_name(),
            domain: None,
            path: default_cookie_path(),
            max_age: default_cookie_max_age(),
            secure: default_cookie_secure(),
            http_only: default_cookie_http_only(),
            same_site: default_cookie_same_site(),
        }
    }
}

fn default_cookie_name() -> String {
    "AUTHENC_SSO".to_string()
}

fn default_cookie_path() -> String {
    "/".to_string()
}

fn default_cookie_max_age() -> i64 {
    3600 // 1 hour
}

fn default_cookie_secure() -> bool {
    true // Always use Secure flag for production
}

fn default_cookie_http_only() -> bool {
    true // Prevent XSS attacks
}

fn default_cookie_same_site() -> String {
    "Lax".to_string() // Balance security and usability
}

/// SAML configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SamlConfig {
    /// SAML entity ID
    pub entity_id: String,
    /// SAML SSO URL
    pub sso_url: String,
    /// SAML certificate
    pub certificate: String,
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UiConfig {
    /// Whether the UI is enabled
    pub enabled: bool,
    /// Optional UI theme
    pub theme: Option<String>,
}

/// Multi-database configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MultiDbConfig {
    /// Whether multi-database support is enabled
    pub enabled: bool,
    /// List of database URLs for multi-database setup
    pub db_urls: Vec<String>,
}

/// Secreton configuration for external secret management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretonConfig {
    /// Whether Secreton integration is enabled
    #[serde(default)]
    pub enabled: bool,
    /// Secreton service endpoint URL
    #[serde(alias = "url", default)]
    pub endpoint: String,
    /// Authentication token for Secreton service
    #[serde(default)]
    pub token: String,
    /// Mount path in Secreton
    #[serde(default)]
    pub mount_path: String,
    /// Key rotation interval in seconds
    #[serde(default)]
    pub key_rotation_interval: u64,
    /// List of secrets to load
    #[serde(default)]
    pub secrets_to_load: Vec<String>,
}

/// Redis configuration for caching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    /// Whether Redis caching is enabled
    pub enabled: bool,
    /// Redis connection URL (redis://host:port/db)
    pub url: String,
    /// Connection pool size
    #[serde(default = "default_redis_pool_size")]
    pub pool_size: u32,
    /// Connection timeout in seconds
    #[serde(default = "default_redis_timeout")]
    pub connection_timeout: u64,
    /// Default TTL for cached items in seconds
    #[serde(default = "default_redis_ttl")]
    pub default_ttl: u64,
    /// MFA cache TTL in seconds
    #[serde(default = "default_mfa_cache_ttl")]
    pub mfa_cache_ttl: u64,
    /// OTP verification cache TTL in seconds (for replay protection)
    #[serde(default = "default_otp_cache_ttl")]
    pub otp_verification_ttl: u64,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            url: "redis://localhost:6379/0".to_string(),
            pool_size: default_redis_pool_size(),
            connection_timeout: default_redis_timeout(),
            default_ttl: default_redis_ttl(),
            mfa_cache_ttl: default_mfa_cache_ttl(),
            otp_verification_ttl: default_otp_cache_ttl(),
        }
    }
}

fn default_redis_pool_size() -> u32 {
    10
}
fn default_redis_timeout() -> u64 {
    5
}
fn default_redis_ttl() -> u64 {
    3600 // 1 hour
}
fn default_mfa_cache_ttl() -> u64 {
    300 // 5 minutes
}
fn default_otp_cache_ttl() -> u64 {
    90 // 1.5 minutes (3 TOTP windows)
}

/// Kafka configuration for audit log streaming
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KafkaConfig {
    /// Whether Kafka audit logging is enabled
    pub enabled: bool,
    /// Kafka broker addresses (comma-separated)
    pub brokers: String,
    /// Kafka topic for audit logs
    pub audit_topic: String,
    /// Kafka topic for user events
    pub user_events_topic: String,
    /// Kafka topic for admin events
    pub admin_events_topic: String,
    /// Client ID for Kafka producer
    pub client_id: Option<String>,
    /// Message timeout in milliseconds
    pub message_timeout_ms: Option<u32>,
    /// Compression type (none, gzip, snappy, lz4, zstd)
    pub compression: Option<String>,
}

/// Event retention and lifecycle configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventsConfig {
    /// Whether event retention is enabled
    pub enabled: bool,
    /// Retention period for user events in days (default: 90)
    #[serde(default = "default_user_event_retention_days")]
    pub user_event_retention_days: u32,
    /// Retention period for admin events in days (default: 365)
    #[serde(default = "default_admin_event_retention_days")]
    pub admin_event_retention_days: u32,
    /// Maximum number of events to delete in a single cleanup batch (default: 10000)
    #[serde(default = "default_max_cleanup_batch_size")]
    pub max_cleanup_batch_size: u32,
    /// Cleanup interval in hours (default: 24)
    #[serde(default = "default_cleanup_interval_hours")]
    pub cleanup_interval_hours: u32,
    /// Whether to archive events before deletion (default: false)
    pub archive_before_delete: bool,
    /// Archive directory path (if archiving is enabled)
    pub archive_directory: Option<String>,
    /// Cold storage configuration for event archiving
    pub cold_storage: Option<ColdStorageConfig>,
}

impl Default for EventsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            user_event_retention_days: default_user_event_retention_days(),
            admin_event_retention_days: default_admin_event_retention_days(),
            max_cleanup_batch_size: default_max_cleanup_batch_size(),
            cleanup_interval_hours: default_cleanup_interval_hours(),
            archive_before_delete: false,
            archive_directory: None,
            cold_storage: None,
        }
    }
}

/// Cold storage configuration for event archiving (S3/MinIO)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColdStorageConfig {
    /// Whether cold storage is enabled
    #[serde(default)]
    pub enabled: bool,
    /// Storage type (s3 or minio)
    #[serde(default = "default_storage_type")]
    pub storage_type: String,
    /// S3/MinIO endpoint URL
    pub endpoint: String,
    /// S3/MinIO region
    #[serde(default = "default_region")]
    pub region: String,
    /// S3/MinIO bucket name for archived events
    pub bucket: String,
    /// Access key ID
    pub access_key_id: Option<String>,
    /// Secret access key
    pub secret_access_key: Option<String>,
    /// Path prefix for archived events
    #[serde(default = "default_path_prefix")]
    pub path_prefix: String,
    /// Whether to use path-style addressing (for MinIO)
    #[serde(default = "default_path_style")]
    pub force_path_style: bool,
}

impl Default for ColdStorageConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            storage_type: default_storage_type(),
            endpoint: String::new(),
            region: default_region(),
            bucket: String::new(),
            access_key_id: None,
            secret_access_key: None,
            path_prefix: default_path_prefix(),
            force_path_style: default_path_style(),
        }
    }
}

fn default_storage_type() -> String {
    "s3".to_string()
}

fn default_region() -> String {
    "us-east-1".to_string()
}

fn default_path_prefix() -> String {
    "authenc/events/archive".to_string()
}

fn default_path_style() -> bool {
    false
}

fn default_user_event_retention_days() -> u32 {
    90
}
fn default_admin_event_retention_days() -> u32 {
    365
}
fn default_max_cleanup_batch_size() -> u32 {
    10000
}
fn default_cleanup_interval_hours() -> u32 {
    24
}

/// SPI provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpiProviderConfig {
    /// Provider ID
    pub id: String,
    /// Whether the provider is enabled
    #[serde(default = "default_provider_enabled")]
    pub enabled: bool,
    /// Provider priority (higher values = higher priority)
    #[serde(default)]
    pub priority: i64,
    /// Provider-specific configuration
    #[serde(default)]
    pub config: serde_json::Value,
}

fn default_provider_enabled() -> bool {
    true
}

/// SPI configuration for enterprise features
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpiConfig {
    /// Organization SPI providers
    #[serde(default)]
    pub organization: Vec<SpiProviderConfig>,
    /// Rich Authorization SPI providers
    #[serde(default)]
    pub rich_authorization: Vec<SpiProviderConfig>,
    /// Migration SPI providers
    #[serde(default)]
    pub migration: Vec<SpiProviderConfig>,
    /// Hostname SPI providers
    #[serde(default)]
    pub hostname: Vec<SpiProviderConfig>,
}

use std::{env, net::SocketAddr, path::PathBuf, time::Duration};

use serde::{Deserialize, Serialize};
use tracing::Level;

use crate::error::{AuthencError, Result};
use crate::middleware::adaptive_rate_limit::AdaptiveRateLimitConfig;
use crate::middleware::rate_limit::RateLimitConfig;

/// Dynamic configuration management for adaptive security and performance
/// This module provides runtime configuration adjustment based on system load,
/// threat levels, and performance metrics. It enables the system to adapt
/// security posture and resource allocation dynamically.
pub mod dynamic;
pub use dynamic::{
    CryptoMode, DynamicConfig, DynamicConfigManager, LoadMetrics, PerformanceProfile,
    PerformanceProfiler, ThreatLevel,
};

/// MFA fallback configuration for local encrypted storage
pub mod mfa_fallback;
pub use mfa_fallback::MfaFallbackConfig;

/// Hybrid configuration loader for multi-source configuration
pub mod hybrid_loader;
pub use hybrid_loader::{ConfigLoaderConfig, HybridConfigLoader};

/// Cluster configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    /// Whether clustering is enabled
    #[serde(default)]
    pub enabled: bool,
    /// Name of the cluster
    #[serde(default = "default_cluster_name")]
    pub cluster_name: String,
    /// Unique identifier for this node
    pub node_id: Option<String>,
    /// Type of cluster communication to use
    #[serde(default)]
    pub communication_type: crate::services::clustering::ClusterCommunicationType,
    /// Type of cluster membership management
    #[serde(default)]
    pub membership_type: crate::services::clustering::ClusterMembershipType,
    /// Type of distributed consensus algorithm
    #[serde(default)]
    pub consensus_type: crate::services::clustering::ClusterConsensusType,
    /// Addresses for service discovery
    #[serde(default)]
    pub discovery_addresses: Vec<String>,
    /// Whether session replication is enabled
    #[serde(default = "default_true")]
    pub session_replication_enabled: bool,
    /// Whether cache replication is enabled
    #[serde(default = "default_true")]
    pub cache_replication_enabled: bool,
}

impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cluster_name: default_cluster_name(),
            node_id: None,
            communication_type: crate::services::clustering::ClusterCommunicationType::Infinispan,
            membership_type: crate::services::clustering::ClusterMembershipType::Kubernetes,
            consensus_type: crate::services::clustering::ClusterConsensusType::Raft,
            discovery_addresses: vec![],
            session_replication_enabled: true,
            cache_replication_enabled: true,
        }
    }
}

fn default_cluster_name() -> String {
    "authenc-cluster".to_string()
}

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    /// Server configuration
    /// Server configuration settings
    pub server: ServerConfig,

    /// Database configuration
    /// Database connection configuration
    pub database: DatabaseConfig,

    /// Rate limiting configuration
    pub rate_limit: RateLimitConfig,

    /// Adaptive rate limiting configuration
    pub adaptive_rate_limit: AdaptiveRateLimitConfig,

    /// MFA-specific rate limiting configuration
    pub mfa_rate_limit: crate::middleware::MfaRateLimitConfig,

    /// Security-related configuration
    pub security: BasicSecurityConfig,

    /// Observability configuration (logging, metrics, etc.)
    pub observability: ObservabilityConfig,

    /// Feature flags and settings
    pub features: FeatureConfig,

    /// Optional OIDC configuration
    pub oidc: Option<OidcConfig>,

    /// Optional SAML configuration
    pub saml: Option<SamlConfig>,

    /// SSO Cookie configuration
    #[serde(default)]
    pub sso_cookie: SsoCookieConfig,

    /// UI configuration
    pub ui: Option<UiConfig>,

    /// Multi-database configuration (if enabled)
    pub multi_db: Option<MultiDbConfig>,

    /// Secret management configuration (if using external secret management)
    pub secreton: Option<SecretonConfig>,

    /// Redis configuration for caching
    pub redis: Option<RedisConfig>,

    /// Kafka configuration for audit log streaming
    pub kafka: Option<KafkaConfig>,

    /// Event retention and lifecycle configuration
    pub events: EventsConfig,

    /// SPI configuration for enterprise features
    #[serde(default)]
    pub spi: SpiConfig,

    /// Clustering configuration for high availability
    #[serde(default)]
    pub clustering: ClusterConfig,

    /// Key rotation configuration for automatic key rotation
    pub key_rotation: Option<crate::services::key_rotation::KeyRotationConfig>,

    /// Federation configuration for user federation
    pub federation: Option<FederationConfig>,

    /// Configuration loader settings (for hybrid config approach)
    #[serde(default)]
    pub config_loader: crate::config::hybrid_loader::ConfigLoaderConfig,
}

/// Backwards-compatibility alias for older tests and integrations
pub type AuthencConfig = AppConfig;

/// Federation configuration for LDAP/AD and social login
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationConfig {
    /// Enable user synchronization
    #[serde(default)]
    pub sync_enabled: bool,

    /// Sync interval in minutes (default: 60)
    pub sync_interval_minutes: Option<u64>,

    /// Batch size for sync operations (default: 100)
    #[serde(default = "default_sync_batch_size")]
    pub batch_size: usize,
}

impl Default for FederationConfig {
    fn default() -> Self {
        Self {
            sync_enabled: false,
            sync_interval_minutes: Some(60),
            batch_size: default_sync_batch_size(),
        }
    }
}

fn default_sync_batch_size() -> usize {
    100
}

// Re-export common config types
pub use lib_common::config::{DatabaseConfig, ServerConfig};

// Shim functions for defaults if they are still needed by other modules,
// though lib_common::config types implement Default.
//
// Note: We might need to keep specific default functions if they differ from lib_common,
// but for migration we will try to rely on lib_common.

// Removed: local ServerConfig and DatabaseConfig definitions.

/// Security configuration for the authentication platform
/// This struct contains all security-related configuration parameters for the
/// authentication platform, including JWT settings, password policies, rate limiting,
/// and brute force protection. All fields have sensible defaults and can be
/// configured via environment variables or configuration files.
/// # Security Considerations
/// - JWT secrets should be cryptographically secure random values
/// - Password policies should follow industry best practices
/// - Rate limiting helps prevent DoS attacks
/// - Brute force protection prevents credential stuffing attacks
/// - All timeouts and limits should be tuned for your security requirements
/// # Example
/// ```rust
/// use authenc::config::BasicSecurityConfig;
/// let config = BasicSecurityConfig {
///     jwt_secret: "your-secure-jwt-secret".to_string(),
///     jwt_expiry: 3600, // 1 hour
///     password_min_length: 12,
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicSecurityConfig {
    /// Secret key for JWT signing and validation
    pub jwt_secret: String,

    /// Path to JWT signing key in Secreton
    pub jwt_secret_path: Option<String>,

    /// JWT token expiration time in seconds
    #[serde(default = "default_jwt_expiry")]
    pub jwt_expiry: u64,

    /// Minimum password length requirement
    #[serde(default = "default_password_min_length")]
    pub password_min_length: u8,

    /// Number of requests allowed per rate limit window
    #[serde(default = "default_rate_limit_requests")]
    pub rate_limit_requests: u32,

    /// Rate limit window in seconds
    #[serde(default = "default_rate_limit_window")]
    pub rate_limit_window: u64,

    /// Maximum number of failed login attempts before account lockout
    #[serde(default = "default_brute_force_max_attempts")]
    pub brute_force_max_attempts: u32,

    /// Brute force detection window in seconds
    #[serde(default = "default_brute_force_window")]
    pub brute_force_window_seconds: u64,

    /// Number of requests allowed per minute (global rate limit)
    #[serde(default = "default_rate_limit_per_minute")]
    pub rate_limit_requests_per_minute: u32,

    /// Number of salt rounds for password hashing
    #[serde(default = "default_password_salt_rounds")]
    pub password_salt_rounds: u32,
}

impl Default for BasicSecurityConfig {
    fn default() -> Self {
        Self {
            jwt_secret: "default_jwt_secret_change_in_production".to_string(),
            jwt_secret_path: None,
            jwt_expiry: default_jwt_expiry(),
            password_min_length: default_password_min_length(),
            rate_limit_requests: default_rate_limit_requests(),
            rate_limit_window: default_rate_limit_window(),
            brute_force_max_attempts: default_brute_force_max_attempts(),
            brute_force_window_seconds: default_brute_force_window(),
            rate_limit_requests_per_minute: default_rate_limit_per_minute(),
            password_salt_rounds: default_password_salt_rounds(),
        }
    }
}

/// Observability configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    /// Log level (trace, debug, info, warn, error)
    #[serde(with = "log_level_serde")]
    pub log_level: tracing::Level,

    /// Enable metrics collection
    #[serde(default = "default_true")]
    pub enable_metrics: bool,

    /// Endpoint for metrics (default: /metrics)
    #[serde(default = "default_metrics_endpoint")]
    pub metrics_endpoint: String,

    /// Enable distributed tracing
    #[serde(default = "default_true")]
    pub enable_tracing: bool,

    /// Enable structured logging (JSON format)
    #[serde(default = "default_true")]
    pub structured_logging: bool,

    /// Optional path to log file (if not set, logs to stderr)
    pub log_file: Option<String>,

    /// Port for metrics server
    #[serde(default = "default_metrics_port")]
    pub metrics_port: u16,
}

/// Feature flags and settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureConfig {
    /// Enable user registration
    #[serde(default = "default_true")]
    pub enable_registration: bool,

    /// Enable password reset functionality
    #[serde(default = "default_true")]
    pub enable_password_reset: bool,

    /// Require email verification for new accounts
    #[serde(default = "default_false")]
    pub enable_email_verification: bool,

    /// Enable multi-factor authentication
    #[serde(default = "default_false")]
    pub enable_multi_factor_auth: bool,

    /// Enable API documentation (OpenAPI/Swagger)
    #[serde(default = "default_true")]
    pub enable_api_docs: bool,

    /// Enable metrics endpoint
    #[serde(default = "default_true")]
    pub enable_metrics: bool,

    /// Enable health check endpoints
    #[serde(default = "default_true")]
    pub enable_health_checks: bool,

    /// Enable rate limiting
    #[serde(default = "default_true")]
    pub enable_rate_limiting: bool,

    /// Enable response caching
    #[serde(default = "default_true")]
    pub enable_caching: bool,

    /// Enable response compression
    #[serde(default = "default_true")]
    pub enable_compression: bool,

    /// Enable CORS
    #[serde(default = "default_true")]
    pub enable_cors: bool,

    /// Enable input validation middleware
    #[serde(default = "default_true")]
    pub enable_input_validation: bool,
}

impl AppConfig {
    /// Load configuration with hierarchy: default.toml → production.toml → env vars
    /// This is the recommended way to load configuration for production deployments.
    pub fn load() -> Result<Self> {
        // 1. Try to load from authenc.toml first (primary config file)
        let mut config = if PathBuf::from("authenc.toml").exists() {
            Self::from_file("authenc.toml")?
        } else if PathBuf::from("/app/authenc.toml").exists() {
            Self::from_file("/app/authenc.toml")?
        } else if PathBuf::from("config/default.toml").exists() {
            Self::from_file("config/default.toml")?
        } else {
            Self::default()
        };

        // 2. Override with environment-specific config (production.toml)
        let env = env::var("AUTHENC_ENV").unwrap_or_else(|_| "production".to_string());
        let env_config_path = format!("config/{}.toml", env);
        if PathBuf::from(&env_config_path).exists() {
            let env_config = Self::from_file(&env_config_path)?;
            config.merge(env_config);
        }

        // 3. Override with environment variables (for secrets)
        config.apply_env_overrides()?;

        // 4. Validate final configuration
        config.validate()?;

        Ok(config)
    }

    /// Load configuration from a TOML file
    pub fn from_file(path: &str) -> Result<Self> {
        let contents = std::fs::read_to_string(path).map_err(|e| {
            AuthencError::validation(&format!("Failed to read config file {}: {}", path, e))
        })?;

        toml::from_str(&contents).map_err(|e| {
            AuthencError::validation(&format!("Failed to parse config file {}: {}", path, e))
        })
    }

    /// Merge another config into this one (other config takes precedence)
    fn merge(&mut self, other: Self) {
        // Server config
        if other.server.host != ServerConfig::default().host {
            self.server.host = other.server.host;
        }
        if other.server.port != ServerConfig::default().port {
            self.server.port = other.server.port;
        }
        if other.server.workers.is_some() {
            self.server.workers = other.server.workers;
        }

        // Database config (always override if different from defaults)
        if other.database.host != "localhost" {
            self.database.host = other.database.host;
        }
        if other.database.port != 5432 {
            self.database.port = other.database.port;
        }
        if other.database.database != "authenc" {
            self.database.database = other.database.database;
        }
        if other.database.username != "postgres" {
            self.database.username = other.database.username;
        }
        if other.database.password != "postgres" {
            self.database.password = other.database.password;
        }

        // Security config
        if other.security.jwt_secret != "default_jwt_secret_change_in_production"
            && !other.security.jwt_secret.is_empty()
        {
            self.security.jwt_secret = other.security.jwt_secret;
        }

        // Merge other fields
        self.observability = other.observability;
        self.features = other.features;
        self.rate_limit = other.rate_limit;
        self.adaptive_rate_limit = other.adaptive_rate_limit;
        self.mfa_rate_limit = other.mfa_rate_limit;
    }

    /// Apply environment variable overrides (for secrets and runtime config)
    fn apply_env_overrides(&mut self) -> Result<()> {
        // Server configuration
        if let Ok(host) = env::var("HOST") {
            self.server.host = host;
        }

        if let Ok(port) = env::var("PORT") {
            self.server.port = port
                .parse()
                .map_err(|_| AuthencError::validation("Invalid PORT"))?;
        }

        if let Ok(workers) = env::var("WORKERS") {
            self.server.workers = Some(
                workers
                    .parse()
                    .map_err(|_| AuthencError::validation("Invalid WORKERS"))?,
            );
        }

        // TLS configuration
        if let Ok(tls_enabled) = env::var("TLS_ENABLED") {
            self.server.tls_enabled = tls_enabled.parse().unwrap_or(false);
        }

        if let Ok(cert_path) = env::var("TLS_CERT_PATH") {
            self.server.tls_cert_path = Some(cert_path);
        }

        if let Ok(key_path) = env::var("TLS_KEY_PATH") {
            self.server.tls_key_path = Some(key_path);
        }

        // Database configuration
        if let Ok(db_url) = env::var("DATABASE_URL") {
            // Parse database URL if provided
            // Format: postgres://username:password@host:port/database
            if let Ok(url) = url::Url::parse(&db_url) {
                if let Some(host) = url.host_str() {
                    self.database.host = host.to_string();
                }
                if let Some(port) = url.port() {
                    self.database.port = port;
                }
                if !url.username().is_empty() {
                    self.database.username = url.username().to_string();
                }
                if let Some(password) = url.password() {
                    self.database.password = password.to_string();
                }
                if let Some(mut segments) = url.path_segments() {
                    if let Some(db) = segments.next() {
                        self.database.database = db.trim_start_matches('/').to_string();
                    }
                }
            }
        }

        // Individual database environment variables (fallback)
        if let Ok(host) = env::var("DB_HOST") {
            self.database.host = host;
        }
        if let Ok(port) = env::var("DB_PORT") {
            if let Ok(p) = port.parse() {
                self.database.port = p;
            }
        }
        if let Ok(user) = env::var("DB_USER") {
            self.database.username = user;
        }
        if let Ok(password) = env::var("DB_PASSWORD") {
            self.database.password = password;
        }
        if let Ok(name) = env::var("DB_NAME") {
            self.database.database = name;
        }

        // Security configuration
        if let Ok(secret) = env::var("JWT_SECRET") {
            self.security.jwt_secret = secret;
        }

        if let Ok(allow_origins) = env::var("CORS_ALLOWED_ORIGINS") {
            self.server.cors_allowed_origins = allow_origins
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
        }

        // Observability configuration
        if let Ok(log_level) = env::var("LOG_LEVEL") {
            if let Ok(level) = log_level.parse::<Level>() {
                self.observability.log_level = level;
            }
        }

        // Feature flags
        if let Ok(features) = env::var("ENABLED_FEATURES") {
            for feature in features.split(',') {
                match feature.trim() {
                    "api_docs" => self.features.enable_api_docs = true,
                    "metrics" => self.features.enable_metrics = true,
                    "health_checks" => self.features.enable_health_checks = true,
                    _ => {}
                }
            }
        }

        // Redis configuration
        if let Ok(redis_url) = env::var("REDIS_URL") {
            // Prioritize full URL if provided directly
            if self.redis.is_none() {
                self.redis = Some(RedisConfig::default());
            }
            if let Some(redis_config) = &mut self.redis {
                redis_config.url = redis_url;
                redis_config.enabled = true;
            }
        } else {
            // Fallback to individual env vars for safer URL construction
            let redis_host = env::var("REDIS_HOST").ok();
            let redis_port = env::var("REDIS_PORT").ok();
            let redis_password = env::var("REDIS_PASSWORD").ok();

            if redis_host.is_some() || redis_port.is_some() || redis_password.is_some() {
                let host = redis_host.unwrap_or_else(|| "localhost".to_string());
                let port = redis_port.unwrap_or_else(|| "6379".to_string());

                let url = if let Some(password) = redis_password {
                    if !password.is_empty() {
                        let encoded_password =
                            url::form_urlencoded::byte_serialize(password.as_bytes())
                                .collect::<String>();
                        format!("redis://:{}@{}:{}/0", encoded_password, host, port)
                    } else {
                        format!("redis://{}:{}/0", host, port)
                    }
                } else {
                    format!("redis://{}:{}/0", host, port)
                };

                if self.redis.is_none() {
                    self.redis = Some(RedisConfig::default());
                }

                if let Some(redis_config) = &mut self.redis {
                    redis_config.url = url;
                    redis_config.enabled = true;
                }
            }
        }

        Ok(())
    }

    /// Load configuration from environment variables only (legacy method)
    /// This method is kept for backward compatibility.
    /// Use `load()` instead for production deployments.
    #[deprecated(
        since = "0.2.0",
        note = "Use `load()` instead for file-based config with env overrides"
    )]
    #[allow(deprecated)]
    pub fn from_env() -> Result<Self> {
        let mut config = Self::default();
        config.apply_env_overrides()?;
        config.validate()?;
        Ok(config)
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate server configuration
        if self.server.port == 0 {
            return Err(AuthencError::validation("Server port cannot be 0"));
        }

        // Validate TLS configuration if enabled
        if self.server.tls_enabled {
            if self.server.tls_cert_path.is_none() || self.server.tls_key_path.is_none() {
                return Err(AuthencError::validation(
                    "TLS certificate and key paths are required when TLS is enabled",
                ));
            }

            // Check if certificate and key files exist
            if let (Some(cert_path), Some(key_path)) =
                (&self.server.tls_cert_path, &self.server.tls_key_path)
            {
                if !PathBuf::from(cert_path).exists() {
                    return Err(AuthencError::validation("TLS certificate file not found"));
                }
                if !PathBuf::from(key_path).exists() {
                    return Err(AuthencError::validation("TLS key file not found"));
                }
            }
        }

        // Validate database configuration
        if self.database.host.is_empty() {
            return Err(AuthencError::validation("Database host cannot be empty"));
        }

        if self.database.database.is_empty() {
            return Err(AuthencError::validation("Database name cannot be empty"));
        }

        // Validate security configuration
        if self.security.jwt_secret.is_empty() {
            return Err(AuthencError::validation("JWT secret cannot be empty"));
        }

        if self.security.password_min_length < 8 {
            return Err(AuthencError::validation(
                "Password minimum length must be at least 8 characters",
            ));
        }

        Ok(())
    }

    /// Get the server socket address
    pub fn server_addr(&self) -> SocketAddr {
        format!("{}:{}", self.server.host, self.server.port)
            .parse()
            .expect("Invalid server address")
    }

    /// Get the database connection string
    pub fn database_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.database.username,
            self.database.password,
            self.database.host,
            self.database.port,
            self.database.database
        )
    }

    /// Get the keep-alive duration
    pub fn keep_alive(&self) -> Duration {
        Duration::from_secs(self.server.keep_alive)
    }

    /// Get the client timeout duration
    pub fn client_timeout(&self) -> Duration {
        Duration::from_secs(self.server.client_timeout)
    }

    /// Get the client disconnect timeout duration
    pub fn client_disconnect_timeout(&self) -> Duration {
        Duration::from_secs(self.server.client_disconnect_timeout)
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            database: DatabaseConfig {
                database: "authenc".to_string(),
                ..DatabaseConfig::default()
            },
            security: BasicSecurityConfig {
                jwt_secret: env::var("JWT_SECRET")
                    .unwrap_or_else(|_| "default_jwt_secret_change_in_production".to_string()),
                jwt_secret_path: None,
                jwt_expiry: default_jwt_expiry(),
                password_min_length: default_password_min_length(),
                rate_limit_requests: default_rate_limit_requests(),
                rate_limit_window: default_rate_limit_window(),
                brute_force_max_attempts: default_brute_force_max_attempts(),
                brute_force_window_seconds: default_brute_force_window(),
                rate_limit_requests_per_minute: default_rate_limit_per_minute(),
                password_salt_rounds: default_password_salt_rounds(),
            },
            observability: ObservabilityConfig {
                log_level: default_log_level(),
                enable_metrics: true,
                metrics_endpoint: default_metrics_endpoint(),
                enable_tracing: true,
                structured_logging: true,
                log_file: None,
                metrics_port: default_metrics_port(),
            },
            features: FeatureConfig {
                enable_registration: true,
                enable_password_reset: true,
                enable_email_verification: false,
                enable_multi_factor_auth: false,
                enable_api_docs: true,
                enable_metrics: true,
                enable_health_checks: true,
                enable_rate_limiting: true,
                enable_caching: true,
                enable_compression: true,
                enable_cors: true,
                enable_input_validation: true,
            },
            rate_limit: RateLimitConfig::default(),
            adaptive_rate_limit: AdaptiveRateLimitConfig::default(),
            mfa_rate_limit: crate::middleware::MfaRateLimitConfig::default(),
            oidc: None,
            saml: None,
            sso_cookie: SsoCookieConfig::default(),
            ui: None,
            multi_db: None,
            secreton: None,
            redis: None,
            kafka: None,
            events: EventsConfig::default(),
            spi: SpiConfig::default(),
            clustering: ClusterConfig::default(),
            key_rotation: None,
            federation: None,
            config_loader: ConfigLoaderConfig::default(),
        }
    }
}

// Default value helpers
fn default_true() -> bool {
    true
}
fn default_false() -> bool {
    false
}

fn default_log_level() -> Level {
    Level::INFO
}
fn default_metrics_endpoint() -> String {
    "/metrics".to_string()
}
fn default_metrics_port() -> u16 {
    9090
}

fn default_jwt_expiry() -> u64 {
    3600
} // 1 hour
fn default_password_min_length() -> u8 {
    8
}
fn default_rate_limit_requests() -> u32 {
    100
}
fn default_rate_limit_window() -> u64 {
    60
} // 1 minute
fn default_brute_force_max_attempts() -> u32 {
    5
}
fn default_brute_force_window() -> u64 {
    300
} // 5 minutes
fn default_rate_limit_per_minute() -> u32 {
    60
}
fn default_password_salt_rounds() -> u32 {
    10
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.database.host, "localhost");
        assert_eq!(config.database.port, 5432);
    }

    #[test]
    #[serial]
    fn test_from_env() {
        temp_env::with_vars(
            vec![
                ("HOST", Some("127.0.0.1")),
                ("PORT", Some("4000")),
                ("JWT_SECRET", Some("test_secret")),
            ],
            || {
                #[allow(deprecated)]
                let config = AppConfig::from_env().unwrap();
                assert_eq!(config.server.host, "127.0.0.1");
                assert_eq!(config.server.port, 4000);
                assert_eq!(config.security.jwt_secret, "test_secret");
            },
        );
    }

    #[test]
    #[serial]
    fn test_validation() {
        let mut config = AppConfig::default();

        // Test valid config
        assert!(config.validate().is_ok());

        // Test invalid port
        config.server.port = 0;
        assert!(config.validate().is_err());
        config.server.port = 3000;

        // Test empty database host
        let old_host = config.database.host.clone();
        config.database.host = String::new();
        assert!(config.validate().is_err());
        config.database.host = old_host;

        // Test empty JWT secret
        let old_secret = config.security.jwt_secret.clone();
        config.security.jwt_secret = String::new();
        assert!(config.validate().is_err());
        config.security.jwt_secret = old_secret;
    }
}

/// Serde module for log level serialization
mod log_level_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use tracing::Level;

    pub fn serialize<S>(level: &Level, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let level_str = match *level {
            Level::TRACE => "trace",
            Level::DEBUG => "debug",
            Level::INFO => "info",
            Level::WARN => "warn",
            Level::ERROR => "error",
        };
        level_str.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Level, D::Error>
    where
        D: Deserializer<'de>,
    {
        let level_str = String::deserialize(deserializer)?;
        match level_str.to_lowercase().as_str() {
            "trace" => Ok(Level::TRACE),
            "debug" => Ok(Level::DEBUG),
            "info" => Ok(Level::INFO),
            "warn" => Ok(Level::WARN),
            "error" => Ok(Level::ERROR),
            _ => Err(serde::de::Error::custom(format!(
                "Invalid log level: {}",
                level_str
            ))),
        }
    }
}

// Security Configuration Module
//
// This module provides a centralized configuration system for all security-related
// middleware and features in the Authenc system. It allows for easy configuration
// and management of security settings across the application.

use crate::middleware::*;

/// Comprehensive security configuration for the Authenc system
#[derive(Clone, Debug)]
pub struct SecurityMiddlewareConfig {
    /// Security headers configuration
    pub headers: SecurityHeadersConfig,
    /// CSRF protection configuration
    pub csrf: CsrfConfig,
    /// Rate limiting configuration
    pub rate_limit: RateLimitConfig,
    /// Input validation configuration
    pub input_validation: InputValidationConfig,
    /// Security monitoring configuration
    pub monitoring: SecurityMonitoringConfig,
}

impl SecurityMiddlewareConfig {
    /// Create a new security configuration with default secure settings
    pub fn secure_defaults() -> Self {
        Self {
            headers: SecurityHeadersConfig::secure(),
            csrf: CsrfConfig::default(),
            rate_limit: RateLimitConfig::default(),
            input_validation: InputValidationConfig::default(),
            monitoring: SecurityMonitoringConfig::default(),
        }
    }

    /// Create a new security configuration with development-friendly settings
    pub fn development_defaults() -> Self {
        Self {
            headers: SecurityHeadersConfig::development(),
            csrf: CsrfConfig {
                enabled: false, // Disable CSRF in development for easier testing
                ..CsrfConfig::default()
            },
            rate_limit: RateLimitConfig::default(),
            input_validation: InputValidationConfig::default(),
            monitoring: SecurityMonitoringConfig::default(),
        }
    }
}

impl Default for SecurityMiddlewareConfig {
    fn default() -> Self {
        Self::secure_defaults()
    }
}

/// Enhanced security headers configuration
#[derive(Clone, Debug)]
pub struct SecurityHeadersConfig {
    /// Whether to enable enhanced security headers
    pub enabled: bool,
    /// HSTS max age in seconds
    pub hsts_max_age: u32,
    /// Whether to include subdomains in HSTS
    pub hsts_include_subdomains: bool,
    /// Whether to enable HSTS preload
    pub hsts_preload: bool,
    /// Content Security Policy directives
    pub csp_directives: Vec<String>,
}

impl SecurityHeadersConfig {
    /// Secure defaults for production
    pub fn secure() -> Self {
        Self {
            enabled: true,
            hsts_max_age: 31536000, // 1 year
            hsts_include_subdomains: true,
            hsts_preload: true,
            csp_directives: vec![
                "default-src 'self'".to_string(),
                "script-src 'self' 'unsafe-inline'".to_string(),
                "style-src 'self' 'unsafe-inline'".to_string(),
                "img-src 'self' data: https:".to_string(),
                "font-src 'self' data:".to_string(),
                "connect-src 'self'".to_string(),
                "media-src 'none'".to_string(),
                "object-src 'none'".to_string(),
                "frame-src 'none'".to_string(),
                "frame-ancestors 'none'".to_string(),
                "form-action 'self'".to_string(),
                "upgrade-insecure-requests".to_string(),
                "block-all-mixed-content".to_string(),
            ],
        }
    }

    /// Development-friendly settings
    pub fn development() -> Self {
        Self {
            enabled: true,
            hsts_max_age: 0, // Disable HSTS in development
            hsts_include_subdomains: false,
            hsts_preload: false,
            csp_directives: vec![
                "default-src 'self' 'unsafe-inline' 'unsafe-eval'".to_string(),
                "script-src 'self' 'unsafe-inline' 'unsafe-eval'".to_string(),
                "style-src 'self' 'unsafe-inline'".to_string(),
                "img-src 'self' data: https:".to_string(),
                "font-src 'self' data:".to_string(),
                "connect-src 'self' ws: http: https:".to_string(),
            ],
        }
    }
}

impl Default for SecurityHeadersConfig {
    fn default() -> Self {
        Self::secure()
    }
}
