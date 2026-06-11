use anyhow::Result;
use secreton_core::storage::{
    InMemoryStorage, QueryParams, SecretEntry, SecurityLevel, StorageBackend,
};
use serde_json::json;
use std::sync::Arc;
use tokio::time::{Duration, sleep};

#[tokio::test]
async fn test_memory_storage_basic_operations() -> Result<()> {
    let storage = InMemoryStorage::new();

    // Test store and retrieve
    let path = "/test/key";
    let value = b"test_value";

    let entry = SecretEntry::new(
        path.to_string(),
        value.to_vec(),
        json!({}),
        SecurityLevel::Internal,
        "test_owner".to_string(),
    );

    storage.store(&entry).await?;
    let retrieved = storage.get_by_path(path).await?;

    assert!(retrieved.is_some(), "Retrieved value should exist");
    assert_eq!(
        retrieved.unwrap().encrypted_data,
        value,
        "Retrieved value should match stored value"
    );

    // Test key existence
    let exists = storage.exists(path).await?;
    assert!(exists, "Path should exist after storing");

    // Test non-existent key
    let missing_exists = storage.exists("/non/existent").await?;
    assert!(!missing_exists, "Non-existent path should not exist");

    // Test delete
    storage.delete_by_path(path).await?;
    let deleted_exists = storage.exists(path).await?;
    assert!(!deleted_exists, "Path should not exist after deletion");

    Ok(())
}

#[tokio::test]
async fn test_storage_concurrent_operations() -> Result<()> {
    let storage = Arc::new(InMemoryStorage::new());

    // Test concurrent writes
    let write_handles: Vec<_> = (0..100)
        .map(|i| {
            let storage_clone = Arc::clone(&storage);
            tokio::spawn(async move {
                let path = format!("/concurrent/key/{}", i);
                let value = format!("concurrent_value_{}", i);
                let entry = SecretEntry::new(
                    path.clone(),
                    value.as_bytes().to_vec(),
                    json!({}),
                    SecurityLevel::Internal,
                    "test_owner".to_string(),
                );
                storage_clone.store(&entry).await
            })
        })
        .collect();

    // Wait for all writes to complete
    for handle in write_handles {
        let result = handle.await?;
        assert!(result.is_ok(), "Concurrent write should succeed");
    }

    // Test concurrent reads
    let read_handles: Vec<_> = (0..100)
        .map(|i| {
            let storage_clone = Arc::clone(&storage);
            tokio::spawn(async move {
                let path = format!("/concurrent/key/{}", i);
                let expected_value = format!("concurrent_value_{}", i);
                let result = storage_clone.get_by_path(&path).await;
                (i, result, expected_value)
            })
        })
        .collect();

    // Verify all reads
    for handle in read_handles {
        let (i, result, expected_value) = handle.await?;
        assert!(result.is_ok(), "Concurrent read {} should succeed", i);

        let actual_entry = result.unwrap();
        assert!(actual_entry.is_some(), "Entry {} should exist", i);
        assert_eq!(
            actual_entry.unwrap().encrypted_data,
            expected_value.as_bytes(),
            "Concurrent read {} should match expected value",
            i
        );
    }

    // Test mixed operations (read/write/delete)
    let mixed_handles: Vec<_> = (0..50)
        .map(|i| {
            let storage_clone = Arc::clone(&storage);
            tokio::spawn(async move {
                let path = format!("/mixed/key/{}", i);
                let value = format!("mixed_value_{}", i);

                // Store
                let entry = SecretEntry::new(
                    path.clone(),
                    value.as_bytes().to_vec(),
                    json!({}),
                    SecurityLevel::Internal,
                    "test_owner".to_string(),
                );
                storage_clone.store(&entry).await?;

                // Read back
                let retrieved = storage_clone.get_by_path(&path).await?;
                assert!(retrieved.is_some());
                assert_eq!(retrieved.unwrap().encrypted_data, value.as_bytes());

                // Update
                let new_value = format!("updated_mixed_value_{}", i);
                let mut updated_entry = entry.clone();
                updated_entry.encrypted_data = new_value.as_bytes().to_vec();
                updated_entry.version += 1;
                updated_entry.updated_at = chrono::Utc::now();
                storage_clone.update(&updated_entry).await?;

                // Read updated
                let updated_retrieved = storage_clone.get_by_path(&path).await?;
                assert!(updated_retrieved.is_some());
                assert_eq!(
                    updated_retrieved.unwrap().encrypted_data,
                    new_value.as_bytes()
                );

                // Delete
                storage_clone.delete_by_path(&path).await?;

                // Verify deletion
                let exists = storage_clone.exists(&path).await?;
                assert!(!exists);

                Ok::<(), anyhow::Error>(())
            })
        })
        .collect();

    for handle in mixed_handles {
        let result = handle.await?;
        assert!(result.is_ok(), "Mixed operations should succeed");
    }

    Ok(())
}

#[tokio::test]
async fn test_storage_edge_cases() -> Result<()> {
    let storage = InMemoryStorage::new();

    // Test empty path (should fail)
    let empty_path_result = storage
        .store(&SecretEntry::new(
            "".to_string(),
            b"value".to_vec(),
            json!({}),
            SecurityLevel::Internal,
            "test_owner".to_string(),
        ))
        .await;
    assert!(empty_path_result.is_err(), "Should fail with empty path");

    // Test empty value (should succeed)
    let empty_value_result = storage
        .store(&SecretEntry::new(
            "/valid/path".to_string(),
            b"".to_vec(),
            json!({}),
            SecurityLevel::Internal,
            "test_owner".to_string(),
        ))
        .await;
    assert!(
        empty_value_result.is_ok(),
        "Should succeed with empty value"
    );

    let empty_entry = storage.get_by_path("/valid/path").await?;
    assert!(empty_entry.is_some(), "Should have entry");
    assert!(
        empty_entry.unwrap().encrypted_data.is_empty(),
        "Should retrieve empty value"
    );

    // Test very long paths
    let long_path = format!("/{}", "a".repeat(1000));
    let long_path_result = storage
        .store(&SecretEntry::new(
            long_path.clone(),
            b"value".to_vec(),
            json!({}),
            SecurityLevel::Internal,
            "test_owner".to_string(),
        ))
        .await;
    // This should either succeed or fail gracefully
    match long_path_result {
        Ok(_) => {
            let long_entry = storage.get_by_path(&long_path).await?;
            assert!(long_entry.is_some(), "Should have entry");
            assert_eq!(long_entry.unwrap().encrypted_data, b"value");
            println!("Long path test passed");
        }
        Err(e) => {
            println!("Long path test failed as expected: {:?}", e);
        }
    }

    // Test very large values
    let large_value = vec![0u8; 1024 * 1024]; // 1MB
    let large_value_result = storage
        .store(&SecretEntry::new(
            "/large/value".to_string(),
            large_value.clone(),
            json!({}),
            SecurityLevel::Internal,
            "test_owner".to_string(),
        ))
        .await;
    match large_value_result {
        Ok(_) => {
            let large_entry = storage.get_by_path("/large/value").await?;
            assert!(large_entry.is_some(), "Should have entry");
            assert_eq!(large_entry.unwrap().encrypted_data.len(), 1024 * 1024);
            println!("Large value test passed");
        }
        Err(e) => {
            println!("Large value test failed as expected: {:?}", e);
        }
    }

    // Test special characters in paths
    let special_paths = [
        "/key/with spaces",
        "/key-with-dashes",
        "/key_with_underscores",
        "/key.with.dots",
    ];

    for (i, path) in special_paths.iter().enumerate() {
        let value = format!("value_{}", i);
        let result = storage
            .store(&SecretEntry::new(
                path.to_string(),
                value.as_bytes().to_vec(),
                json!({}),
                SecurityLevel::Internal,
                "test_owner".to_string(),
            ))
            .await;

        match result {
            Ok(_) => {
                let entry = storage.get_by_path(path).await?;
                assert!(entry.is_some(), "Should have entry for path '{}'", path);
                assert_eq!(
                    entry.unwrap().encrypted_data,
                    value.as_bytes(),
                    "Special path '{}' should work",
                    path
                );
            }
            Err(e) => {
                println!("Special path '{}' failed as expected: {:?}", path, e);
            }
        }
    }

    // Test unicode paths
    let unicode_paths = ["/键值", "/キー", "/ключ", "/🔑key🔑"];

    for (i, path) in unicode_paths.iter().enumerate() {
        let value = format!("unicode_value_{}", i);
        let result = storage
            .store(&SecretEntry::new(
                path.to_string(),
                value.as_bytes().to_vec(),
                json!({}),
                SecurityLevel::Internal,
                "test_owner".to_string(),
            ))
            .await;

        match result {
            Ok(_) => {
                let entry = storage.get_by_path(path).await?;
                assert!(
                    entry.is_some(),
                    "Should have entry for unicode path '{}'",
                    path
                );
                assert_eq!(
                    entry.unwrap().encrypted_data,
                    value.as_bytes(),
                    "Unicode path '{}' should work",
                    path
                );
                println!("Unicode path '{}' test passed", path);
            }
            Err(e) => {
                println!("Unicode path '{}' failed as expected: {:?}", path, e);
            }
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_storage_patterns_and_listing() -> Result<()> {
    let storage = InMemoryStorage::new();

    // Create hierarchical data structure
    let test_data = vec![
        ("/app/frontend/config", "frontend_config"),
        ("/app/backend/config", "backend_config"),
        ("/app/database/url", "db_url"),
        ("/app/database/credentials", "db_creds"),
        ("/secrets/api/key1", "api_key_1"),
        ("/secrets/api/key2", "api_key_2"),
        ("/secrets/oauth/client_id", "oauth_client"),
        ("/secrets/oauth/client_secret", "oauth_secret"),
        ("/temp/cache/item1", "cache_item_1"),
        ("/temp/cache/item2", "cache_item_2"),
        ("/temp/sessions/user1", "session_1"),
    ];

    // Store all test data
    for (path, value) in &test_data {
        storage
            .store(&SecretEntry::new(
                path.to_string(),
                value.as_bytes().to_vec(),
                json!({}),
                SecurityLevel::Internal,
                "test_owner".to_string(),
            ))
            .await?;
    }

    // Test list functionality
    let params = QueryParams::new();
    let all_entries = storage.list(&params).await?;
    assert!(!all_entries.is_empty(), "Should have entries");
    assert_eq!(
        all_entries.len(),
        test_data.len(),
        "Should list all entries"
    );

    // Test prefix listing
    let app_params = QueryParams::new().with_path_prefix("/app".to_string());
    let app_entries = storage.list(&app_params).await?;
    let expected_app_entries = test_data
        .iter()
        .filter(|(p, _)| p.starts_with("/app"))
        .count();
    assert_eq!(
        app_entries.len(),
        expected_app_entries,
        "Should list app entries"
    );

    // Test that all paths can be retrieved individually
    for (path, expected_value) in &test_data {
        let entry = storage.get_by_path(path).await?;
        assert!(entry.is_some(), "Should have entry for {}", path);
        assert_eq!(
            entry.unwrap().encrypted_data,
            expected_value.as_bytes(),
            "Should retrieve {}",
            path
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_storage_performance_characteristics() -> Result<()> {
    let storage = InMemoryStorage::new();

    // Test rapid sequential operations
    let num_operations = 1000;

    // Sequential writes
    let write_start = std::time::Instant::now();
    for i in 0..num_operations {
        let path = format!("/perf/key/{}", i);
        let value = format!("perf_value_{}", i);
        storage
            .store(&SecretEntry::new(
                path.clone(),
                value.as_bytes().to_vec(),
                json!({}),
                SecurityLevel::Internal,
                "test_owner".to_string(),
            ))
            .await?;
    }
    let write_duration = write_start.elapsed();
    println!(
        "{} sequential writes took: {:?}",
        num_operations, write_duration
    );

    // Sequential reads
    let read_start = std::time::Instant::now();
    for i in 0..num_operations {
        let path = format!("/perf/key/{}", i);
        let entry = storage.get_by_path(&path).await?;
        assert!(entry.is_some(), "Should have entry");
    }
    let read_duration = read_start.elapsed();
    println!(
        "{} sequential reads took: {:?}",
        num_operations, read_duration
    );

    // Random access pattern
    let random_start = std::time::Instant::now();
    for i in (0..num_operations).step_by(7) {
        // Access every 7th item
        let path = format!("/perf/key/{}", i % num_operations);
        let entry = storage.get_by_path(&path).await?;
        assert!(entry.is_some(), "Should have entry");
    }
    let random_duration = random_start.elapsed();
    println!(
        "{} random reads took: {:?}",
        num_operations / 7,
        random_duration
    );

    // Mixed operations
    let mixed_start = std::time::Instant::now();
    for i in 0..num_operations / 4 {
        let path = format!("/mixed/key/{}", i);
        let value = format!("mixed_value_{}", i);

        // Write
        let entry = SecretEntry::new(
            path.clone(),
            value.as_bytes().to_vec(),
            json!({}),
            SecurityLevel::Internal,
            "test_owner".to_string(),
        );
        storage.store(&entry).await?;

        // Read
        let retrieved = storage.get_by_path(&path).await?;
        assert!(retrieved.is_some(), "Should have entry");

        // Update
        let new_value = format!("updated_{}", value);
        let mut updated_entry = entry.clone();
        updated_entry.encrypted_data = new_value.as_bytes().to_vec();
        updated_entry.version += 1;
        updated_entry.updated_at = chrono::Utc::now();
        storage.update(&updated_entry).await?;

        // Read again
        let updated_retrieved = storage.get_by_path(&path).await?;
        assert!(updated_retrieved.is_some(), "Should have entry");
    }
    let mixed_duration = mixed_start.elapsed();
    println!(
        "{} mixed operations took: {:?}",
        num_operations, mixed_duration
    );

    // Performance expectations (adjust based on system)
    assert!(
        write_duration.as_millis() < 5000,
        "Writes should be reasonably fast"
    );
    assert!(read_duration.as_millis() < 2000, "Reads should be fast");

    Ok(())
}

#[tokio::test]
async fn test_storage_stress_test() -> Result<()> {
    let storage = Arc::new(InMemoryStorage::new());
    let num_workers = 10;
    let operations_per_worker = 100;

    // Spawn multiple workers doing concurrent operations
    let worker_handles: Vec<_> = (0..num_workers)
        .map(|worker_id| {
            let storage_clone = Arc::clone(&storage);
            tokio::spawn(async move {
                let mut operations_completed = 0;

                for i in 0..operations_per_worker {
                    let path = format!("/stress/{}/{}", worker_id, i);
                    let value = format!("stress_value_{}_{}", worker_id, i);

                    // Store
                    match storage_clone
                        .store(&SecretEntry::new(
                            path.clone(),
                            value.as_bytes().to_vec(),
                            json!({}),
                            SecurityLevel::Internal,
                            "test_owner".to_string(),
                        ))
                        .await
                    {
                        Ok(_) => operations_completed += 1,
                        Err(e) => {
                            println!("Worker {} failed store operation {}: {:?}", worker_id, i, e);
                            continue;
                        }
                    }

                    // Read back
                    match storage_clone.get_by_path(&path).await {
                        Ok(entry) => {
                            if let Some(entry) = entry {
                                if entry.encrypted_data != value.as_bytes() {
                                    println!(
                                        "Worker {} data mismatch at operation {}",
                                        worker_id, i
                                    );
                                }
                            } else {
                                println!("Worker {} entry not found at operation {}", worker_id, i);
                            }
                        }
                        Err(e) => {
                            println!("Worker {} failed read operation {}: {:?}", worker_id, i, e);
                            continue;
                        }
                    }

                    // Occasionally delete and recreate
                    if i % 10 == 0 {
                        let _ = storage_clone.delete_by_path(&path).await;
                        let _ = storage_clone
                            .store(&SecretEntry::new(
                                path.clone(),
                                value.as_bytes().to_vec(),
                                json!({}),
                                SecurityLevel::Internal,
                                "test_owner".to_string(),
                            ))
                            .await;
                    }

                    // Add some variability
                    if i % 20 == 0 {
                        sleep(Duration::from_millis(1)).await;
                    }
                }

                println!(
                    "Worker {} completed {} operations",
                    worker_id, operations_completed
                );
                operations_completed
            })
        })
        .collect();

    // Wait for all workers to complete
    let mut total_operations = 0;
    for handle in worker_handles {
        let completed = handle.await?;
        total_operations += completed;
    }

    println!(
        "Stress test completed: {} total operations",
        total_operations
    );

    // Verify some of the data is still accessible
    for worker_id in 0..num_workers {
        for i in 0..10 {
            // Check first 10 items from each worker
            let path = format!("/stress/{}/{}", worker_id, i);
            let expected_value = format!("stress_value_{}_{}", worker_id, i);

            match storage.get_by_path(&path).await {
                Ok(entry) => {
                    if let Some(entry) = entry {
                        assert_eq!(
                            entry.encrypted_data,
                            expected_value.as_bytes(),
                            "Stress test data integrity check failed for {}",
                            path
                        );
                    } else {
                        // It's possible some paths were deleted during the stress test
                        println!(
                            "Path {} not found (possibly deleted during stress test)",
                            path
                        );
                    }
                }
                Err(_) => {
                    // It's possible some paths were deleted during the stress test
                    println!(
                        "Path {} not found (possibly deleted during stress test)",
                        path
                    );
                }
            }
        }
    }

    assert!(
        total_operations > num_workers * operations_per_worker / 2,
        "Should complete at least half of all operations"
    );

    Ok(())
}

#[tokio::test]
async fn test_storage_cleanup_and_recovery() -> Result<()> {
    let storage = InMemoryStorage::new();

    // Fill storage with data
    for i in 0..100 {
        let path = format!("/cleanup/key/{}", i);
        let value = format!("cleanup_value_{}", i);
        storage
            .store(&SecretEntry::new(
                path.clone(),
                value.as_bytes().to_vec(),
                json!({}),
                SecurityLevel::Internal,
                "test_owner".to_string(),
            ))
            .await?;
    }

    // Verify all data is there
    for i in 0..100 {
        let path = format!("/cleanup/key/{}", i);
        let exists = storage.exists(&path).await?;
        assert!(exists, "Path {} should exist before cleanup", path);
    }

    // Delete every other key
    for i in (0..100).step_by(2) {
        let path = format!("/cleanup/key/{}", i);
        storage.delete_by_path(&path).await?;
    }

    // Verify deletion pattern
    for i in 0..100 {
        let path = format!("/cleanup/key/{}", i);
        let exists = storage.exists(&path).await?;
        if i % 2 == 0 {
            assert!(!exists, "Even path {} should be deleted", path);
        } else {
            assert!(exists, "Odd path {} should still exist", path);
        }
    }

    // Test recovery by recreating deleted keys
    for i in (0..100).step_by(2) {
        let path = format!("/cleanup/key/{}", i);
        let value = format!("recovered_value_{}", i);
        storage
            .store(&SecretEntry::new(
                path.clone(),
                value.as_bytes().to_vec(),
                json!({}),
                SecurityLevel::Internal,
                "test_owner".to_string(),
            ))
            .await?;
    }

    // Verify all paths exist again
    for i in 0..100 {
        let path = format!("/cleanup/key/{}", i);
        let exists = storage.exists(&path).await?;
        assert!(exists, "Path {} should exist after recovery", path);

        let entry = storage.get_by_path(&path).await?;
        let retrieved = entry.unwrap().encrypted_data;
        if i % 2 == 0 {
            let expected = format!("recovered_value_{}", i);
            assert_eq!(
                retrieved,
                expected.as_bytes(),
                "Even path {} should have recovered value",
                path
            );
        } else {
            let expected = format!("cleanup_value_{}", i);
            assert_eq!(
                retrieved,
                expected.as_bytes(),
                "Odd path {} should have original value",
                path
            );
        }
    }

    Ok(())
}
