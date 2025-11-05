//! Integration tests for Transit Engine
//!
//! NOTE: This test file may reference outdated Transit Engine API.
//! TODO: Review and update for current implementation

// DISABLED: Pending API review
#![cfg(feature = "transit-integration-tests")]

use secreton_crypto::transit::{KeyOptions, KeyType, TransitEngine};

#[tokio::test]
async fn test_transit_engine_aes_gcm_roundtrip() {
    let engine = TransitEngine::new();

    // Create key
    engine
        .create_key("test-key".to_string(), KeyType::Aes256Gcm, None)
        .await
        .expect("Failed to create key");

    // Encrypt data
    let plaintext = b"Hello, World!";
    let ciphertext = engine
        .encrypt("test-key", plaintext, None)
        .await
        .expect("Failed to encrypt");

    // Decrypt data
    let decrypted = engine
        .decrypt("test-key", &ciphertext, None)
        .await
        .expect("Failed to decrypt");

    assert_eq!(plaintext, decrypted.as_slice());
}

#[tokio::test]
async fn test_transit_engine_chacha20_roundtrip() {
    let engine = TransitEngine::new();

    // Create key
    engine
        .create_key("chacha-key".to_string(), KeyType::ChaCha20Poly1305, None)
        .await
        .expect("Failed to create key");

    // Encrypt data
    let plaintext = b"Secret message with ChaCha20";
    let ciphertext = engine
        .encrypt("chacha-key", plaintext, None)
        .await
        .expect("Failed to encrypt");

    // Decrypt data
    let decrypted = engine
        .decrypt("chacha-key", &ciphertext, None)
        .await
        .expect("Failed to decrypt");

    assert_eq!(plaintext, decrypted.as_slice());
}

#[tokio::test]
async fn test_transit_engine_list_keys() {
    let engine = TransitEngine::new();

    // Create multiple keys
    engine
        .create_key("key1".to_string(), KeyType::Aes256Gcm, None)
        .await
        .expect("Failed to create key1");

    engine
        .create_key("key2".to_string(), KeyType::ChaCha20Poly1305, None)
        .await
        .expect("Failed to create key2");

    // List keys
    let keys = engine.list_keys().await;

    assert_eq!(keys.len(), 2);
    assert!(keys.contains(&"key1".to_string()));
    assert!(keys.contains(&"key2".to_string()));
}

#[tokio::test]
async fn test_transit_engine_key_already_exists() {
    let engine = TransitEngine::new();

    // Create key
    engine
        .create_key("duplicate-key".to_string(), KeyType::Aes256Gcm, None)
        .await
        .expect("Failed to create key");

    // Try to create same key again
    let result = engine
        .create_key("duplicate-key".to_string(), KeyType::Aes256Gcm, None)
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_transit_engine_encrypt_with_nonexistent_key() {
    let engine = TransitEngine::new();

    let plaintext = b"Test data";
    let result = engine.encrypt("nonexistent-key", plaintext, None).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_transit_engine_decrypt_with_wrong_key() {
    let engine = TransitEngine::new();

    // Create two keys
    engine
        .create_key("key1".to_string(), KeyType::Aes256Gcm, None)
        .await
        .expect("Failed to create key1");

    engine
        .create_key("key2".to_string(), KeyType::Aes256Gcm, None)
        .await
        .expect("Failed to create key2");

    // Encrypt with key1
    let plaintext = b"Secret data";
    let ciphertext = engine
        .encrypt("key1", plaintext, None)
        .await
        .expect("Failed to encrypt");

    // Try to decrypt with key2 - should fail
    let result = engine.decrypt("key2", &ciphertext, None).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_transit_engine_with_context() {
    let engine = TransitEngine::new();

    engine
        .create_key("context-key".to_string(), KeyType::Aes256Gcm, None)
        .await
        .expect("Failed to create key");

    let plaintext = b"Data with context";
    let context = b"user-id-123";

    // Encrypt with context
    let ciphertext = engine
        .encrypt("context-key", plaintext, Some(context))
        .await
        .expect("Failed to encrypt");

    // Decrypt with same context
    let decrypted = engine
        .decrypt("context-key", &ciphertext, Some(context))
        .await
        .expect("Failed to decrypt");

    assert_eq!(plaintext, decrypted.as_slice());

    // Try to decrypt with wrong context - should fail
    let wrong_context = b"wrong-user-id";
    let result = engine
        .decrypt("context-key", &ciphertext, Some(wrong_context))
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_transit_engine_large_data() {
    let engine = TransitEngine::new();

    engine
        .create_key("large-key".to_string(), KeyType::Aes256Gcm, None)
        .await
        .expect("Failed to create key");

    // Create 1MB of data
    let plaintext = vec![0x42u8; 1024 * 1024];

    let ciphertext = engine
        .encrypt("large-key", &plaintext, None)
        .await
        .expect("Failed to encrypt large data");

    let decrypted = engine
        .decrypt("large-key", &ciphertext, None)
        .await
        .expect("Failed to decrypt large data");

    assert_eq!(plaintext, decrypted);
}
