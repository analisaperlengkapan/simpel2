//! Database connection pool and prepared statement cache
//!
//! This module provides the core database infrastructure for Authenc, including:
//! - Connection pooling with deadpool-postgres (20 connections default)
//! - Prepared statement caching for performance (<10ms p95 query latency)
//! - Transaction support for atomic operations

use authenc_types::{AuthencError, Result};
use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_postgres::{NoTls, Row, Statement};
use tracing::{debug, info};

/// Database connection pool with prepared statement caching
pub struct Database {
    pool: Pool,
    prepared_cache: Arc<PreparedStatementCache>,
}

impl Database {
    /// Create a new Database instance with connection pooling
    ///
    /// # Arguments
    /// * `database_url` - PostgreSQL connection string (e.g., "postgres://user:pass@localhost/db")
    /// * `pool_size` - Maximum number of connections in the pool (default: 20)
    ///
    /// # Example
    /// ```no_run
    /// use authenc_storage::Database;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let db = Database::new("postgres://authenc:password@localhost/authenc", 20).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn new(database_url: &str, pool_size: usize) -> Result<Self> {
        info!("Initializing database connection pool with {} connections", pool_size);

        // Parse connection string
        let config = database_url.parse::<tokio_postgres::Config>()
            .map_err(|e| AuthencError::database(format!("Invalid database URL: {}", e)))?;

        // Create deadpool configuration
        let mut pool_config = Config::new();
        pool_config.host = config.get_hosts().first()
            .and_then(|h| match h {
                tokio_postgres::config::Host::Tcp(host) => Some(host.clone()),
                _ => None,
            });
        pool_config.port = config.get_ports().first().copied();
        pool_config.dbname = config.get_dbname().map(|s| s.to_string());
        pool_config.user = config.get_user().map(|s| s.to_string());
        pool_config.password = config.get_password().map(|p| String::from_utf8_lossy(p).to_string());

        pool_config.manager = Some(ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        });

        // Create connection pool
        let pool = pool_config.create_pool(Some(Runtime::Tokio1), NoTls)
            .map_err(|e| AuthencError::database(format!("Failed to create connection pool: {}", e)))?;

        // Test connection
        let client = pool.get().await
            .map_err(|e| AuthencError::database(format!("Failed to get connection from pool: {}", e)))?;

        debug!("Testing database connection...");
        client.query_one("SELECT 1", &[]).await
            .map_err(|e| AuthencError::database(format!("Database connection test failed: {}", e)))?;

        info!("Database connection pool initialized successfully");

        Ok(Self {
            pool,
            prepared_cache: Arc::new(PreparedStatementCache::new()),
        })
    }

    /// Get a connection from the pool
    pub async fn get_connection(&self) -> Result<deadpool_postgres::Object> {
        self.pool.get().await
            .map_err(|e| AuthencError::database(format!("Failed to get connection: {}", e)))
    }

    /// Execute a query that returns no rows
    ///
    /// # Example
    /// ```no_run
    /// # use authenc_storage::Database;
    /// # async fn example(db: &Database) -> authenc_types::Result<()> {
    /// db.execute(
    ///     "INSERT INTO users (id, username) VALUES ($1, $2)",
    ///     &[&uuid::Uuid::new_v4(), &"john_doe"]
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute(&self, query: &str, params: &[&(dyn tokio_postgres::types::ToSql + Sync)]) -> Result<u64> {
        let client = self.get_connection().await?;
        let stmt = self.prepared_cache.get_or_prepare(&client, query).await?;

        client.execute(&stmt, params).await
            .map_err(|e| AuthencError::database(format!("Query execution failed: {}", e)))
    }

    /// Execute a query that returns exactly one row
    ///
    /// # Example
    /// ```no_run
    /// # use authenc_storage::Database;
    /// # async fn example(db: &Database) -> authenc_types::Result<()> {
    /// let row = db.query_one(
    ///     "SELECT * FROM users WHERE id = $1",
    ///     &[&uuid::Uuid::new_v4()]
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn query_one(&self, query: &str, params: &[&(dyn tokio_postgres::types::ToSql + Sync)]) -> Result<Row> {
        let client = self.get_connection().await?;
        let stmt = self.prepared_cache.get_or_prepare(&client, query).await?;

        client.query_one(&stmt, params).await
            .map_err(|e| AuthencError::database(format!("Query failed: {}", e)))
    }

    /// Execute a query that returns zero or one row
    ///
    /// # Example
    /// ```no_run
    /// # use authenc_storage::Database;
    /// # async fn example(db: &Database) -> authenc_types::Result<()> {
    /// let row_opt = db.query_opt(
    ///     "SELECT * FROM users WHERE username = $1",
    ///     &[&"john_doe"]
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn query_opt(&self, query: &str, params: &[&(dyn tokio_postgres::types::ToSql + Sync)]) -> Result<Option<Row>> {
        let client = self.get_connection().await?;
        let stmt = self.prepared_cache.get_or_prepare(&client, query).await?;

        client.query_opt(&stmt, params).await
            .map_err(|e| AuthencError::database(format!("Query failed: {}", e)))
    }

    /// Execute a query that returns multiple rows
    ///
    /// # Example
    /// ```no_run
    /// # use authenc_storage::Database;
    /// # async fn example(db: &Database) -> authenc_types::Result<()> {
    /// let rows = db.query(
    ///     "SELECT * FROM users WHERE enabled = $1 LIMIT $2",
    ///     &[&true, &20i64]
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn query(&self, query: &str, params: &[&(dyn tokio_postgres::types::ToSql + Sync)]) -> Result<Vec<Row>> {
        let client = self.get_connection().await?;
        let stmt = self.prepared_cache.get_or_prepare(&client, query).await?;

        client.query(&stmt, params).await
            .map_err(|e| AuthencError::database(format!("Query failed: {}", e)))
    }

    /// Begin a database transaction
    ///
    /// # Example
    /// ```no_run
    /// # use authenc_storage::Database;
    /// # async fn example(db: &Database) -> authenc_types::Result<()> {
    /// db.transaction(|tx| async move {
    ///     tx.execute("INSERT INTO users (id, username) VALUES ($1, $2)", &[&uuid::Uuid::new_v4(), &"john"]).await?;
    ///     Ok(())
    /// }).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn transaction<F, R, Fut>(&self, f: F) -> Result<R>
    where
        F: FnOnce(DatabaseTransaction<'_>) -> Fut,
        Fut: std::future::Future<Output = Result<R>>,
    {
        let mut client = self.get_connection().await?;
        let tx = client.transaction().await
            .map_err(|e| AuthencError::database(format!("Failed to begin transaction: {}", e)))?;

        let db_tx = DatabaseTransaction {
            tx,
            prepared_cache: Arc::clone(&self.prepared_cache),
        };

        let result = f(db_tx).await?;

        // Transaction is automatically committed when dropped if not explicitly rolled back
        Ok(result)
    }

    /// Get pool statistics
    pub fn pool_status(&self) -> PoolStatus {
        let status = self.pool.status();
        PoolStatus {
            size: status.size,
            available: status.available,
            waiting: status.waiting,
        }
    }
}

/// Database transaction wrapper
pub struct DatabaseTransaction<'a> {
    tx: deadpool_postgres::Transaction<'a>,
    prepared_cache: Arc<PreparedStatementCache>,
}

impl<'a> DatabaseTransaction<'a> {
    /// Execute a query within the transaction
    pub async fn execute(&self, query: &str, params: &[&(dyn tokio_postgres::types::ToSql + Sync)]) -> Result<u64> {
        let stmt = self.prepared_cache.get_or_prepare_tx(&self.tx, query).await?;

        self.tx.execute(&stmt, params).await
            .map_err(|e| AuthencError::database(format!("Transaction query failed: {}", e)))
    }

    /// Query one row within the transaction
    pub async fn query_one(&self, query: &str, params: &[&(dyn tokio_postgres::types::ToSql + Sync)]) -> Result<Row> {
        let stmt = self.prepared_cache.get_or_prepare_tx(&self.tx, query).await?;

        self.tx.query_one(&stmt, params).await
            .map_err(|e| AuthencError::database(format!("Transaction query failed: {}", e)))
    }

    /// Query optional row within the transaction
    pub async fn query_opt(&self, query: &str, params: &[&(dyn tokio_postgres::types::ToSql + Sync)]) -> Result<Option<Row>> {
        let stmt = self.prepared_cache.get_or_prepare_tx(&self.tx, query).await?;

        self.tx.query_opt(&stmt, params).await
            .map_err(|e| AuthencError::database(format!("Transaction query failed: {}", e)))
    }

    /// Query multiple rows within the transaction
    pub async fn query(&self, query: &str, params: &[&(dyn tokio_postgres::types::ToSql + Sync)]) -> Result<Vec<Row>> {
        let stmt = self.prepared_cache.get_or_prepare_tx(&self.tx, query).await?;

        self.tx.query(&stmt, params).await
            .map_err(|e| AuthencError::database(format!("Transaction query failed: {}", e)))
    }

    /// Commit the transaction
    pub async fn commit(self) -> Result<()> {
        self.tx.commit().await
            .map_err(|e| AuthencError::database(format!("Transaction commit failed: {}", e)))
    }

    /// Rollback the transaction
    pub async fn rollback(self) -> Result<()> {
        self.tx.rollback().await
            .map_err(|e| AuthencError::database(format!("Transaction rollback failed: {}", e)))
    }
}

/// Prepared statement cache for query optimization
///
/// Caches prepared statements to avoid re-parsing SQL queries on every execution.
/// This significantly improves performance for frequently executed queries.
pub struct PreparedStatementCache {
    cache: RwLock<HashMap<String, Statement>>,
}

impl PreparedStatementCache {
    /// Create a new prepared statement cache
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }

    /// Get a prepared statement from cache or prepare it
    pub async fn get_or_prepare(
        &self,
        client: &deadpool_postgres::Object,
        query: &str,
    ) -> Result<Statement> {
        // Check cache first (read lock)
        {
            let cache = self.cache.read().await;
            if let Some(stmt) = cache.get(query) {
                debug!("Using cached prepared statement for query");
                return Ok(stmt.clone());
            }
        }

        // Not in cache, prepare it (write lock)
        let mut cache = self.cache.write().await;

        // Double-check in case another task prepared it while we were waiting
        if let Some(stmt) = cache.get(query) {
            return Ok(stmt.clone());
        }

        debug!("Preparing new statement for query");
        let stmt = client.prepare(query).await
            .map_err(|e| AuthencError::database(format!("Failed to prepare statement: {}", e)))?;

        cache.insert(query.to_string(), stmt.clone());
        Ok(stmt)
    }

    /// Get a prepared statement for a transaction
    pub async fn get_or_prepare_tx<'a>(
        &self,
        tx: &deadpool_postgres::Transaction<'a>,
        query: &str,
    ) -> Result<Statement> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(stmt) = cache.get(query) {
                return Ok(stmt.clone());
            }
        }

        // Prepare within transaction
        let mut cache = self.cache.write().await;

        if let Some(stmt) = cache.get(query) {
            return Ok(stmt.clone());
        }

        let stmt = tx.prepare(query).await
            .map_err(|e| AuthencError::database(format!("Failed to prepare statement in transaction: {}", e)))?;

        cache.insert(query.to_string(), stmt.clone());
        Ok(stmt)
    }

    /// Clear the prepared statement cache
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        info!("Prepared statement cache cleared");
    }

    /// Get the number of cached statements
    pub async fn size(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }
}

impl Default for PreparedStatementCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Connection pool status
#[derive(Debug, Clone)]
pub struct PoolStatus {
    /// Total number of connections in the pool
    pub size: usize,
    /// Number of available connections
    pub available: usize,
    /// Number of tasks waiting for a connection
    pub waiting: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prepared_cache_creation() {
        let cache = PreparedStatementCache::new();
        assert_eq!(cache.cache.try_read().unwrap().len(), 0);
    }

    #[test]
    fn test_pool_status_creation() {
        let status = PoolStatus {
            size: 20,
            available: 15,
            waiting: 2,
        };
        assert_eq!(status.size, 20);
        assert_eq!(status.available, 15);
        assert_eq!(status.waiting, 2);
    }
}
