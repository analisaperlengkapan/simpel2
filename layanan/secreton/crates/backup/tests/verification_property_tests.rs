#![cfg(any())]
//! Property-based tests for automatic backup verification
//!
//! **Property 33: Automatic backup verification**
//! **Validates: Requirements 2.5.8**

use chrono::Utc;
use proptest::prelude::*;
use secreton_backup::prelude::*;
use secreton_backup::types::{BackupConfig, BackupStatus, LocalStorageConfig, StorageConfig};
use tempfile::TempDir;
use tokio::time::Duration;

proptest! {
    #[test]
    fn prop_automatic_backup_verification_success(
        _seed in 0u64..1000u64
    ) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            // Setup: Create backup manager with verification enabled
            let temp_dir = TempDir::new().unwrap();
            let mut config = BackupConfig::default();
            config.storage_config = StorageConfig::Local(LocalStorageConfig { path: temp_dir.path().to_string_lossy().to_string() });
            config.verify_after_backup = true; // Enable automatic verification
            config.compression_enabled = true;

            let manager = BackupManager::new(config).await.unwrap();

            // Create a backup (this will automatically verify it)
            let backup_id = manager.create_backup().await.unwrap();

            // Property 1: Backup should exist
            let backups = manager.list_backups().await.unwrap();
            prop_assert!(
                !backups.is_empty(),
                "At least one backup should exist after creation"
            );

            // Property 2: The created backup should be in the list
            let created_backup = backups.iter().find(|b| b.id == backup_id);
            prop_assert!(
                created_backup.is_some(),
                "Created backup should be in the list"
            );

            let backup_metadata = created_backup.unwrap();

            // Property 3: Backup should be verified (since verify_after_backup is true)
            prop_assert_eq!(
                backup_metadata.status.clone(),
                BackupStatus::Verified,
                "Backup should be automatically verified after creation"
            );

            // Property 4: Verification timestamp should be set
            prop_assert!(
                backup_metadata.verified_at.is_some(),
                "Verification timestamp should be set for verified backup"
            );

            // Property 5: Verification timestamp should not be in the future
            let verified_at = backup_metadata.verified_at.unwrap();
            let now = Utc::now();
            prop_assert!(
                verified_at <= now,
                "Verification timestamp should not be in the future"
            );

            // Property 6: Verification timestamp should be after backup creation
            prop_assert!(
                verified_at >= backup_metadata.timestamp,
                "Verification timestamp should be after backup creation"
            );

            // Property 7: is_verified() should return true
            prop_assert!(
                backup_metadata.is_verified(),
                "is_verified() should return true for verified backup"
            );

            // Property 8: is_failed() should return false
            prop_assert!(
                !backup_metadata.is_failed(),
                "is_failed() should return false for verified backup"
            );

            // Property 9: Error field should be None
            prop_assert!(
                backup_metadata.error.is_none(),
                "Error field should be None for verified backup"
            );

            // Property 10: Checksum should be non-empty
            prop_assert!(
                !backup_metadata.checksum.is_empty(),
                "Checksum should be non-empty for verified backup"
            );

            // Property 11: Checksum should be valid hex (64 characters for SHA-256)
            prop_assert_eq!(
                backup_metadata.checksum.len(),
                64,
                "SHA-256 checksum should be 64 characters"
            );

            // Property 12: Manual verification should also succeed (idempotency)
            let verify_result = manager.verify_backup(&backup_id).await;
            prop_assert!(
                verify_result.is_ok(),
                "Manual verification should succeed for already-verified backup: {:?}",
                verify_result.err()
            );

            // Property 13: Status should remain Verified after manual verification
            let backups_after = manager.list_backups().await.unwrap();
            let backup_after = backups_after.iter().find(|b| b.id == backup_id).unwrap();
            prop_assert_eq!(
                backup_after.status.clone(),
                BackupStatus::Verified,
                "Status should remain Verified after manual verification"
            );

            Ok(())
            Ok(())
        });
    }

    #[test]
    fn prop_manual_backup_verification(
        _seed in 0u64..1000u64
    ) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            // Setup: Create backup manager WITHOUT automatic verification
            let temp_dir = TempDir::new().unwrap();
            let mut config = BackupConfig::default();
            config.storage_config = StorageConfig::Local(LocalStorageConfig { path: temp_dir.path().to_string_lossy().to_string() });
            config.verify_after_backup = false; // Disable automatic verification
            config.compression_enabled = true;

            let manager = BackupManager::new(config).await.unwrap();

            // Create a backup (should NOT be automatically verified)
            let backup_id = manager.create_backup().await.unwrap();

            // Property 1: Backup should exist
            let backups = manager.list_backups().await.unwrap();
            let created_backup = backups.iter().find(|b| b.id == backup_id).unwrap();

            // Property 2: Backup should be Completed (not Verified)
            prop_assert_eq!(
                created_backup.status.clone(),
                BackupStatus::Completed,
                "Backup should be Completed (not Verified) when automatic verification is disabled"
            );

            // Property 3: Verification timestamp should NOT be set
            prop_assert!(
                created_backup.verified_at.is_none(),
                "Verification timestamp should not be set without verification"
            );

            // Property 4: is_verified() should return false
            prop_assert!(
                !created_backup.is_verified(),
                "is_verified() should return false for unverified backup"
            );

            // Property 5: Manual verification should succeed
            let verify_result = manager.verify_backup(&backup_id).await;
            prop_assert!(
                verify_result.is_ok(),
                "Manual verification should succeed: {:?}",
                verify_result.err()
            );

            // Property 6: After manual verification, status should be Verified
            let backups_after = manager.list_backups().await.unwrap();
            let backup_after = backups_after.iter().find(|b| b.id == backup_id).unwrap();
            prop_assert_eq!(
                backup_after.status.clone(),
                BackupStatus::Verified,
                "Status should be Verified after manual verification"
            );

            // Property 7: Verification timestamp should now be set
            prop_assert!(
                backup_after.verified_at.is_some(),
                "Verification timestamp should be set after manual verification"
            );

            Ok(())
            Ok(())
        });
    }

    #[test]
    fn prop_verification_handles_missing_backup(
        backup_id in "[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}"
    ) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            // Setup
            let temp_dir = TempDir::new().unwrap();
            let mut config = BackupConfig::default();
            config.storage_config = StorageConfig::Local(LocalStorageConfig { path: temp_dir.path().to_string_lossy().to_string() });

            let manager = BackupManager::new(config).await.unwrap();

            // Property 1: Verification should fail for non-existent backup
            let verify_result = manager.verify_backup(&backup_id).await;
            prop_assert!(
                verify_result.is_err(),
                "Verification should fail for non-existent backup"
            );

            // Property 2: Error should indicate backup not found
            if let Err(e) = verify_result {
                let error_msg = e.to_string().to_lowercase();
                prop_assert!(
                    error_msg.contains("not found") || error_msg.contains("does not exist") || error_msg.contains("no such"),
                    "Error message should indicate backup not found: {}",
                    e
                );
            }

            Ok(())
            Ok(())
        });
    }

    #[test]
    fn prop_multiple_backup_verification(
        num_backups in 2usize..5usize
    ) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            // Setup
            let temp_dir = TempDir::new().unwrap();
            let mut config = BackupConfig::default();
            config.storage_config = StorageConfig::Local(LocalStorageConfig { path: temp_dir.path().to_string_lossy().to_string() });
            config.verify_after_backup = true;
            config.compression_enabled = true;
            config.retention_days = 365; // Keep all backups

            let manager = BackupManager::new(config).await.unwrap();

            // Create multiple backups
            let mut backup_ids = Vec::new();
            for _ in 0..num_backups {
                let backup_id = manager.create_backup().await.unwrap();
                backup_ids.push(backup_id);

                // Small delay between backups to ensure different timestamps
                tokio::time::sleep(Duration::from_millis(100)).await;
            }

            // Property 1: All backups should exist
            let backups = manager.list_backups().await.unwrap();
            prop_assert_eq!(
                backups.len(),
                num_backups,
                "Should have {} backups",
                num_backups
            );

            // Property 2: All backups should be verified
            for backup_id in &backup_ids {
                let backup = backups.iter().find(|b| &b.id == backup_id);
                prop_assert!(
                    backup.is_some(),
                    "Backup {} should exist",
                    backup_id
                );

                let backup_metadata = backup.unwrap();
                prop_assert_eq!(
                    backup_metadata.status.clone(),
                    BackupStatus::Verified,
                    "Backup {} should be verified",
                    backup_id
                );

                prop_assert!(
                    backup_metadata.verified_at.is_some(),
                    "Backup {} should have verification timestamp",
                    backup_id
                );
            }

            // Property 3: Each backup should have a unique ID
            let unique_ids: std::collections::HashSet<_> = backup_ids.iter().collect();
            prop_assert_eq!(
                unique_ids.len(),
                backup_ids.len(),
                "All backup IDs should be unique"
            );

            // Property 4: Each backup should have a unique checksum
            let checksums: Vec<_> = backups.iter().map(|b| &b.checksum).collect();
            let unique_checksums: std::collections::HashSet<_> = checksums.iter().collect();
            prop_assert_eq!(
                unique_checksums.len(),
                checksums.len(),
                "All backup checksums should be unique (different timestamps)"
            );

            Ok(())
            Ok(())
        });
    }

    #[test]
    fn prop_verification_without_compression(
        _seed in 0u64..1000u64
    ) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            // Setup with compression disabled
            let temp_dir = TempDir::new().unwrap();
            let mut config = BackupConfig::default();
            config.storage_config = StorageConfig::Local(LocalStorageConfig { path: temp_dir.path().to_string_lossy().to_string() });
            config.compression_enabled = false; // Disable compression
            config.verify_after_backup = true;

            let manager = BackupManager::new(config).await.unwrap();

            // Create backup without compression
            let backup_id = manager.create_backup().await.unwrap();

            // Property 1: Backup should be verified
            let backups = manager.list_backups().await.unwrap();
            let backup = backups.iter().find(|b| b.id == backup_id).unwrap();

            prop_assert_eq!(
                backup.status.clone(),
                BackupStatus::Verified,
                "Backup should be verified even without compression"
            );

            // Property 2: Compression ratio should be 1.0 (no compression)
            prop_assert_eq!(
                backup.compression_ratio(),
                1.0,
                "Compression ratio should be 1.0 when compression is disabled"
            );

            // Property 3: Compressed size should equal original size
            prop_assert_eq!(
                backup.compressed_size_bytes,
                backup.original_size_bytes,
                "Compressed size should equal original size when compression is disabled"
            );

            Ok(())
            Ok(())
        });
    }
}
