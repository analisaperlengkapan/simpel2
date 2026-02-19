//! Storage module - Re-exports from secreton-storage crate
//!
//! This module provides a thin compatibility layer and re-exports
//! storage functionality from the secreton-storage crate.

// Re-export from secreton-storage crate
pub use secreton_storage::{
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
    SecretEntry,
    SecurityLevel,
    StorageBackend,
    StorageError,
    StorageResult,
    StorageStats,
    StorageTransaction,
};

// Conditional re-exports based on features
#[cfg(feature = "postgres")]
pub use secreton_storage::PostgresBackend;

// Application-specific storage modules
pub mod mfa;
pub mod sealed_keys;
pub mod secure;

pub use mfa::{MfaRecoveryCodes, MfaSecret, MfaStorage};
pub use sealed_keys::{
    PostgresSealedKeyStorage, ProviderType, SealedKeyStorage, SealedMasterKey,
};
pub use secure::{
    KeyConfig, KeyEntry, KeyStore, MemoryKeyStore, SecureStorage, SharedSecureStorage,
};

// Re-export AuditLog from audit module
pub use crate::audit::AuditLog;

// Legacy type alias for backward compatibility
pub use MemoryBackend as InMemoryStorage;
