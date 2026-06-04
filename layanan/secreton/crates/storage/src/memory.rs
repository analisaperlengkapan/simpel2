//! In-memory storage backend for development and testing

use crate::{
    HealthStatus, QueryParams, SecretEntry, StorageBackend, StorageError, StorageResult,
    StorageStats, StorageTransaction,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// In-memory storage backend
#[derive(Clone)]
pub struct MemoryBackend {
    store: Arc<RwLock<HashMap<Uuid, SecretEntry>>>,
    path_index: Arc<RwLock<HashMap<String, Uuid>>>,
}

impl MemoryBackend {
    /// Create a new in-memory backend
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            path_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for MemoryBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StorageBackend for MemoryBackend {
    async fn store(&self, entry: &SecretEntry) -> StorageResult<()> {
        // Basic validation: path must not be empty. This matches
        // expectations from comprehensive_storage_tests edge cases.
        if entry.path.trim().is_empty() {
            return Err(StorageError::InvalidQuery {
                message: "Path cannot be empty".to_string(),
            });
        }

        let mut store = self.store.write().await;
        let mut path_index = self.path_index.write().await;

        store.insert(entry.id, entry.clone());
        path_index.insert(entry.path.clone(), entry.id);

        Ok(())
    }

    async fn get_by_id(&self, id: Uuid) -> StorageResult<Option<SecretEntry>> {
        let store = self.store.read().await;
        Ok(store.get(&id).cloned())
    }

    async fn get_by_path(&self, path: &str) -> StorageResult<Option<SecretEntry>> {
        // Lock ordering: always acquire `store` before `path_index` (see `store()`,
        // `delete_by_id`, `delete_expired`). Resolve the id from path_index, drop
        // that guard, THEN lock store — never hold path_index while taking store, or
        // we deadlock against writers that hold store and wait on path_index.
        let id = {
            let path_index = self.path_index.read().await;
            path_index.get(path).copied()
        };
        match id {
            Some(id) => {
                let store = self.store.read().await;
                Ok(store.get(&id).cloned())
            }
            None => Ok(None),
        }
    }

    async fn update(&self, entry: &SecretEntry) -> StorageResult<()> {
        let mut store = self.store.write().await;

        if !store.contains_key(&entry.id) {
            return Err(StorageError::NotFound {
                resource_type: "SecretEntry".to_string(),
                id: entry.id.to_string(),
            });
        }

        store.insert(entry.id, entry.clone());
        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> StorageResult<bool> {
        let mut store = self.store.write().await;
        let mut path_index = self.path_index.write().await;

        if let Some(entry) = store.remove(&id) {
            path_index.remove(&entry.path);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn delete_by_path(&self, path: &str) -> StorageResult<bool> {
        let mut store = self.store.write().await;
        let mut path_index = self.path_index.write().await;

        if let Some(id) = path_index.remove(path) {
            store.remove(&id);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn count(&self, params: &QueryParams) -> StorageResult<u64> {
        // Optimization: If no filters other than path_prefix and include_expired=true,
        // use path_index to count keys. This is O(N) on keys only.
        let only_prefix_filter = params.security_level.is_none()
            && params.tags.is_empty()
            && params.owner_id.is_none()
            && params.metadata_filters.is_empty();

        // If we want to include expired (or don't care about checking expiration)
        // AND we only have prefix filter, we can use the path_index directly.
        if only_prefix_filter && params.include_expired {
            let path_index = self.path_index.read().await;
            if let Some(prefix) = &params.path_prefix {
                let count = path_index.keys().filter(|k| k.starts_with(prefix)).count();
                return Ok(count as u64);
            } else {
                return Ok(path_index.len() as u64);
            }
        }

        // If we need to filter out expired entries (common case), we iterate
        // over the store but avoid cloning the full result vector.
        if only_prefix_filter && !params.include_expired {
            let store = self.store.read().await;
            let prefix = params.path_prefix.as_deref();

            let count = store
                .values()
                .filter(|entry| {
                    // Check prefix if it exists
                    if let Some(p) = prefix
                        && !entry.path.starts_with(p)
                    {
                        return false;
                    }
                    // Check expiration
                    !entry.is_expired()
                })
                .count();

            return Ok(count as u64);
        }

        // Fallback to list for complex filters (tags, owner, metadata)
        let entries = self.list(params).await?;
        Ok(entries.len() as u64)
    }

    async fn exists(&self, path: &str) -> StorageResult<bool> {
        let path_index = self.path_index.read().await;
        Ok(path_index.contains_key(path))
    }

    async fn migrate(&self) -> StorageResult<()> {
        // Memory backend doesn't need migration
        Ok(())
    }

    async fn delete_expired(&self) -> StorageResult<u64> {
        let mut store = self.store.write().await;
        let mut path_index = self.path_index.write().await;
        let mut deleted_count = 0;

        let expired_ids: Vec<Uuid> = store
            .values()
            .filter(|entry| entry.is_expired())
            .map(|entry| entry.id)
            .collect();

        for id in expired_ids {
            if let Some(entry) = store.remove(&id) {
                path_index.remove(&entry.path);
                deleted_count += 1;
            }
        }

        Ok(deleted_count)
    }

    async fn compact(&self) -> StorageResult<()> {
        let mut store = self.store.write().await;
        let mut path_index = self.path_index.write().await;

        store.shrink_to_fit();
        path_index.shrink_to_fit();

        Ok(())
    }

    async fn list(&self, params: &QueryParams) -> StorageResult<Vec<SecretEntry>> {
        let store = self.store.read().await;

        // Start with all entries
        let mut entries: Vec<SecretEntry> = store.values().cloned().collect();

        // Apply path prefix filtering if requested. Other filters can be
        // added here as they are needed by callers.
        if let Some(prefix) = &params.path_prefix {
            entries.retain(|entry| entry.path.starts_with(prefix));
        }

        // Filter expired entries unless requested
        if !params.include_expired {
            entries.retain(|entry| !entry.is_expired());
        }

        Ok(entries)
    }

    async fn health_check(&self) -> StorageResult<HealthStatus> {
        Ok(HealthStatus {
            is_healthy: true,
            response_time_ms: 0.0,
            connections_active: 0,
            connections_idle: 0,
            last_error: None,
            uptime_seconds: 0,
        })
    }

    async fn get_stats(&self) -> StorageResult<StorageStats> {
        let store = self.store.read().await;
        Ok(StorageStats {
            backend_type: "memory".to_string(),
            total_entries: store.len() as u64,
            total_size_bytes: 0, // Not tracked in memory
            average_entry_size: 0.0,
            entries_by_security_level: HashMap::new(),
            entries_created_today: 0,
            entries_updated_today: 0,
            expired_entries: 0,
            last_backup: None,
            metadata: serde_json::json!({}),
        })
    }

    async fn begin_transaction(&self) -> StorageResult<Box<dyn StorageTransaction>> {
        Err(StorageError::TransactionNotSupported {
            backend: "memory".to_string(),
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SecurityLevel;

    #[tokio::test]
    async fn test_memory_backend_store_and_retrieve() {
        let backend = MemoryBackend::new();

        let entry = SecretEntry {
            id: Uuid::new_v4(),
            path: "test/secret".to_string(),
            encrypted_data: vec![1, 2, 3, 4],
            encryption_metadata: serde_json::json!({"algorithm": "aes256-gcm"}),
            security_level: SecurityLevel::Confidential,
            metadata: serde_json::json!({"owner": "test"}),
            tags: vec!["test".to_string()],
            version: 1,
            owner_id: "user123".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            expires_at: None,
        };

        // Store
        backend.store(&entry).await.unwrap();

        // Retrieve by ID
        let retrieved = backend.get_by_id(entry.id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().path, "test/secret");

        // Retrieve by path
        let retrieved = backend.get_by_path("test/secret").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, entry.id);
    }

    #[tokio::test]
    async fn test_memory_backend_delete() {
        let backend = MemoryBackend::new();

        let entry = SecretEntry {
            id: Uuid::new_v4(),
            path: "test/secret".to_string(),
            encrypted_data: vec![1, 2, 3, 4],
            encryption_metadata: serde_json::json!({}),
            security_level: SecurityLevel::Internal,
            metadata: serde_json::json!({}),
            tags: vec![],
            version: 1,
            owner_id: "user123".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            expires_at: None,
        };

        backend.store(&entry).await.unwrap();
        backend.delete_by_id(entry.id).await.unwrap();

        let retrieved = backend.get_by_id(entry.id).await.unwrap();
        assert!(retrieved.is_none());
    }
}
