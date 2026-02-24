//! Property-based tests for backup retention functionality
//!
//! **Property 31: Backup retention**
//! **Validates: Requirements 2.5.5**

use chrono::{Duration as ChronoDuration, Utc};
use proptest::prelude::*;
use secreton_backup::prelude::*;
use secreton_backup::types::{BackupConfig, LocalStorageConfig, StorageConfig};
use tempfile::TempDir;

// Set a dummy DATABASE_URL for testing (backup manager requires it)
fn setup_test_env() {
    std::env::set_var("DATABASE_URL", "postgresql://test:test@localhost:5432/test");
}

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
        Just(0u32),    // Delete all backups immediately
        Just(1u32),    // Keep only today's backups
        Just(7u32),    // One week
        Just(30u32),   // One month (default)
        Just(90u32),   // Three months
        Just(365u32),  // One year
        1u32..=365u32, // Random value between 1 and 365
    ]
}

/// Strategy to generate backup ages in days
fn backup_age_days_strategy() -> impl Strategy<Value = i64> {
    prop_oneof![
        Just(0i64),    // Today
        Just(1i64),    // Yesterday
        Just(7i64),    // One week ago
        Just(30i64),   // One month ago
        Just(60i64),   // Two months ago
        Just(90i64),   // Three months ago
        Just(180i64),  // Six months ago
        Just(365i64),  // One year ago
        0i64..=365i64, // Random age
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
        setup_test_env(); // Set DATABASE_URL for testing
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            // Setup: Create temporary storage
            let temp_dir = TempDir::new().unwrap();
            let mut config = BackupConfig::default();
            config.storage_config = StorageConfig::Local(LocalStorageConfig {
                path: temp_dir.path().to_string_lossy().to_string(),
            });
            config.retention_days = retention_days;
            config.verify_after_backup = false; // Disable for speed
            config.database_url = None; // Disable PostgreSQL dump for testing

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

            Ok(())
        })?;
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
        setup_test_env(); // Set DATABASE_URL for testing
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let temp_dir = TempDir::new().unwrap();
            let mut config = BackupConfig::default();
            config.storage_config = StorageConfig::Local(LocalStorageConfig {
                path: temp_dir.path().to_string_lossy().to_string(),
            });
            config.retention_days = 0; // Delete all backups
            config.verify_after_backup = false;
            config.database_url = None; // Disable PostgreSQL dump for testing

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

            Ok(())
        })?;
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
        setup_test_env(); // Set DATABASE_URL for testing
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let temp_dir = TempDir::new().unwrap();
            let mut config = BackupConfig::default();
            config.storage_config = StorageConfig::Local(LocalStorageConfig {
                path: temp_dir.path().to_string_lossy().to_string(),
            });
            config.retention_days = 3650; // 10 years
            config.verify_after_backup = false;
            config.database_url = None; // Disable PostgreSQL dump for testing

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

            Ok(())
        })?;
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
        setup_test_env(); // Set DATABASE_URL for testing
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let temp_dir = TempDir::new().unwrap();
            let mut config = BackupConfig::default();
            config.storage_config = StorageConfig::Local(LocalStorageConfig {
                path: temp_dir.path().to_string_lossy().to_string(),
            });
            config.retention_days = retention_days;
            config.verify_after_backup = false;
            config.database_url = None; // Disable PostgreSQL dump for testing

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

            Ok(())
        })?;
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
        setup_test_env(); // Set DATABASE_URL for testing
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let temp_dir = TempDir::new().unwrap();
            let mut config = BackupConfig::default();
            config.storage_config = StorageConfig::Local(LocalStorageConfig {
                path: temp_dir.path().to_string_lossy().to_string(),
            });
            config.retention_days = retention_days;
            config.verify_after_backup = false;
            config.database_url = None; // Disable PostgreSQL dump for testing

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

            Ok(())
        })?;
    }
}
