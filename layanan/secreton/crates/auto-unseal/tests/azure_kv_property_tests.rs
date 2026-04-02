//! Property-based tests for Azure Key Vault auto-unseal provider
//!
//! **Validates: Requirements 2.1.4**
//!
//! These tests verify that the Azure Key Vault provider correctly implements
//! the encrypt/decrypt round-trip property.

#[cfg(feature = "azure-kv")]
use proptest::prelude::*;
#[cfg(feature = "azure-kv")]
use secreton_auto_unseal::{
    AutoUnsealProvider, azure_kv::AzureKeyVaultProvider, config::AzureKeyVaultConfig,
};

/// Property 1: Auto-unseal round trip (Azure Key Vault)
///
/// **Property**: For any plaintext data, encrypting with Azure Key Vault and then
/// decrypting should return the original plaintext.
///
/// **Validates**: Requirements 2.1.4
///
/// **Formal specification**:
/// ```text
/// ∀ plaintext ∈ Bytes:
///   decrypt_azure_kv(encrypt_azure_kv(plaintext)) = plaintext
/// ```
///
/// **Note**: These tests require Azure credentials and a valid Key Vault key.
/// They are integration tests and should be run with:
/// ```bash
/// AZURE_SECRETON_NAME=secreton-test \
/// AZURE_KEY_NAME=auto-unseal-key \
/// cargo test --features azure-kv -- --ignored
/// ```
///
/// **Authentication**: Uses Azure default credential chain:
/// - Managed Identity (recommended for Azure VMs/AKS)
/// - Environment variables (AZURE_TENANT_ID, AZURE_CLIENT_ID, AZURE_CLIENT_SECRET)
/// - Azure CLI credentials
#[cfg(all(test, feature = "azure-kv"))]
mod azure_kv_round_trip_tests {
    use super::*;

    /// Helper to create Azure Key Vault provider from environment variables
    async fn create_test_provider() -> Option<AzureKeyVaultProvider> {
        let vault_name = std::env::var("AZURE_SECRETON_NAME").ok()?;
        let key_name = std::env::var("AZURE_KEY_NAME").ok()?;

        let config = AzureKeyVaultConfig {
            vault_name,
            key_name,
            key_version: std::env::var("AZURE_KEY_VERSION").ok(),
            tenant_id: std::env::var("AZURE_TENANT_ID").ok(),
            client_id: std::env::var("AZURE_CLIENT_ID").ok(),
            client_secret: std::env::var("AZURE_CLIENT_SECRET").ok(),
        };

        AzureKeyVaultProvider::new(config).await.ok()
    }

    proptest! {
        #[test]
        #[ignore] // Requires Azure credentials
        fn prop_azure_kv_round_trip(
            plaintext in prop::collection::vec(any::<u8>(), 1..1024)
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let provider = match create_test_provider().await {
                    Some(p) => p,
                    None => {
                        eprintln!("Skipping test: AZURE_SECRETON_NAME or AZURE_KEY_NAME not set");
                        return;
                    }
                };

                // Encrypt plaintext
                let ciphertext = provider.encrypt(&plaintext).await
                    .expect("Azure Key Vault encryption should succeed");

                // Decrypt ciphertext
                let decrypted = provider.decrypt(&ciphertext).await
                    .expect("Azure Key Vault decryption should succeed");

                // Verify round-trip property
                assert_eq!(
                    plaintext, decrypted,
                    "Azure Key Vault round-trip failed: plaintext != decrypt(encrypt(plaintext))"
                );
            });
        }
    }

    proptest! {
        /// Test with master key size (32 bytes)
        #[test]
        #[ignore] // Requires Azure credentials
        fn prop_azure_kv_round_trip_master_key_size(
            plaintext in prop::collection::vec(any::<u8>(), 32..=32)
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let provider = match create_test_provider().await {
                    Some(p) => p,
                    None => {
                        eprintln!("Skipping test: AZURE_SECRETON_NAME or AZURE_KEY_NAME not set");
                        return;
                    }
                };

                let ciphertext = provider.encrypt(&plaintext).await
                    .expect("Azure Key Vault encryption of 32-byte key should succeed");

                let decrypted = provider.decrypt(&ciphertext).await
                    .expect("Azure Key Vault decryption of 32-byte key should succeed");

                assert_eq!(
                    plaintext, decrypted,
                    "Azure Key Vault round-trip failed for 32-byte master key"
                );
            });
        }
    }

    proptest! {
        /// Test idempotency: multiple encrypt/decrypt cycles
        #[test]
        #[ignore] // Requires Azure credentials
        fn prop_azure_kv_idempotent(
            plaintext in prop::collection::vec(any::<u8>(), 1..256),
            cycles in 1..3usize // Fewer cycles to avoid rate limits
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let provider = match create_test_provider().await {
                    Some(p) => p,
                    None => {
                        eprintln!("Skipping test: AZURE_SECRETON_NAME or AZURE_KEY_NAME not set");
                        return;
                    }
                };

                let mut current = plaintext.clone();

                // Perform multiple encrypt/decrypt cycles
                for _ in 0..cycles {
                    let encrypted = provider.encrypt(&current).await
                        .expect("Azure Key Vault encryption should succeed");

                    current = provider.decrypt(&encrypted).await
                        .expect("Azure Key Vault decryption should succeed");
                }

                // After all cycles, should still equal original plaintext
                assert_eq!(plaintext, current, "Azure Key Vault multiple round-trips failed");
            });
        }
    }

    proptest! {
        /// Test that different plaintexts produce different ciphertexts
        #[test]
        #[ignore] // Requires Azure credentials
        fn prop_azure_kv_different_plaintexts(
            plaintext1 in prop::collection::vec(any::<u8>(), 1..256),
            plaintext2 in prop::collection::vec(any::<u8>(), 1..256)
        ) {
            prop_assume!(plaintext1 != plaintext2);

            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let provider = match create_test_provider().await {
                    Some(p) => p,
                    None => {
                        eprintln!("Skipping test: AZURE_SECRETON_NAME or AZURE_KEY_NAME not set");
                        return;
                    }
                };

                let ciphertext1 = provider.encrypt(&plaintext1).await
                    .expect("Azure Key Vault encryption should succeed");

                let ciphertext2 = provider.encrypt(&plaintext2).await
                    .expect("Azure Key Vault encryption should succeed");

                // Different plaintexts should produce different ciphertexts
                assert_ne!(
                    ciphertext1, ciphertext2,
                    "Different plaintexts produced same ciphertext with Azure Key Vault"
                );
            });
        }
    }

    proptest! {
        /// Test that encrypting the same plaintext twice produces different ciphertexts
        /// (due to random padding in RSA-OAEP)
        #[test]
        #[ignore] // Requires Azure credentials
        fn prop_azure_kv_non_deterministic_encryption(
            plaintext in prop::collection::vec(any::<u8>(), 1..256)
        ) {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let provider = match create_test_provider().await {
                    Some(p) => p,
                    None => {
                        eprintln!("Skipping test: AZURE_SECRETON_NAME or AZURE_KEY_NAME not set");
                        return;
                    }
                };

                let ciphertext1 = provider.encrypt(&plaintext).await
                    .expect("Azure Key Vault encryption should succeed");

                let ciphertext2 = provider.encrypt(&plaintext).await
                    .expect("Azure Key Vault encryption should succeed");

                // Same plaintext should produce different ciphertexts (RSA-OAEP uses random padding)
                assert_ne!(
                    ciphertext1, ciphertext2,
                    "Same plaintext produced identical ciphertext (encryption should be non-deterministic)"
                );

                // But both should decrypt to the same plaintext
                let decrypted1 = provider.decrypt(&ciphertext1).await
                    .expect("Azure Key Vault decryption should succeed");
                let decrypted2 = provider.decrypt(&ciphertext2).await
                    .expect("Azure Key Vault decryption should succeed");

                assert_eq!(plaintext, decrypted1);
                assert_eq!(plaintext, decrypted2);
            });
        }
    }
}

/// Integration tests with realistic scenarios
#[cfg(all(test, feature = "azure-kv"))]
mod azure_kv_integration_tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Azure credentials
    async fn test_azure_kv_master_key_encryption() {
        let provider = match create_test_provider().await {
            Some(p) => p,
            None => {
                eprintln!("Skipping test: AZURE_SECRETON_NAME or AZURE_KEY_NAME not set");
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
            .expect("Azure Key Vault master key encryption should succeed");

        // Verify ciphertext is different from plaintext
        assert_ne!(
            encrypted, master_key,
            "Ciphertext should differ from plaintext"
        );

        // Decrypt master key
        let decrypted = provider
            .decrypt(&encrypted)
            .await
            .expect("Azure Key Vault master key decryption should succeed");

        // Verify round-trip
        assert_eq!(
            master_key, decrypted,
            "Azure Key Vault master key round-trip failed"
        );
    }

    #[tokio::test]
    #[ignore] // Requires Azure credentials
    async fn test_azure_kv_health_check() {
        let provider = match create_test_provider().await {
            Some(p) => p,
            None => {
                eprintln!("Skipping test: AZURE_SECRETON_NAME or AZURE_KEY_NAME not set");
                return;
            }
        };

        let result = provider.health_check().await;

        assert!(
            result.is_ok(),
            "Azure Key Vault health check should succeed"
        );
    }

    #[tokio::test]
    #[ignore] // Requires Azure credentials
    async fn test_azure_kv_metadata() {
        let provider = match create_test_provider().await {
            Some(p) => p,
            None => {
                eprintln!("Skipping test: AZURE_SECRETON_NAME or AZURE_KEY_NAME not set");
                return;
            }
        };

        let metadata = provider.metadata();

        assert_eq!(metadata.provider_type, "azure-key-vault");
        assert!(!metadata.key_id.is_empty());
        assert!(metadata.endpoint.is_some());
        assert!(
            metadata
                .endpoint
                .as_ref()
                .unwrap()
                .contains("vault.azure.net")
        );
    }

    #[tokio::test]
    #[ignore] // Requires Azure credentials
    async fn test_azure_kv_invalid_ciphertext() {
        let provider = match create_test_provider().await {
            Some(p) => p,
            None => {
                eprintln!("Skipping test: AZURE_SECRETON_NAME or AZURE_KEY_NAME not set");
                return;
            }
        };

        // Try to decrypt invalid ciphertext
        let invalid_ciphertext = vec![0x00; 256];
        let result = provider.decrypt(&invalid_ciphertext).await;

        assert!(result.is_err(), "Decrypting invalid ciphertext should fail");
    }

    #[tokio::test]
    #[ignore] // Requires Azure credentials
    async fn test_azure_kv_empty_plaintext() {
        let provider = match create_test_provider().await {
            Some(p) => p,
            None => {
                eprintln!("Skipping test: AZURE_SECRETON_NAME or AZURE_KEY_NAME not set");
                return;
            }
        };

        // Azure Key Vault should handle empty plaintext
        let empty_plaintext = vec![];

        // Note: Azure Key Vault may reject empty plaintext
        // This test verifies the behavior is consistent
        let encrypt_result = provider.encrypt(&empty_plaintext).await;

        if let Ok(ciphertext) = encrypt_result {
            let decrypted = provider
                .decrypt(&ciphertext)
                .await
                .expect("Decryption should succeed if encryption succeeded");
            assert_eq!(empty_plaintext, decrypted);
        }
        // If encryption fails, that's also acceptable behavior
    }

    #[tokio::test]
    #[ignore] // Requires Azure credentials
    async fn test_azure_kv_large_plaintext() {
        let provider = match create_test_provider().await {
            Some(p) => p,
            None => {
                eprintln!("Skipping test: AZURE_SECRETON_NAME or AZURE_KEY_NAME not set");
                return;
            }
        };

        // Test with maximum size for RSA-OAEP-256 (depends on key size)
        // For 2048-bit RSA key with OAEP-256: max plaintext = 190 bytes
        // For 4096-bit RSA key with OAEP-256: max plaintext = 446 bytes
        let large_plaintext = vec![0x42; 190];

        let encrypted = provider
            .encrypt(&large_plaintext)
            .await
            .expect("Azure Key Vault should encrypt 190-byte plaintext");

        let decrypted = provider
            .decrypt(&encrypted)
            .await
            .expect("Azure Key Vault should decrypt 190-byte ciphertext");

        assert_eq!(large_plaintext, decrypted);
    }

    /// Helper to create Azure Key Vault provider from environment variables
    async fn create_test_provider() -> Option<AzureKeyVaultProvider> {
        let vault_name = std::env::var("AZURE_SECRETON_NAME").ok()?;
        let key_name = std::env::var("AZURE_KEY_NAME").ok()?;

        let config = AzureKeyVaultConfig {
            vault_name,
            key_name,
            key_version: std::env::var("AZURE_KEY_VERSION").ok(),
            tenant_id: std::env::var("AZURE_TENANT_ID").ok(),
            client_id: std::env::var("AZURE_CLIENT_ID").ok(),
            client_secret: std::env::var("AZURE_CLIENT_SECRET").ok(),
        };

        AzureKeyVaultProvider::new(config).await.ok()
    }
}

/// Error handling property tests
#[cfg(all(test, feature = "azure-kv"))]
mod azure_kv_error_tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Azure credentials
    async fn test_azure_kv_invalid_key_name() {
        let vault_name =
            std::env::var("AZURE_SECRETON_NAME").unwrap_or_else(|_| "test-vault".to_string());

        let config = AzureKeyVaultConfig {
            vault_name,
            key_name: "nonexistent-key-12345".to_string(),
            key_version: None,
            tenant_id: std::env::var("AZURE_TENANT_ID").ok(),
            client_id: std::env::var("AZURE_CLIENT_ID").ok(),
            client_secret: std::env::var("AZURE_CLIENT_SECRET").ok(),
        };

        let provider = match AzureKeyVaultProvider::new(config).await {
            Ok(p) => p,
            Err(_) => {
                eprintln!("Skipping test: Could not create provider");
                return;
            }
        };

        // Health check should fail for nonexistent key
        let result = provider.health_check().await;
        assert!(
            result.is_err(),
            "Health check should fail for nonexistent key"
        );
    }

    #[tokio::test]
    async fn test_azure_kv_invalid_vault_name() {
        let config = AzureKeyVaultConfig {
            vault_name: "".to_string(),
            key_name: "test-key".to_string(),
            key_version: None,
            tenant_id: None,
            client_id: None,
            client_secret: None,
        };

        let result = AzureKeyVaultProvider::new(config).await;
        assert!(result.is_err(), "Should fail with empty vault name");
    }

    #[tokio::test]
    async fn test_azure_kv_invalid_key_name_format() {
        let config = AzureKeyVaultConfig {
            vault_name: "test-vault".to_string(),
            key_name: "".to_string(),
            key_version: None,
            tenant_id: None,
            client_id: None,
            client_secret: None,
        };

        let result = AzureKeyVaultProvider::new(config).await;
        assert!(result.is_err(), "Should fail with empty key name");
    }
}
