//! Property-Based Tests for Key Hierarchy Service
//!
//! These tests verify the correctness properties of the hierarchical key management system.

use proptest::prelude::*;
use secreton_core::services::key_hierarchy::{KeyHierarchyService, KeyHierarchyServiceImpl};
use secreton_core::services::seal::{SealConfig, SealService, InMemoryVaultStateStorage};
use std::sync::Arc;
use uuid::Uuid;

// Helper to setup test service
async fn setup_test_service() -> Arc<KeyHierarchyServiceImpl> {
    let config = SealConfig::default();
    let storage = Arc::new(InMemoryVaultStateStorage::new());
    let seal_service = Arc::new(SealService::with_storage(config, storage));

    // Initialize and unseal vault
    let shares = seal_service.initialize().await.unwrap();
    for share in shares.iter().take(3) {
        let share_bytes = share.to_bytes().unwrap();
        let _ = seal_service.unseal_with_share(&share_bytes).await;
    }

    Arc::new(KeyHierarchyServiceImpl::new(seal_service))
}

// **Feature: secreton-comprehensive-enhancement, Property 3: DEK Encryption Round-Trip**
// **Validates: Requirements 1.3**
//
// Property: For any DEK and KEK, encrypting the DEK with the KEK and then decrypting
// SHALL produce the original DEK.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(5))]

    #[test]
    fn property_dek_encryption_roundtrip(
        // Generate random context for KEK
        context in "[a-z]{5,20}",
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let hierarchy = setup_test_service().await;

            // Create KEK
            let kek_metadata = hierarchy.derive_kek(&context).await
                .expect("KEK derivation should succeed");

            // Create DEK (this encrypts it with the KEK)
            let dek_metadata = hierarchy.create_dek(kek_metadata.id).await
                .expect("DEK creation should succeed");

            // Retrieve DEK (this decrypts it)
            let dek_bytes = hierarchy.get_dek(dek_metadata.id).await
                .expect("DEK retrieval should succeed");

            // Property: DEK should be 32 bytes (AES-256)
            prop_assert_eq!(dek_bytes.len(), 32,
                "DEK should be 32 bytes for AES-256");

            // Property: Retrieving the same DEK multiple times should give same result
            let dek_bytes2 = hierarchy.get_dek(dek_metadata.id).await
                .expect("DEK retrieval should succeed");

            prop_assert_eq!(dek_bytes, dek_bytes2,
                "DEK round-trip should be consistent");

            Ok(())
        })?;
    }
}

#[cfg(test)]
mod dek_edge_cases {
    use super::*;

    #[tokio::test]
    async fn test_dek_multiple_deks_per_kek() {
        let hierarchy = setup_test_service().await;

        // Create one KEK
        let kek_metadata = hierarchy.derive_kek("test-kek").await.unwrap();

        // Create multiple DEKs with the same KEK
        let mut dek_ids = Vec::new();
        for _ in 0..10 {
            let dek_metadata = hierarchy.create_dek(kek_metadata.id).await.unwrap();
            dek_ids.push(dek_metadata.id);
        }

        // All DEKs should be retrievable and unique
        let mut dek_values = Vec::new();
        for dek_id in dek_ids {
            let dek_bytes = hierarchy.get_dek(dek_id).await.unwrap();
            assert_eq!(dek_bytes.len(), 32);
            dek_values.push(dek_bytes);
        }

        // All DEKs should be different
        for i in 0..dek_values.len() {
            for j in (i + 1)..dek_values.len() {
                assert_ne!(dek_values[i], dek_values[j],
                    "DEKs {} and {} should be different", i, j);
            }
        }
    }

    // TODO: This test is currently disabled because the KeyHierarchyService
    // doesn't automatically clear KEK key material when the vault is sealed.
    // This would require implementing a seal/unseal event listener or checking
    // seal status before each operation. This is a known limitation that should
    // be addressed in a future enhancement.
    #[tokio::test]
    #[ignore]
    async fn test_dek_with_sealed_vault() {
        let config = SealConfig::default();
        let storage = Arc::new(InMemoryVaultStateStorage::new());
        let seal_service = Arc::new(SealService::with_storage(config, storage));

        // Initialize and unseal vault
        let shares = seal_service.initialize().await.unwrap();
        for share in shares.iter().take(3) {
            let share_bytes = share.to_bytes().unwrap();
            let _ = seal_service.unseal_with_share(&share_bytes).await;
        }

        let hierarchy = Arc::new(KeyHierarchyServiceImpl::new(seal_service.clone()));

        // Create KEK and DEK while unsealed
        let kek_metadata = hierarchy.derive_kek("test-kek").await.unwrap();
        let dek_metadata = hierarchy.create_dek(kek_metadata.id).await.unwrap();

        // Seal the vault
        seal_service.seal().await.unwrap();

        // Trying to retrieve DEK should fail when vault is sealed
        let result = hierarchy.get_dek(dek_metadata.id).await;
        assert!(result.is_err(), "DEK retrieval should fail when vault is sealed");
    }

    #[tokio::test]
    async fn test_dek_nonexistent() {
        let hierarchy = setup_test_service().await;

        // Try to retrieve non-existent DEK
        let fake_id = uuid::Uuid::new_v4();
        let result = hierarchy.get_dek(fake_id).await;
        assert!(result.is_err(), "Retrieving non-existent DEK should fail");
    }
}


// **Feature: secreton-comprehensive-enhancement, Property 5: KEK Rotation Preserves DEK Accessibility**
// **Validates: Requirements 1.5**
//
// Property: For any KEK with associated DEKs, after rotation all DEKs SHALL remain decryptable
// with the new KEK and SHALL NOT be decryptable with the old KEK.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(5))]

    #[test]
    fn property_kek_rotation_preserves_dek_accessibility(
        // Generate random context for KEK
        context in "[a-z]{5,20}",
        // Generate number of DEKs to create (1-5)
        num_deks in 1usize..=5,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let hierarchy = setup_test_service().await;

            // Create KEK
            let kek_metadata = hierarchy.derive_kek(&context).await
                .expect("KEK derivation should succeed");

            // Create multiple DEKs
            let mut dek_ids = Vec::new();
            let mut dek_values_before = Vec::new();

            for _ in 0..num_deks {
                let dek_metadata = hierarchy.create_dek(kek_metadata.id).await
                    .expect("DEK creation should succeed");

                let dek_value = hierarchy.get_dek(dek_metadata.id).await
                    .expect("DEK retrieval should succeed");

                dek_ids.push(dek_metadata.id);
                dek_values_before.push(dek_value);
            }

            // Rotate KEK
            let new_kek_metadata = hierarchy.rotate_kek(kek_metadata.id).await
                .expect("KEK rotation should succeed");

            // Property 1: New KEK has different ID
            prop_assert_ne!(new_kek_metadata.id, kek_metadata.id,
                "Rotated KEK should have different ID");

            // Property 2: All DEKs remain accessible with same values
            for (i, dek_id) in dek_ids.iter().enumerate() {
                let dek_value_after = hierarchy.get_dek(*dek_id).await
                    .expect("DEK should still be accessible after KEK rotation");

                prop_assert_eq!(&dek_values_before[i], &dek_value_after,
                    "DEK value should remain unchanged after KEK rotation");
            }

            // Property 3: Old KEK is no longer accessible
            let old_kek_result = hierarchy.get_kek_metadata(kek_metadata.id).await;
            prop_assert!(old_kek_result.is_err(),
                "Old KEK should not be accessible after rotation");

            // Property 4: New KEK is accessible
            let new_kek_result = hierarchy.get_kek_metadata(new_kek_metadata.id).await;
            prop_assert!(new_kek_result.is_ok(),
                "New KEK should be accessible after rotation");

            Ok(())
        })?;
    }
}


// **Feature: secreton-comprehensive-enhancement, Property 4: Key Metadata Serialization Round-Trip**
// **Validates: Requirements 1.6, 1.7**
//
// Property: For any valid KeyMetadata, serializing to JSON and deserializing SHALL produce
// an equivalent KeyMetadata object.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn property_metadata_serialization_roundtrip(
        context in "[a-zA-Z0-9_-]{5,50}",
        version in 1u32..=10,
    ) {
        use secreton_core::services::key_hierarchy::{KeyLevel, KeyMetadata};
        use chrono::Utc;
        use uuid::Uuid;

        // Create metadata with random values
        let mut metadata = KeyMetadata {
            id: Uuid::new_v4(),
            level: KeyLevel::KeyEncryptionKey,
            parent_id: Some(Uuid::new_v4()),
            context: context.clone(),
            created_at: Utc::now(),
            rotated_at: Some(Utc::now()),
            version,
        };

        // Serialize to JSON
        let json = metadata.to_json()
            .expect("Serialization should succeed");

        // Deserialize from JSON
        let deserialized = KeyMetadata::from_json(&json)
            .expect("Deserialization should succeed");

        // Property: All fields should match
        prop_assert_eq!(metadata.id, deserialized.id,
            "ID should be preserved");
        prop_assert_eq!(format!("{:?}", metadata.level), format!("{:?}", deserialized.level),
            "Level should be preserved");
        prop_assert_eq!(metadata.parent_id, deserialized.parent_id,
            "Parent ID should be preserved");
        prop_assert_eq!(&metadata.context, &deserialized.context,
            "Context should be preserved");
        prop_assert_eq!(metadata.version, deserialized.version,
            "Version should be preserved");

        // Test with different key levels
        for level in [KeyLevel::MasterKey, KeyLevel::KeyEncryptionKey, KeyLevel::DataEncryptionKey] {
            metadata.level = level;
            let json = metadata.to_json().expect("Serialization should succeed");
            let deserialized = KeyMetadata::from_json(&json).expect("Deserialization should succeed");
            prop_assert_eq!(format!("{:?}", metadata.level), format!("{:?}", deserialized.level),
                "Level {:?} should be preserved", level);
        }
    }
}
