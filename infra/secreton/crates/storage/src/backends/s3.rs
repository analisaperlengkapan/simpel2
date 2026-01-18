//! AWS S3 Storage Backend
//!
//! Implements HashiCorp Vault-compatible S3 storage backend for cloud-native deployments.
//! S3 provides:
//!
//! - **Unlimited Scalability**: Petabyte-scale storage capacity
//! - **Durability**: 99.999999999% (11 9's) durability
//! - **Availability**: 99.99% availability SLA
//! - **Encryption**: Server-side encryption (SSE-S3, SSE-KMS)
//! - **Versioning**: Built-in object versioning
//! - **Cross-Region**: Multi-region replication
//!
//! # Architecture
//!
//! ```text
//! ┌──────────────┐     ┌─────────────────────────┐
//! │  Secreton 1  │────▶│   AWS S3 Bucket         │
//! └──────────────┘     │  ┌──────────────────┐   │
//!                      │  │ secreton/keys/   │   │
//! ┌──────────────┐     │  │ secreton/audit/  │   │
//! │  Secreton 2  │────▶│  │ secreton/config/ │   │
//! └──────────────┘     │  └──────────────────┘   │
//!                      └─────────────────────────┘
//! ```
//!
//! # Configuration
//!
//! ```toml
//! [storage]
//! backend = "s3"
//! bucket = "my-secreton-vault"
//! region = "us-east-1"
//! access_key = "${AWS_ACCESS_KEY_ID}"
//! secret_key = "${AWS_SECRET_ACCESS_KEY}"
//! prefix = "secreton/"
//! sse_kms_key_id = "arn:aws:kms:us-east-1:..."
//! ```

use crate::{BackendMetrics, KvBackend, StorageError, StorageResult};
use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_sdk_s3::Client;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::error::SdkError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, error};

/// S3 storage backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Config {
    /// S3 bucket name
    pub bucket: String,

    /// AWS region
    pub region: String,

    /// AWS access key ID
    pub access_key: String,

    /// AWS secret access key
    pub secret_key: String,

    /// Key prefix for all Secreton data
    pub prefix: String,

    /// S3 endpoint (for S3-compatible services like MinIO)
    pub endpoint: Option<String>,

    /// Enable server-side encryption with KMS
    pub sse_kms_key_id: Option<String>,

    /// Enable server-side encryption with S3-managed keys
    pub sse_s3: bool,

    /// Request timeout in seconds
    pub timeout_secs: u64,

    /// Maximum number of retries
    pub max_retries: u32,

    /// Enable versioning
    pub versioning: bool,
}

impl Default for S3Config {
    fn default() -> Self {
        Self {
            bucket: "secreton-vault".to_string(),
            region: "us-east-1".to_string(),
            access_key: String::new(),
            secret_key: String::new(),
            prefix: "secreton/".to_string(),
            endpoint: None,
            sse_kms_key_id: None,
            sse_s3: true,
            timeout_secs: 30,
            max_retries: 3,
            versioning: false,
        }
    }
}

/// S3 storage backend implementation
pub struct S3Backend {
    config: S3Config,
    client: Client,
    metrics: Arc<RwLock<BackendMetrics>>,
}

impl S3Backend {
    /// Create a new S3 storage backend
    pub async fn new(config: S3Config) -> StorageResult<Self> {
        let mut sdk_config_loader = aws_config::defaults(BehaviorVersion::latest())
            .region(aws_config::Region::new(config.region.clone()));

        if !config.access_key.is_empty() && !config.secret_key.is_empty() {
            let credentials = aws_sdk_s3::config::Credentials::new(
                config.access_key.clone(),
                config.secret_key.clone(),
                None,
                None,
                "static",
            );
            sdk_config_loader = sdk_config_loader.credentials_provider(
                aws_sdk_s3::config::SharedCredentialsProvider::new(credentials)
            );
        }

        if let Some(ref endpoint) = config.endpoint {
            sdk_config_loader = sdk_config_loader.endpoint_url(endpoint);
        }

        let sdk_config = sdk_config_loader.load().await;

        let client_config_builder = aws_sdk_s3::config::Builder::from(&sdk_config)
             .force_path_style(true); // Useful for MinIO/testing

        let client = Client::from_conf(client_config_builder.build());

        let backend = Self {
            config: config.clone(),
            client,
            metrics: Arc::new(RwLock::new(BackendMetrics::default())),
        };

        // Verify bucket exists and is accessible
        backend.verify_bucket().await?;

        info!(
            "S3 storage backend initialized for bucket: {}",
            config.bucket
        );
        Ok(backend)
    }

    /// Construct S3 object key
    fn object_key(&self, key: &str) -> String {
        format!("{}{}", self.config.prefix, key.trim_start_matches('/'))
    }

    /// Verify bucket exists and is accessible
    async fn verify_bucket(&self) -> StorageResult<()> {
        debug!("Verifying S3 bucket access: {}", self.config.bucket);

        self.client
            .head_bucket()
            .bucket(&self.config.bucket)
            .send()
            .await
            .map_err(|e| {
                 error!("Failed to verify S3 bucket: {}", e);
                 StorageError::ConnectionFailed {
                    message: format!("Failed to verify S3 bucket '{}': {}", self.config.bucket, e),
                    source: Some(Box::new(e)),
                }
            })?;

        Ok(())
    }
}

#[async_trait]
impl KvBackend for S3Backend {
    async fn get(&self, key: &str) -> StorageResult<Option<Vec<u8>>> {
        let object_key = self.object_key(key);
        debug!("S3 GET: {}/{}", self.config.bucket, object_key);

        let result = self.client
            .get_object()
            .bucket(&self.config.bucket)
            .key(&object_key)
            .send()
            .await;

        match result {
            Ok(output) => {
                let bytes = output.body.collect().await
                    .map_err(|e| StorageError::BackendError {
                        backend: "s3".to_string(),
                        message: format!("Failed to read object body: {}", e),
                    })?
                    .into_bytes();
                Ok(Some(bytes.to_vec()))
            },
            Err(e) => {
                 let is_not_found = matches!(&e, SdkError::ServiceError(context) if context.err().is_no_such_key());

                 if is_not_found {
                     return Ok(None);
                 }

                 Err(StorageError::BackendError {
                    backend: "s3".to_string(),
                    message: format!("S3 GetObject failed: {}", e),
                 })
            }
        }
    }

    async fn put(&self, key: &str, value: &[u8]) -> StorageResult<()> {
        let object_key = self.object_key(key);
        debug!("S3 PUT: {}/{}", self.config.bucket, object_key);

        let body = ByteStream::from(value.to_vec());

        let mut builder = self.client
            .put_object()
            .bucket(&self.config.bucket)
            .key(&object_key)
            .body(body);

        if let Some(ref kms_key) = self.config.sse_kms_key_id {
            builder = builder.server_side_encryption(aws_sdk_s3::types::ServerSideEncryption::AwsKms)
                             .ssekms_key_id(kms_key);
        } else if self.config.sse_s3 {
             builder = builder.server_side_encryption(aws_sdk_s3::types::ServerSideEncryption::Aes256);
        }

        builder
            .send()
            .await
            .map_err(|e| StorageError::BackendError {
                backend: "s3".to_string(),
                message: format!("S3 PutObject failed: {}", e),
            })?;

        let mut metrics = self.metrics.write().await;
        metrics.writes += 1;
        metrics.bytes_written += value.len() as u64;

        Ok(())
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        let object_key = self.object_key(key);
        debug!("S3 DELETE: {}/{}", self.config.bucket, object_key);

        self.client
            .delete_object()
            .bucket(&self.config.bucket)
            .key(&object_key)
            .send()
            .await
            .map_err(|e| StorageError::BackendError {
                backend: "s3".to_string(),
                message: format!("S3 DeleteObject failed: {}", e),
            })?;

        let mut metrics = self.metrics.write().await;
        metrics.deletes += 1;

        Ok(())
    }

    async fn list(&self, prefix: &str) -> StorageResult<Vec<String>> {
        let list_prefix = self.object_key(prefix);
        debug!("S3 LIST: {}/{}", self.config.bucket, list_prefix);

        let mut keys = Vec::new();
        let mut continuation_token = None;

        loop {
            let resp = self.client
                .list_objects_v2()
                .bucket(&self.config.bucket)
                .prefix(&list_prefix)
                .set_continuation_token(continuation_token)
                .send()
                .await
                .map_err(|e| StorageError::BackendError {
                    backend: "s3".to_string(),
                    message: format!("S3 ListObjectsV2 failed: {}", e),
                })?;

            if let Some(contents) = resp.contents {
                for object in contents {
                     if let Some(key) = object.key {
                         // Strip the global prefix to return relative keys
                         if let Some(stripped) = key.strip_prefix(&self.config.prefix) {
                             keys.push(stripped.to_string());
                         } else {
                             keys.push(key);
                         }
                     }
                }
            }

            if resp.is_truncated == Some(true) {
                continuation_token = resp.next_continuation_token;
            } else {
                break;
            }
        }

        Ok(keys)
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        let object_key = self.object_key(key);

        match self.client
            .head_object()
            .bucket(&self.config.bucket)
            .key(&object_key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => {
                let is_not_found = matches!(&e, SdkError::ServiceError(context) if context.err().is_not_found());

                if is_not_found {
                    Ok(false)
                } else {
                    Err(StorageError::BackendError {
                        backend: "s3".to_string(),
                        message: format!("S3 HeadObject failed: {}", e),
                    })
                }
            }
        }
    }

    async fn metrics(&self) -> StorageResult<BackendMetrics> {
        Ok(self.metrics.read().await.clone())
    }

    async fn health_check(&self) -> StorageResult<crate::HealthStatus> {
        let start = std::time::Instant::now();
        match self.verify_bucket().await {
            Ok(_) => {
                 Ok(crate::HealthStatus {
                    is_healthy: true,
                    response_time_ms: start.elapsed().as_millis() as f64,
                    connections_active: 1,
                    connections_idle: 0,
                    last_error: None,
                    uptime_seconds: 0, // Not tracked in this simple backend
                })
            }
            Err(e) => {
                Ok(crate::HealthStatus {
                    is_healthy: false,
                    response_time_ms: start.elapsed().as_millis() as f64,
                    connections_active: 0,
                    connections_idle: 0,
                    last_error: Some(e.to_string()),
                    uptime_seconds: 0,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires AWS credentials and S3 bucket
    async fn test_s3_backend() {
        let config = S3Config {
            bucket: "test-secreton-vault".to_string(),
            region: "us-east-1".to_string(),
            access_key: "test-key".to_string(),
            secret_key: "test-secret".to_string(),
            ..Default::default()
        };

        // This will fail without real S3
        let result = S3Backend::new(config).await;
        // Just ensuring it compiles and runs up to the network call
        assert!(result.is_err());
    }
}
