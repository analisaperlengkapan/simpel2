//! Connection pool health monitoring service
//!
//! Provides periodic health checks and monitoring for database connection pools.
//! Implements automatic connection validation and metrics collection.

use crate::database::{Database, PoolStats, ValidationResult};
use crate::error::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

/// Connection pool monitor configuration
#[derive(Debug, Clone)]
pub struct PoolMonitorConfig {
    /// Interval between health checks (default: 30 seconds)
    pub check_interval: Duration,
    /// Warning threshold for pool utilization (default: 80%)
    pub utilization_warning_threshold: f64,
    /// Critical threshold for pool utilization (default: 90%)
    pub utilization_critical_threshold: f64,
    /// Warning threshold for average wait time in milliseconds (default: 100ms)
    pub wait_time_warning_threshold_ms: u64,
    /// Enable automatic logging of pool statistics
    pub enable_stats_logging: bool,
}

impl Default for PoolMonitorConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(30),
            utilization_warning_threshold: 80.0,
            utilization_critical_threshold: 90.0,
            wait_time_warning_threshold_ms: 100,
            enable_stats_logging: true,
        }
    }
}

/// Connection pool monitor service
pub struct PoolMonitor {
    database: Arc<Database>,
    config: PoolMonitorConfig,
}

impl PoolMonitor {
    /// Create a new pool monitor
    pub fn new(database: Arc<Database>, config: PoolMonitorConfig) -> Self {
        Self { database, config }
    }

    /// Create a pool monitor with default configuration
    pub fn with_defaults(database: Arc<Database>) -> Self {
        Self::new(database, PoolMonitorConfig::default())
    }

    /// Start the monitoring service
    ///
    /// This runs indefinitely and should be spawned as a background task.
    pub async fn start(self) {
        info!(
            "Starting connection pool monitor (check interval: {:?})",
            self.config.check_interval
        );

        let mut check_interval = interval(self.config.check_interval);

        loop {
            check_interval.tick().await;

            if let Err(e) = self.perform_health_check().await {
                error!("Pool health check failed: {}", e);
            }

            if self.config.enable_stats_logging {
                self.log_pool_statistics();
            }
        }
    }

    /// Perform a single health check
    async fn perform_health_check(&self) -> Result<()> {
        let validation_result = self.database.validate_connections().await?;

        if !validation_result.healthy {
            error!(
                "Connection pool validation failed: {}",
                validation_result
                    .error
                    .as_deref()
                    .unwrap_or("Unknown error")
            );
        } else {
            debug!(
                "Connection pool validation successful (took {:?})",
                validation_result.validation_time
            );
        }

        // Check pool health thresholds
        self.check_thresholds(&validation_result);

        Ok(())
    }

    /// Check pool health against configured thresholds
    fn check_thresholds(&self, validation: &ValidationResult) {
        let health = &validation.pool_health;

        // Check utilization
        if health.utilization >= self.config.utilization_critical_threshold {
            error!(
                "CRITICAL: Connection pool utilization at {:.1}% (threshold: {:.1}%)",
                health.utilization, self.config.utilization_critical_threshold
            );
        } else if health.utilization >= self.config.utilization_warning_threshold {
            warn!(
                "WARNING: Connection pool utilization at {:.1}% (threshold: {:.1}%)",
                health.utilization, self.config.utilization_warning_threshold
            );
        }

        // Check wait time
        let stats = self.database.pool_stats();
        let avg_wait_time_ms = stats.avg_wait_time_us / 1000;

        if avg_wait_time_ms > self.config.wait_time_warning_threshold_ms {
            warn!(
                "WARNING: Average connection wait time is {}ms (threshold: {}ms)",
                avg_wait_time_ms, self.config.wait_time_warning_threshold_ms
            );
        }

        // Check for low availability
        if health.available == 0 {
            error!("CRITICAL: No available connections in pool!");
        } else if health.available < 2 {
            warn!(
                "WARNING: Only {} connection(s) available in pool",
                health.available
            );
        }
    }

    /// Log detailed pool statistics
    fn log_pool_statistics(&self) {
        let stats = self.database.pool_stats();
        let health = self.database.pool_health();

        info!(
            "Pool Stats: size={}/{}, available={}, waiting={}, utilization={:.1}%",
            stats.size, stats.max_size, stats.available, stats.waiting, stats.utilization
        );

        debug!(
            "Pool Metrics: acquired={}, failures={}, created={}, reuses={} ({:.1}% reuse rate)",
            stats.total_acquired,
            stats.total_failures,
            stats.total_created,
            stats.total_reuses,
            stats.reuse_rate
        );

        debug!(
            "Pool Timing: avg_acquisition={}μs, avg_wait={}μs",
            stats.avg_acquisition_time_us, stats.avg_wait_time_us
        );

        debug!(
            "Pool Health: checks={}, success_rate={:.1}%",
            stats.total_health_checks, stats.health_check_success_rate
        );

        // Log warnings if needed
        if !health.is_healthy() {
            warn!("Pool health status: {}", health.status());
        }
    }

    /// Get current pool statistics
    pub fn get_stats(&self) -> PoolStats {
        self.database.pool_stats()
    }

    /// Perform an immediate health check
    pub async fn check_now(&self) -> Result<ValidationResult> {
        self.database.validate_connections().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_monitor_config_defaults() {
        let config = PoolMonitorConfig::default();
        assert_eq!(config.check_interval, Duration::from_secs(30));
        assert_eq!(config.utilization_warning_threshold, 80.0);
        assert_eq!(config.utilization_critical_threshold, 90.0);
        assert_eq!(config.wait_time_warning_threshold_ms, 100);
        assert!(config.enable_stats_logging);
    }

    #[tokio::test]
    async fn test_pool_monitor_creation() {
        let db = Arc::new(Database::mock().await);
        let monitor = PoolMonitor::with_defaults(db);

        // Should be able to get stats immediately
        let stats = monitor.get_stats();
        assert!(stats.max_size > 0);
    }
}
