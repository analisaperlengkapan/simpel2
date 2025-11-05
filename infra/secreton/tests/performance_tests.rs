//! Performance Tests for Secreton
//!
//! Basic performance validation tests

use anyhow::Result;
use secreton_core::storage::{InMemoryStorage, SecurityLevel, StorageBackend, VaultEntry};
use std::time::Instant;

#[tokio::test]
async fn test_storage_performance_basic() -> Result<()> {
    let storage = InMemoryStorage::new();
    let start = Instant::now();

    // Store 100 entries
    for i in 0..100 {
        let entry = VaultEntry::new(
            format!("/perf/test_{}", i),
            vec![0u8; 1024], // 1KB data
            serde_json::json!({}),
            SecurityLevel::Internal,
            "test_owner".to_string(),
        );
        storage.store(&entry).await?;
    }

    let duration = start.elapsed();
    println!("✓ Stored 100 entries in {:?}", duration);

    // Should complete in reasonable time
    assert!(duration.as_secs() < 5, "Storage should be fast");

    Ok(())
}

#[tokio::test]
async fn test_retrieval_performance() -> Result<()> {
    let storage = InMemoryStorage::new();

    // Setup: Store entry
    let entry = VaultEntry::new(
        "/perf/retrieve".to_string(),
        vec![0u8; 10240], // 10KB data
        serde_json::json!({}),
        SecurityLevel::Internal,
        "test_owner".to_string(),
    );
    storage.store(&entry).await?;

    // Test: Retrieve multiple times
    let start = Instant::now();
    for _ in 0..100 {
        let _ = storage.get_by_path("/perf/retrieve").await?;
    }
    let duration = start.elapsed();

    println!("✓ Retrieved 100 times in {:?}", duration);
    assert!(duration.as_secs() < 2, "Retrieval should be fast");

    Ok(())
}
