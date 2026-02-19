//! S3-compatible storage backend for backups
//!
//! This module provides an S3-compatible storage backend that works with:
//! - Amazon S3
//! - MinIO
//! - DigitalOcean Spaces
//! - Wasabi
//! - Any S3-compatible object storage

use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_sdk_s3::{
    config::{Credentials, Region},
    primitives::ByteStream,
    Client as S3Client, Config,
};

use crate::error::{BackupError, Result};
use crate::metadata::{Backup, BackupMetadata};
use crate::storage::BackupStorage;
use crate::types::S3StorageConfig;

/// S3-compatible storage backend
pub struct S3Storage {
    client: S3Client,
    bucket: String,
    prefix: String,
}

impl S3Storage {
    /// Create a new S3 storage backend
    pub async fn new(config: S3StorageConfig) -> Result<Self> {
        let region = Region::new(config.region.clone());

        // Build S3 config
        let mut s3_config_builder = Config::builder().region(region.clone());

        // Set custom endpoint if provided (for MinIO, etc.)
        if let Some(endpoint) = &config.endpoint {
            s3_config_builder = s3_config_builder.endpoint_url(endpoint);
        }

        // Set credentials if provided
        if let (Some(access_key), Some(secret_key)) =
            (&config.access_key_id, &config.secret_access_key)
        {
            let credentials = Credentials::new(
                access_key,
                secret_key,
                None, // session_token
                None, // expiration
                "secreton-backup",
            );
            s3_config_builder = s3_config_builder.credentials_provider(credentials);
        } else {
            // Use default credential chain (IAM role, environment variables, etc.)
            let aws_config = aws_config::defaults(BehaviorVersion::latest())
                .region(region)
                .load()
                .await;
            s3_config_builder = s3_config_builder.credentials_provider(
                aws_config.credentials_provider().unwrap().clone(),
            );
        }

        // Force path-style addressing if configured (required for MinIO)
        if config.force_path_style {
            s3_config_builder = s3_config_builder.force_path_style(true);
        }

        let s3_config = s3_config_builder.build();
        let client = S3Client::from_conf(s3_config);

        let prefix = config.prefix.unwrap_or_default();

        Ok(Self {
            client,
            bucket: config.bucket,
            prefix,
        })
    }

    /// Get the S3 key for a backup file
    fn backup_key(&self, backup_id: &str) -> String {
        format!("{}{}.backup", self.prefix, backup_id)
    }

    /// Get the S3 key for a metadata file
    fn metadata_key(&self, backup_id: &str) -> String {
        format!("{}{}.metadata.json", self.prefix, backup_id)
    }
}

#[async_trait]
impl BackupStorage for S3Storage {
    async fn upload(&self, backup: &Backup) -> Result<()> {
        let backup_key = self.backup_key(&backup.id);
        let metadata_key = self.metadata_key(&backup.id);

        // Serialize backup
        let backup_data = serde_json::to_vec(backup).map_err(|e| {
            BackupError::Serialization(format!("Failed to serialize backup: {}", e))
        })?;

        // Upload backup file
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&backup_key)
            .body(ByteStream::from(backup_data))
            .content_type("application/octet-stream")
            .send()
            .await
            .map_err(|e| {
                BackupError::Storage(format!("Failed to upload backup to S3: {}", e))
            })?;

        // Serialize metadata
        let metadata_data = serde_json::to_vec_pretty(&backup.metadata).map_err(|e| {
            BackupError::Serialization(format!("Failed to serialize metadata: {}", e))
        })?;

        // Upload metadata file
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&metadata_key)
            .body(ByteStream::from(metadata_data))
            .content_type("application/json")
            .send()
            .await
            .map_err(|e| {
                BackupError::Storage(format!("Failed to upload metadata to S3: {}", e))
            })?;

        tracing::info!(
            backup_id = %backup.id,
            bucket = %self.bucket,
            key = %backup_key,
            "Backup uploaded to S3"
        );

        Ok(())
    }

    async fn download(&self, backup_id: &str) -> Result<Backup> {
        let backup_key = self.backup_key(backup_id);

        // Download backup file
        let response = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(&backup_key)
            .send()
            .await
            .map_err(|e| {
                if e.to_string().contains("NoSuchKey") {
                    BackupError::NotFound(format!("Backup not found: {}", backup_id))
                } else {
                    BackupError::Storage(format!("Failed to download backup from S3: {}", e))
                }
            })?;

        // Read backup data
        let backup_data = response.body.collect().await.map_err(|e| {
            BackupError::Storage(format!("Failed to read backup data: {}", e))
        })?;

        // Deserialize backup
        let backup: Backup = serde_json::from_slice(&backup_data.into_bytes()).map_err(|e| {
            BackupError::Deserialization(format!("Failed to deserialize backup: {}", e))
        })?;

        tracing::info!(
            backup_id = %backup_id,
            bucket = %self.bucket,
            key = %backup_key,
            "Backup downloaded from S3"
        );

        Ok(backup)
    }

    async fn list(&self) -> Result<Vec<BackupMetadata>> {
        let mut metadata_list = Vec::new();

        // List objects with prefix
        let mut continuation_token: Option<String> = None;

        loop {
            let mut request = self
                .client
                .list_objects_v2()
                .bucket(&self.bucket)
                .prefix(&self.prefix);

            if let Some(token) = continuation_token {
                request = request.continuation_token(token);
            }

            let response = request.send().await.map_err(|e| {
                BackupError::Storage(format!("Failed to list S3 objects: {}", e))
            })?;

            // Process objects
            if let Some(contents) = response.contents {
                for object in contents {
                    if let Some(key) = object.key {
                        // Only process metadata files
                        if key.ends_with(".metadata.json") {
                            // Download and parse metadata
                            let metadata_response = self
                                .client
                                .get_object()
                                .bucket(&self.bucket)
                                .key(&key)
                                .send()
                                .await
                                .map_err(|e| {
                                    BackupError::Storage(format!(
                                        "Failed to download metadata: {}",
                                        e
                                    ))
                                })?;

                            let metadata_data =
                                metadata_response.body.collect().await.map_err(|e| {
                                    BackupError::Storage(format!("Failed to read metadata: {}", e))
                                })?;

                            let metadata: BackupMetadata =
                                serde_json::from_slice(&metadata_data.into_bytes()).map_err(|e| {
                                    BackupError::Deserialization(format!(
                                        "Failed to deserialize metadata: {}",
                                        e
                                    ))
                                })?;

                            metadata_list.push(metadata);
                        }
                    }
                }
            }

            // Check if there are more results
            if response.is_truncated == Some(true) {
                continuation_token = response.next_continuation_token;
            } else {
                break;
            }
        }

        // Sort by timestamp (newest first)
        metadata_list.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(metadata_list)
    }

    async fn delete(&self, backup_id: &str) -> Result<()> {
        let backup_key = self.backup_key(backup_id);
        let metadata_key = self.metadata_key(backup_id);

        // Delete backup file
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(&backup_key)
            .send()
            .await
            .map_err(|e| BackupError::Storage(format!("Failed to delete backup from S3: {}", e)))?;

        // Delete metadata file
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(&metadata_key)
            .send()
            .await
            .map_err(|e| {
                BackupError::Storage(format!("Failed to delete metadata from S3: {}", e))
            })?;

        tracing::info!(
            backup_id = %backup_id,
            bucket = %self.bucket,
            "Backup deleted from S3"
        );

        Ok(())
    }

    async fn exists(&self, backup_id: &str) -> Result<bool> {
        let backup_key = self.backup_key(backup_id);

        // Try to get object metadata (head_object is cheaper than get_object)
        match self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(&backup_key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => {
                if e.to_string().contains("NotFound") || e.to_string().contains("NoSuchKey") {
                    Ok(false)
                } else {
                    Err(BackupError::Storage(format!(
                        "Failed to check backup existence: {}",
                        e
                    )))
                }
            }
        }
    }

    async fn get_metadata(&self, backup_id: &str) -> Result<BackupMetadata> {
        let metadata_key = self.metadata_key(backup_id);

        // Download metadata file
        let response = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(&metadata_key)
            .send()
            .await
            .map_err(|e| {
                if e.to_string().contains("NoSuchKey") {
                    BackupError::NotFound(format!("Backup metadata not found: {}", backup_id))
                } else {
                    BackupError::Storage(format!("Failed to download metadata from S3: {}", e))
                }
            })?;

        // Read metadata data
        let metadata_data = response.body.collect().await.map_err(|e| {
            BackupError::Storage(format!("Failed to read metadata data: {}", e))
        })?;

        // Deserialize metadata
        let metadata: BackupMetadata =
            serde_json::from_slice(&metadata_data.into_bytes()).map_err(|e| {
                BackupError::Deserialization(format!("Failed to deserialize metadata: {}", e))
            })?;

        Ok(metadata)
    }

    async fn update_metadata(&self, metadata: &BackupMetadata) -> Result<()> {
        let metadata_key = self.metadata_key(&metadata.id);

        // Check if metadata exists
        if !self.exists(&metadata.id).await? {
            return Err(BackupError::NotFound(format!(
                "Backup metadata not found: {}",
                metadata.id
            )));
        }

        // Serialize metadata
        let metadata_data = serde_json::to_vec_pretty(metadata).map_err(|e| {
            BackupError::Serialization(format!("Failed to serialize metadata: {}", e))
        })?;

        // Upload updated metadata file
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&metadata_key)
            .body(ByteStream::from(metadata_data))
            .content_type("application/json")
            .send()
            .await
            .map_err(|e| {
                BackupError::Storage(format!("Failed to update metadata in S3: {}", e))
            })?;

        tracing::debug!(
            backup_id = %metadata.id,
            status = ?metadata.status,
            verified_at = ?metadata.verified_at,
            bucket = %self.bucket,
            "Backup metadata updated in S3"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create S3 config for testing
    /// Note: These tests require a running S3-compatible service (e.g., MinIO)
    /// Set environment variables:
    /// - S3_TEST_BUCKET
    /// - S3_TEST_ENDPOINT (optional, defaults to AWS S3)
    /// - S3_TEST_ACCESS_KEY
    /// - S3_TEST_SECRET_KEY
    fn get_test_config() -> Option<S3StorageConfig> {
        let bucket = std::env::var("S3_TEST_BUCKET").ok()?;
        let endpoint = std::env::var("S3_TEST_ENDPOINT").ok();
        let access_key_id = std::env::var("S3_TEST_ACCESS_KEY").ok();
        let secret_access_key = std::env::var("S3_TEST_SECRET_KEY").ok();

        Some(S3StorageConfig {
            bucket,
            region: "us-east-1".to_string(),
            endpoint,
            access_key_id,
            secret_access_key,
            prefix: Some("test-backups/".to_string()),
            force_path_style: true, // Required for MinIO
        })
    }

    #[tokio::test]
    #[ignore] // Requires S3 credentials
    async fn test_s3_storage_upload_download() {
        let Some(config) = get_test_config() else {
            eprintln!("Skipping S3 test: S3_TEST_BUCKET not set");
            return;
        };

        let storage = S3Storage::new(config).await.unwrap();

        let backup = Backup::new(vec![1, 2, 3], vec![4, 5, 6]);
        let backup_id = backup.id.clone();

        // Upload
        storage.upload(&backup).await.unwrap();

        // Download
        let downloaded = storage.download(&backup_id).await.unwrap();

        assert_eq!(downloaded.id, backup.id);
        assert_eq!(downloaded.raft_snapshot, backup.raft_snapshot);
        assert_eq!(downloaded.postgres_dump, backup.postgres_dump);

        // Cleanup
        storage.delete(&backup_id).await.unwrap();
    }

    #[tokio::test]
    #[ignore] // Requires S3 credentials
    async fn test_s3_storage_list() {
        let Some(config) = get_test_config() else {
            eprintln!("Skipping S3 test: S3_TEST_BUCKET not set");
            return;
        };

        let storage = S3Storage::new(config).await.unwrap();

        // Upload multiple backups
        let backup1 = Backup::new(vec![1], vec![2]);
        let backup2 = Backup::new(vec![3], vec![4]);

        storage.upload(&backup1).await.unwrap();
        storage.upload(&backup2).await.unwrap();

        // List
        let list = storage.list().await.unwrap();

        assert!(list.len() >= 2);

        // Cleanup
        storage.delete(&backup1.id).await.unwrap();
        storage.delete(&backup2.id).await.unwrap();
    }

    #[tokio::test]
    #[ignore] // Requires S3 credentials
    async fn test_s3_storage_delete() {
        let Some(config) = get_test_config() else {
            eprintln!("Skipping S3 test: S3_TEST_BUCKET not set");
            return;
        };

        let storage = S3Storage::new(config).await.unwrap();

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
    #[ignore] // Requires S3 credentials
    async fn test_s3_storage_get_metadata() {
        let Some(config) = get_test_config() else {
            eprintln!("Skipping S3 test: S3_TEST_BUCKET not set");
            return;
        };

        let storage = S3Storage::new(config).await.unwrap();

        let backup = Backup::new(vec![1, 2, 3], vec![4, 5, 6]);
        let backup_id = backup.id.clone();

        storage.upload(&backup).await.unwrap();

        // Get metadata
        let metadata = storage.get_metadata(&backup_id).await.unwrap();

        assert_eq!(metadata.id, backup.id);
        assert_eq!(metadata.original_size_bytes, 6);

        // Cleanup
        storage.delete(&backup_id).await.unwrap();
    }

    #[tokio::test]
    #[ignore] // Requires S3 credentials
    async fn test_s3_storage_not_found() {
        let Some(config) = get_test_config() else {
            eprintln!("Skipping S3 test: S3_TEST_BUCKET not set");
            return;
        };

        let storage = S3Storage::new(config).await.unwrap();

        // Try to download non-existent backup
        let result = storage.download("non-existent-id").await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BackupError::NotFound(_)));
    }
}
