#![cfg(any())]
//! Integration tests for backup manager

use secreton_backup::prelude::*;
use secreton_backup::types::{BackupConfig, LocalStorageConfig, RestoreOptions, StorageConfig};
use tempfile::TempDir;
use tokio::time::{Duration, sleep};

/// Helper to create a test backup manager
async fn create_test_manager() -> (BackupManager, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let mut config = BackupConfig::default();
    config.storage_config = StorageConfig::Local(LocalStorageConfig { path: temp_dir.path().to_string_lossy().to_string() });
    config.verify_after_backup = true;
    config.compression_enabled = true;

    let manager = BackupManager::new(config).await.unwrap();
    (manager, temp_dir)
}

#[tokio::test]
async fn test_backup_lifecycle() {
    let (manager, _temp_dir) = create_test_manager().await;

    // Create backup
    let backup_id = manager.create_backup().await.unwrap();
    assert!(!backup_id.is_empty());

    // List backups
    let backups = manager.list_backups().await.unwrap();
    assert_eq!(backups.len(), 1);
    assert_eq!(backups[0].id, backup_id);
    assert!(backups[0].is_verified());

    // Verify backup
    manager.verify_backup(&backup_id).await.unwrap();

    // Restore backup
    let restore_options = RestoreOptions {
        backup_id: backup_id.clone(),
        restore_raft: true,
        restore_postgres: true,
        skip_verification: false,
        force: false,
    };
    manager.restore_backup(restore_options).await.unwrap();

    // Delete backup
    manager.delete_backup(&backup_id).await.unwrap();

    // Verify deleted
    let backups = manager.list_backups().await.unwrap();
    assert_eq!(backups.len(), 0);
}

#[tokio::test]
async fn test_multiple_backups() {
    let (manager, _temp_dir) = create_test_manager().await;

    // Create multiple backups
    let id1 = manager.create_backup().await.unwrap();
    sleep(Duration::from_millis(100)).await; // Ensure different timestamps
    let id2 = manager.create_backup().await.unwrap();
    sleep(Duration::from_millis(100)).await;
    let id3 = manager.create_backup().await.unwrap();

    // List backups (should be sorted by timestamp, newest first)
    let backups = manager.list_backups().await.unwrap();
    assert_eq!(backups.len(), 3);

    // Verify all backups exist
    let ids: Vec<String> = backups.iter().map(|b| b.id.clone()).collect();
    assert!(ids.contains(&id1));
    assert!(ids.contains(&id2));
    assert!(ids.contains(&id3));

    // Verify timestamps are in descending order
    assert!(backups[0].timestamp >= backups[1].timestamp);
    assert!(backups[1].timestamp >= backups[2].timestamp);
}

#[tokio::test]
async fn test_backup_metadata() {
    let (manager, _temp_dir) = create_test_manager().await;

    let backup_id = manager.create_backup().await.unwrap();

    let backups = manager.list_backups().await.unwrap();
    assert_eq!(backups.len(), 1);

    let metadata = &backups[0];

    // Check metadata fields
    assert_eq!(metadata.id, backup_id);
    assert!(!metadata.version.is_empty());
    assert!(metadata.original_size_bytes > 0);
    assert!(metadata.encrypted_size_bytes > 0);
    assert!(!metadata.checksum.is_empty());
    assert_eq!(metadata.encryption_algorithm, "chacha20-poly1305");
    assert_eq!(metadata.compression_algorithm, Some("gzip".to_string()));
    assert!(metadata.is_verified());

    // Check compression ratio
    let ratio = metadata.compression_ratio();
    assert!(ratio > 0.0);

    // Check human-readable size
    let size_str = metadata.human_readable_size();
    assert!(!size_str.is_empty());
}

#[tokio::test]
async fn test_backup_verification_failure() {
    let (manager, _temp_dir) = create_test_manager().await;

    // Verify non-existent backup should fail
    let result = manager.verify_backup("non-existent-id").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_restore_with_skip_verification() {
    let (manager, _temp_dir) = create_test_manager().await;

    let backup_id = manager.create_backup().await.unwrap();

    // Restore with skip_verification
    let restore_options = RestoreOptions {
        backup_id,
        restore_raft: true,
        restore_postgres: true,
        skip_verification: true,
        force: false,
    };

    manager.restore_backup(restore_options).await.unwrap();
}

#[tokio::test]
async fn test_restore_partial() {
    let (manager, _temp_dir) = create_test_manager().await;

    let backup_id = manager.create_backup().await.unwrap();

    // Restore only Raft
    let restore_options = RestoreOptions {
        backup_id: backup_id.clone(),
        restore_raft: true,
        restore_postgres: false,
        skip_verification: false,
        force: false,
    };
    manager.restore_backup(restore_options).await.unwrap();

    // Restore only PostgreSQL
    let restore_options = RestoreOptions {
        backup_id,
        restore_raft: false,
        restore_postgres: true,
        skip_verification: false,
        force: false,
    };
    manager.restore_backup(restore_options).await.unwrap();
}

#[tokio::test]
async fn test_compression_effectiveness() {
    let (manager, _temp_dir) = create_test_manager().await;

    let backup_id = manager.create_backup().await.unwrap();

    let backups = manager.list_backups().await.unwrap();
    let metadata = &backups[0];

    // Compression should reduce size
    assert!(metadata.compressed_size_bytes < metadata.original_size_bytes);

    // Compression ratio should be > 1.0
    assert!(metadata.compression_ratio() > 1.0);

    // Space savings should be > 0%
    assert!(metadata.space_savings_percent() > 0.0);
}

#[tokio::test]
async fn test_encryption_key_validation() {
    let temp_dir = TempDir::new().unwrap();

    // Invalid key length should fail
    let mut config = BackupConfig::default();
    config.storage_config = StorageConfig::Local(LocalStorageConfig { path: temp_dir.path().to_string_lossy().to_string() });
    config.encryption_key = Some(vec![0u8; 16]); // Wrong length

    let result = BackupManager::new(config).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_backup_with_custom_encryption_key() {
    let temp_dir = TempDir::new().unwrap();

    let mut config = BackupConfig::default();
    config.storage_config = StorageConfig::Local(LocalStorageConfig { path: temp_dir.path().to_string_lossy().to_string() });
    config.encryption_key = Some(vec![0u8; 32]); // Valid key

    let manager = BackupManager::new(config).await.unwrap();

    let backup_id = manager.create_backup().await.unwrap();
    assert!(!backup_id.is_empty());

    // Verify backup works with custom key
    manager.verify_backup(&backup_id).await.unwrap();
}

#[tokio::test]
async fn test_backup_without_compression() {
    let temp_dir = TempDir::new().unwrap();

    let mut config = BackupConfig::default();
    config.storage_config = StorageConfig::Local(LocalStorageConfig { path: temp_dir.path().to_string_lossy().to_string() });
    config.compression_enabled = false;

    let manager = BackupManager::new(config).await.unwrap();

    let backup_id = manager.create_backup().await.unwrap();

    let backups = manager.list_backups().await.unwrap();
    let metadata = &backups[0];

    // Without compression, compressed size should equal original size
    assert_eq!(metadata.compressed_size_bytes, metadata.original_size_bytes);
    assert_eq!(metadata.compression_ratio(), 1.0);
    assert_eq!(metadata.space_savings_percent(), 0.0);
}

#[tokio::test]
async fn test_concurrent_backups() {
    let (manager, _temp_dir) = create_test_manager().await;

    // Create multiple backups concurrently
    let handles: Vec<_> = (0..3)
        .map(|_| {
            let manager = manager.clone_for_scheduler();
            tokio::spawn(async move { manager.create_backup().await })
        })
        .collect();

    // Wait for all backups to complete
    let mut backup_ids = Vec::new();
    for handle in handles {
        let backup_id = handle.await.unwrap().unwrap();
        backup_ids.push(backup_id);
    }

    // Verify all backups were created
    assert_eq!(backup_ids.len(), 3);

    let backups = manager.list_backups().await.unwrap();
    assert_eq!(backups.len(), 3);
}
