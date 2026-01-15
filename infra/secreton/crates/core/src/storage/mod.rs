//! Storage module - Re-exports from secreton-storage crate
//!
//! This module provides a thin compatibility layer and re-exports
//! storage functionality from the lib-storage crate.

// Re-export from lib-storage crate
pub use lib_storage::{
    // Backends
    CacheBackend,
    CachedStorage,
    EncryptedStorage,
    EncryptionMetadata,
    HealthStatus,
    InMemoryCache,
    ListOptions,
    MemoryBackend,
    QueryParams,
    SecurityLevel,
    StorageBackend,
    StorageError,
    StorageResult,
    StorageStats,
    StorageTransaction,
    VaultEntry,
};

// Conditional re-exports based on features
#[cfg(feature = "postgres")]
pub use lib_storage::PostgresBackend;

// Application-specific storage modules
pub mod mfa;
pub mod secure;

pub use mfa::{MfaRecoveryCodes, MfaSecret, MfaStorage};
pub use secure::{
    KeyConfig, KeyEntry, KeyStore, MemoryKeyStore, SecureStorage, SharedSecureStorage,
};

// Re-export AuditLog from audit module
pub use crate::audit::AuditLog;

// Legacy type alias for backward compatibility
pub use MemoryBackend as InMemoryStorage;
