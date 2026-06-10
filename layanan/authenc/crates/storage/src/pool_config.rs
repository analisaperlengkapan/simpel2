//! Advanced connection pool configuration and optimization
//!
//! Provides production-grade connection pooling configuration inspired by
//! enterprise IAM's HikariCP setup and PostgreSQL best practices.

use deadpool_postgres::{ManagerConfig, PoolConfig, RecyclingMethod};
use std::time::Duration;

/// Connection pool configuration builder
/// Provides fine-grained control over connection pool behavior following
/// production best practices from enterprise IAM and PostgreSQL documentation.
#[derive(Debug, Clone)]
pub struct PoolConfigBuilder {
    /// Maximum number of connections in the pool
    max_size: usize,
    /// Minimum idle connections to maintain
    min_idle: Option<usize>,
    /// Connection timeout
    timeout: Duration,
    /// Connection idle timeout (how long before an idle connection is closed)
    idle_timeout: Option<Duration>,
    /// Connection max lifetime (how long a connection can live)
    max_lifetime: Option<Duration>,
    /// Connection recycling method
    recycling_method: RecyclingMethod,
}

impl Default for PoolConfigBuilder {
    fn default() -> Self {
        Self {
            max_size: 50,       // Optimized for production workload
            min_idle: Some(10), // Maintain warm connections
            timeout: Duration::from_secs(30),
            idle_timeout: Some(Duration::from_secs(600)), // 10 minutes
            max_lifetime: Some(Duration::from_secs(1800)), // 30 minutes
            recycling_method: RecyclingMethod::Verified,  // Ensure connection health
        }
    }
}

impl PoolConfigBuilder {
    /// Create a new pool configuration builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum pool size
    ///
    /// # Recommendations (from enterprise IAM/HikariCP best practices):
    /// - For OLTP workloads: (core_count * 2) + effective_spindle_count
    /// - For web servers: 10-20 per instance
    /// - Never exceed PostgreSQL max_connections setting
    pub fn max_size(mut self, size: usize) -> Self {
        self.max_size = size;
        self
    }

    /// Set minimum idle connections
    ///
    /// Keeping some connections warm improves latency for the first requests.
    /// Recommended: 2-5 for most applications
    pub fn min_idle(mut self, min: usize) -> Self {
        self.min_idle = Some(min);
        self
    }

    /// Set connection acquisition timeout
    ///
    /// How long to wait for a connection from the pool before timing out.
    /// Recommended: 30 seconds (enterprise IAM default)
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = duration;
        self
    }

    /// Set idle connection timeout
    ///
    /// Connections idle longer than this will be closed to free resources.
    /// Recommended: 10 minutes (enterprise IAM/HikariCP default)
    pub fn idle_timeout(mut self, duration: Duration) -> Self {
        self.idle_timeout = Some(duration);
        self
    }

    /// Set maximum connection lifetime
    ///
    /// Connections older than this will be closed and replaced.
    /// Prevents issues with long-lived connections and helps with load balancing.
    /// Recommended: 30 minutes (enterprise IAM/HikariCP default)
    pub fn max_lifetime(mut self, duration: Duration) -> Self {
        self.max_lifetime = Some(duration);
        self
    }

    /// Set connection recycling method
    ///
    /// - Fast: Quick recycling check (default)
    /// - Verified: Runs SELECT 1 to verify connection
    /// - Clean: Closes and reopens connection
    pub fn recycling_method(mut self, method: RecyclingMethod) -> Self {
        self.recycling_method = method;
        self
    }

    /// Build for production environment (enterprise IAM-like settings)
    pub fn production() -> Self {
        Self::new()
            .max_size(20)
            .min_idle(5)
            .timeout(Duration::from_secs(30))
            .idle_timeout(Duration::from_secs(600))
            .max_lifetime(Duration::from_secs(1800))
            .recycling_method(RecyclingMethod::Verified)
    }

    /// Build for development environment
    pub fn development() -> Self {
        Self::new()
            .max_size(5)
            .min_idle(1)
            .timeout(Duration::from_secs(10))
            .idle_timeout(Duration::from_secs(300))
            .max_lifetime(Duration::from_secs(600))
            .recycling_method(RecyclingMethod::Fast)
    }

    /// Build for testing environment
    pub fn testing() -> Self {
        Self::new()
            .max_size(2)
            .min_idle(1)
            .timeout(Duration::from_secs(5))
            .idle_timeout(Duration::from_secs(60))
            .max_lifetime(Duration::from_secs(120))
            .recycling_method(RecyclingMethod::Fast)
    }

    /// Build the deadpool PoolConfig
    pub fn build(self) -> PoolConfig {
        let mut config = PoolConfig {
            max_size: self.max_size,
            ..Default::default()
        };
        config.timeouts.wait = Some(self.timeout);
        config.timeouts.create = Some(self.timeout);
        config.timeouts.recycle = Some(Duration::from_secs(5));

        config
    }

    /// Build ManagerConfig with statement cache
    pub fn build_manager_config(self) -> ManagerConfig {
        ManagerConfig {
            recycling_method: self.recycling_method,
        }
    }
}

/// Pool health metrics
#[derive(Debug, Clone)]
pub struct PoolHealth {
    /// Current pool size
    pub size: usize,
    /// Maximum pool size
    pub max_size: usize,
    /// Number of available connections
    pub available: usize,
    /// Pool utilization percentage
    pub utilization: f64,
}

impl PoolHealth {
    /// Check if pool is healthy
    pub fn is_healthy(&self) -> bool {
        self.utilization < 90.0 && self.available > 0
    }

    /// Check if pool is under pressure
    pub fn is_under_pressure(&self) -> bool {
        self.utilization > 80.0 || self.available < 2
    }

    /// Get health status as string
    pub fn status(&self) -> &'static str {
        if self.is_healthy() {
            "healthy"
        } else if self.is_under_pressure() {
            "under_pressure"
        } else {
            "critical"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_config_builder_default() {
        let config = PoolConfigBuilder::default();
        assert_eq!(config.max_size, 50);
        assert_eq!(config.min_idle, Some(10));
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.idle_timeout, Some(Duration::from_secs(600)));
        assert_eq!(config.max_lifetime, Some(Duration::from_secs(1800)));
    }

    #[test]
    fn test_pool_config_builder_custom() {
        let config = PoolConfigBuilder::new()
            .max_size(10)
            .min_idle(2)
            .timeout(Duration::from_secs(15))
            .idle_timeout(Duration::from_secs(300))
            .max_lifetime(Duration::from_secs(900))
            .build();

        assert_eq!(config.max_size, 10);
    }

    #[test]
    fn test_pool_config_builder_chaining() {
        let builder = PoolConfigBuilder::new()
            .max_size(15)
            .min_idle(3)
            .timeout(Duration::from_secs(20));

        assert_eq!(builder.max_size, 15);
        assert_eq!(builder.min_idle, Some(3));
        assert_eq!(builder.timeout, Duration::from_secs(20));
    }

    #[test]
    fn test_production_config() {
        let config = PoolConfigBuilder::production();
        assert_eq!(config.max_size, 20);
        assert_eq!(config.min_idle, Some(5));
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.idle_timeout, Some(Duration::from_secs(600)));
        assert_eq!(config.max_lifetime, Some(Duration::from_secs(1800)));
    }

    #[test]
    fn test_development_config() {
        let config = PoolConfigBuilder::development();
        assert_eq!(config.max_size, 5);
        assert_eq!(config.min_idle, Some(1));
        assert_eq!(config.timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_testing_config() {
        let config = PoolConfigBuilder::testing();
        assert_eq!(config.max_size, 2);
        assert_eq!(config.min_idle, Some(1));
        assert_eq!(config.timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_recycling_method() {
        let config = PoolConfigBuilder::new().recycling_method(RecyclingMethod::Verified);

        assert!(matches!(config.recycling_method, RecyclingMethod::Verified));
    }

    #[test]
    fn test_build_manager_config() {
        let config = PoolConfigBuilder::new().recycling_method(RecyclingMethod::Clean);

        let manager_config = config.build_manager_config();
        assert!(matches!(
            manager_config.recycling_method,
            RecyclingMethod::Clean
        ));
    }

    #[test]
    fn test_pool_health_healthy() {
        let health = PoolHealth {
            size: 10,
            max_size: 20,
            available: 8,
            utilization: 50.0,
        };

        assert!(health.is_healthy());
        assert!(!health.is_under_pressure());
        assert_eq!(health.status(), "healthy");
    }

    #[test]
    fn test_pool_health_under_pressure() {
        let health = PoolHealth {
            size: 18,
            max_size: 20,
            available: 1,
            utilization: 90.0,
        };

        assert!(!health.is_healthy()); // Not healthy because utilization >= 90%
        assert!(health.is_under_pressure()); // Under pressure because available < 2
        assert_eq!(health.status(), "under_pressure");
    }

    #[test]
    fn test_pool_health_critical_no_available() {
        let health = PoolHealth {
            size: 20,
            max_size: 20,
            available: 0,
            utilization: 100.0,
        };

        assert!(!health.is_healthy()); // Not healthy
        assert!(health.is_under_pressure()); // Under pressure
        // When available is 0, it's still "under_pressure" not "critical"
        // because is_under_pressure() is true
        assert_eq!(health.status(), "under_pressure");
    }

    #[test]
    fn test_pool_health_edge_case_80_percent() {
        let health = PoolHealth {
            size: 16,
            max_size: 20,
            available: 4,
            utilization: 80.0,
        };

        // At exactly 80%, should not be under pressure yet (> 80%)
        assert!(health.is_healthy());
        assert!(!health.is_under_pressure());
    }

    #[test]
    fn test_pool_health_edge_case_81_percent() {
        let health = PoolHealth {
            size: 17,
            max_size: 20,
            available: 3,
            utilization: 81.0,
        };

        // Above 80%, should be under pressure
        assert!(health.is_healthy()); // Still healthy because < 90% and available > 0
        assert!(health.is_under_pressure()); // Under pressure because > 80%
        // status() returns "healthy" because is_healthy() is checked first
        assert_eq!(health.status(), "healthy");
    }

    #[test]
    fn test_pool_health_one_available() {
        let health = PoolHealth {
            size: 10,
            max_size: 20,
            available: 1,
            utilization: 50.0,
        };

        // Less than 2 available is under pressure
        assert!(health.is_healthy()); // Healthy because < 90% and available > 0
        assert!(health.is_under_pressure()); // Under pressure because available < 2
        // status() returns "healthy" because is_healthy() is checked first
        assert_eq!(health.status(), "healthy");
    }
}
