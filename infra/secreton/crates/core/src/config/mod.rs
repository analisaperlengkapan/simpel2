//! Configuration management for Secreton
//!
//! This module provides configuration loading and management capabilities
//! for various Secreton components including MFA policies and dynamic configuration.

use crate::error::{CoreError, Result};
use config;
use serde::Deserialize;
use std::path::Path;

pub mod dynamic;
pub mod mfa_policy_loader;

pub use dynamic::*;
pub use mfa_policy_loader::*;

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
} // 1 hour
fn default_refresh_token_ttl() -> i64 {
    2_592_000
} // 30 days
fn default_password_reset_ttl() -> i64 {
    3600
} // 1 hour
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
