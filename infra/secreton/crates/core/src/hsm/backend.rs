//! HSM backend implementation
//!
//! This module implements the HsmVault trait from Authenc, providing a complete
//! HSM integration for Secreton.

use super::config::{HsmConfig, HsmProvider as HsmProviderType};
use super::error::{HsmError, HsmResult};
use super::pkcs11::Pkcs11Provider;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// HSM backend for Secreton
pub struct HsmBackend {
    config: HsmConfig,
    provider: Arc<RwLock<Option<Box<dyn HsmProviderTrait + Send + Sync>>>>,
    key_metadata: Arc<RwLock<HashMap<String, HsmKeyMetadata>>>,
}

/// HSM key metadata
#[derive(Debug, Clone)]
pub struct HsmKeyMetadata {
    /// Key ID in HSM
    pub key_id: String,
    /// Key algorithm (RSA, ECDSA, AES, etc.)
    pub algorithm: String,
    /// Key size in bits
    pub key_size: u32,
    /// Whether key is exportable
    pub exportable: bool,
    /// Key usage (sign, encrypt, wrap, etc.)
    pub usage: Vec<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

/// HSM provider trait for different HSM types
#[async_trait]
trait HsmProviderTrait {
    async fn initialize(&self) -> HsmResult<()>;
    async fn login(&self) -> HsmResult<()>;
    async fn logout(&self) -> HsmResult<()>;
    async fn generate_key(
        &self,
        key_id: &str,
        algorithm: &str,
        key_size: u32,
        usage: &[String],
    ) -> HsmResult<()>;
    async fn sign(&self, key_id: &str, data: &[u8], algorithm: &str) -> HsmResult<Vec<u8>>;
    async fn encrypt(&self, key_id: &str, plaintext: &[u8]) -> HsmResult<Vec<u8>>;
    async fn decrypt(&self, key_id: &str, ciphertext: &[u8]) -> HsmResult<Vec<u8>>;
    async fn list_keys(&self) -> HsmResult<Vec<String>>;
    async fn delete_key(&self, key_id: &str) -> HsmResult<()>;
    async fn health_check(&self) -> HsmResult<bool>;
}

#[async_trait]
impl HsmProviderTrait for Pkcs11Provider {
    async fn initialize(&self) -> HsmResult<()> {
        self.initialize().await
    }

    async fn login(&self) -> HsmResult<()> {
        self.login().await
    }

    async fn logout(&self) -> HsmResult<()> {
        self.logout().await
    }

    async fn generate_key(
        &self,
        key_id: &str,
        algorithm: &str,
        key_size: u32,
        usage: &[String],
    ) -> HsmResult<()> {
        self.generate_key(key_id, algorithm, key_size, usage).await
    }

    async fn sign(&self, key_id: &str, data: &[u8], algorithm: &str) -> HsmResult<Vec<u8>> {
        self.sign(key_id, data, algorithm).await
    }

    async fn encrypt(&self, key_id: &str, plaintext: &[u8]) -> HsmResult<Vec<u8>> {
        self.encrypt(key_id, plaintext).await
    }

    async fn decrypt(&self, key_id: &str, ciphertext: &[u8]) -> HsmResult<Vec<u8>> {
        self.decrypt(key_id, ciphertext).await
    }

    async fn list_keys(&self) -> HsmResult<Vec<String>> {
        self.list_keys().await
    }

    async fn delete_key(&self, key_id: &str) -> HsmResult<()> {
        self.delete_key(key_id).await
    }

    async fn health_check(&self) -> HsmResult<bool> {
        self.health_check().await
    }
}

impl HsmBackend {
    /// Create a new HSM backend
    pub fn new(config: HsmConfig) -> HsmResult<Self> {
        config.validate().map_err(|e| HsmError::ConfigError(e))?;

        Ok(Self {
            config,
            provider: Arc::new(RwLock::new(None)),
            key_metadata: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Initialize HSM backend
    pub async fn initialize(&self) -> HsmResult<()> {
        if !self.config.enabled {
            info!("HSM integration is disabled");
            return Ok(());
        }

        info!("Initializing HSM backend with provider: {:?}", self.config.provider);

        let provider: Box<dyn HsmProviderTrait + Send + Sync> = match self.config.provider {
            HsmProviderType::Pkcs11 => {
                let pkcs11 = Pkcs11Provider::new(self.config.clone())?;
                pkcs11.initialize().await?;
                pkcs11.login().await?;
                Box::new(pkcs11)
            }
            HsmProviderType::AwsKms => {
                return Err(HsmError::ConfigError(
                    "AWS KMS provider not yet implemented".to_string(),
                ));
            }
            HsmProviderType::AzureKeyVault => {
                return Err(HsmError::ConfigError(
                    "Azure Key Vault provider not yet implemented".to_string(),
                ));
            }
            HsmProviderType::GcpKms => {
                return Err(HsmError::ConfigError(
                    "GCP KMS provider not yet implemented".to_string(),
                ));
            }
        };

        let mut provider_lock = self.provider.write().await;
        *provider_lock = Some(provider);

        info!("HSM backend initialized successfully");
        Ok(())
    }

    /// Generate a key in HSM
    pub async fn generate_hsm_key(
        &self,
        key_id: &str,
        algorithm: &str,
        key_size: u32,
        usage: Vec<String>,
    ) -> HsmResult<HsmKeyMetadata> {
        let provider_lock = self.provider.read().await;
        let provider = provider_lock
            .as_ref()
            .ok_or_else(|| HsmError::NotInitialized)?;

        // Check if key already exists
        let metadata_lock = self.key_metadata.read().await;
        if metadata_lock.contains_key(key_id) {
            return Err(HsmError::KeyAlreadyExists(key_id.to_string()));
        }
        drop(metadata_lock);

        // Generate key in HSM
        provider.generate_key(key_id, algorithm, key_size, &usage).await?;

        // Create metadata
        let metadata = HsmKeyMetadata {
            key_id: key_id.to_string(),
            algorithm: algorithm.to_string(),
            key_size,
            exportable: false, // HSM keys are typically not exportable
            usage: usage.clone(),
            created_at: Utc::now(),
        };

        // Store metadata
        let mut metadata_lock = self.key_metadata.write().await;
        metadata_lock.insert(key_id.to_string(), metadata.clone());

        info!("Generated HSM key: {}", key_id);
        Ok(metadata)
    }

    /// Sign data using HSM key
    pub async fn hsm_sign(
        &self,
        key_id: &str,
        data: &[u8],
        algorithm: &str,
    ) -> HsmResult<Vec<u8>> {
        let provider_lock = self.provider.read().await;
        let provider = provider_lock
            .as_ref()
            .ok_or_else(|| HsmError::NotInitialized)?;

        // Verify key exists
        let metadata_lock = self.key_metadata.read().await;
        if !metadata_lock.contains_key(key_id) {
            return Err(HsmError::KeyNotFound(key_id.to_string()));
        }
        drop(metadata_lock);

        // Sign data
        let signature = provider.sign(key_id, data, algorithm).await?;

        debug!("Signed data with HSM key: {}", key_id);
        Ok(signature)
    }

    /// Encrypt data using HSM key
    pub async fn hsm_encrypt(&self, key_id: &str, plaintext: &[u8]) -> HsmResult<Vec<u8>> {
        let provider_lock = self.provider.read().await;
        let provider = provider_lock
            .as_ref()
            .ok_or_else(|| HsmError::NotInitialized)?;

        // Verify key exists
        let metadata_lock = self.key_metadata.read().await;
        if !metadata_lock.contains_key(key_id) {
            return Err(HsmError::KeyNotFound(key_id.to_string()));
        }
        drop(metadata_lock);

        // Encrypt data
        let ciphertext = provider.encrypt(key_id, plaintext).await?;

        debug!("Encrypted data with HSM key: {}", key_id);
        Ok(ciphertext)
    }

    /// Decrypt data using HSM key
    pub async fn hsm_decrypt(&self, key_id: &str, ciphertext: &[u8]) -> HsmResult<Vec<u8>> {
        let provider_lock = self.provider.read().await;
        let provider = provider_lock
            .as_ref()
            .ok_or_else(|| HsmError::NotInitialized)?;

        // Verify key exists
        let metadata_lock = self.key_metadata.read().await;
        if !metadata_lock.contains_key(key_id) {
            return Err(HsmError::KeyNotFound(key_id.to_string()));
        }
        drop(metadata_lock);

        // Decrypt data
        let plaintext = provider.decrypt(key_id, ciphertext).await?;

        debug!("Decrypted data with HSM key: {}", key_id);
        Ok(plaintext)
    }

    /// List all HSM keys
    pub async fn list_hsm_keys(&self) -> HsmResult<Vec<HsmKeyMetadata>> {
        let metadata_lock = self.key_metadata.read().await;
        let keys: Vec<HsmKeyMetadata> = metadata_lock.values().cloned().collect();

        debug!("Listed {} HSM keys", keys.len());
        Ok(keys)
    }

    /// Delete HSM key
    pub async fn delete_hsm_key(&self, key_id: &str) -> HsmResult<()> {
        let provider_lock = self.provider.read().await;
        let provider = provider_lock
            .as_ref()
            .ok_or_else(|| HsmError::NotInitialized)?;

        // Verify key exists
        let mut metadata_lock = self.key_metadata.write().await;
        if !metadata_lock.contains_key(key_id) {
            return Err(HsmError::KeyNotFound(key_id.to_string()));
        }

        // Delete key from HSM
        provider.delete_key(key_id).await?;

        // Remove metadata
        metadata_lock.remove(key_id);

        info!("Deleted HSM key: {}", key_id);
        Ok(())
    }

    /// Check HSM health
    pub async fn health_check(&self) -> HsmResult<bool> {
        if !self.config.enabled {
            return Ok(true); // HSM disabled, consider healthy
        }

        let provider_lock = self.provider.read().await;
        let provider = provider_lock
            .as_ref()
            .ok_or_else(|| HsmError::NotInitialized)?;

        provider.health_check().await
    }

    /// Shutdown HSM backend
    pub async fn shutdown(&self) -> HsmResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        info!("Shutting down HSM backend");

        let provider_lock = self.provider.read().await;
        if let Some(provider) = provider_lock.as_ref() {
            provider.logout().await?;
        }

        info!("HSM backend shut down successfully");
        Ok(())
    }
}

impl Drop for HsmBackend {
    fn drop(&mut self) {
        debug!("Dropping HSM backend");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_config() -> HsmConfig {
        HsmConfig {
            enabled: true,
            provider: HsmProvider::Pkcs11,
            pkcs11_library_path: Some(PathBuf::from("/usr/lib/libpkcs11.so")),
            slot_id: Some(0),
            token_label: Some("test-token".to_string()),
            pin: Some("1234".to_string()),
            key_label_prefix: "test-".to_string(),
            connection_timeout: 30,
            operation_timeout: 60,
            health_check_enabled: true,
            health_check_interval: 60,
            max_retries: 3,
            retry_delay_ms: 1000,
        }
    }

    #[tokio::test]
    async fn test_backend_creation() {
        let config = create_test_config();
        let backend = HsmBackend::new(config);
        assert!(backend.is_ok());
    }

    #[tokio::test]
    async fn test_backend_initialization() {
        let config = create_test_config();
        let backend = HsmBackend::new(config).unwrap();
        let result = backend.initialize().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_generate_key() {
        let config = create_test_config();
        let backend = HsmBackend::new(config).unwrap();
        backend.initialize().await.unwrap();

        let result = backend
            .generate_hsm_key("test-key", "RSA", 2048, vec!["sign".to_string()])
            .await;
        assert!(result.is_ok());

        let metadata = result.unwrap();
        assert_eq!(metadata.key_id, "test-key");
        assert_eq!(metadata.algorithm, "RSA");
        assert_eq!(metadata.key_size, 2048);
    }

    #[tokio::test]
    async fn test_duplicate_key() {
        let config = create_test_config();
        let backend = HsmBackend::new(config).unwrap();
        backend.initialize().await.unwrap();

        // Generate first key
        backend
            .generate_hsm_key("test-key", "RSA", 2048, vec!["sign".to_string()])
            .await
            .unwrap();

        // Try to generate duplicate
        let result = backend
            .generate_hsm_key("test-key", "RSA", 2048, vec!["sign".to_string()])
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_keys() {
        let config = create_test_config();
        let backend = HsmBackend::new(config).unwrap();
        backend.initialize().await.unwrap();

        // Generate some keys
        backend
            .generate_hsm_key("key1", "RSA", 2048, vec!["sign".to_string()])
            .await
            .unwrap();
        backend
            .generate_hsm_key("key2", "AES", 256, vec!["encrypt".to_string()])
            .await
            .unwrap();

        let keys = backend.list_hsm_keys().await.unwrap();
        assert_eq!(keys.len(), 2);
    }

    #[tokio::test]
    async fn test_delete_key() {
        let config = create_test_config();
        let backend = HsmBackend::new(config).unwrap();
        backend.initialize().await.unwrap();

        // Generate key
        backend
            .generate_hsm_key("test-key", "RSA", 2048, vec!["sign".to_string()])
            .await
            .unwrap();

        // Delete key
        let result = backend.delete_hsm_key("test-key").await;
        assert!(result.is_ok());

        // Verify key is gone
        let keys = backend.list_hsm_keys().await.unwrap();
        assert_eq!(keys.len(), 0);
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = create_test_config();
        let backend = HsmBackend::new(config).unwrap();
        backend.initialize().await.unwrap();

        let result = backend.health_check().await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}

