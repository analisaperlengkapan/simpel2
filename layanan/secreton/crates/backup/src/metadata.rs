//! Backup metadata structures

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::BackupStatus;

/// Complete backup containing Raft snapshot and PostgreSQL dump
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backup {
    /// Unique backup identifier
    pub id: String,

    /// Backup creation timestamp
    pub timestamp: DateTime<Utc>,

    /// Encrypted and compressed Raft snapshot
    pub raft_snapshot: Vec<u8>,

    /// Encrypted and compressed PostgreSQL dump
    pub postgres_dump: Vec<u8>,

    /// Backup metadata
    pub metadata: BackupMetadata,
}

/// Metadata about a backup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    /// Backup ID
    pub id: String,

    /// Backup creation timestamp
    pub timestamp: DateTime<Utc>,

    /// Secreton version at backup time
    pub version: String,

    /// Total size in bytes (before compression/encryption)
    pub original_size_bytes: u64,

    /// Compressed size in bytes (after compression, before encryption)
    pub compressed_size_bytes: u64,

    /// Final size in bytes (after compression and encryption)
    pub encrypted_size_bytes: u64,

    /// Raft snapshot size (original)
    pub raft_snapshot_size: u64,

    /// PostgreSQL dump size (original)
    pub postgres_dump_size: u64,

    /// Backup status
    pub status: BackupStatus,

    /// SHA-256 checksum of the backup (for verification)
    pub checksum: String,

    /// Encryption algorithm used
    pub encryption_algorithm: String,

    /// Compression algorithm used
    pub compression_algorithm: Option<String>,

    /// Compression level (if compression enabled)
    pub compression_level: Option<u32>,

    /// Verification timestamp (if verified)
    pub verified_at: Option<DateTime<Utc>>,

    /// Error message (if backup failed)
    pub error: Option<String>,

    /// Additional tags
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Backup {
    /// Create a new backup with generated ID
    pub fn new(raft_snapshot: Vec<u8>, postgres_dump: Vec<u8>) -> Self {
        let id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();

        Self {
            id: id.clone(),
            timestamp,
            raft_snapshot: raft_snapshot.clone(),
            postgres_dump: postgres_dump.clone(),
            metadata: BackupMetadata {
                id,
                timestamp,
                version: env!("CARGO_PKG_VERSION").to_string(),
                original_size_bytes: (raft_snapshot.len() + postgres_dump.len()) as u64,
                compressed_size_bytes: 0, // Will be set after compression
                encrypted_size_bytes: 0,  // Will be set after encryption
                raft_snapshot_size: raft_snapshot.len() as u64,
                postgres_dump_size: postgres_dump.len() as u64,
                status: BackupStatus::InProgress,
                checksum: String::new(), // Will be calculated
                encryption_algorithm: "chacha20-poly1305".to_string(),
                compression_algorithm: Some("gzip".to_string()),
                compression_level: Some(6),
                verified_at: None,
                error: None,
                tags: Vec::new(),
            },
        }
    }

    /// Calculate SHA-256 checksum of the backup
    pub fn calculate_checksum(&self) -> String {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(&self.raft_snapshot);
        hasher.update(&self.postgres_dump);

        hex::encode(hasher.finalize())
    }

    /// Update metadata after compression
    pub fn set_compressed_size(&mut self, size: u64) {
        self.metadata.compressed_size_bytes = size;
    }

    /// Update metadata after encryption
    pub fn set_encrypted_size(&mut self, size: u64) {
        self.metadata.encrypted_size_bytes = size;
    }

    /// Mark backup as completed
    pub fn mark_completed(&mut self) {
        self.metadata.status = BackupStatus::Completed;
        self.metadata.checksum = self.calculate_checksum();
    }

    /// Mark backup as failed
    pub fn mark_failed(&mut self, error: String) {
        self.metadata.status = BackupStatus::Failed;
        self.metadata.error = Some(error);
    }

    /// Mark backup as verified
    pub fn mark_verified(&mut self) {
        self.metadata.status = BackupStatus::Verified;
        self.metadata.verified_at = Some(Utc::now());
    }

    /// Mark backup verification as failed
    pub fn mark_verification_failed(&mut self, error: String) {
        self.metadata.status = BackupStatus::VerificationFailed;
        self.metadata.error = Some(error);
    }
}

impl BackupMetadata {
    /// Get compression ratio (original / compressed)
    pub fn compression_ratio(&self) -> f64 {
        if self.compressed_size_bytes == 0 {
            1.0
        } else {
            self.original_size_bytes as f64 / self.compressed_size_bytes as f64
        }
    }

    /// Get space savings percentage
    pub fn space_savings_percent(&self) -> f64 {
        if self.original_size_bytes == 0 {
            0.0
        } else {
            let saved = self.original_size_bytes - self.compressed_size_bytes;
            (saved as f64 / self.original_size_bytes as f64) * 100.0
        }
    }

    /// Check if backup is verified
    pub fn is_verified(&self) -> bool {
        self.status == BackupStatus::Verified
    }

    /// Check if backup is failed
    pub fn is_failed(&self) -> bool {
        matches!(
            self.status,
            BackupStatus::Failed | BackupStatus::VerificationFailed
        )
    }

    /// Get human-readable size
    pub fn human_readable_size(&self) -> String {
        human_readable_bytes(self.encrypted_size_bytes)
    }
}

/// Convert bytes to human-readable format
fn human_readable_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup_creation() {
        let raft_data = vec![1, 2, 3, 4, 5];
        let postgres_data = vec![6, 7, 8, 9, 10];

        let backup = Backup::new(raft_data.clone(), postgres_data.clone());

        assert_eq!(backup.raft_snapshot, raft_data);
        assert_eq!(backup.postgres_dump, postgres_data);
        assert_eq!(backup.metadata.original_size_bytes, 10);
        assert_eq!(backup.metadata.status, BackupStatus::InProgress);
    }

    #[test]
    fn test_checksum_calculation() {
        let backup = Backup::new(vec![1, 2, 3], vec![4, 5, 6]);
        let checksum = backup.calculate_checksum();

        // Checksum should be consistent
        assert_eq!(checksum, backup.calculate_checksum());
        assert!(!checksum.is_empty());
    }

    #[test]
    fn test_compression_ratio() {
        let mut metadata = BackupMetadata {
            id: "test".to_string(),
            timestamp: Utc::now(),
            version: "0.1.0".to_string(),
            original_size_bytes: 1000,
            compressed_size_bytes: 500,
            encrypted_size_bytes: 550,
            raft_snapshot_size: 600,
            postgres_dump_size: 400,
            status: BackupStatus::Completed,
            checksum: "test".to_string(),
            encryption_algorithm: "chacha20-poly1305".to_string(),
            compression_algorithm: Some("gzip".to_string()),
            compression_level: Some(6),
            verified_at: None,
            error: None,
            tags: Vec::new(),
        };

        assert_eq!(metadata.compression_ratio(), 2.0);
        assert_eq!(metadata.space_savings_percent(), 50.0);
    }

    #[test]
    fn test_human_readable_bytes() {
        assert_eq!(human_readable_bytes(500), "500.00 B");
        assert_eq!(human_readable_bytes(1024), "1.00 KB");
        assert_eq!(human_readable_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(human_readable_bytes(1024 * 1024 * 1024), "1.00 GB");
    }
}
