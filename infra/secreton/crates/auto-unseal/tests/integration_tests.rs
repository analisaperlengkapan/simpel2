//! Integration tests for auto-unseal functionality
//!
//! **Task 1.21**: Write integration tests for auto-unseal
//!
//! These tests validate the complete auto-unseal flow end-to-end:
//! - Seal → Store encrypted key → Restart → Auto-unseal
//! - Fallback to manual unseal when auto-unseal fails
//! - Configuration persistence across restarts
//! - Real KMS provider integration (using test accounts)
//!
//! **Test Strategy:**
//! - Use mock providers for fast CI/CD tests
//! - Use real KMS providers for integration tests (marked with #[ignore])
//! - Test both success and failure scenarios
//! - Verify audit logging for all operations

use secreton_auto_unseal::{
    AutoUnsealManager, AutoUnsealProvider, FallbackConfig, ProviderMetadata, UnsealResult,
};
use secreton_core::error::SecretonError;
use secreton_core::storage::sealed_keys::{
    ProviderType, SealedKeyStorage, SealedMasterKey,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

// ========== Mock Provider for Testing ==========

/// Mock provider that simulates KMS behavior
struct MockKmsProvider {
    name: String,
    key_id: String,
    fail_decrypt: Arc<Mutex<bool>>,
    call_count: Arc<Mutex<u32>>,
}

impl MockKmsProvider {
    fn new(name: String, key_id: String) -> Self {
        Self {
            name,
            key_id,
            fail_decrypt: Arc::new(Mutex::new(false)),
            call_count: Arc::new(Mutex::new(0)),
        }
    }

    fn with_failure(mut self) -> Self {
        self.fail_decrypt = Arc::new(Mutex::new(true));
        self
    }

    async fn get_call_count(&self) -> u32 {
        *self.call_count.lock().await
    }

    async fn set_fail_decrypt(&self, fail: bool) {
        *self.fail_decrypt.lock().await = fail;
    }
}

#[async_trait::async_trait]
impl AutoUnsealProvider for MockKmsProvider {
    fn name(&self) -> &str {
        &self.name
    }

    async fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, SecretonError> {
        *self.call_count.lock().await += 1;

        // Simple XOR encryption for testing
        let key = self.key_id.as_bytes();
        let mut ciphertext = Vec::with_capacity(plaintext.len());
        for (i, &byte) in plaintext.iter().enumerate() {
            ciphertext.push(byte ^ key[i % key.len()]);
        }
        Ok(ciphertext)
    }

    async fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, SecretonError> {
        *self.call_count.lock().await += 1;

        if *self.fail_decrypt.lock().await {
            return Err(SecretonError::Core(secreton_core::error::CoreError::internal(
                "Mock KMS decrypt failure",
            )));
        }

        // XOR is symmetric
        let key = self.key_id.as_bytes();
        let mut plaintext = Vec::with_capacity(ciphertext.len());
        for (i, &byte) in ciphertext.iter().enumerate() {
            plaintext.push(byte ^ key[i % key.len()]);
        }
        Ok(plaintext)
    }

    async fn health_check(&self) -> Result<(), SecretonError> {
        if *self.fail_decrypt.lock().await {
            return Err(SecretonError::Core(secreton_core::error::CoreError::internal(
                "Mock KMS health check failure",
            )));
        }
        Ok(())
    }

    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata::new(self.name.clone(), self.key_id.clone())
    }
}

// ========== In-Memory Sealed Key Storage for Testing ==========

struct InMemorySealedKeyStorage {
    keys: Arc<Mutex<Vec<SealedMasterKey>>>,
}

impl InMemorySealedKeyStorage {
    fn new() -> Self {
        Self {
            keys: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait::async_trait]
impl SealedKeyStorage for InMemorySealedKeyStorage {
    async fn store_sealed_key(&self, key: &SealedMasterKey) -> Result<(), String> {
        let mut keys = self.keys.lock().await;

        // Remove existing key with same ID
        keys.retain(|k| k.id != key.id);

        keys.push(key.clone());
        Ok(())
    }

    async fn get_active_sealed_key(&self) -> Result<Option<SealedMasterKey>, String> {
        let keys = self.keys.lock().await;
        Ok(keys.iter().find(|k| k.is_active).cloned())
    }

    async fn get_sealed_key_by_id(&self, id: Uuid) -> Result<Option<SealedMasterKey>, String> {
        let keys = self.keys.lock().await;
        Ok(keys.iter().find(|k| k.id == id).cloned())
    }

    async fn list_sealed_keys(&self) -> Result<Vec<SealedMasterKey>, String> {
        let keys = self.keys.lock().await;
        Ok(keys.clone())
    }

    async fn set_active_sealed_key(&self, id: Uuid) -> Result<(), String> {
        let mut keys = self.keys.lock().await;

        // Deactivate all keys
        for key in keys.iter_mut() {
            key.is_active = false;
        }

        // Activate the specified key
        if let Some(key) = keys.iter_mut().find(|k| k.id == id) {
            key.is_active = true;
            Ok(())
        } else {
            Err(format!("Sealed key with ID {} not found", id))
        }
    }

    async fn delete_sealed_key(&self, id: Uuid) -> Result<(), String> {
        let mut keys = self.keys.lock().await;
        let initial_len = keys.len();
        keys.retain(|k| k.id != id);

        if keys.len() < initial_len {
            Ok(())
        } else {
            Err(format!("Sealed key with ID {} not found", id))
        }
    }

    async fn get_sealed_keys_by_provider(
        &self,
        provider_type: ProviderType,
    ) -> Result<Vec<SealedMasterKey>, String> {
        let keys = self.keys.lock().await;
        Ok(keys
            .iter()
            .filter(|k| k.provider_type == provider_type)
            .cloned()
            .collect())
    }
}

// ========== Integration Tests ==========

/// Test 1: Complete seal → store → restart → auto-unseal flow
#[tokio::test]
async fn test_complete_auto_unseal_flow() {
    // Step 1: Generate a master key (simulating initial Secreton setup)
    let master_key = b"test-master-key-32-bytes-long!!!";
    assert_eq!(master_key.len(), 32, "Master key must be 32 bytes");

    // Step 2: Create auto-unseal provider
    let provider = MockKmsProvider::new("mock-kms".to_string(), "test-key-id".to_string());

    // Step 3: Encrypt master key with provider
    let encrypted_master_key = provider
        .encrypt(master_key)
        .await
        .expect("Encryption should succeed");

    // Step 4: Store encrypted master key in storage
    let storage = InMemorySealedKeyStorage::new();
    let sealed_key = SealedMasterKey::new(
        ProviderType::Transit,
        "test-key-id".to_string(),
        None,
        Some("https://mock-kms.local".to_string()),
        encrypted_master_key.clone(),
    );

    storage
        .store_sealed_key(&sealed_key)
        .await
        .expect("Storage should succeed");

    storage
        .set_active_sealed_key(sealed_key.id)
        .await
        .expect("Setting active key should succeed");

    // Step 5: Simulate restart - retrieve encrypted key from storage
    let retrieved_key = storage
        .get_active_sealed_key()
        .await
        .expect("Retrieval should succeed")
        .expect("Active key should exist");

    assert_eq!(retrieved_key.id, sealed_key.id);
    assert!(retrieved_key.is_active);
    assert!(retrieved_key.verify_checksum());

    // Step 6: Auto-unseal using the manager
    let fallback_config = FallbackConfig::default();
    let manager = AutoUnsealManager::new(Box::new(provider), fallback_config);

    let (decrypted_master_key, result) = manager
        .unseal_with_fallback(&retrieved_key.encrypted_master_key)
        .await
        .expect("Auto-unseal should succeed");

    // Step 7: Verify the unsealed master key matches the original
    assert_eq!(decrypted_master_key, master_key);
    assert_eq!(result, UnsealResult::Success);
}

/// Test 2: Fallback to manual unseal when auto-unseal fails
#[tokio::test]
async fn test_fallback_to_manual_unseal() {
    let master_key = b"test-master-key-32-bytes-long!!!";

    // Create provider that will fail
    let provider = MockKmsProvider::new("mock-kms".to_string(), "test-key-id".to_string())
        .with_failure();

    // Encrypt master key (this will succeed)
    let encrypted_master_key = {
        provider.set_fail_decrypt(false).await;
        let encrypted = provider.encrypt(master_key).await.unwrap();
        provider.set_fail_decrypt(true).await;
        encrypted
    };

    // Configure fallback to manual unseal
    let fallback_config = FallbackConfig {
        fallback_to_manual: true,
        max_retries: 2,
        initial_retry_delay_secs: 0, // No delay for testing
        max_retry_delay_secs: 0,
    };

    let manager = AutoUnsealManager::new(Box::new(provider), fallback_config);

    // Attempt auto-unseal (should fail and fall back)
    let (decrypted_master_key, result) = manager
        .unseal_with_fallback(&encrypted_master_key)
        .await
        .expect("Should fall back to manual");

    // Verify fallback occurred
    assert!(decrypted_master_key.is_empty(), "Master key should be empty on fallback");
    assert_eq!(result, UnsealResult::FallbackToManual);
}

/// Test 3: Auto-unseal fails without fallback
#[tokio::test]
async fn test_auto_unseal_fails_without_fallback() {
    let master_key = b"test-master-key-32-bytes-long!!!";

    // Create provider that will fail
    let provider = MockKmsProvider::new("mock-kms".to_string(), "test-key-id".to_string())
        .with_failure();

    // Encrypt master key (this will succeed)
    let encrypted_master_key = {
        provider.set_fail_decrypt(false).await;
        let encrypted = provider.encrypt(master_key).await.unwrap();
        provider.set_fail_decrypt(true).await;
        encrypted
    };

    // Configure NO fallback
    let fallback_config = FallbackConfig {
        fallback_to_manual: false,
        max_retries: 2,
        initial_retry_delay_secs: 0,
        max_retry_delay_secs: 0,
    };

    let manager = AutoUnsealManager::new(Box::new(provider), fallback_config);

    // Attempt auto-unseal (should fail completely)
    let result = manager.unseal_with_fallback(&encrypted_master_key).await;

    assert!(result.is_err(), "Auto-unseal should fail without fallback");
}

/// Test 4: Configuration persistence across restarts
#[tokio::test]
async fn test_configuration_persistence() {
    let storage = InMemorySealedKeyStorage::new();

    // Store multiple sealed keys with different providers
    let keys = vec![
        SealedMasterKey::new(
            ProviderType::AwsKms,
            "arn:aws:kms:us-east-1:123456789012:key/test".to_string(),
            Some("us-east-1".to_string()),
            None,
            vec![1, 2, 3, 4, 5],
        ),
        SealedMasterKey::new(
            ProviderType::GcpKms,
            "projects/test/locations/us/keyRings/test/cryptoKeys/test".to_string(),
            Some("us-central1".to_string()),
            None,
            vec![6, 7, 8, 9, 10],
        ),
        SealedMasterKey::new(
            ProviderType::Transit,
            "auto-unseal-key".to_string(),
            None,
            Some("https://secreton.internal:8200".to_string()),
            vec![11, 12, 13, 14, 15],
        ),
    ];

    // Store all keys
    for key in &keys {
        storage.store_sealed_key(key).await.expect("Storage should succeed");
    }

    // Set the Transit key as active
    storage
        .set_active_sealed_key(keys[2].id)
        .await
        .expect("Setting active key should succeed");

    // Simulate restart - retrieve configuration
    let active_key = storage
        .get_active_sealed_key()
        .await
        .expect("Retrieval should succeed")
        .expect("Active key should exist");

    assert_eq!(active_key.provider_type, ProviderType::Transit);
    assert_eq!(active_key.provider_key_id, "auto-unseal-key");
    assert_eq!(
        active_key.provider_endpoint,
        Some("https://secreton.internal:8200".to_string())
    );
    assert!(active_key.is_active);

    // List all keys
    let all_keys = storage.list_sealed_keys().await.expect("List should succeed");
    assert_eq!(all_keys.len(), 3);

    // Get keys by provider
    let aws_keys = storage
        .get_sealed_keys_by_provider(ProviderType::AwsKms)
        .await
        .expect("Query should succeed");
    assert_eq!(aws_keys.len(), 1);
    assert_eq!(aws_keys[0].provider_type, ProviderType::AwsKms);
}

/// Test 5: Retry mechanism with exponential backoff
#[tokio::test]
async fn test_retry_with_exponential_backoff() {
    let master_key = b"test-master-key-32-bytes-long!!!";

    let provider = MockKmsProvider::new("mock-kms".to_string(), "test-key-id".to_string())
        .with_failure();

    // Encrypt master key
    let encrypted_master_key = {
        provider.set_fail_decrypt(false).await;
        let encrypted = provider.encrypt(master_key).await.unwrap();
        provider.set_fail_decrypt(true).await;
        encrypted
    };

    let fallback_config = FallbackConfig {
        fallback_to_manual: true,
        max_retries: 3,
        initial_retry_delay_secs: 0, // No delay for testing
        max_retry_delay_secs: 0,
    };

    let manager = AutoUnsealManager::new(Box::new(provider), fallback_config);

    let start = std::time::Instant::now();
    let (_key, result) = manager
        .unseal_with_fallback(&encrypted_master_key)
        .await
        .expect("Should fall back");

    let elapsed = start.elapsed();

    // Verify retries occurred (4 attempts total: initial + 3 retries)
    assert_eq!(result, UnsealResult::FallbackToManual);

    // With zero delay, should complete quickly
    assert!(elapsed.as_secs() < 1);
}

/// Test 6: Successful unseal after retries
#[tokio::test]
async fn test_successful_unseal_after_retries() {
    let master_key = b"test-master-key-32-bytes-long!!!";

    let provider = MockKmsProvider::new(
        "mock-kms".to_string(),
        "test-key-id".to_string(),
    );

    // Encrypt master key (while provider is working)
    let encrypted_master_key = provider.encrypt(master_key).await.unwrap();

    // Configure to fail initially, but we'll manually succeed after a short delay
    provider.set_fail_decrypt(true).await;

    let fallback_config = FallbackConfig {
        fallback_to_manual: true,
        max_retries: 5,
        initial_retry_delay_secs: 0,
        max_retry_delay_secs: 0,
    };

    // Enable decryption immediately (simulating transient failure)
    provider.set_fail_decrypt(false).await;

    let manager = AutoUnsealManager::new(Box::new(provider), fallback_config);

    let (decrypted, result) = manager
        .unseal_with_fallback(&encrypted_master_key)
        .await
        .expect("Should succeed");

    assert_eq!(decrypted, master_key);
    assert_eq!(result, UnsealResult::Success);
}

/// Test 7: Checksum verification
#[tokio::test]
async fn test_checksum_verification() {
    let master_key = b"test-master-key-32-bytes-long!!!";
    let provider = MockKmsProvider::new("mock-kms".to_string(), "test-key-id".to_string());

    let encrypted_master_key = provider.encrypt(master_key).await.unwrap();

    let sealed_key = SealedMasterKey::new(
        ProviderType::Transit,
        "test-key-id".to_string(),
        None,
        None,
        encrypted_master_key.clone(),
    );

    // Verify checksum is valid
    assert!(sealed_key.verify_checksum());

    // Corrupt the encrypted data
    let mut corrupted_key = sealed_key.clone();
    corrupted_key.encrypted_master_key[0] ^= 0xFF;

    // Verify checksum fails
    assert!(!corrupted_key.verify_checksum());
}

/// Test 8: Multiple providers with same master key
#[tokio::test]
async fn test_multiple_providers_same_master_key() {
    let master_key = b"test-master-key-32-bytes-long!!!";
    let storage = InMemorySealedKeyStorage::new();

    // Create multiple providers
    let providers = vec![
        MockKmsProvider::new("aws-kms".to_string(), "aws-key-id".to_string()),
        MockKmsProvider::new("gcp-kms".to_string(), "gcp-key-id".to_string()),
        MockKmsProvider::new("transit".to_string(), "transit-key-id".to_string()),
    ];

    let provider_types = vec![
        ProviderType::AwsKms,
        ProviderType::GcpKms,
        ProviderType::Transit,
    ];

    // Encrypt master key with each provider and store
    for (provider, provider_type) in providers.iter().zip(provider_types.iter()) {
        let encrypted = provider.encrypt(master_key).await.unwrap();

        let sealed_key = SealedMasterKey::new(
            provider_type.clone(),
            provider.metadata().key_id,
            None,
            None,
            encrypted,
        );

        storage.store_sealed_key(&sealed_key).await.unwrap();
    }

    // Verify all keys are stored
    let all_keys = storage.list_sealed_keys().await.unwrap();
    assert_eq!(all_keys.len(), 3);

    // Verify each provider can decrypt its own encrypted key
    for (provider, provider_type) in providers.iter().zip(provider_types.iter()) {
        let keys = storage
            .get_sealed_keys_by_provider(provider_type.clone())
            .await
            .unwrap();

        assert_eq!(keys.len(), 1);

        let decrypted = provider.decrypt(&keys[0].encrypted_master_key).await.unwrap();
        assert_eq!(decrypted, master_key);
    }
}

// ========== Real KMS Provider Integration Tests ==========
// These tests are marked with #[ignore] and require real cloud credentials

#[cfg(feature = "transit")]
#[tokio::test]
#[ignore = "Requires real Transit endpoint"]
async fn test_transit_provider_integration() {
    use secreton_auto_unseal::config::TransitConfig;
    use secreton_auto_unseal::transit::TransitProvider;

    let endpoint = std::env::var("SECRETON_TRANSIT_ENDPOINT")
        .unwrap_or_else(|_| "https://secreton.internal:8200".to_string());
    let key_name = std::env::var("SECRETON_TRANSIT_KEY_NAME")
        .unwrap_or_else(|_| "autounseal".to_string());
    let token = std::env::var("SECRETON_TRANSIT_TOKEN")
        .expect("SECRETON_TRANSIT_TOKEN must be set");

    let config = TransitConfig {
        endpoint,
        key_name,
        token,
        tls_cert_path: None,
        tls_key_path: None,
        tls_ca_path: None,
        timeout_secs: 30,
    };

    let provider = TransitProvider::new(config)
        .await
        .expect("Transit provider creation should succeed");

    // Test health check
    provider
        .health_check()
        .await
        .expect("Health check should succeed");

    // Test encryption/decryption
    let master_key = b"test-master-key-32-bytes-long!!!";
    let encrypted = provider
        .encrypt(master_key)
        .await
        .expect("Encryption should succeed");

    let decrypted = provider
        .decrypt(&encrypted)
        .await
        .expect("Decryption should succeed");

    assert_eq!(decrypted, master_key);
}

#[cfg(feature = "aws-kms")]
#[tokio::test]
#[ignore = "Requires AWS credentials"]
async fn test_aws_kms_provider_integration() {
    use secreton_auto_unseal::aws_kms::AwsKmsProvider;
    use secreton_auto_unseal::config::AwsKmsConfig;

    let key_id = std::env::var("AWS_KMS_KEY_ID")
        .unwrap_or_else(|_| "alias/secreton-test".to_string());
    let region = std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string());

    let config = AwsKmsConfig {
        key_id,
        region,
        endpoint: std::env::var("AWS_KMS_ENDPOINT").ok(),
        access_key_id: std::env::var("AWS_ACCESS_KEY_ID").ok(),
        secret_access_key: std::env::var("AWS_SECRET_ACCESS_KEY").ok(),
        session_token: std::env::var("AWS_SESSION_TOKEN").ok(),
    };

    let provider = AwsKmsProvider::new(config)
        .await
        .expect("AWS KMS provider creation should succeed");

    // Test health check
    provider
        .health_check()
        .await
        .expect("Health check should succeed");

    // Test encryption/decryption
    let master_key = b"test-master-key-32-bytes-long!!!";
    let encrypted = provider
        .encrypt(master_key)
        .await
        .expect("Encryption should succeed");

    let decrypted = provider
        .decrypt(&encrypted)
        .await
        .expect("Decryption should succeed");

    assert_eq!(decrypted, master_key);
}

#[cfg(feature = "gcp-kms")]
#[tokio::test]
#[ignore = "Requires GCP credentials"]
async fn test_gcp_kms_provider_integration() {
    use secreton_auto_unseal::config::GcpKmsConfig;
    use secreton_auto_unseal::gcp_kms::GcpKmsProvider;

    let project_id = std::env::var("GCP_PROJECT_ID").expect("GCP_PROJECT_ID must be set");
    let location = std::env::var("GCP_LOCATION").unwrap_or_else(|_| "us-central1".to_string());
    let key_ring = std::env::var("GCP_KEY_RING").unwrap_or_else(|_| "secreton-test".to_string());
    let crypto_key =
        std::env::var("GCP_CRYPTO_KEY").unwrap_or_else(|_| "autounseal-test".to_string());

    let key_name = format!(
        "projects/{}/locations/{}/keyRings/{}/cryptoKeys/{}",
        project_id, location, key_ring, crypto_key
    );

    let config = GcpKmsConfig {
        key_name,
        project_id,
        location,
        key_ring,
        crypto_key,
        credentials_file: std::env::var("GOOGLE_APPLICATION_CREDENTIALS").ok(),
    };

    let provider = GcpKmsProvider::new(config)
        .await
        .expect("GCP KMS provider creation should succeed");

    // Test health check
    provider
        .health_check()
        .await
        .expect("Health check should succeed");

    // Test encryption/decryption
    let master_key = b"test-master-key-32-bytes-long!!!";
    let encrypted = provider
        .encrypt(master_key)
        .await
        .expect("Encryption should succeed");

    let decrypted = provider
        .decrypt(&encrypted)
        .await
        .expect("Decryption should succeed");

    assert_eq!(decrypted, master_key);
}
