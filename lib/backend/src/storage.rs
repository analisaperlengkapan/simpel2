//! Storage module for S3/MinIO operations
//!
//! Provides a client wrapper for S3-compatible storage with
//! file upload/download functions and SHA-256 checksum validation.

#[cfg(feature = "storage")]
use crate::{CommonError, Result};
#[cfg(feature = "storage")]
use aws_config::BehaviorVersion;
#[cfg(feature = "storage")]
use aws_sdk_s3::{
    Client as S3Client, Config,
    config::{Credentials, Region},
    primitives::ByteStream,
};
#[cfg(feature = "storage")]
use sha2::{Digest, Sha256};
#[cfg(feature = "storage")]
use std::path::Path;

/// Storage configuration
#[cfg(feature = "storage")]
#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub endpoint: String,
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket: String,
    pub path_style: bool, // Use path-style URLs (required for MinIO)
}

#[cfg(feature = "storage")]
impl StorageConfig {
    pub fn new(
        endpoint: String,
        region: String,
        access_key: String,
        secret_key: String,
        bucket: String,
    ) -> Self {
        Self {
            endpoint,
            region,
            access_key,
            secret_key,
            bucket,
            path_style: true, // Default to path-style for MinIO compatibility
        }
    }

    pub fn with_path_style(mut self, path_style: bool) -> Self {
        self.path_style = path_style;
        self
    }
}

/// Storage client wrapper for S3/MinIO
#[cfg(feature = "storage")]
pub struct StorageClient {
    client: S3Client,
    bucket: String,
}

#[cfg(feature = "storage")]
impl StorageClient {
    /// Create a new storage client
    pub async fn new(config: StorageConfig) -> Result<Self> {
        let credentials =
            Credentials::new(&config.access_key, &config.secret_key, None, None, "static");

        let s3_config = Config::builder()
            .region(Region::new(config.region))
            .endpoint_url(&config.endpoint)
            .credentials_provider(credentials)
            .force_path_style(config.path_style)
            .behavior_version(BehaviorVersion::latest())
            .build();

        let client = S3Client::from_conf(s3_config);

        Ok(Self {
            client,
            bucket: config.bucket,
        })
    }

    /// Upload a file to storage
    pub async fn upload_file(
        &self,
        key: &str,
        data: Vec<u8>,
        content_type: Option<String>,
    ) -> Result<UploadResult> {
        // Calculate SHA-256 checksum
        let checksum = calculate_sha256(&data);

        // Upload to S3/MinIO
        let mut request = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(data.clone()));

        if let Some(ct) = content_type {
            request = request.content_type(ct);
        }

        // Add checksum as metadata
        request = request.metadata("sha256", &checksum);

        request
            .send()
            .await
            .map_err(|e| CommonError::Internal(format!("Failed to upload file: {}", e)))?;

        Ok(UploadResult {
            key: key.to_string(),
            size: data.len(),
            checksum,
        })
    }

    /// Upload a file from local path
    pub async fn upload_file_from_path(
        &self,
        key: &str,
        file_path: &Path,
        content_type: Option<String>,
    ) -> Result<UploadResult> {
        let data = tokio::fs::read(file_path)
            .await
            .map_err(|e| CommonError::Internal(format!("Failed to read file: {}", e)))?;

        self.upload_file(key, data, content_type).await
    }

    /// Download a file from storage
    pub async fn download_file(&self, key: &str) -> Result<DownloadResult> {
        let response = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| CommonError::Internal(format!("Failed to download file: {}", e)))?;

        // Get stored checksum from metadata
        let stored_checksum = response
            .metadata()
            .and_then(|m| m.get("sha256"))
            .map(|s| s.to_string());

        // Get content type before consuming body
        let content_type = response.content_type().map(|s| s.to_string());

        // Read body
        let data = response
            .body
            .collect()
            .await
            .map_err(|e| CommonError::Internal(format!("Failed to read file body: {}", e)))?
            .into_bytes()
            .to_vec();

        // Calculate checksum of downloaded data
        let calculated_checksum = calculate_sha256(&data);

        // Verify checksum if available
        if let Some(stored) = &stored_checksum
            && stored != &calculated_checksum
        {
            return Err(CommonError::Internal(format!(
                "Checksum mismatch: expected {}, got {}",
                stored, calculated_checksum
            )));
        }

        Ok(DownloadResult {
            key: key.to_string(),
            data,
            checksum: calculated_checksum,
            content_type,
        })
    }

    /// Download a file and save to local path
    pub async fn download_file_to_path(&self, key: &str, file_path: &Path) -> Result<()> {
        let result = self.download_file(key).await?;

        tokio::fs::write(file_path, result.data)
            .await
            .map_err(|e| CommonError::Internal(format!("Failed to write file: {}", e)))?;

        Ok(())
    }

    /// Delete a file from storage
    pub async fn delete_file(&self, key: &str) -> Result<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| CommonError::Internal(format!("Failed to delete file: {}", e)))?;

        Ok(())
    }

    /// Check if a file exists
    pub async fn file_exists(&self, key: &str) -> Result<bool> {
        match self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => {
                let error_str = e.to_string();
                if error_str.contains("NotFound") || error_str.contains("404") {
                    Ok(false)
                } else {
                    Err(CommonError::Internal(format!(
                        "Failed to check file existence: {}",
                        e
                    )))
                }
            }
        }
    }

    /// Get file metadata
    pub async fn get_file_metadata(&self, key: &str) -> Result<FileMetadata> {
        let response = self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| CommonError::Internal(format!("Failed to get file metadata: {}", e)))?;

        Ok(FileMetadata {
            key: key.to_string(),
            size: response.content_length().unwrap_or(0) as usize,
            content_type: response.content_type().map(|s| s.to_string()),
            last_modified: response.last_modified().map(|dt| dt.to_string()),
            checksum: response
                .metadata()
                .and_then(|m| m.get("sha256"))
                .map(|s| s.to_string()),
        })
    }

    /// List files with a prefix
    pub async fn list_files(&self, prefix: Option<&str>) -> Result<Vec<String>> {
        let mut request = self.client.list_objects_v2().bucket(&self.bucket);

        if let Some(p) = prefix {
            request = request.prefix(p);
        }

        let response = request
            .send()
            .await
            .map_err(|e| CommonError::Internal(format!("Failed to list files: {}", e)))?;

        let keys = response
            .contents()
            .iter()
            .filter_map(|obj| obj.key().map(|k| k.to_string()))
            .collect();

        Ok(keys)
    }

    /// Copy a file within the same bucket
    pub async fn copy_file(&self, source_key: &str, dest_key: &str) -> Result<()> {
        let copy_source = format!("{}/{}", self.bucket, source_key);

        self.client
            .copy_object()
            .bucket(&self.bucket)
            .copy_source(&copy_source)
            .key(dest_key)
            .send()
            .await
            .map_err(|e| CommonError::Internal(format!("Failed to copy file: {}", e)))?;

        Ok(())
    }

    /// Generate a presigned URL for temporary access
    pub async fn generate_presigned_url(
        &self,
        key: &str,
        expires_in: std::time::Duration,
    ) -> Result<String> {
        let presigned = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(
                aws_sdk_s3::presigning::PresigningConfig::expires_in(expires_in)
                    .map_err(|e| CommonError::Internal(format!("Invalid expiration: {}", e)))?,
            )
            .await
            .map_err(|e| {
                CommonError::Internal(format!("Failed to generate presigned URL: {}", e))
            })?;

        Ok(presigned.uri().to_string())
    }
}

/// Upload result
#[cfg(feature = "storage")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UploadResult {
    pub key: String,
    pub size: usize,
    pub checksum: String,
}

/// Download result
#[cfg(feature = "storage")]
#[derive(Debug, Clone)]
pub struct DownloadResult {
    pub key: String,
    pub data: Vec<u8>,
    pub checksum: String,
    pub content_type: Option<String>,
}

/// File metadata
#[cfg(feature = "storage")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileMetadata {
    pub key: String,
    pub size: usize,
    pub content_type: Option<String>,
    pub last_modified: Option<String>,
    pub checksum: Option<String>,
}

/// Calculate SHA-256 checksum of data
#[cfg(feature = "storage")]
pub fn calculate_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

/// Verify SHA-256 checksum
#[cfg(feature = "storage")]
pub fn verify_sha256(data: &[u8], expected_checksum: &str) -> bool {
    let calculated = calculate_sha256(data);
    calculated == expected_checksum
}

#[cfg(all(test, feature = "storage"))]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_sha256() {
        let data = b"hello world";
        let checksum = calculate_sha256(data);
        assert_eq!(
            checksum,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_verify_sha256() {
        let data = b"hello world";
        let checksum = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        assert!(verify_sha256(data, checksum));
        assert!(!verify_sha256(data, "invalid"));
    }
}
