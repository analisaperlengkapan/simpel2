//! Database-dependent validation extensions

use lib_core::error::{CommonError, Result};

/// Async validator trait
#[async_trait::async_trait]
pub trait AsyncValidator<T> {
    async fn validate(&self, value: &T) -> Result<()>;
}

/// Async validation context for database lookups
pub struct AsyncValidationContext {
    pub db_pool: Option<deadpool_postgres::Pool>,
}

impl AsyncValidationContext {
    pub fn new() -> Self {
        Self { db_pool: None }
    }

    pub fn with_db_pool(mut self, pool: deadpool_postgres::Pool) -> Self {
        self.db_pool = Some(pool);
        self
    }
}

impl Default for AsyncValidationContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Async unique validator (checks database for uniqueness)
pub struct UniqueValidator {
    table: String,
    column: String,
    exclude_id: Option<uuid::Uuid>,
}

impl UniqueValidator {
    pub fn new(table: impl Into<String>, column: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            column: column.into(),
            exclude_id: None,
        }
    }

    pub fn excluding(mut self, id: uuid::Uuid) -> Self {
        self.exclude_id = Some(id);
        self
    }

    pub async fn validate_with_pool(
        &self,
        value: &str,
        pool: &deadpool_postgres::Pool,
    ) -> Result<()> {
        let client = pool.get().await.map_err(|e| {
            CommonError::Database(format!("Failed to get database connection: {}", e))
        })?;

        let query = if let Some(_exclude_id) = self.exclude_id {
            format!(
                "SELECT COUNT(*) FROM {} WHERE {} = $1 AND id != $2",
                self.table, self.column
            )
        } else {
            format!(
                "SELECT COUNT(*) FROM {} WHERE {} = $1",
                self.table, self.column
            )
        };

        let row = if let Some(exclude_id) = self.exclude_id {
            let exclude_id_str = exclude_id.to_string();
            client.query_one(&query, &[&value, &exclude_id_str]).await
        } else {
            client.query_one(&query, &[&value]).await
        }
        .map_err(|e| CommonError::Database(format!("Uniqueness check failed: {}", e)))?;

        let count: i64 = row.get(0);

        if count > 0 {
            Err(CommonError::Validation {
                message: format!("{} already exists", self.column),
            })
        } else {
            Ok(())
        }
    }
}

/// Async exists validator (checks if referenced entity exists)
pub struct ExistsValidator {
    table: String,
    column: String,
}

impl ExistsValidator {
    pub fn new(table: impl Into<String>, column: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            column: column.into(),
        }
    }

    pub async fn validate_with_pool(
        &self,
        value: &uuid::Uuid,
        pool: &deadpool_postgres::Pool,
    ) -> Result<()> {
        let client = pool.get().await.map_err(|e| {
            CommonError::Database(format!("Failed to get database connection: {}", e))
        })?;

        let query = format!(
            "SELECT COUNT(*) FROM {} WHERE {} = $1",
            self.table, self.column
        );

        let value_str = value.to_string();
        let row = client
            .query_one(&query, &[&value_str])
            .await
            .map_err(|e| CommonError::Database(format!("Existence check failed: {}", e)))?;

        let count: i64 = row.get(0);

        if count == 0 {
            Err(CommonError::Validation {
                message: format!("{} does not exist", self.column),
            })
        } else {
            Ok(())
        }
    }
}

/// Validate CIDR notation
pub fn validate_cidr(cidr: &str) -> Result<()> {
    cidr.parse::<ipnetwork::IpNetwork>()
        .map_err(|e| CommonError::Validation {
            message: format!("Invalid CIDR notation: {}", e),
        })?;
    Ok(())
}
