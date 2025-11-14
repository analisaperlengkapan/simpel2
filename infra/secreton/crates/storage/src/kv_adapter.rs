//! KV Backend to StorageBackend Adapter
//!
//! Wraps simple KV backends (File, Consul, S3) to implement the full StorageBackend trait.
//! This allows HashiCorp Vault-style KV backends to work with the existing Secreton infrastructure.

use crate::{
    BackendMetrics, HealthStatus, HealthStatusEnum, KvBackend, QueryParams, StorageBackend,
    StorageError, StorageResult, StorageStats, StorageTransaction, VaultEntry,
};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Adapter that wraps a KvBackend to implement StorageBackend
///
/// This is the bridge between simple key-value backends (File, Consul, S3)
/// and the full-featured StorageBackend trait that Secreton expects.
pub struct KvBackendAdapter<B: KvBackend> {
    inner: B,
    metrics: Arc<RwLock<BackendMetrics>>,
}

impl<B: KvBackend> KvBackendAdapter<B> {
    /// Create a new adapter wrapping a KvBackend
    pub fn new(backend: B) -> Self {
        Self {
            inner: backend,
            metrics: Arc::new(RwLock::new(BackendMetrics::default())),
        }
    }

    /// Get reference to the inner backend
    pub fn inner(&self) -> &B {
        &self.inner
    }
}

#[async_trait]
impl<B: KvBackend + Send + Sync + 'static> StorageBackend for KvBackendAdapter<B> {
    /// Store a vault entry (serialized as JSON)
    async fn store(&self, entry: &VaultEntry) -> StorageResult<()> {
        let key = &entry.path;
        let bytes = serde_json::to_vec(entry).map_err(|e| StorageError::SerializationError {
            source: Some(Box::new(e)),
            message: "Failed to serialize VaultEntry".to_string(),
        })?;
        self.inner.put(key, &bytes).await
    }

    /// Retrieve by ID (search all entries - inefficient for KV)
    async fn get_by_id(&self, id: Uuid) -> StorageResult<Option<VaultEntry>> {
        // KV backends don't have efficient ID lookup
        // This is a limitation - consider using path-based lookups instead
        Err(StorageError::BackendError {
            backend: "KV Backend Adapter".to_string(),
            message: "get_by_id not supported - use get_by_path instead".to_string(),
        })
    }

    /// Retrieve by path (primary access pattern for KV)
    async fn get_by_path(&self, path: &str) -> StorageResult<Option<VaultEntry>> {
        let data = self.inner.get(path).await?;
        match data {
            Some(bytes) => {
                let entry = serde_json::from_slice::<VaultEntry>(&bytes).map_err(|e| {
                    StorageError::SerializationError {
                        source: Some(Box::new(e)),
                        message: "Failed to deserialize VaultEntry".to_string(),
                    }
                })?;
                Ok(Some(entry))
            }
            None => Ok(None),
        }
    }

    /// Update entry
    async fn update(&self, entry: &VaultEntry) -> StorageResult<()> {
        self.store(entry).await
    }

    /// Delete by ID (not efficient for KV)
    async fn delete_by_id(&self, _id: Uuid) -> StorageResult<bool> {
        Err(StorageError::BackendError {
            backend: "KV Backend Adapter".to_string(),
            message: "delete_by_id not supported - use delete_by_path instead".to_string(),
        })
    }

    /// Delete by path
    async fn delete_by_path(&self, path: &str) -> StorageResult<bool> {
        if self.inner.exists(path).await? {
            self.inner.delete(path).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// List entries (basic prefix listing)
    async fn list(&self, params: &QueryParams) -> StorageResult<Vec<VaultEntry>> {
        let prefix = params.path_prefix.as_deref().unwrap_or("");
        let keys = self.inner.list(prefix).await?;

        let mut entries = Vec::new();
        for key in keys {
            if let Some(entry) = self.get_by_path(&key).await? {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    /// Count entries
    async fn count(&self, params: &QueryParams) -> StorageResult<u64> {
        let entries = self.list(params).await?;
        Ok(entries.len() as u64)
    }

    /// Check existence
    async fn exists(&self, path: &str) -> StorageResult<bool> {
        self.inner.exists(path).await
    }

    /// Transactions not supported
    async fn begin_transaction(&self) -> StorageResult<Box<dyn StorageTransaction>> {
        Err(StorageError::TransactionNotSupported {
            backend: "KV Backend Adapter".to_string(),
        })
    }

    /// Health check
    async fn health_check(&self) -> StorageResult<HealthStatus> {
        self.inner.health_check().await
    }

    /// Get stats
    async fn get_stats(&self) -> StorageResult<StorageStats> {
        let metrics = self.inner.metrics().await?;
        Ok(StorageStats {
            backend_type: "KV Backend Adapter".to_string(),
            total_entries: 0, // Unknown without scanning
            total_size_bytes: metrics.bytes_written,
            average_entry_size: 0.0,
            entries_by_security_level: std::collections::HashMap::new(),
            entries_created_today: 0,
            entries_updated_today: 0,
            expired_entries: 0,
            last_backup: None,
            metadata: serde_json::json!({}),
        })
    }

    /// Migrations not applicable
    async fn migrate(&self) -> StorageResult<()> {
        Ok(())
    }

    /// Downcast support
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::{FileBackend, FileConfig};
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_adapter_basic_operations() {
        let temp_dir = TempDir::new().unwrap();
        let config = FileConfig {
            path: temp_dir.path().to_path_buf(),
            sync_writes: false,
            file_permissions: 0o600,
            dir_permissions: 0o700,
        };

        let file_backend = FileBackend::new(config).await.unwrap();
        let adapter = KvBackendAdapter::new(file_backend);

        // Test put/get
        adapter
            .put("test_key", b"test_value".to_vec())
            .await
            .unwrap();
        let result = adapter.get("test_key").await.unwrap();
        assert_eq!(result, Some(b"test_value".to_vec()));

        // Test delete
        adapter.delete("test_key").await.unwrap();
        let result = adapter.get("test_key").await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_adapter_entry_operations() {
        let temp_dir = TempDir::new().unwrap();
        let config = FileConfig {
            path: temp_dir.path().to_path_buf(),
            sync_writes: false,
            file_permissions: 0o600,
            dir_permissions: 0o700,
        };

        let file_backend = FileBackend::new(config).await.unwrap();
        let adapter = KvBackendAdapter::new(file_backend);

        // Test put_entry/get_entry
        let entry = VaultEntry {
            key: "test_entry".to_string(),
            data: b"entry_data".to_vec(),
            metadata: Default::default(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        adapter.put_entry(entry.clone()).await.unwrap();
        let result = adapter.get_entry("test_entry").await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().data, b"entry_data".to_vec());
    }
}
