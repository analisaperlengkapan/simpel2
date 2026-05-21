//! Backup storage backends

use async_trait::async_trait;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::error::{BackupError, Result};
use crate::metadata::{Backup, BackupMetadata};

pub mod s3;

pub use s3::S3Storage;

/// Trait for backup storage backends
#[async_trait]
pub trait BackupStorage: Send + Sync {
    /// Upload a backup to storage
    async fn upload(&self, backup: &Backup) -> Result<()>;

    /// Download a backup from storage
    async fn download(&self, backup_id: &str) -> Result<Backup>;

    /// List all backups in storage
    async fn list(&self) -> Result<Vec<BackupMetadata>>;

    /// Delete a backup from storage
    async fn delete(&self, backup_id: &str) -> Result<()>;

    /// Check if a backup exists
    async fn exists(&self, backup_id: &str) -> Result<bool>;

    /// Get backup metadata without downloading full backup
    async fn get_metadata(&self, backup_id: &str) -> Result<BackupMetadata>;

    /// Update backup metadata (e.g., after verification)
    async fn update_metadata(&self, metadata: &BackupMetadata) -> Result<()>;
}

/// Local filesystem storage backend
pub struct LocalStorage {
    base_path: PathBuf,
}

impl LocalStorage {
    /// Create a new local storage backend
    pub fn new(base_path: impl Into<PathBuf>) -> Result<Self> {
        let base_path = base_path.into();

        // Create directory if it doesn't exist
        std::fs::create_dir_all(&base_path).map_err(|e| {
            BackupError::Storage(format!("Failed to create backup directory: {}", e))
        })?;

        Ok(Self { base_path })
    }

    /// Get the path for a backup file
    fn backup_path(&self, backup_id: &str) -> PathBuf {
        self.base_path.join(format!("{}.backup", backup_id))
    }

    /// Get the path for a metadata file
    fn metadata_path(&self, backup_id: &str) -> PathBuf {
        self.base_path.join(format!("{}.metadata.json", backup_id))
    }
}

#[async_trait]
impl BackupStorage for LocalStorage {
    async fn upload(&self, backup: &Backup) -> Result<()> {
        let backup_path = self.backup_path(&backup.id);
        let metadata_path = self.metadata_path(&backup.id);

        // Serialize backup
        let backup_data = serde_json::to_vec(backup)?;

        // Write backup file
        let mut file = fs::File::create(&backup_path)
            .await
            .map_err(|e| BackupError::Storage(format!("Failed to create backup file: {}", e)))?;

        file.write_all(&backup_data)
            .await
            .map_err(|e| BackupError::Storage(format!("Failed to write backup file: {}", e)))?;

        file.sync_all()
            .await
            .map_err(|e| BackupError::Storage(format!("Failed to sync backup file: {}", e)))?;

        // Write metadata file
        let metadata_data = serde_json::to_vec_pretty(&backup.metadata)?;

        let mut metadata_file = fs::File::create(&metadata_path)
            .await
            .map_err(|e| BackupError::Storage(format!("Failed to create metadata file: {}", e)))?;

        metadata_file
            .write_all(&metadata_data)
            .await
            .map_err(|e| BackupError::Storage(format!("Failed to write metadata file: {}", e)))?;

        metadata_file
            .sync_all()
            .await
            .map_err(|e| BackupError::Storage(format!("Failed to sync metadata file: {}", e)))?;

        tracing::info!(
            backup_id = %backup.id,
            path = %backup_path.display(),
            "Backup uploaded to local storage"
        );

        Ok(())
    }

    async fn download(&self, backup_id: &str) -> Result<Backup> {
        let backup_path = self.backup_path(backup_id);

        if !backup_path.exists() {
            return Err(BackupError::NotFound(format!(
                "Backup not found: {}",
                backup_id
            )));
        }

        // Read backup file
        let mut file = fs::File::open(&backup_path)
            .await
            .map_err(|e| BackupError::Storage(format!("Failed to open backup file: {}", e)))?;

        let mut backup_data = Vec::new();
        file.read_to_end(&mut backup_data)
            .await
            .map_err(|e| BackupError::Storage(format!("Failed to read backup file: {}", e)))?;

        // Deserialize backup
        let backup: Backup = serde_json::from_slice(&backup_data)?;

        tracing::info!(
            backup_id = %backup_id,
            path = %backup_path.display(),
            "Backup downloaded from local storage"
        );

        Ok(backup)
    }

    async fn list(&self) -> Result<Vec<BackupMetadata>> {
        let mut metadata_list = Vec::new();

        let mut entries = fs::read_dir(&self.base_path)
            .await
            .map_err(|e| BackupError::Storage(format!("Failed to read backup directory: {}", e)))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| BackupError::Storage(format!("Failed to read directory entry: {}", e)))?
        {
            let path = entry.path();

            // Only process metadata files
            if path.extension().and_then(|s| s.to_str()) == Some("json")
                && let Some(file_name) = path.file_name().and_then(|s| s.to_str())
                && file_name.ends_with(".metadata.json")
            {
                // Read metadata file
                let mut file = fs::File::open(&path).await.map_err(|e| {
                    BackupError::Storage(format!("Failed to open metadata file: {}", e))
                })?;

                let mut metadata_data = Vec::new();
                file.read_to_end(&mut metadata_data).await.map_err(|e| {
                    BackupError::Storage(format!("Failed to read metadata file: {}", e))
                })?;

                // Deserialize metadata
                let metadata: BackupMetadata = serde_json::from_slice(&metadata_data)?;
                metadata_list.push(metadata);
            }
        }

        // Sort by timestamp (newest first)
        metadata_list.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(metadata_list)
    }

    async fn delete(&self, backup_id: &str) -> Result<()> {
        let backup_path = self.backup_path(backup_id);
        let metadata_path = self.metadata_path(backup_id);

        // Delete backup file
        if backup_path.exists() {
            fs::remove_file(&backup_path).await.map_err(|e| {
                BackupError::Storage(format!("Failed to delete backup file: {}", e))
            })?;
        }

        // Delete metadata file
        if metadata_path.exists() {
            fs::remove_file(&metadata_path).await.map_err(|e| {
                BackupError::Storage(format!("Failed to delete metadata file: {}", e))
            })?;
        }

        tracing::info!(
            backup_id = %backup_id,
            "Backup deleted from local storage"
        );

        Ok(())
    }

    async fn exists(&self, backup_id: &str) -> Result<bool> {
        let backup_path = self.backup_path(backup_id);
        Ok(backup_path.exists())
    }

    async fn get_metadata(&self, backup_id: &str) -> Result<BackupMetadata> {
        let metadata_path = self.metadata_path(backup_id);

        if !metadata_path.exists() {
            return Err(BackupError::NotFound(format!(
                "Backup metadata not found: {}",
                backup_id
            )));
        }

        // Read metadata file
        let mut file = fs::File::open(&metadata_path)
            .await
            .map_err(|e| BackupError::Storage(format!("Failed to open metadata file: {}", e)))?;

        let mut metadata_data = Vec::new();
        file.read_to_end(&mut metadata_data)
            .await
            .map_err(|e| BackupError::Storage(format!("Failed to read metadata file: {}", e)))?;

        // Deserialize metadata
        let metadata: BackupMetadata = serde_json::from_slice(&metadata_data)?;

        Ok(metadata)
    }

    async fn update_metadata(&self, metadata: &BackupMetadata) -> Result<()> {
        let metadata_path = self.metadata_path(&metadata.id);

        if !metadata_path.exists() {
            return Err(BackupError::NotFound(format!(
                "Backup metadata not found: {}",
                metadata.id
            )));
        }

        // Serialize metadata
        let metadata_data = serde_json::to_vec_pretty(metadata)?;

        // Write metadata file
        let mut file = fs::File::create(&metadata_path)
            .await
            .map_err(|e| BackupError::Storage(format!("Failed to create metadata file: {}", e)))?;

        file.write_all(&metadata_data)
            .await
            .map_err(|e| BackupError::Storage(format!("Failed to write metadata file: {}", e)))?;

        file.sync_all()
            .await
            .map_err(|e| BackupError::Storage(format!("Failed to sync metadata file: {}", e)))?;

        tracing::debug!(
            backup_id = %metadata.id,
            status = ?metadata.status,
            verified_at = ?metadata.verified_at,
            "Backup metadata updated in local storage"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_local_storage_upload_download() {
        let temp_dir = TempDir::new().unwrap();
        let storage = LocalStorage::new(temp_dir.path()).unwrap();

        let backup = Backup::new(vec![1, 2, 3], vec![4, 5, 6]);
        let backup_id = backup.id.clone();

        // Upload
        storage.upload(&backup).await.unwrap();

        // Download
        let downloaded = storage.download(&backup_id).await.unwrap();

        assert_eq!(downloaded.id, backup.id);
        assert_eq!(downloaded.raft_snapshot, backup.raft_snapshot);
        assert_eq!(downloaded.postgres_dump, backup.postgres_dump);
    }

    #[tokio::test]
    async fn test_local_storage_list() {
        let temp_dir = TempDir::new().unwrap();
        let storage = LocalStorage::new(temp_dir.path()).unwrap();

        // Upload multiple backups
        let backup1 = Backup::new(vec![1], vec![2]);
        let backup2 = Backup::new(vec![3], vec![4]);

        storage.upload(&backup1).await.unwrap();
        storage.upload(&backup2).await.unwrap();

        // List
        let list = storage.list().await.unwrap();

        assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn test_local_storage_delete() {
        let temp_dir = TempDir::new().unwrap();
        let storage = LocalStorage::new(temp_dir.path()).unwrap();

        let backup = Backup::new(vec![1], vec![2]);
        let backup_id = backup.id.clone();

        // Upload
        storage.upload(&backup).await.unwrap();

        // Verify exists
        assert!(storage.exists(&backup_id).await.unwrap());

        // Delete
        storage.delete(&backup_id).await.unwrap();

        // Verify deleted
        assert!(!storage.exists(&backup_id).await.unwrap());
    }

    #[tokio::test]
    async fn test_local_storage_get_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let storage = LocalStorage::new(temp_dir.path()).unwrap();

        let backup = Backup::new(vec![1, 2, 3], vec![4, 5, 6]);
        let backup_id = backup.id.clone();

        storage.upload(&backup).await.unwrap();

        // Get metadata
        let metadata = storage.get_metadata(&backup_id).await.unwrap();

        assert_eq!(metadata.id, backup.id);
        assert_eq!(metadata.original_size_bytes, 6);
    }

    #[tokio::test]
    async fn test_local_storage_update_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let storage = LocalStorage::new(temp_dir.path()).unwrap();

        let mut backup = Backup::new(vec![1, 2, 3], vec![4, 5, 6]);
        backup.mark_completed();
        let backup_id = backup.id.clone();

        storage.upload(&backup).await.unwrap();

        // Get initial metadata
        let mut metadata = storage.get_metadata(&backup_id).await.unwrap();
        assert_eq!(metadata.status, crate::types::BackupStatus::Completed);
        assert!(metadata.verified_at.is_none());

        // Mark as verified
        metadata.status = crate::types::BackupStatus::Verified;
        metadata.verified_at = Some(chrono::Utc::now());

        // Update metadata
        storage.update_metadata(&metadata).await.unwrap();

        // Verify update persisted
        let updated_metadata = storage.get_metadata(&backup_id).await.unwrap();
        assert_eq!(
            updated_metadata.status,
            crate::types::BackupStatus::Verified
        );
        assert!(updated_metadata.verified_at.is_some());
    }
}
