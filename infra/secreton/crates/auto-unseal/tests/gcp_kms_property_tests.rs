//! Property-based tests for GCP KMS auto-unseal provider
//!
//! **Validates: Requirements 2.1.3**
//!
//! These tests verify that the GCP KMS provider correctly implements
//! the encrypt/decrypt round-trip property.

#[cfg(feature = "gcp-kms")]
use proptest::prelude::*;
#[cfg(feature = "gcp-kms")]
use secreton_auto_unseal::{config::GcpKmsConfig, gcp_kms::GcpKmsProvider, AutoUnsealProvider};

/// Property 1: Auto-unseal round trip (GCP KMS)
///
/// **Property**: For any plaintext data, encrypting with GCP KMS and then
/// decrypting should return the original plaintext.
///
/// **Validates**: Requirements 2.1.3
///
/// **Formal specification**:
/// ```text
/// ∀ plaintext ∈ Bytes:
///   decrypt_gcp_kms(encrypt_gcp_kms(plaintext)) = plaintext
/// ```
///
/// **Note**: These tests require GCP credentials and a valid KMS key.
/// They are integration tests and should be run with:
/// ```bash
/// GCP_KMS_KEY_NAME=projects/my-project/locations/us-central1/keyRings/my-ring/cryptoKeys/my-key \
/// GCP_PROJECT_ID=my-project \
/// GCP_LOCATION=us-central1 \
/// GCP_KEY_RING=my-ring \
/// GCP_CRYPTO_KEY=my-key \
/// cargo test --features gcp-kms -- --ignored
/// ```
#[cfg(all(test, feature = "gcp-kms"))]
mod gcp_kms_round_trip_tests {
    use super::*;

    /// Helper to create GCP KMS provider from environment variables
    async fn create_test_provider() -> Option<GcpKmsProvider> {
        let key_name = std::env::var("GCP_KMS_KEY_NAME").ok()?;
        let project_id = std::env::var("GCP_PROJECT_ID").ok()?;
        let location = std::env::var("GCP_LOCATION").ok()?;
        let key_ring = std::env::var("GCP_KEY_RING").ok()?;
        let crypto_key = std::env::var("GCP_CRYPTO_KEY").ok()?;

        let config = GcpKmsConfig {
            key_name,
            project_id,
            location,
            key_ring,
            crypto_key,
            credentials_file: std::env::var("GOOGLE_APPLICATION_CREDENTIALS").ok(),
        };

        GcpKmsProvider::new(config).await.ok()
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        #[ignore] // Requires GCP credentials
        fn prop_gcp_kms_round_trip(
            plaintext in prop::collection::vec(any::<u8>(), 1..1024)
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let provider = match create_test_provider().await {
                    Some(p) => p,
                    None => {
                        eprintln!("Skipping test: GCP_KMS_KEY_NAME not set");
                        return;
                    }
                };

                // Encrypt plaintext
                let ciphertext = provider.encrypt(&plaintext).await
                    .expect("GCP KMS encryption should succeed");

                // Decrypt ciphertext
                let decrypted = provider.decrypt(&ciphertext).await
                    .expect("GCP KMS decryption should succeed");

                // Verify round-trip property
                assert_eq!(
                    plaintext, decrypted,
                    "GCP KMS round-trip failed: plaintext != decrypt(encrypt(plaintext))"
                );
            });
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Test with master key size (32 bytes)
        #[test]
        #[ignore] // Requires GCP credentials
        fn prop_gcp_kms_round_trip_master_key_size(
            plaintext in prop::collection::vec(any::<u8>(), 32..=32)
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let provider = match create_test_provider().await {
                    Some(p) => p,
                    None => {
                        eprintln!("Skipping test: GCP_KMS_KEY_NAME not set");
                        return;
                    }
                };

                let ciphertext = provider.encrypt(&plaintext).await
                    .expect("GCP KMS encryption of 32-byte key should succeed");

                let decrypted = provider.decrypt(&ciphertext).await
                    .expect("GCP KMS decryption of 32-byte key should succeed");

                assert_eq!(
                    plaintext, decrypted,
                    "GCP KMS round-trip failed for 32-byte master key"
                );
            });
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Test idempotency: multiple encrypt/decrypt cycles
        #[test]
        #[ignore] // Requires GCP credentials
        fn prop_gcp_kms_idempotent(
            plaintext in prop::collection::vec(any::<u8>(), 1..256),
            cycles in 1..3usize // Fewer cycles to avoid rate limits
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let provider = match create_test_provider().await {
                    Some(p) => p,
                    None => {
                        eprintln!("Skipping test: GCP_KMS_KEY_NAME not set");
                        return;
                    }
                };

                let mut current = plaintext.clone();

                // Perform multiple encrypt/decrypt cycles
                for _ in 0..cycles {
                    let encrypted = provider.encrypt(&current).await
                        .expect("GCP KMS encryption should succeed");

                    current = provider.decrypt(&encrypted).await
                        .expect("GCP KMS decryption should succeed");
                }

                // After all cycles, should still equal original plaintext
                assert_eq!(plaintext, current, "GCP KMS multiple round-trips failed");
            });
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Test that different plaintexts produce different ciphertexts
        #[test]
        #[ignore] // Requires GCP credentials
        fn prop_gcp_kms_different_plaintexts(
            plaintext1 in prop::collection::vec(any::<u8>(), 1..256),
            plaintext2 in prop::collection::vec(any::<u8>(), 1..256)
        ) {
            prop_assume!(plaintext1 != plaintext2);

            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let provider = match create_test_provider().await {
                    Some(p) => p,
                    None => {
                        eprintln!("Skipping test: GCP_KMS_KEY_NAME not set");
                        return;
                    }
                };

                let ciphertext1 = provider.encrypt(&plaintext1).await
                    .expect("GCP KMS encryption should succeed");

                let ciphertext2 = provider.encrypt(&plaintext2).await
                    .expect("GCP KMS encryption should succeed");

                // Different plaintexts should produce different ciphertexts
                assert_ne!(
                    ciphertext1, ciphertext2,
                    "Different plaintexts produced same ciphertext with GCP KMS"
                );
            });
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Test that encrypting the same plaintext twice produces different ciphertexts
        /// (due to nonce/IV randomization)
        #[test]
        #[ignore] // Requires GCP credentials
        fn prop_gcp_kms_non_deterministic_encryption(
            plaintext in prop::collection::vec(any::<u8>(), 1..256)
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let provider = match create_test_provider().await {
                    Some(p) => p,
                    None => {
                        eprintln!("Skipping test: GCP_KMS_KEY_NAME not set");
                        return;
                    }
                };

                let ciphertext1 = provider.encrypt(&plaintext).await
                    .expect("GCP KMS encryption should succeed");

                let ciphertext2 = provider.encrypt(&plaintext).await
                    .expect("GCP KMS encryption should succeed");

                // Same plaintext encrypted twice should produce different ciphertexts
                // (due to randomized nonce/IV)
                assert_ne!(
                    ciphertext1, ciphertext2,
                    "Same plaintext encrypted twice produced identical ciphertext"
                );

                // But both should decrypt to the same plaintext
                let decrypted1 = provider.decrypt(&ciphertext1).await
                    .expect("GCP KMS decryption should succeed");
                let decrypted2 = provider.decrypt(&ciphertext2).await
                    .expect("GCP KMS decryption should succeed");

                assert_eq!(plaintext, decrypted1);
                assert_eq!(plaintext, decrypted2);
            });
        }
    }
}

/// Integration tests with realistic scenarios
#[cfg(all(test, feature = "gcp-kms"))]
mod gcp_kms_integration_tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires GCP credentials
    async fn test_gcp_kms_master_key_encryption() {
        let provider = match create_test_provider().await {
            Some(p) => p,
            None => {
                eprintln!("Skipping test: GCP_KMS_KEY_NAME not set");
                return;
            }
        };

        // Simulate a realistic master key (32 bytes)
        let master_key = vec![
            0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54,
            0x32, 0x10, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc,
            0xdd, 0xee, 0xff, 0x00,
        ];

        // Encrypt master key
        let encrypted = provider
            .encrypt(&master_key)
            .await
            .expect("GCP KMS master key encryption should succeed");

        // Verify ciphertext is different from plaintext
        assert_ne!(
            encrypted, master_key,
            "Ciphertext should differ from plaintext"
        );

        // Decrypt master key
        let decrypted = provider
            .decrypt(&encrypted)
            .await
            .expect("GCP KMS master key decryption should succeed");

        // Verify round-trip
        assert_eq!(master_key, decrypted, "GCP KMS master key round-trip failed");
    }

    #[tokio::test]
    #[ignore] // Requires GCP credentials
    async fn test_gcp_kms_health_check() {
        let provider = match create_test_provider().await {
            Some(p) => p,
            None => {
                eprintln!("Skipping test: GCP_KMS_KEY_NAME not set");
                return;
            }
        };

        let result = provider.health_check().await;

        assert!(result.is_ok(), "GCP KMS health check should succeed");
    }

    #[tokio::test]
    #[ignore] // Requires GCP credentials
    async fn test_gcp_kms_metadata() {
        let provider = match create_test_provider().await {
            Some(p) => p,
            None => {
                eprintln!("Skipping test: GCP_KMS_KEY_NAME not set");
                return;
            }
        };

        let metadata = provider.metadata();

        assert_eq!(metadata.provider_type, "gcp-kms");
        assert!(!metadata.key_id.is_empty());
        assert!(metadata.region.is_some());
    }

    #[tokio::test]
    #[ignore] // Requires GCP credentials
    async fn test_gcp_kms_empty_plaintext() {
        let provider = match create_test_provider().await {
            Some(p) => p,
            None => {
                eprintln!("Skipping test: GCP_KMS_KEY_NAME not set");
                return;
            }
        };

        // Test with empty plaintext
        let plaintext = vec![];

        let ciphertext = provider
            .encrypt(&plaintext)
            .await
            .expect("GCP KMS encryption of empty plaintext should succeed");

        let decrypted = provider
            .decrypt(&ciphertext)
            .await
            .expect("GCP KMS decryption should succeed");

        assert_eq!(plaintext, decrypted, "Empty plaintext round-trip failed");
    }

    #[tokio::test]
    #[ignore] // Requires GCP credentials
    async fn test_gcp_kms_large_plaintext() {
        let provider = match create_test_provider().await {
            Some(p) => p,
            None => {
                eprintln!("Skipping test: GCP_KMS_KEY_NAME not set");
                return;
            }
        };

        // Test with large plaintext (4KB)
        let plaintext = vec![0x42; 4096];

        let ciphertext = provider
            .encrypt(&plaintext)
            .await
            .expect("GCP KMS encryption of large plaintext should succeed");

        let decrypted = provider
            .decrypt(&ciphertext)
            .await
            .expect("GCP KMS decryption should succeed");

        assert_eq!(plaintext, decrypted, "Large plaintext round-trip failed");
    }

    #[tokio::test]
    #[ignore] // Requires GCP credentials
    async fn test_gcp_kms_invalid_ciphertext() {
        let provider = match create_test_provider().await {
            Some(p) => p,
            None => {
                eprintln!("Skipping test: GCP_KMS_KEY_NAME not set");
                return;
            }
        };

        // Test with invalid ciphertext
        let invalid_ciphertext = vec![0xff; 64];

        let result = provider.decrypt(&invalid_ciphertext).await;

        assert!(
            result.is_err(),
            "Decryption of invalid ciphertext should fail"
        );
    }

    #[tokio::test]
    #[ignore] // Requires GCP credentials
    async fn test_gcp_kms_concurrent_operations() {
        use std::sync::Arc;

        let provider = match create_test_provider().await {
            Some(p) => Arc::new(p),
            None => {
                eprintln!("Skipping test: GCP_KMS_KEY_NAME not set");
                return;
            }
        };

        // Test concurrent encrypt/decrypt operations
        let mut handles = vec![];

        for i in 0..10 {
            let provider_clone = Arc::clone(&provider);
            let handle = tokio::spawn(async move {
                let plaintext = vec![i as u8; 32];

                let ciphertext = provider_clone
                    .encrypt(&plaintext)
                    .await
                    .expect("Encryption should succeed");

                let decrypted = provider_clone
                    .decrypt(&ciphertext)
                    .await
                    .expect("Decryption should succeed");

                assert_eq!(plaintext, decrypted);
            });

            handles.push(handle);
        }

        // Wait for all operations to complete
        for handle in handles {
            handle.await.expect("Task should complete successfully");
        }
    }

    /// Helper to create GCP KMS provider from environment variables
    async fn create_test_provider() -> Option<GcpKmsProvider> {
        let key_name = std::env::var("GCP_KMS_KEY_NAME").ok()?;
        let project_id = std::env::var("GCP_PROJECT_ID").ok()?;
        let location = std::env::var("GCP_LOCATION").ok()?;
        let key_ring = std::env::var("GCP_KEY_RING").ok()?;
        let crypto_key = std::env::var("GCP_CRYPTO_KEY").ok()?;

        let config = GcpKmsConfig {
            key_name,
            project_id,
            location,
            key_ring,
            crypto_key,
            credentials_file: std::env::var("GOOGLE_APPLICATION_CREDENTIALS").ok(),
        };

        GcpKmsProvider::new(config).await.ok()
    }
}
