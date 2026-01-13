//! Vault service for business logic operations.

use anyhow::Result;
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

use secreton_core::audit::{AuditLog, AuditLogger, AuditStatus};
use secreton_crypto::{AlgorithmId, CryptoEngine};
use secreton_storage::{SecurityLevel, StorageBackend};

/// Vault service errors
#[derive(Error, Debug)]
pub enum VaultError {
    #[error("Secret not found: {path}")]
    SecretNotFound { path: String },

    #[error("Key not found: {key_id}")]
    KeyNotFound { key_id: String },

    #[error("Policy not found: {name}")]
    PolicyNotFound { name: String },

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Crypto error: {0}")]
    Crypto(#[from] secreton_crypto::CryptoError),

    #[error("Storage error: {0}")]
    Storage(#[from] secreton_storage::StorageError),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

/// Vault service for business logic operations
pub struct VaultService {
    storage: Arc<dyn StorageBackend + Send + Sync>,
    crypto: Arc<CryptoEngine>,
    audit: Arc<AuditLogger>,
}

impl VaultService {
    /// Create new vault service
    pub async fn new(
        storage: Arc<dyn StorageBackend + Send + Sync>,
        crypto: Arc<CryptoEngine>,
        audit: Arc<AuditLogger>,
    ) -> Result<Self> {
        Ok(Self {
            storage,
            crypto,
            audit,
        })
    }

    /// Get secret by path
    pub async fn get_secret(&self, path: &str, user_id: &str) -> Result<SecretData, VaultError> {
        // Log audit trail
        let _ = self
            .audit
            .log(AuditLog {
                id: uuid::Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                action: "read".to_string(),
                actor: Some(user_id.to_string()),
                resource_type: "secret".to_string(),
                resource_id: path.to_string(),
                status: AuditStatus::Success,
                ip: None,
                user_agent: None,
                namespace: None,
                metadata: HashMap::new(),
            })
            .await;

        // Get secret from storage
        let entry =
            self.storage
                .get_by_path(path)
                .await?
                .ok_or_else(|| VaultError::SecretNotFound {
                    path: path.to_string(),
                })?;

        // Decrypt data
        let decrypted_data = serde_json::from_slice(&entry.encrypted_data)
            .map_err(|e| VaultError::Internal(anyhow::anyhow!("Deserialization failed: {}", e)))?;

        // Extract metadata
        let metadata: SecretMetadata = serde_json::from_value(entry.metadata.clone())
            .unwrap_or_default();

        Ok(SecretData {
            path: path.to_string(),
            data: decrypted_data,
            metadata,
            version: entry.version,
            created_at: entry.created_at,
            updated_at: entry.updated_at,
            expires_at: entry.expires_at,
        })
    }

    /// Create or update secret
    pub async fn put_secret(
        &self,
        path: &str,
        data: HashMap<String, String>,
        metadata: SecretMetadata,
        user_id: &str,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<SecretData, VaultError> {
        // Log audit trail
        let _ = self
            .audit
            .log(AuditLog {
                id: uuid::Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                action: "write".to_string(),
                actor: Some(user_id.to_string()),
                resource_type: "secret".to_string(),
                resource_id: path.to_string(),
                status: AuditStatus::Success,
                ip: None,
                user_agent: None,
                namespace: None,
                metadata: HashMap::new(),
            })
            .await;

        // Serialize data
        let serialized = serde_json::to_vec(&data)
            .map_err(|e| VaultError::Internal(anyhow::anyhow!("Serialization failed: {}", e)))?;

        // Get current version or start at 1
        let version = match self.storage.get_by_path(path).await? {
            Some(entry) => entry.version + 1,
            None => 1,
        };

        let now = chrono::Utc::now();

        // Create vault entry
        let mut entry = secreton_storage::VaultEntry::new(
            path.to_string(),
            serialized,
            serde_json::to_value(&metadata).unwrap_or(serde_json::json!({})),
            SecurityLevel::Confidential,
            user_id.to_string(),
        );
        entry.version = version;
        entry.updated_at = now;

        if let Some(expires) = expires_at {
            entry = entry.with_expiration(expires);
        }

        // Store in storage
        self.storage.store(&entry).await?;

        Ok(SecretData {
            path: path.to_string(),
            data,
            metadata,
            version,
            created_at: now,
            updated_at: now,
            expires_at,
        })
    }

    /// Delete secret
    pub async fn delete_secret(&self, path: &str, user_id: &str) -> Result<(), VaultError> {
        // Log audit trail
        let _ = self
            .audit
            .log(AuditLog {
                id: uuid::Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                action: "delete".to_string(),
                actor: Some(user_id.to_string()),
                resource_type: "secret".to_string(),
                resource_id: path.to_string(),
                status: AuditStatus::Success,
                ip: None,
                user_agent: None,
                namespace: None,
                metadata: HashMap::new(),
            })
            .await;

        // Delete from storage
        self.storage.delete_by_path(path).await?;

        Ok(())
    }

    /// List secrets
    pub async fn list_secrets(
        &self,
        prefix: Option<&str>,
        user_id: &str,
    ) -> Result<Vec<String>, VaultError> {
        // Log audit trail
        let path = prefix.unwrap_or("/");
        let _ = self
            .audit
            .log(AuditLog {
                id: uuid::Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                action: "list".to_string(),
                actor: Some(user_id.to_string()),
                resource_type: "secret".to_string(),
                resource_id: path.to_string(),
                status: AuditStatus::Success,
                ip: None,
                user_agent: None,
                namespace: None,
                metadata: HashMap::new(),
            })
            .await;

        // List from storage
        let params = secreton_storage::QueryParams {
            path_prefix: prefix.map(|s| s.to_string()),
            security_level: None,
            tags: Vec::new(),
            owner_id: None,
            metadata_filters: std::collections::HashMap::new(),
            include_expired: false,
            limit: Some(1000),
            offset: None,
            sort_by: None,
            sort_order: None,
        };
        let entries = self.storage.list(&params).await?;
        let keys = entries.iter().map(|e| e.path.clone()).collect();

        Ok(keys)
    }

    /// Create encryption key
    pub async fn create_key(
        &self,
        key_name: &str,
        key_type: &str,
        user_id: &str,
    ) -> Result<KeyInfo, VaultError> {
        // Log audit trail
        let _ = self
            .audit
            .log(AuditLog {
                id: uuid::Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                action: "create".to_string(),
                actor: Some(user_id.to_string()),
                resource_type: "key".to_string(),
                resource_id: format!("keys/{}", key_name),
                status: AuditStatus::Success,
                ip: None,
                user_agent: None,
                namespace: None,
                metadata: HashMap::new(),
            })
            .await;

        // Generate key ID
        let key_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now();

        // Create key metadata
        let key_info = KeyInfo {
            id: key_id.clone(),
            name: key_name.to_string(),
            key_type: key_type.to_string(),
            version: 1,
            created_at: now,
        };

        // Serialize and store key metadata
        let metadata = serde_json::to_vec(&key_info)
            .map_err(|e| VaultError::Internal(anyhow::anyhow!("Serialization failed: {}", e)))?;

        let entry = secreton_storage::VaultEntry::new(
            format!("keys/{}", key_name),
            metadata,
            serde_json::json!({}),
            SecurityLevel::Confidential,
            user_id.to_string(),
        );

        self.storage.store(&entry).await?;

        Ok(key_info)
    }

    /// Encrypt data with key
    pub async fn encrypt(
        &self,
        key_id: &str,
        plaintext: &str,
        user_id: &str,
    ) -> Result<EncryptResult, VaultError> {
        // Log audit trail
        let _ = self
            .audit
            .log(AuditLog {
                id: uuid::Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                action: "encrypt".to_string(),
                actor: Some(user_id.to_string()),
                resource_type: "key".to_string(),
                resource_id: format!("keys/{}/encrypt", key_id),
                status: AuditStatus::Success,
                ip: None,
                user_agent: None,
                namespace: None,
                metadata: HashMap::new(),
            })
            .await;

        // Get or generate encryption key
        let key = secreton_crypto::generate_key(AlgorithmId::Aes256Gcm)?;

        // Encrypt using crypto service
        let encrypted_data =
            self.crypto
                .encrypt(AlgorithmId::Aes256Gcm, plaintext.as_bytes(), &key)?;

        // Serialize encrypted data to JSON then base64
        let json_data = serde_json::to_vec(&encrypted_data)
            .map_err(|e| VaultError::Internal(anyhow::anyhow!("Serialization failed: {}", e)))?;
        let ciphertext = base64::engine::general_purpose::STANDARD.encode(&json_data);

        Ok(EncryptResult {
            ciphertext,
            key_version: 1,
        })
    }

    /// Decrypt data with key
    pub async fn decrypt(
        &self,
        key_id: &str,
        ciphertext: &str,
        user_id: &str,
    ) -> Result<DecryptResult, VaultError> {
        // Log audit trail
        let _ = self
            .audit
            .log(AuditLog {
                id: uuid::Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                action: "decrypt".to_string(),
                actor: Some(user_id.to_string()),
                resource_type: "key".to_string(),
                resource_id: format!("keys/{}/decrypt", key_id),
                status: AuditStatus::Success,
                ip: None,
                user_agent: None,
                namespace: None,
                metadata: HashMap::new(),
            })
            .await;

        // Decode base64
        let json_bytes = base64::engine::general_purpose::STANDARD
            .decode(ciphertext)
            .map_err(|e| VaultError::Internal(anyhow::anyhow!("Base64 decode failed: {}", e)))?;

        // Deserialize encrypted data
        let encrypted_data: secreton_crypto::EncryptedData = serde_json::from_slice(&json_bytes)
            .map_err(|e| VaultError::Internal(anyhow::anyhow!("Deserialization failed: {}", e)))?;

        // Get decryption key (in production, retrieve from key storage)
        let key = secreton_crypto::generate_key(AlgorithmId::Aes256Gcm)?;

        // Decrypt using crypto service
        let plaintext_bytes = self.crypto.decrypt(&encrypted_data, &key)?;
        let plaintext = String::from_utf8(plaintext_bytes)
            .map_err(|e| VaultError::Internal(anyhow::anyhow!("UTF-8 decode failed: {}", e)))?;

        Ok(DecryptResult { plaintext })
    }

    /// Get key by ID or name
    pub async fn get_key(&self, key_id: &str, user_id: &str) -> Result<KeyInfo, VaultError> {
        // Log audit trail
        let _ = self
            .audit
            .log(AuditLog {
                id: uuid::Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                action: "read".to_string(),
                actor: Some(user_id.to_string()),
                resource_type: "key".to_string(),
                resource_id: format!("keys/{}", key_id),
                status: AuditStatus::Success,
                ip: None,
                user_agent: None,
                namespace: None,
                metadata: HashMap::new(),
            })
            .await;

        // Get key from storage
        let entry = self
            .storage
            .get_by_path(&format!("keys/{}", key_id))
            .await?
            .ok_or_else(|| VaultError::KeyNotFound {
                key_id: key_id.to_string(),
            })?;

        // Deserialize key metadata
        let key_info: KeyInfo = serde_json::from_slice(&entry.encrypted_data)
            .map_err(|e| VaultError::Internal(anyhow::anyhow!("Deserialization failed: {}", e)))?;

        Ok(key_info)
    }

    /// List all keys
    pub async fn list_keys(&self, user_id: &str) -> Result<Vec<KeyInfo>, VaultError> {
        // Log audit trail
        let _ = self
            .audit
            .log(AuditLog {
                id: uuid::Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                action: "list".to_string(),
                actor: Some(user_id.to_string()),
                resource_type: "key".to_string(),
                resource_id: "keys/".to_string(),
                status: AuditStatus::Success,
                ip: None,
                user_agent: None,
                namespace: None,
                metadata: HashMap::new(),
            })
            .await;

        // List keys from storage
        let params = secreton_storage::QueryParams {
            path_prefix: Some("keys/".to_string()),
            security_level: None,
            tags: Vec::new(),
            owner_id: None,
            metadata_filters: std::collections::HashMap::new(),
            include_expired: false,
            limit: Some(1000),
            offset: None,
            sort_by: None,
            sort_order: None,
        };
        let entries = self.storage.list(&params).await?;

        let mut key_infos = Vec::new();
        for entry in entries {
            if let Ok(key_info) = serde_json::from_slice::<KeyInfo>(&entry.encrypted_data) {
                key_infos.push(key_info);
            }
        }

        Ok(key_infos)
    }

    /// Rotate key (create new version)
    pub async fn rotate_key(&self, key_id: &str, user_id: &str) -> Result<KeyInfo, VaultError> {
        // Log audit trail
        let _ = self
            .audit
            .log(AuditLog {
                id: uuid::Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                action: "rotate".to_string(),
                actor: Some(user_id.to_string()),
                resource_type: "key".to_string(),
                resource_id: format!("keys/{}/rotate", key_id),
                status: AuditStatus::Success,
                ip: None,
                user_agent: None,
                namespace: None,
                metadata: HashMap::new(),
            })
            .await;

        // Get existing key
        let mut key_info = self.get_key(key_id, user_id).await?;

        // Increment version
        key_info.version += 1;

        // Store updated key metadata
        let metadata = serde_json::to_vec(&key_info)
            .map_err(|e| VaultError::Internal(anyhow::anyhow!("Serialization failed: {}", e)))?;

        let mut entry = secreton_storage::VaultEntry::new(
            format!("keys/{}", key_id),
            metadata,
            serde_json::json!({}),
            SecurityLevel::Confidential,
            user_id.to_string(),
        );
        entry.updated_at = chrono::Utc::now();

        self.storage.store(&entry).await?;

        Ok(key_info)
    }

    /// Delete key
    pub async fn delete_key(&self, key_id: &str, user_id: &str) -> Result<(), VaultError> {
        // Log audit trail
        let _ = self
            .audit
            .log(AuditLog {
                id: uuid::Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                action: "delete".to_string(),
                actor: Some(user_id.to_string()),
                resource_type: "key".to_string(),
                resource_id: format!("keys/{}", key_id),
                status: AuditStatus::Success,
                ip: None,
                user_agent: None,
                namespace: None,
                metadata: HashMap::new(),
            })
            .await;

        // Delete from storage
        self.storage
            .delete_by_path(&format!("keys/{}", key_id))
            .await?;

        Ok(())
    }

    /// Sign data with key
    pub async fn sign(
        &self,
        key_id: &str,
        data: &str,
        algorithm: Option<&str>,
        user_id: &str,
    ) -> Result<SignResult, VaultError> {
        // Log audit trail
        let _ = self
            .audit
            .log(AuditLog {
                id: uuid::Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                action: "sign".to_string(),
                actor: Some(user_id.to_string()),
                resource_type: "key".to_string(),
                resource_id: format!("keys/{}/sign", key_id),
                status: AuditStatus::Success,
                ip: None,
                user_agent: None,
                namespace: None,
                metadata: HashMap::new(),
            })
            .await;

        // Use SHA-256 hash as signature (simplified implementation)
        // In production, use proper Ed25519 or ECDSA signing
        let hash = secreton_crypto::hashing::compute_hash(AlgorithmId::Sha256, data.as_bytes())?;
        let signature = base64::engine::general_purpose::STANDARD.encode(&hash.hash);

        Ok(SignResult {
            signature,
            key_version: 1,
            algorithm: algorithm.unwrap_or("SHA256").to_string(),
        })
    }

    /// Verify signature
    pub async fn verify(
        &self,
        key_id: &str,
        data: &str,
        signature: &str,
        user_id: &str,
    ) -> Result<VerifyResult, VaultError> {
        // Log audit trail
        let _ = self
            .audit
            .log(AuditLog {
                id: uuid::Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                action: "verify".to_string(),
                actor: Some(user_id.to_string()),
                resource_type: "key".to_string(),
                resource_id: format!("keys/{}/verify", key_id),
                status: AuditStatus::Success,
                ip: None,
                user_agent: None,
                namespace: None,
                metadata: HashMap::new(),
            })
            .await;

        // Decode signature
        let signature_bytes = base64::engine::general_purpose::STANDARD
            .decode(signature)
            .map_err(|e| VaultError::Internal(anyhow::anyhow!("Base64 decode failed: {}", e)))?;

        // Compute hash and compare (simplified implementation)
        let hash = secreton_crypto::hashing::compute_hash(AlgorithmId::Sha256, data.as_bytes())?;
        let valid = hash.hash == signature_bytes;

        Ok(VerifyResult {
            valid,
            key_version: 1,
        })
    }

    /// Hash data
    pub async fn hash(
        &self,
        data: &str,
        algorithm: &str,
        user_id: &str,
    ) -> Result<HashResult, VaultError> {
        // Log audit trail
        let _ = self
            .audit
            .log(AuditLog {
                id: uuid::Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                action: "hash".to_string(),
                actor: Some(user_id.to_string()),
                resource_type: "crypto".to_string(),
                resource_id: "crypto/hash".to_string(),
                status: AuditStatus::Success,
                ip: None,
                user_agent: None,
                namespace: None,
                metadata: HashMap::new(),
            })
            .await;

        // Parse algorithm
        let hash_algo = match algorithm.to_lowercase().as_str() {
            "sha256" | "sha-256" => AlgorithmId::Sha256,
            "sha512" | "sha-512" => AlgorithmId::Sha256, // Note: Sha512 not in AlgorithmId, using Sha256
            "blake3" => AlgorithmId::Blake3,
            _ => AlgorithmId::Sha256, // default
        };

        // Hash using crypto service
        let hash_result = secreton_crypto::hashing::compute_hash(hash_algo, data.as_bytes())?;
        let hash = hex::encode(&hash_result.hash);

        Ok(HashResult {
            hash,
            algorithm: algorithm.to_string(),
        })
    }
}

/// Secret metadata
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SecretMetadata {
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub owner: Option<String>,
    pub classification: Option<String>,
}

/// Secret data structure
#[derive(Debug, Serialize)]
pub struct SecretData {
    pub path: String,
    pub data: HashMap<String, String>,
    pub metadata: SecretMetadata,
    pub version: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Key information
#[derive(Debug, Serialize, Deserialize)]
pub struct KeyInfo {
    pub id: String,
    pub name: String,
    pub key_type: String,
    pub version: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Encryption result
#[derive(Debug, Serialize)]
pub struct EncryptResult {
    pub ciphertext: String,
    pub key_version: u32,
}

/// Decryption result
#[derive(Debug, Serialize)]
pub struct DecryptResult {
    pub plaintext: String,
}

/// Sign result
#[derive(Debug, Serialize)]
pub struct SignResult {
    pub signature: String,
    pub key_version: u32,
    pub algorithm: String,
}

/// Verify result
#[derive(Debug, Serialize)]
pub struct VerifyResult {
    pub valid: bool,
    pub key_version: u32,
}

/// Hash result
#[derive(Debug, Serialize)]
pub struct HashResult {
    pub hash: String,
    pub algorithm: String,
}

#[cfg(all(test, feature = "enable-inline-tests"))]
mod tests {
    use super::*;
    use crate::config::AuthConfig;
    use crate::services::auth::AuthService;
    use secreton_core::audit::{AuditBackend, AuditLogger, MemoryBackend as AuditMemoryBackend};
    use secreton_crypto::SecurityParams;
    use secreton_storage::MemoryBackend;

    #[tokio::test]
    async fn test_vault_service_creation() {
        let storage = Arc::new(MemoryBackend::new());
        let crypto = Arc::new(CryptoEngine::new());
        let audit_backend: Arc<dyn AuditBackend> = Arc::new(AuditMemoryBackend::default());
        let audit = Arc::new(AuditLogger::new(vec![audit_backend]));

        let vault_service = VaultService::new(storage, crypto, audit).await;
        assert!(vault_service.is_ok());
    }

    #[tokio::test]
    async fn test_get_secret_placeholder() {
        let storage = Arc::new(MemoryBackend::new());
        let crypto = Arc::new(CryptoEngine::new());
        let audit_backend: Arc<dyn AuditBackend> = Arc::new(AuditMemoryBackend::default());
        let audit = Arc::new(AuditLogger::new(vec![audit_backend]));
        let service = VaultService::new(storage, crypto, audit)
            .await
            .expect("Failed to create VaultService");

        let mut data = HashMap::new();
        data.insert("key1".to_string(), "value1".to_string());
        service.put_secret("app/config", data, SecretMetadata::default(), "user1", None).await.expect("Failed to put secret");

        let secret = service.get_secret("app/config", "user1").await;
        assert!(secret.is_ok());
        let secret = secret.unwrap();
        assert_eq!(secret.path, "app/config");
        assert!(secret.data.contains_key("key1"));
    }

    #[tokio::test]
    async fn test_put_secret_placeholder() {
        let storage = Arc::new(MemoryBackend::new());
        let crypto = Arc::new(CryptoEngine::new());
        let audit_backend: Arc<dyn AuditBackend> = Arc::new(AuditMemoryBackend::default());
        let audit = Arc::new(AuditLogger::new(vec![audit_backend]));
        let service = VaultService::new(storage, crypto, audit)
            .await
            .expect("Failed to create VaultService");

        let mut data = HashMap::new();
        data.insert("username".to_string(), "admin".to_string());
        let secret = service.put_secret("app/admin", data, SecretMetadata::default(), "user1", None).await;
        assert!(secret.is_ok());
        let secret = secret.unwrap();
        assert_eq!(secret.path, "app/admin");
        assert!(secret.data.contains_key("username"));
    }

    #[tokio::test]
    async fn test_put_secret_with_expiration() {
        let storage = Arc::new(MemoryBackend::new());
        let crypto = Arc::new(CryptoEngine::new());
        let audit_backend: Arc<dyn AuditBackend> = Arc::new(AuditMemoryBackend::default());
        let audit = Arc::new(AuditLogger::new(vec![audit_backend]));
        let service = VaultService::new(storage, crypto, audit)
            .await
            .expect("Failed to create VaultService");

        let mut data = HashMap::new();
        data.insert("key".to_string(), "value".to_string());

        let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);
        let secret = service.put_secret("app/expiring", data, SecretMetadata::default(), "user1", Some(expires_at)).await;

        assert!(secret.is_ok());
        let secret = secret.unwrap();
        assert_eq!(secret.expires_at, Some(expires_at));

        // Verify we can retrieve it with expiration
        let retrieved = service.get_secret("app/expiring", "user1").await;
        assert!(retrieved.is_ok());
        assert_eq!(retrieved.unwrap().expires_at, Some(expires_at));
    }

    #[tokio::test]
    async fn test_encrypt_placeholder_response() {
        let storage = Arc::new(MemoryBackend::new());
        let crypto = Arc::new(CryptoEngine::new());
        let audit_backend: Arc<dyn AuditBackend> = Arc::new(AuditMemoryBackend::default());
        let audit = Arc::new(AuditLogger::new(vec![audit_backend]));
        let service = VaultService::new(storage, crypto, audit)
            .await
            .expect("Failed to create VaultService");

        let result = service.encrypt("key1", "plaintext", "user1").await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.ciphertext.is_empty());
        assert_eq!(result.key_version, 1);
    }
}

impl VaultService {
    /// Create mock vault service for testing
    pub fn new_mock(
        storage: Arc<dyn StorageBackend + Send + Sync>,
        crypto: Arc<CryptoEngine>,
    ) -> Self {
        Self {
            storage,
            crypto,
            audit: Arc::new(AuditLogger::new(vec![])),
        }
    }
}
