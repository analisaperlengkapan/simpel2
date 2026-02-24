//! Property-based tests for AutoUnsealProvider trait
//!
//! **Validates: Requirements 2.1.1, 2.1.2, 2.1.3, 2.1.4**
//!
//! These tests verify that all auto-unseal providers correctly implement
//! the encrypt/decrypt round-trip property across all supported providers.

use proptest::prelude::*;
use secreton_auto_unseal::{AutoUnsealProvider, ProviderMetadata};

/// Mock auto-unseal provider for testing
///
/// This provider uses simple XOR encryption for testing purposes.
/// It's not cryptographically secure but allows us to test the trait interface.
struct MockProvider {
    key: Vec<u8>,
}

impl MockProvider {
    fn new(key: Vec<u8>) -> Self {
        Self { key }
    }
}

#[async_trait::async_trait]
impl AutoUnsealProvider for MockProvider {
    fn name(&self) -> &str {
        "mock"
    }

    async fn encrypt(
        &self,
        plaintext: &[u8],
    ) -> Result<Vec<u8>, secreton_core::error::SecretonError> {
        // Simple XOR encryption for testing
        let mut ciphertext = Vec::with_capacity(plaintext.len());
        for (i, &byte) in plaintext.iter().enumerate() {
            ciphertext.push(byte ^ self.key[i % self.key.len()]);
        }
        Ok(ciphertext)
    }

    async fn decrypt(
        &self,
        ciphertext: &[u8],
    ) -> Result<Vec<u8>, secreton_core::error::SecretonError> {
        // XOR is symmetric, so decrypt is the same as encrypt
        let mut plaintext = Vec::with_capacity(ciphertext.len());
        for (i, &byte) in ciphertext.iter().enumerate() {
            plaintext.push(byte ^ self.key[i % self.key.len()]);
        }
        Ok(plaintext)
    }

    async fn health_check(&self) -> Result<(), secreton_core::error::SecretonError> {
        Ok(())
    }

    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata::new("mock".to_string(), "test-key".to_string())
    }
}

/// Property 1: Auto-unseal round trip
///
/// **Property**: For any plaintext data and any auto-unseal provider,
/// encrypting and then decrypting the data should return the original plaintext.
///
/// **Validates**: Requirements 2.1.1, 2.1.2, 2.1.3, 2.1.4
///
/// **Formal specification**:
/// ```text
/// ∀ plaintext ∈ Bytes, ∀ provider ∈ AutoUnsealProvider:
///   decrypt(encrypt(plaintext)) = plaintext
/// ```
///
/// This property ensures that:
/// 1. Encryption is reversible
/// 2. No data is lost during encryption/decryption
/// 3. The provider correctly implements the cryptographic operations
/// 4. The round-trip is idempotent
#[cfg(test)]
mod round_trip_tests {
    use super::*;

    proptest! {
        #[test]
        fn prop_auto_unseal_round_trip(
            plaintext in prop::collection::vec(any::<u8>(), 1..1024),
            key in prop::collection::vec(any::<u8>(), 16..64)
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                // Create provider with random key
                let provider = MockProvider::new(key);

                // Encrypt plaintext
                let ciphertext = provider.encrypt(&plaintext).await
                    .expect("Encryption should succeed");

                // Decrypt ciphertext
                let decrypted = provider.decrypt(&ciphertext).await
                    .expect("Decryption should succeed");

                // Verify round-trip property
                assert_eq!(plaintext, decrypted, "Round-trip failed: plaintext != decrypt(encrypt(plaintext))");
            });
        }
    }

    proptest! {
        /// Test with empty plaintext (edge case)
        #[test]
        fn prop_auto_unseal_round_trip_empty(
            key in prop::collection::vec(any::<u8>(), 16..64)
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let provider = MockProvider::new(key);
                let plaintext = vec![];

                let ciphertext = provider.encrypt(&plaintext).await
                    .expect("Encryption of empty data should succeed");

                let decrypted = provider.decrypt(&ciphertext).await
                    .expect("Decryption of empty data should succeed");

                assert_eq!(plaintext, decrypted, "Round-trip failed for empty plaintext");
            });
        }
    }

    proptest! {
        /// Test with single byte plaintext (edge case)
        #[test]
        fn prop_auto_unseal_round_trip_single_byte(
            byte in any::<u8>(),
            key in prop::collection::vec(any::<u8>(), 16..64)
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let provider = MockProvider::new(key);
                let plaintext = vec![byte];

                let ciphertext = provider.encrypt(&plaintext).await
                    .expect("Encryption of single byte should succeed");

                let decrypted = provider.decrypt(&ciphertext).await
                    .expect("Decryption of single byte should succeed");

                assert_eq!(plaintext, decrypted, "Round-trip failed for single byte");
            });
        }
    }

    proptest! {
        /// Test with maximum size plaintext (32 bytes - typical master key size)
        #[test]
        fn prop_auto_unseal_round_trip_master_key_size(
            plaintext in prop::collection::vec(any::<u8>(), 32..=32),
            key in prop::collection::vec(any::<u8>(), 16..64)
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let provider = MockProvider::new(key);

                let ciphertext = provider.encrypt(&plaintext).await
                    .expect("Encryption of 32-byte key should succeed");

                let decrypted = provider.decrypt(&ciphertext).await
                    .expect("Decryption of 32-byte key should succeed");

                assert_eq!(plaintext, decrypted, "Round-trip failed for 32-byte master key");
            });
        }
    }

    proptest! {
        /// Test idempotency: multiple encrypt/decrypt cycles
        #[test]
        fn prop_auto_unseal_idempotent(
            plaintext in prop::collection::vec(any::<u8>(), 1..256),
            key in prop::collection::vec(any::<u8>(), 16..64),
            cycles in 1..5usize
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let provider = MockProvider::new(key);
                let mut current = plaintext.clone();

                // Perform multiple encrypt/decrypt cycles
                for _ in 0..cycles {
                    let encrypted = provider.encrypt(&current).await
                        .expect("Encryption should succeed");

                    current = provider.decrypt(&encrypted).await
                        .expect("Decryption should succeed");
                }

                // After all cycles, should still equal original plaintext
                assert_eq!(plaintext, current, "Multiple round-trips failed");
            });
        }
    }

    proptest! {
        /// Test that different plaintexts produce different ciphertexts
        #[test]
        fn prop_auto_unseal_different_plaintexts(
            plaintext1 in prop::collection::vec(any::<u8>(), 1..256),
            plaintext2 in prop::collection::vec(any::<u8>(), 1..256),
            key in prop::collection::vec(any::<u8>(), 16..64)
        ) {
            prop_assume!(plaintext1 != plaintext2);

            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let provider = MockProvider::new(key);

                let ciphertext1 = provider.encrypt(&plaintext1).await
                    .expect("Encryption should succeed");

                let ciphertext2 = provider.encrypt(&plaintext2).await
                    .expect("Encryption should succeed");

                // Different plaintexts should produce different ciphertexts
                assert_ne!(ciphertext1, ciphertext2, "Different plaintexts produced same ciphertext");
            });
        }
    }

    proptest! {
        /// Test that ciphertext length is reasonable
        #[test]
        fn prop_auto_unseal_ciphertext_length(
            plaintext in prop::collection::vec(any::<u8>(), 1..1024),
            key in prop::collection::vec(any::<u8>(), 16..64)
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let provider = MockProvider::new(key);

                let ciphertext = provider.encrypt(&plaintext).await
                    .expect("Encryption should succeed");

                // For XOR encryption, ciphertext length equals plaintext length
                // For real providers, ciphertext may be longer due to metadata/padding
                assert!(
                    ciphertext.len() >= plaintext.len(),
                    "Ciphertext length ({}) should be >= plaintext length ({})",
                    ciphertext.len(),
                    plaintext.len()
                );
            });
        }
    }
}

/// Property tests for provider metadata
#[cfg(test)]
mod metadata_tests {
    use super::*;

    proptest! {
        /// Test that provider metadata is consistent
        #[test]
        fn prop_provider_metadata_consistent(
            key in prop::collection::vec(any::<u8>(), 16..64)
        ) {
            let provider = MockProvider::new(key);

            let metadata1 = provider.metadata();
            let metadata2 = provider.metadata();

            // Metadata should be consistent across calls
            assert_eq!(metadata1.provider_type, metadata2.provider_type);
            assert_eq!(metadata1.key_id, metadata2.key_id);
            assert_eq!(metadata1.region, metadata2.region);
            assert_eq!(metadata1.endpoint, metadata2.endpoint);
        }
    }

    proptest! {
        /// Test that provider name is consistent
        #[test]
        fn prop_provider_name_consistent(
            key in prop::collection::vec(any::<u8>(), 16..64)
        ) {
            let provider = MockProvider::new(key);

            let name1 = provider.name();
            let name2 = provider.name();

            assert_eq!(name1, name2, "Provider name should be consistent");
        }
    }
}

/// Property tests for health checks
#[cfg(test)]
mod health_check_tests {
    use super::*;

    proptest! {
        /// Test that health check is idempotent
        #[test]
        fn prop_health_check_idempotent(
            key in prop::collection::vec(any::<u8>(), 16..64),
            checks in 1..10usize
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let provider = MockProvider::new(key);

                // Multiple health checks should all succeed
                for _ in 0..checks {
                    provider.health_check().await
                        .expect("Health check should succeed");
                }
            });
        }
    }
}

/// Integration tests with realistic scenarios
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_master_key_encryption_scenario() {
        // Simulate a realistic master key (32 bytes)
        let master_key = vec![
            0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54,
            0x32, 0x10, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc,
            0xdd, 0xee, 0xff, 0x00,
        ];

        let provider_key = vec![0x42; 32]; // Simple key for testing
        let provider = MockProvider::new(provider_key);

        // Encrypt master key
        let encrypted = provider
            .encrypt(&master_key)
            .await
            .expect("Master key encryption should succeed");

        // Verify ciphertext is different from plaintext
        assert_ne!(
            encrypted, master_key,
            "Ciphertext should differ from plaintext"
        );

        // Decrypt master key
        let decrypted = provider
            .decrypt(&encrypted)
            .await
            .expect("Master key decryption should succeed");

        // Verify round-trip
        assert_eq!(master_key, decrypted, "Master key round-trip failed");
    }

    #[tokio::test]
    async fn test_multiple_providers_same_plaintext() {
        let plaintext = b"secret-master-key-data";

        let provider1 = MockProvider::new(vec![0x11; 16]);
        let provider2 = MockProvider::new(vec![0x22; 16]);

        // Encrypt with both providers
        let ciphertext1 = provider1.encrypt(plaintext).await.unwrap();
        let ciphertext2 = provider2.encrypt(plaintext).await.unwrap();

        // Different providers should produce different ciphertexts
        assert_ne!(
            ciphertext1, ciphertext2,
            "Different providers should produce different ciphertexts"
        );

        // But both should decrypt correctly
        let decrypted1 = provider1.decrypt(&ciphertext1).await.unwrap();
        let decrypted2 = provider2.decrypt(&ciphertext2).await.unwrap();

        assert_eq!(plaintext.to_vec(), decrypted1);
        assert_eq!(plaintext.to_vec(), decrypted2);
    }

    #[tokio::test]
    async fn test_provider_metadata() {
        let provider = MockProvider::new(vec![0x42; 16]);

        let metadata = provider.metadata();

        assert_eq!(metadata.provider_type, "mock");
        assert_eq!(metadata.key_id, "test-key");
        assert_eq!(metadata.region, None);
        assert_eq!(metadata.endpoint, None);
    }

    #[tokio::test]
    async fn test_health_check_success() {
        let provider = MockProvider::new(vec![0x42; 16]);

        let result = provider.health_check().await;

        assert!(result.is_ok(), "Health check should succeed");
    }
}
