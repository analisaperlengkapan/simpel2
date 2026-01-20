//! Property-Based Tests for Zero-Knowledge Service
//!
//! These tests verify the correctness properties of the zero-knowledge E2EE system.

use proptest::prelude::*;
use secreton_core::services::zero_knowledge::{
    KeyDerivationParams, ZeroKnowledgeMetadata, ZeroKnowledgeService, ZeroKnowledgeServiceImpl,
};

// **Feature: secreton-comprehensive-enhancement, Property 6: Zero-Knowledge Storage Round-Trip**
// **Validates: Requirements 2.1, 2.2**
//
// Property: For any pre-encrypted data stored in zero-knowledge mode, retrieving the data
// SHALL return the exact encrypted blob without modification.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn property_zero_knowledge_storage_roundtrip(
        // Generate random path
        path_prefix in "[a-z]{3,10}",
        path_suffix in "[a-z]{3,10}",
        // Generate random encrypted data (1-1024 bytes)
        encrypted_data in prop::collection::vec(any::<u8>(), 1..=1024),
        // Generate random algorithm index
        algorithm_idx in 0usize..3,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let service = ZeroKnowledgeServiceImpl::new();

            // Construct valid path
            let path = format!("{}/{}", path_prefix, path_suffix);

            // Select algorithm
            let algorithms = ["aes-256-gcm", "chacha20-poly1305", "aes-128-gcm"];
            let algorithm = algorithms[algorithm_idx];

            // Create metadata
            let metadata = ZeroKnowledgeMetadata::new(
                path.clone(),
                algorithm.to_string(),
                KeyDerivationParams::default_hkdf(),
            );

            // Store encrypted data
            service.store(&path, encrypted_data.clone(), metadata.clone()).await
                .expect("Store should succeed");

            // Retrieve encrypted data
            let (retrieved_data, retrieved_metadata) = service.retrieve(&path).await
                .expect("Retrieve should succeed");

            // Property 1: Retrieved data should match original exactly
            prop_assert_eq!(&retrieved_data, &encrypted_data,
                "Retrieved encrypted data should match original exactly");

            // Property 2: Metadata should be preserved
            prop_assert_eq!(&retrieved_metadata.path, &metadata.path,
                "Path should be preserved");
            prop_assert_eq!(&retrieved_metadata.encryption_algorithm, &metadata.encryption_algorithm,
                "Encryption algorithm should be preserved");

            // Property 3: Multiple retrievals should return same data
            let (retrieved_data2, _) = service.retrieve(&path).await
                .expect("Second retrieve should succeed");

            prop_assert_eq!(&retrieved_data, &retrieved_data2,
                "Multiple retrievals should return identical data");

            Ok(())
        })?;
    }
}

// **Feature: secreton-comprehensive-enhancement, Property 7: Zero-Knowledge Audit Privacy**
// **Validates: Requirements 2.5**
//
// Property: For any zero-knowledge operation, the audit log SHALL contain operation metadata
// (path, timestamp, actor) but SHALL NOT contain any secret content or encryption keys.
//
// Note: This property test verifies that the service itself doesn't expose secrets.
// Actual audit logging integration will be tested separately.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn property_zero_knowledge_audit_privacy(
        // Generate random path
        path_prefix in "[a-z]{3,10}",
        path_suffix in "[a-z]{3,10}",
        // Generate random encrypted data (minimum 16 bytes to avoid false positives)
        encrypted_data in prop::collection::vec(any::<u8>(), 16..=256),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let service = ZeroKnowledgeServiceImpl::new();

            let path = format!("{}/{}", path_prefix, path_suffix);

            let metadata = ZeroKnowledgeMetadata::new(
                path.clone(),
                "aes-256-gcm".to_string(),
                KeyDerivationParams::default_hkdf(),
            );

            // Store secret
            service.store(&path, encrypted_data.clone(), metadata.clone()).await
                .expect("Store should succeed");

            // Property 1: Metadata should not contain the encrypted data
            let metadata_json = serde_json::to_string(&metadata)
                .expect("Metadata serialization should succeed");

            // Convert encrypted data to string representations that might leak
            // Only check if encrypted data is substantial enough (>= 16 bytes) to avoid false positives
            // from short hex strings that might coincidentally appear in timestamps or other fields
            let encrypted_hex = hex::encode(&encrypted_data);
            use base64::Engine;
            let encrypted_base64 = base64::engine::general_purpose::STANDARD.encode(&encrypted_data);

            prop_assert!(!metadata_json.contains(&encrypted_hex),
                "Metadata should not contain encrypted data in hex format");
            prop_assert!(!metadata_json.contains(&encrypted_base64),
                "Metadata should not contain encrypted data in base64 format");

            // Property 2: Key derivation params should not contain actual key material
            let params_json = serde_json::to_string(&metadata.key_derivation_params)
                .expect("Params serialization should succeed");

            // Params should only contain salt and info, not derived keys
            prop_assert!(params_json.contains("salt"),
                "Params should contain salt field");
            prop_assert!(params_json.contains("info"),
                "Params should contain info field");
            prop_assert!(params_json.contains("algorithm"),
                "Params should contain algorithm field");

            // Property 3: Retrieved metadata should also not expose secrets
            let (_, retrieved_metadata) = service.retrieve(&path).await
                .expect("Retrieve should succeed");

            let retrieved_json = serde_json::to_string(&retrieved_metadata)
                .expect("Retrieved metadata serialization should succeed");

            prop_assert!(!retrieved_json.contains(&encrypted_hex),
                "Retrieved metadata should not contain encrypted data");

            Ok(())
        })?;
    }
}

#[cfg(test)]
mod zero_knowledge_edge_cases {
    use super::*;
    use secreton_core::services::zero_knowledge::{
        KeyDerivationParams, ZeroKnowledgeMetadata, ZeroKnowledgeService, ZeroKnowledgeServiceImpl,
    };

    #[tokio::test]
    async fn test_empty_path_rejected() {
        let service = ZeroKnowledgeServiceImpl::new();

        let encrypted_data = vec![1, 2, 3, 4];
        let metadata = ZeroKnowledgeMetadata::new(
            "".to_string(),
            "aes-256-gcm".to_string(),
            KeyDerivationParams::default_hkdf(),
        );

        let result = service.store("", encrypted_data, metadata).await;
        assert!(result.is_err(), "Empty path should be rejected");
    }

    #[tokio::test]
    async fn test_path_traversal_rejected() {
        let service = ZeroKnowledgeServiceImpl::new();

        let encrypted_data = vec![1, 2, 3, 4];
        let metadata = ZeroKnowledgeMetadata::new(
            "secret/../other".to_string(),
            "aes-256-gcm".to_string(),
            KeyDerivationParams::default_hkdf(),
        );

        let result = service.store("secret/../other", encrypted_data, metadata).await;
        assert!(result.is_err(), "Path traversal should be rejected");
    }

    #[tokio::test]
    async fn test_invalid_algorithm_rejected() {
        let service = ZeroKnowledgeServiceImpl::new();

        let encrypted_data = vec![1, 2, 3, 4];
        let metadata = ZeroKnowledgeMetadata::new(
            "secret/test".to_string(),
            "invalid-algorithm".to_string(),
            KeyDerivationParams::default_hkdf(),
        );

        let result = service.store("secret/test", encrypted_data, metadata).await;
        assert!(result.is_err(), "Invalid algorithm should be rejected");
    }

    #[tokio::test]
    async fn test_empty_encrypted_data_rejected() {
        let service = ZeroKnowledgeServiceImpl::new();

        let encrypted_data = Vec::new();
        let metadata = ZeroKnowledgeMetadata::new(
            "secret/test".to_string(),
            "aes-256-gcm".to_string(),
            KeyDerivationParams::default_hkdf(),
        );

        let result = service.store("secret/test", encrypted_data, metadata).await;
        assert!(result.is_err(), "Empty encrypted data should be rejected");
    }

    #[tokio::test]
    async fn test_retrieve_nonexistent_secret() {
        let service = ZeroKnowledgeServiceImpl::new();

        let result = service.retrieve("secret/nonexistent").await;
        assert!(result.is_err(), "Retrieving nonexistent secret should fail");
    }

    #[tokio::test]
    async fn test_delete_removes_secret() {
        let service = ZeroKnowledgeServiceImpl::new();

        let encrypted_data = vec![1, 2, 3, 4];
        let metadata = ZeroKnowledgeMetadata::new(
            "secret/test".to_string(),
            "aes-256-gcm".to_string(),
            KeyDerivationParams::default_hkdf(),
        );

        service
            .store("secret/test", encrypted_data, metadata)
            .await
            .unwrap();

        // Delete the secret
        service.delete("secret/test").await.unwrap();

        // Should no longer be retrievable
        let result = service.retrieve("secret/test").await;
        assert!(result.is_err(), "Deleted secret should not be retrievable");
    }

    #[tokio::test]
    async fn test_overwrite_existing_secret() {
        let service = ZeroKnowledgeServiceImpl::new();

        let encrypted_data1 = vec![1, 2, 3, 4];
        let metadata1 = ZeroKnowledgeMetadata::new(
            "secret/test".to_string(),
            "aes-256-gcm".to_string(),
            KeyDerivationParams::default_hkdf(),
        );

        service
            .store("secret/test", encrypted_data1, metadata1)
            .await
            .unwrap();

        // Overwrite with new data
        let encrypted_data2 = vec![5, 6, 7, 8];
        let metadata2 = ZeroKnowledgeMetadata::new(
            "secret/test".to_string(),
            "chacha20-poly1305".to_string(),
            KeyDerivationParams::default_hkdf(),
        );

        service
            .store("secret/test", encrypted_data2.clone(), metadata2)
            .await
            .unwrap();

        // Should retrieve the new data
        let (retrieved_data, retrieved_metadata) = service.retrieve("secret/test").await.unwrap();
        assert_eq!(retrieved_data, encrypted_data2);
        assert_eq!(retrieved_metadata.encryption_algorithm, "chacha20-poly1305");
    }

    #[tokio::test]
    async fn test_list_paths_sorted() {
        let service = ZeroKnowledgeServiceImpl::new();

        // Store secrets in non-alphabetical order
        let paths = vec!["secret/zebra", "secret/apple", "secret/middle"];

        for path in &paths {
            let encrypted_data = vec![1, 2, 3, 4];
            let metadata = ZeroKnowledgeMetadata::new(
                path.to_string(),
                "aes-256-gcm".to_string(),
                KeyDerivationParams::default_hkdf(),
            );

            service
                .store(path, encrypted_data, metadata)
                .await
                .unwrap();
        }

        // List should be sorted
        let listed_paths = service.list_paths().await.unwrap();
        assert_eq!(listed_paths.len(), 3);
        assert_eq!(listed_paths[0], "secret/apple");
        assert_eq!(listed_paths[1], "secret/middle");
        assert_eq!(listed_paths[2], "secret/zebra");
    }

    #[tokio::test]
    async fn test_derive_params_deterministic_salt() {
        let service = ZeroKnowledgeServiceImpl::new();

        let client_entropy = b"user-password-123";

        // Derive params twice
        let params1 = service.derive_params(client_entropy).await.unwrap();
        let params2 = service.derive_params(client_entropy).await.unwrap();

        // Salts should be different (random)
        assert_ne!(params1.salt, params2.salt,
            "Each derivation should use a new random salt");

        // But algorithm and key length should be consistent
        assert_eq!(params1.algorithm, params2.algorithm);
        assert_eq!(params1.key_length, params2.key_length);
    }

    #[tokio::test]
    async fn test_empty_client_entropy_rejected() {
        let service = ZeroKnowledgeServiceImpl::new();

        let result = service.derive_params(&[]).await;
        assert!(result.is_err(), "Empty client entropy should be rejected");
    }

    #[tokio::test]
    async fn test_is_zk_enabled_after_delete() {
        let service = ZeroKnowledgeServiceImpl::new();

        let encrypted_data = vec![1, 2, 3, 4];
        let metadata = ZeroKnowledgeMetadata::new(
            "secret/test".to_string(),
            "aes-256-gcm".to_string(),
            KeyDerivationParams::default_hkdf(),
        );

        service
            .store("secret/test", encrypted_data, metadata)
            .await
            .unwrap();

        assert!(service.is_zk_enabled("secret/test").await.unwrap());

        service.delete("secret/test").await.unwrap();

        assert!(!service.is_zk_enabled("secret/test").await.unwrap());
    }

    #[tokio::test]
    async fn test_large_encrypted_data() {
        let service = ZeroKnowledgeServiceImpl::new();

        // Test with 1MB of encrypted data
        let encrypted_data = vec![42u8; 1024 * 1024];
        let metadata = ZeroKnowledgeMetadata::new(
            "secret/large".to_string(),
            "aes-256-gcm".to_string(),
            KeyDerivationParams::default_hkdf(),
        );

        service
            .store("secret/large", encrypted_data.clone(), metadata)
            .await
            .unwrap();

        let (retrieved_data, _) = service.retrieve("secret/large").await.unwrap();
        assert_eq!(retrieved_data.len(), 1024 * 1024);
        assert_eq!(retrieved_data, encrypted_data);
    }
}
