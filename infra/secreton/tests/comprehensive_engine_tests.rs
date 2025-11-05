//! Comprehensive Engine Tests for Secreton
//!
//! This module tests the core storage and engine functionality.

use anyhow::Result;
use secreton_core::storage::{InMemoryStorage, SecurityLevel, StorageBackend, VaultEntry};
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn test_storage_backend_creation() -> Result<()> {
    let storage = InMemoryStorage::new();
    assert!(Arc::strong_count(&Arc::new(storage)) == 1);
    Ok(())
}

#[tokio::test]
async fn test_vault_entry_creation() -> Result<()> {
    let entry = VaultEntry::new(
        "/test/path".to_string(),
        b"test data".to_vec(),
        json!({}),
        SecurityLevel::Internal,
        "test_owner".to_string(),
    );

    assert_eq!(entry.path, "/test/path");
    assert_eq!(entry.encrypted_data, b"test data");
    Ok(())
}

#[tokio::test]
async fn test_storage_store_and_retrieve() -> Result<()> {
    let storage = InMemoryStorage::new();

    let entry = VaultEntry::new(
        "/test/secret".to_string(),
        b"secret value".to_vec(),
        json!({"type": "test"}),
        SecurityLevel::Confidential,
        "test_owner".to_string(),
    );

    storage.store(&entry).await?;
    let retrieved = storage.get_by_path("/test/secret").await?;

    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.path, "/test/secret");
    assert_eq!(retrieved.encrypted_data, b"secret value");
    Ok(())
}

#[tokio::test]
async fn test_storage_delete() -> Result<()> {
    let storage = InMemoryStorage::new();

    let entry = VaultEntry::new(
        "/test/delete_me".to_string(),
        b"data".to_vec(),
        json!({}),
        SecurityLevel::Internal,
        "test_owner".to_string(),
    );

    storage.store(&entry).await?;
    assert!(storage.exists("/test/delete_me").await?);

    storage.delete_by_path("/test/delete_me").await?;
    assert!(!storage.exists("/test/delete_me").await?);

    Ok(())
}

#[tokio::test]
async fn test_storage_list() -> Result<()> {
    use secreton_core::storage::QueryParams;

    let storage = InMemoryStorage::new();

    for i in 0..5 {
        let entry = VaultEntry::new(
            format!("/test/item_{}", i),
            format!("data_{}", i).into_bytes(),
            json!({}),
            SecurityLevel::Internal,
            "test_owner".to_string(),
        );
        storage.store(&entry).await?;
    }

    let params = QueryParams::new().with_path_prefix("/test/".to_string());
    let list = storage.list(&params).await?;
    assert!(list.len() >= 5);

    Ok(())
}
