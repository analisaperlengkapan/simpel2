use crate::{
    HealthStatus, QueryParams, StorageBackend, StorageResult, StorageStats, StorageTransaction,
    VaultEntry,
};
use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

pub struct EncryptedStorage {
    backend: Arc<dyn StorageBackend>,
    key_id: String,
}

impl EncryptedStorage {
    pub fn new(backend: Arc<dyn StorageBackend>, key_id: String) -> Self {
        Self { backend, key_id }
    }

    pub fn backend(&self) -> &Arc<dyn StorageBackend> {
        &self.backend
    }

    pub fn key_id(&self) -> &str {
        &self.key_id
    }

    pub fn rotate_key(&mut self, new_key_id: String) {
        self.key_id = new_key_id;
    }
}

#[async_trait]
impl StorageBackend for EncryptedStorage {
    async fn store(&self, entry: &VaultEntry) -> StorageResult<()> {
        self.backend.store(entry).await
    }

    async fn get_by_id(&self, id: Uuid) -> StorageResult<Option<VaultEntry>> {
        self.backend.get_by_id(id).await
    }

    async fn get_by_path(&self, path: &str) -> StorageResult<Option<VaultEntry>> {
        self.backend.get_by_path(path).await
    }

    async fn update(&self, entry: &VaultEntry) -> StorageResult<()> {
        self.backend.update(entry).await
    }

    async fn delete_by_id(&self, id: Uuid) -> StorageResult<bool> {
        self.backend.delete_by_id(id).await
    }

    async fn delete_by_path(&self, path: &str) -> StorageResult<bool> {
        self.backend.delete_by_path(path).await
    }

    async fn count(&self, params: &QueryParams) -> StorageResult<u64> {
        self.backend.count(params).await
    }

    async fn exists(&self, path: &str) -> StorageResult<bool> {
        self.backend.exists(path).await
    }

    async fn begin_transaction(&self) -> StorageResult<Box<dyn StorageTransaction>> {
        self.backend.begin_transaction().await
    }

    async fn migrate(&self) -> StorageResult<()> {
        self.backend.migrate().await
    }

    async fn list(&self, params: &QueryParams) -> StorageResult<Vec<VaultEntry>> {
        self.backend.list(params).await
    }

    async fn health_check(&self) -> StorageResult<HealthStatus> {
        self.backend.health_check().await
    }

    async fn get_stats(&self) -> StorageResult<StorageStats> {
        let mut stats = self.backend.get_stats().await?;

        if let serde_json::Value::Object(ref mut map) = stats.metadata {
            map.insert("encryption_enabled".to_string(), serde_json::json!(true));
            map.insert(
                "encryption_key_id".to_string(),
                serde_json::json!(self.key_id),
            );
        }

        Ok(stats)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MemoryBackend, SecurityLevel};
    use chrono::Utc;

    fn create_test_entry(path: &str) -> VaultEntry {
        VaultEntry {
            id: Uuid::new_v4(),
            path: path.to_string(),
            encrypted_data: vec![1, 2, 3],
            encryption_metadata: serde_json::json!({}),
            security_level: SecurityLevel::Confidential,
            metadata: serde_json::json!({}),
            tags: vec![],
            version: 1,
            owner_id: "test".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: None,
        }
    }

    #[tokio::test]
    async fn test_encrypted_storage_wrapper() {
        let backend = Arc::new(MemoryBackend::new());
        let encrypted = EncryptedStorage::new(backend, "test-key".to_string());

        let entry = create_test_entry("test/path");
        encrypted.store(&entry).await.unwrap();

        let retrieved = encrypted.get_by_id(entry.id).await.unwrap();
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_key_rotation() {
        let backend = Arc::new(MemoryBackend::new());
        let mut encrypted = EncryptedStorage::new(backend, "old-key".to_string());

        assert_eq!(encrypted.key_id(), "old-key");
        encrypted.rotate_key("new-key".to_string());
        assert_eq!(encrypted.key_id(), "new-key");
    }
}
