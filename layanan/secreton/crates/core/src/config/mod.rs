//! Configuration types.
//!
//! There is no single "the config". Four distinct types load from four different
//! places, and picking the wrong one is the usual mistake:
//!
//! | Type | Loaded from | When |
//! |---|---|---|
//! | [`BootstrapConfig`] | a file, via [`BootstrapConfig::from_file`] | first thing at start-up: listeners, storage backend, seal type |
//! | [`Config`] (this module) | TOML file and/or `SECRETON__*` env, via [`Config::from_file`] / [`Config::from_env`] | server / database / auth / lease settings |
//! | [`ApplicationConfig`] | the storage backend, **encrypted**, via [`ApplicationConfig::load_from_storage`] | after unseal — it needs the master key |
//! | [`DynamicConfig`] | nothing; built in memory and adapted at runtime | cache/crypto/perf knobs that react to load and threat level |
//!
//! A fifth, unrelated `Config` lives in [`crate::utils::config`] and reads
//! `VAULT_*` variables. It is not part of this module.
//!
//! # Environment variables
//!
//! [`Config::from_env`] uses the `Secreton` prefix with `__` as the **separator**,
//! so the variable for `server.port` is `SECRETON__SERVER__PORT` — a single
//! underscore does not bind, it is silently ignored and the default is kept.
//!
//! ```rust
//! use secreton_core::config::Config;
//!
//! let config = Config::default();
//! assert_eq!(config.server.port, 8080);
//! assert_eq!(config.auth.token_ttl, 3600);       // 1 hour
//! assert_eq!(config.lease.check_interval_secs, 60);
//! ```
//!
//! # Application config is encrypted at rest
//!
//! [`ApplicationConfig`] is not a file. It is serialised, encrypted with the
//! master key and stored under `config/system` in the storage backend, so
//! reading or writing it requires an unsealed engine — [`ApplicationConfig::load_from_storage`]
//! and [`ApplicationConfig::save_to_storage`] both take a `SealService` and fail
//! while sealed. Its defaults are available without any of that:
//!
//! ```rust
//! use secreton_core::config::ApplicationConfig;
//!
//! let config = ApplicationConfig::default();
//! assert!(config.auth.require_auth);
//! assert_eq!(config.metadata.version, 1);
//! ```
//!
//! # Dynamic config
//!
//! [`DynamicConfig`] carries the knobs that change while the process runs.
//! [`DynamicConfig::adapt_to_load`] shifts the performance profile from load
//! metrics; [`DynamicConfig::update_security_posture`] tightens crypto and cache
//! behaviour as the threat level rises. [`DynamicConfigManager`] wraps it in a
//! `watch` channel so subscribers see updates.
//!
//! ```rust
//! use secreton_core::config::{DynamicConfig, ThreatLevel};
//!
//! let mut dynamic = DynamicConfig::new();
//! dynamic.update_security_posture(ThreatLevel::High);
//! assert_eq!(dynamic.threat_level, ThreatLevel::High);
//! ```
//!
//! # MFA policy
//!
//! [`MfaPolicyConfig`] and friends in [`mfa_policy_loader`] describe the MFA
//! policy document (per-role and per-satker settings, recovery codes, rate
//! limiting, audit). They are deserialisation targets, not a loader service.

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
