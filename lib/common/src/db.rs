//! Database utilities for SIMPelv2
//!
//! Provides connection pool factory, prepared statement caching,
//! and transaction helper functions.

use anyhow::Result;
use deadpool_postgres::{Config, Pool, Runtime};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_postgres::{NoTls, Statement};

/// Standard database configuration shared across services
#[derive(Debug, Clone)]
pub struct DbConfig {
    pub url: String,
    pub max_size: usize,
    pub min_idle: Option<usize>,
    pub connection_timeout: Option<std::time::Duration>,
    pub idle_timeout: Option<std::time::Duration>,
    pub max_lifetime: Option<std::time::Duration>,
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            max_size: 20,
            min_idle: Some(5),
            connection_timeout: Some(std::time::Duration::from_secs(30)),
            idle_timeout: Some(std::time::Duration::from_secs(600)), // 10 minutes
            max_lifetime: Some(std::time::Duration::from_secs(1800)), // 30 minutes
        }
    }
}

impl DbConfig {
    pub fn new(url: String) -> Self {
        Self {
            url,
            ..Default::default()
        }
    }

    pub fn with_max_size(mut self, max_size: usize) -> Self {
        self.max_size = max_size;
        self
    }

    pub fn with_min_idle(mut self, min_idle: usize) -> Self {
        self.min_idle = Some(min_idle);
        self
    }

    pub fn with_connection_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.connection_timeout = Some(timeout);
        self
    }

    pub fn with_idle_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.idle_timeout = Some(timeout);
        self
    }

    pub fn with_max_lifetime(mut self, lifetime: std::time::Duration) -> Self {
        self.max_lifetime = Some(lifetime);
        self
    }
}

/// Create a standardized PostgreSQL connection pool
pub fn create_postgres_pool(config: DbConfig) -> Result<Pool> {
    let mut pg_config = Config::new();
    pg_config.url = Some(config.url);

    // Configure pool settings
    let mut pool_config = deadpool_postgres::PoolConfig::new(config.max_size);

    if let Some(timeout) = config.connection_timeout {
        pool_config.timeouts.wait = Some(timeout);
    }

    if let Some(timeout) = config.idle_timeout {
        pool_config.timeouts.recycle = Some(timeout);
    }

    pg_config.pool = Some(pool_config);

    let pool = pg_config.create_pool(Some(Runtime::Tokio1), NoTls)?;

    Ok(pool)
}

/// Prepared statement cache for query optimization
pub struct PreparedStatementCache {
    cache: Arc<RwLock<HashMap<String, Statement>>>,
}

impl PreparedStatementCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get or prepare a statement
    pub async fn get_or_prepare(
        &self,
        client: &tokio_postgres::Client,
        query: &str,
    ) -> Result<Statement> {
        // Check if statement is already cached
        {
            let cache = self.cache.read().await;
            if let Some(stmt) = cache.get(query) {
                return Ok(stmt.clone());
            }
        }

        // Prepare the statement
        let stmt = client.prepare(query).await?;

        // Cache it
        {
            let mut cache = self.cache.write().await;
            cache.insert(query.to_string(), stmt.clone());
        }

        Ok(stmt)
    }

    /// Clear the cache
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Get cache size
    pub async fn size(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }

    /// Remove a specific statement from cache
    pub async fn remove(&self, query: &str) -> Option<Statement> {
        let mut cache = self.cache.write().await;
        cache.remove(query)
    }
}

impl Default for PreparedStatementCache {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for PreparedStatementCache {
    fn clone(&self) -> Self {
        Self {
            cache: Arc::clone(&self.cache),
        }
    }
}

/// Transaction helper functions
pub struct TransactionHelper;

impl TransactionHelper {
    /// Execute a function within a transaction
    pub async fn execute<F, T>(pool: &Pool, f: F) -> Result<T>
    where
        F: for<'a> FnOnce(
            &'a tokio_postgres::Transaction<'a>,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<T>> + Send + 'a>,
        >,
    {
        let mut client = pool.get().await?;
        let tx = client.transaction().await?;

        match f(&tx).await {
            Ok(result) => {
                tx.commit().await?;
                Ok(result)
            }
            Err(e) => {
                tx.rollback().await?;
                Err(e)
            }
        }
    }

    /// Execute multiple operations in a transaction with automatic rollback on error
    pub async fn execute_batch<F>(pool: &Pool, operations: Vec<F>) -> Result<Vec<u64>>
    where
        F: for<'a> FnOnce(
            &'a tokio_postgres::Transaction<'a>,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<u64>> + Send + 'a>,
        >,
    {
        let mut client = pool.get().await?;
        let tx = client.transaction().await?;

        let mut results = Vec::new();

        for operation in operations {
            match operation(&tx).await {
                Ok(count) => results.push(count),
                Err(e) => {
                    tx.rollback().await?;
                    return Err(e);
                }
            }
        }

        tx.commit().await?;
        Ok(results)
    }
}

/// Database health check utilities
pub struct DbHealthCheck;

impl DbHealthCheck {
    /// Standard health check for database
    pub async fn check_db_health(pool: &Pool) -> Result<()> {
        let client = pool.get().await?;
        client.query("SELECT 1", &[]).await?;
        Ok(())
    }

    /// Detailed health check with connection pool stats
    pub async fn check_detailed(pool: &Pool) -> Result<DbHealthStatus> {
        // Check basic connectivity
        let start = std::time::Instant::now();
        Self::check_db_health(pool).await?;
        let latency = start.elapsed();

        // Get pool status
        let status = pool.status();

        Ok(DbHealthStatus {
            healthy: true,
            latency_ms: latency.as_millis() as u64,
            pool_size: status.size,
            available_connections: status.available,
            max_size: status.max_size,
        })
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DbHealthStatus {
    pub healthy: bool,
    pub latency_ms: u64,
    pub pool_size: usize,
    pub available_connections: usize,
    pub max_size: usize,
}

/// Query builder helper for common patterns
pub struct QueryBuilder {
    query: String,
    params: Vec<String>,
    param_count: usize,
}

impl QueryBuilder {
    pub fn new(base_query: &str) -> Self {
        Self {
            query: base_query.to_string(),
            params: Vec::new(),
            param_count: 0,
        }
    }

    pub fn add_condition(&mut self, condition: &str) -> &mut Self {
        if self.param_count == 0 {
            self.query.push_str(" WHERE ");
        } else {
            self.query.push_str(" AND ");
        }
        self.query.push_str(condition);
        self
    }

    pub fn add_param(&mut self, value: String) -> String {
        self.param_count += 1;
        self.params.push(value);
        format!("${}", self.param_count)
    }

    pub fn add_order_by(&mut self, column: &str, direction: &str) -> &mut Self {
        self.query
            .push_str(&format!(" ORDER BY {} {}", column, direction));
        self
    }

    pub fn add_limit(&mut self, limit: i64) -> &mut Self {
        self.param_count += 1;
        self.query.push_str(&format!(" LIMIT ${}", self.param_count));
        self.params.push(limit.to_string());
        self
    }

    pub fn add_offset(&mut self, offset: i64) -> &mut Self {
        self.param_count += 1;
        self.query.push_str(&format!(" OFFSET ${}", self.param_count));
        self.params.push(offset.to_string());
        self
    }

    pub fn build(self) -> (String, Vec<String>) {
        (self.query, self.params)
    }

    pub fn get_query(&self) -> &str {
        &self.query
    }

    pub fn get_params(&self) -> &[String] {
        &self.params
    }
}

/// Pagination helper
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Pagination {
    pub page: i64,
    pub page_size: i64,
    pub total_count: Option<i64>,
}

impl Pagination {
    pub fn new(page: i64, page_size: i64) -> Self {
        Self {
            page: page.max(1),
            page_size: page_size.clamp(1, 100),
            total_count: None,
        }
    }

    pub fn with_total_count(mut self, total: i64) -> Self {
        self.total_count = Some(total);
        self
    }

    pub fn offset(&self) -> i64 {
        (self.page - 1) * self.page_size
    }

    pub fn limit(&self) -> i64 {
        self.page_size
    }

    pub fn total_pages(&self) -> Option<i64> {
        self.total_count
            .map(|total| (total + self.page_size - 1) / self.page_size)
    }

    pub fn has_next_page(&self) -> Option<bool> {
        self.total_pages().map(|total| self.page < total)
    }

    pub fn has_prev_page(&self) -> bool {
        self.page > 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_builder() {
        let mut builder = QueryBuilder::new("SELECT * FROM users");
        builder.add_condition("username = $1");
        builder.add_condition("enabled = $2");
        builder.add_order_by("created_at", "DESC");
        builder.add_limit(10);
        builder.add_offset(0);

        let (query, _params) = builder.build();
        assert!(query.contains("WHERE"));
        assert!(query.contains("AND"));
        assert!(query.contains("ORDER BY"));
        assert!(query.contains("LIMIT"));
        assert!(query.contains("OFFSET"));
    }

    #[test]
    fn test_pagination() {
        let pagination = Pagination::new(2, 10).with_total_count(45);

        assert_eq!(pagination.offset(), 10);
        assert_eq!(pagination.limit(), 10);
        assert_eq!(pagination.total_pages(), Some(5));
        assert_eq!(pagination.has_next_page(), Some(true));
        assert_eq!(pagination.has_prev_page(), true);
    }

    #[test]
    fn test_pagination_bounds() {
        let pagination = Pagination::new(0, 200);

        assert_eq!(pagination.page, 1); // Minimum page is 1
        assert_eq!(pagination.page_size, 100); // Maximum page size is 100
    }
}
