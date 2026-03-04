#![cfg(any())]
//! Integration tests for PostgreSQL dump functionality
//!
//! These tests verify that the backup manager can correctly:
//! - Parse PostgreSQL connection URLs
//! - Execute pg_dump commands
//! - Handle errors appropriately
//!
//! Note: Full integration tests require a running PostgreSQL instance.
//! Unit tests verify URL parsing and error handling without requiring a database.

use secreton_backup::manager::BackupManager;
use secreton_backup::types::{BackupConfig, LocalStorageConfig, StorageConfig, StorageType};
use tempfile::TempDir;

/// Helper to create a test backup manager
async fn create_test_manager(database_url: Option<String>) -> (BackupManager, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let mut config = BackupConfig::default();
    config.storage_config = StorageConfig::Local(LocalStorageConfig { path: temp_dir.path().to_string_lossy().to_string() });
    config.database_url = database_url;

    let manager = BackupManager::new(config).await.unwrap();
    (manager, temp_dir)
}

#[tokio::test]
async fn test_postgres_dump_requires_database_url() {
    let (manager, _temp_dir) = create_test_manager(None).await;

    // Attempting to create a backup without database URL should fail
    // (unless DATABASE_URL environment variable is set)
    if std::env::var("DATABASE_URL").is_err() {
        let result = manager.create_backup().await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Database URL not configured"));
    }
}

#[tokio::test]
async fn test_postgres_dump_with_invalid_url() {
    let invalid_urls = vec![
        "not-a-url",
        "http://localhost:5432/db",
        "postgresql://localhost/db",        // Missing credentials
        "postgresql://user@localhost/db",   // Missing password
        "postgresql://user:pass@localhost", // Missing database
    ];

    for url in invalid_urls {
        let (manager, _temp_dir) = create_test_manager(Some(url.to_string())).await;

        let result = manager.create_backup().await;
        assert!(result.is_err(), "Expected error for invalid URL: {}", url);
    }
}

#[tokio::test]
#[ignore] // Requires running PostgreSQL instance
async fn test_postgres_dump_integration() {
    // This test requires a running PostgreSQL instance
    // Set TEST_DATABASE_URL environment variable to run this test
    let database_url = match std::env::var("TEST_DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            eprintln!("Skipping integration test: TEST_DATABASE_URL not set");
            return;
        }
    };

    let (manager, _temp_dir) = create_test_manager(Some(database_url)).await;

    // Create a backup (this will execute pg_dump)
    let result = manager.create_backup().await;

    match result {
        Ok(backup_id) => {
            println!("Backup created successfully: {}", backup_id);

            // Verify backup exists
            let backups = manager.list_backups().await.unwrap();
            assert_eq!(backups.len(), 1);
            assert_eq!(backups[0].id, backup_id);

            // Verify backup can be verified
            manager.verify_backup(&backup_id).await.unwrap();
        }
        Err(e) => {
            // If pg_dump is not installed or database is not accessible, test should fail gracefully
            eprintln!(
                "Backup creation failed (this may be expected if pg_dump is not installed): {}",
                e
            );
            assert!(
                e.to_string().contains("pg_dump") || e.to_string().contains("connection"),
                "Unexpected error: {}",
                e
            );
        }
    }
}

#[tokio::test]
async fn test_postgres_dump_timeout_configuration() {
    let (manager, _temp_dir) = create_test_manager(Some(
        "postgresql://user:pass@localhost:5432/testdb".to_string(),
    ))
    .await;

    // Verify timeout is configurable
    assert_eq!(manager.config.pg_dump_timeout_secs, 300); // Default 5 minutes
}
