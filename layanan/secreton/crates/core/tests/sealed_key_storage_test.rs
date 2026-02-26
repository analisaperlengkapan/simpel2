//! Integration tests for sealed master key storage

use secreton_core::storage::sealed_keys::{
    PostgresSealedKeyStorage, ProviderType, SealedKeyStorage, SealedMasterKey,
};
use std::sync::Arc;
use uuid::Uuid;

/// Helper function to create a test database pool
async fn create_test_pool() -> Arc<deadpool_postgres::Pool> {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://secreton:secreton@localhost:5432/secreton_test".to_string()
    });

    let mut cfg = deadpool_postgres::Config::new();
    cfg.url = Some(database_url);

    let pool = cfg
        .create_pool(
            Some(deadpool_postgres::Runtime::Tokio1),
            tokio_postgres::NoTls,
        )
        .expect("Failed to create test pool");

    Arc::new(pool)
}

#[tokio::test]
#[ignore] // Requires PostgreSQL database
async fn test_store_and_retrieve_sealed_key() {
    let pool = create_test_pool().await;
    let storage = PostgresSealedKeyStorage::new(pool);

    // Create a test sealed key
    let encrypted_data = vec![1, 2, 3, 4, 5, 6, 7, 8];
    let key = SealedMasterKey::new(
        ProviderType::AwsKms,
        "arn:aws:kms:us-east-1:123456789012:key/test-key".to_string(),
        Some("us-east-1".to_string()),
        None,
        encrypted_data.clone(),
    );

    // Store the key
    storage
        .store_sealed_key(&key)
        .await
        .expect("Failed to store sealed key");

    // Retrieve the key by ID
    let retrieved = storage
        .get_sealed_key_by_id(key.id)
        .await
        .expect("Failed to retrieve sealed key")
        .expect("Sealed key not found");

    assert_eq!(retrieved.id, key.id);
    assert_eq!(retrieved.provider_type, ProviderType::AwsKms);
    assert_eq!(
        retrieved.provider_key_id,
        "arn:aws:kms:us-east-1:123456789012:key/test-key"
    );
    assert_eq!(retrieved.provider_region, Some("us-east-1".to_string()));
    assert_eq!(retrieved.encrypted_master_key, encrypted_data);
    assert!(retrieved.verify_checksum());
}

#[tokio::test]
#[ignore] // Requires PostgreSQL database
async fn test_set_active_sealed_key() {
    let pool = create_test_pool().await;
    let storage = PostgresSealedKeyStorage::new(pool);

    // Create two test sealed keys
    let key1 = SealedMasterKey::new(
        ProviderType::Transit,
        "auto-unseal-key-1".to_string(),
        None,
        Some("https://secreton1.internal:8200".to_string()),
        vec![1, 2, 3],
    );

    let key2 = SealedMasterKey::new(
        ProviderType::Transit,
        "auto-unseal-key-2".to_string(),
        None,
        Some("https://secreton2.internal:8200".to_string()),
        vec![4, 5, 6],
    );

    // Store both keys
    storage
        .store_sealed_key(&key1)
        .await
        .expect("Failed to store key1");
    storage
        .store_sealed_key(&key2)
        .await
        .expect("Failed to store key2");

    // Set key1 as active
    storage
        .set_active_sealed_key(key1.id)
        .await
        .expect("Failed to set key1 as active");

    // Verify key1 is active
    let active = storage
        .get_active_sealed_key()
        .await
        .expect("Failed to get active key")
        .expect("No active key found");

    assert_eq!(active.id, key1.id);
    assert!(active.is_active);

    // Set key2 as active
    storage
        .set_active_sealed_key(key2.id)
        .await
        .expect("Failed to set key2 as active");

    // Verify key2 is now active and key1 is not
    let active = storage
        .get_active_sealed_key()
        .await
        .expect("Failed to get active key")
        .expect("No active key found");

    assert_eq!(active.id, key2.id);
    assert!(active.is_active);

    // Verify key1 is no longer active
    let key1_retrieved = storage
        .get_sealed_key_by_id(key1.id)
        .await
        .expect("Failed to retrieve key1")
        .expect("Key1 not found");

    assert!(!key1_retrieved.is_active);
}

#[tokio::test]
#[ignore] // Requires PostgreSQL database
async fn test_list_sealed_keys_by_provider() {
    let pool = create_test_pool().await;
    let storage = PostgresSealedKeyStorage::new(pool);

    // Create keys with different providers
    let aws_key = SealedMasterKey::new(
        ProviderType::AwsKms,
        "arn:aws:kms:us-east-1:123456789012:key/test".to_string(),
        Some("us-east-1".to_string()),
        None,
        vec![1, 2, 3],
    );

    let gcp_key = SealedMasterKey::new(
        ProviderType::GcpKms,
        "projects/test-project/locations/us-central1/keyRings/test/cryptoKeys/test".to_string(),
        Some("us-central1".to_string()),
        None,
        vec![4, 5, 6],
    );

    let transit_key = SealedMasterKey::new(
        ProviderType::Transit,
        "auto-unseal-key".to_string(),
        None,
        Some("https://secreton.internal:8200".to_string()),
        vec![7, 8, 9],
    );

    // Store all keys
    storage
        .store_sealed_key(&aws_key)
        .await
        .expect("Failed to store AWS key");
    storage
        .store_sealed_key(&gcp_key)
        .await
        .expect("Failed to store GCP key");
    storage
        .store_sealed_key(&transit_key)
        .await
        .expect("Failed to store Transit key");

    // List AWS keys
    let aws_keys = storage
        .get_sealed_keys_by_provider(ProviderType::AwsKms)
        .await
        .expect("Failed to list AWS keys");

    assert!(aws_keys.iter().any(|k| k.id == aws_key.id));
    assert!(
        aws_keys
            .iter()
            .all(|k| k.provider_type == ProviderType::AwsKms)
    );

    // List Transit keys
    let transit_keys = storage
        .get_sealed_keys_by_provider(ProviderType::Transit)
        .await
        .expect("Failed to list Transit keys");

    assert!(transit_keys.iter().any(|k| k.id == transit_key.id));
    assert!(
        transit_keys
            .iter()
            .all(|k| k.provider_type == ProviderType::Transit)
    );
}

#[tokio::test]
#[ignore] // Requires PostgreSQL database
async fn test_delete_sealed_key() {
    let pool = create_test_pool().await;
    let storage = PostgresSealedKeyStorage::new(pool);

    // Create a test sealed key
    let key = SealedMasterKey::new(
        ProviderType::AzureKv,
        "https://test-vault.vault.azure.net/keys/test-key".to_string(),
        Some("eastus".to_string()),
        None,
        vec![1, 2, 3, 4, 5],
    );

    // Store the key
    storage
        .store_sealed_key(&key)
        .await
        .expect("Failed to store sealed key");

    // Verify key exists
    let retrieved = storage
        .get_sealed_key_by_id(key.id)
        .await
        .expect("Failed to retrieve sealed key");
    assert!(retrieved.is_some());

    // Delete the key
    storage
        .delete_sealed_key(key.id)
        .await
        .expect("Failed to delete sealed key");

    // Verify key no longer exists
    let retrieved = storage
        .get_sealed_key_by_id(key.id)
        .await
        .expect("Failed to retrieve sealed key");
    assert!(retrieved.is_none());
}

#[tokio::test]
#[ignore] // Requires PostgreSQL database
async fn test_unique_active_constraint() {
    let pool = create_test_pool().await;
    let storage = PostgresSealedKeyStorage::new(pool);

    // Create two keys
    let key1 = SealedMasterKey::new(
        ProviderType::AwsKms,
        "key1".to_string(),
        None,
        None,
        vec![1, 2, 3],
    );

    let key2 = SealedMasterKey::new(
        ProviderType::AwsKms,
        "key2".to_string(),
        None,
        None,
        vec![4, 5, 6],
    );

    // Store both keys
    storage
        .store_sealed_key(&key1)
        .await
        .expect("Failed to store key1");
    storage
        .store_sealed_key(&key2)
        .await
        .expect("Failed to store key2");

    // Set key1 as active
    storage
        .set_active_sealed_key(key1.id)
        .await
        .expect("Failed to set key1 as active");

    // Set key2 as active (should deactivate key1)
    storage
        .set_active_sealed_key(key2.id)
        .await
        .expect("Failed to set key2 as active");

    // Verify only key2 is active
    let active = storage
        .get_active_sealed_key()
        .await
        .expect("Failed to get active key")
        .expect("No active key found");

    assert_eq!(active.id, key2.id);

    // Verify key1 is not active
    let key1_retrieved = storage
        .get_sealed_key_by_id(key1.id)
        .await
        .expect("Failed to retrieve key1")
        .expect("Key1 not found");

    assert!(!key1_retrieved.is_active);
}
