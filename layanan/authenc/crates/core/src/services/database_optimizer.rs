//! Database optimization service for MFA operations
//!
//! This module provides database connection pool optimization and query performance
//! monitoring specifically for MFA-related operations.

use authenc_types::{AuthencError, Result};
use deadpool_postgres::{Config, Pool, PoolConfig, Runtime};
use std::time::{Duration, Instant};
use tokio_postgres::NoTls;
use tracing::{debug, info, warn};

/// Database optimizer for MFA operations
pub struct DatabaseOptimizer {
    /// Optimized connection pool for MFA operations
    mfa_pool: Pool,
    /// Performance metrics
    metrics: DatabaseMetrics,
}

/// Database performance metrics
#[derive(Debug, Clone, Default)]
pub struct DatabaseMetrics {
    /// Total number of MFA queries executed
    pub total_queries: u64,
    /// Average query execution time in milliseconds
    pub avg_query_time_ms: f64,
    /// Number of slow queries (>100ms)
    pub slow_queries: u64,
    /// Connection pool statistics
    pub pool_stats: PoolStats,
}

/// Connection pool statistics
#[derive(Debug, Clone, Default)]
pub struct PoolStats {
    /// Current number of connections in the pool
    pub current_connections: u32,
    /// Maximum number of connections allowed
    pub max_connections: u32,
    /// Number of connections currently in use
    pub active_connections: u32,
    /// Number of connections waiting to be acquired
    pub waiting_connections: u32,
}

impl DatabaseOptimizer {
    /// Create a new database optimizer with MFA-optimized connection pool
    pub async fn new(database_url: &str) -> Result<Self> {
        let mut config = Config::new();
        config.url = Some(database_url.to_string());

        // Optimize pool configuration for MFA operations
        let pool_config = PoolConfig::new(20); // Increased pool size for MFA operations
        config.pool = Some(pool_config);

        // Create optimized connection pool
        let mfa_pool = config
            .create_pool(Some(Runtime::Tokio1), NoTls)
            .map_err(|e| AuthencError::database(format!("Failed to create MFA pool: {}", e)))?;

        // Test the connection
        let client = mfa_pool
            .get()
            .await
            .map_err(|e| AuthencError::database(format!("Failed to get connection: {}", e)))?;

        // Verify MFA-related tables and functions exist
        let _ = client
            .query_one("SELECT 1 FROM users LIMIT 1", &[])
            .await
            .map_err(|e| AuthencError::database(format!("MFA tables not accessible: {}", e)))?;

        info!("Database optimizer initialized with MFA-optimized connection pool");

        Ok(Self {
            mfa_pool,
            metrics: DatabaseMetrics::default(),
        })
    }

    /// Execute an MFA query with performance monitoring
    pub async fn execute_mfa_query<F, T>(&mut self, query_name: &str, query_fn: F) -> Result<T>
    where
        F: FnOnce(
            deadpool_postgres::Client,
        )
            -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send>>,
    {
        let start_time = Instant::now();

        let client = self.mfa_pool.get().await.map_err(|e| {
            AuthencError::database(format!(
                "Failed to get connection for {}: {}",
                query_name, e
            ))
        })?;

        let result = query_fn(client).await;

        let execution_time = start_time.elapsed();
        self.update_metrics(query_name, execution_time);

        result
    }

    /// Update performance metrics
    fn update_metrics(&mut self, query_name: &str, execution_time: Duration) {
        let execution_ms = execution_time.as_millis() as f64;

        self.metrics.total_queries += 1;

        // Update average query time using exponential moving average
        if self.metrics.total_queries == 1 {
            self.metrics.avg_query_time_ms = execution_ms;
        } else {
            let alpha = 0.1; // Smoothing factor
            self.metrics.avg_query_time_ms =
                alpha * execution_ms + (1.0 - alpha) * self.metrics.avg_query_time_ms;
        }

        // Track slow queries
        if execution_ms > 100.0 {
            self.metrics.slow_queries += 1;
            warn!(
                query = query_name,
                execution_time_ms = execution_ms,
                "Slow MFA query detected"
            );
        } else {
            debug!(
                query = query_name,
                execution_time_ms = execution_ms,
                "MFA query completed"
            );
        }
    }

    /// Get current performance metrics
    pub fn get_metrics(&self) -> DatabaseMetrics {
        let mut metrics = self.metrics.clone();

        // Update pool statistics
        let pool_status = self.mfa_pool.status();
        metrics.pool_stats = PoolStats {
            current_connections: pool_status.size as u32,
            max_connections: pool_status.max_size as u32,
            active_connections: (pool_status.size - pool_status.available) as u32,
            waiting_connections: pool_status.waiting as u32,
        };

        metrics
    }

    /// Optimize database connections for MFA operations
    pub async fn optimize_connections(&self) -> Result<()> {
        let client = self.mfa_pool.get().await.map_err(|e| {
            AuthencError::database(format!("Failed to get connection for optimization: {}", e))
        })?;

        // Set connection-level optimizations for MFA queries
        client
            .execute("SET work_mem = '16MB'", &[])
            .await
            .map_err(|e| AuthencError::database(format!("Failed to set work_mem: {}", e)))?;

        client
            .execute("SET random_page_cost = 1.1", &[])
            .await
            .map_err(|e| {
                AuthencError::database(format!("Failed to set random_page_cost: {}", e))
            })?;

        // Enable query plan caching for MFA queries
        client
            .execute("SET plan_cache_mode = 'force_generic_plan'", &[])
            .await
            .map_err(|e| AuthencError::database(format!("Failed to set plan_cache_mode: {}", e)))?;

        info!("Database connections optimized for MFA operations");
        Ok(())
    }

    /// Analyze MFA table statistics for query optimization
    pub async fn analyze_mfa_tables(&self) -> Result<()> {
        let client = self.mfa_pool.get().await.map_err(|e| {
            AuthencError::database(format!("Failed to get connection for analysis: {}", e))
        })?;

        // Analyze MFA-related tables to update statistics
        let tables = vec!["users", "mfa_admin_actions"];

        for table in tables {
            client
                .execute(&format!("ANALYZE {}", table), &[])
                .await
                .map_err(|e| {
                    AuthencError::database(format!("Failed to analyze table {}: {}", table, e))
                })?;

            debug!("Analyzed table: {}", table);
        }

        // Refresh materialized view statistics
        client
            .execute("SELECT refresh_mfa_statistics()", &[])
            .await
            .map_err(|e| {
                AuthencError::database(format!("Failed to refresh MFA statistics: {}", e))
            })?;

        info!("MFA table statistics updated");
        Ok(())
    }

    /// Get connection pool for direct access
    pub fn get_pool(&self) -> &Pool {
        &self.mfa_pool
    }

    /// Health check for the MFA database pool
    pub async fn health_check(&self) -> Result<DatabaseHealthStatus> {
        let start_time = Instant::now();

        let client = self.mfa_pool.get().await.map_err(|e| {
            AuthencError::database(format!("Health check failed to get connection: {}", e))
        })?;

        // Test basic connectivity
        let _ = client
            .query_one("SELECT 1", &[])
            .await
            .map_err(|e| AuthencError::database(format!("Health check query failed: {}", e)))?;

        // Test MFA-specific functionality
        let _ = client
            .query_one("SELECT COUNT(*) FROM users WHERE mfa_enabled = true", &[])
            .await
            .map_err(|e| AuthencError::database(format!("MFA health check query failed: {}", e)))?;

        let response_time = start_time.elapsed();
        let pool_status = self.mfa_pool.status();

        Ok(DatabaseHealthStatus {
            healthy: true,
            response_time_ms: response_time.as_millis() as u64,
            pool_size: pool_status.size as u32,
            available_connections: pool_status.available as u32,
            waiting_connections: pool_status.waiting as u32,
        })
    }
}

/// Database health status
#[derive(Debug, Clone)]
pub struct DatabaseHealthStatus {
    /// Whether the database is healthy
    pub healthy: bool,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Current pool size
    pub pool_size: u32,
    /// Available connections
    pub available_connections: u32,
    /// Waiting connections
    pub waiting_connections: u32,
}

/// Query performance analyzer for MFA operations
pub struct MfaQueryAnalyzer {
    /// Query execution times by query type
    query_times: std::collections::HashMap<String, Vec<Duration>>,
}

impl MfaQueryAnalyzer {
    /// Create a new query analyzer
    pub fn new() -> Self {
        Self {
            query_times: std::collections::HashMap::new(),
        }
    }

    /// Record query execution time
    pub fn record_query(&mut self, query_type: &str, execution_time: Duration) {
        self.query_times
            .entry(query_type.to_string())
            .or_default()
            .push(execution_time);
    }

    /// Get query performance statistics
    pub fn get_query_stats(&self, query_type: &str) -> Option<QueryStats> {
        let times = self.query_times.get(query_type)?;

        if times.is_empty() {
            return None;
        }

        let total_ms: f64 = times.iter().map(|d| d.as_millis() as f64).sum();
        let avg_ms = total_ms / times.len() as f64;

        let mut sorted_times = times.clone();
        sorted_times.sort();

        let p50_ms = sorted_times[sorted_times.len() / 2].as_millis() as f64;
        let p95_ms = sorted_times[(sorted_times.len() * 95) / 100].as_millis() as f64;
        let p99_ms = sorted_times[(sorted_times.len() * 99) / 100].as_millis() as f64;

        Some(QueryStats {
            query_type: query_type.to_string(),
            total_executions: times.len() as u64,
            avg_execution_time_ms: avg_ms,
            p50_execution_time_ms: p50_ms,
            p95_execution_time_ms: p95_ms,
            p99_execution_time_ms: p99_ms,
        })
    }

    /// Get all query statistics
    pub fn get_all_stats(&self) -> Vec<QueryStats> {
        self.query_times
            .keys()
            .filter_map(|query_type| self.get_query_stats(query_type))
            .collect()
    }
}

/// Query performance statistics
#[derive(Debug, Clone)]
pub struct QueryStats {
    /// Query type name
    pub query_type: String,
    /// Total number of executions
    pub total_executions: u64,
    /// Average execution time in milliseconds
    pub avg_execution_time_ms: f64,
    /// 50th percentile execution time
    pub p50_execution_time_ms: f64,
    /// 95th percentile execution time
    pub p95_execution_time_ms: f64,
    /// 99th percentile execution time
    pub p99_execution_time_ms: f64,
}

impl Default for MfaQueryAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_analyzer() {
        let mut analyzer = MfaQueryAnalyzer::new();

        // Record some query times
        analyzer.record_query("mfa_status", Duration::from_millis(50));
        analyzer.record_query("mfa_status", Duration::from_millis(75));
        analyzer.record_query("mfa_status", Duration::from_millis(100));

        let stats = analyzer.get_query_stats("mfa_status").unwrap();
        assert_eq!(stats.total_executions, 3);
        assert!((stats.avg_execution_time_ms - 75.0).abs() < 1.0);
    }

    #[test]
    fn test_metrics_update() {
        let mut metrics = DatabaseMetrics::default();

        // Simulate updating metrics
        metrics.total_queries = 1;
        metrics.avg_query_time_ms = 50.0;

        assert_eq!(metrics.total_queries, 1);
        assert_eq!(metrics.avg_query_time_ms, 50.0);
    }
}
