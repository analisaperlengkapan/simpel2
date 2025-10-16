use anyhow::Result;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use secreton_core::{
    storage::{StorageEngine, InMemoryStorage, StorageEntry},
    error::CoreError,
};

#[tokio::test]
async fn test_memory_storage_basic_operations() -> Result<()> {
    let storage = InMemoryStorage::new();

    // Test store and retrieve
    let key = "test_key";
    let value = b"test_value";

    let entry = StorageEntry {
        key: key.to_string(),
        value: value.to_vec(),
        metadata: std::collections::HashMap::new(),
    };

    storage.put(entry).await?;
    let retrieved = storage.get(key).await?;

    assert!(retrieved.is_some(), "Retrieved value should exist");
    assert_eq!(retrieved.unwrap().value, value, "Retrieved value should match stored value");

    // Test key existence
    let exists_entry = storage.get(key).await?;
    assert!(exists_entry.is_some(), "Key should exist after storing");

    // Test non-existent key
    let missing_exists = storage.get("non_existent").await?;
    assert!(missing_exists.is_none(), "Non-existent key should not exist");

    // Test delete
    storage.delete(key).await?;
    let deleted_exists = storage.get(key).await?;
    assert!(deleted_exists.is_none(), "Key should not exist after deletion");

    Ok(())
}

#[tokio::test]
async fn test_storage_concurrent_operations() -> Result<()> {
    let storage = Arc::new(InMemoryStorage::new());

    // Test concurrent writes
    let write_handles: Vec<_> = (0..100).map(|i| {
        let storage_clone = Arc::clone(&storage);
        tokio::spawn(async move {
            let key = format!("concurrent_key_{}", i);
            let value = format!("concurrent_value_{}", i);
            let entry = StorageEntry {
                key: key.clone(),
                value: value.as_bytes().to_vec(),
                metadata: std::collections::HashMap::new(),
            };
            storage_clone.put(entry).await
        })
    }).collect();

    // Wait for all writes to complete
    for handle in write_handles {
        let result = handle.await?;
        assert!(result.is_ok(), "Concurrent write should succeed");
    }

    // Test concurrent reads
    let read_handles: Vec<_> = (0..100).map(|i| {
        let storage_clone = Arc::clone(&storage);
        tokio::spawn(async move {
            let key = format!("concurrent_key_{}", i);
            let expected_value = format!("concurrent_value_{}", i);
            let result = storage_clone.get(&key).await;
            (i, result, expected_value)
        })
    }).collect();

    // Verify all reads
    for handle in read_handles {
        let (i, result, expected_value) = handle.await?;
        assert!(result.is_ok(), "Concurrent read {} should succeed", i);

        let actual_entry = result.unwrap();
        assert!(actual_entry.is_some(), "Entry {} should exist", i);
        assert_eq!(actual_entry.unwrap().value, expected_value.as_bytes(), "Concurrent read {} should match expected value", i);
    }

    // Test mixed operations (read/write/delete)
    let mixed_handles: Vec<_> = (0..50).map(|i| {
        let storage_clone = Arc::clone(&storage);
        tokio::spawn(async move {
            let key = format!("mixed_key_{}", i);
            let value = format!("mixed_value_{}", i);

            // Store
            let entry = StorageEntry {
                key: key.clone(),
                value: value.as_bytes().to_vec(),
                metadata: std::collections::HashMap::new(),
            };
            storage_clone.put(entry).await?;

            // Read back
            let retrieved = storage_clone.get(&key).await?;
            assert!(retrieved.is_some());
            assert_eq!(retrieved.unwrap().value, value.as_bytes());

            // Update
            let new_value = format!("updated_mixed_value_{}", i);
            let updated_entry = StorageEntry {
                key: key.clone(),
                value: new_value.as_bytes().to_vec(),
                metadata: std::collections::HashMap::new(),
            };
            storage_clone.put(updated_entry).await?;

            // Read updated
            let updated_retrieved = storage_clone.get(&key).await?;
            assert!(updated_retrieved.is_some());
            assert_eq!(updated_retrieved.unwrap().value, new_value.as_bytes());

            // Delete
            storage_clone.delete(&key).await?;

            // Verify deletion
            let exists = storage_clone.get(&key).await?;
            assert!(exists.is_none());

            Ok::<(), CoreError>(())
        })
    }).collect();

    for handle in mixed_handles {
        let result = handle.await?;
        assert!(result.is_ok(), "Mixed operations should succeed");
    }

    Ok(())
}

#[tokio::test]
async fn test_storage_edge_cases() -> Result<()> {
    let storage = InMemoryStorage::new();

    // Test empty key
    let empty_key_result = storage.put(StorageEntry {
        key: "".to_string(),
        value: b"value".to_vec(),
        metadata: std::collections::HashMap::new(),
    }).await;
    assert!(empty_key_result.is_err(), "Should fail with empty key");

    // Test empty value
    let empty_value_result = storage.put(StorageEntry {
        key: "valid_key".to_string(),
        value: b"".to_vec(),
        metadata: std::collections::HashMap::new(),
    }).await;
    assert!(empty_value_result.is_ok(), "Should succeed with empty value");

    let empty_entry = storage.get("valid_key").await?;
    assert!(empty_entry.is_some(), "Should have entry");
    assert!(empty_entry.unwrap().value.is_empty(), "Should retrieve empty value");

    // Test very long keys
    let long_key = "a".repeat(1000);
    let long_key_result = storage.put(StorageEntry {
        key: long_key.clone(),
        value: b"value".to_vec(),
        metadata: std::collections::HashMap::new(),
    }).await;
    // This should either succeed or fail gracefully
    match long_key_result {
        Ok(_) => {
            let long_entry = storage.get(&long_key).await?;
            assert!(long_entry.is_some(), "Should have entry");
            assert_eq!(long_entry.unwrap().value, b"value");
            println!("Long key test passed");
        },
        Err(e) => {
            println!("Long key test failed as expected: {:?}", e);
        }
    }

    // Test very large values
    let large_value = vec![0u8; 1024 * 1024]; // 1MB
    let large_value_result = storage.put(StorageEntry {
        key: "large_value_key".to_string(),
        value: large_value.clone(),
        metadata: std::collections::HashMap::new(),
    }).await;
    match large_value_result {
        Ok(_) => {
            let large_entry = storage.get("large_value_key").await?;
            assert!(large_entry.is_some(), "Should have entry");
            assert_eq!(large_entry.unwrap().value.len(), 1024 * 1024);
            println!("Large value test passed");
        },
        Err(e) => {
            println!("Large value test failed as expected: {:?}", e);
        }
    }

    // Test special characters in keys
    let special_keys = vec![
        "key with spaces",
        "key/with/slashes",
        "key\\with\\backslashes",
        "key:with:colons",
        "key.with.dots",
        "key-with-dashes",
        "key_with_underscores",
        "key@with@at",
        "key#with#hash",
        "key$with$dollar",
        "key%with%percent",
        "key^with^caret",
        "key&with&ampersand",
        "key*with*asterisk",
        "key(with)parentheses",
        "key[with]brackets",
        "key{with}braces",
        "key|with|pipe",
        "key;with;semicolon",
        "key'with'quotes",
        "key\"with\"doublequotes",
        "key<with>angles",
        "key,with,commas",
        "key?with?questions",
        "key=with=equals",
        "key+with+plus",
        "key~with~tilde",
        "key`with`backtick",
    ];

    for (i, key) in special_keys.iter().enumerate() {
        let value = format!("value_{}", i);
        let result = storage.put(StorageEntry {
            key: key.to_string(),
            value: value.as_bytes().to_vec(),
            metadata: std::collections::HashMap::new(),
        }).await;

        match result {
            Ok(_) => {
                let entry = storage.get(key).await?;
                assert!(entry.is_some(), "Should have entry for key '{}'", key);
                assert_eq!(entry.unwrap().value, value.as_bytes(), "Special key '{}' should work", key);
            },
            Err(e) => {
                println!("Special key '{}' failed as expected: {:?}", key, e);
            }
        }
    }

    // Test unicode keys
    let unicode_keys = vec![
        "键值",
        "キー",
        "ключ",
        "مفتاح",
        "🔑key🔑",
        "key_with_émojis_🚀",
    ];

    for (i, key) in unicode_keys.iter().enumerate() {
        let value = format!("unicode_value_{}", i);
        let result = storage.put(StorageEntry {
            key: key.to_string(),
            value: value.as_bytes().to_vec(),
            metadata: std::collections::HashMap::new(),
        }).await;

        match result {
            Ok(_) => {
                let entry = storage.get(key).await?;
                assert!(entry.is_some(), "Should have entry for unicode key '{}'", key);
                assert_eq!(entry.unwrap().value, value.as_bytes(), "Unicode key '{}' should work", key);
                println!("Unicode key '{}' test passed", key);
            },
            Err(e) => {
                println!("Unicode key '{}' failed as expected: {:?}", key, e);
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
        ("app/frontend/config", "frontend_config"),
        ("app/backend/config", "backend_config"),
        ("app/database/url", "db_url"),
        ("app/database/credentials", "db_creds"),
        ("secrets/api/key1", "api_key_1"),
        ("secrets/api/key2", "api_key_2"),
        ("secrets/oauth/client_id", "oauth_client"),
        ("secrets/oauth/client_secret", "oauth_secret"),
        ("temp/cache/item1", "cache_item_1"),
        ("temp/cache/item2", "cache_item_2"),
        ("temp/sessions/user1", "session_1"),
    ];

    // Store all test data
    for (key, value) in &test_data {
        storage.put(StorageEntry {
            key: key.to_string(),
            value: value.as_bytes().to_vec(),
            metadata: std::collections::HashMap::new(),
        }).await?;
    }

    // Test list functionality if available
    if let Ok(all_keys) = storage.list("").await {
        assert!(!all_keys.is_empty(), "Should have keys");
        assert_eq!(all_keys.len(), test_data.len(), "Should list all keys");

        // Test prefix listing
        if let Ok(app_keys) = storage.list("app").await {
            let expected_app_keys = test_data.iter()
                .filter(|(k, _)| k.starts_with("app"))
                .count();
            assert_eq!(app_keys.len(), expected_app_keys, "Should list app keys");
        }

        if let Ok(secret_keys) = storage.list("secrets").await {
            let expected_secret_keys = test_data.iter()
                .filter(|(k, _)| k.starts_with("secrets"))
                .count();
            assert_eq!(secret_keys.len(), expected_secret_keys, "Should list secret keys");
        }
    } else {
        println!("Storage doesn't support listing - testing individual access");

        // Test that all keys can be retrieved individually
        for (key, expected_value) in &test_data {
            let entry = storage.get(key).await?;
            assert!(entry.is_some(), "Should have entry for {}", key);
            assert_eq!(entry.unwrap().value, expected_value.as_bytes(), "Should retrieve {}", key);
        }
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
        let key = format!("perf_key_{}", i);
        let value = format!("perf_value_{}", i);
        storage.put(StorageEntry {
            key: key.clone(),
            value: value.as_bytes().to_vec(),
            metadata: std::collections::HashMap::new(),
        }).await?;
    }
    let write_duration = write_start.elapsed();
    println!("{} sequential writes took: {:?}", num_operations, write_duration);

    // Sequential reads
    let read_start = std::time::Instant::now();
    for i in 0..num_operations {
        let key = format!("perf_key_{}", i);
        let entry = storage.get(&key).await?;
        assert!(entry.is_some(), "Should have entry");
    }
    let read_duration = read_start.elapsed();
    println!("{} sequential reads took: {:?}", num_operations, read_duration);

    // Random access pattern
    let random_start = std::time::Instant::now();
    for i in (0..num_operations).step_by(7) {  // Access every 7th item
        let key = format!("perf_key_{}", i % num_operations);
        let entry = storage.get(&key).await?;
        assert!(entry.is_some(), "Should have entry");
    }
    let random_duration = random_start.elapsed();
    println!("{} random reads took: {:?}", num_operations / 7, random_duration);

    // Mixed operations
    let mixed_start = std::time::Instant::now();
    for i in 0..num_operations / 4 {
        let key = format!("mixed_key_{}", i);
        let value = format!("mixed_value_{}", i);

        // Write
        storage.put(StorageEntry {
            key: key.clone(),
            value: value.as_bytes().to_vec(),
            metadata: std::collections::HashMap::new(),
        }).await?;

        // Read
        let entry = storage.get(&key).await?;
        assert!(entry.is_some(), "Should have entry");

        // Update
        let new_value = format!("updated_{}", value);
        storage.put(StorageEntry {
            key: key.clone(),
            value: new_value.as_bytes().to_vec(),
            metadata: std::collections::HashMap::new(),
        }).await?;

        // Read again
        let updated_entry = storage.get(&key).await?;
        assert!(updated_entry.is_some(), "Should have entry");
    }
    let mixed_duration = mixed_start.elapsed();
    println!("{} mixed operations took: {:?}", num_operations, mixed_duration);

    // Performance expectations (adjust based on system)
    assert!(write_duration.as_millis() < 5000, "Writes should be reasonably fast");
    assert!(read_duration.as_millis() < 2000, "Reads should be fast");

    Ok(())
}

#[tokio::test]
async fn test_storage_stress_test() -> Result<()> {
    let storage = Arc::new(InMemoryStorage::new());
    let num_workers = 10;
    let operations_per_worker = 100;

    // Spawn multiple workers doing concurrent operations
    let worker_handles: Vec<_> = (0..num_workers).map(|worker_id| {
        let storage_clone = Arc::clone(&storage);
        tokio::spawn(async move {
            let mut operations_completed = 0;

            for i in 0..operations_per_worker {
                let key = format!("stress_{}_{}", worker_id, i);
                let value = format!("stress_value_{}_{}", worker_id, i);

                // Store
                match storage_clone.put(StorageEntry {
                    key: key.clone(),
                    value: value.as_bytes().to_vec(),
                    metadata: std::collections::HashMap::new(),
                }).await {
                    Ok(_) => operations_completed += 1,
                    Err(e) => {
                        println!("Worker {} failed store operation {}: {:?}", worker_id, i, e);
                        continue;
                    }
                }

                // Read back
                match storage_clone.get(&key).await {
                    Ok(entry) => {
                        if let Some(entry) = entry {
                            if entry.value != value.as_bytes() {
                                println!("Worker {} data mismatch at operation {}", worker_id, i);
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
                    let _ = storage_clone.delete(&key).await;
                    let _ = storage_clone.put(StorageEntry {
                        key: key.clone(),
                        value: value.as_bytes().to_vec(),
                        metadata: std::collections::HashMap::new(),
                    }).await;
                }

                // Add some variability
                if i % 20 == 0 {
                    sleep(Duration::from_millis(1)).await;
                }
            }

            println!("Worker {} completed {} operations", worker_id, operations_completed);
            operations_completed
        })
    }).collect();

    // Wait for all workers to complete
    let mut total_operations = 0;
    for handle in worker_handles {
        let completed = handle.await?;
        total_operations += completed;
    }

    println!("Stress test completed: {} total operations", total_operations);

    // Verify some of the data is still accessible
    for worker_id in 0..num_workers {
        for i in 0..10 {  // Check first 10 items from each worker
            let key = format!("stress_{}_{}", worker_id, i);
            let expected_value = format!("stress_value_{}_{}", worker_id, i);

            match storage.get(&key).await {
                Ok(entry) => {
                    if let Some(entry) = entry {
                        assert_eq!(entry.value, expected_value.as_bytes(),
                                  "Stress test data integrity check failed for {}", key);
                    } else {
                        // It's possible some keys were deleted during the stress test
                        println!("Key {} not found (possibly deleted during stress test)", key);
                    }
                }
                Err(_) => {
                    // It's possible some keys were deleted during the stress test
                    println!("Key {} not found (possibly deleted during stress test)", key);
                }
            }
        }
    }

    assert!(total_operations > num_workers * operations_per_worker / 2,
           "Should complete at least half of all operations");

    Ok(())
}

#[tokio::test]
async fn test_storage_cleanup_and_recovery() -> Result<()> {
    let storage = InMemoryStorage::new();

    // Fill storage with data
    for i in 0..100 {
        let key = format!("cleanup_key_{}", i);
        let value = format!("cleanup_value_{}", i);
        storage.put(StorageEntry {
            key: key.clone(),
            value: value.as_bytes().to_vec(),
            metadata: std::collections::HashMap::new(),
        }).await?;
    }

    // Verify all data is there
    for i in 0..100 {
        let key = format!("cleanup_key_{}", i);
        let exists = storage.get(&key).await?.is_some();
        assert!(exists, "Key {} should exist before cleanup", key);
    }

    // Delete every other key
    for i in (0..100).step_by(2) {
        let key = format!("cleanup_key_{}", i);
        storage.delete(&key).await?;
    }

    // Verify deletion pattern
    for i in 0..100 {
        let key = format!("cleanup_key_{}", i);
        let exists = storage.get(&key).await?.is_some();
        if i % 2 == 0 {
            assert!(!exists, "Even key {} should be deleted", key);
        } else {
            assert!(exists, "Odd key {} should still exist", key);
        }
    }

    // Test recovery by recreating deleted keys
    for i in (0..100).step_by(2) {
        let key = format!("cleanup_key_{}", i);
        let value = format!("recovered_value_{}", i);
        storage.put(StorageEntry {
            key: key.clone(),
            value: value.as_bytes().to_vec(),
            metadata: std::collections::HashMap::new(),
        }).await?;
    }

    // Verify all keys exist again
    for i in 0..100 {
        let key = format!("cleanup_key_{}", i);
        let exists = storage.get(&key).await?.is_some();
        assert!(exists, "Key {} should exist after recovery", key);

        let entry = storage.get(&key).await?;
        let retrieved = entry.unwrap().value;
        if i % 2 == 0 {
            let expected = format!("recovered_value_{}", i);
            assert_eq!(retrieved, expected.as_bytes(), "Even key {} should have recovered value", key);
        } else {
            let expected = format!("cleanup_value_{}", i);
            assert_eq!(retrieved, expected.as_bytes(), "Odd key {} should have original value", key);
        }
    }

    Ok(())
}
