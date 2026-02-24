//! Connection pool health monitoring service
//!
//! Provides periodic health checks and monitoring for database connection pools.
//! Implements automatic connection validation and metrics collection.

use crate::Database;
use authenc_types::Result;
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
        // Simple health check - try to get a connection
        let start = std::time::Instant::now();
        match self.database.get_connection().await {
            Ok(_conn) => {
                let duration = start.elapsed();
                debug!(
                    "Connection pool validation successful (took {:?})",
                    duration
                );
                Ok(())
            }
            Err(e) => {
                error!("Connection pool validation failed: {}", e);
                Err(e)
            }
        }
    }

    /// Log detailed pool statistics
    fn log_pool_statistics(&self) {
        let status = self.database.pool_status();

        info!(
            "Pool Stats: size={}, available={}, waiting={}",
            status.size, status.available, status.waiting
        );

        // Calculate utilization
        let utilization = if status.size > 0 {
            ((status.size - status.available) as f64 / status.size as f64) * 100.0
        } else {
            0.0
        };

        // Check utilization thresholds
        if utilization >= self.config.utilization_critical_threshold {
            error!(
                "CRITICAL: Connection pool utilization at {:.1}% (threshold: {:.1}%)",
                utilization, self.config.utilization_critical_threshold
            );
        } else if utilization >= self.config.utilization_warning_threshold {
            warn!(
                "WARNING: Connection pool utilization at {:.1}% (threshold: {:.1}%)",
                utilization, self.config.utilization_warning_threshold
            );
        }

        // Check for low availability
        if status.available == 0 {
            error!("CRITICAL: No available connections in pool!");
        } else if status.available < 2 {
            warn!(
                "WARNING: Only {} connection(s) available in pool",
                status.available
            );
        }
    }

    /// Perform an immediate health check
    pub async fn check_now(&self) -> Result<()> {
        self.perform_health_check().await
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
}
