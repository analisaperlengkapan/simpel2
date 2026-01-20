//! Storage Integration Layer
//!
//! Provides seamless integration between cryptographic operations and storage layer.

use crate::{
    error::{CryptoError, CryptoResult},
    transit::{KeyType, TransitEngine},
};
use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Encrypted vault entry for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedVaultEntry {
    /// Entry ID
    pub id: String,

    /// Encrypted data
    pub encrypted_data: Vec<u8>,

    /// Encryption metadata
    pub metadata: EncryptionMetadata,

    /// Key ID used for encryption
    pub key_id: String,

    /// Key version
    pub key_version: u32,
}

/// Encryption metadata for vault entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionMetadata {
    /// Algorithm used
    pub algorithm: String,

    /// Key derivation parameters
    pub kdf_params: Option<serde_json::Value>,

    /// Additional authenticated data
    pub aad: Option<Vec<u8>>,

    /// Timestamp
    pub encrypted_at: chrono::DateTime<chrono::Utc>,
}

/// Crypto-Storage bridge for seamless encryption/decryption
pub struct CryptoStorageBridge {
    /// Transit engine for key management
    transit: Arc<RwLock<TransitEngine>>,

    /// Default key for encryption
    default_key: String,
}

impl CryptoStorageBridge {
    /// Create a new crypto-storage bridge
    pub async fn new(default_key_name: String) -> CryptoResult<Self> {
        let transit = Arc::new(RwLock::new(TransitEngine::new()));

        // Create default key if it doesn't exist
        {
            let t = transit.write().await;
            if t.get_key_info(&default_key_name).await.is_err() {
                t.create_key(default_key_name.clone(), KeyType::Aes256Gcm, None)
                    .await?;
            }
        }

        Ok(Self {
            transit,
            default_key: default_key_name,
        })
    }

    /// Encrypt data for storage
    pub async fn encrypt_for_storage(
        &self,
        data: &[u8],
        key_name: Option<&str>,
    ) -> CryptoResult<EncryptedVaultEntry> {
        let key_name = key_name.unwrap_or(&self.default_key);

        // Encrypt data using transit engine
        let transit = self.transit.read().await;
        let encrypted = transit.encrypt(key_name, data, None, None).await?;

        // Parse encrypted data format: v<version>:<ciphertext>
        let parts: Vec<&str> = encrypted.split(':').collect();
        let version = if parts.len() == 2 {
            parts[0].trim_start_matches('v').parse().unwrap_or(1)
        } else {
            1
        };

        let encrypted_bytes = if parts.len() == 2 {
            general_purpose::STANDARD
                .decode(parts[1])
                .map_err(|e| CryptoError::InvalidCiphertext(e.to_string()))?
        } else {
            general_purpose::STANDARD
                .decode(&encrypted)
                .map_err(|e| CryptoError::InvalidCiphertext(e.to_string()))?
        };

        // Get key info for metadata
        let key_info = transit.get_key_info(key_name).await?;

        Ok(EncryptedVaultEntry {
            id: uuid::Uuid::new_v4().to_string(),
            encrypted_data: encrypted_bytes,
            metadata: EncryptionMetadata {
                algorithm: format!("{:?}", key_info.key_type),
                kdf_params: None,
                aad: None,
                encrypted_at: chrono::Utc::now(),
            },
            key_id: key_name.to_string(),
            key_version: version,
        })
    }

    /// Decrypt data from storage
    pub async fn decrypt_from_storage(&self, entry: &EncryptedVaultEntry) -> CryptoResult<Vec<u8>> {
        let transit = self.transit.read().await;

        // Reconstruct encrypted format
        let encrypted_str = format!(
            "v{}:{}",
            entry.key_version,
            general_purpose::STANDARD.encode(&entry.encrypted_data)
        );

        // Decrypt using transit engine
        transit.decrypt(&entry.key_id, &encrypted_str, None).await
    }

    /// Rotate encryption key
    pub async fn rotate_key(&self, key_name: &str) -> CryptoResult<()> {
        let transit = self.transit.write().await;
        transit.rotate_key(key_name).await?;
        Ok(())
    }

    /// Re-encrypt data with new key version
    pub async fn reencrypt(
        &self,
        entry: &EncryptedVaultEntry,
    ) -> CryptoResult<EncryptedVaultEntry> {
        // Decrypt with old key
        let plaintext = self.decrypt_from_storage(entry).await?;

        // Encrypt with new key version
        self.encrypt_for_storage(&plaintext, Some(&entry.key_id))
            .await
    }

    /// Get key information
    pub async fn get_key_info(
        &self,
        key_name: &str,
    ) -> CryptoResult<crate::transit::keys::KeyInfo> {
        let transit = self.transit.read().await;
        transit.get_key_info(key_name).await
    }
}

/// Storage-specific key information (deprecated - use transit::keys::KeyInfo instead)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageKeyInfo {
    pub name: String,
    pub key_type: String,
    pub latest_version: u32,
    pub min_decryption_version: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub supports_encryption: bool,
    pub supports_decryption: bool,
    pub supports_signing: bool,
    pub supports_derivation: bool,
}

// Re-export transit KeyInfo as the canonical KeyInfo
pub use crate::transit::keys::KeyInfo;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_crypto_storage_bridge_creation() {
        let bridge = CryptoStorageBridge::new("test-key".to_string()).await;
        assert!(bridge.is_ok());
    }

    #[tokio::test]
    #[ignore = "requires secreton/vault backend"]
    async fn test_encrypt_decrypt_roundtrip() {
        let bridge = CryptoStorageBridge::new("test-key".to_string())
            .await
            .unwrap();

        let plaintext = b"Hello, World!";

        // Encrypt
        let encrypted = bridge.encrypt_for_storage(plaintext, None).await.unwrap();

        // Decrypt
        let decrypted = bridge.decrypt_from_storage(&encrypted).await.unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[tokio::test]
    #[ignore = "requires secreton/vault backend"]
    async fn test_key_rotation() {
        let bridge = CryptoStorageBridge::new("rotate-key".to_string())
            .await
            .unwrap();

        // Encrypt with version 1
        let plaintext = b"Test data";
        let encrypted_v1 = bridge.encrypt_for_storage(plaintext, None).await.unwrap();

        assert_eq!(encrypted_v1.key_version, 1);

        // Rotate key
        bridge.rotate_key("rotate-key").await.unwrap();

        // Encrypt with version 2
        let encrypted_v2 = bridge.encrypt_for_storage(plaintext, None).await.unwrap();

        assert_eq!(encrypted_v2.key_version, 2);

        // Both should decrypt successfully
        let decrypted_v1 = bridge.decrypt_from_storage(&encrypted_v1).await.unwrap();
        let decrypted_v2 = bridge.decrypt_from_storage(&encrypted_v2).await.unwrap();

        assert_eq!(plaintext, decrypted_v1.as_slice());
        assert_eq!(plaintext, decrypted_v2.as_slice());
    }
}
