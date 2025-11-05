//! Storage Backend Factory
//!
//! Provides easy creation and configuration of different storage backends.

use crate::{
    MemoryBackend, StorageBackend, StorageError, StorageResult, backends::PostgresBackend,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Storage backend type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StorageBackendType {
    Memory,
    Postgres,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageFactoryConfig {
    pub backend_type: StorageBackendType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub postgres_config: Option<PostgresBackendConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresBackendConfig {
    pub connection_string: String,
    pub max_connections: Option<u32>,
}

impl Default for StorageFactoryConfig {
    fn default() -> Self {
        Self {
            backend_type: StorageBackendType::Memory,
            postgres_config: None,
        }
    }
}

/// Storage factory for creating backend instances
pub struct StorageFactory;

impl StorageFactory {
    /// Create a storage backend from configuration
    pub async fn create(config: StorageFactoryConfig) -> StorageResult<Arc<dyn StorageBackend>> {
        match config.backend_type {
            StorageBackendType::Memory => Ok(Arc::new(MemoryBackend::new())),

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
        }
    }

    pub fn create_memory() -> Arc<dyn StorageBackend> {
        Arc::new(MemoryBackend::new())
    }

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
