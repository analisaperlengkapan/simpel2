//! Type definitions for backup operations

use serde::{Deserialize, Serialize};

/// Backup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    /// Enable automated backups
    pub enabled: bool,

    /// Cron schedule for automated backups (e.g., "0 0 2 * * * *" for daily at 2 AM)
    pub schedule: String,

    /// Retention period in days
    pub retention_days: u32,

    /// Storage backend type (local, s3, etc.)
    pub storage_type: StorageType,

    /// Storage backend configuration
    pub storage_config: StorageConfig,

    /// Encryption key for backups (separate from master key)
    /// This should be stored securely (e.g., in Secreton itself or KMS)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encryption_key: Option<Vec<u8>>,

    /// Enable compression
    #[serde(default = "default_compression")]
    pub compression_enabled: bool,

    /// Compression level (0-9, where 9 is maximum compression)
    #[serde(default = "default_compression_level")]
    pub compression_level: u32,

    /// Enable automatic verification after backup
    #[serde(default = "default_verification")]
    pub verify_after_backup: bool,

    /// PostgreSQL database connection URL
    /// Format: postgresql://user:password@host:port/database
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database_url: Option<String>,

    /// Path to pg_dump binary (defaults to "pg_dump" in PATH)
    #[serde(default = "default_pg_dump_path")]
    pub pg_dump_path: String,

    /// Timeout for pg_dump operation in seconds
    #[serde(default = "default_pg_dump_timeout")]
    pub pg_dump_timeout_secs: u64,
}

fn default_compression() -> bool {
    true
}

fn default_compression_level() -> u32 {
    6
}

fn default_verification() -> bool {
    true
}

fn default_pg_dump_path() -> String {
    "pg_dump".to_string()
}

fn default_pg_dump_timeout() -> u64 {
    300 // 5 minutes
}

/// Storage backend type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StorageType {
    /// Local filesystem storage
    Local,
    /// S3-compatible storage
    S3,
    /// Azure Blob Storage
    Azure,
    /// Google Cloud Storage
    Gcs,
}

/// Storage backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StorageConfig {
    /// Local filesystem configuration
    Local(LocalStorageConfig),
    /// S3-compatible storage configuration
    S3(S3StorageConfig),
    /// Azure Blob Storage configuration
    Azure(AzureStorageConfig),
    /// Google Cloud Storage configuration
    Gcs(GcsStorageConfig),
}

/// Local filesystem storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalStorageConfig {
    /// Base directory for backups
    pub path: String,
}

/// S3-compatible storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3StorageConfig {
    /// S3 bucket name
    pub bucket: String,
    /// S3 region
    pub region: String,
    /// S3 endpoint (for S3-compatible services)
    pub endpoint: Option<String>,
    /// Access key ID
    pub access_key_id: Option<String>,
    /// Secret access key
    pub secret_access_key: Option<String>,
    /// Prefix for backup objects
    pub prefix: Option<String>,
    /// Force path-style addressing (required for MinIO)
    #[serde(default)]
    pub force_path_style: bool,
}

/// Azure Blob Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureStorageConfig {
    /// Storage account name
    pub account_name: String,
    /// Container name
    pub container: String,
    /// Access key
    pub access_key: Option<String>,
    /// Prefix for backup blobs
    pub prefix: Option<String>,
}

/// Google Cloud Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcsStorageConfig {
    /// GCS bucket name
    pub bucket: String,
    /// Service account key JSON
    pub service_account_key: Option<String>,
    /// Prefix for backup objects
    pub prefix: Option<String>,
}

/// Backup status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BackupStatus {
    /// Backup is in progress
    InProgress,
    /// Backup completed successfully
    Completed,
    /// Backup failed
    Failed,
    /// Backup is being verified
    Verifying,
    /// Backup verification passed
    Verified,
    /// Backup verification failed
    VerificationFailed,
}

/// Restore options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreOptions {
    /// Backup ID to restore from
    pub backup_id: String,

    /// Restore Raft state
    #[serde(default = "default_true")]
    pub restore_raft: bool,

    /// Restore PostgreSQL data
    #[serde(default = "default_true")]
    pub restore_postgres: bool,

    /// Skip verification before restore
    #[serde(default)]
    pub skip_verification: bool,

    /// Force restore even if current data exists
    #[serde(default)]
    pub force: bool,
}

fn default_true() -> bool {
    true
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            schedule: "0 0 2 * * * *".to_string(), // Daily at 2 AM
            retention_days: 30,
            storage_type: StorageType::Local,
            storage_config: StorageConfig::Local(LocalStorageConfig {
                path: "/var/backups/secreton".to_string(),
            }),
            encryption_key: None,
            compression_enabled: true,
            compression_level: 6,
            verify_after_backup: true,
            database_url: None,
            pg_dump_path: default_pg_dump_path(),
            pg_dump_timeout_secs: default_pg_dump_timeout(),
        }
    }
}
