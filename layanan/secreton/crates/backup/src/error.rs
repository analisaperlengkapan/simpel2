//! Error types for backup operations

use thiserror::Error;

/// Result type for backup operations
pub type Result<T> = std::result::Result<T, BackupError>;

/// Errors that can occur during backup operations
#[derive(Debug, Error)]
pub enum BackupError {
    /// Storage backend error
    #[error("Storage error: {0}")]
    Storage(String),

    /// Encryption error
    #[error("Encryption error: {0}")]
    Encryption(String),

    /// Decryption error
    #[error("Decryption error: {0}")]
    Decryption(String),

    /// Compression error
    #[error("Compression error: {0}")]
    Compression(String),

    /// Decompression error
    #[error("Decompression error: {0}")]
    Decompression(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// Backup not found
    #[error("Backup not found: {0}")]
    NotFound(String),

    /// Invalid backup format
    #[error("Invalid backup format: {0}")]
    InvalidFormat(String),

    /// Verification failed
    #[error("Backup verification failed: {0}")]
    VerificationFailed(String),

    /// Scheduler error
    #[error("Scheduler error: {0}")]
    Scheduler(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Raft snapshot error
    #[error("Raft snapshot error: {0}")]
    RaftSnapshot(String),

    /// Raft restore error
    #[error("Raft restore error: {0}")]
    RaftRestore(String),

    /// PostgreSQL dump error
    #[error("PostgreSQL dump error: {0}")]
    PostgresDump(String),

    /// PostgreSQL restore error
    #[error("PostgreSQL restore error: {0}")]
    PostgresRestore(String),

    /// Restore error
    #[error("Restore error: {0}")]
    Restore(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic error
    #[error("{0}")]
    Other(String),
}

impl From<serde_json::Error> for BackupError {
    fn from(err: serde_json::Error) -> Self {
        BackupError::Serialization(err.to_string())
    }
}

// Note: aes_gcm::Error and chacha20poly1305::Error are the same type
// Only implement From for one of them to avoid conflict
impl From<chacha20poly1305::Error> for BackupError {
    fn from(err: chacha20poly1305::Error) -> Self {
        BackupError::Encryption(err.to_string())
    }
}
