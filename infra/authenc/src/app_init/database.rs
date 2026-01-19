//! Database initialization helpers
//!
//! Extracted from app.rs to improve code organization.
//! Handles database connection pool setup, audit store initialization,
//! and UMA table initialization.

use crate::config::AppConfig;
use crate::error::{AuthencError, Result};
use std::sync::Arc;

pub async fn initialize_database(config: &AppConfig) -> Result<Arc<crate::database::Database>> {
    let database = Arc::new(
        crate::database::Database::new(&config.database)
            .await
            .map_err(|e| AuthencError::database(format!("Failed to initialize database: {}", e)))?,
    );
    Ok(database)
}

/// Initialize audit log store
///
/// Sets up the PostgreSQL-backed audit log storage.
pub async fn initialize_audit_store(
    config: &AppConfig,
) -> Result<Arc<crate::services::pg_audit_log_store::PgAuditLogStore>> {
    let audit_log_store = Arc::new(
        crate::services::pg_audit_log_store::PgAuditLogStore::new(&config.database_url())
            .await
            .map_err(|e| AuthencError::database(format!("Failed to init audit store: {}", e)))?,
    );
    Ok(audit_log_store)
}

/// Initialize UMA 2.0 database tables
///
/// Creates UMA 2.0 tables if they don't already exist.
/// Logs a warning but doesn't fail if tables already exist.
pub async fn init_uma_tables(database: &Arc<crate::database::Database>) -> Result<()> {
    crate::services::uma::init::init_uma_tables(database)
        .await
        .inspect_err(|e| {
            tracing::warn!("Failed to initialize UMA tables (may already exist): {}", e);
        })
        .ok();
    Ok(())
}

/// Initialize consent store
/// Initialize database connection pool
///
/// Creates and configures the database connection pool based on the
/// provided configuration.
pub fn initialize_consent_store(
    database: Arc<crate::database::Database>,
) -> Arc<crate::services::stores::consent_store::ConsentStore> {
    Arc::new(crate::services::stores::consent_store::ConsentStore::new(
        database,
    ))
}

/// Initialize authentication flow store
pub fn initialize_auth_flow_store(
    database: Arc<crate::database::Database>,
) -> Arc<crate::services::stores::auth_flow_store::AuthFlowStore> {
    Arc::new(crate::services::stores::auth_flow_store::AuthFlowStore::new(database))
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_module_compiles() {
        // Basic compilation test
    }
}
