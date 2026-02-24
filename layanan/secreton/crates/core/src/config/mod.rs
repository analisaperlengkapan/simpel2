//! Configuration Management for Secreton
//!
//! This module provides centralized configuration loading, validation, and hot-reloading
//! capabilities for all Secreton components, following 12-Factor App principles.
//!
//! # Configuration Sources (Priority Order)
//!
//! 1. **Environment Variables** - Highest priority (production secrets)
//! 2. **Config File** - YAML/TOML files (dev/staging)
//! 3. **Engine** - Remote config from Secreton itself (self-hosting)
//! 4. **Defaults** - Hardcoded fallbacks
//!
//! ```text
//! ┌────────────────────────────────────────┐
//! │   Environment Variables (highest)      │
//! └──────────────┬─────────────────────────┘
//!                ▼
//! ┌────────────────────────────────────────┐
//! │   Config File (config.yaml)            │
//! └──────────────┬─────────────────────────┘
//!                ▼
//! ┌────────────────────────────────────────┐
//! │   Engine (remote config)                │
//! └──────────────┬─────────────────────────┘
//!                ▼
//! ┌────────────────────────────────────────┐
//! │   Default Values (fallback)            │
//! └────────────────────────────────────────┘
//! ```
//!
//! # Example: Load Configuration
//!
//! ```ignore
//! use secreton_core::config::SecretonConfig;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Load from default locations
//! let config = SecretonConfig::load()?;
//!
//! println!("Server: {::}", config.server.host, config.server.port);
//! println!("Database: {}", config.database.url);
//! println!("MFA required: {}", config.mfa.require_for_admin);
//! # Ok(())
//! # }
//! ```
//!
//! # Example: Environment Override
//!
//! ```bash
//! # Override config with environment variables
//! export SECRETON_SERVER_PORT=9000
//! export SECRETON_DATABASE_URL=postgresql://prod-db:5432/secreton
//! export SECRETON_MFA_REQUIRE_FOR_ADMIN=true
//! ```
//!
//! # Configuration Modules
//!
//! - **Server Config** - Host, port, TLS settings, worker threads
//! - **Database Config** - Connection string, pool size, timeouts
//! - **MFA Policy** - Required methods, grace period, backup codes
//! - **Audit Config** - Backends, retention, sampling rate
//! - **Cache Config** - TTL, size limits, eviction policy
//! - **Crypto Config** - Algorithms, key sizes, rotation policy
//!
//! # Dynamic Configuration
//!
//! Some settings can be changed at runtime without restart:
//!
//! ```ignore
//! use secreton_core::config::DynamicConfig;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let dynamic = DynamicConfig::new();
//!
//! // Update MFA requirement at runtime
//! dynamic.update_mfa_policy(|policy| {
//!     policy.require_for_admin = true;
//!     policy.grace_period_hours = 24;
//! }).await?;
//!
//! // Changes apply immediately to new requests
//! # Ok(())
//! # }
//! ```
//!
//! # MFA Policy Configuration
//!
//! ```ignore
//! use secreton_core::config::MfaPolicyLoader;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let policy = MfaPolicyLoader::load_from_file("mfa-policy.yaml")?;
//!
//! println!("Required for admin: {}", policy.require_for_admin);
//! println!("Allowed methods: {:?}", policy.allowed_methods);
//! println!("Backup codes enabled: {}", policy.allow_backup_codes);
//! # Ok(())
//! # }
//! ```
//!
//! Example `mfa-policy.yaml`:
//!
//! ```yaml
//! require_for_admin: true
//! require_for_secrets: true
//! allowed_methods:
//!   - totp
//!   - webauthn
//! grace_period_hours: 24
//! allow_backup_codes: true
//! backup_code_count: 10
//! ```
//!
//! # Validation
//!
//! Configuration validated on load:
//!
//! - Database URLs must be valid PostgreSQL URIs
//! - Port numbers in valid range (1-65535)
//! - File paths must be readable/writable
//! - MFA methods must be supported
//! - Crypto algorithms must be approved (FIPS 140-3)
//!
//! ```ignore
//! # use secreton_core::config::SecretonConfig;
//! # fn example() {
//! match SecretonConfig::load() {
//!     Ok(config) => println!("Config valid: {:#?}", config),
//!     Err(e) => {
//!         eprintln!("Invalid config: {}", e);
//!         std::process::exit(1);
//!     }
//! }
//! # }
//! ```
//!
//! # Hot Reload
//!
//! Watch config files for changes:
//!
//! ```ignore
//! # use secreton_core::config::DynamicConfig;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let dynamic = DynamicConfig::new();
//!
//! // Watch config file for changes
//! dynamic.watch("config.yaml").await?;
//!
//! // Automatically reloads on file change
//! // Non-critical settings updated without restart
//! # Ok(())
//! # }
//! ```
//!
//! # Secrets in Configuration
//!
//! **NEVER** put secrets directly in config files. Use:
//!
//! 1. **Environment Variables**: `SECRETON_DATABASE_PASSWORD=...`
//! 2. **Engine References**: `database_password: ${engine:database/prod/password}`
//! 3. **File References**: `tls_cert: ${file:/etc/secreton/cert.pem}`
//!
//! ```yaml
//! # ❌ BAD - Secret in config file
//! database:
//!   url: postgresql://user:password123@db/secreton
//!
//! # ✅ GOOD - Secret from environment
//! database:
//!   url: ${env:DATABASE_URL}
//!
//! # ✅ GOOD - Secret from Engine
//! database:
//!   url: ${engine:secrets/database/url}
//! ```
//!
//! # Configuration Struct
//!
//! Main configuration structure:
//!
//! ```rust,ignore
//! pub struct SecretonConfig {
//!     pub server: ServerConfig,
//!     pub database: DatabaseConfig,
//!     pub mfa: MfaPolicyConfig,
//!     pub audit: AuditConfig,
//!     pub cache: CacheConfig,
//!     pub crypto: CryptoConfig,
//!     pub namespace: NamespaceConfig,
//! }
//! ```
//!
//! # Default Configuration
//!
//! Development-safe defaults:
//!
//! ```ignore
//! # use secreton_core::config::SecretonConfig;
//! let config = SecretonConfig::default();
//! // server: 127.0.0.1:8200
//! // database: postgresql://localhost/secreton
//! // mfa: disabled for dev, enabled for prod
//! ```
//!
//! # Environment Variable Mapping
//!
//! | Environment Variable | Config Path | Example |
//! |---------------------|-------------|---------|
//! | `SECRETON_SERVER_PORT` | `server.port` | `8200` |
//! | `SECRETON_DATABASE_URL` | `database.url` | `postgresql://...` |
//! | `SECRETON_MFA_REQUIRE_FOR_ADMIN` | `mfa.require_for_admin` | `true` |
//! | `SECRETON_LOG_LEVEL` | `server.log_level` | `info` |
//!
//! # See Also
//!
//! - [`DynamicConfig`] - Runtime configuration updates
//! - [`MfaPolicyLoader`] - MFA policy management
//! - `crate::storage` - Database connection configuration
//! - `crate::audit` - Audit logging configuration

use crate::error::{CoreError, Result};
use config;
use serde::Deserialize;
use std::path::Path;

pub mod api;
pub mod application;
pub mod bootstrap;
pub mod dynamic;
pub mod mfa_policy_loader;
pub mod migrate;

// pub use api::*; // Avoid conflicts with local structs
pub use application::*;

// Re-export bootstrap config (StorageConfig renamed to BootstrapStorageConfig to avoid conflict)
/// Alias for bootstrap StorageConfig to avoid naming conflict with dynamic::StorageConfig
pub use bootstrap::StorageConfig as BootstrapStorageConfig;
pub use bootstrap::{
    AwsKmsSealConfig, AzureKvSealConfig, BootstrapConfig, FileStorageConfig, GcpKmsSealConfig,
    GrpcListenerConfig, HttpListenerConfig, ListenerConfig, PostgresStorageConfig,
    RaftPerformanceConfig, RaftRetryJoin, RaftStorageConfig, SealConfigBootstrap, SealType,
    SealTypeConfig, ShamirSealConfig, StorageBackend, StorageBackendConfig, TelemetryConfig,
    TlsConfig,
};

// Re-export dynamic config (explicit to avoid ambiguity)
/// Dynamic storage configuration (runtime adjustable)
pub use dynamic::StorageConfig as DynamicStorageConfig;
pub use dynamic::{
    CacheConfig, CryptoMode, DynamicConfig, DynamicConfigManager, LoadMetrics, PerformanceProfile,
    PerformanceProfiler, SecurityConfig, ThreatLevel,
};

// Re-export MFA policy loader
pub use mfa_policy_loader::*;

// Re-export migrate
pub use migrate::*;

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub log_level: String,
    pub storage_path: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    #[serde(default = "default_token_ttl")]
    pub token_ttl: i64, // in seconds

    #[serde(default = "default_refresh_token_ttl")]
    pub refresh_token_ttl: i64, // in seconds

    #[serde(default = "default_jwt_secret")]
    pub jwt_secret: String,

    #[serde(default = "default_refresh_secret")]
    pub refresh_secret: String,

    #[serde(default = "default_password_reset_ttl")]
    pub password_reset_ttl: i64, // in seconds

    #[serde(default = "default_mfa_enabled")]
    pub mfa_enabled: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LeaseConfig {
    /// Lease expiration check interval in seconds (default: 60)
    #[serde(default = "default_lease_check_interval")]
    pub check_interval_secs: u64,

    /// Notification threshold in seconds (default: 300 = 5 minutes)
    #[serde(default = "default_lease_notification_threshold")]
    pub notification_threshold_secs: i64,

    /// Enable notifications before expiration (default: false)
    #[serde(default = "default_lease_notifications_enabled")]
    pub enable_notifications: bool,
}

// Default configuration values
fn default_token_ttl() -> i64 {
    3600
}
fn default_refresh_token_ttl() -> i64 {
    2_592_000
}
fn default_password_reset_ttl() -> i64 {
    3600
}
fn default_mfa_enabled() -> bool {
    true
}

fn default_jwt_secret() -> String {
    // In production, this should be overridden via environment variables
    "default-jwt-secret-please-change-in-production".to_string()
}

fn default_refresh_secret() -> String {
    // In production, this should be overridden via environment variables
    "default-refresh-secret-please-change-in-production".to_string()
}

fn default_lease_check_interval() -> u64 {
    60 // 60 seconds
}

fn default_lease_notification_threshold() -> i64 {
    300 // 5 minutes
}

fn default_lease_notifications_enabled() -> bool {
    false
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,

    #[serde(default)]
    pub lease: LeaseConfig,
}

impl Default for LeaseConfig {
    fn default() -> Self {
        Self {
            check_interval_secs: default_lease_check_interval(),
            notification_threshold_secs: default_lease_notification_threshold(),
            enable_notifications: default_lease_notifications_enabled(),
        }
    }
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config = config::Config::builder()
            .add_source(config::File::from(path.as_ref()))
            .add_source(config::Environment::with_prefix("Secreton").separator("__"))
            .build()
            .map_err(|e| CoreError::configuration(format!("Failed to build config: {}", e)))?;

        config
            .try_deserialize()
            .map_err(|e| CoreError::configuration(format!("Failed to deserialize config: {}", e)))
    }

    pub fn from_env() -> Result<Self> {
        let config = config::Config::builder()
            .add_source(config::Environment::with_prefix("Secreton").separator("__"))
            .build()
            .map_err(|e| {
                CoreError::configuration(format!("Failed to build config from env: {}", e))
            })?;

        config.try_deserialize().map_err(|e| {
            CoreError::configuration(format!("Failed to deserialize config from env: {}", e))
        })
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                log_level: "info".to_string(),
                storage_path: "./data".to_string(),
            },
            database: DatabaseConfig {
                url: "sqlite:./data/Secreton.db".to_string(),
                max_connections: 5,
            },
            auth: AuthConfig {
                token_ttl: default_token_ttl(),
                refresh_token_ttl: default_refresh_token_ttl(),
                jwt_secret: default_jwt_secret(),
                refresh_secret: default_refresh_secret(),
                password_reset_ttl: default_password_reset_ttl(),
                mfa_enabled: default_mfa_enabled(),
            },
            lease: LeaseConfig::default(),
        }
    }
}
