//! Error types for replication operations

use thiserror::Error;

/// Errors that can occur during replication operations
#[derive(Error, Debug)]
pub enum ReplicationError {
    /// Configuration error
    #[error("Replication configuration error: {0}")]
    ConfigError(String),

    /// Connection error to secondary node
    #[error("Failed to connect to secondary node {node_id}: {source}")]
    ConnectionError {
        node_id: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Replication lag exceeded threshold
    #[error("Replication lag ({lag_ms}ms) exceeded threshold ({threshold_ms}ms)")]
    LagExceeded {
        lag_ms: u64,
        threshold_ms: u64,
    },

    /// Operation not supported in current mode
    #[error("Operation not supported in {mode} mode: {operation}")]
    UnsupportedOperation {
        mode: String,
        operation: String,
    },

    /// Secondary node not found
    #[error("Secondary node not found: {node_id}")]
    NodeNotFound {
        node_id: String,
    },

    /// Promotion failed
    #[error("Failed to promote secondary to primary: {reason}")]
    PromotionFailed {
        reason: String,
    },

    /// Replication stream error
    #[error("Replication stream error: {0}")]
    StreamError(String),

    /// Snapshot error
    #[error("Snapshot error: {0}")]
    SnapshotError(String),

    /// Storage error
    #[error("Storage error during replication: {0}")]
    StorageError(#[from] secreton_storage::StorageError),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// gRPC transport error
    #[error("gRPC transport error: {0}")]
    TransportError(#[from] tonic::transport::Error),

    /// gRPC status error
    #[error("gRPC status error: {0}")]
    StatusError(#[from] tonic::Status),

    /// Internal error
    #[error("Internal replication error: {0}")]
    Internal(String),
}

impl ReplicationError {
    /// Create a configuration error
    pub fn config<S: Into<String>>(msg: S) -> Self {
        Self::ConfigError(msg.into())
    }

    /// Create a connection error
    pub fn connection<E>(node_id: String, source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::ConnectionError {
            node_id,
            source: Box::new(source),
        }
    }

    /// Create an internal error
    pub fn internal<S: Into<String>>(msg: S) -> Self {
        Self::Internal(msg.into())
    }

    /// Create a snapshot error
    pub fn snapshot<S: Into<String>>(msg: S) -> Self {
        Self::SnapshotError(msg.into())
    }
}
