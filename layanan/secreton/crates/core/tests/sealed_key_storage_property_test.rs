//! Property-based tests for sealed master key storage
//!
//! **Validates: Requirements 2.1.5**
//!
//! This test suite verifies that sealed master key storage operations maintain
//! correctness properties across all possible inputs using property-based testing.

use proptest::prelude::*;
use proptest::strategy::ValueTree;
use secreton_core::storage::sealed_keys::{
    PostgresSealedKeyStorage, ProviderType, SealedKeyStorage, SealedMasterKey,
};
use std::sync::Arc;

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

/// Strategy for generating ProviderType
fn provider_type_strategy() -> impl Strategy<Value = ProviderType> {
    prop_oneof![
        Just(ProviderType::AwsKms),
        Just(ProviderType::GcpKms),
        Just(ProviderType::AzureKv),
        Just(ProviderType::Transit),
    ]
}

/// Strategy for generating provider key IDs
fn provider_key_id_strategy(provider_type: ProviderType) -> impl Strategy<Value = String> {
    match provider_type {
        ProviderType::AwsKms => {
            // AWS KMS ARN format
            "[a-z0-9]{8}-[a-z0-9]{4}-[a-z0-9]{4}-[a-z0-9]{4}-[a-z0-9]{12}"
                .prop_map(|uuid| format!("arn:aws:kms:us-east-1:123456789012:key/{}", uuid))
                .boxed()
        }
        ProviderType::GcpKms => {
            // GCP KMS resource name format
            "[a-z0-9-]{5,20}"
                .prop_map(|name| {
                    format!(
                        "projects/test-project/locations/us-central1/keyRings/test/cryptoKeys/{}",
                        name
                    )
                })
                .boxed()
        }
        ProviderType::AzureKv => {
            // Azure Key Vault URL format
            "[a-z0-9-]{5,20}"
                .prop_map(|name| format!("https://test-vault.vault.azure.net/keys/{}", name))
                .boxed()
        }
        ProviderType::Transit => {
            // Transit key name
            "[a-z0-9-]{5,20}".boxed()
        }
    }
}

/// Strategy for generating provider regions
fn provider_region_strategy(provider_type: ProviderType) -> impl Strategy<Value = Option<String>> {
    match provider_type {
        ProviderType::AwsKms => prop_oneof![
            Just(Some("us-east-1".to_string())),
            Just(Some("us-west-2".to_string())),
            Just(Some("eu-west-1".to_string())),
        ]
        .boxed(),
        ProviderType::GcpKms => prop_oneof![
            Just(Some("us-central1".to_string())),
            Just(Some("europe-west1".to_string())),
            Just(Some("asia-east1".to_string())),
        ]
        .boxed(),
        ProviderType::AzureKv => prop_oneof![
            Just(Some("eastus".to_string())),
            Just(Some("westus".to_string())),
            Just(Some("westeurope".to_string())),
        ]
        .boxed(),
        ProviderType::Transit => Just(None).boxed(),
    }
}

/// Strategy for generating provider endpoints
fn provider_endpoint_strategy(
    provider_type: ProviderType,
) -> impl Strategy<Value = Option<String>> {
    match provider_type {
        ProviderType::Transit => prop_oneof![
            Just(Some("https://secreton.internal:8200".to_string())),
            Just(Some("https://secreton-primary.internal:8200".to_string())),
            Just(Some("https://secreton-dr.internal:8200".to_string())),
        ]
        .boxed(),
        _ => Just(None).boxed(),
    }
}

/// Strategy for generating encrypted master keys (random bytes)
fn encrypted_master_key_strategy() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>(), 32..256)
}

/// Strategy for generating metadata
fn metadata_strategy() -> impl Strategy<Value = serde_json::Value> {
    prop_oneof![
        Just(serde_json::json!({})),
        Just(serde_json::json!({"rotation_count": 1})),
        Just(serde_json::json!({"created_by": "admin", "purpose": "auto-unseal"})),
    ]
}

/// Strategy for generating complete SealedMasterKey instances
fn sealed_master_key_strategy() -> impl Strategy<Value = SealedMasterKey> {
    provider_type_strategy().prop_flat_map(|provider_type| {
        (
            Just(provider_type.clone()),
            provider_key_id_strategy(provider_type.clone()),
            provider_region_strategy(provider_type.clone()),
            provider_endpoint_strategy(provider_type),
            encrypted_master_key_strategy(),
            metadata_strategy(),
        )
            .prop_map(
                |(provider_type, key_id, region, endpoint, encrypted_data, metadata)| {
                    let mut key = SealedMasterKey::new(
                        provider_type,
                        key_id,
                        region,
                        endpoint,
                        encrypted_data,
                    );
                    key.metadata = metadata;
                    key
                },
            )
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    /// **Property 2: Auto-unseal configuration persistence**
    ///
    /// **Validates: Requirements 2.1.5**
    ///
    /// This property verifies that any sealed master key that is stored can be
    /// retrieved with the same data, ensuring persistence correctness.
    ///
    /// **Property:** For any valid SealedMasterKey, storing it and then retrieving
    /// it by ID should return the exact same data.
    #[test]
    #[ignore] // Requires PostgreSQL database
    fn prop_sealed_key_round_trip(key in sealed_master_key_strategy()) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let pool = create_test_pool().await;
            let storage = PostgresSealedKeyStorage::new(pool);

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

            // Verify all fields match
            prop_assert_eq!(retrieved.id, key.id);
            prop_assert_eq!(&retrieved.provider_type, &key.provider_type);
            prop_assert_eq!(&retrieved.provider_key_id, &key.provider_key_id);
            prop_assert_eq!(&retrieved.provider_region, &key.provider_region);
            prop_assert_eq!(&retrieved.provider_endpoint, &key.provider_endpoint);
            prop_assert_eq!(&retrieved.encrypted_master_key, &key.encrypted_master_key);
            prop_assert_eq!(&retrieved.checksum, &key.checksum);
            prop_assert_eq!(retrieved.is_active, key.is_active);
            prop_assert_eq!(retrieved.version, key.version);

            // Verify checksum integrity
            prop_assert!(retrieved.verify_checksum(), "Checksum verification failed");

            Ok(())
        })?;
    }

    /// **Property 2.1: Active key uniqueness**
    ///
    /// **Validates: Requirements 2.1.5**
    ///
    /// This property verifies that setting a key as active makes it the only active key,
    /// ensuring the unique active constraint is maintained.
    ///
    /// **Property:** When setting a key as active, it should be the only active key
    /// in the system, and all other keys should be inactive.
    #[test]
    #[ignore] // Requires PostgreSQL database
    fn prop_active_key_uniqueness(
        key1 in sealed_master_key_strategy(),
        key2 in sealed_master_key_strategy()
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let pool = create_test_pool().await;
            let storage = PostgresSealedKeyStorage::new(pool);

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

            // Verify key1 is the only active key
            let active = storage
                .get_active_sealed_key()
                .await
                .expect("Failed to get active key")
                .expect("No active key found");

            prop_assert_eq!(active.id, key1.id);
            prop_assert!(active.is_active);

            // Set key2 as active
            storage
                .set_active_sealed_key(key2.id)
                .await
                .expect("Failed to set key2 as active");

            // Verify key2 is now the only active key
            let active = storage
                .get_active_sealed_key()
                .await
                .expect("Failed to get active key")
                .expect("No active key found");

            prop_assert_eq!(active.id, key2.id);
            prop_assert!(active.is_active);

            // Verify key1 is no longer active
            let key1_retrieved = storage
                .get_sealed_key_by_id(key1.id)
                .await
                .expect("Failed to retrieve key1")
                .expect("Key1 not found");

            prop_assert!(!key1_retrieved.is_active, "Key1 should not be active");

            Ok(())
        })?;
    }

    /// **Property 2.2: Checksum verification correctness**
    ///
    /// **Validates: Requirements 2.1.5**
    ///
    /// This property verifies that the checksum verification works correctly
    /// for all stored sealed keys.
    ///
    /// **Property:** Any sealed key retrieved from storage should have a valid
    /// checksum that matches its encrypted data.
    #[test]
    #[ignore] // Requires PostgreSQL database
    fn prop_checksum_verification(key in sealed_master_key_strategy()) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let pool = create_test_pool().await;
            let storage = PostgresSealedKeyStorage::new(pool);

            // Store the key
            storage
                .store_sealed_key(&key)
                .await
                .expect("Failed to store sealed key");

            // Retrieve the key
            let retrieved = storage
                .get_sealed_key_by_id(key.id)
                .await
                .expect("Failed to retrieve sealed key")
                .expect("Sealed key not found");

            // Verify checksum is valid
            prop_assert!(
                retrieved.verify_checksum(),
                "Checksum verification failed for retrieved key"
            );

            // Verify checksum matches original
            prop_assert_eq!(
                retrieved.checksum,
                key.checksum,
                "Checksum mismatch between original and retrieved key"
            );

            Ok(())
        })?;
    }

    /// **Property 2.3: Provider metadata preservation**
    ///
    /// **Validates: Requirements 2.1.5**
    ///
    /// This property verifies that provider metadata is preserved across
    /// store/retrieve cycles.
    ///
    /// **Property:** All provider-specific metadata (type, key ID, region, endpoint)
    /// should be preserved exactly when storing and retrieving sealed keys.
    #[test]
    #[ignore] // Requires PostgreSQL database
    fn prop_provider_metadata_preservation(key in sealed_master_key_strategy()) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let pool = create_test_pool().await;
            let storage = PostgresSealedKeyStorage::new(pool);

            // Store the key
            storage
                .store_sealed_key(&key)
                .await
                .expect("Failed to store sealed key");

            // Retrieve the key
            let retrieved = storage
                .get_sealed_key_by_id(key.id)
                .await
                .expect("Failed to retrieve sealed key")
                .expect("Sealed key not found");

            // Verify provider metadata is preserved
            prop_assert_eq!(
                retrieved.provider_type,
                key.provider_type,
                "Provider type mismatch"
            );
            prop_assert_eq!(
                retrieved.provider_key_id,
                key.provider_key_id,
                "Provider key ID mismatch"
            );
            prop_assert_eq!(
                retrieved.provider_region,
                key.provider_region,
                "Provider region mismatch"
            );
            prop_assert_eq!(
                retrieved.provider_endpoint,
                key.provider_endpoint,
                "Provider endpoint mismatch"
            );
            prop_assert_eq!(
                retrieved.metadata,
                key.metadata,
                "Additional metadata mismatch"
            );

            Ok(())
        })?;
    }

    /// **Property 2.4: List operations consistency**
    ///
    /// **Validates: Requirements 2.1.5**
    ///
    /// This property verifies that list operations return consistent results
    /// with individual get operations.
    ///
    /// **Property:** If a key is stored, it should appear in list operations
    /// and be retrievable individually with the same data.
    #[test]
    #[ignore] // Requires PostgreSQL database
    fn prop_list_operations_consistency(keys in prop::collection::vec(sealed_master_key_strategy(), 1..5)) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let pool = create_test_pool().await;
            let storage = PostgresSealedKeyStorage::new(pool);

            // Store all keys
            for key in &keys {
                storage
                    .store_sealed_key(key)
                    .await
                    .expect("Failed to store sealed key");
            }

            // List all keys
            let listed = storage
                .list_sealed_keys()
                .await
                .expect("Failed to list sealed keys");

            // Verify all stored keys appear in the list
            for key in &keys {
                let found = listed.iter().any(|k| k.id == key.id);
                prop_assert!(found, "Key {} not found in list", key.id);

                // Verify individual retrieval matches list entry
                let retrieved = storage
                    .get_sealed_key_by_id(key.id)
                    .await
                    .expect("Failed to retrieve sealed key")
                    .expect("Sealed key not found");

                let listed_entry = listed.iter().find(|k| k.id == key.id).unwrap();

                prop_assert_eq!(
                    &retrieved.encrypted_master_key,
                    &listed_entry.encrypted_master_key,
                    "Encrypted data mismatch between get and list"
                );
            }

            Ok(())
        })?;
    }

    /// **Property 2.5: Provider filtering correctness**
    ///
    /// **Validates: Requirements 2.1.5**
    ///
    /// This property verifies that filtering keys by provider type returns
    /// only keys of that provider type.
    ///
    /// **Property:** When filtering by provider type, all returned keys should
    /// have that provider type, and all keys of that type should be returned.
    #[test]
    #[ignore] // Requires PostgreSQL database
    fn prop_provider_filtering(keys in prop::collection::vec(sealed_master_key_strategy(), 2..10)) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let pool = create_test_pool().await;
            let storage = PostgresSealedKeyStorage::new(pool);

            // Store all keys
            for key in &keys {
                storage
                    .store_sealed_key(key)
                    .await
                    .expect("Failed to store sealed key");
            }

            // Test filtering for each provider type
            for provider_type in &[
                ProviderType::AwsKms,
                ProviderType::GcpKms,
                ProviderType::AzureKv,
                ProviderType::Transit,
            ] {
                let filtered = storage
                    .get_sealed_keys_by_provider(provider_type.clone())
                    .await
                    .expect("Failed to filter sealed keys");

                // Verify all returned keys have the correct provider type
                for key in &filtered {
                    prop_assert_eq!(
                        &key.provider_type,
                        provider_type,
                        "Filtered key has wrong provider type"
                    );
                }

                // Verify all keys of this type are returned
                let expected_count = keys
                    .iter()
                    .filter(|k| &k.provider_type == provider_type)
                    .count();

                let actual_count = filtered
                    .iter()
                    .filter(|k| keys.iter().any(|orig| orig.id == k.id))
                    .count();

                prop_assert_eq!(
                    actual_count,
                    expected_count,
                    "Not all keys of provider type {:?} were returned",
                    provider_type
                );
            }

            Ok(())
        })?;
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_provider_type_strategy_generates_all_types() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let mut types_seen = std::collections::HashSet::new();

        for _ in 0..100 {
            let provider_type = runtime.block_on(async {
                provider_type_strategy()
                    .new_tree(&mut proptest::test_runner::TestRunner::default())
                    .unwrap()
                    .current()
            });

            types_seen.insert(format!("{:?}", provider_type));
        }

        // Verify all provider types are generated
        assert!(types_seen.contains("AwsKms"));
        assert!(types_seen.contains("GcpKms"));
        assert!(types_seen.contains("AzureKv"));
        assert!(types_seen.contains("Transit"));
    }

    #[test]
    fn test_encrypted_master_key_strategy_generates_valid_sizes() {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        for _ in 0..10 {
            let encrypted_data = runtime.block_on(async {
                encrypted_master_key_strategy()
                    .new_tree(&mut proptest::test_runner::TestRunner::default())
                    .unwrap()
                    .current()
            });

            assert!(
                encrypted_data.len() >= 32 && encrypted_data.len() < 256,
                "Encrypted data size out of range: {}",
                encrypted_data.len()
            );
        }
    }
}
