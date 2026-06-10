// ============================================================================
// Connection Pooling Configuration
// Description: Optimized connection pool configuration for database and Redis
// Author: SIMPEL Team
// Created: 2026-02-10
// Requirements: NFR-SC001
// ============================================================================

use std::time::Duration;

/// Database connection pool configuration
#[derive(Debug, Clone)]
pub struct DatabasePoolConfig {
    /// Maximum number of connections in the pool
    pub max_size: usize,

    /// Minimum number of idle connections to maintain
    pub min_idle: usize,

    /// Connection timeout (how long to wait for a connection from the pool)
    pub connection_timeout: Duration,

    /// Idle timeout (how long a connection can be idle before being closed)
    pub idle_timeout: Duration,

    /// Maximum lifetime of a connection
    pub max_lifetime: Duration,
}

impl Default for DatabasePoolConfig {
    fn default() -> Self {
        Self {
            max_size: 50,                                // Max 50 connections
            min_idle: 10,                                // Min 10 idle connections
            connection_timeout: Duration::from_secs(30), // 30 seconds
            idle_timeout: Duration::from_secs(600),      // 10 minutes
            max_lifetime: Duration::from_secs(1800),     // 30 minutes
        }
    }
}

impl DatabasePoolConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            max_size: std::env::var("DB_POOL_MAX_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(50),
            min_idle: std::env::var("DB_POOL_MIN_IDLE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),
            connection_timeout: std::env::var("DB_CONNECTION_TIMEOUT_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .map(Duration::from_secs)
                .unwrap_or(Duration::from_secs(30)),
            idle_timeout: std::env::var("DB_IDLE_TIMEOUT_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .map(Duration::from_secs)
                .unwrap_or(Duration::from_secs(600)),
            max_lifetime: std::env::var("DB_MAX_LIFETIME_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .map(Duration::from_secs)
                .unwrap_or(Duration::from_secs(1800)),
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.max_size == 0 {
            return Err("max_size must be greater than 0".to_string());
        }

        if self.min_idle > self.max_size {
            return Err("min_idle cannot be greater than max_size".to_string());
        }

        if self.connection_timeout.as_secs() == 0 {
            return Err("connection_timeout must be greater than 0".to_string());
        }

        Ok(())
    }
}

/// Redis connection pool configuration
#[derive(Debug, Clone)]
pub struct RedisPoolConfig {
    /// Maximum number of connections in the pool
    pub max_size: usize,

    /// Minimum number of idle connections to maintain
    pub min_idle: usize,

    /// Connection timeout
    pub connection_timeout: Duration,

    /// Response timeout for Redis commands
    pub response_timeout: Duration,

    /// Maximum number of retries for failed operations
    pub max_retries: u32,

    /// Retry delay
    pub retry_delay: Duration,
}

impl Default for RedisPoolConfig {
    fn default() -> Self {
        Self {
            max_size: 20,                                // Max 20 connections
            min_idle: 5,                                 // Min 5 idle connections
            connection_timeout: Duration::from_secs(10), // 10 seconds
            response_timeout: Duration::from_secs(5),    // 5 seconds
            max_retries: 3,                              // 3 retries
            retry_delay: Duration::from_millis(100),     // 100ms between retries
        }
    }
}

impl RedisPoolConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            max_size: std::env::var("REDIS_POOL_MAX_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(20),
            min_idle: std::env::var("REDIS_POOL_MIN_IDLE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5),
            connection_timeout: std::env::var("REDIS_CONNECTION_TIMEOUT_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .map(Duration::from_secs)
                .unwrap_or(Duration::from_secs(10)),
            response_timeout: std::env::var("REDIS_RESPONSE_TIMEOUT_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .map(Duration::from_secs)
                .unwrap_or(Duration::from_secs(5)),
            max_retries: std::env::var("REDIS_MAX_RETRIES")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3),
            retry_delay: std::env::var("REDIS_RETRY_DELAY_MS")
                .ok()
                .and_then(|s| s.parse().ok())
                .map(Duration::from_millis)
                .unwrap_or(Duration::from_millis(100)),
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.max_size == 0 {
            return Err("max_size must be greater than 0".to_string());
        }

        if self.min_idle > self.max_size {
            return Err("min_idle cannot be greater than max_size".to_string());
        }

        if self.connection_timeout.as_secs() == 0 {
            return Err("connection_timeout must be greater than 0".to_string());
        }

        Ok(())
    }
}

/// Connection pool health monitoring
pub struct PoolHealthMonitor {
    db_pool: deadpool_postgres::Pool,
}

impl PoolHealthMonitor {
    pub fn new(db_pool: deadpool_postgres::Pool) -> Self {
        Self { db_pool }
    }

    /// Get current pool status
    pub fn get_status(&self) -> PoolStatus {
        let status = self.db_pool.status();

        PoolStatus {
            size: status.size,
            available: status.available,
            max_size: status.max_size,
            utilization_percent: if status.max_size > 0 {
                (status.size - status.available) as f64 / status.max_size as f64 * 100.0
            } else {
                0.0
            },
        }
    }

    /// Check if pool is healthy
    pub fn is_healthy(&self) -> bool {
        let status = self.get_status();

        // Pool is healthy if:
        // 1. There are available connections
        // 2. Utilization is below 90%
        status.available > 0 && status.utilization_percent < 90.0
    }

    /// Get pool health metrics
    pub async fn get_health_metrics(&self) -> Result<PoolHealthMetrics, String> {
        let status = self.get_status();

        // Test connection
        let start = std::time::Instant::now();
        let connection_test = self.db_pool.get().await;
        let connection_latency = start.elapsed();

        let connection_healthy = connection_test.is_ok();

        Ok(PoolHealthMetrics {
            status,
            connection_healthy,
            connection_latency_ms: connection_latency.as_millis() as u64,
        })
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PoolStatus {
    /// Current number of connections in the pool
    pub size: usize,

    /// Number of available (idle) connections
    pub available: usize,

    /// Maximum pool size
    pub max_size: usize,

    /// Pool utilization percentage
    pub utilization_percent: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PoolHealthMetrics {
    pub status: PoolStatus,
    pub connection_healthy: bool,
    pub connection_latency_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_pool_config_default() {
        let config = DatabasePoolConfig::default();
        assert_eq!(config.max_size, 50);
        assert_eq!(config.min_idle, 10);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_database_pool_config_validation() {
        // Test invalid max_size
        let mut config = DatabasePoolConfig {
            max_size: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());

        // Test invalid min_idle
        config.max_size = 10;
        config.min_idle = 20;
        assert!(config.validate().is_err());

        // Test valid config
        config.min_idle = 5;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_redis_pool_config_default() {
        let config = RedisPoolConfig::default();
        assert_eq!(config.max_size, 20);
        assert_eq!(config.min_idle, 5);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_pool_status_utilization() {
        let status = PoolStatus {
            size: 30,
            available: 10,
            max_size: 50,
            utilization_percent: 40.0,
        };

        assert_eq!(status.utilization_percent, 40.0);
    }
}
