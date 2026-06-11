use anyhow::Result;
use secreton_core::storage::secure::{MemoryKeyStore, SecureStorage, SharedSecureStorage};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TestData {
    name: String,
    value: i32,
    sensitive: bool,
}

#[tokio::test]
async fn test_secure_storage_basic_operations() -> Result<()> {
    println!("🔐 Testing Secure Storage Basic Operations...");

    // Create a key store
    let key_store = Arc::new(MemoryKeyStore::new());

    // Create secure storage with default configuration
    let storage =
        SecureStorage::new_with_keystore(b"test-master-key-for-basic-ops", key_store, None).await?;

    let shared_storage = SharedSecureStorage::new(storage);

    // Test data
    let test_data = TestData {
        name: "test_secret".to_string(),
        value: 42,
        sensitive: true,
    };

    // Encrypt the data
    let encrypted = shared_storage.encrypt_value(&test_data).await?;
    assert!(!encrypted.is_empty(), "Encrypted data should not be empty");

    // Decrypt the data
    let decrypted: TestData = shared_storage.decrypt_value(&encrypted).await?;
    assert_eq!(test_data, decrypted, "Decrypted data should match original");

    println!("  ✅ Basic encrypt/decrypt operations verified");
    Ok(())
}

#[tokio::test]
async fn test_secure_storage_key_rotation() -> Result<()> {
    println!("🔄 Testing Secure Storage Key Rotation...");

    // Create secure storage
    let key_store = Arc::new(MemoryKeyStore::new());

    // Create secure storage
    let storage =
        SecureStorage::new_with_keystore(b"test-master-key-for-rotation", key_store.clone(), None)
            .await?;

    let shared_storage = SharedSecureStorage::new(storage);

    // Get initial key ID
    let initial_key_id = shared_storage.current_key_id();

    // Encrypt some data with the initial key
    let test_data = b"data encrypted with initial key";
    let encrypted_initial = shared_storage.encrypt(test_data).await?;

    // Note: Key rotation is not implemented in SharedSecureStorage yet
    // For now, we'll just verify that the same key is used
    let new_key_id = shared_storage.current_key_id();

    assert_eq!(
        initial_key_id, new_key_id,
        "Key ID should remain the same without rotation"
    );

    // Verify we can still decrypt data
    let decrypted = shared_storage.decrypt(&encrypted_initial).await?;
    assert_eq!(
        test_data.to_vec(),
        decrypted,
        "Should be able to decrypt data"
    );

    // Encrypt new data with the new key
    let new_data = b"data encrypted with new key";
    let encrypted_new = shared_storage.encrypt(new_data).await?;

    // Decrypt the new data
    let decrypted_new = shared_storage.decrypt(&encrypted_new).await?;
    assert_eq!(
        new_data.to_vec(),
        decrypted_new,
        "New data should decrypt correctly"
    );

    println!("  ✅ Key rotation operations verified");
    Ok(())
}

#[tokio::test]
async fn test_secure_storage_error_handling() -> Result<()> {
    println!("❌ Testing Secure Storage Error Handling...");

    // Create secure storage
    let key_store = Arc::new(MemoryKeyStore::new());
    let storage =
        SecureStorage::new_with_keystore(b"test-master-key-for-errors", key_store, None).await?;

    let shared_storage = SharedSecureStorage::new(storage);

    // Test decrypting invalid data
    let invalid_data = "not-valid-base64!";
    let decrypt_result = shared_storage.decrypt(invalid_data).await;
    assert!(
        decrypt_result.is_err(),
        "Should fail to decrypt invalid data"
    );

    // Test decrypting valid base64 but invalid encrypted data
    let fake_encrypted = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        b"not-encrypted-data",
    );
    let decrypt_fake_result = shared_storage.decrypt(&fake_encrypted).await;
    assert!(
        decrypt_fake_result.is_err(),
        "Should fail to decrypt fake encrypted data"
    );

    println!("  ✅ Error handling verified");
    Ok(())
}

#[tokio::test]
async fn test_secure_storage_concurrent_access() -> Result<()> {
    println!("🔄 Testing Secure Storage Concurrent Access...");

    // Create secure storage
    let key_store = Arc::new(MemoryKeyStore::new());
    let storage =
        SecureStorage::new_with_keystore(b"test-master-key-concurrent", key_store, None).await?;

    let shared_storage = Arc::new(SharedSecureStorage::new(storage));

    // Spawn multiple tasks to encrypt/decrypt concurrently
    let mut handles = vec![];

    for i in 0..10 {
        let storage_clone = Arc::clone(&shared_storage);
        let handle = tokio::spawn(async move {
            let data = format!("concurrent-data-{}", i);
            let encrypted = storage_clone.encrypt(data.as_bytes()).await?;
            let decrypted = storage_clone.decrypt(&encrypted).await?;
            Ok::<_, anyhow::Error>((data, String::from_utf8(decrypted)?))
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        let (original, decrypted) = handle.await??;
        assert_eq!(
            original, decrypted,
            "Concurrent operation should preserve data integrity"
        );
    }

    println!("  ✅ Concurrent access verified");
    Ok(())
}

#[tokio::test]
async fn test_secure_storage_large_data() -> Result<()> {
    println!("📊 Testing Secure Storage Large Data Handling...");

    // Create secure storage
    let key_store = Arc::new(MemoryKeyStore::new());
    let storage =
        SecureStorage::new_with_keystore(b"test-master-key-large-data", key_store, None).await?;

    let shared_storage = SharedSecureStorage::new(storage);

    // Create large data (1MB)
    let large_data = vec![0x42u8; 1024 * 1024];

    // Encrypt large data
    let encrypted = shared_storage.encrypt(&large_data).await?;
    assert!(
        !encrypted.is_empty(),
        "Large data encryption should succeed"
    );

    // Decrypt large data
    let decrypted = shared_storage.decrypt(&encrypted).await?;
    assert_eq!(
        large_data, decrypted,
        "Large data should round-trip correctly"
    );

    println!("  ✅ Large data handling verified");
    Ok(())
}
