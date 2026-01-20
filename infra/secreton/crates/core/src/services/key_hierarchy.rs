//! Key Hierarchy Service
//!
//! Implements explicit Master Key → Key Encryption Key → Data Encryption Key hierarchy
//! following BSSN (Badan Siber dan Sandi Negara) standards for defense-in-depth key protection.
//!
//! Architecture:
//! - Master Key (MK): Generated via Shamir Secret Sharing, managed by SealService
//! - Key Encryption Keys (KEK): Derived from MK using HKDF with unique contexts
//! - Data Encryption Keys (DEK): Encrypted with KEKs before storage
//!
//! This provides:
//! - Key isolation: Compromise of one DEK doesn't expose others
//! - Key rotation: KEKs can be rotated without re-encrypting all data
//! - Audit trail: Complete lineage tracking for compliance

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use zeroize::{Zeroize, ZeroizeOnDrop};

use secreton_crypto::key_derivation::stretch::derive_multiple_keys;
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use rand::RngCore;
use rand::rngs::OsRng;

use crate::services::seal::SealService;

/// Key hierarchy errors
#[derive(Debug, thiserror::Error)]
pub enum KeyError {
    #[error("Vault is sealed")]
    VaultSealed,

    #[error("Master key not available")]
    MasterKeyNotAvailable,

    #[error("KEK not found: {0}")]
    KekNotFound(Uuid),

    #[error("DEK not found: {0}")]
    DekNotFound(Uuid),

    #[error("Invalid key level")]
    InvalidKeyLevel,

    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),

    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Serialization failed: {0}")]
    SerializationFailed(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Invalid context")]
    InvalidContext,
}

/// Key hierarchy levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyLevel {
    MasterKey,
    KeyEncryptionKey,
    DataEncryptionKey,
}

/// Key metadata without exposing key material
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetadata {
    pub id: Uuid,
    pub level: KeyLevel,
    pub parent_id: Option<Uuid>,
    pub context: String,
    pub created_at: DateTime<Utc>,
    pub rotated_at: Option<DateTime<Utc>>,
    pub version: u32,
}

impl KeyMetadata {
    /// Create new key metadata
    pub fn new(level: KeyLevel, parent_id: Option<Uuid>, context: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            level,
            parent_id,
            context,
            created_at: Utc::now(),
            rotated_at: None,
            version: 1,
        }
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, KeyError> {
        serde_json::to_string(self)
            .map_err(|e| KeyError::SerializationFailed(e.to_string()))
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self, KeyError> {
        serde_json::from_str(json)
            .map_err(|e| KeyError::SerializationFailed(e.to_string()))
    }
}

/// Encrypted DEK storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedDek {
    pub id: Uuid,
    pub kek_id: Uuid,
    pub encrypted_key: Vec<u8>,
    pub nonce: Vec<u8>,
    pub version: u32,
    pub created_at: DateTime<Utc>,
}

/// KEK metadata with derived key material (in memory only)
#[derive(Clone)]
struct KekEntry {
    metadata: KeyMetadata,
    key_material: Option<Vec<u8>>, // Only in memory when vault unsealed
}

impl Drop for KekEntry {
    fn drop(&mut self) {
        if let Some(ref mut key) = self.key_material {
            key.zeroize();
        }
    }
}

/// DEK entry with encrypted storage
#[derive(Clone)]
struct DekEntry {
    metadata: KeyMetadata,
    encrypted: EncryptedDek,
}

/// Key Hierarchy Service trait
#[async_trait]
pub trait KeyHierarchyService: Send + Sync {
    /// Derive KEK from master key
    async fn derive_kek(&self, context: &str) -> Result<KeyMetadata, KeyError>;

    /// Create DEK encrypted with KEK
    async fn create_dek(&self, kek_id: Uuid) -> Result<KeyMetadata, KeyError>;

    /// Get key lineage
    async fn get_lineage(&self, key_id: Uuid) -> Result<Vec<KeyMetadata>, KeyError>;

    /// Rotate KEK and re-encrypt all DEKs
    async fn rotate_kek(&self, kek_id: Uuid) -> Result<KeyMetadata, KeyError>;

    /// Get DEK (decrypted)
    async fn get_dek(&self, dek_id: Uuid) -> Result<Vec<u8>, KeyError>;

    /// Get KEK metadata
    async fn get_kek_metadata(&self, kek_id: Uuid) -> Result<KeyMetadata, KeyError>;

    /// Get DEK metadata
    async fn get_dek_metadata(&self, dek_id: Uuid) -> Result<KeyMetadata, KeyError>;
}

/// Key Hierarchy Service implementation
pub struct KeyHierarchyServiceImpl {
    seal_service: Arc<SealService>,
    keks: Arc<RwLock<HashMap<Uuid, KekEntry>>>,
    deks: Arc<RwLock<HashMap<Uuid, DekEntry>>>,
}

impl KeyHierarchyServiceImpl {
    /// Create new key hierarchy service
    pub fn new(seal_service: Arc<SealService>) -> Self {
        Self {
            seal_service,
            keks: Arc::new(RwLock::new(HashMap::new())),
            deks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Derive KEK from master key using HKDF-like derivation
    async fn derive_kek_material(&self, context: &str) -> Result<Vec<u8>, KeyError> {
        // Get master key from seal service
        let master_key = self
            .seal_service
            .get_master_key()
            .await
            .map_err(|_| KeyError::VaultSealed)?;

        // Derive KEK using HKDF-like derivation
        let keys = derive_multiple_keys(&master_key, &[context], 32)
            .map_err(|e| KeyError::KeyDerivationFailed(e.to_string()))?;

        Ok(keys.into_iter().next().unwrap())
    }

    /// Encrypt DEK with KEK
    fn encrypt_dek_with_kek(
        dek: &[u8],
        kek: &[u8],
    ) -> Result<(Vec<u8>, Vec<u8>), KeyError> {
        // Generate random nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from(nonce_bytes);

        // Create AES-256-GCM cipher
        let cipher = Aes256Gcm::new_from_slice(kek)
            .map_err(|e| KeyError::EncryptionFailed(e.to_string()))?;

        // Encrypt DEK
        let ciphertext = cipher
            .encrypt(&nonce, dek)
            .map_err(|e| KeyError::EncryptionFailed(e.to_string()))?;

        Ok((ciphertext, nonce_bytes.to_vec()))
    }

    /// Decrypt DEK with KEK
    fn decrypt_dek_with_kek(
        encrypted_dek: &[u8],
        kek: &[u8],
        nonce_bytes: &[u8],
    ) -> Result<Vec<u8>, KeyError> {
        // Create AES-256-GCM cipher
        let cipher = Aes256Gcm::new_from_slice(kek)
            .map_err(|e| KeyError::DecryptionFailed(e.to_string()))?;

        // Create nonce
        if nonce_bytes.len() != 12 {
            return Err(KeyError::DecryptionFailed("Invalid nonce size".to_string()));
        }
        let mut nonce_arr = [0u8; 12];
        nonce_arr.copy_from_slice(nonce_bytes);
        let nonce = Nonce::from(nonce_arr);

        // Decrypt DEK
        let plaintext = cipher
            .decrypt(&nonce, encrypted_dek)
            .map_err(|e| KeyError::DecryptionFailed(e.to_string()))?;

        Ok(plaintext)
    }
}

#[async_trait]
impl KeyHierarchyService for KeyHierarchyServiceImpl {
    /// Derive KEK from master key
    async fn derive_kek(&self, context: &str) -> Result<KeyMetadata, KeyError> {
        if context.is_empty() {
            return Err(KeyError::InvalidContext);
        }

        // Derive KEK material from master key
        let kek_material = self.derive_kek_material(context).await?;

        // Create KEK metadata
        let metadata = KeyMetadata::new(KeyLevel::KeyEncryptionKey, None, context.to_string());

        // Store KEK entry
        let entry = KekEntry {
            metadata: metadata.clone(),
            key_material: Some(kek_material),
        };

        let mut keks = self.keks.write().await;
        keks.insert(metadata.id, entry);

        Ok(metadata)
    }

    /// Create DEK encrypted with KEK
    async fn create_dek(&self, kek_id: Uuid) -> Result<KeyMetadata, KeyError> {
        // Get KEK
        let keks = self.keks.read().await;
        let kek_entry = keks.get(&kek_id).ok_or(KeyError::KekNotFound(kek_id))?;

        let kek_material = kek_entry
            .key_material
            .as_ref()
            .ok_or(KeyError::VaultSealed)?;

        // Generate random DEK (32 bytes for AES-256)
        let mut dek_bytes = vec![0u8; 32];
        OsRng.fill_bytes(&mut dek_bytes);

        // Encrypt DEK with KEK
        let (encrypted_key, nonce) = Self::encrypt_dek_with_kek(&dek_bytes, kek_material)?;

        // Create DEK metadata
        let metadata = KeyMetadata::new(
            KeyLevel::DataEncryptionKey,
            Some(kek_id),
            format!("dek-{}", Uuid::new_v4()),
        );

        // Create encrypted DEK
        let encrypted_dek = EncryptedDek {
            id: metadata.id,
            kek_id,
            encrypted_key,
            nonce,
            version: 1,
            created_at: Utc::now(),
        };

        // Store DEK entry
        let entry = DekEntry {
            metadata: metadata.clone(),
            encrypted: encrypted_dek,
        };

        drop(keks);
        let mut deks = self.deks.write().await;
        deks.insert(metadata.id, entry);

        // Zeroize DEK bytes
        let mut dek_bytes_mut = dek_bytes;
        dek_bytes_mut.zeroize();

        Ok(metadata)
    }

    /// Get key lineage
    async fn get_lineage(&self, key_id: Uuid) -> Result<Vec<KeyMetadata>, KeyError> {
        let mut lineage = Vec::new();

        // Check if it's a DEK
        let deks = self.deks.read().await;
        if let Some(dek_entry) = deks.get(&key_id) {
            lineage.push(dek_entry.metadata.clone());

            // Get parent KEK
            if let Some(kek_id) = dek_entry.metadata.parent_id {
                drop(deks);
                let keks = self.keks.read().await;
                if let Some(kek_entry) = keks.get(&kek_id) {
                    lineage.push(kek_entry.metadata.clone());
                }
            }

            return Ok(lineage);
        }
        drop(deks);

        // Check if it's a KEK
        let keks = self.keks.read().await;
        if let Some(kek_entry) = keks.get(&key_id) {
            lineage.push(kek_entry.metadata.clone());
            return Ok(lineage);
        }

        Err(KeyError::DekNotFound(key_id))
    }

    /// Rotate KEK and re-encrypt all DEKs
    async fn rotate_kek(&self, kek_id: Uuid) -> Result<KeyMetadata, KeyError> {
        // Get old KEK
        let mut keks = self.keks.write().await;
        let old_kek_entry = keks.get(&kek_id).ok_or(KeyError::KekNotFound(kek_id))?;

        let old_kek_material = old_kek_entry
            .key_material
            .as_ref()
            .ok_or(KeyError::VaultSealed)?
            .clone();

        let context = old_kek_entry.metadata.context.clone();

        // Derive new KEK material
        drop(keks);
        let new_kek_material = self.derive_kek_material(&format!("{}-v2", context)).await?;

        // Create new KEK metadata
        let mut new_metadata = KeyMetadata::new(KeyLevel::KeyEncryptionKey, None, context);
        new_metadata.version = 2;
        new_metadata.rotated_at = Some(Utc::now());

        // Re-encrypt all DEKs associated with this KEK
        let mut deks = self.deks.write().await;
        let mut updated_deks = Vec::new();

        for (dek_id, dek_entry) in deks.iter() {
            if dek_entry.encrypted.kek_id == kek_id {
                // Decrypt DEK with old KEK
                let dek_bytes = Self::decrypt_dek_with_kek(
                    &dek_entry.encrypted.encrypted_key,
                    &old_kek_material,
                    &dek_entry.encrypted.nonce,
                )?;

                // Re-encrypt DEK with new KEK
                let (encrypted_key, nonce) = Self::encrypt_dek_with_kek(&dek_bytes, &new_kek_material)?;

                // Create updated encrypted DEK
                let mut updated_encrypted = dek_entry.encrypted.clone();
                updated_encrypted.encrypted_key = encrypted_key;
                updated_encrypted.nonce = nonce;
                updated_encrypted.kek_id = new_metadata.id;
                updated_encrypted.version += 1;

                // Update metadata parent
                let mut updated_metadata = dek_entry.metadata.clone();
                updated_metadata.parent_id = Some(new_metadata.id);
                updated_metadata.rotated_at = Some(Utc::now());
                updated_metadata.version += 1;

                updated_deks.push((*dek_id, updated_metadata, updated_encrypted));
            }
        }

        // Apply updates atomically
        for (dek_id, metadata, encrypted) in updated_deks {
            let entry = DekEntry { metadata, encrypted };
            deks.insert(dek_id, entry);
        }

        drop(deks);

        // Store new KEK
        let new_entry = KekEntry {
            metadata: new_metadata.clone(),
            key_material: Some(new_kek_material),
        };

        let mut keks = self.keks.write().await;
        keks.insert(new_metadata.id, new_entry);

        // Remove old KEK
        keks.remove(&kek_id);

        Ok(new_metadata)
    }

    /// Get DEK (decrypted)
    async fn get_dek(&self, dek_id: Uuid) -> Result<Vec<u8>, KeyError> {
        // Get DEK entry
        let deks = self.deks.read().await;
        let dek_entry = deks.get(&dek_id).ok_or(KeyError::DekNotFound(dek_id))?;

        let kek_id = dek_entry.encrypted.kek_id;
        let encrypted_key = dek_entry.encrypted.encrypted_key.clone();
        let nonce = dek_entry.encrypted.nonce.clone();

        drop(deks);

        // Get KEK
        let keks = self.keks.read().await;
        let kek_entry = keks.get(&kek_id).ok_or(KeyError::KekNotFound(kek_id))?;

        let kek_material = kek_entry
            .key_material
            .as_ref()
            .ok_or(KeyError::VaultSealed)?;

        // Decrypt DEK
        Self::decrypt_dek_with_kek(&encrypted_key, kek_material, &nonce)
    }

    /// Get KEK metadata
    async fn get_kek_metadata(&self, kek_id: Uuid) -> Result<KeyMetadata, KeyError> {
        let keks = self.keks.read().await;
        let kek_entry = keks.get(&kek_id).ok_or(KeyError::KekNotFound(kek_id))?;
        Ok(kek_entry.metadata.clone())
    }

    /// Get DEK metadata
    async fn get_dek_metadata(&self, dek_id: Uuid) -> Result<KeyMetadata, KeyError> {
        let deks = self.deks.read().await;
        let dek_entry = deks.get(&dek_id).ok_or(KeyError::DekNotFound(dek_id))?;
        Ok(dek_entry.metadata.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::seal::{SealConfig, InMemoryVaultStateStorage};

    async fn setup_test_service() -> (Arc<SealService>, Arc<KeyHierarchyServiceImpl>) {
        let config = SealConfig::default();
        let storage = Arc::new(InMemoryVaultStateStorage::new());
        let seal_service = Arc::new(SealService::with_storage(config, storage));

        // Initialize and unseal vault
        let shares = seal_service.initialize().await.unwrap();
        for share in shares.iter().take(3) {
            let share_bytes = share.to_bytes().unwrap();
            let _ = seal_service.unseal_with_share(&share_bytes).await;
        }

        let key_hierarchy = Arc::new(KeyHierarchyServiceImpl::new(seal_service.clone()));

        (seal_service, key_hierarchy)
    }

    #[tokio::test]
    async fn test_derive_kek() {
        let (_seal, hierarchy) = setup_test_service().await;

        let metadata = hierarchy.derive_kek("test-context").await.unwrap();

        assert_eq!(metadata.level, KeyLevel::KeyEncryptionKey);
        assert_eq!(metadata.context, "test-context");
        assert!(metadata.parent_id.is_none());
    }

    #[tokio::test]
    async fn test_create_dek() {
        let (_seal, hierarchy) = setup_test_service().await;

        // First create a KEK
        let kek_metadata = hierarchy.derive_kek("test-kek").await.unwrap();

        // Then create a DEK
        let dek_metadata = hierarchy.create_dek(kek_metadata.id).await.unwrap();

        assert_eq!(dek_metadata.level, KeyLevel::DataEncryptionKey);
        assert_eq!(dek_metadata.parent_id, Some(kek_metadata.id));
    }

    #[tokio::test]
    async fn test_get_lineage() {
        let (_seal, hierarchy) = setup_test_service().await;

        // Create KEK and DEK
        let kek_metadata = hierarchy.derive_kek("lineage-test").await.unwrap();
        let dek_metadata = hierarchy.create_dek(kek_metadata.id).await.unwrap();

        // Get lineage
        let lineage = hierarchy.get_lineage(dek_metadata.id).await.unwrap();

        assert_eq!(lineage.len(), 2);
        assert_eq!(lineage[0].id, dek_metadata.id);
        assert_eq!(lineage[1].id, kek_metadata.id);
    }

    #[tokio::test]
    async fn test_get_dek() {
        let (_seal, hierarchy) = setup_test_service().await;

        // Create KEK and DEK
        let kek_metadata = hierarchy.derive_kek("dek-test").await.unwrap();
        let dek_metadata = hierarchy.create_dek(kek_metadata.id).await.unwrap();

        // Get DEK
        let dek_bytes = hierarchy.get_dek(dek_metadata.id).await.unwrap();

        assert_eq!(dek_bytes.len(), 32); // AES-256 key
    }

    #[tokio::test]
    async fn test_metadata_serialization() {
        let metadata = KeyMetadata::new(
            KeyLevel::KeyEncryptionKey,
            None,
            "test-context".to_string(),
        );

        let json = metadata.to_json().unwrap();
        let deserialized = KeyMetadata::from_json(&json).unwrap();

        assert_eq!(metadata.id, deserialized.id);
        assert_eq!(metadata.level, deserialized.level);
        assert_eq!(metadata.context, deserialized.context);
    }

    #[tokio::test]
    async fn test_kek_rotation() {
        let (_seal, hierarchy) = setup_test_service().await;

        // Create KEK and multiple DEKs
        let kek_metadata = hierarchy.derive_kek("rotation-test").await.unwrap();
        let dek1_metadata = hierarchy.create_dek(kek_metadata.id).await.unwrap();
        let dek2_metadata = hierarchy.create_dek(kek_metadata.id).await.unwrap();

        // Get DEK values before rotation
        let dek1_before = hierarchy.get_dek(dek1_metadata.id).await.unwrap();
        let dek2_before = hierarchy.get_dek(dek2_metadata.id).await.unwrap();

        // Rotate KEK
        let new_kek_metadata = hierarchy.rotate_kek(kek_metadata.id).await.unwrap();

        // Verify new KEK has different ID
        assert_ne!(new_kek_metadata.id, kek_metadata.id);
        assert_eq!(new_kek_metadata.version, 2);
        assert!(new_kek_metadata.rotated_at.is_some());

        // Verify DEKs are still accessible with same values
        let dek1_after = hierarchy.get_dek(dek1_metadata.id).await.unwrap();
        let dek2_after = hierarchy.get_dek(dek2_metadata.id).await.unwrap();

        assert_eq!(dek1_before, dek1_after);
        assert_eq!(dek2_before, dek2_after);

        // Verify old KEK is no longer accessible
        let old_kek_result = hierarchy.get_kek_metadata(kek_metadata.id).await;
        assert!(old_kek_result.is_err());

        // Verify new KEK is accessible
        let new_kek_result = hierarchy.get_kek_metadata(new_kek_metadata.id).await;
        assert!(new_kek_result.is_ok());
    }
}
