//! Storage prelude - commonly used storage types and traits
//!
//! This module re-exports the most commonly used items from the storage crate
//! to simplify imports in other modules.

// Re-export error types
pub use crate::{StorageError, StorageResult};

// Re-export core types
pub use crate::{
    EncryptionMetadata, HealthStatus, ListOptions, QueryParams, SecretEntry, SecurityLevel,
    StorageStats,
};

// Re-export traits
pub use crate::{StorageBackend, StorageTransaction};

// Re-export backends
#[cfg(feature = "postgres")]
pub use crate::backends::PostgresBackend;
pub use crate::memory::MemoryBackend;

// Re-export storage wrappers
pub use crate::cache::{CacheBackend, CacheStats, CachedStorage, InMemoryCache};
pub use crate::encrypted_storage::EncryptedStorage;

// Re-export factory
pub use crate::factory::{StorageBackendType, StorageFactory, StorageFactoryConfig};

// Re-export Raft components (if enabled)
#[cfg(feature = "raft-consensus")]
pub use crate::raft::{
    Raft, RaftCluster, RaftClusterConfig, RaftStatus, SecretonRaftStorage, SecretonStateMachine,
};

// Re-export commonly used external crates
pub use async_trait::async_trait;
pub use chrono::{DateTime, Utc};
pub use serde::{Deserialize, Serialize};
pub use std::sync::Arc;
pub use uuid::Uuid;
