//! Replication configuration

use crate::{ReplicationMode, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for replication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationConfig {
    /// Replication mode (Performance or DR)
    pub mode: ReplicationMode,

    /// Primary cluster endpoint (for secondary nodes)
    pub primary_endpoint: Option<String>,

    /// List of secondary node endpoints (for primary)
    #[serde(default)]
    pub secondary_endpoints: Vec<String>,

    /// Maximum acceptable replication lag in milliseconds
    #[serde(default = "default_max_lag_ms")]
    pub max_lag_ms: u64,

    /// Replication batch size (number of operations per batch)
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,

    /// Replication interval in milliseconds
    #[serde(default = "default_interval_ms")]
    pub interval_ms: u64,

    /// Enable automatic failover
    #[serde(default = "default_auto_failover")]
    pub auto_failover: bool,

    /// Failover timeout in seconds
    #[serde(default = "default_failover_timeout_secs")]
    pub failover_timeout_secs: u64,

    /// Enable namespace filtering
    #[serde(default)]
    pub enable_namespace_filter: bool,

    /// Namespaces to replicate (empty = all)
    #[serde(default)]
    pub namespaces: Vec<String>,

    /// Staleness threshold in milliseconds for secondary reads
    /// If replication lag exceeds this, reads will be rejected
    #[serde(default = "default_staleness_threshold_ms")]
    pub staleness_threshold_ms: Option<u64>,

    /// Heartbeat timeout in milliseconds
    /// If no heartbeat is received within this time, primary is considered failed
    #[serde(default = "default_heartbeat_timeout_ms")]
    pub heartbeat_timeout_ms: Option<u64>,

    /// Number of consecutive failures before triggering failover
    #[serde(default = "default_failure_threshold")]
    pub failure_threshold: Option<u32>,

    /// Heartbeat check interval in milliseconds
    /// How often to check if primary heartbeat has timed out
    #[serde(default = "default_heartbeat_check_interval_ms")]
    pub heartbeat_check_interval_ms: Option<u64>,

    /// TLS configuration for replication
    pub tls: Option<TlsConfig>,

    /// Retry configuration
    #[serde(default)]
    pub retry: RetryConfig,
}

/// TLS configuration for replication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Path to CA certificate
    pub ca_cert: String,

    /// Path to client certificate
    pub client_cert: String,

    /// Path to client private key
    pub client_key: String,

    /// Server name for SNI
    pub server_name: Option<String>,
}

/// Retry configuration for replication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retries
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Initial retry delay in milliseconds
    #[serde(default = "default_initial_delay_ms")]
    pub initial_delay_ms: u64,

    /// Maximum retry delay in milliseconds
    #[serde(default = "default_max_delay_ms")]
    pub max_delay_ms: u64,

    /// Backoff multiplier
    #[serde(default = "default_backoff_multiplier")]
    pub backoff_multiplier: f64,
}

// Default values
fn default_max_lag_ms() -> u64 {
    100 // 100ms
}

fn default_batch_size() -> usize {
    100
}

fn default_interval_ms() -> u64 {
    100 // 100ms
}

fn default_auto_failover() -> bool {
    true
}

fn default_failover_timeout_secs() -> u64 {
    30
}

fn default_max_retries() -> u32 {
    3
}

fn default_initial_delay_ms() -> u64 {
    100
}

fn default_max_delay_ms() -> u64 {
    30000 // 30 seconds
}

fn default_backoff_multiplier() -> f64 {
    2.0
}

fn default_staleness_threshold_ms() -> Option<u64> {
    Some(100) // 100ms default
}

fn default_heartbeat_timeout_ms() -> Option<u64> {
    Some(5000) // 5 seconds default
}

fn default_failure_threshold() -> Option<u32> {
    Some(3) // 3 consecutive failures
}

fn default_heartbeat_check_interval_ms() -> Option<u64> {
    Some(1000) // Check every 1 second
}

impl Default for ReplicationConfig {
    fn default() -> Self {
        Self {
            mode: ReplicationMode::Performance,
            primary_endpoint: None,
            secondary_endpoints: Vec::new(),
            max_lag_ms: default_max_lag_ms(),
            batch_size: default_batch_size(),
            interval_ms: default_interval_ms(),
            auto_failover: default_auto_failover(),
            failover_timeout_secs: default_failover_timeout_secs(),
            enable_namespace_filter: false,
            namespaces: Vec::new(),
            staleness_threshold_ms: default_staleness_threshold_ms(),
            heartbeat_timeout_ms: default_heartbeat_timeout_ms(),
            failure_threshold: default_failure_threshold(),
            heartbeat_check_interval_ms: default_heartbeat_check_interval_ms(),
            tls: None,
            retry: RetryConfig::default(),
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: default_max_retries(),
            initial_delay_ms: default_initial_delay_ms(),
            max_delay_ms: default_max_delay_ms(),
            backoff_multiplier: default_backoff_multiplier(),
        }
    }
}

impl ReplicationConfig {
    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // For secondary nodes, primary_endpoint is required
        if self.primary_endpoint.is_some() && self.secondary_endpoints.is_empty() {
            // This is a secondary node - valid
        } else if self.primary_endpoint.is_none() {
            // This is a primary node - secondary_endpoints can be empty (standalone primary)
        } else {
            return Err(crate::error::ReplicationError::config(
                "Invalid configuration: cannot have both primary_endpoint and secondary_endpoints",
            ));
        }

        // Validate lag threshold
        if self.max_lag_ms == 0 {
            return Err(crate::error::ReplicationError::config(
                "max_lag_ms must be greater than 0",
            ));
        }

        // Validate batch size
        if self.batch_size == 0 {
            return Err(crate::error::ReplicationError::config(
                "batch_size must be greater than 0",
            ));
        }

        // Validate interval
        if self.interval_ms == 0 {
            return Err(crate::error::ReplicationError::config(
                "interval_ms must be greater than 0",
            ));
        }

        // Validate retry config
        if self.retry.max_retries == 0 {
            return Err(crate::error::ReplicationError::config(
                "retry.max_retries must be greater than 0",
            ));
        }

        if self.retry.backoff_multiplier <= 1.0 {
            return Err(crate::error::ReplicationError::config(
                "retry.backoff_multiplier must be greater than 1.0",
            ));
        }

        Ok(())
    }

    /// Get the maximum lag as a Duration
    pub fn max_lag(&self) -> Duration {
        Duration::from_millis(self.max_lag_ms)
    }

    /// Get the replication interval as a Duration
    pub fn interval(&self) -> Duration {
        Duration::from_millis(self.interval_ms)
    }

    /// Get the failover timeout as a Duration
    pub fn failover_timeout(&self) -> Duration {
        Duration::from_secs(self.failover_timeout_secs)
    }

    /// Check if a namespace should be replicated
    pub fn should_replicate_namespace(&self, namespace: &str) -> bool {
        if !self.enable_namespace_filter {
            return true;
        }

        if self.namespaces.is_empty() {
            return true;
        }

        self.namespaces.iter().any(|ns| ns == namespace)
    }
}

impl RetryConfig {
    /// Get the initial delay as a Duration
    pub fn initial_delay(&self) -> Duration {
        Duration::from_millis(self.initial_delay_ms)
    }

    /// Get the maximum delay as a Duration
    pub fn max_delay(&self) -> Duration {
        Duration::from_millis(self.max_delay_ms)
    }

    /// Calculate the delay for a given retry attempt
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let delay_ms = (self.initial_delay_ms as f64
            * self.backoff_multiplier.powi(attempt as i32)) as u64;
        Duration::from_millis(delay_ms.min(self.max_delay_ms))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ReplicationConfig::default();
        assert_eq!(config.mode, ReplicationMode::Performance);
        assert_eq!(config.max_lag_ms, 100);
        assert_eq!(config.batch_size, 100);
        assert!(config.auto_failover);
    }

    #[test]
    fn test_config_validation() {
        // Primary with secondaries - valid
        let mut config = ReplicationConfig::default();
        config.primary_endpoint = None;
        config.secondary_endpoints = vec!["https://secondary:50051".to_string()];
        assert!(config.validate().is_ok());

        // Secondary with primary - valid
        config.primary_endpoint = Some("https://primary:50051".to_string());
        config.secondary_endpoints.clear();
        assert!(config.validate().is_ok());

        // Standalone primary (no secondaries yet) - valid
        config.primary_endpoint = None;
        config.secondary_endpoints.clear();
        assert!(config.validate().is_ok());

        // Both primary and secondaries - invalid
        config.primary_endpoint = Some("https://primary:50051".to_string());
        config.secondary_endpoints = vec!["https://secondary:50051".to_string()];
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_namespace_filtering() {
        let mut config = ReplicationConfig::default();
        config.enable_namespace_filter = false;
        assert!(config.should_replicate_namespace("any-namespace"));

        config.enable_namespace_filter = true;
        config.namespaces = vec!["ns1".to_string(), "ns2".to_string()];
        assert!(config.should_replicate_namespace("ns1"));
        assert!(config.should_replicate_namespace("ns2"));
        assert!(!config.should_replicate_namespace("ns3"));
    }

    #[test]
    fn test_retry_delay_calculation() {
        let retry = RetryConfig::default();
        assert_eq!(retry.delay_for_attempt(0), Duration::from_millis(100));
        assert_eq!(retry.delay_for_attempt(1), Duration::from_millis(200));
        assert_eq!(retry.delay_for_attempt(2), Duration::from_millis(400));
        assert_eq!(retry.delay_for_attempt(3), Duration::from_millis(800));

        // Should cap at max_delay_ms
        assert_eq!(retry.delay_for_attempt(20), Duration::from_millis(30000));
    }
}
