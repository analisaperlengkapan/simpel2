//! Storage Backend Factory
//!
//! Provides easy creation and configuration of different storage backends.
//! Follows HashiCorp Vault patterns for backend selection and configuration.

use crate::{
    MemoryBackend, StorageBackend, StorageError, StorageResult,
    backends::{FileBackend, FileConfig},
};

#[cfg(feature = "postgres")]
use crate::backends::PostgresBackend;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[cfg(feature = "consul")]
use crate::backends::{ConsulBackend, ConsulConfig};

#[cfg(feature = "s3")]
use crate::backends::{S3Backend, S3Config};

/// Storage backend type enumeration
///
/// # Recommendations (HashiCorp Vault-style)
///
/// - **Production HA**: `Consul` or `Raft` (no database required!)
/// - **Cloud**: `S3` for AWS, Azure Blob, or GCS
/// - **Development**: `File` or `Memory`
/// - **Legacy**: `Postgres` (optional, not recommended)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StorageBackendType {
    /// In-memory storage (ephemeral, testing only)
    Memory,

    /// File system storage (local, single-node)
    File,

    /// Consul distributed KV store (HA, recommended)
    #[cfg(feature = "consul")]
    Consul,

    /// OpenRaft consensus (HA, built-in)
    #[cfg(feature = "raft-consensus")]
    Raft,

    /// AWS S3 and S3-compatible storage (cloud-native)
    #[cfg(feature = "s3")]
    S3,

    /// PostgreSQL relational database (optional, not for HA)
    #[cfg(feature = "postgres")]
    Postgres,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageFactoryConfig {
    pub backend_type: StorageBackendType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_config: Option<FileConfig>,

    #[cfg(feature = "consul")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consul_config: Option<ConsulConfig>,

    #[cfg(feature = "s3")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub s3_config: Option<S3Config>,

    #[cfg(feature = "postgres")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub postgres_config: Option<PostgresBackendConfig>,
}

#[cfg(feature = "postgres")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresBackendConfig {
    pub connection_string: String,
    pub max_connections: Option<u32>,
}

impl Default for StorageFactoryConfig {
    fn default() -> Self {
        Self {
            // Default to file-based storage (no external dependencies)
            backend_type: StorageBackendType::File,
            file_config: Some(FileConfig::default()),

            #[cfg(feature = "consul")]
            consul_config: None,

            #[cfg(feature = "s3")]
            s3_config: None,

            #[cfg(feature = "postgres")]
            postgres_config: None,
        }
    }
}

/// Storage factory for creating backend instances
///
/// # Examples
///
/// ```rust,no_run
/// use secreton_storage::{StorageFactory, StorageBackendType, StorageFactoryConfig};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create file-based storage (default, no external dependencies)
/// let backend = StorageFactory::create_file("/var/lib/secreton/data").await?;
///
/// // Create Consul storage (HA, like HashiCorp Vault)
/// #[cfg(feature = "consul")]
/// let backend = StorageFactory::create_consul("127.0.0.1:8500", "secreton/").await?;
///
/// // Create from config
/// let config = StorageFactoryConfig::default(); // Uses file backend
/// let backend = StorageFactory::create(config).await?;
/// # Ok(())
/// # }
/// ```
pub struct StorageFactory;

impl StorageFactory {
    /// Create a storage backend from configuration
    pub async fn create(config: StorageFactoryConfig) -> StorageResult<Arc<dyn StorageBackend>> {
        match config.backend_type {
            StorageBackendType::Memory => Ok(Arc::new(MemoryBackend::new())),

            StorageBackendType::File => {
                let file_config = config.file_config.unwrap_or_default();
                let backend = FileBackend::new(file_config).await?;
                let adapter = crate::KvBackendAdapter::new(backend);
                Ok(Arc::new(adapter))
            }

            #[cfg(feature = "consul")]
            StorageBackendType::Consul => {
                let consul_config =
                    config
                        .consul_config
                        .ok_or_else(|| StorageError::ConfigurationError {
                            message: "Consul backend configuration required".to_string(),
                        })?;
                let backend = ConsulBackend::new(consul_config).await?;
                let adapter = crate::KvBackendAdapter::new(backend);
                Ok(Arc::new(adapter))
            }

            #[cfg(feature = "s3")]
            StorageBackendType::S3 => {
                let s3_config =
                    config
                        .s3_config
                        .ok_or_else(|| StorageError::ConfigurationError {
                            message: "S3 backend configuration required".to_string(),
                        })?;
                let backend = S3Backend::new(s3_config).await?;
                let adapter = crate::KvBackendAdapter::new(backend);
                Ok(Arc::new(adapter))
            }

            #[cfg(feature = "postgres")]
            StorageBackendType::Postgres => {
                let postgres_config =
                    config
                        .postgres_config
                        .ok_or_else(|| StorageError::ConfigurationError {
                            message: "PostgreSQL backend configuration required".to_string(),
                        })?;
                let backend = PostgresBackend::new(&postgres_config.connection_string).await?;
                Ok(Arc::new(backend))
            }

            #[cfg(feature = "raft-consensus")]
            StorageBackendType::Raft => Err(StorageError::ConfigurationError {
                message: "Raft backend requires RaftCluster - use RaftCluster::new() directly"
                    .to_string(),
            }),
        }
    }

    // Convenience constructors

    pub fn create_memory() -> Arc<dyn StorageBackend> {
        Arc::new(MemoryBackend::new())
    }

    pub async fn create_file(
        path: impl Into<std::path::PathBuf>,
    ) -> StorageResult<Arc<dyn StorageBackend>> {
        let config = FileConfig {
            path: path.into(),
            ..Default::default()
        };
        let backend = FileBackend::new(config).await?;
        let adapter = crate::KvBackendAdapter::new(backend);
        Ok(Arc::new(adapter))
    }

    #[cfg(feature = "consul")]
    pub async fn create_consul(
        address: &str,
        path: &str,
    ) -> StorageResult<Arc<dyn StorageBackend>> {
        let config = ConsulConfig {
            address: address.to_string(),
            path: path.to_string(),
            ..Default::default()
        };
        let backend = ConsulBackend::new(config).await?;
        let adapter = crate::KvBackendAdapter::new(backend);
        Ok(Arc::new(adapter))
    }

    #[cfg(feature = "s3")]
    pub async fn create_s3(
        bucket: &str,
        region: &str,
        access_key: &str,
        secret_key: &str,
    ) -> StorageResult<Arc<dyn StorageBackend>> {
        let config = S3Config {
            bucket: bucket.to_string(),
            region: region.to_string(),
            access_key: access_key.to_string(),
            secret_key: secret_key.to_string(),
            ..Default::default()
        };
        let backend = S3Backend::new(config).await?;
        let adapter = crate::KvBackendAdapter::new(backend);
        Ok(Arc::new(adapter))
    }

    #[cfg(feature = "postgres")]
    pub async fn create_postgres(
        connection_string: &str,
        _max_connections: Option<u32>,
    ) -> StorageResult<Arc<dyn StorageBackend>> {
        let backend = PostgresBackend::new(connection_string).await?;
        Ok(Arc::new(backend))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = StorageFactoryConfig::default();
        assert_eq!(config.backend_type, StorageBackendType::Memory);
    }

    #[tokio::test]
    async fn test_create_memory_backend() {
        let config = StorageFactoryConfig {
            backend_type: StorageBackendType::Memory,
            ..Default::default()
        };

        let backend = StorageFactory::create(config).await;
        assert!(backend.is_ok());
    }

    #[test]
    fn test_create_memory_convenience() {
        let backend = StorageFactory::create_memory();
        assert!(Arc::strong_count(&backend) == 1);
    }
}
