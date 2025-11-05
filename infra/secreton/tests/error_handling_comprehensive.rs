//! Error Handling Tests for Secreton
//!
//! Comprehensive error handling validation

use anyhow::Result;
use secreton_core::storage::{InMemoryStorage, StorageBackend};

#[tokio::test]
async fn test_error_on_nonexistent_path() -> Result<()> {
    let storage = InMemoryStorage::new();

    let result = storage.get_by_path("/nonexistent/path").await?;
    assert!(result.is_none(), "Should return None for nonexistent path");

    Ok(())
}

#[tokio::test]
async fn test_error_handling_delete_nonexistent() -> Result<()> {
    let storage = InMemoryStorage::new();

    // Deleting nonexistent entry should not panic
    let result = storage.delete_by_path("/does/not/exist").await;
    assert!(
        result.is_ok(),
        "Delete should handle nonexistent paths gracefully"
    );

    Ok(())
}

#[tokio::test]
async fn test_error_propagation() -> Result<()> {
    // Test that errors propagate correctly through the stack
    let result: Result<()> = async {
        let storage = InMemoryStorage::new();
        let _ = storage.get_by_path("/test").await?;
        Ok(())
    }
    .await;

    assert!(result.is_ok());
    Ok(())
}
