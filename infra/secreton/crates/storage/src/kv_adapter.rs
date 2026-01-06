//! KV Backend to StorageBackend Adapter
//!
//! Wraps simple KV backends (File, Consul, S3) to implement the full StorageBackend trait.
//! This allows any KV store to be used as a backend for Secreton.

use crate::{
    BackendMetrics, HealthStatus, KvBackend, QueryParams, StorageBackend, StorageError,
    StorageResult, StorageStats, StorageTransaction, VaultEntry,
};
use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

/// Adapter that wraps a KvBackend to implement StorageBackend
///
/// This adapter handles:
/// - Serialization/Deserialization of VaultEntry
/// - Path indexing (for list operations)
/// - Metadata management
///
/// Note: This is a simplified adapter. A production implementation would need
/// more robust indexing and transaction support.
pub struct KvBackendAdapter<B: KvBackend> {
    backend: B,
}

impl<B: KvBackend> KvBackendAdapter<B> {
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

    fn key_for_id(id: Uuid) -> String {
        format!("entry/id/{}", id)
    }

    fn key_for_path(path: &str) -> String {
        format!("entry/path/{}", path)
    }

    fn index_key_for_path(path: &str) -> String {
        format!("index/path/{}", path)
    }
}

#[async_trait]
impl<B: KvBackend + Send + Sync + 'static> StorageBackend for KvBackendAdapter<B> {
    async fn store(&self, entry: &VaultEntry) -> StorageResult<()> {
        let serialized = serde_json::to_vec(entry).map_err(|e| StorageError::SerializationError {
            message: format!("Failed to serialize entry: {}", e),
            source: Some(Box::new(e)),
        })?;

        // Store by ID
        self.backend
            .put(&Self::key_for_id(entry.id), &serialized)
            .await?;

        // Store index by path (mapping path -> ID)
        self.backend
            .put(
                &Self::index_key_for_path(&entry.path),
                entry.id.as_bytes(),
            )
            .await?;

        Ok(())
    }

    async fn get_by_id(&self, id: Uuid) -> StorageResult<Option<VaultEntry>> {
        let data = self.backend.get(&Self::key_for_id(id)).await?;

        if let Some(bytes) = data {
            let entry =
                serde_json::from_slice(&bytes).map_err(|e| StorageError::SerializationError {
                    message: format!("Failed to deserialize entry: {}", e),
                    source: Some(Box::new(e)),
                })?;
            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }

    async fn get_by_path(&self, path: &str) -> StorageResult<Option<VaultEntry>> {
        // Look up ID from path index
        let index_data = self.backend.get(&Self::index_key_for_path(path)).await?;

        if let Some(id_bytes) = index_data {
            if let Ok(id) = Uuid::from_slice(&id_bytes) {
                return self.get_by_id(id).await;
            }
        }

        Ok(None)
    }

    async fn update(&self, entry: &VaultEntry) -> StorageResult<()> {
        // Check if exists
        if self.get_by_id(entry.id).await?.is_none() {
            return Err(StorageError::NotFound {
                resource_type: "VaultEntry".to_string(),
                id: entry.id.to_string(),
            });
        }

        self.store(entry).await
    }

    async fn delete_by_id(&self, id: Uuid) -> StorageResult<bool> {
        let entry = self.get_by_id(id).await?;

        if let Some(entry) = entry {
            self.backend.delete(&Self::key_for_id(id)).await?;
            self.backend
                .delete(&Self::index_key_for_path(&entry.path))
                .await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn delete_by_path(&self, path: &str) -> StorageResult<bool> {
        let entry = self.get_by_path(path).await?;

        if let Some(entry) = entry {
            self.delete_by_id(entry.id).await
        } else {
            Ok(false)
        }
    }

    async fn list(&self, params: &QueryParams) -> StorageResult<Vec<VaultEntry>> {
        // This is inefficient for KV stores as it requires listing all keys
        // and retrieving each entry. Production implementations should use
        // proper indexing or search backend.

        let prefix = if let Some(p) = &params.path_prefix {
            format!("index/path/{}", p)
        } else {
            "index/path/".to_string()
        };

        let keys = self.backend.list(&prefix).await?;
        let mut entries = Vec::new();

        for key in keys {
            // Key format: index/path/<actual_path>
            // We need to get the ID stored at this key
            if let Some(id_bytes) = self.backend.get(&key).await? {
                if let Ok(id) = Uuid::from_slice(&id_bytes) {
                    if let Some(entry) = self.get_by_id(id).await? {
                        // Apply filters in memory
                        if let Some(level) = params.security_level {
                            if entry.security_level < level {
                                continue;
                            }
                        }

                        if let Some(owner) = params.owner_id {
                            if entry.owner_id != owner.to_string() {
                                continue;
                            }
                        }

                        entries.push(entry);

                        if let Some(limit) = params.limit {
                            if entries.len() >= limit as usize {
                                break;
                            }
                        }
                    }
                }
            }
        }

        Ok(entries)
    }

    async fn count(&self, params: &QueryParams) -> StorageResult<u64> {
        let entries = self.list(params).await?;
        Ok(entries.len() as u64)
    }

    async fn exists(&self, path: &str) -> StorageResult<bool> {
        self.backend.exists(&Self::index_key_for_path(path)).await
    }

    async fn begin_transaction(&self) -> StorageResult<Box<dyn StorageTransaction>> {
        Err(StorageError::TransactionNotSupported {
            backend: "kv-adapter".to_string(),
        })
    }

    async fn health_check(&self) -> StorageResult<HealthStatus> {
        self.backend.health_check().await
    }

    async fn get_stats(&self) -> StorageResult<StorageStats> {
        let metrics = self.backend.metrics().await?;

        Ok(StorageStats {
            backend_type: "kv-adapter".to_string(),
            total_entries: 0, // Difficult to count efficiently
            total_size_bytes: metrics.bytes_written,
            average_entry_size: 0.0,
            entries_by_security_level: HashMap::new(),
            entries_created_today: 0,
            entries_updated_today: 0,
            expired_entries: 0,
            last_backup: None,
            metadata: serde_json::json!({
                "reads": metrics.reads,
                "writes": metrics.writes,
                "deletes": metrics.deletes
            }),
        })
    }

    async fn migrate(&self) -> StorageResult<()> {
        Ok(())
    }

    async fn delete_expired(&self) -> StorageResult<u64> {
        // Inefficient implementation: iterate over all entries
        let all_entries = self.list(&QueryParams::new()).await?;
        let mut count = 0;

        for entry in all_entries {
            if entry.is_expired() {
                if self.delete_by_id(entry.id).await? {
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
