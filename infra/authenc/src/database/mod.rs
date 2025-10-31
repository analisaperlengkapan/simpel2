use deadpool_postgres::{Pool, Runtime};
use tokio_postgres::NoTls;
use tracing::{error, info};

use crate::{
    config::DatabaseConfig,
    error::{AuthencError, Result},
};

/// Database connection pool manager
#[derive(Clone)] // Derive Clone for easy sharing across handlers
pub struct Database {
    pool: Pool,
    /// Prepared statement cache for improved performance
    prepared_cache: PreparedStatementCache,
    /// Pool metrics for monitoring
    metrics: std::sync::Arc<PoolMetrics>,
}

impl std::fmt::Debug for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Database")
            .field("pool", &"Pool")
            .field(
                "prepared_cache",
                &format!("{} cached statements", self.prepared_cache.len()),
            )
            .field("metrics", &self.metrics)
            .finish()
    }
}

/// Pool metrics for monitoring connection pool health
#[derive(Debug, Default)]
pub struct PoolMetrics {
    /// Total connections acquired
    pub connections_acquired: std::sync::atomic::AtomicU64,
    /// Total connection acquisition failures
    pub acquisition_failures: std::sync::atomic::AtomicU64,
    /// Total connection acquisition time in microseconds
    pub total_acquisition_time_us: std::sync::atomic::AtomicU64,
    /// Number of connection acquisitions
    pub acquisition_count: std::sync::atomic::AtomicU64,
}

impl PoolMetrics {
    /// Create new pool metrics
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a successful connection acquisition
    pub fn record_acquisition(&self, duration: std::time::Duration) {
        self.connections_acquired
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.total_acquisition_time_us.fetch_add(
            duration.as_micros() as u64,
            std::sync::atomic::Ordering::Relaxed,
        );
        self.acquisition_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Record a failed connection acquisition
    pub fn record_failure(&self) {
        self.acquisition_failures
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Get average acquisition time in microseconds
    pub fn avg_acquisition_time_us(&self) -> u64 {
        let count = self
            .acquisition_count
            .load(std::sync::atomic::Ordering::Relaxed);
        if count == 0 {
            return 0;
        }
        let total = self
            .total_acquisition_time_us
            .load(std::sync::atomic::Ordering::Relaxed);
        total / count
    }

    /// Get total connections acquired
    pub fn total_acquired(&self) -> u64 {
        self.connections_acquired
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Get total acquisition failures
    pub fn total_failures(&self) -> u64 {
        self.acquisition_failures
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl Database {
    /// Create new database connection pool with optimized configuration
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        use std::time::Duration;

        // Build optimized pool configuration
        let pool_config = PoolConfigBuilder::new()
            .max_size(config.max_connections as usize)
            .min_idle(config.min_connections as usize)
            .timeout(Duration::from_secs(config.connection_timeout))
            .idle_timeout(Duration::from_secs(config.idle_timeout))
            .max_lifetime(Duration::from_secs(config.max_lifetime))
            .recycling_method(deadpool_postgres::RecyclingMethod::Verified)
            .build();

        let manager_config = PoolConfigBuilder::new()
            .recycling_method(deadpool_postgres::RecyclingMethod::Verified)
            .build_manager_config();

        let cfg = deadpool_postgres::Config {
            host: Some(config.host.clone()),
            port: Some(config.port),
            user: Some(config.username.clone()),
            password: Some(config.password.clone()),
            dbname: Some(config.database.clone()),
            pool: Some(pool_config),
            manager: Some(manager_config),
            ..Default::default()
        };

        let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls).map_err(|e| {
            error!("Failed to create database pool: {}", e);
            AuthencError::database("Failed to create database pool")
        })?;

        // Test connection
        match pool.get().await {
            Ok(_) => {
                info!(
                    "✅ Database connection pool established to {}:{}/{} (max: {}, min: {})",
                    config.host, config.port, config.database, config.max_connections, config.min_connections
                );
                Ok(Self {
                    pool,
                    prepared_cache: PreparedStatementCache::new(1000),
                    metrics: std::sync::Arc::new(PoolMetrics::new()),
                })
            }
            Err(e) => {
                error!("Failed to connect to database: {}", e);
                Err(AuthencError::database("Failed to connect to database"))
            }
        }
    }

    /// Get a database connection from the pool with metrics tracking
    pub async fn get_connection(&self) -> Result<deadpool_postgres::Client> {
        let start = std::time::Instant::now();

        match self.pool.get().await {
            Ok(conn) => {
                let duration = start.elapsed();
                self.metrics.record_acquisition(duration);
                Ok(conn)
            }
            Err(e) => {
                self.metrics.record_failure();
                error!("Failed to get database connection: {}", e);
                Err(AuthencError::database("Failed to get database connection"))
            }
        }
    }

    /// Get the database connection pool
    pub fn get_pool(&self) -> deadpool_postgres::Pool {
        self.pool.clone()
    }

    /// Execute a read-only query and return results
    pub async fn query<T>(
        &self,
        statement: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<Vec<T>>
    where
        T: Send + 'static + TryFrom<tokio_postgres::Row>,
        <T as TryFrom<tokio_postgres::Row>>::Error: std::fmt::Debug,
    {
        let client = self.get_connection().await?;
        let rows = client.query(statement, params).await.map_err(|e| {
            error!("Query failed: {}\nStatement: {}", e, statement);
            AuthencError::database(format!("Database query failed: {}", e))
        })?;

        let mut results = Vec::with_capacity(rows.len());
        for row in rows {
            results.push(row.try_into().map_err(|e| {
                error!("Failed to convert row: {:?}", e);
                AuthencError::database("Failed to convert database row")
            })?);
        }

        Ok(results)
    }

    /// Execute a read-only query and return raw rows
    pub async fn query_raw(
        &self,
        statement: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<Vec<tokio_postgres::Row>> {
        let client = self.get_connection().await?;
        client.query(statement, params).await.map_err(|e| {
            error!("Query failed: {}\nStatement: {}", e, statement);
            AuthencError::database(format!("Database query failed: {}", e))
        })
    }

    /// Execute a query that returns a single row
    pub async fn query_one<T>(
        &self,
        statement: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<T>
    where
        T: Send + 'static + TryFrom<tokio_postgres::Row>,
        <T as TryFrom<tokio_postgres::Row>>::Error: std::fmt::Debug,
    {
        let client = self.get_connection().await?;
        let row = client.query_one(statement, params).await.map_err(|e| {
            error!("Query one failed: {}\nStatement: {}", e, statement);
            AuthencError::database(format!("Database query failed: {}", e))
        })?;

        row.try_into().map_err(|e| {
            error!("Failed to convert row: {:?}", e);
            AuthencError::database("Failed to convert database row")
        })
    }

    /// Execute a query that returns an optional single row
    pub async fn query_opt(
        &self,
        statement: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<Option<tokio_postgres::Row>> {
        let client = self.get_connection().await?;
        client.query_opt(statement, params).await.map_err(|e| {
            error!("Query opt failed: {}\nStatement: {}", e, statement);
            AuthencError::database(format!("Database query failed: {}", e))
        })
    }

    /// Execute a statement that doesn't return any rows
    pub async fn execute(
        &self,
        statement: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<u64> {
        let client = self.get_connection().await?;
        client.execute(statement, params).await.map_err(|e| {
            error!("Execute failed: {}\nStatement: {}", e, statement);
            AuthencError::database(format!("Database execute failed: {}", e))
        })
    }

    /// Execute multiple queries within a single connection
    /// Note: For true transactions, use the database client directly
    pub async fn execute_batch<F, Fut, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&deadpool_postgres::Client) -> Fut,
        Fut: std::future::Future<Output = Result<R>>,
        R: Send,
    {
        let client = self.get_connection().await?;
        f(&client).await
    }

    /// Check if database is healthy with connection validation
    pub async fn health_check(&self) -> Result<()> {
        match self.get_connection().await {
            Ok(conn) => conn.query("SELECT 1", &[]).await.map(|_| ()).map_err(|e| {
                error!("Database health check failed: {}", e);
                AuthencError::database("Database health check failed")
            }),
            Err(e) => {
                error!("Database connection failed: {}", e);
                Err(e)
            }
        }
    }

    /// Get pool health metrics
    pub fn pool_health(&self) -> PoolHealth {
        let status = self.pool.status();
        let size = status.size;
        let max_size = status.max_size;
        let available = status.available;
        let utilization = if max_size > 0 {
            ((size - available) as f64 / max_size as f64) * 100.0
        } else {
            0.0
        };

        PoolHealth {
            size,
            max_size,
            available,
            utilization,
        }
    }

    /// Get pool metrics for monitoring
    pub fn pool_metrics(&self) -> &PoolMetrics {
        &self.metrics
    }

    /// Get detailed pool statistics
    pub fn pool_stats(&self) -> PoolStats {
        let status = self.pool.status();
        let health = self.pool_health();

        PoolStats {
            size: status.size,
            max_size: status.max_size,
            available: status.available,
            waiting: status.waiting,
            utilization: health.utilization,
            total_acquired: self.metrics.total_acquired(),
            total_failures: self.metrics.total_failures(),
            avg_acquisition_time_us: self.metrics.avg_acquisition_time_us(),
        }
    }

    /// Perform periodic connection validation
    /// This should be called periodically (e.g., every 30 seconds) to ensure connections are healthy
    pub async fn validate_connections(&self) -> Result<ValidationResult> {
        let start = std::time::Instant::now();
        let health = self.pool_health();

        // Try to acquire a connection and validate it
        match self.get_connection().await {
            Ok(conn) => {
                match conn.query("SELECT 1", &[]).await {
                    Ok(_) => {
                        let duration = start.elapsed();
                        Ok(ValidationResult {
                            healthy: true,
                            validation_time: duration,
                            pool_health: health,
                            error: None,
                        })
                    }
                    Err(e) => {
                        let duration = start.elapsed();
                        Ok(ValidationResult {
                            healthy: false,
                            validation_time: duration,
                            pool_health: health,
                            error: Some(format!("Query validation failed: {}", e)),
                        })
                    }
                }
            }
            Err(e) => {
                let duration = start.elapsed();
                Ok(ValidationResult {
                    healthy: false,
                    validation_time: duration,
                    pool_health: health,
                    error: Some(format!("Connection acquisition failed: {}", e)),
                })
            }
        }
    }
}

/// Detailed pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    /// Current pool size
    pub size: usize,
    /// Maximum pool size
    pub max_size: usize,
    /// Number of available connections
    pub available: usize,
    /// Number of waiting requests
    pub waiting: usize,
    /// Pool utilization percentage
    pub utilization: f64,
    /// Total connections acquired
    pub total_acquired: u64,
    /// Total acquisition failures
    pub total_failures: u64,
    /// Average acquisition time in microseconds
    pub avg_acquisition_time_us: u64,
}

/// Connection validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the pool is healthy
    pub healthy: bool,
    /// Time taken to validate
    pub validation_time: std::time::Duration,
    /// Pool health snapshot
    pub pool_health: PoolHealth,
    /// Error message if validation failed
    pub error: Option<String>,
}

// Mock database for testing
impl Database {
    /// Create a mock database for testing
    pub async fn mock() -> Self {
        use deadpool_postgres::{Manager, Pool};
        use std::env;
        use tokio_postgres::NoTls;

        // Use test database if specified, otherwise use mock
        if let Ok(_database_url) = env::var("TEST_DATABASE_URL") {
            // Use real test database if URL is provided
            let config = DatabaseConfig {
                host: "localhost".to_string(),
                port: 5432,
                username: "postgres".to_string(),
                password: "postgres".to_string(),
                database: "test_authenc".to_string(),
                max_connections: 5,
                min_connections: 1,
                connection_timeout: 5,
                idle_timeout: 60,
                max_lifetime: 120,
                audit_log_url: None,
                connection_timeout_seconds: 5,
            };

            if let Ok(db) = Database::new(&config).await {
                return db;
            }
        }

        // Fall back to mock implementation
        let mut config = tokio_postgres::Config::new();
        config
            .user("test")
            .password("test")
            .host("localhost")
            .port(5432)
            .dbname("test");

        let manager = Manager::new(config, NoTls);
        let pool = Pool::builder(manager)
            .max_size(1)
            .build()
            .expect("Failed to create mock database pool");

        Self {
            pool,
            prepared_cache: PreparedStatementCache::new(1),
            metrics: std::sync::Arc::new(PoolMetrics::new()),
        }
    }

    /// Execute work within a transaction
    ///
    /// The provided closure receives the client and can perform multiple operations.
    /// The transaction is automatically committed if the closure succeeds, or rolled back on error.
    pub async fn with_transaction<F, R>(&self, f: F) -> Result<R>
    where
        F: for<'a> FnOnce(
            &'a deadpool_postgres::Client,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<R>> + Send + 'a>,
        >,
        R: Send,
    {
        let client = self.get_connection().await?;

        // Begin transaction
        client.execute("BEGIN", &[]).await.map_err(|e| {
            error!("Failed to begin transaction: {}", e);
            AuthencError::database(format!("Failed to begin transaction: {}", e))
        })?;

        match f(&client).await {
            Ok(result) => {
                client.execute("COMMIT", &[]).await.map_err(|e| {
                    error!("Failed to commit transaction: {}", e);
                    AuthencError::database(format!("Failed to commit transaction: {}", e))
                })?;
                Ok(result)
            }
            Err(e) => {
                let _ = client.execute("ROLLBACK", &[]).await;
                Err(e)
            }
        }
    }

    /// Get the prepared statement cache
    pub fn prepared_cache(&self) -> &PreparedStatementCache {
        &self.prepared_cache
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        self.prepared_cache.stats()
    }

    /// Execute batch operations
    ///
    /// The provided closure receives a BatchOperations builder for bulk inserts, updates, deletes.
    pub async fn with_batch_operations<F, R>(&self, f: F) -> Result<R>
    where
        F: for<'a> FnOnce(
            BatchOperations<'a>,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<R>> + Send + 'a>,
        >,
        R: Send,
    {
        let client = self.get_connection().await?;
        let batch_ops = BatchOperations::new(&client);
        f(batch_ops).await
    }

    /// Clear prepared statement cache (useful for schema changes)
    pub fn clear_prepared_cache(&self) {
        self.prepared_cache.clear();
    }

    /// Execute a query using prepared statements
    pub async fn query_prepared(
        &self,
        statement: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<Vec<tokio_postgres::Row>> {
        let client = self.get_connection().await?;
        let stmt = self.prepared_cache.get_or_prepare(&client, statement).await?;

        client.query(stmt.as_ref(), params).await.map_err(|e| {
            error!("Prepared query failed: {}\nStatement: {}", e, statement);
            AuthencError::database(format!("Database query failed: {}", e))
        })
    }

    /// Execute a query that returns a single row using prepared statements
    pub async fn query_one_prepared(
        &self,
        statement: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<tokio_postgres::Row> {
        let client = self.get_connection().await?;
        let stmt = self.prepared_cache.get_or_prepare(&client, statement).await?;

        client.query_one(stmt.as_ref(), params).await.map_err(|e| {
            error!("Prepared query one failed: {}\nStatement: {}", e, statement);
            AuthencError::database(format!("Database query failed: {}", e))
        })
    }

    /// Execute a query that returns an optional single row using prepared statements
    pub async fn query_opt_prepared(
        &self,
        statement: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<Option<tokio_postgres::Row>> {
        let client = self.get_connection().await?;
        let stmt = self.prepared_cache.get_or_prepare(&client, statement).await?;

        client.query_opt(stmt.as_ref(), params).await.map_err(|e| {
            error!("Prepared query opt failed: {}\nStatement: {}", e, statement);
            AuthencError::database(format!("Database query failed: {}", e))
        })
    }

    /// Execute a statement that doesn't return any rows using prepared statements
    pub async fn execute_prepared(
        &self,
        statement: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<u64> {
        let client = self.get_connection().await?;
        let stmt = self.prepared_cache.get_or_prepare(&client, statement).await?;

        client.execute(stmt.as_ref(), params).await.map_err(|e| {
            error!("Prepared execute failed: {}\nStatement: {}", e, statement);
            AuthencError::database(format!("Database execute failed: {}", e))
        })
    }
}

/// Database migration utilities and schema management
///
/// This module contains database migration scripts and utilities for managing
/// schema changes, version control, and database upgrades. Migrations ensure
/// that the database schema remains consistent across different deployments
/// and versions of the authentication platform.
pub mod migrations;

/// Database operation utilities and transaction management
///
/// This module provides high-level database operations and transaction management
/// utilities for common database tasks. It includes connection pooling, query
/// execution, and error handling for database operations.
pub mod operations;
/// Database module exports
pub mod queries;

/// Transaction management for Keycloak-like transaction semantics
pub mod transaction;

/// Prepared statement caching for improved performance
pub mod prepared_cache;

/// Batch operations for efficient bulk database operations
pub mod batch;

/// Advanced connection pool configuration
pub mod pool_config;

/// CAPTCHA database operations
pub mod captcha_operations;

/// Audit operations with tamper-proof signatures
pub mod audit_operations;

/// Specialized batch operations for performance optimization
pub mod batch_operations;

// Re-export commonly used types
pub use audit_operations::{
    store_admin_event_with_signature, store_event_with_signature, verify_admin_event_signature,
    verify_event_signature,
};
pub use batch::{BatchInsertable, BatchOperations, BatchUpdateable};
pub use batch_operations::{
    batch_insert_audit_logs, batch_lookup_users, batch_query_user_permissions,
    batch_validate_sessions, AuditLogEntry, SessionValidationResult,
};
pub use captcha_operations::{CaptchaAnalyticsSummary, CaptchaOperations, ValidationAttempt};
pub use pool_config::{PoolConfigBuilder, PoolHealth};
pub use prepared_cache::{CacheStats, PreparedStatementCache};
pub use transaction::{DatabaseTransaction, IsolationLevel, TransactionManager};
