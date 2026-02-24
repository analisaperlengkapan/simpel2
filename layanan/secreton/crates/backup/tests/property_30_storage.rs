//! Property 30: Backup storage
//!
//! **Validates: Requirements 2.5.4**
//!
//! For any created backup, it should be uploaded to the configured storage
//! and maintain data integrity through upload/download cycles.

use proptest::prelude::*;
use secreton_backup::error::BackupError;
use secreton_backup::metadata::Backup;
use secreton_backup::storage::{BackupStorage, LocalStorage};
use tempfile::TempDir;

// ============================================================================
// Test Strategies
// ============================================================================

/// Strategy to generate random Raft snapshot data
fn raft_snapshot_strategy() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>(), 100..10000)
}

/// Strategy to generate random PostgreSQL dump data
fn postgres_dump_strategy() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>(), 100..10000)
}

// ============================================================================
// Property 30: Backup storage operations maintain data integrity
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    /// **Property 30: Backup storage operations maintain data integrity**
    ///
    /// For any backup created and uploaded to storage:
    /// 1. Upload operation succeeds
    /// 2. Downloaded backup matches original backup
    /// 3. Metadata is correctly stored and retrievable
    /// 4. List operation includes the uploaded backup
    /// 5. Exists check returns true for uploaded backup
    /// 6. Delete operation removes the backup completely
    ///
    /// This property ensures that backup storage operations (upload, download,
    /// list, delete) maintain data integrity and consistency across the full
    /// lifecycle of a backup.
    ///
    /// **Validates: Requirements 2.5.4**
    #[test]
    fn prop_backup_storage_integrity(
        raft_data in raft_snapshot_strategy(),
        postgres_data in postgres_dump_strategy()
    ) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            // Setup: Create temporary storage
            let temp_dir = TempDir::new().unwrap();
            let storage = LocalStorage::new(temp_dir.path()).unwrap();

            // Create a backup with the generated data
            let mut backup = Backup::new(raft_data.clone(), postgres_data.clone());
            backup.mark_completed();

            let backup_id = backup.id.clone();

            // Property 1: Upload operation should succeed
            let upload_result = storage.upload(&backup).await;
            prop_assert!(
                upload_result.is_ok(),
                "Upload operation should succeed: {:?}",
                upload_result.err()
            );

            // Property 2: Backup should exist after upload
            let exists = storage.exists(&backup_id).await.unwrap();
            prop_assert!(
                exists,
                "Backup should exist after upload"
            );

            // Property 3: Download should succeed
            let download_result = storage.download(&backup_id).await;
            prop_assert!(
                download_result.is_ok(),
                "Download operation should succeed: {:?}",
                download_result.err()
            );

            let downloaded = download_result.unwrap();

            // Property 4: Downloaded backup ID should match original
            prop_assert_eq!(
                downloaded.id,
                backup.id,
                "Downloaded backup ID should match original"
            );

            // Property 5: Downloaded Raft snapshot should match original
            prop_assert_eq!(
                downloaded.raft_snapshot,
                raft_data,
                "Downloaded Raft snapshot should match original data"
            );

            // Property 6: Downloaded PostgreSQL dump should match original
            prop_assert_eq!(
                downloaded.postgres_dump,
                postgres_data,
                "Downloaded PostgreSQL dump should match original data"
            );

            // Property 7: Downloaded metadata should match original metadata
            prop_assert_eq!(
                downloaded.metadata.id,
                backup.metadata.id,
                "Downloaded metadata ID should match original"
            );

            prop_assert_eq!(
                downloaded.metadata.original_size_bytes,
                backup.metadata.original_size_bytes,
                "Downloaded metadata size should match original"
            );

            prop_assert_eq!(
                downloaded.metadata.checksum,
                backup.metadata.checksum,
                "Downloaded metadata checksum should match original"
            );

            // Property 8: Get metadata should return correct metadata
            let metadata_result = storage.get_metadata(&backup_id).await;
            prop_assert!(
                metadata_result.is_ok(),
                "Get metadata operation should succeed"
            );

            let metadata = metadata_result.unwrap();
            prop_assert_eq!(
                metadata.id,
                backup.id,
                "Metadata ID should match backup ID"
            );

            prop_assert_eq!(
                metadata.checksum,
                backup.metadata.checksum,
                "Metadata checksum should match backup checksum"
            );

            // Property 9: List should include the uploaded backup
            let list_result = storage.list().await;
            prop_assert!(
                list_result.is_ok(),
                "List operation should succeed"
            );

            let list = list_result.unwrap();
            prop_assert!(
                list.iter().any(|m| m.id == backup_id),
                "List should include the uploaded backup"
            );

            // Property 10: Delete should succeed
            let delete_result = storage.delete(&backup_id).await;
            prop_assert!(
                delete_result.is_ok(),
                "Delete operation should succeed: {:?}",
                delete_result.err()
            );

            // Property 11: Backup should not exist after delete
            let exists_after_delete = storage.exists(&backup_id).await.unwrap();
            prop_assert!(
                !exists_after_delete,
                "Backup should not exist after delete"
            );

            // Property 12: Download after delete should fail
            let download_after_delete = storage.download(&backup_id).await;
            prop_assert!(
                download_after_delete.is_err(),
                "Download after delete should fail"
            );

            // Property 13: Get metadata after delete should fail
            let metadata_after_delete = storage.get_metadata(&backup_id).await;
            prop_assert!(
                metadata_after_delete.is_err(),
                "Get metadata after delete should fail"
            );

            // Property 14: List should not include deleted backup
            let list_after_delete = storage.list().await.unwrap();
            prop_assert!(
                !list_after_delete.iter().any(|m| m.id == backup_id),
                "List should not include deleted backup"
            )
        });
    }

    /// **Property 30.1: Multiple backup storage operations**
    ///
    /// Storage should correctly handle multiple backups:
    /// 1. Multiple backups can be uploaded
    /// 2. List returns all uploaded backups
    /// 3. Each backup can be downloaded independently
    /// 4. Deleting one backup doesn't affect others
    ///
    /// **Validates: Requirements 2.5.4**
    #[test]
    fn prop_multiple_backup_storage(
        backups_data in prop::collection::vec(
            (raft_snapshot_strategy(), postgres_dump_strategy()),
            2..5
        )
    ) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let temp_dir = TempDir::new().unwrap();
            let storage = LocalStorage::new(temp_dir.path()).unwrap();

            // Create and upload multiple backups
            let mut backup_ids = Vec::new();
            for (raft_data, postgres_data) in &backups_data {
                let mut backup = Backup::new(raft_data.clone(), postgres_data.clone());
                backup.mark_completed();
                backup_ids.push(backup.id.clone());

                storage.upload(&backup).await.unwrap();
            }

            // Property 1: All backups should exist
            for backup_id in &backup_ids {
                let exists = storage.exists(backup_id).await.unwrap();
                prop_assert!(
                    exists,
                    "Backup {} should exist after upload",
                    backup_id
                );
            }

            // Property 2: List should return all backups
            let list = storage.list().await.unwrap();
            prop_assert_eq!(
                list.len(),
                backup_ids.len(),
                "List should return all {} uploaded backups",
                backup_ids.len()
            );

            for backup_id in &backup_ids {
                prop_assert!(
                    list.iter().any(|m| m.id == *backup_id),
                    "List should include backup {}",
                    backup_id
                );
            }

            // Property 3: Each backup can be downloaded independently
            for (i, backup_id) in backup_ids.iter().enumerate() {
                let downloaded = storage.download(backup_id).await.unwrap();
                let (expected_raft, expected_postgres) = &backups_data[i];

                prop_assert_eq!(
                    downloaded.raft_snapshot,
                    *expected_raft,
                    "Downloaded Raft snapshot should match original for backup {}",
                    backup_id
                );

                prop_assert_eq!(
                    downloaded.postgres_dump,
                    *expected_postgres,
                    "Downloaded PostgreSQL dump should match original for backup {}",
                    backup_id
                );
            }

            // Property 4: Delete one backup
            let deleted_id = &backup_ids[0];
            storage.delete(deleted_id).await.unwrap();

            // Property 5: Deleted backup should not exist
            let exists_deleted = storage.exists(deleted_id).await.unwrap();
            prop_assert!(
                !exists_deleted,
                "Deleted backup should not exist"
            );

            // Property 6: Other backups should still exist
            for backup_id in &backup_ids[1..] {
                let exists = storage.exists(backup_id).await.unwrap();
                prop_assert!(
                    exists,
                    "Other backups should still exist after deleting one"
                );
            }

            // Property 7: List should not include deleted backup
            let list_after_delete = storage.list().await.unwrap();
            prop_assert_eq!(
                list_after_delete.len(),
                backup_ids.len() - 1,
                "List should have one fewer backup after delete"
            );

            prop_assert!(
                !list_after_delete.iter().any(|m| m.id == *deleted_id),
                "List should not include deleted backup"
            );

            // Property 8: Other backups should still be downloadable
            for backup_id in &backup_ids[1..] {
                let download_result = storage.download(backup_id).await;
                prop_assert!(
                    download_result.is_ok(),
                    "Other backups should still be downloadable after deleting one"
                )?;
            }
            Ok(())
        });
    }

    /// **Property 30.2: Storage handles various data sizes**
    ///
    /// Storage should correctly handle backups of various sizes:
    /// 1. Small backups (< 1KB)
    /// 2. Medium backups (1KB - 100KB)
    /// 3. Large backups (> 100KB)
    ///
    /// **Validates: Requirements 2.5.4**
    #[test]
    fn prop_storage_handles_various_sizes(
        size_multiplier in 1u32..100u32
    ) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let temp_dir = TempDir::new().unwrap();
            let storage = LocalStorage::new(temp_dir.path()).unwrap();

            // Generate data of varying sizes
            let data_size = (size_multiplier * 1000) as usize;
            let raft_data = vec![0xABu8; data_size];
            let postgres_data = vec![0xCDu8; data_size];

            let mut backup = Backup::new(raft_data.clone(), postgres_data.clone());
            backup.mark_completed();
            let backup_id = backup.id.clone();

            // Property 1: Upload should succeed for any size
            let upload_result = storage.upload(&backup).await;
            prop_assert!(
                upload_result.is_ok(),
                "Upload should succeed for size {} bytes: {:?}",
                data_size * 2,
                upload_result.err()
            );

            // Property 2: Download should succeed and match original
            let downloaded = storage.download(&backup_id).await.unwrap();

            prop_assert_eq!(
                downloaded.raft_snapshot.len(),
                data_size,
                "Downloaded Raft snapshot size should match original"
            );

            prop_assert_eq!(
                downloaded.postgres_dump.len(),
                data_size,
                "Downloaded PostgreSQL dump size should match original"
            );

            prop_assert_eq!(
                downloaded.raft_snapshot,
                raft_data,
                "Downloaded Raft snapshot should match original for size {}",
                data_size
            );

            prop_assert_eq!(
                downloaded.postgres_dump,
                postgres_data,
                "Downloaded PostgreSQL dump should match original for size {}",
                data_size
            );

            // Property 3: Metadata should reflect correct sizes
            let metadata = storage.get_metadata(&backup_id).await.unwrap();

            prop_assert_eq!(
                metadata.raft_snapshot_size,
                data_size as u64,
                "Metadata Raft snapshot size should match actual size"
            );

            prop_assert_eq!(
                metadata.postgres_dump_size,
                data_size as u64,
                "Metadata PostgreSQL dump size should match actual size"
            );

            prop_assert_eq!(
                metadata.original_size_bytes,
                (data_size * 2) as u64,
                "Metadata total size should match sum of components"
            );

            // Cleanup
            storage.delete(&backup_id).await.unwrap();
            Ok(())
        });
    }

    /// **Property 30.3: Storage error handling**
    ///
    /// Storage should handle error conditions gracefully:
    /// 1. Download non-existent backup fails with NotFound error
    /// 2. Get metadata for non-existent backup fails
    /// 3. Delete non-existent backup succeeds (idempotent)
    /// 4. Exists returns false for non-existent backup
    ///
    /// **Validates: Requirements 2.5.4**
    #[test]
    fn prop_storage_error_handling(
        non_existent_id in "[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}"
    ) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let temp_dir = TempDir::new().unwrap();
            let storage = LocalStorage::new(temp_dir.path()).unwrap();

            // Property 1: Exists should return false for non-existent backup
            let exists = storage.exists(&non_existent_id).await.unwrap();
            prop_assert!(
                !exists,
                "Exists should return false for non-existent backup"
            );

            // Property 2: Download non-existent backup should fail
            let download_result = storage.download(&non_existent_id).await;
            prop_assert!(
                download_result.is_err(),
                "Download should fail for non-existent backup"
            );

            // Property 3: Error should be NotFound
            if let Err(e) = download_result {
                prop_assert!(
                    matches!(e, BackupError::NotFound(_)),
                    "Error should be NotFound, got: {:?}",
                    e
                );
            }

            // Property 4: Get metadata for non-existent backup should fail
            let metadata_result = storage.get_metadata(&non_existent_id).await;
            prop_assert!(
                metadata_result.is_err(),
                "Get metadata should fail for non-existent backup"
            );

            // Property 5: Error should be NotFound
            if let Err(e) = metadata_result {
                prop_assert!(
                    matches!(e, BackupError::NotFound(_)),
                    "Error should be NotFound, got: {:?}",
                    e
                );
            }

            // Property 6: Delete non-existent backup should succeed (idempotent)
            let delete_result = storage.delete(&non_existent_id).await;
            prop_assert!(
                delete_result.is_ok(),
                "Delete should succeed (idempotent) for non-existent backup"
            );

            // Property 7: List should be empty
            let list = storage.list().await.unwrap();
            prop_assert_eq!(
                list.len(),
                0,
                "List should be empty when no backups exist"
            )
        });
    }
}
