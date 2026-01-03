//! Secreton Storage Abstraction Layer
//!
//! This crate provides a unified storage interface for the Secreton vault system,
//! enabling seamless switching between different backend implementations without
//! changing application code.
//!
//! # Supported Backends
//!
//! - **PostgreSQL** - Production-ready, ACID-compliant relational storage
//! - **Memory** - Fast in-memory storage for testing and development
//! - **Encrypted Storage** - Wrapper adding encryption layer to any backend
//! - **Cached Storage** - Wrapper adding caching layer for performance
//! - **Raft** (optional) - Distributed consensus-based storage for HA clusters
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────┐
//! │     Application Layer               │
//! │  (Core Services, API Handlers)      │
//! └────────────────┬────────────────────┘
//!                  │
//!                  ▼
//! ┌─────────────────────────────────────┐
//! │     StorageBackend Trait            │
//! │  (Unified Interface)                │
//! └────────────────┬────────────────────┘
//!                  │
//!       ┌──────────┴──────────┐
//!       ▼                     ▼
//! ┌──────────┐          ┌──────────┐
//! │ Postgres │          │  Memory  │
//! │ Backend  │          │ Backend  │
//! └──────────┘          └──────────┘
//!       │                     │
//!       └──────────┬──────────┘
//!                  ▼
//!       ┌─────────────────────┐
//!       │ Optional Wrappers:  │
//!       │ - EncryptedStorage  │
//!       │ - CachedStorage     │
//!       └─────────────────────┘
//! ```
//!
//! # Core Trait: StorageBackend
//!
//! All storage implementations must implement the [`StorageBackend`] trait which provides:
//!
//! - **CRUD Operations**: get, put, delete, list
//! - **Transaction Support**: Atomic operations
//! - **Metadata Management**: Custom key-value metadata
//! - **Batch Operations**: Efficient bulk reads/writes
//! - **Health Checking**: Backend availability monitoring
//!
//! # Example: Using PostgreSQL Backend
//!
//! ```rust,no_run
//! # #[cfg(feature = "postgres")]
//! # {
//! use secreton_storage::{StorageBackend, PostgresBackend, VaultEntry, SecurityLevel};
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create PostgreSQL backend
//! let backend = PostgresBackend::new("postgres://localhost/secreton").await?;
//! let storage: Arc<dyn StorageBackend + Send + Sync> = Arc::new(backend);
//!
//! // Create and store an entry
//! let entry = VaultEntry {
//!     id: uuid::Uuid::new_v4(),
//!     path: "app/config".to_string(),
//!     encrypted_data: b"secret-value".to_vec(),
//!     security_level: SecurityLevel::Internal,
//!     // ... other fields with defaults
//! #   encryption_metadata: serde_json::json!({}),
//! #   metadata: serde_json::json!({}),
//! #   tags: vec![],
//! #   version: 1,
//! #   owner_id: "system".to_string(),
//! #   created_at: chrono::Utc::now(),
//! #   updated_at: chrono::Utc::now(),
//! #   expires_at: None,
//! };
//! storage.store(&entry).await?;
//!
//! // Retrieve by path
//! let retrieved = storage.get_by_path("app/config").await?;
//!
//! // Delete by path
//! storage.delete_by_path("app/config").await?;
//! # Ok(())
//! # }
//! # }
//! ```
//!
//! # Example: Adding Encryption Layer
//!
//! ```rust,no_run
//! use secreton_storage::{StorageBackend, MemoryBackend, EncryptedStorage, VaultEntry, SecurityLevel};
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create base backend
//! let base_backend = MemoryBackend::new();
//!
//! // Wrap with encryption (specify encryption key ID)
//! let encrypted = EncryptedStorage::new(Arc::new(base_backend), "primary-key-001".to_string());
//!
//! let storage: Arc<dyn StorageBackend + Send + Sync> = Arc::new(encrypted);
//!
//! // All operations use the encryption wrapper
//! let entry = VaultEntry {
//!     id: uuid::Uuid::new_v4(),
//!     path: "sensitive/data".to_string(),
//!     encrypted_data: b"plaintext".to_vec(),
//!     security_level: SecurityLevel::Secret,
//! #   encryption_metadata: serde_json::json!({"key_id": "primary-key-001"}),
//! #   metadata: serde_json::json!({}),
//! #   tags: vec![],
//! #   version: 1,
//! #   owner_id: "system".to_string(),
//! #   created_at: chrono::Utc::now(),
//! #   updated_at: chrono::Utc::now(),
//! #   expires_at: None,
//! };
//! storage.store(&entry).await?;
//! // Data is stored with encryption key tracking
//! # Ok(())
//! # }
//! ```
//!
//! # Example: Adding Cache Layer
//!
//! ```rust,no_run
//! # #[cfg(feature = "postgres")]
//! # {
//! use secreton_storage::{StorageBackend, PostgresBackend, CachedStorage, InMemoryCache};
//! use std::sync::Arc;
//! use std::time::Duration;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let backend = PostgresBackend::new("postgres://localhost/secreton").await?;
//! let cache = InMemoryCache::new(1000); // 1000 entry capacity
//!
//! let cached = CachedStorage::new(backend, cache, Duration::from_secs(300));
//! let storage: Arc<dyn StorageBackend + Send + Sync> = Arc::new(cached);
//!
//! // Reads are cached, writes invalidate cache
//! storage.get_by_path("hot/key").await?; // Miss - reads from backend
//! storage.get_by_path("hot/key").await?; // Hit - reads from cache
//! # Ok(())
//! # }
//! # }
//! ```
//!
//! # Performance Considerations
//!
//! - **PostgreSQL**: ~1-5ms latency per operation, unlimited capacity
//! - **Memory**: ~10-50μs latency, limited by RAM
//! - **Encrypted**: +20-30% overhead for encryption/decryption
//! - **Cached**: Near-memory speeds for cache hits, configurable TTL
//!
//! # Thread Safety
//!
//! All storage backends are `Send + Sync` and can be safely shared across threads
//! using `Arc<dyn StorageBackend>`. Internal synchronization is handled by each
//! backend implementation.
//!
//! # Error Handling
//!
//! Storage operations return [`Result<T, StorageError>`](StorageError) with variants for:
//! - Connection failures
//! - Not found errors
//! - Permission errors
//! - Serialization errors
//! - Backend-specific errors
//!
//! # Feature Flags
//!
//! - `raft-consensus` - Enable Raft-based distributed storage (requires additional setup)
//!
//! # See Also
//!
//! - [`StorageBackend`] - Core trait all backends implement
//! - [`StorageFactory`](factory::StorageFactory) - Factory for creating backends from config
//! - [`EncryptedStorage`] - Encryption wrapper
//! - [`CachedStorage`] - Caching wrapper

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

// Re-export SecurityLevel from secreton-types
pub use secreton_types::SecurityLevel;

pub mod backends;
pub mod cache;
pub mod encrypted_storage;
pub mod factory;
pub mod kv_adapter;
pub mod memory;
pub mod prelude;

// OpenRaft consensus module (migrated from old raft)
#[cfg(feature = "raft-consensus")]
#[cfg(feature = "raft-consensus")]
pub mod raft;

// Re-export essential backends
pub use backends::{FileBackend, FileConfig};
pub use cache::{CacheBackend, CacheStats, CachedStorage, InMemoryCache};
pub use encrypted_storage::EncryptedStorage;
pub use kv_adapter::KvBackendAdapter;
pub use memory::MemoryBackend;

#[cfg(feature = "consul")]
pub use backends::{ConsulBackend, ConsulConfig};

#[cfg(feature = "s3")]
pub use backends::{S3Backend, S3Config};

#[cfg(feature = "postgres")]
pub use backends::PostgresBackend;

// Re-export OpenRaft components
#[cfg(feature = "raft-consensus")]
pub use raft::{
    Raft, RaftCluster, RaftClusterConfig, RaftStatus, SecretonRaftStorage, SecretonStateMachine,
    StateMachineCommand, StateMachineResponse,
};

// Re-export factory
pub use factory::{StorageBackendType, StorageFactory, StorageFactoryConfig};

/// Encryption metadata for vault entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionMetadata {
    /// Encryption algorithm used
    pub algorithm: String,
    /// Key ID used for encryption
    pub key_id: String,
    /// Initialization vector
    pub iv: Vec<u8>,
    /// Authentication tag for AEAD ciphers
    pub auth_tag: Option<Vec<u8>>,
    /// Additional authenticated data
    pub aad: Option<Vec<u8>>,
    /// Key derivation parameters
    pub kdf_params: Option<HashMap<String, String>>,
}

// SecurityLevel is now re-exported from secreton-types (see line 164)

/// Vault entry for storing secrets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultEntry {
    /// Unique identifier for the entry
    pub id: Uuid,
    /// Path to the secret
    pub path: String,
    /// Encrypted secret data
    pub encrypted_data: Vec<u8>,
    /// Encryption metadata
    pub encryption_metadata: serde_json::Value,
    /// Security level of the data
    pub security_level: SecurityLevel,
    /// Additional metadata
    pub metadata: serde_json::Value,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Version number
    pub version: u32,
    /// Owner of the entry
    pub owner_id: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Optional expiration time
    pub expires_at: Option<DateTime<Utc>>,
}

impl VaultEntry {
    /// Create a new vault entry
    pub fn new(
        path: String,
        encrypted_data: Vec<u8>,
        encryption_metadata: serde_json::Value,
        security_level: SecurityLevel,
        owner_id: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            path,
            encrypted_data,
            encryption_metadata,
            security_level,
            metadata: serde_json::json!({}),
            tags: Vec::new(),
            version: 1,
            owner_id,
            created_at: now,
            updated_at: now,
            expires_at: None,
        }
    }

    /// Check if the entry is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Set expiration time
    pub fn with_expiration(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    /// Add metadata
    pub fn add_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        if let serde_json::Value::Object(ref mut map) = self.metadata {
            map.insert(key, value);
        }
        self
    }

    /// Add tag
    pub fn add_tag(mut self, tag: String) -> Self {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
        self
    }
}

impl Default for EncryptionMetadata {
    fn default() -> Self {
        Self {
            algorithm: "aes-256-gcm".to_string(),
            key_id: "default-key".to_string(),
            iv: vec![0; 12],
            auth_tag: None,
            aad: None,
            kdf_params: None,
        }
    }
}

/// Options for listing entries
#[derive(Debug, Clone, Default)]
pub struct ListOptions {
    /// Path prefix to filter by
    pub prefix: Option<String>,

    /// Maximum number of results
    pub limit: Option<usize>,

    /// Offset for pagination
    pub offset: Option<usize>,

    /// Include metadata in results
    pub include_metadata: bool,
}

/// Query parameters for filtering vault entries
#[derive(Debug, Clone, Default)]
pub struct QueryParams {
    /// Filter by path prefix
    pub path_prefix: Option<String>,

    /// Filter by security level (minimum)
    pub security_level: Option<SecurityLevel>,

    /// Filter by tags
    pub tags: Vec<String>,

    /// Filter by owner
    pub owner_id: Option<Uuid>,

    /// Filter by metadata
    pub metadata_filters: HashMap<String, String>,

    /// Include expired entries
    pub include_expired: bool,

    /// Maximum number of results
    pub limit: Option<u32>,

    /// Results offset
    pub offset: Option<u32>,

    /// Sort order
    pub sort_by: Option<String>,

    /// Sort direction (asc/desc)
    pub sort_order: Option<String>,
}

impl QueryParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_path_prefix(mut self, prefix: String) -> Self {
        self.path_prefix = Some(prefix);
        self
    }

    pub fn with_security_level(mut self, level: SecurityLevel) -> Self {
        self.security_level = Some(level);
        self
    }

    pub fn with_tag(mut self, tag: String) -> Self {
        self.tags.push(tag);
        self
    }

    pub fn with_owner(mut self, owner_id: Uuid) -> Self {
        self.owner_id = Some(owner_id);
        self
    }

    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// Storage operation errors
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Connection failed: {message}")]
    ConnectionFailed {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Query failed: {message}")]
    QueryFailed {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Transaction failed: {message}")]
    TransactionFailed {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Serialization error: {message}")]
    SerializationError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Not found: {resource_type} with ID {id}")]
    NotFound { resource_type: String, id: String },

    #[error("Duplicate entry: {resource_type} with ID {id}")]
    Duplicate { resource_type: String, id: String },

    #[error("Constraint violation: {constraint} - {message}")]
    ConstraintViolation { constraint: String, message: String },

    #[error("Permission denied for operation: {operation}")]
    PermissionDenied { operation: String },

    #[error("Storage backend error: {backend} - {message}")]
    BackendError { backend: String, message: String },

    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    #[error("Migration error: {message}")]
    MigrationError { message: String },

    #[error("Not leader, redirect to: {leader_id}")]
    NotLeader { leader_id: String },

    #[error("Replication error: {0}")]
    ReplicationError(String),

    #[error("Transaction not supported by backend: {backend}")]
    TransactionNotSupported { backend: String },

    #[error("Invalid query: {message}")]
    InvalidQuery { message: String },

    #[error("Timeout: {operation}")]
    Timeout { operation: String },
}

/// Type alias for Results with StorageError
pub type StorageResult<T> = Result<T, StorageError>;

impl StorageError {
    /// Create a connection failed error with context
    pub fn connection_failed<S: Into<String>>(message: S) -> Self {
        Self::ConnectionFailed {
            message: message.into(),
            source: None,
        }
    }

    /// Create a connection failed error with source
    pub fn connection_failed_with_source<
        S: Into<String>,
        E: std::error::Error + Send + Sync + 'static,
    >(
        message: S,
        source: E,
    ) -> Self {
        Self::ConnectionFailed {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a query failed error with context
    pub fn query_failed<S: Into<String>>(message: S) -> Self {
        Self::QueryFailed {
            message: message.into(),
            source: None,
        }
    }

    /// Create a query failed error with source
    pub fn query_failed_with_source<
        S: Into<String>,
        E: std::error::Error + Send + Sync + 'static,
    >(
        message: S,
        source: E,
    ) -> Self {
        Self::QueryFailed {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a transaction failed error with context
    pub fn transaction_failed<S: Into<String>>(message: S) -> Self {
        Self::TransactionFailed {
            message: message.into(),
            source: None,
        }
    }

    /// Create a transaction failed error with source
    pub fn transaction_failed_with_source<
        S: Into<String>,
        E: std::error::Error + Send + Sync + 'static,
    >(
        message: S,
        source: E,
    ) -> Self {
        Self::TransactionFailed {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a serialization error with context
    pub fn serialization_error<S: Into<String>>(message: S) -> Self {
        Self::SerializationError {
            message: message.into(),
            source: None,
        }
    }

    /// Create a serialization error with source
    pub fn serialization_error_with_source<
        S: Into<String>,
        E: std::error::Error + Send + Sync + 'static,
    >(
        message: S,
        source: E,
    ) -> Self {
        Self::SerializationError {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }
}

/// Simple key-value storage backend trait (HashiCorp Vault-style)
/// This is the core trait for physical storage backends. All data is pre-encrypted
/// before being passed to the backend (untrusted storage principle).
#[async_trait]
pub trait KvBackend: Send + Sync {
    /// Get a value by key
    async fn get(&self, key: &str) -> StorageResult<Option<Vec<u8>>>;

    /// Put a value by key
    async fn put(&self, key: &str, value: &[u8]) -> StorageResult<()>;

    /// Delete a value by key
    async fn delete(&self, key: &str) -> StorageResult<()>;

    /// List keys with a prefix
    async fn list(&self, prefix: &str) -> StorageResult<Vec<String>>;

    /// Check if key exists
    async fn exists(&self, key: &str) -> StorageResult<bool>;

    /// Get backend metrics
    async fn metrics(&self) -> StorageResult<BackendMetrics>;

    /// Perform health check on the backend
    async fn health_check(&self) -> StorageResult<HealthStatus>;
}

/// High-level storage backend trait for VaultEntry operations
/// This trait provides structured access to vault entries with metadata,
/// versioning, and advanced querying capabilities.
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// Store a vault entry
    async fn store(&self, entry: &VaultEntry) -> StorageResult<()>;

    /// Retrieve a vault entry by ID
    async fn get_by_id(&self, id: Uuid) -> StorageResult<Option<VaultEntry>>;

    /// Retrieve a vault entry by path
    async fn get_by_path(&self, path: &str) -> StorageResult<Option<VaultEntry>>;

    /// Update an existing vault entry
    async fn update(&self, entry: &VaultEntry) -> StorageResult<()>;

    /// Delete a vault entry by ID
    async fn delete_by_id(&self, id: Uuid) -> StorageResult<bool>;

    /// Delete a vault entry by path
    async fn delete_by_path(&self, path: &str) -> StorageResult<bool>;

    /// List vault entries with filtering
    async fn list(&self, params: &QueryParams) -> StorageResult<Vec<VaultEntry>>;

    /// Count vault entries matching query
    async fn count(&self, params: &QueryParams) -> StorageResult<u64>;

    /// Check if path exists
    async fn exists(&self, path: &str) -> StorageResult<bool>;

    /// Begin a transaction
    async fn begin_transaction(&self) -> StorageResult<Box<dyn StorageTransaction>>;

    /// Perform health check
    async fn health_check(&self) -> StorageResult<HealthStatus>;

    /// Get storage statistics
    async fn get_stats(&self) -> StorageResult<StorageStats>;

    /// Run migrations
    async fn migrate(&self) -> StorageResult<()>;

    /// Delete expired entries
    async fn delete_expired(&self) -> StorageResult<u64>;

    /// Downcast to concrete type for specialized operations
    ///
    /// This allows accessing backend-specific functionality like Raft cluster operations.
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Backend metrics for monitoring
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BackendMetrics {
    pub reads: u64,
    pub writes: u64,
    pub deletes: u64,
    pub bytes_read: u64,
    pub bytes_written: u64,
}

/// Transaction interface for atomic operations
#[async_trait]
pub trait StorageTransaction: Send + Sync {
    /// Store entry within transaction
    async fn store(&mut self, entry: &VaultEntry) -> StorageResult<()>;

    /// Update entry within transaction
    async fn update(&mut self, entry: &VaultEntry) -> StorageResult<()>;

    /// Delete entry within transaction
    async fn delete(&mut self, id: Uuid) -> StorageResult<bool>;

    /// Commit the transaction
    async fn commit(self: Box<Self>) -> StorageResult<()>;

    /// Rollback the transaction
    async fn rollback(self: Box<Self>) -> StorageResult<()>;
}

/// Health status enum for simple status checks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatusEnum {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Storage health status with detailed information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub response_time_ms: f64,
    pub connections_active: u32,
    pub connections_idle: u32,
    pub last_error: Option<String>,
    pub uptime_seconds: u64,
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub backend_type: String,
    pub total_entries: u64,
    pub total_size_bytes: u64,
    pub average_entry_size: f64,
    pub entries_by_security_level: HashMap<SecurityLevel, u64>,
    pub entries_created_today: u64,
    pub entries_updated_today: u64,
    pub expired_entries: u64,
    pub last_backup: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Backend type (postgres, redis, file)
    pub backend_type: String,

    /// Connection string or path
    pub connection_string: String,

    /// Connection pool settings
    pub pool_settings: PoolSettings,

    /// Encryption settings
    pub encryption_enabled: bool,

    /// Compression settings
    pub compression_enabled: bool,

    /// Backup settings
    pub backup_enabled: bool,

    /// Cache settings
    pub cache_enabled: bool,
}

/// Connection pool settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolSettings {
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout_seconds: u64,
    pub idle_timeout_seconds: u64,
    pub max_lifetime_seconds: u64,
}

impl Default for PoolSettings {
    fn default() -> Self {
        Self {
            max_connections: 10,
            min_connections: 1,
            connection_timeout_seconds: 30,
            idle_timeout_seconds: 600,
            max_lifetime_seconds: 3600,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_vault_entry_initialization_defaults() {
        let owner = Uuid::new_v4();
        let entry = VaultEntry::new(
            "secret/path".to_string(),
            vec![1, 2, 3],
            serde_json::json!({
                "algorithm": "aes-256-gcm",
                "key_id": "key-123"
            }),
            SecurityLevel::Secret,
            owner.to_string(),
        );

        assert_eq!(entry.path, "secret/path");
        assert_eq!(entry.version, 1);
        assert_eq!(entry.security_level, SecurityLevel::Secret);
        assert_eq!(entry.owner_id, owner.to_string());
        assert!(entry.metadata.is_object());
        assert!(entry.tags.is_empty());
        assert!(!entry.is_expired());
    }

    #[test]
    fn test_vault_entry_tag_and_metadata_helpers() {
        let owner = Uuid::new_v4();
        let entry = VaultEntry::new(
            "secret/path".to_string(),
            vec![],
            serde_json::json!({"algorithm": "aes-256-gcm"}),
            SecurityLevel::Confidential,
            owner.to_string(),
        )
        .add_metadata("env".to_string(), serde_json::json!("prod"))
        .add_metadata("region".to_string(), serde_json::json!("apac"))
        .add_tag("finance".to_string())
        .add_tag("finance".to_string())
        .add_tag("internal".to_string());

        assert_eq!(entry.metadata.get("env"), Some(&serde_json::json!("prod")));
        assert_eq!(
            entry.metadata.get("region"),
            Some(&serde_json::json!("apac"))
        );

        let tag_set: HashSet<String> = entry.tags.iter().cloned().collect();
        assert_eq!(tag_set.len(), 2);
        assert!(tag_set.contains(&"finance".to_string()));
        assert!(tag_set.contains(&"internal".to_string()));
    }

    #[test]
    fn test_query_params_helpers() {
        let owner = Uuid::new_v4();
        let params = QueryParams::new()
            .with_path_prefix("apps/".to_string())
            .with_security_level(SecurityLevel::Internal)
            .with_tag("pci".to_string())
            .with_tag("finance".to_string())
            .with_owner(owner)
            .with_limit(50);

        assert_eq!(params.path_prefix.as_deref(), Some("apps/"));
        assert_eq!(params.security_level, Some(SecurityLevel::Internal));
        assert_eq!(params.tags.len(), 2);
        assert_eq!(params.owner_id, Some(owner));
        assert_eq!(params.limit, Some(50));
    }

    #[test]
    fn test_storage_error_debug_and_display() {
        let error = StorageError::NotFound {
            resource_type: "vault_entry".to_string(),
            id: "123".to_string(),
        };

        let display = format!("{}", error);
        assert!(display.contains("Not found"));
        assert!(display.contains("vault_entry"));
        assert!(display.contains("123"));

        let debug = format!("{:?}", error);
        assert!(debug.contains("NotFound"));
    }
}
