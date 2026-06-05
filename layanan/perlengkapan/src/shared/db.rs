//! # Database Connection
//!
//! Database connection and migration management using deadpool-postgres

use anyhow::Result;
use deadpool_postgres::Pool;
use tracing::info;

use crate::shared::error::AppError;

#[derive(Debug, Clone)]
pub struct Database {
    pool: Pool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        // Use optimized configuration from lib-common
        let db_config = lib_backend::db::DbConfig::new(database_url.to_string())
            .with_max_size(50) // Max 50 connections (NFR-SC001)
            .with_min_idle(10) // Min 10 idle connections
            .with_connection_timeout(std::time::Duration::from_secs(30))
            .with_idle_timeout(std::time::Duration::from_secs(600)) // 10 minutes
            .with_max_lifetime(std::time::Duration::from_secs(1800)); // 30 minutes

        let pool = lib_backend::db::create_postgres_pool(db_config)?;

        // Test the connection
        let client = pool.get().await?;
        client.query_one("SELECT 1", &[]).await?;

        info!("Database connection pool established (min: 10, max: 50)");

        Ok(Self { pool })
    }

    /// Wrap an existing pool. Used by background tasks (e.g. export jobs) that
    /// receive a cloned `Pool` and need a `Database` handle without re-running
    /// the connection-test in [`Database::new`].
    pub fn from_pool(pool: Pool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &Pool {
        &self.pool
    }

    /// Get a database connection from the pool
    pub async fn get_connection(&self) -> Result<deadpool_postgres::Object, AppError> {
        self.pool
            .get()
            .await
            .map_err(|e| AppError::Database(format!("Failed to get database connection: {}", e)))
    }

    /// Execute a query that returns exactly one row
    pub async fn query_one(
        &self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<tokio_postgres::Row, AppError> {
        let client = self.get_connection().await?;
        client
            .query_one(query, params)
            .await
            .map_err(|e| AppError::Database(e.to_string()))
    }

    /// Execute a query that returns zero or more rows
    pub async fn query(
        &self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<Vec<tokio_postgres::Row>, AppError> {
        let client = self.get_connection().await?;
        client
            .query(query, params)
            .await
            .map_err(|e| AppError::Database(e.to_string()))
    }

    /// Execute a query that returns zero or one row
    pub async fn query_opt(
        &self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<Option<tokio_postgres::Row>, AppError> {
        let client = self.get_connection().await?;
        client
            .query_opt(query, params)
            .await
            .map_err(|e| AppError::Database(e.to_string()))
    }

    /// Execute a statement that modifies data (INSERT, UPDATE, DELETE)
    pub async fn execute(
        &self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<u64, AppError> {
        let client = self.get_connection().await?;
        client
            .execute(query, params)
            .await
            .map_err(|e| AppError::Database(e.to_string()))
    }
}
