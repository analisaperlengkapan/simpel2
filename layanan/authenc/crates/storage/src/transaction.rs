//! Database transaction management
//!
//! Provides explicit transaction control following enterprise IAM's transaction patterns.
//! Supports commit, rollback, savepoints, and isolation levels.

use authenc_types::{AuthencError, Result};
use tokio_postgres::{Client, IsolationLevel as PgIsolationLevel, Transaction as PgTransaction};
use tracing::{debug, error, warn};

/// Transaction isolation levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolationLevel {
    /// Read uncommitted (lowest isolation)
    ReadUncommitted,
    /// Read committed (PostgreSQL default)
    ReadCommitted,
    /// Repeatable read
    RepeatableRead,
    /// Serializable (highest isolation)
    Serializable,
}

impl From<IsolationLevel> for PgIsolationLevel {
    fn from(level: IsolationLevel) -> Self {
        match level {
            IsolationLevel::ReadUncommitted => PgIsolationLevel::ReadUncommitted,
            IsolationLevel::ReadCommitted => PgIsolationLevel::ReadCommitted,
            IsolationLevel::RepeatableRead => PgIsolationLevel::RepeatableRead,
            IsolationLevel::Serializable => PgIsolationLevel::Serializable,
        }
    }
}

/// Transaction state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransactionState {
    /// Transaction is active
    Active,
    /// Transaction marked for rollback only
    RollbackOnly,
    /// Transaction has been committed
    Committed,
    /// Transaction has been rolled back
    RolledBack,
}

/// Database transaction wrapper providing enterprise IAM-like transaction semantics
pub struct DatabaseTransaction<'a> {
    transaction: Option<PgTransaction<'a>>,
    state: TransactionState,
    savepoint_counter: usize,
}

impl<'a> DatabaseTransaction<'a> {
    /// Create a new transaction from a database client
    pub async fn new(
        client: &'a mut Client,
        isolation_level: Option<IsolationLevel>,
    ) -> Result<Self> {
        // Start transaction
        let query = match isolation_level {
            Some(IsolationLevel::ReadUncommitted) => "BEGIN ISOLATION LEVEL READ UNCOMMITTED",
            Some(IsolationLevel::ReadCommitted) => "BEGIN ISOLATION LEVEL READ COMMITTED",
            Some(IsolationLevel::RepeatableRead) => "BEGIN ISOLATION LEVEL REPEATABLE READ",
            Some(IsolationLevel::Serializable) => "BEGIN ISOLATION LEVEL SERIALIZABLE",
            None => "BEGIN",
        };

        client.execute(query, &[]).await.map_err(|e| {
            error!("Failed to begin transaction: {}", e);
            AuthencError::database(format!("Failed to begin transaction: {}", e))
        })?;

        debug!(
            "Transaction started with isolation level: {:?}",
            isolation_level
        );

        // Build transaction using the builder
        let transaction = client.transaction().await.map_err(|e| {
            error!("Failed to build transaction: {}", e);
            AuthencError::database(format!("Failed to build transaction: {}", e))
        })?;

        Ok(Self {
            transaction: Some(transaction),
            state: TransactionState::Active,
            savepoint_counter: 0,
        })
    }

    /// Check if transaction is active
    pub fn is_active(&self) -> bool {
        self.state == TransactionState::Active
    }

    /// Check if transaction is marked for rollback only
    pub fn is_rollback_only(&self) -> bool {
        self.state == TransactionState::RollbackOnly
    }

    /// Set transaction to rollback only (cannot be committed)
    pub fn set_rollback_only(&mut self) {
        if self.state == TransactionState::Active {
            self.state = TransactionState::RollbackOnly;
            warn!("Transaction marked for rollback only");
        }
    }

    /// Commit the transaction
    pub async fn commit(mut self) -> Result<()> {
        match self.state {
            TransactionState::Active => {
                if let Some(tx) = self.transaction.take() {
                    tx.commit().await.map_err(|e| {
                        error!("Transaction commit failed: {}", e);
                        AuthencError::database(format!("Transaction commit failed: {}", e))
                    })?;
                    self.state = TransactionState::Committed;
                    debug!("Transaction committed successfully");
                    Ok(())
                } else {
                    Err(AuthencError::database("Transaction already consumed"))
                }
            }
            TransactionState::RollbackOnly => {
                // Auto-rollback if marked for rollback only
                warn!("Cannot commit transaction marked for rollback only - rolling back");
                self.rollback().await
            }
            TransactionState::Committed => {
                Err(AuthencError::database("Transaction already committed"))
            }
            TransactionState::RolledBack => {
                Err(AuthencError::database("Transaction already rolled back"))
            }
        }
    }

    /// Rollback the transaction
    pub async fn rollback(mut self) -> Result<()> {
        match self.state {
            TransactionState::Active | TransactionState::RollbackOnly => {
                if let Some(tx) = self.transaction.take() {
                    tx.rollback().await.map_err(|e| {
                        error!("Transaction rollback failed: {}", e);
                        AuthencError::database(format!("Transaction rollback failed: {}", e))
                    })?;
                    self.state = TransactionState::RolledBack;
                    debug!("Transaction rolled back");
                    Ok(())
                } else {
                    Err(AuthencError::database("Transaction already consumed"))
                }
            }
            TransactionState::Committed => Err(AuthencError::database(
                "Cannot rollback committed transaction",
            )),
            TransactionState::RolledBack => {
                Err(AuthencError::database("Transaction already rolled back"))
            }
        }
    }

    /// Create a savepoint (nested transaction)
    pub async fn savepoint(&mut self) -> Result<Savepoint<'_>> {
        if !self.is_active() {
            return Err(AuthencError::database("Transaction is not active"));
        }

        self.savepoint_counter += 1;
        let name = format!("sp_{}", self.savepoint_counter);

        if let Some(tx) = &self.transaction {
            tx.execute(&format!("SAVEPOINT {}", name), &[])
                .await
                .map_err(|e| {
                    error!("Failed to create savepoint: {}", e);
                    AuthencError::database(format!("Failed to create savepoint: {}", e))
                })?;

            debug!("Savepoint created: {}", name);
            Ok(Savepoint {
                name,
                transaction: self,
                released: false,
            })
        } else {
            Err(AuthencError::database("Transaction not available"))
        }
    }

    /// Execute a query within the transaction
    pub async fn execute(
        &self,
        statement: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<u64> {
        if !self.is_active() {
            return Err(AuthencError::database("Transaction is not active"));
        }

        if let Some(tx) = &self.transaction {
            tx.execute(statement, params).await.map_err(|e| {
                error!("Transaction execute failed: {}", e);
                AuthencError::database(format!("Transaction execute failed: {}", e))
            })
        } else {
            Err(AuthencError::database("Transaction not available"))
        }
    }

    /// Query within the transaction
    pub async fn query(
        &self,
        statement: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<Vec<tokio_postgres::Row>> {
        if !self.is_active() {
            return Err(AuthencError::database("Transaction is not active"));
        }

        if let Some(tx) = &self.transaction {
            tx.query(statement, params).await.map_err(|e| {
                error!("Transaction query failed: {}", e);
                AuthencError::database(format!("Transaction query failed: {}", e))
            })
        } else {
            Err(AuthencError::database("Transaction not available"))
        }
    }

    /// Query one row within the transaction
    pub async fn query_one(
        &self,
        statement: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<tokio_postgres::Row> {
        if !self.is_active() {
            return Err(AuthencError::database("Transaction is not active"));
        }

        if let Some(tx) = &self.transaction {
            tx.query_one(statement, params).await.map_err(|e| {
                error!("Transaction query_one failed: {}", e);
                AuthencError::database(format!("Transaction query_one failed: {}", e))
            })
        } else {
            Err(AuthencError::database("Transaction not available"))
        }
    }

    /// Query optional row within the transaction
    pub async fn query_opt(
        &self,
        statement: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<Option<tokio_postgres::Row>> {
        if !self.is_active() {
            return Err(AuthencError::database("Transaction is not active"));
        }

        if let Some(tx) = &self.transaction {
            tx.query_opt(statement, params).await.map_err(|e| {
                error!("Transaction query_opt failed: {}", e);
                AuthencError::database(format!("Transaction query_opt failed: {}", e))
            })
        } else {
            Err(AuthencError::database("Transaction not available"))
        }
    }
}

/// Savepoint for nested transaction control
pub struct Savepoint<'a> {
    name: String,
    transaction: &'a DatabaseTransaction<'a>,
    released: bool,
}

impl<'a> Savepoint<'a> {
    /// Rollback to this savepoint
    pub async fn rollback(mut self) -> Result<()> {
        if self.released {
            return Err(AuthencError::database("Savepoint already released"));
        }

        if let Some(tx) = &self.transaction.transaction {
            tx.execute(&format!("ROLLBACK TO SAVEPOINT {}", self.name), &[])
                .await
                .map_err(|e| {
                    error!("Failed to rollback savepoint: {}", e);
                    AuthencError::database(format!("Failed to rollback savepoint: {}", e))
                })?;

            debug!("Rolled back to savepoint: {}", self.name);
            self.released = true;
            Ok(())
        } else {
            Err(AuthencError::database("Transaction not available"))
        }
    }

    /// Release (commit) this savepoint
    pub async fn release(mut self) -> Result<()> {
        if self.released {
            return Err(AuthencError::database("Savepoint already released"));
        }

        if let Some(tx) = &self.transaction.transaction {
            tx.execute(&format!("RELEASE SAVEPOINT {}", self.name), &[])
                .await
                .map_err(|e| {
                    error!("Failed to release savepoint: {}", e);
                    AuthencError::database(format!("Failed to release savepoint: {}", e))
                })?;

            debug!("Released savepoint: {}", self.name);
            self.released = true;
            Ok(())
        } else {
            Err(AuthencError::database("Transaction not available"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isolation_level_conversion() {
        // Test that conversion doesn't panic
        let _level: PgIsolationLevel = IsolationLevel::RepeatableRead.into();
        let _level2: PgIsolationLevel = IsolationLevel::Serializable.into();
        let _level3: PgIsolationLevel = IsolationLevel::ReadCommitted.into();
        let _level4: PgIsolationLevel = IsolationLevel::ReadUncommitted.into();
    }

    #[test]
    fn test_isolation_level_equality() {
        assert_eq!(IsolationLevel::ReadCommitted, IsolationLevel::ReadCommitted);
        assert_ne!(IsolationLevel::ReadCommitted, IsolationLevel::Serializable);
    }

    #[test]
    fn test_isolation_level_copy() {
        let level = IsolationLevel::RepeatableRead;
        let level_copy = level;
        assert_eq!(level, level_copy);
    }

    #[test]
    fn test_transaction_state_equality() {
        assert_eq!(TransactionState::Active, TransactionState::Active);
        assert_ne!(TransactionState::Active, TransactionState::Committed);
    }

    #[test]
    fn test_all_isolation_levels() {
        let levels = vec![
            IsolationLevel::ReadUncommitted,
            IsolationLevel::ReadCommitted,
            IsolationLevel::RepeatableRead,
            IsolationLevel::Serializable,
        ];

        for level in levels {
            let _pg_level: PgIsolationLevel = level.into();
        }
    }
}
