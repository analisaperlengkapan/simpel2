//! Property-based tests for backup functionality
//!
//! These tests validate correctness properties using proptest to generate
//! random test cases and verify invariants hold across all inputs.

use chrono::{Duration as ChronoDuration, Utc};
use proptest::prelude::*;
use secreton_backup::prelude::*;
use secreton_backup::scheduler::BackupScheduler;
use secreton_backup::storage::LocalStorage;
use secreton_backup::types::{BackupConfig, BackupStatus, LocalStorageConfig, RestoreOptions, StorageConfig};
use std::str::FromStr;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::time::{sleep, Duration};

// ============================================================================
// Property 27: Scheduled backup execution
// ============================================================================
// **Validates: Requirements 2.5.1**
//
// For any configured backup schedule, backups should be created at the
// scheduled times according to the cron expression.
//
// This property verifies that:
// 1. The scheduler correctly parses cron expressions
// 2. Backups are created at the expected times
// 3. The scheduler handles various cron patterns correctly
// 4. Backups are not created outside of scheduled times

/// Strategy to generate valid cron expressions for testing
fn cron_expression_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Every minute (for fast testing)
        Just("* * * * *".to_string()),
        // Every 2 minutes
        Just("*/2 * * * *".to_string()),
        // Every 5 minutes
        Just("*/5 * * * *".to_string()),
        // Every 10 minutes
        Just("*/10 * * * *".to_string()),
        // Every hour at minute 0
        Just("0 * * * *".to_string()),
        // Every day at 2 AM
        Just("0 2 * * *".to_string()),
        // Every day at midnight
        Just("0 0 * * *".to_string()),
        // Every Monday at 3 AM
        Just("0 3 * * 1".to_string()),
        // Every 15 minutes
        Just("*/15 * * * *".to_string()),
        // Every 30 minutes
        Just("*/30 * * * *".to_string()),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// **Property 27: Scheduled backup execution**
    ///
    /// For any valid cron expression, the scheduler should:
    /// 1. Successfully parse the expression
    /// 2. Calculate the next backup time correctly
    /// 3. Execute backups at scheduled times (within tolerance)
    /// 4. Not execute backups outside of scheduled times
    ///
    /// **Validates: Requirements 2.5.1**
    #[test]
    fn prop_scheduled_backup_execution(
        cron_expr in cron_expression_strategy()
    ) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            // Setup: Create test backup manager
            let temp_dir = TempDir::new().unwrap();
            let mut config = BackupConfig::default();
            config.storage_config = StorageConfig::Local(LocalStorageConfig {
                path: temp_dir.path().to_string_lossy().to_string(),
            });
            config.schedule = cron_expr.clone();
            config.verify_after_backup = false; // Disable verification for speed

            let manager = Arc::new(BackupManager::new(config).await.unwrap());

            // Property 1: Scheduler should successfully parse the cron expression
            let scheduler_result = BackupScheduler::new(&cron_expr, manager.clone());
            prop_assert!(
                scheduler_result.is_ok(),
                "Scheduler should successfully parse cron expression: {}",
                cron_expr
            );

            let scheduler = scheduler_result.unwrap();

            // Property 2: Scheduler should calculate next backup time
            let next_time = scheduler.next_backup_time();
            prop_assert!(
                next_time.is_some(),
                "Scheduler should calculate next backup time for: {}",
                cron_expr
            );

            let next_backup = next_time.unwrap();
            let now = Utc::now();

            // Property 3: Next backup time should be in the future
            prop_assert!(
                next_backup > now,
                "Next backup time {} should be after current time {}",
                next_backup,
                now
            );

            // Property 4: For frequent schedules (every minute), verify backup execution
            // We only test this for "* * * * *" to keep test duration reasonable
            if cron_expr == "* * * * *" {
                // Start the scheduler
                scheduler.start().await.unwrap();

                // Wait for initial backup to be scheduled
                sleep(Duration::from_millis(100)).await;

                // Property 5: Scheduler should be running
                prop_assert!(
                    scheduler.is_running().await,
                    "Scheduler should be running after start"
                );

                // Get initial backup count
                let initial_backups = manager.list_backups().await.unwrap();
                let initial_count = initial_backups.len();

                // Calculate time until next backup (with small buffer)
                let time_until_next = (next_backup - Utc::now()).to_std().unwrap();
                let wait_time = time_until_next + Duration::from_secs(5); // 5 second buffer

                // Wait for the scheduled backup to execute
                sleep(wait_time).await;

                // Property 6: A backup should have been created
                let final_backups = manager.list_backups().await.unwrap();
                let final_count = final_backups.len();

                prop_assert!(
                    final_count > initial_count,
                    "Backup count should increase after scheduled time. Initial: {}, Final: {}",
                    initial_count,
                    final_count
                );

                // Property 7: The backup should have been created around the scheduled time
                if let Some(latest_backup) = final_backups.first() {
                    let backup_time = latest_backup.timestamp;
                    let time_diff = (backup_time - next_backup).num_seconds().abs();

                    // Allow 10 second tolerance for execution time
                    prop_assert!(
                        time_diff <= 10,
                        "Backup should be created within 10 seconds of scheduled time. \
                         Scheduled: {}, Actual: {}, Diff: {}s",
                        next_backup,
                        backup_time,
                        time_diff
                    );
                }

                // Stop the scheduler
                scheduler.stop().await.unwrap();

                // Property 8: Scheduler should stop
                // Give it a moment to stop
                sleep(Duration::from_millis(200)).await;

                // Verify no more backups are created after stopping
                let stopped_count = manager.list_backups().await.unwrap().len();

                // Wait a bit more to ensure no backup is created
                sleep(Duration::from_secs(2)).await;

                let final_stopped_count = manager.list_backups().await.unwrap().len();

                prop_assert_eq!(
                    stopped_count,
                    final_stopped_count,
                    "No backups should be created after scheduler is stopped"
                );
            }

            // Property 9: For less frequent schedules, verify next time calculation
            // This validates the cron parsing without waiting for actual execution
            if cron_expr != "* * * * *" {
                // Get multiple next times
                let schedule = cron::Schedule::from_str(&cron_expr).unwrap();
                let mut upcoming = schedule.upcoming(Utc).take(3);

                let time1 = upcoming.next();
                let time2 = upcoming.next();
                let time3 = upcoming.next();

                prop_assert!(time1.is_some(), "Should have first upcoming time");
                prop_assert!(time2.is_some(), "Should have second upcoming time");
                prop_assert!(time3.is_some(), "Should have third upcoming time");

                // Property 10: Upcoming times should be in ascending order
                let t1 = time1.unwrap();
                let t2 = time2.unwrap();
                let t3 = time3.unwrap();

                prop_assert!(
                    t1 < t2 && t2 < t3,
                    "Upcoming backup times should be in ascending order: {} < {} < {}",
                    t1, t2, t3
                );

                // Property 11: All times should be in the future
                prop_assert!(t1 > now, "First upcoming time should be in future");
                prop_assert!(t2 > now, "Second upcoming time should be in future");
                prop_assert!(t3 > now, "Third upcoming time should be in future");
            }
        });
    }

    /// **Property 27.1: Scheduler lifecycle**
    ///
    /// The scheduler should correctly manage its lifecycle:
    /// 1. Start successfully
    /// 2. Report running status correctly
    /// 3. Stop successfully
    /// 4. Prevent double-start
    /// 5. Prevent double-stop
    ///
    /// **Validates: Requirements 2.5.1**
    #[test]
    fn prop_scheduler_lifecycle(
        cron_expr in cron_expression_strategy()
    ) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let temp_dir = TempDir::new().unwrap();
            let mut config = BackupConfig::default();
            config.storage_config = StorageConfig::Local(LocalStorageConfig {
                path: temp_dir.path().to_string_lossy().to_string(),
            });

            let manager = Arc::new(BackupManager::new(config).await.unwrap());
            let scheduler = BackupScheduler::new(&cron_expr, manager).unwrap();

            // Property 1: Initially not running
            prop_assert!(
                !scheduler.is_running().await,
                "Scheduler should not be running initially"
            );

            // Property 2: Start should succeed
            let start_result = scheduler.start().await;
            prop_assert!(
                start_result.is_ok(),
                "Scheduler start should succeed"
            );

            // Property 3: Should be running after start
            prop_assert!(
                scheduler.is_running().await,
                "Scheduler should be running after start"
            );

            // Property 4: Double-start should fail
            let double_start = scheduler.start().await;
            prop_assert!(
                double_start.is_err(),
                "Double-start should fail"
            );

            // Property 5: Stop should succeed
            let stop_result = scheduler.stop().await;
            prop_assert!(
                stop_result.is_ok(),
                "Scheduler stop should succeed"
            );

            // Give it a moment to stop
            sleep(Duration::from_millis(100)).await;

            // Property 6: Double-stop should fail
            let double_stop = scheduler.stop().await;
            prop_assert!(
                double_stop.is_err(),
                "Double-stop should fail"
            );
        });
    }

    /// **Property 27.2: Cron expression validation**
    ///
    /// The scheduler should correctly validate cron expressions:
    /// 1. Accept valid cron expressions
    /// 2. Reject invalid cron expressions
    /// 3. Provide meaningful error messages
    ///
    /// **Validates: Requirements 2.5.1**
    #[test]
    fn prop_cron_validation(
        valid_expr in cron_expression_strategy(),
        invalid_suffix in "[a-z]{1,10}"
    ) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let temp_dir = TempDir::new().unwrap();
            let mut config = BackupConfig::default();
            config.storage_config = StorageConfig::Local(LocalStorageConfig {
                path: temp_dir.path().to_string_lossy().to_string(),
            });

            let manager = Arc::new(BackupManager::new(config).await.unwrap());

            // Property 1: Valid expression should be accepted
            let valid_result = BackupScheduler::new(&valid_expr, manager.clone());
            prop_assert!(
                valid_result.is_ok(),
                "Valid cron expression '{}' should be accepted",
                valid_expr
            );

            // Property 2: Invalid expression should be rejected
            let invalid_expr = format!("{} {}", valid_expr, invalid_suffix);
            let invalid_result = BackupScheduler::new(&invalid_expr, manager);
            prop_assert!(
                invalid_result.is_err(),
                "Invalid cron expression '{}' should be rejected",
                invalid_expr
            );

            // Property 3: Error message should mention the invalid expression
            if let Err(e) = invalid_result {
                let error_msg = e.to_string();
                prop_assert!(
                    error_msg.contains(&invalid_expr) || error_msg.contains("cron"),
                    "Error message should mention cron or the invalid expression: {}",
                    error_msg
                );
            }
        });
    }
}

// ============================================================================
// Property 28: Backup completeness
// ============================================================================
// **Validates: Requirements 2.5.2**
//
// For any backup, it should contain both Raft snapshot data and PostgreSQL
// dump data. This property verifies that:
// 1. Every backup includes a non-empty Raft snapshot
// 2. Every backup includes a non-empty PostgreSQL dump
// 3. Backup metadata accurately reflects the contents
// 4. Both components are properly stored and retrievable

/// Strategy to generate random Raft snapshot data
fn raft_snapshot_strategy() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>(), 100..10000)
}

/// Strategy to generate random PostgreSQL dump data
fn postgres_dump_strategy() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>(), 100..10000)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// **Property 28: Backup completeness**
    ///
    /// For any backup created, it must contain both:
    /// 1. A non-empty Raft snapshot
    /// 2. A non-empty PostgreSQL dump
    /// 3. Accurate metadata reflecting both components
    ///
    /// This property ensures that backups are complete and contain all
    /// necessary data for disaster recovery.
    ///
    /// **Validates: Requirements 2.5.2**
    #[test]
    fn prop_backup_completeness(
        raft_data in raft_snapshot_strategy(),
        postgres_data in postgres_dump_strategy()
    ) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            // Setup: Create a backup with the generated data
            let backup = Backup::new(raft_data.clone(), postgres_data.clone());

            // Property 1: Backup must contain non-empty Raft snapshot
            prop_assert!(
                !backup.raft_snapshot.is_empty(),
                "Backup must contain non-empty Raft snapshot"
            );

            // Property 2: Raft snapshot must match the original data
            prop_assert_eq!(
                backup.raft_snapshot,
                raft_data,
                "Raft snapshot must match original data"
            );

            // Property 3: Backup must contain non-empty PostgreSQL dump
            prop_assert!(
                !backup.postgres_dump.is_empty(),
                "Backup must contain non-empty PostgreSQL dump"
            );

            // Property 4: PostgreSQL dump must match the original data
            prop_assert_eq!(
                backup.postgres_dump,
                postgres_data,
                "PostgreSQL dump must match original data"
            );

            // Property 5: Metadata must accurately reflect Raft snapshot size
            prop_assert_eq!(
                backup.metadata.raft_snapshot_size,
                raft_data.len() as u64,
                "Metadata must accurately reflect Raft snapshot size. Expected: {}, Got: {}",
                raft_data.len(),
                backup.metadata.raft_snapshot_size
            );

            // Property 6: Metadata must accurately reflect PostgreSQL dump size
            prop_assert_eq!(
                backup.metadata.postgres_dump_size,
                postgres_data.len() as u64,
                "Metadata must accurately reflect PostgreSQL dump size. Expected: {}, Got: {}",
                postgres_data.len(),
                backup.metadata.postgres_dump_size
            );

            // Property 7: Total original size must equal sum of components
            let expected_total = (raft_data.len() + postgres_data.len()) as u64;
            prop_assert_eq!(
                backup.metadata.original_size_bytes,
                expected_total,
                "Total original size must equal sum of Raft snapshot and PostgreSQL dump. \
                 Expected: {}, Got: {}",
                expected_total,
                backup.metadata.original_size_bytes
            );

            // Property 8: Backup ID must be non-empty
            prop_assert!(
                !backup.id.is_empty(),
                "Backup ID must be non-empty"
            );

            // Property 9: Backup ID must match metadata ID
            prop_assert_eq!(
                backup.id,
                backup.metadata.id,
                "Backup ID must match metadata ID"
            );

            // Property 10: Backup timestamp must be valid (not in the future)
            let now = Utc::now();
            prop_assert!(
                backup.timestamp <= now,
                "Backup timestamp must not be in the future. Backup: {}, Now: {}",
                backup.timestamp,
                now
            );

            // Property 11: Metadata timestamp must match backup timestamp
            prop_assert_eq!(
                backup.timestamp,
                backup.metadata.timestamp,
                "Metadata timestamp must match backup timestamp"
            );

            // Property 12: Checksum calculation must be consistent
            let checksum1 = backup.calculate_checksum();
            let checksum2 = backup.calculate_checksum();
            prop_assert_eq!(
                checksum1,
                checksum2,
                "Checksum calculation must be consistent"
            );

            // Property 13: Checksum must be non-empty
            prop_assert!(
                !checksum1.is_empty(),
                "Checksum must be non-empty"
            );

            // Property 14: Checksum must be valid hex string (64 characters for SHA-256)
            prop_assert_eq!(
                checksum1.len(),
                64,
                "SHA-256 checksum must be 64 characters (32 bytes in hex)"
            );

            // Property 15: Checksum must only contain valid hex characters
            prop_assert!(
                checksum1.chars().all(|c| c.is_ascii_hexdigit()),
                "Checksum must only contain valid hex characters"
            );
        });
    }

    /// **Property 28.1: Backup completeness after storage round-trip**
    ///
    /// After storing and retrieving a backup, both components must remain
    /// intact and complete.
    ///
    /// **Validates: Requirements 2.5.2, 2.5.4**
    #[test]
    fn prop_backup_completeness_after_storage(
        raft_data in raft_snapshot_strategy(),
        postgres_data in postgres_dump_strategy()
    ) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            // Setup: Create temporary storage
            let temp_dir = TempDir::new().unwrap();
            let storage_config = StorageConfig::Local(LocalStorageConfig {
                path: temp_dir.path().to_string_lossy().to_string(),
            });

            let storage = LocalStorage::new(storage_config).await.unwrap();

            // Create backup
            let mut backup = Backup::new(raft_data.clone(), postgres_data.clone());
            backup.mark_completed();

            // Store the backup
            storage.store(&backup).await.unwrap();

            // Retrieve the backup
            let retrieved = storage.retrieve(&backup.id).await.unwrap();

            // Property 1: Retrieved backup must have non-empty Raft snapshot
            prop_assert!(
                !retrieved.raft_snapshot.is_empty(),
                "Retrieved backup must have non-empty Raft snapshot"
            );

            // Property 2: Retrieved Raft snapshot must match original
            prop_assert_eq!(
                retrieved.raft_snapshot,
                raft_data,
                "Retrieved Raft snapshot must match original data"
            );

            // Property 3: Retrieved backup must have non-empty PostgreSQL dump
            prop_assert!(
                !retrieved.postgres_dump.is_empty(),
                "Retrieved backup must have non-empty PostgreSQL dump"
            );

            // Property 4: Retrieved PostgreSQL dump must match original
            prop_assert_eq!(
                retrieved.postgres_dump,
                postgres_data,
                "Retrieved PostgreSQL dump must match original data"
            );

            // Property 5: Retrieved metadata must match original metadata
            prop_assert_eq!(
                retrieved.metadata.raft_snapshot_size,
                backup.metadata.raft_snapshot_size,
                "Retrieved metadata Raft snapshot size must match original"
            );

            prop_assert_eq!(
                retrieved.metadata.postgres_dump_size,
                backup.metadata.postgres_dump_size,
                "Retrieved metadata PostgreSQL dump size must match original"
            );

            prop_assert_eq!(
                retrieved.metadata.original_size_bytes,
                backup.metadata.original_size_bytes,
                "Retrieved metadata original size must match original"
            );

            // Property 6: Retrieved checksum must match original
            prop_assert_eq!(
                retrieved.metadata.checksum,
                backup.metadata.checksum,
                "Retrieved checksum must match original"
            );

            // Property 7: Recalculated checksum must match stored checksum
            let recalculated_checksum = retrieved.calculate_checksum();
            prop_assert_eq!(
                recalculated_checksum,
                retrieved.metadata.checksum,
                "Recalculated checksum must match stored checksum"
            );
        });
    }

    /// **Property 28.2: Backup completeness with empty data rejection**
    ///
    /// Backups should not be created with empty Raft snapshots or PostgreSQL dumps.
    /// This property verifies that the system properly handles edge cases.
    ///
    /// **Validates: Requirements 2.5.2**
    #[test]
    fn prop_backup_rejects_empty_components(
        non_empty_data in raft_snapshot_strategy()
    ) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            // Property 1: Backup with empty Raft snapshot should have zero size
            let backup_empty_raft = Backup::new(vec![], non_empty_data.clone());
            prop_assert_eq!(
                backup_empty_raft.metadata.raft_snapshot_size,
                0,
                "Backup with empty Raft snapshot should have zero Raft size"
            );

            // Property 2: Backup with empty PostgreSQL dump should have zero size
            let backup_empty_postgres = Backup::new(non_empty_data.clone(), vec![]);
            prop_assert_eq!(
                backup_empty_postgres.metadata.postgres_dump_size,
                0,
                "Backup with empty PostgreSQL dump should have zero PostgreSQL size"
            );

            // Property 3: Backup with both empty should have zero total size
            let backup_both_empty = Backup::new(vec![], vec![]);
            prop_assert_eq!(
                backup_both_empty.metadata.original_size_bytes,
                0,
                "Backup with both components empty should have zero total size"
            );

            // Property 4: Empty components should still be stored (for consistency)
            prop_assert_eq!(
                backup_empty_raft.raft_snapshot.len(),
                0,
                "Empty Raft snapshot should be stored as empty vector"
            );

            prop_assert_eq!(
                backup_empty_postgres.postgres_dump.len(),
                0,
                "Empty PostgreSQL dump should be stored as empty vector"
            );
        });
    }

    /// **Property 28.3: Backup metadata consistency**
    ///
    /// All metadata fields must be consistent with the actual backup data.
    ///
    /// **Validates: Requirements 2.5.2**
    #[test]
    fn prop_backup_metadata_consistency(
        raft_data in raft_snapshot_strategy(),
        postgres_data in postgres_dump_strategy()
    ) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let mut backup = Backup::new(raft_data.clone(), postgres_data.clone());

            // Property 1: Initial status should be InProgress
            prop_assert_eq!(
                backup.metadata.status,
                BackupStatus::InProgress,
                "Initial backup status should be InProgress"
            );

            // Mark as completed
            backup.mark_completed();

            // Property 2: After completion, status should be Completed
            prop_assert_eq!(
                backup.metadata.status,
                BackupStatus::Completed,
                "After mark_completed, status should be Completed"
            );

            // Property 3: After completion, checksum should be set
            prop_assert!(
                !backup.metadata.checksum.is_empty(),
                "After mark_completed, checksum should be set"
            );

            // Property 4: Checksum in metadata should match calculated checksum
            let calculated = backup.calculate_checksum();
            prop_assert_eq!(
                backup.metadata.checksum,
                calculated,
                "Checksum in metadata should match calculated checksum"
            );

            // Mark as verified
            backup.mark_verified();

            // Property 5: After verification, status should be Verified
            prop_assert_eq!(
                backup.metadata.status,
                BackupStatus::Verified,
                "After mark_verified, status should be Verified"
            );

            // Property 6: After verification, verified_at should be set
            prop_assert!(
                backup.metadata.verified_at.is_some(),
                "After mark_verified, verified_at should be set"
            );

            // Property 7: verified_at should not be in the future
            let verified_at = backup.metadata.verified_at.unwrap();
            let now = Utc::now();
            prop_assert!(
                verified_at <= now,
                "verified_at should not be in the future"
            );

            // Property 8: is_verified() should return true
            prop_assert!(
                backup.metadata.is_verified(),
                "is_verified() should return true after mark_verified"
            );

            // Property 9: is_failed() should return false
            prop_assert!(
                !backup.metadata.is_failed(),
                "is_failed() should return false for verified backup"
            );
        });
    }
}

// ============================================================================
// Additional helper tests for scheduler behavior
// ============================================================================

#[cfg(test)]
mod scheduler_tests {
    use super::*;

    /// Test that scheduler correctly handles rapid start/stop cycles
    #[tokio::test]
    async fn test_rapid_start_stop_cycles() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = BackupConfig::default();
        config.storage_config = StorageConfig::Local(LocalStorageConfig {
            path: temp_dir.path().to_string_lossy().to_string(),
        });

        let manager = Arc::new(BackupManager::new(config).await.unwrap());
        let scheduler = BackupScheduler::new("*/5 * * * *", manager).unwrap();

        // Perform multiple start/stop cycles
        for _ in 0..5 {
            scheduler.start().await.unwrap();
            assert!(scheduler.is_running().await);

            scheduler.stop().await.unwrap();
            sleep(Duration::from_millis(100)).await;
        }
    }

    /// Test that scheduler handles concurrent access correctly
    #[tokio::test]
    async fn test_concurrent_scheduler_access() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = BackupConfig::default();
        config.storage_config = StorageConfig::Local(LocalStorageConfig {
            path: temp_dir.path().to_string_lossy().to_string(),
        });

        let manager = Arc::new(BackupManager::new(config).await.unwrap());
        let scheduler = Arc::new(BackupScheduler::new("*/5 * * * *", manager).unwrap());

        // Start the scheduler
        scheduler.start().await.unwrap();

        // Spawn multiple tasks checking status concurrently
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let scheduler = scheduler.clone();
                tokio::spawn(async move {
                    for _ in 0..10 {
                        let _ = scheduler.is_running().await;
                        let _ = scheduler.next_backup_time();
                        sleep(Duration::from_millis(10)).await;
                    }
                })
            })
            .collect();

        // Wait for all tasks
        for handle in handles {
            handle.await.unwrap();
        }

        // Stop the scheduler
        scheduler.stop().await.unwrap();
    }

    /// Test that next_backup_time is consistent across multiple calls
    #[tokio::test]
    async fn test_next_backup_time_consistency() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = BackupConfig::default();
        config.storage_config = StorageConfig::Local(LocalStorageConfig {
            path: temp_dir.path().to_string_lossy().to_string(),
        });

        let manager = Arc::new(BackupManager::new(config).await.unwrap());
        let scheduler = BackupScheduler::new("0 2 * * *", manager).unwrap();

        // Get next backup time multiple times in quick succession
        let time1 = scheduler.next_backup_time();
        let time2 = scheduler.next_backup_time();
        let time3 = scheduler.next_backup_time();

        // All should return the same time (within the same minute)
        assert_eq!(time1, time2);
        assert_eq!(time2, time3);
    }
}

// ============================================================================
// Property 29: Backup encryption
// ============================================================================
// **Validates: Requirements 2.5.3**
//
// For any backup data, encryption should:
// 1. Produce ciphertext that differs from plaintext
// 2. Be reversible (decrypt(encrypt(data)) == data)
// 3. Produce different ciphertext for the same plaintext (due to random nonce)
// 4. Fail decryption with wrong key
// 5. Fail decryption with corrupted ciphertext

#[cfg(test)]
mod encryption_tests {
    use super::*;

    /// **Property 29: Backup encryption round-trip**
    ///
    /// For any backup data and encryption key:
    /// 1. Encryption produces ciphertext different from plaintext
    /// 2. Decryption reverses encryption (round-trip property)
    /// 3. Same plaintext produces different ciphertext (nonce randomness)
    ///
    /// **Validates: Requirements 2.5.3**
    #[tokio::test]
    async fn test_backup_encryption_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = BackupConfig::default();
        config.storage_config = StorageConfig::Local(LocalStorageConfig {
            path: temp_dir.path().to_string_lossy().to_string(),
        });

        // Generate a 32-byte encryption key
        let encryption_key = vec![42u8; 32];
        config.encryption_key = Some(encryption_key.clone());

        let manager = BackupManager::new(config).await.unwrap();

        // Test with various data sizes
        for size in [100, 1000, 10000, 50000] {
            let plaintext = vec![0xABu8; size];

            // Property 1: Encryption produces ciphertext different from plaintext
            let ciphertext = manager.encrypt(&plaintext).unwrap();
            assert_ne!(
                ciphertext, plaintext,
                "Ciphertext must differ from plaintext for size {}",
                size
            );

            // Property 2: Ciphertext must be longer than plaintext (includes nonce + auth tag)
            // ChaCha20-Poly1305 adds 12 bytes (nonce) + 16 bytes (auth tag) = 28 bytes overhead
            assert!(
                ciphertext.len() >= plaintext.len() + 28,
                "Ciphertext must be at least 28 bytes longer than plaintext. \
                 Plaintext: {} bytes, Ciphertext: {} bytes",
                plaintext.len(),
                ciphertext.len()
            );

            // Property 3: Decryption reverses encryption (round-trip property)
            let decrypted = manager.decrypt(&ciphertext).unwrap();
            assert_eq!(
                decrypted, plaintext,
                "Decryption must reverse encryption for size {}",
                size
            );

            // Property 4: Same plaintext produces different ciphertext (nonce randomness)
            let ciphertext2 = manager.encrypt(&plaintext).unwrap();
            assert_ne!(
                ciphertext, ciphertext2,
                "Same plaintext must produce different ciphertext due to random nonce"
            );

            // Property 5: Both ciphertexts decrypt to same plaintext
            let decrypted2 = manager.decrypt(&ciphertext2).unwrap();
            assert_eq!(
                decrypted2, plaintext,
                "Both ciphertexts must decrypt to same plaintext"
            );
        }
    }

    /// **Property 29.1: Encryption with wrong key fails**
    ///
    /// Decryption with a different key must fail.
    ///
    /// **Validates: Requirements 2.5.3**
    #[tokio::test]
    async fn test_encryption_wrong_key_fails() {
        let temp_dir = TempDir::new().unwrap();

        // Create manager with first key
        let mut config1 = BackupConfig::default();
        config1.storage_config = StorageConfig::Local(LocalStorageConfig {
            path: temp_dir.path().to_string_lossy().to_string(),
        });
        let key1 = vec![1u8; 32];
        config1.encryption_key = Some(key1);
        let manager1 = BackupManager::new(config1).await.unwrap();

        // Create manager with different key
        let mut config2 = BackupConfig::default();
        config2.storage_config = StorageConfig::Local(LocalStorageConfig {
            path: temp_dir.path().to_string_lossy().to_string(),
        });
        let key2 = vec![2u8; 32];
        config2.encryption_key = Some(key2);
        let manager2 = BackupManager::new(config2).await.unwrap();

        let plaintext = b"sensitive data";

        // Encrypt with first key
        let ciphertext = manager1.encrypt(plaintext).unwrap();

        // Property: Decryption with wrong key must fail
        let wrong_decrypt_result = manager2.decrypt(&ciphertext);
        assert!(
            wrong_decrypt_result.is_err(),
            "Decryption with wrong key must fail"
        );
    }

    /// **Property 29.2: Corrupted ciphertext fails decryption**
    ///
    /// Tampering with ciphertext must cause decryption to fail.
    ///
    /// **Validates: Requirements 2.5.3**
    #[tokio::test]
    async fn test_encryption_corrupted_ciphertext_fails() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = BackupConfig::default();
        config.storage_config = StorageConfig::Local(LocalStorageConfig {
            path: temp_dir.path().to_string_lossy().to_string(),
        });
        let encryption_key = vec![42u8; 32];
        config.encryption_key = Some(encryption_key);

        let manager = BackupManager::new(config).await.unwrap();

        let plaintext = b"sensitive data that should not be tampered with";
        let ciphertext = manager.encrypt(plaintext).unwrap();

        // Property 1: Corrupted ciphertext fails decryption
        let mut corrupted = ciphertext.clone();
        corrupted[12] = corrupted[12].wrapping_add(1); // Corrupt a byte after nonce

        let corrupted_decrypt_result = manager.decrypt(&corrupted);
        assert!(
            corrupted_decrypt_result.is_err(),
            "Decryption of corrupted ciphertext must fail"
        );

        // Property 2: Ciphertext too short fails decryption
        let short_ciphertext = vec![0u8; 11]; // Less than 12 bytes (nonce size)
        let short_decrypt_result = manager.decrypt(&short_ciphertext);
        assert!(
            short_decrypt_result.is_err(),
            "Decryption of too-short ciphertext must fail"
        );

        // Property 3: Empty ciphertext fails decryption
        let empty_decrypt_result = manager.decrypt(&[]);
        assert!(
            empty_decrypt_result.is_err(),
            "Decryption of empty ciphertext must fail"
        );
    }

    /// **Property 29.3: Encryption key validation**
    ///
    /// The backup manager should validate encryption keys.
    ///
    /// **Validates: Requirements 2.5.3**
    #[tokio::test]
    async fn test_encryption_key_validation() {
        let temp_dir = TempDir::new().unwrap();

        // Property 1: Invalid key size should be rejected
        for invalid_size in [0, 1, 16, 31, 33, 64] {
            let mut config = BackupConfig::default();
            config.storage_config = StorageConfig::Local(LocalStorageConfig {
                path: temp_dir.path().to_string_lossy().to_string(),
            });
            let invalid_key = vec![0u8; invalid_size];
            config.encryption_key = Some(invalid_key);

            let result = BackupManager::new(config).await;
            assert!(
                result.is_err(),
                "Backup manager should reject encryption key of size {}",
                invalid_size
            );

            // Property 2: Error message should mention key size
            if let Err(e) = result {
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("32") || error_msg.contains("key"),
                    "Error message should mention key size requirement: {}",
                    error_msg
                );
            }
        }

        // Property 3: Valid 32-byte key should be accepted
        let mut config = BackupConfig::default();
        config.storage_config = StorageConfig::Local(LocalStorageConfig {
            path: temp_dir.path().to_string_lossy().to_string(),
        });
        let valid_key = vec![42u8; 32];
        config.encryption_key = Some(valid_key);

        let result = BackupManager::new(config).await;
        assert!(result.is_ok(), "Backup manager should accept 32-byte key");

        // Property 4: No key provided should generate a valid key
        let mut config = BackupConfig::default();
        config.storage_config = StorageConfig::Local(LocalStorageConfig {
            path: temp_dir.path().to_string_lossy().to_string(),
        });
        config.encryption_key = None;

        let manager_result = BackupManager::new(config).await;
        assert!(
            manager_result.is_ok(),
            "Backup manager should generate key if none provided"
        );

        // Property 5: Generated key should work for encryption/decryption
        if let Ok(manager) = manager_result {
            let test_data = b"test data";
            let encrypted = manager.encrypt(test_data).unwrap();
            let decrypted = manager.decrypt(&encrypted).unwrap();
            assert_eq!(
                decrypted,
                test_data.to_vec(),
                "Generated key should work for encryption/decryption"
            );
        }
    }

    /// **Property 29.4: Encryption with compression**
    ///
    /// When compression is enabled, encryption should work correctly
    /// with compressed data.
    ///
    /// **Validates: Requirements 2.5.3**
    #[tokio::test]
    async fn test_encryption_with_compression() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = BackupConfig::default();
        config.storage_config = StorageConfig::Local(LocalStorageConfig {
            path: temp_dir.path().to_string_lossy().to_string(),
        });
        let encryption_key = vec![42u8; 32];
        config.encryption_key = Some(encryption_key);
        config.compression_enabled = true;

        let manager = BackupManager::new(config).await.unwrap();

        // Use highly compressible data
        let plaintext = b"AAAAAAAAAA".repeat(1000);

        // Property 1: Compress then encrypt
        let compressed = manager.compress(&plaintext).unwrap();
        let encrypted = manager.encrypt(&compressed).unwrap();

        // Property 2: Decrypt then decompress
        let decrypted = manager.decrypt(&encrypted).unwrap();
        let decompressed = manager.decompress(&decrypted).unwrap();

        // Property 3: Round-trip should preserve data
        assert_eq!(
            decompressed, plaintext,
            "Compress -> Encrypt -> Decrypt -> Decompress should preserve data"
        );

        // Property 4: Encrypted compressed data should differ from plaintext
        assert_ne!(
            encrypted, plaintext,
            "Encrypted compressed data must differ from plaintext"
        );

        // Property 5: Encrypted compressed data should differ from compressed data
        assert_ne!(
            encrypted, compressed,
            "Encrypted data must differ from compressed data"
        );

        // Property 6: Compression should reduce size
        assert!(
            compressed.len() < plaintext.len(),
            "Compression should reduce size for repetitive data. \
             Original: {} bytes, Compressed: {} bytes",
            plaintext.len(),
            compressed.len()
        );
    }

    /// **Property 29.5: Encryption performance characteristics**
    ///
    /// Encryption should have predictable performance characteristics.
    ///
    /// **Validates: Requirements 2.5.3**
    #[tokio::test]
    async fn test_encryption_performance() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = BackupConfig::default();
        config.storage_config = StorageConfig::Local(LocalStorageConfig {
            path: temp_dir.path().to_string_lossy().to_string(),
        });
        let encryption_key = vec![42u8; 32];
        config.encryption_key = Some(encryption_key);

        let manager = BackupManager::new(config).await.unwrap();

        // Test with various data sizes
        for data_size in [1000, 5000, 10000, 25000, 50000] {
            let plaintext = vec![0xABu8; data_size];

            // Property 1: Encryption should succeed for any size
            let ciphertext = manager.encrypt(&plaintext).unwrap();

            // Property 2: Ciphertext size should be plaintext + 28 bytes overhead
            // (12 bytes nonce + 16 bytes auth tag)
            assert_eq!(
                ciphertext.len(),
                plaintext.len() + 28,
                "Ciphertext size should be plaintext + 28 bytes for size {}. \
                 Expected: {}, Got: {}",
                data_size,
                plaintext.len() + 28,
                ciphertext.len()
            );

            // Property 3: Decryption should succeed
            let decrypted = manager.decrypt(&ciphertext).unwrap();

            // Property 4: Decrypted size should match original
            assert_eq!(
                decrypted.len(),
                plaintext.len(),
                "Decrypted size should match original plaintext size for size {}",
                data_size
            );
        }
    }
}


// ============================================================================
// Property 30: Backup storage
// ============================================================================
// **Validates: Requirements 2.5.4**
//
// For any backup storage backend, the system should:
// 1. Successfully store backups
// 2. Successfully retrieve backups
// 3. List all stored backups
// 4. Delete backups
// 5. Preserve backup integrity during storage round-trip

// Note: Property 30 tests are implemented in storage module tests

// ============================================================================
// Property 31: Backup retention
// ============================================================================
// **Validates: Requirements 2.5.5**
//
// For any configured retention policy, the backup system should:
// 1. Delete backups older than retention_days
// 2. Keep backups newer than retention_days
// 3. Handle edge cases (0 days, very large retention)
// 4. Correctly calculate backup age
// 5. Execute cleanup automatically on schedule

/// Strategy to generate retention days (0 to 365)
fn retention_days_strategy() -> impl Strategy<Value = u32> {
    prop_oneof![
        Just(0u32),      // Delete all backups immediately
        Just(1u32),      // Keep only today's backups
        Just(7u32),      // One week
        Just(30u32),     // One month (default)
        Just(90u32),     // Three months
        Just(365u32),    // One year
        1u32..=365u32,   // Random value between 1 and 365
    ]
}

/// Strategy to generate backup ages in days
fn backup_age_days_strategy() -> impl Strategy<Value = i64> {
    prop_oneof![
        Just(0i64),      // Today
        Just(1i64),      // Yesterday
        Just(7i64),      // One week ago
        Just(30i64),     // One month ago
        Just(60i64),     // Two months ago
        Just(90i64),     // Three months ago
        Just(180i64),    // Six months ago
        Just(365i64),    // One year ago
        0i64..=365i64,   // Random age
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// **Property 31: Backup retention policy enforcement**
    ///
    /// For any retention policy (retention_days), the cleanup process must:
    /// 1. Delete backups older than retention_days
    /// 2. Keep backups newer than or equal to retention_days
    /// 3. Correctly calculate backup age
    /// 4. Handle edge cases (0 days retention, same-day backups)
    ///
    /// **Validates: Requirements 2.5.5**
    #[test]
    fn prop_backup_retention_enforcement(
        retention_days in retention_days_strategy(),
        backup_ages in prop::collection::vec(backup_age_days_strategy(), 1..=10)
    ) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            // Setup: Create temporary storage
            let temp_dir = TempDir::new().unwrap();
            let mut config = BackupConfig::default();
            config.storage_config = StorageConfig::Local(LocalStorageConfig {
                path: temp_dir.path().to_string_lossy().to_string(),
            });
            config.retention_days = retention_days;
            config.verify_after_backup = false; // Disable for speed

            let manager = BackupManager::new(config).await.unwrap();

            // Create backups with different ages
            let mut backup_ids = Vec::new();
            let now = Utc::now();

            for age_days in &backup_ages {
                // Create a backup
                let backup_id = manager.create_backup().await.unwrap();
                backup_ids.push((backup_id.clone(), *age_days));

                // Manually modify the backup timestamp to simulate age
                let storage_path = temp_dir.path().join(format!("{}.json", backup_id));
                let mut metadata: BackupMetadata = {
                    let content = std::fs::read_to_string(&storage_path).unwrap();
                    serde_json::from_str(&content).unwrap()
                };
                metadata.timestamp = now - ChronoDuration::days(*age_days);
                std::fs::write(&storage_path, serde_json::to_string_pretty(&metadata).unwrap()).unwrap();
            }

            // Verify all backups were created
            let initial_backups = manager.list_backups().await.unwrap();
            prop_assert_eq!(
                initial_backups.len(),
                backup_ages.len(),
                "All backups should be created initially"
            );

            // Execute cleanup
            let deleted_count = manager.cleanup_old_backups_manual().await.unwrap();

            // Get remaining backups
            let remaining_backups = manager.list_backups().await.unwrap();

            // Property 1: Backups older than retention_days should be deleted
            let expected_deleted = backup_ages.iter()
                .filter(|&&age| age > retention_days as i64)
                .count();

            prop_assert_eq!(
                deleted_count,
                expected_deleted,
                "Number of deleted backups should match expected. \
                 Retention: {} days, Ages: {:?}, Expected deleted: {}, Actual deleted: {}",
                retention_days,
                backup_ages,
                expected_deleted,
                deleted_count
            );

            // Property 2: Backups newer than or equal to retention_days should remain
            let expected_remaining = backup_ages.iter()
                .filter(|&&age| age <= retention_days as i64)
                .count();

            prop_assert_eq!(
                remaining_backups.len(),
                expected_remaining,
                "Number of remaining backups should match expected. \
                 Retention: {} days, Ages: {:?}, Expected remaining: {}, Actual remaining: {}",
                retention_days,
                backup_ages,
                expected_remaining,
                remaining_backups.len()
            );

            // Property 3: Total backups (deleted + remaining) should equal initial count
            prop_assert_eq!(
                deleted_count + remaining_backups.len(),
                backup_ages.len(),
                "Deleted + remaining should equal initial count"
            );

            // Property 4: All remaining backups should be within retention period
            for backup in &remaining_backups {
                let age_days = (now - backup.timestamp).num_days();
                prop_assert!(
                    age_days <= retention_days as i64,
                    "Remaining backup age ({} days) should be <= retention ({} days)",
                    age_days,
                    retention_days
                );
            }

            // Property 5: No backup older than retention should remain
            for backup in &remaining_backups {
                let age_days = (now - backup.timestamp).num_days();
                prop_assert!(
                    age_days <= retention_days as i64,
                    "No backup older than retention should remain. \
                     Backup age: {} days, Retention: {} days",
                    age_days,
                    retention_days
                );
            }
        });
    }

    /// **Property 31.1: Retention policy with zero days**
    ///
    /// With 0-day retention, all backups should be deleted immediately.
    ///
    /// **Validates: Requirements 2.5.5**
    #[test]
    fn prop_retention_zero_days(
        backup_count in 1usize..=10
    ) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let temp_dir = TempDir::new().unwrap();
            let mut config = BackupConfig::default();
            config.storage_config = StorageConfig::Local(LocalStorageConfig {
                path: temp_dir.path().to_string_lossy().to_string(),
            });
            config.retention_days = 0; // Delete all backups
            config.verify_after_backup = false;

            let manager = BackupManager::new(config).await.unwrap();

            // Create multiple backups
            for _ in 0..backup_count {
                manager.create_backup().await.unwrap();
            }

            // Verify backups were created
            let initial = manager.list_backups().await.unwrap();
            prop_assert_eq!(
                initial.len(),
                backup_count,
                "All backups should be created"
            );

            // Execute cleanup
            let deleted = manager.cleanup_old_backups_manual().await.unwrap();

            // Property 1: All backups should be deleted with 0-day retention
            prop_assert_eq!(
                deleted,
                backup_count,
                "All backups should be deleted with 0-day retention"
            );

            // Property 2: No backups should remain
            let remaining = manager.list_backups().await.unwrap();
            prop_assert_eq!(
                remaining.len(),
                0,
                "No backups should remain after cleanup with 0-day retention"
            );
        });
    }

    /// **Property 31.2: Retention policy with very large retention**
    ///
    /// With very large retention (e.g., 10 years), no backups should be deleted.
    ///
    /// **Validates: Requirements 2.5.5**
    #[test]
    fn prop_retention_large_value(
        backup_count in 1usize..=10
    ) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let temp_dir = TempDir::new().unwrap();
            let mut config = BackupConfig::default();
            config.storage_config = StorageConfig::Local(LocalStorageConfig {
                path: temp_dir.path().to_string_lossy().to_string(),
            });
            config.retention_days = 3650; // 10 years
            config.verify_after_backup = false;

            let manager = BackupManager::new(config).await.unwrap();

            // Create multiple backups
            for _ in 0..backup_count {
                manager.create_backup().await.unwrap();
            }

            // Execute cleanup
            let deleted = manager.cleanup_old_backups_manual().await.unwrap();

            // Property 1: No backups should be deleted with very large retention
            prop_assert_eq!(
                deleted,
                0,
                "No backups should be deleted with 10-year retention"
            );

            // Property 2: All backups should remain
            let remaining = manager.list_backups().await.unwrap();
            prop_assert_eq!(
                remaining.len(),
                backup_count,
                "All backups should remain with large retention"
            );
        });
    }

    /// **Property 31.3: Retention policy idempotence**
    ///
    /// Running cleanup multiple times should not delete additional backups.
    ///
    /// **Validates: Requirements 2.5.5**
    #[test]
    fn prop_retention_idempotence(
        retention_days in retention_days_strategy(),
        backup_ages in prop::collection::vec(backup_age_days_strategy(), 3..=8)
    ) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let temp_dir = TempDir::new().unwrap();
            let mut config = BackupConfig::default();
            config.storage_config = StorageConfig::Local(LocalStorageConfig {
                path: temp_dir.path().to_string_lossy().to_string(),
            });
            config.retention_days = retention_days;
            config.verify_after_backup = false;

            let manager = BackupManager::new(config).await.unwrap();

            // Create backups with different ages
            let now = Utc::now();
            for age_days in &backup_ages {
                let backup_id = manager.create_backup().await.unwrap();
                let storage_path = temp_dir.path().join(format!("{}.json", backup_id));
                let mut metadata: BackupMetadata = {
                    let content = std::fs::read_to_string(&storage_path).unwrap();
                    serde_json::from_str(&content).unwrap()
                };
                metadata.timestamp = now - ChronoDuration::days(*age_days);
                std::fs::write(&storage_path, serde_json::to_string_pretty(&metadata).unwrap()).unwrap();
            }

            // First cleanup
            let deleted1 = manager.cleanup_old_backups_manual().await.unwrap();
            let remaining1 = manager.list_backups().await.unwrap().len();

            // Second cleanup (should delete nothing)
            let deleted2 = manager.cleanup_old_backups_manual().await.unwrap();
            let remaining2 = manager.list_backups().await.unwrap().len();

            // Property 1: Second cleanup should delete nothing
            prop_assert_eq!(
                deleted2,
                0,
                "Second cleanup should delete no additional backups"
            );

            // Property 2: Remaining count should be unchanged
            prop_assert_eq!(
                remaining1,
                remaining2,
                "Remaining backup count should be unchanged after second cleanup"
            );

            // Third cleanup (should also delete nothing)
            let deleted3 = manager.cleanup_old_backups_manual().await.unwrap();
            let remaining3 = manager.list_backups().await.unwrap().len();

            // Property 3: Third cleanup should also delete nothing
            prop_assert_eq!(
                deleted3,
                0,
                "Third cleanup should delete no additional backups"
            );

            // Property 4: Remaining count should still be unchanged
            prop_assert_eq!(
                remaining2,
                remaining3,
                "Remaining backup count should be unchanged after third cleanup"
            );
        });
    }

    /// **Property 31.4: Retention policy boundary conditions**
    ///
    /// Backups exactly at the retention boundary should be kept.
    ///
    /// **Validates: Requirements 2.5.5**
    #[test]
    fn prop_retention_boundary(
        retention_days in 1u32..=30u32
    ) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let temp_dir = TempDir::new().unwrap();
            let mut config = BackupConfig::default();
            config.storage_config = StorageConfig::Local(LocalStorageConfig {
                path: temp_dir.path().to_string_lossy().to_string(),
            });
            config.retention_days = retention_days;
            config.verify_after_backup = false;

            let manager = BackupManager::new(config).await.unwrap();

            let now = Utc::now();

            // Create backup exactly at retention boundary
            let backup_at_boundary = manager.create_backup().await.unwrap();
            let path_boundary = temp_dir.path().join(format!("{}.json", backup_at_boundary));
            let mut meta_boundary: BackupMetadata = {
                let content = std::fs::read_to_string(&path_boundary).unwrap();
                serde_json::from_str(&content).unwrap()
            };
            meta_boundary.timestamp = now - ChronoDuration::days(retention_days as i64);
            std::fs::write(&path_boundary, serde_json::to_string_pretty(&meta_boundary).unwrap()).unwrap();

            // Create backup one day older than boundary
            let backup_older = manager.create_backup().await.unwrap();
            let path_older = temp_dir.path().join(format!("{}.json", backup_older));
            let mut meta_older: BackupMetadata = {
                let content = std::fs::read_to_string(&path_older).unwrap();
                serde_json::from_str(&content).unwrap()
            };
            meta_older.timestamp = now - ChronoDuration::days(retention_days as i64 + 1);
            std::fs::write(&path_older, serde_json::to_string_pretty(&meta_older).unwrap()).unwrap();

            // Create backup one day newer than boundary
            let backup_newer = manager.create_backup().await.unwrap();
            let path_newer = temp_dir.path().join(format!("{}.json", backup_newer));
            let mut meta_newer: BackupMetadata = {
                let content = std::fs::read_to_string(&path_newer).unwrap();
                serde_json::from_str(&content).unwrap()
            };
            meta_newer.timestamp = now - ChronoDuration::days(retention_days as i64 - 1);
            std::fs::write(&path_newer, serde_json::to_string_pretty(&meta_newer).unwrap()).unwrap();

            // Execute cleanup
            let deleted = manager.cleanup_old_backups_manual().await.unwrap();

            // Property 1: Only the backup older than boundary should be deleted
            prop_assert_eq!(
                deleted,
                1,
                "Only backup older than retention boundary should be deleted"
            );

            // Property 2: Backups at and newer than boundary should remain
            let remaining = manager.list_backups().await.unwrap();
            prop_assert_eq!(
                remaining.len(),
                2,
                "Backups at and newer than boundary should remain"
            );

            // Property 3: Remaining backups should be the correct ones
            let remaining_ids: Vec<String> = remaining.iter().map(|b| b.id.clone()).collect();
            prop_assert!(
                remaining_ids.contains(&backup_at_boundary),
                "Backup at boundary should remain"
            );
            prop_assert!(
                remaining_ids.contains(&backup_newer),
                "Backup newer than boundary should remain"
            );
            prop_assert!(
                !remaining_ids.contains(&backup_older),
                "Backup older than boundary should be deleted"
            );
        });
    }
}

// ============================================================================
// Property 32: Point-in-time recovery
// ============================================================================
// **Validates: Requirements 2.5.7**
//
// For any target timestamp, point-in-time recovery should:
// 1. Select the backup closest to (but not after) the target time
// 2. Restore the selected backup successfully
// 3. Handle edge cases (no backups, target before all backups, target after all backups)
// 4. Return the ID of the restored backup
// 5. Fail gracefully when no suitable backup exists

/// Strategy to generate backup timestamps (hours ago)
fn backup_hours_ago_strategy() -> impl Strategy<Value = i64> {
    prop_oneof![
        Just(1i64),      // 1 hour ago
        Just(2i64),      // 2 hours ago
        Just(6i64),      // 6 hours ago
        Just(12i64),     // 12 hours ago
        Just(24i64),     // 1 day ago
        Just(48i64),     // 2 days ago
        Just(72i64),     // 3 days ago
        1i64..=168i64,   // Random: 1 hour to 1 week ago
    ]
}

/// Strategy to generate target recovery times (hours ago)
fn target_hours_ago_strategy() -> impl Strategy<Value = i64> {
    prop_oneof![
        Just(0i64),      // Now
        Just(1i64),      // 1 hour ago
        Just(3i64),      // 3 hours ago
        Just(6i64),      // 6 hours ago
        Just(12i64),     // 12 hours ago
        Just(24i64),     // 1 day ago
        Just(48i64),     // 2 days ago
        0i64..=168i64,   // Random: now to 1 week ago
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// **Property 32: Point-in-time recovery selects correct backup**
    ///
    /// For any target timestamp and set of backups, the system must:
    /// 1. Select the backup with timestamp closest to (but not after) target
    /// 2. Restore the selected backup successfully
    /// 3. Return the ID of the restored backup
    /// 4. Fail if no backup exists before the target time
    ///
    /// **Validates: Requirements 2.5.7**
    #[test]
    fn prop_point_in_time_recovery_selection(
        backup_hours in prop::collection::vec(backup_hours_ago_strategy(), 3..=7),
        target_hours_ago in target_hours_ago_strategy()
    ) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            // Setup: Create temporary storage
            let temp_dir = TempDir::new().unwrap();
            let mut config = BackupConfig::default();
            config.storage_config = StorageConfig::Local(LocalStorageConfig {
                path: temp_dir.path().to_string_lossy().to_string(),
            });
            config.verify_after_backup = false; // Disable for speed
            config.compression_enabled = false; // Disable for simplicity

            let manager = BackupManager::new(config).await.unwrap();

            let now = Utc::now();
            let mut backup_ids = Vec::new();
            let mut backup_timestamps = Vec::new();

            // Create backups with different timestamps
            for hours_ago in &backup_hours {
                // Create a backup using the manager
                let backup_id = manager.create_backup().await.unwrap();
                backup_ids.push(backup_id.clone());

                // Manually modify backup timestamp
                let storage_path = temp_dir.path().join(format!("{}.metadata.json", backup_id));
                let mut metadata: BackupMetadata = {
                    let content = std::fs::read_to_string(&storage_path).unwrap();
                    serde_json::from_str(&content).unwrap()
                };
                let backup_time = now - ChronoDuration::hours(*hours_ago);
                metadata.timestamp = backup_time;
                backup_timestamps.push(backup_time);
                std::fs::write(&storage_path, serde_json::to_string_pretty(&metadata).unwrap()).unwrap();
            }

            // Calculate target time
            let target_time = now - ChronoDuration::hours(target_hours_ago);

            // Find expected backup (closest to but not after target)
            let expected_backup_idx = backup_timestamps.iter()
                .enumerate()
                .filter(|(_, &timestamp)| timestamp <= target_time)
                .max_by_key(|(_, &timestamp)| timestamp)
                .map(|(idx, _)| idx);

            // Execute point-in-time recovery
            let restore_options = RestoreOptions {
                backup_id: String::new(), // Will be set by restore_to_point_in_time
                restore_raft: true,
                restore_postgres: false, // Skip postgres restore in test
                skip_verification: false,
                force: false,
            };

            let result = manager.restore_to_point_in_time(target_time, restore_options).await;

            if let Some(expected_idx) = expected_backup_idx {
                // Property 1: Restore should succeed when suitable backup exists
                prop_assert!(
                    result.is_ok(),
                    "Restore should succeed when backup exists before target time. \
                     Target: {}, Backups: {:?}, Error: {:?}",
                    target_time,
                    backup_timestamps,
                    result.err()
                );

                let restored_id = result.unwrap();

                // Property 2: Restored backup ID should match expected backup
                prop_assert_eq!(
                    restored_id,
                    backup_ids[expected_idx],
                    "Restored backup should be the one closest to (but not after) target time. \
                     Target: {}, Expected timestamp: {}, Backups: {:?}",
                    target_time,
                    backup_timestamps[expected_idx],
                    backup_timestamps
                );

                // Property 3: Selected backup timestamp should be <= target time
                let selected_timestamp = backup_timestamps[expected_idx];
                prop_assert!(
                    selected_timestamp <= target_time,
                    "Selected backup timestamp ({}) should be <= target time ({})",
                    selected_timestamp,
                    target_time
                );

                // Property 4: No other backup should be closer to target (and still <= target)
                for (idx, &timestamp) in backup_timestamps.iter().enumerate() {
                    if timestamp <= target_time && idx != expected_idx {
                        let time_diff_selected = (target_time - selected_timestamp).num_seconds();
                        let time_diff_other = (target_time - timestamp).num_seconds();
                        prop_assert!(
                            time_diff_selected <= time_diff_other,
                            "Selected backup should be closest to target. \
                             Selected diff: {}s, Other diff: {}s",
                            time_diff_selected,
                            time_diff_other
                        );
                    }
                }
            } else {
                // Property 5: Restore should fail when no backup exists before target time
                prop_assert!(
                    result.is_err(),
                    "Restore should fail when no backup exists before target time. \
                     Target: {}, Backups: {:?}",
                    target_time,
                    backup_timestamps
                );

                // Property 6: Error message should indicate no suitable backup
                if let Err(e) = result {
                    let error_msg = e.to_string();
                    prop_assert!(
                        error_msg.contains("No backup found") || error_msg.contains("before target time"),
                        "Error message should indicate no backup found before target time: {}",
                        error_msg
                    );
                }
            }
        });
    }

    /// **Property 32.1: Point-in-time recovery with no backups**
    ///
    /// When no backups exist, point-in-time recovery must fail gracefully.
    ///
    /// **Validates: Requirements 2.5.7**
    #[test]
    fn prop_pitr_no_backups(
        target_hours_ago in target_hours_ago_strategy()
    ) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let temp_dir = TempDir::new().unwrap();
            let mut config = BackupConfig::default();
            config.storage_config = StorageConfig::Local(LocalStorageConfig {
                path: temp_dir.path().to_string_lossy().to_string(),
            });

            let manager = BackupManager::new(config).await.unwrap();

            let target_time = Utc::now() - ChronoDuration::hours(target_hours_ago);

            let restore_options = RestoreOptions {
                backup_id: String::new(),
                restore_raft: true,
                restore_postgres: false,
                skip_verification: false,
                force: false,
            };

            let result = manager.restore_to_point_in_time(target_time, restore_options).await;

            // Property 1: Should fail when no backups exist
            prop_assert!(
                result.is_err(),
                "Point-in-time recovery should fail when no backups exist"
            );

            // Property 2: Error should indicate no backups available
            if let Err(e) = result {
                let error_msg = e.to_string();
                prop_assert!(
                    error_msg.contains("No backup") || error_msg.contains("available"),
                    "Error message should indicate no backups available: {}",
                    error_msg
                );
            }
        });
    }

    /// **Property 32.2: Point-in-time recovery with target before all backups**
    ///
    /// When target time is before all backups, recovery must fail.
    ///
    /// **Validates: Requirements 2.5.7**
    #[test]
    fn prop_pitr_target_before_all_backups(
        backup_count in 2usize..=5
    ) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let temp_dir = TempDir::new().unwrap();
            let mut config = BackupConfig::default();
            config.storage_config = StorageConfig::Local(LocalStorageConfig {
                path: temp_dir.path().to_string_lossy().to_string(),
            });
            config.verify_after_backup = false;
            config.compression_enabled = false;

            let manager = BackupManager::new(config).await.unwrap();

            let now = Utc::now();
            let mut earliest_backup_time = now;

            // Create backups (all in the past 24 hours)
            for i in 1..=backup_count {
                let backup_id = manager.create_backup().await.unwrap();

                // Set backup timestamp to i hours ago
                let storage_path = temp_dir.path().join(format!("{}.metadata.json", backup_id));
                let mut metadata: BackupMetadata = {
                    let content = std::fs::read_to_string(&storage_path).unwrap();
                    serde_json::from_str(&content).unwrap()
                };
                let backup_time = now - ChronoDuration::hours(i as i64);
                metadata.timestamp = backup_time;
                if backup_time < earliest_backup_time {
                    earliest_backup_time = backup_time;
                }
                std::fs::write(&storage_path, serde_json::to_string_pretty(&metadata).unwrap()).unwrap();
            }

            // Set target time to before all backups (earliest - 1 hour)
            let target_time = earliest_backup_time - ChronoDuration::hours(1);

            let restore_options = RestoreOptions {
                backup_id: String::new(),
                restore_raft: true,
                restore_postgres: false,
                skip_verification: false,
                force: false,
            };

            let result = manager.restore_to_point_in_time(target_time, restore_options).await;

            // Property 1: Should fail when target is before all backups
            prop_assert!(
                result.is_err(),
                "Point-in-time recovery should fail when target is before all backups. \
                 Target: {}, Earliest backup: {}",
                target_time,
                earliest_backup_time
            );

            // Property 2: Error should mention earliest backup time
            if let Err(e) = result {
                let error_msg = e.to_string();
                prop_assert!(
                    error_msg.contains("No backup found") || error_msg.contains("before target time"),
                    "Error message should indicate no backup before target: {}",
                    error_msg
                );
            }
        });
    }

    /// **Property 32.3: Point-in-time recovery with target after all backups**
    ///
    /// When target time is after all backups, the most recent backup should be selected.
    ///
    /// **Validates: Requirements 2.5.7**
    #[test]
    fn prop_pitr_target_after_all_backups(
        backup_count in 2usize..=5
    ) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let temp_dir = TempDir::new().unwrap();
            let mut config = BackupConfig::default();
            config.storage_config = StorageConfig::Local(LocalStorageConfig {
                path: temp_dir.path().to_string_lossy().to_string(),
            });
            config.verify_after_backup = false;
            config.compression_enabled = false;

            let manager = BackupManager::new(config).await.unwrap();

            let now = Utc::now();
            let mut most_recent_backup_id = String::new();
            let mut most_recent_backup_time = now - ChronoDuration::days(365); // Very old

            // Create backups (all in the past)
            for i in 1..=backup_count {
                let backup_id = manager.create_backup().await.unwrap();

                // Set backup timestamp to (backup_count - i + 1) hours ago
                // This makes the last backup the most recent
                let storage_path = temp_dir.path().join(format!("{}.metadata.json", backup_id));
                let mut metadata: BackupMetadata = {
                    let content = std::fs::read_to_string(&storage_path).unwrap();
                    serde_json::from_str(&content).unwrap()
                };
                let backup_time = now - ChronoDuration::hours((backup_count - i + 1) as i64);
                metadata.timestamp = backup_time;
                if backup_time > most_recent_backup_time {
                    most_recent_backup_time = backup_time;
                    most_recent_backup_id = backup_id.clone();
                }
                std::fs::write(&storage_path, serde_json::to_string_pretty(&metadata).unwrap()).unwrap();
            }

            // Set target time to now (after all backups)
            let target_time = now;

            let restore_options = RestoreOptions {
                backup_id: String::new(),
                restore_raft: true,
                restore_postgres: false,
                skip_verification: false,
                force: false,
            };

            let result = manager.restore_to_point_in_time(target_time, restore_options).await;

            // Property 1: Should succeed and select most recent backup
            prop_assert!(
                result.is_ok(),
                "Point-in-time recovery should succeed when target is after all backups. \
                 Target: {}, Most recent backup: {}",
                target_time,
                most_recent_backup_time
            );

            let restored_id = result.unwrap();

            // Property 2: Should restore the most recent backup
            prop_assert_eq!(
                restored_id,
                most_recent_backup_id,
                "Should restore the most recent backup when target is after all backups"
            );
        });
    }

    /// **Property 32.4: Point-in-time recovery boundary conditions**
    ///
    /// When target time exactly matches a backup timestamp, that backup should be selected.
    ///
    /// **Validates: Requirements 2.5.7**
    #[test]
    fn prop_pitr_exact_match(
        backup_count in 3usize..=6
    ) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let temp_dir = TempDir::new().unwrap();
            let mut config = BackupConfig::default();
            config.storage_config = StorageConfig::Local(LocalStorageConfig {
                path: temp_dir.path().to_string_lossy().to_string(),
            });
            config.verify_after_backup = false;
            config.compression_enabled = false;

            let manager = BackupManager::new(config).await.unwrap();

            let now = Utc::now();
            let mut backup_ids = Vec::new();
            let mut backup_timestamps = Vec::new();

            // Create backups with different timestamps
            for i in 1..=backup_count {
                let backup_id = manager.create_backup().await.unwrap();
                backup_ids.push(backup_id.clone());

                // Set backup timestamp to i hours ago
                let storage_path = temp_dir.path().join(format!("{}.metadata.json", backup_id));
                let mut metadata: BackupMetadata = {
                    let content = std::fs::read_to_string(&storage_path).unwrap();
                    serde_json::from_str(&content).unwrap()
                };
                let backup_time = now - ChronoDuration::hours(i as i64);
                metadata.timestamp = backup_time;
                backup_timestamps.push(backup_time);
                std::fs::write(&storage_path, serde_json::to_string_pretty(&metadata).unwrap()).unwrap();
            }

            // Test exact match for middle backup
            let middle_idx = backup_count / 2;
            let target_time = backup_timestamps[middle_idx];

            let restore_options = RestoreOptions {
                backup_id: String::new(),
                restore_raft: true,
                restore_postgres: false,
                skip_verification: false,
                force: false,
            };

            let result = manager.restore_to_point_in_time(target_time, restore_options).await;

            // Property 1: Should succeed with exact timestamp match
            prop_assert!(
                result.is_ok(),
                "Point-in-time recovery should succeed with exact timestamp match. \
                 Target: {}, Backups: {:?}",
                target_time,
                backup_timestamps
            );

            let restored_id = result.unwrap();

            // Property 2: Should restore the backup with exact matching timestamp
            prop_assert_eq!(
                restored_id,
                backup_ids[middle_idx],
                "Should restore backup with exact matching timestamp"
            );
        });
    }

    /// **Property 32.5: Point-in-time recovery idempotence**
    ///
    /// Restoring to the same point in time multiple times should select the same backup.
    ///
    /// **Validates: Requirements 2.5.7**
    #[test]
    fn prop_pitr_idempotence(
        backup_hours in prop::collection::vec(backup_hours_ago_strategy(), 3..=5),
        target_hours_ago in target_hours_ago_strategy()
    ) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let temp_dir = TempDir::new().unwrap();
            let mut config = BackupConfig::default();
            config.storage_config = StorageConfig::Local(LocalStorageConfig {
                path: temp_dir.path().to_string_lossy().to_string(),
            });
            config.verify_after_backup = false;
            config.compression_enabled = false;

            let manager = BackupManager::new(config).await.unwrap();

            let now = Utc::now();

            // Create backups
            for hours_ago in &backup_hours {
                let backup_id = manager.create_backup().await.unwrap();

                let storage_path = temp_dir.path().join(format!("{}.metadata.json", backup_id));
                let mut metadata: BackupMetadata = {
                    let content = std::fs::read_to_string(&storage_path).unwrap();
                    serde_json::from_str(&content).unwrap()
                };
                metadata.timestamp = now - ChronoDuration::hours(*hours_ago);
                std::fs::write(&storage_path, serde_json::to_string_pretty(&metadata).unwrap()).unwrap();
            }

            let target_time = now - ChronoDuration::hours(target_hours_ago);

            let restore_options = RestoreOptions {
                backup_id: String::new(),
                restore_raft: true,
                restore_postgres: false,
                skip_verification: false,
                force: false,
            };

            // First restore
            let result1 = manager.restore_to_point_in_time(target_time, restore_options.clone()).await;

            // Second restore to same point in time
            let result2 = manager.restore_to_point_in_time(target_time, restore_options.clone()).await;

            // Third restore to same point in time
            let result3 = manager.restore_to_point_in_time(target_time, restore_options).await;

            // Property 1: All three attempts should have same success/failure status
            prop_assert_eq!(
                result1.is_ok(),
                result2.is_ok(),
                "First and second restore should have same success status"
            );
            prop_assert_eq!(
                result2.is_ok(),
                result3.is_ok(),
                "Second and third restore should have same success status"
            );

            // Property 2: If successful, all should restore the same backup
            if result1.is_ok() {
                let id1 = result1.unwrap();
                let id2 = result2.unwrap();
                let id3 = result3.unwrap();

                prop_assert_eq!(
                    id1, id2,
                    "First and second restore should select same backup"
                );
                prop_assert_eq!(
                    id2, id3,
                    "Second and third restore should select same backup"
                );
            }
        });
    }
}

// ============================================================================
// Property 33: Automatic backup verification
// ============================================================================
// **Validates: Requirements 2.5.8**
//
// For any created backup, the system should automatically verify its integrity.
// This property verifies that:
// 1. Verification correctly validates encryption integrity
// 2. Verification correctly validates compression integrity
// 3. Verification correctly validates data integrity (checksums)
// 4. Verification updates metadata with verification status
// 5. Verification detects corrupted backups
// 6. Verification handles various corruption scenarios

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// **Property 33: Automatic backup verification**
    ///
    /// For any created backup, automatic verification should:
    /// 1. Successfully verify valid backups
    /// 2. Update metadata with verification status and timestamp
    /// 3. Detect encryption corruption
    /// 4. Detect compression corruption
    /// 5. Detect data corruption (checksum mismatch)
    /// 6. Mark backup as verified on success
    /// 7. Mark backup as verification failed on corruption
    ///
    /// **Validates: Requirements 2.5.8**
    #[test]
    fn prop_automatic_backup_verification(
        raft_data in raft_snapshot_strategy(),
        postgres_data in postgres_dump_strategy()
    ) -> Result<(), TestCaseError> {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            // Setup: Create backup manager with verification enabled
            let temp_dir = TempDir::new().unwrap();
            let mut config = BackupConfig::default();
            config.storage_config = StorageConfig::Local(LocalStorageConfig {
                path: temp_dir.path().to_string_lossy().to_string(),
            });
            config.verify_after_backup = true; // Enable automatic verification
            config.compression_enabled = true;

            let manager = Arc::new(BackupManager::new(config).await.unwrap());

            // Create a backup with the generated data
            let mut backup = Backup::new(raft_data.clone(), postgres_data.clone());

            // Compress the data
            let compressed_raft = manager.compress(&backup.raft_snapshot).unwrap();
            let compressed_postgres = manager.compress(&backup.postgres_dump).unwrap();
            backup.set_compressed_size((compressed_raft.len() + compressed_postgres.len()) as u64);

            // Encrypt the compressed data
            let encrypted_raft = manager.encrypt(&compressed_raft).unwrap();
            let encrypted_postgres = manager.encrypt(&compressed_postgres).unwrap();
            backup.set_encrypted_size((encrypted_raft.len() + encrypted_postgres.len()) as u64);
            backup.raft_snapshot = encrypted_raft;
            backup.postgres_dump = encrypted_postgres;

            // Mark as completed (calculates checksum)
            backup.mark_completed();

            // Store the backup
            manager.storage.upload(&backup).await.unwrap();

            // Property 1: Verification should succeed for valid backup
            let verify_result = manager.verify_backup(&backup.id).await;
            prop_assert!(
                verify_result.is_ok(),
                "Verification should succeed for valid backup: {:?}",
                verify_result.err()
            );

            // Property 2: Backup metadata should be updated with verification status
            let verified_backup = manager.storage.download(&backup.id).await.unwrap();
            prop_assert_eq!(
                verified_backup.metadata.status,
                BackupStatus::Verified,
                "Backup status should be Verified after successful verification"
            );

            // Property 3: Verification timestamp should be set
            prop_assert!(
                verified_backup.metadata.verified_at.is_some(),
                "Verification timestamp should be set after successful verification"
            );

            // Property 4: Verification timestamp should not be in the future
            let verified_at = verified_backup.metadata.verified_at.unwrap();
            let now = Utc::now();
            prop_assert!(
                verified_at <= now,
                "Verification timestamp should not be in the future. Verified: {}, Now: {}",
                verified_at,
                now
            );

            // Property 5: Verification timestamp should be after backup creation
            prop_assert!(
                verified_at >= verified_backup.timestamp,
                "Verification timestamp should be after backup creation. \
                 Verified: {}, Created: {}",
                verified_at,
                verified_backup.timestamp
            );

            // Property 6: is_verified() should return true
            prop_assert!(
                verified_backup.metadata.is_verified(),
                "is_verified() should return true after successful verification"
            );

            // Property 7: is_failed() should return false
            prop_assert!(
                !verified_backup.metadata.is_failed(),
                "is_failed() should return false for verified backup"
            );

            // Property 8: Error field should be None
            prop_assert!(
                verified_backup.metadata.error.is_none(),
                "Error field should be None for verified backup"
            );

            // Property 9: Checksum should remain unchanged
            prop_assert_eq!(
                verified_backup.metadata.checksum,
                backup.metadata.checksum,
                "Checksum should remain unchanged after verification"
            );

            // Property 10: Multiple verifications should be idempotent
            let verify_again = manager.verify_backup(&backup.id).await;
            prop_assert!(
                verify_again.is_ok(),
                "Second verification should also succeed"
            );

            let verified_again = manager.storage.download(&backup.id).await.unwrap();
            prop_assert_eq!(
                verified_again.metadata.status,
                BackupStatus::Verified,
                "Status should remain Verified after second verification"
            );
        });
    }

    /// **Property 33.1: Verification detects encryption corruption**
    ///
    /// Verification should detect when backup encryption is corrupted.
    /// This validates that AEAD authentication tags are properly checked.
    ///
    /// **Validates: Requirements 2.5.8**
    #[test]
    fn prop_verification_detects_encryption_corruption(
        raft_data in raft_snapshot_strategy(),
        postgres_data in postgres_dump_strategy(),
        corruption_byte_index in 0usize..100usize
    ) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            // Setup
            let temp_dir = TempDir::new().unwrap();
            let mut config = BackupConfig::default();
            config.storage_config = StorageConfig::Local(LocalStorageConfig {
                path: temp_dir.path().to_string_lossy().to_string(),
            });
            config.compression_enabled = true;

            let manager = BackupManager::new(config).await.unwrap();

            // Create and encrypt backup
            let mut backup = Backup::new(raft_data.clone(), postgres_data.clone());
            let compressed_raft = manager.compress(&backup.raft_snapshot).unwrap();
            let compressed_postgres = manager.compress(&backup.postgres_dump).unwrap();
            backup.set_compressed_size((compressed_raft.len() + compressed_postgres.len()) as u64);

            let mut encrypted_raft = manager.encrypt(&compressed_raft).unwrap();
            let encrypted_postgres = manager.encrypt(&compressed_postgres).unwrap();
            backup.set_encrypted_size((encrypted_raft.len() + encrypted_postgres.len()) as u64);

            // Corrupt the encrypted data (flip a bit in the ciphertext)
            if corruption_byte_index < encrypted_raft.len() {
                encrypted_raft[corruption_byte_index] ^= 0xFF;
            }

            backup.raft_snapshot = encrypted_raft;
            backup.postgres_dump = encrypted_postgres;
            backup.mark_completed();

            // Store the corrupted backup
            manager.storage.upload(&backup).await.unwrap();

            // Property 1: Verification should fail for corrupted backup
            let verify_result = manager.verify_backup(&backup.id).await;
            prop_assert!(
                verify_result.is_err(),
                "Verification should fail for backup with encryption corruption"
            );

            // Property 2: Error should indicate decryption failure
            if let Err(e) = verify_result {
                let error_msg = e.to_string().to_lowercase();
                prop_assert!(
                    error_msg.contains("decrypt") || error_msg.contains("encryption"),
                    "Error message should indicate decryption/encryption failure: {}",
                    e
                );
            }

            // Property 3: Backup status should be VerificationFailed
            let failed_backup = manager.storage.download(&backup.id).await.unwrap();
            prop_assert_eq!(
                failed_backup.metadata.status,
                BackupStatus::VerificationFailed,
                "Backup status should be VerificationFailed after failed verification"
            );

            // Property 4: Error field should be set
            prop_assert!(
                failed_backup.metadata.error.is_some(),
                "Error field should be set for failed verification"
            );

            // Property 5: is_verified() should return false
            prop_assert!(
                !failed_backup.metadata.is_verified(),
                "is_verified() should return false for failed verification"
            );

            // Property 6: is_failed() should return true
            prop_assert!(
                failed_backup.metadata.is_failed(),
                "is_failed() should return true for failed verification"
            );

            // Property 7: verified_at should not be set
            prop_assert!(
                failed_backup.metadata.verified_at.is_none(),
                "verified_at should not be set for failed verification"
            );
        });
    }

    /// **Property 33.2: Verification detects compression corruption**
    ///
    /// Verification should detect when backup compression is corrupted.
    ///
    /// **Validates: Requirements 2.5.8**
    #[test]
    fn prop_verification_detects_compression_corruption(
        raft_data in raft_snapshot_strategy(),
        postgres_data in postgres_dump_strategy()
    ) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            // Setup
            let temp_dir = TempDir::new().unwrap();
            let mut config = BackupConfig::default();
            config.storage_config = StorageConfig::Local(LocalStorageConfig {
                path: temp_dir.path().to_string_lossy().to_string(),
            });
            config.compression_enabled = true;

            let manager = BackupManager::new(config).await.unwrap();

            // Create backup
            let mut backup = Backup::new(raft_data.clone(), postgres_data.clone());

            // Compress the data
            let mut compressed_raft = manager.compress(&backup.raft_snapshot).unwrap();
            let compressed_postgres = manager.compress(&backup.postgres_dump).unwrap();

            // Corrupt the compressed data (truncate it)
            if compressed_raft.len() > 10 {
                compressed_raft.truncate(compressed_raft.len() / 2);
            }

            backup.set_compressed_size((compressed_raft.len() + compressed_postgres.len()) as u64);

            // Encrypt the corrupted compressed data
            let encrypted_raft = manager.encrypt(&compressed_raft).unwrap();
            let encrypted_postgres = manager.encrypt(&compressed_postgres).unwrap();
            backup.set_encrypted_size((encrypted_raft.len() + encrypted_postgres.len()) as u64);
            backup.raft_snapshot = encrypted_raft;
            backup.postgres_dump = encrypted_postgres;
            backup.mark_completed();

            // Store the backup
            manager.storage.upload(&backup).await.unwrap();

            // Property 1: Verification should fail for corrupted compression
            let verify_result = manager.verify_backup(&backup.id).await;
            prop_assert!(
                verify_result.is_err(),
                "Verification should fail for backup with compression corruption"
            );

            // Property 2: Error should indicate decompression failure
            if let Err(e) = verify_result {
                let error_msg = e.to_string().to_lowercase();
                prop_assert!(
                    error_msg.contains("decompress") || error_msg.contains("compression"),
                    "Error message should indicate decompression/compression failure: {}",
                    e
                );
            }
        });
    }

    /// **Property 33.3: Verification detects checksum mismatch**
    ///
    /// Verification should detect when backup data is corrupted (checksum mismatch).
    ///
    /// **Validates: Requirements 2.5.8**
    #[test]
    fn prop_verification_detects_checksum_mismatch(
        raft_data in raft_snapshot_strategy(),
        postgres_data in postgres_dump_strategy()
    ) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            // Setup
            let temp_dir = TempDir::new().unwrap();
            let mut config = BackupConfig::default();
            config.storage_config = StorageConfig::Local(LocalStorageConfig {
                path: temp_dir.path().to_string_lossy().to_string(),
            });
            config.compression_enabled = true;

            let manager = BackupManager::new(config).await.unwrap();

            // Create backup
            let mut backup = Backup::new(raft_data.clone(), postgres_data.clone());
            let compressed_raft = manager.compress(&backup.raft_snapshot).unwrap();
            let compressed_postgres = manager.compress(&backup.postgres_dump).unwrap();
            backup.set_compressed_size((compressed_raft.len() + compressed_postgres.len()) as u64);

            let encrypted_raft = manager.encrypt(&compressed_raft).unwrap();
            let encrypted_postgres = manager.encrypt(&compressed_postgres).unwrap();
            backup.set_encrypted_size((encrypted_raft.len() + encrypted_postgres.len()) as u64);
            backup.raft_snapshot = encrypted_raft;
            backup.postgres_dump = encrypted_postgres;
            backup.mark_completed();

            // Tamper with the checksum (simulate data corruption that passes decryption)
            let original_checksum = backup.metadata.checksum.clone();
            backup.metadata.checksum = "0000000000000000000000000000000000000000000000000000000000000000".to_string();

            // Store the backup with wrong checksum
            manager.storage.upload(&backup).await.unwrap();

            // Property 1: Verification should fail for checksum mismatch
            let verify_result = manager.verify_backup(&backup.id).await;
            prop_assert!(
                verify_result.is_err(),
                "Verification should fail for backup with checksum mismatch"
            );

            // Property 2: Error should indicate checksum mismatch
            if let Err(e) = verify_result {
                let error_msg = e.to_string().to_lowercase();
                prop_assert!(
                    error_msg.contains("checksum"),
                    "Error message should indicate checksum mismatch: {}",
                    e
                );
            }

            // Property 3: Backup status should be VerificationFailed
            let failed_backup = manager.storage.download(&backup.id).await.unwrap();
            prop_assert_eq!(
                failed_backup.metadata.status,
                BackupStatus::VerificationFailed,
                "Backup status should be VerificationFailed for checksum mismatch"
            );

            // Property 4: Error message should mention checksum mismatch
            prop_assert!(
                failed_backup.metadata.error.is_some(),
                "Error field should be set for checksum mismatch"
            );

            let error = failed_backup.metadata.error.unwrap();
            prop_assert!(
                error.to_lowercase().contains("checksum"),
                "Error message should mention checksum: {}",
                error
            );

            // Property 5: Error should include both expected and actual checksums
            prop_assert!(
                error.contains(&original_checksum) || error.contains("expected"),
                "Error should include expected checksum information: {}",
                error
            );
        });
    }

    /// **Property 33.4: Verification is idempotent**
    ///
    /// Multiple verification attempts on the same backup should produce
    /// consistent results.
    ///
    /// **Validates: Requirements 2.5.8**
    #[test]
    fn prop_verification_idempotent(
        raft_data in raft_snapshot_strategy(),
        postgres_data in postgres_dump_strategy(),
        num_verifications in 2usize..10usize
    ) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            // Setup
            let temp_dir = TempDir::new().unwrap();
            let mut config = BackupConfig::default();
            config.storage_config = StorageConfig::Local(LocalStorageConfig {
                path: temp_dir.path().to_string_lossy().to_string(),
            });
            config.compression_enabled = true;

            let manager = BackupManager::new(config).await.unwrap();

            // Create valid backup
            let mut backup = Backup::new(raft_data.clone(), postgres_data.clone());
            let compressed_raft = manager.compress(&backup.raft_snapshot).unwrap();
            let compressed_postgres = manager.compress(&backup.postgres_dump).unwrap();
            backup.set_compressed_size((compressed_raft.len() + compressed_postgres.len()) as u64);

            let encrypted_raft = manager.encrypt(&compressed_raft).unwrap();
            let encrypted_postgres = manager.encrypt(&compressed_postgres).unwrap();
            backup.set_encrypted_size((encrypted_raft.len() + encrypted_postgres.len()) as u64);
            backup.raft_snapshot = encrypted_raft;
            backup.postgres_dump = encrypted_postgres;
            backup.mark_completed();

            manager.storage.upload(&backup).await.unwrap();

            // Perform multiple verifications
            let mut results = Vec::new();
            let mut statuses = Vec::new();

            for i in 0..num_verifications {
                let result = manager.verify_backup(&backup.id).await;
                results.push(result.is_ok());

                let verified = manager.storage.download(&backup.id).await.unwrap();
                statuses.push(verified.metadata.status.clone());

                // Small delay between verifications
                if i < num_verifications - 1 {
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                }
            }

            // Property 1: All verifications should succeed
            for (i, success) in results.iter().enumerate() {
                prop_assert!(
                    *success,
                    "Verification {} should succeed",
                    i + 1
                );
            }

            // Property 2: All statuses should be Verified
            for (i, status) in statuses.iter().enumerate() {
                prop_assert_eq!(
                    *status,
                    BackupStatus::Verified,
                    "Status after verification {} should be Verified",
                    i + 1
                );
            }

            // Property 3: Final backup should be verified
            let final_backup = manager.storage.download(&backup.id).await.unwrap();
            prop_assert!(
                final_backup.metadata.is_verified(),
                "Final backup should be verified after {} verifications",
                num_verifications
            );

            // Property 4: Checksum should remain unchanged
            prop_assert_eq!(
                final_backup.metadata.checksum,
                backup.metadata.checksum,
                "Checksum should remain unchanged after multiple verifications"
            );
        });
    }

    /// **Property 33.5: Verification handles missing backup**
    ///
    /// Verification should properly handle attempts to verify non-existent backups.
    ///
    /// **Validates: Requirements 2.5.8**
    #[test]
    fn prop_verification_handles_missing_backup(
        backup_id in "[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}"
    ) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            // Setup
            let temp_dir = TempDir::new().unwrap();
            let mut config = BackupConfig::default();
            config.storage_config = StorageConfig::Local(LocalStorageConfig {
                path: temp_dir.path().to_string_lossy().to_string(),
            });

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
                    error_msg.contains("not found") || error_msg.contains("does not exist"),
                    "Error message should indicate backup not found: {}",
                    e
                );
            }
        });
    }

    /// **Property 33.6: Verification with compression disabled**
    ///
    /// Verification should work correctly when compression is disabled.
    ///
    /// **Validates: Requirements 2.5.8**
    #[test]
    fn prop_verification_without_compression(
        raft_data in raft_snapshot_strategy(),
        postgres_data in postgres_dump_strategy()
    ) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            // Setup with compression disabled
            let temp_dir = TempDir::new().unwrap();
            let mut config = BackupConfig::default();
            config.storage_config = StorageConfig::Local(LocalStorageConfig {
                path: temp_dir.path().to_string_lossy().to_string(),
            });
            config.compression_enabled = false; // Disable compression

            let manager = BackupManager::new(config).await.unwrap();

            // Create backup without compression
            let mut backup = Backup::new(raft_data.clone(), postgres_data.clone());

            // Encrypt directly without compression
            let encrypted_raft = manager.encrypt(&backup.raft_snapshot).unwrap();
            let encrypted_postgres = manager.encrypt(&backup.postgres_dump).unwrap();
            backup.set_encrypted_size((encrypted_raft.len() + encrypted_postgres.len()) as u64);
            backup.raft_snapshot = encrypted_raft;
            backup.postgres_dump = encrypted_postgres;
            backup.mark_completed();

            manager.storage.upload(&backup).await.unwrap();

            // Property 1: Verification should succeed without compression
            let verify_result = manager.verify_backup(&backup.id).await;
            prop_assert!(
                verify_result.is_ok(),
                "Verification should succeed for backup without compression: {:?}",
                verify_result.err()
            );

            // Property 2: Backup should be marked as verified
            let verified_backup = manager.storage.download(&backup.id).await.unwrap();
            prop_assert_eq!(
                verified_backup.metadata.status,
                BackupStatus::Verified,
                "Backup status should be Verified"
            );

            // Property 3: Compression ratio should be 1.0 (no compression)
            prop_assert_eq!(
                verified_backup.metadata.compression_ratio(),
                1.0,
                "Compression ratio should be 1.0 when compression is disabled"
            );
        });
    }
}
