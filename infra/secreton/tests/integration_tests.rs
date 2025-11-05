//! Integration Tests for Secreton
//!
//! End-to-end integration testing

use anyhow::Result;
use secreton_core::storage::{InMemoryStorage, SecurityLevel, StorageBackend, VaultEntry};
use std::sync::Arc;

#[tokio::test]
async fn test_basic_integration() -> Result<()> {
    let storage = InMemoryStorage::new();

    // Create entry
    let entry = VaultEntry::new(
        "/integration/test".to_string(),
        b"integration test data".to_vec(),
        serde_json::json!({"test": true}),
        SecurityLevel::Confidential,
        "integration_test".to_string(),
    );

    // Store
    storage.store(&entry).await?;

    // Retrieve
    let retrieved = storage.get_by_path("/integration/test").await?;
    assert!(retrieved.is_some());

    // Verify
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.path, "/integration/test");
    assert_eq!(retrieved.encrypted_data, b"integration test data");

    Ok(())
}

#[tokio::test]
async fn test_multi_storage_integration() -> Result<()> {
    let storage1 = Arc::new(InMemoryStorage::new());
    let storage2 = Arc::new(InMemoryStorage::new());

    // Test that multiple storage instances work independently
    let entry1 = VaultEntry::new(
        "/storage1/data".to_string(),
        b"data1".to_vec(),
        serde_json::json!({}),
        SecurityLevel::Internal,
        "test".to_string(),
    );

    let entry2 = VaultEntry::new(
        "/storage2/data".to_string(),
        b"data2".to_vec(),
        serde_json::json!({}),
        SecurityLevel::Internal,
        "test".to_string(),
    );

    storage1.store(&entry1).await?;
    storage2.store(&entry2).await?;

    assert!(storage1.exists("/storage1/data").await?);
    assert!(!storage1.exists("/storage2/data").await?);
    assert!(!storage2.exists("/storage1/data").await?);
    assert!(storage2.exists("/storage2/data").await?);

    Ok(())
}
