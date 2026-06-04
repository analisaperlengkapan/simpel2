//! # authenc-storage
//!
//! Database layer for the Authenc identity provider, providing PostgreSQL-based
//! persistence with connection pooling, prepared statement caching, and transaction support.
//!
//! ## Overview
//!
//! This crate is part of the Authenc multi-crate architecture and provides the complete
//! database abstraction layer. It handles all database interactions including:
//!
//! - **Connection Management**: Robust connection pooling with health monitoring
//! - **Query Optimization**: Prepared statement caching for improved performance
//! - **Transaction Support**: ACID transactions with savepoint support
//! - **Entity Stores**: High-level store implementations for domain entities
//! - **Database Operations**: Low-level CRUD operations for all tables
//! - **Schema Migrations**: Automated database schema versioning and migration
//!
//! ## Architecture
//!
//! The crate is organized into several layers:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                    Store Layer                          │
//! │  (High-level domain entity stores)                      │
//! │  - PostgresUserStore, PostgresSessionStore, etc.        │
//! └─────────────────────────────────────────────────────────┘
//!                           │
//! ┌─────────────────────────────────────────────────────────┐
//! │                  Operations Layer                       │
//! │  (Low-level database operations)                        │
//! │  - CRUD operations, queries, batch operations           │
//! └─────────────────────────────────────────────────────────┘
//!                           │
//! ┌─────────────────────────────────────────────────────────┐
//! │                 Database Core Layer                     │
//! │  (Connection pooling, transactions, caching)            │
//! │  - Database, PoolMonitor, PreparedStatementCache        │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Key Features
//!
//! ### Connection Pooling
//!
//! Efficient connection pooling with configurable size, timeouts, and health checks:
//!
//! ```ignore
//! use authenc_storage::{Database, PoolConfigBuilder};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let pool_config = PoolConfigBuilder::new()
//!     .max_size(20)
//!     .connection_timeout(std::time::Duration::from_secs(30))
//!     .build();
//!
//! let db = Database::new("postgres://user:pass@localhost/authenc", pool_config).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Prepared Statement Caching
//!
//! Automatic caching of prepared statements for improved query performance:
//!
//! ```ignore
//! # use authenc_storage::Database;
//! # async fn example(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
//! let row = db.query_one_prepared(
//!     "SELECT * FROM users WHERE id = $1",
//!     &[&user_id],
//! ).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Transaction Support
//!
//! Full ACID transaction support with savepoints:
//!
//! ```ignore
//! # use authenc_storage::{Database, IsolationLevel};
//! # async fn example(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
//! db.transaction(|tx| async move {
//!     // All operations here are atomic
//!     tx.execute("INSERT INTO users ...", &[]).await?;
//!     tx.execute("INSERT INTO sessions ...", &[]).await?;
//!     tx.commit().await?; // IMPORTANT: must commit explicitly or changes are rolled back on drop
//!     Ok(())
//! }).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Store Implementations
//!
//! High-level store abstractions for domain entities:
//!
//! ```ignore
//! # use authenc_storage::{Database, PostgresUserStore};
//! # async fn example(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
//! let user_store = PostgresUserStore::new(db.clone());
//! let user = user_store.get_by_username("admin").await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Module Organization
//!
//! - [`database`]: Core database connection and query execution
//! - [`pool_config`]: Connection pool configuration
//! - [`pool_monitor`]: Pool health monitoring and metrics
//! - [`prepared_cache`]: Prepared statement caching
//! - [`transaction`]: Transaction management and isolation levels
//! - [`stores`]: High-level store implementations
//! - [`operations`]: Low-level database operations
//! - [`queries`]: Common query patterns
//! - [`batch`]: Batch operation support
//! - [`migrations`]: Database schema migration system
//!
//! ## Migration Status
//!
//! This crate is part of the Authenc comprehensive refactoring (Phase 2).
//! Some modules are temporarily disabled pending model migration to `authenc-types`:
//!
//! - `batch_operations`: Needs Permission, User models
//! - `audit_operations`: Needs events models and audit signature service
//! - `captcha_operations`: Needs captcha service integration
//! - `satker_operations`: Needs satker hierarchy models
//!
//! These will be re-enabled in Phase 3 after model migration is complete.

// ============================================================================
// Type Re-exports
// ============================================================================

// Re-export all types from authenc-types for convenience
pub use authenc_types::*;

// ============================================================================
// Core Infrastructure Modules
// ============================================================================

/// Core database connection and query execution
pub mod database;

/// Connection pool configuration and builder
pub mod pool_config;

/// Pool health monitoring and metrics collection
pub mod pool_monitor;

/// Prepared statement caching for query optimization
pub mod prepared_cache;

/// Transaction management with savepoint support
pub mod transaction;

/// Database schema migration system
pub mod migrations;

// ============================================================================
// Store Layer (High-Level Domain Entity Stores)
// ============================================================================

/// High-level store implementations for domain entities
///
/// Provides convenient abstractions over raw database operations:
/// - `PostgresUserStore`: User management
/// - `PostgresSessionStore`: Session management
/// - `PostgresRealmStore`: Realm/tenant management
/// - `PostgresClientStore`: OAuth2 client management
/// - `PostgresCredentialStore`: Credential management
pub mod stores;

// ============================================================================
// Operations Layer (Low-Level Database Operations)
// ============================================================================

/// Error shim re-exporting error types from authenc_types for legacy operations
///
/// Provides backward-compatible `crate::error::Result` and `crate::error::AuthencError`
/// import patterns used by legacy operation files.
pub mod error;

/// Models shim re-exporting domain types from authenc_types for legacy operations
///
/// Provides backward-compatible `crate::models::*` import patterns used by
/// legacy operation files while types live in authenc_types.
pub mod models;

/// Low-level database operations for all tables
///
/// Contains CRUD operations organized by domain:
/// - Legacy operations (split from 11,263-line monolith)
/// - Token operations
/// - Client registration operations (disabled)
/// - Protocol mapper operations (disabled)
pub mod operations;

/// Common query patterns and utilities
pub mod queries;

/// Batch operation support for bulk inserts/updates
pub mod batch;

// ============================================================================
// Specialized Operations (Temporarily Disabled)
// ============================================================================

// The following modules are disabled pending model migration to authenc-types.
// They will be re-enabled in Phase 3 of the refactoring.

// pub mod batch_operations;
// Disabled: Needs Permission, User models from authenc-types
// Provides: Batch permission checks, bulk user operations

// pub mod audit_operations;
// Disabled: Needs models::events, services::audit_signature
// Provides: Audit log storage, event querying, signature verification

pub mod captcha_operations;
pub mod satker_operations;

// ============================================================================
// Public API Re-exports
// ============================================================================

// Database Core Types
// ----------------------------------------------------------------------------

/// Main database connection pool and query interface
pub use database::Database;

/// Transaction handle for atomic operations
pub use database::DatabaseTransaction;

/// Prepared statement cache for query optimization
pub use prepared_cache::PreparedStatementCache;

/// Connection pool status information
pub use database::PoolStatus;

// Pool Management Types
// ----------------------------------------------------------------------------

/// Builder for configuring connection pool parameters
pub use pool_config::PoolConfigBuilder;

/// Pool health status and metrics
pub use pool_config::PoolHealth;

/// Background pool monitor for health checks
pub use pool_monitor::PoolMonitor;

/// Configuration for pool monitoring behavior
pub use pool_monitor::PoolMonitorConfig;

/// Statistics about prepared statement cache usage
pub use prepared_cache::CacheStats;

// Transaction Types
// ----------------------------------------------------------------------------

/// SQL transaction isolation levels
pub use transaction::IsolationLevel;

/// Transaction savepoint for partial rollback
pub use transaction::Savepoint;

// Store Implementations
// ----------------------------------------------------------------------------

/// PostgreSQL-backed user store
pub use stores::PostgresUserStore;

/// PostgreSQL-backed session store
pub use stores::PostgresSessionStore;

/// PostgreSQL-backed realm store
pub use stores::PostgresRealmStore;

/// PostgreSQL-backed OAuth2 client store
pub use stores::PostgresClientStore;

/// PostgreSQL-backed credential store
pub use stores::PostgresCredentialStore;

/// PostgreSQL-backed token revocation list (F2H)
pub use stores::PostgresRevocationStore;

// Migration Types
// ----------------------------------------------------------------------------

/// Database migration definition
pub use migrations::Migration;

/// Migration execution engine
pub use migrations::MigrationRunner;

/// Result of a migration operation
pub use migrations::MigrationResult;

/// Status of a migration (pending, applied, failed)
pub use migrations::MigrationStatus;

/// Run all pending migrations
pub use migrations::run_migrations;

/// Run migrations with custom schema name
pub use migrations::run_migrations_with_schema;

/// Check status of all migrations
pub use migrations::check_migration_status;

/// Get current schema version
pub use migrations::get_schema_version;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
