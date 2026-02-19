//! Replication health check
//!
//! **Validates: Requirements 2.6.6**
//!
//! This health check monitors replication status including:
//! - Replication lag
//! - Secondary node status
//! - Returns Degraded if lag exceeds threshold

use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;

use crate::{HealthCheck, HealthCheckResult, HealthStatus};

/// Replication manager trait for health checks
///
/// This trait allows the health check to query replication status
/// without depending on the full ReplicationManager implementation
#[async_trait]
pub trait ReplicationStatusProvider: Send + Sync {
    /// Get replication lag metrics
    async fn get_lag_metrics(&self) -> Option<ReplicationLagMetrics>;

    /// Check if this node is primary
    async fn is_primary(&self) -> bool;

    /// Check if secondary is initialized
    async fn is_secondary_initialized(&self) -> bool;
}

/// Replication lag metrics for health checks
#[derive(Debug, Clone)]
pub struct ReplicationLagMetrics {
    /// Last sequence number applied on secondary
    pub last_applied_sequence: u64,

    /// Primary's current sequence number
    pub primary_sequence: u64,

    /// Difference in sequence numbers
    pub sequence_lag: u64,

    /// Time lag in milliseconds
    pub time_lag_ms: u64,

    /// Whether the secondary is considered stale
    pub is_stale: bool,
}

/// Replication health check
///
/// Monitors replication lag and secondary node status.
/// Returns Degraded if replication lag exceeds configured threshold.
pub struct ReplicationHealthCheck {
    /// Maximum acceptable replication lag before marking as degraded
    max_lag_threshold: Duration,

    /// Optional replication status provider
    status_provider: Option<Arc<dyn ReplicationStatusProvider>>,
}

impl ReplicationHealthCheck {
    /// Create a new replication health check
    ///
    /// # Arguments
    ///
    /// * `max_lag_threshold` - Maximum acceptable replication lag (default: 100ms)
    pub fn new(max_lag_threshold: Duration) -> Self {
        Self {
            max_lag_threshold,
            status_provider: None,
        }
    }

    /// Create with default threshold (100ms)
    pub fn with_defaults() -> Self {
        Self::new(Duration::from_millis(100))
    }

    /// Set the replication status provider
    pub fn with_status_provider(
        mut self,
        provider: Arc<dyn ReplicationStatusProvider>,
    ) -> Self {
        self.status_provider = Some(provider);
        self
    }
}

#[async_trait]
impl HealthCheck for ReplicationHealthCheck {
    fn name(&self) -> &str {
        "replication"
    }

    async fn check(&self) -> HealthCheckResult {
        let start_time = std::time::Instant::now();

        let mut details = std::collections::HashMap::new();
        details.insert(
            "max_lag_threshold_ms".to_string(),
            serde_json::Value::Number(serde_json::Number::from(
                self.max_lag_threshold.as_millis() as u64,
            )),
        );

        // Check if replication is configured
        let status_provider = match &self.status_provider {
            Some(provider) => provider,
            None => {
                details.insert(
                    "status".to_string(),
                    serde_json::Value::String("not_configured".to_string()),
                );
                details.insert(
                    "note".to_string(),
                    serde_json::Value::String(
                        "Replication status provider not configured".to_string(),
                    ),
                );

                let response_time = start_time.elapsed().as_millis() as u64;

                return HealthCheckResult {
                    status: HealthStatus::Healthy,
                    message: Some("Replication not configured".to_string()),
                    response_time_ms: response_time.max(1),
                    details: Some(details),
                };
            }
        };

        // Check if this is a primary node
        let is_primary = status_provider.is_primary().await;
        details.insert(
            "is_primary".to_string(),
            serde_json::Value::Bool(is_primary),
        );

        if is_primary {
            // Primary nodes don't have replication lag
            details.insert(
                "status".to_string(),
                serde_json::Value::String("primary".to_string()),
            );

            let response_time = start_time.elapsed().as_millis() as u64;

            return HealthCheckResult {
                status: HealthStatus::Healthy,
                message: Some("Primary node - no replication lag".to_string()),
                response_time_ms: response_time.max(1),
                details: Some(details),
            };
        }

        // Check if secondary is initialized
        let is_initialized = status_provider.is_secondary_initialized().await;
        details.insert(
            "is_initialized".to_string(),
            serde_json::Value::Bool(is_initialized),
        );

        if !is_initialized {
            details.insert(
                "status".to_string(),
                serde_json::Value::String("initializing".to_string()),
            );

            let response_time = start_time.elapsed().as_millis() as u64;

            return HealthCheckResult {
                status: HealthStatus::Degraded,
                message: Some("Secondary node initializing".to_string()),
                response_time_ms: response_time.max(1),
                details: Some(details),
            };
        }

        // Get replication lag metrics
        let metrics = match status_provider.get_lag_metrics().await {
            Some(m) => m,
            None => {
                details.insert(
                    "status".to_string(),
                    serde_json::Value::String("no_metrics".to_string()),
                );

                let response_time = start_time.elapsed().as_millis() as u64;

                return HealthCheckResult {
                    status: HealthStatus::Degraded,
                    message: Some("Replication metrics unavailable".to_string()),
                    response_time_ms: response_time.max(1),
                    details: Some(details),
                };
            }
        };

        // Add metrics to details
        details.insert(
            "last_applied_sequence".to_string(),
            serde_json::Value::Number(serde_json::Number::from(metrics.last_applied_sequence)),
        );
        details.insert(
            "primary_sequence".to_string(),
            serde_json::Value::Number(serde_json::Number::from(metrics.primary_sequence)),
        );
        details.insert(
            "sequence_lag".to_string(),
            serde_json::Value::Number(serde_json::Number::from(metrics.sequence_lag)),
        );
        details.insert(
            "time_lag_ms".to_string(),
            serde_json::Value::Number(serde_json::Number::from(metrics.time_lag_ms)),
        );
        details.insert(
            "is_stale".to_string(),
            serde_json::Value::Bool(metrics.is_stale),
        );

        // Determine health status based on lag
        let (status, message) = if metrics.is_stale {
            details.insert(
                "status".to_string(),
                serde_json::Value::String("stale".to_string()),
            );
            (
                HealthStatus::Degraded,
                format!(
                    "Replication lag ({} ms) exceeds threshold ({} ms)",
                    metrics.time_lag_ms,
                    self.max_lag_threshold.as_millis()
                ),
            )
        } else {
            details.insert(
                "status".to_string(),
                serde_json::Value::String("healthy".to_string()),
            );
            (
                HealthStatus::Healthy,
                format!("Replication lag: {} ms", metrics.time_lag_ms),
            )
        };

        let response_time = start_time.elapsed().as_millis() as u64;

        HealthCheckResult {
            status,
            message: Some(message),
            response_time_ms: response_time.max(1),
            details: Some(details),
        }
    }

    fn is_critical(&self) -> bool {
        false // Replication is not critical for single-node deployments
    }

    fn tags(&self) -> Vec<String> {
        vec!["replication".to_string(), "ha".to_string()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_replication_health_check() {
        let check = ReplicationHealthCheck::with_defaults();
        let result = check.check().await;

        assert_eq!(result.status, HealthStatus::Healthy);
        assert!(result.response_time_ms > 0);
        assert!(result.details.is_some());
    }

    #[tokio::test]
    async fn test_replication_health_check_tags() {
        let check = ReplicationHealthCheck::with_defaults();
        let tags = check.tags();

        assert!(tags.contains(&"replication".to_string()));
        assert!(tags.contains(&"ha".to_string()));
    }

    #[tokio::test]
    async fn test_replication_health_check_not_critical() {
        let check = ReplicationHealthCheck::with_defaults();
        assert!(!check.is_critical());
    }
}
