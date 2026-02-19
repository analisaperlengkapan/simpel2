//! Property-based tests for AWS KMS auto-unseal provider
//!
//! **Validates: Requirements 2.1.2**
//!
//! These tests verify that the AWS KMS provider correctly implements
//! the encrypt/decrypt round-trip property.

#[cfg(feature = "aws-kms")]
use proptest::prelude::*;
#[cfg(feature = "aws-kms")]
use secreton_auto_unseal::{aws_kms::AwsKmsProvider, config::AwsKmsConfig, AutoUnsealProvider};

/// Property 1: Auto-unseal round trip (AWS KMS)
///
/// **Property**: For any plaintext data, encrypting with AWS KMS and then
/// decrypting should return the original plaintext.
///
/// **Validates**: Requirements 2.1.2
///
/// **Formal specification**:
/// ```text
/// ∀ plaintext ∈ Bytes:
///   decrypt_aws_kms(encrypt_aws_kms(plaintext)) = plaintext
/// ```
///
/// **Note**: These tests require AWS credentials and a valid KMS key.
/// They are integration tests and should be run with:
/// ```bash
/// AWS_KMS_KEY_ID=alias/secreton-test \
/// AWS_REGION=us-east-1 \
/// cargo test --features aws-kms -- --ignored
/// ```
#[cfg(all(test, feature = "aws-kms"))]
mod aws_kms_round_trip_tests {
    use super::*;

    /// Helper to create AWS KMS provider from environment variables
    async fn create_test_provider() -> Option<AwsKmsProvider> {
        let key_id = std::env::var("AWS_KMS_KEY_ID").ok()?;
        let region = std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string());

        let config = AwsKmsConfig {
            key_id,
            region,
            endpoint: std::env::var("AWS_KMS_ENDPOINT").ok(),
            access_key_id: std::env::var("AWS_ACCESS_KEY_ID").ok(),
            secret_access_key: std::env::var("AWS_SECRET_ACCESS_KEY").ok(),
            session_token: std::env::var("AWS_SESSION_TOKEN").ok(),
        };

        AwsKmsProvider::new(config).await.ok()
    }

    proptest! {
        #[test]
        #[ignore] // Requires AWS credentials
        fn prop_aws_kms_round_trip(
            plaintext in prop::collection::vec(any::<u8>(), 1..1024)
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let provider = match create_test_provider().await {
                    Some(p) => p,
                    None => {
                        eprintln!("Skipping test: AWS_KMS_KEY_ID not set");
                        return;
                    }
                };

                // Encrypt plaintext
                let ciphertext = provider.encrypt(&plaintext).await
                    .expect("AWS KMS encryption should succeed");

                // Decrypt ciphertext
                let decrypted = provider.decrypt(&ciphertext).await
                    .expect("AWS KMS decryption should succeed");

                // Verify round-trip property
                assert_eq!(
                    plaintext, decrypted,
                    "AWS KMS round-trip failed: plaintext != decrypt(encrypt(plaintext))"
                );
            });
        }
    }

    proptest! {
        /// Test with master key size (32 bytes)
        #[test]
        #[ignore] // Requires AWS credentials
        fn prop_aws_kms_round_trip_master_key_size(
            plaintext in prop::collection::vec(any::<u8>(), 32..=32)
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let provider = match create_test_provider().await {
                    Some(p) => p,
                    None => {
                        eprintln!("Skipping test: AWS_KMS_KEY_ID not set");
                        return;
                    }
                };

                let ciphertext = provider.encrypt(&plaintext).await
                    .expect("AWS KMS encryption of 32-byte key should succeed");

                let decrypted = provider.decrypt(&ciphertext).await
                    .expect("AWS KMS decryption of 32-byte key should succeed");

                assert_eq!(
                    plaintext, decrypted,
                    "AWS KMS round-trip failed for 32-byte master key"
                );
            });
        }
    }

    proptest! {
        /// Test idempotency: multiple encrypt/decrypt cycles
        #[test]
        #[ignore] // Requires AWS credentials
        fn prop_aws_kms_idempotent(
            plaintext in prop::collection::vec(any::<u8>(), 1..256),
            cycles in 1..3usize // Fewer cycles to avoid rate limits
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let provider = match create_test_provider().await {
                    Some(p) => p,
                    None => {
                        eprintln!("Skipping test: AWS_KMS_KEY_ID not set");
                        return;
                    }
                };

                let mut current = plaintext.clone();

                // Perform multiple encrypt/decrypt cycles
                for _ in 0..cycles {
                    let encrypted = provider.encrypt(&current).await
                        .expect("AWS KMS encryption should succeed");

                    current = provider.decrypt(&encrypted).await
                        .expect("AWS KMS decryption should succeed");
                }

                // After all cycles, should still equal original plaintext
                assert_eq!(plaintext, current, "AWS KMS multiple round-trips failed");
            });
        }
    }

    proptest! {
        /// Test that different plaintexts produce different ciphertexts
        #[test]
        #[ignore] // Requires AWS credentials
        fn prop_aws_kms_different_plaintexts(
            plaintext1 in prop::collection::vec(any::<u8>(), 1..256),
            plaintext2 in prop::collection::vec(any::<u8>(), 1..256)
        ) {
            prop_assume!(plaintext1 != plaintext2);

            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let provider = match create_test_provider().await {
                    Some(p) => p,
                    None => {
                        eprintln!("Skipping test: AWS_KMS_KEY_ID not set");
                        return;
                    }
                };

                let ciphertext1 = provider.encrypt(&plaintext1).await
                    .expect("AWS KMS encryption should succeed");

                let ciphertext2 = provider.encrypt(&plaintext2).await
                    .expect("AWS KMS encryption should succeed");

                // Different plaintexts should produce different ciphertexts
                assert_ne!(
                    ciphertext1, ciphertext2,
                    "Different plaintexts produced same ciphertext with AWS KMS"
                );
            });
        }
    }
}

/// Integration tests with realistic scenarios
#[cfg(all(test, feature = "aws-kms"))]
mod aws_kms_integration_tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires AWS credentials
    async fn test_aws_kms_master_key_encryption() {
        let provider = match create_test_provider().await {
            Some(p) => p,
            None => {
                eprintln!("Skipping test: AWS_KMS_KEY_ID not set");
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
            .expect("AWS KMS master key encryption should succeed");

        // Verify ciphertext is different from plaintext
        assert_ne!(
            encrypted, master_key,
            "Ciphertext should differ from plaintext"
        );

        // Decrypt master key
        let decrypted = provider
            .decrypt(&encrypted)
            .await
            .expect("AWS KMS master key decryption should succeed");

        // Verify round-trip
        assert_eq!(master_key, decrypted, "AWS KMS master key round-trip failed");
    }

    #[tokio::test]
    #[ignore] // Requires AWS credentials
    async fn test_aws_kms_health_check() {
        let provider = match create_test_provider().await {
            Some(p) => p,
            None => {
                eprintln!("Skipping test: AWS_KMS_KEY_ID not set");
                return;
            }
        };

        let result = provider.health_check().await;

        assert!(result.is_ok(), "AWS KMS health check should succeed");
    }

    #[tokio::test]
    #[ignore] // Requires AWS credentials
    async fn test_aws_kms_metadata() {
        let provider = match create_test_provider().await {
            Some(p) => p,
            None => {
                eprintln!("Skipping test: AWS_KMS_KEY_ID not set");
                return;
            }
        };

        let metadata = provider.metadata();

        assert_eq!(metadata.provider_type, "aws-kms");
        assert!(!metadata.key_id.is_empty());
        assert!(metadata.region.is_some());
    }

    /// Helper to create AWS KMS provider from environment variables
    async fn create_test_provider() -> Option<AwsKmsProvider> {
        let key_id = std::env::var("AWS_KMS_KEY_ID").ok()?;
        let region = std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string());

        let config = AwsKmsConfig {
            key_id,
            region,
            endpoint: std::env::var("AWS_KMS_ENDPOINT").ok(),
            access_key_id: std::env::var("AWS_ACCESS_KEY_ID").ok(),
            secret_access_key: std::env::var("AWS_SECRET_ACCESS_KEY").ok(),
            session_token: std::env::var("AWS_SESSION_TOKEN").ok(),
        };

        AwsKmsProvider::new(config).await.ok()
    }
}
