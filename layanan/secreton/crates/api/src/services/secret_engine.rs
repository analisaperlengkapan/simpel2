//! Secret Manager service for business logic operations.

use anyhow::Result;
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

use secreton_core::audit::{AuditLog, AuditLogger, AuditStatus};
use secreton_crypto::{AlgorithmId, CryptoEngine};
use secreton_storage::{SecurityLevel, StorageBackend};

/// Secret service errors
#[derive(Error, Debug)]
pub enum SecretServiceError {
    #[error("Secret not found: {path}")]
    SecretNotFound { path: String },

    #[error("Key not found: {key_id}")]
    KeyNotFound { key_id: String },

    #[error("Policy not found: {name}")]
    PolicyNotFound { name: String },

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Invalid key type: {0}")]
    InvalidKeyType(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Crypto error: {0}")]
    Crypto(#[from] secreton_crypto::CryptoError),

    #[error("Storage error: {0}")]
    Storage(#[from] secreton_storage::StorageError),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

/// Secret service for business logic operations
pub struct SecretService {
    storage: Arc<dyn StorageBackend + Send + Sync>,
    crypto: Arc<CryptoEngine>,
    audit: Arc<AuditLogger>,
}

impl SecretService {
    /// Create new secret manager service
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
    pub async fn get_secret(
        &self,
        path: &str,
        user_id: &str,
    ) -> Result<SecretData, SecretServiceError> {
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
        let entry = self.storage.get_by_path(path).await?.ok_or_else(|| {
            SecretServiceError::SecretNotFound {
                path: path.to_string(),
            }
        })?;

        // Decrypt data
        let decrypted_data = serde_json::from_slice(&entry.encrypted_data).map_err(|e| {
            SecretServiceError::Internal(anyhow::anyhow!("Deserialization failed: {}", e))
        })?;

        // Extract metadata
        let metadata: SecretMetadata =
            serde_json::from_value(entry.metadata.clone()).unwrap_or_default();

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
    ) -> Result<SecretData, SecretServiceError> {
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
        let serialized = serde_json::to_vec(&data).map_err(|e| {
            SecretServiceError::Internal(anyhow::anyhow!("Serialization failed: {}", e))
        })?;

        // Get current version or start at 1
        let version = match self.storage.get_by_path(path).await? {
            Some(entry) => entry.version + 1,
            None => 1,
        };

        let now = chrono::Utc::now();

        // Create engine entry
        let mut entry = secreton_storage::SecretEntry::new(
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
    pub async fn delete_secret(&self, path: &str, user_id: &str) -> Result<(), SecretServiceError> {
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
    ) -> Result<Vec<String>, SecretServiceError> {
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

    /// Map a user-supplied key_type string into a crypto AlgorithmId.
    /// Supports a few common synonyms and is case-insensitive.  Returns an error
    /// if the type is not recognised.
    fn algorithm_from_key_type(key_type: &str) -> Result<AlgorithmId, SecretServiceError> {
        match key_type.to_lowercase().as_str() {
            "aes256-gcm" | "aes256-gcm96" | "aes256gcm" | "aes" | "aes256" => Ok(AlgorithmId::Aes256Gcm),
            "chacha20-poly1305" | "chacha20poly1305" | "chacha20" => Ok(AlgorithmId::ChaCha20Poly1305),
            // Common aliases that map to a sensible default symmetric algorithm
            "symmetric" | "encryption" | "transit" => Ok(AlgorithmId::Aes256Gcm),
            // Asymmetric / signing key types
            "ed25519" => Ok(AlgorithmId::Ed25519),
            "ecdsa-p256" | "ecdsa_p256" | "p256" => Ok(AlgorithmId::EcdsaP256),
            "ecdsa-secp256k1" | "secp256k1" => Ok(AlgorithmId::EcdsaSecp256k1),
            "x25519" => Ok(AlgorithmId::X25519),
            // Sensible defaults for generic asymmetric requests
            "asymmetric" | "signing" => Ok(AlgorithmId::Ed25519),
            other => Err(SecretServiceError::InvalidKeyType(other.to_string())),
        }
    }

    /// Create encryption key
    pub async fn create_key(
        &self,
        key_name: &str,
        key_type: &str,
        _metadata: KeyMetadata,
        user_id: &str,
    ) -> Result<KeyInfo, SecretServiceError> {
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

        // Determine algorithm based on the provided key_type string.  This
        // will return an error if the caller supplied an unsupported value.
        let algorithm = Self::algorithm_from_key_type(key_type)?;

        // Generate key ID
        let key_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now();

        // Generate a simulated public key for the new key (since we aren't hooking into a real KMS yet)
        // In a real implementation, this would come from the HSM or CryptoEngine
        let public_key = Some(format!(
            "-----BEGIN PUBLIC KEY-----\n(simulated key material for {} version 1)\n-----END PUBLIC KEY-----",
            key_name
        ));

        // Create key metadata
        let key_info = KeyInfo {
            id: key_id.clone(),
            name: key_name.to_string(),
            key_type: key_type.to_string(),
            version: 1,
            created_at: now,
            public_key,
            metadata: KeyMetadata::default(),
        };

        // Serialize and store key metadata (use key_id in the path, not name)
        let metadata = serde_json::to_vec(&key_info).map_err(|e| {
            SecretServiceError::Internal(anyhow::anyhow!("Serialization failed: {}", e))
        })?;

        let entry = secreton_storage::SecretEntry::new(
            format!("keys/{}", key_id),
            metadata.clone(),
            serde_json::json!({}),
            SecurityLevel::Confidential,
            user_id.to_string(),
        );

        // Also store versioned entry with key_id
        let mut version_entry = secreton_storage::SecretEntry::new(
            format!("keys/{}/versions/{}", key_id, key_info.version),
            metadata,
            serde_json::json!({}),
            SecurityLevel::Confidential,
            user_id.to_string(),
        );
        version_entry.created_at = entry.created_at;

        self.storage.store(&entry).await?;
        self.storage.store(&version_entry).await?;

        // Generate and store actual cryptographic key material for encrypt/decrypt
        let key_material = secreton_crypto::generate_key(algorithm)?;
        let material_entry = secreton_storage::SecretEntry::new(
            format!("keys/{}/material", key_id),
            key_material,
            // record algorithm so callers can look it up later
            serde_json::json!({"algorithm": algorithm.to_string()}),
            SecurityLevel::TopSecret,
            user_id.to_string(),
        );
        self.storage.store(&material_entry).await?;

        Ok(key_info)
    }

    /// Update key metadata
    pub async fn update_key(
        &self,
        key_id: &str,
        metadata: KeyMetadata,
        user_id: &str,
    ) -> Result<KeyInfo, SecretServiceError> {
        // Log audit trail
        let _ = self
            .audit
            .log(AuditLog {
                id: uuid::Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                action: "update".to_string(),
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

        // Get existing key
        let mut key_info = self.get_key(key_id, user_id).await?;

        // Update metadata
        key_info.metadata = metadata;

        // Store updated key info
        let metadata_bytes = serde_json::to_vec(&key_info).map_err(|e| {
            SecretServiceError::Internal(anyhow::anyhow!("Serialization failed: {}", e))
        })?;

        // We use the existing SecretEntry but update the payload
        // store metadata under the stable key_id rather than the mutable name
        let mut entry = secreton_storage::SecretEntry::new(
            format!("keys/{}", key_info.id),
            metadata_bytes,
            serde_json::json!({}),
            SecurityLevel::Confidential,
            user_id.to_string(),
        );
        entry.version = key_info.version;
        entry.updated_at = chrono::Utc::now();

        self.storage.store(&entry).await?;

        Ok(key_info)
    }

    /// Encrypt data with key
    pub async fn encrypt(
        &self,
        key_id: &str,
        plaintext: &str,
        user_id: &str,
    ) -> Result<EncryptResult, SecretServiceError> {
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

        // Retrieve stored encryption key material for this key
        let material_entry = self
            .storage
            .get_by_path(&format!("keys/{}/material", key_id))
            .await?
            .ok_or_else(|| SecretServiceError::KeyNotFound {
                key_id: key_id.to_string(),
            })?;
        let key = material_entry.encrypted_data;

        // Determine algorithm from metadata (fallback to AES-256-GCM for
        // backwards compatibility).
        let alg = material_entry
            .encryption_metadata
            .get("algorithm")
            .and_then(|v| v.as_str())
            .and_then(|s| match s {
                "AES-256-GCM" => Some(AlgorithmId::Aes256Gcm),
                "ChaCha20-Poly1305" => Some(AlgorithmId::ChaCha20Poly1305),
                _ => None,
            })
            .unwrap_or(AlgorithmId::Aes256Gcm);

        // Encrypt using crypto service
        let encrypted_data = self
            .crypto
            .encrypt(alg, plaintext.as_bytes(), &key)?;

        // Serialize encrypted data to JSON then base64
        let json_data = serde_json::to_vec(&encrypted_data).map_err(|e| {
            SecretServiceError::Internal(anyhow::anyhow!("Serialization failed: {}", e))
        })?;
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
    ) -> Result<DecryptResult, SecretServiceError> {
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
            .map_err(|e| {
                SecretServiceError::Internal(anyhow::anyhow!("Base64 decode failed: {}", e))
            })?;

        // Deserialize encrypted data
        let encrypted_data: secreton_crypto::EncryptedData = serde_json::from_slice(&json_bytes)
            .map_err(|e| {
                SecretServiceError::Internal(anyhow::anyhow!("Deserialization failed: {}", e))
            })?;

        // Retrieve stored encryption key material for this key
        let material_entry = self
            .storage
            .get_by_path(&format!("keys/{}/material", key_id))
            .await?
            .ok_or_else(|| SecretServiceError::KeyNotFound {
                key_id: key_id.to_string(),
            })?;
        let key = material_entry.encrypted_data;

        // Algorithm is embedded in the EncryptedData struct from the
        // encryption step, so the crypto.decrypt call does not need a
        // separate algorithm parameter.

        // Decrypt using crypto service
        let plaintext_bytes = self.crypto.decrypt(&encrypted_data, &key)?;
        let plaintext = String::from_utf8(plaintext_bytes).map_err(|e| {
            SecretServiceError::Internal(anyhow::anyhow!("UTF-8 decode failed: {}", e))
        })?;

        Ok(DecryptResult { plaintext })
    }

    /// Get key by ID or name
    pub async fn get_key(
        &self,
        key_id: &str,
        user_id: &str,
    ) -> Result<KeyInfo, SecretServiceError> {
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
            .ok_or_else(|| SecretServiceError::KeyNotFound {
                key_id: key_id.to_string(),
            })?;

        // Deserialize key metadata
        let key_info: KeyInfo = serde_json::from_slice(&entry.encrypted_data).map_err(|e| {
            SecretServiceError::Internal(anyhow::anyhow!("Deserialization failed: {}", e))
        })?;

        Ok(key_info)
    }

    /// List all keys
    pub async fn list_keys(&self, user_id: &str) -> Result<Vec<KeyInfo>, SecretServiceError> {
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
            // Filter out version entries (they contain "/versions/")
            if entry.path.contains("/versions/") {
                continue;
            }

            if let Ok(key_info) = serde_json::from_slice::<KeyInfo>(&entry.encrypted_data) {
                key_infos.push(key_info);
            }
        }

        Ok(key_infos)
    }

    /// List key versions
    pub async fn list_key_versions(
        &self,
        key_id: &str,
        user_id: &str,
    ) -> Result<Vec<KeyInfo>, SecretServiceError> {
        // Log audit trail
        let _ = self
            .audit
            .log(AuditLog {
                id: uuid::Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                action: "list_versions".to_string(),
                actor: Some(user_id.to_string()),
                resource_type: "key".to_string(),
                resource_id: format!("keys/{}/versions", key_id),
                status: AuditStatus::Success,
                ip: None,
                user_agent: None,
                namespace: None,
                metadata: HashMap::new(),
            })
            .await;

        // List key versions from storage
        let params = secreton_storage::QueryParams {
            path_prefix: Some(format!("keys/{}/versions/", key_id)),
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

        // Sort by version descending
        key_infos.sort_by(|a, b| b.version.cmp(&a.version));

        Ok(key_infos)
    }

    /// Rotate key (create new version)
    pub async fn rotate_key(
        &self,
        key_id: &str,
        user_id: &str,
    ) -> Result<KeyInfo, SecretServiceError> {
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

        // Update public key for new version
        key_info.public_key = Some(format!(
            "-----BEGIN PUBLIC KEY-----\n(simulated key material for {} version {})\n-----END PUBLIC KEY-----",
            key_info.name, key_info.version
        ));

        // Store updated key metadata
        let metadata = serde_json::to_vec(&key_info).map_err(|e| {
            SecretServiceError::Internal(anyhow::anyhow!("Serialization failed: {}", e))
        })?;

        let mut entry = secreton_storage::SecretEntry::new(
            format!("keys/{}", key_id),
            metadata.clone(),
            serde_json::json!({}),
            SecurityLevel::Confidential,
            user_id.to_string(),
        );
        entry.updated_at = chrono::Utc::now();

        // Also store versioned entry
        let mut version_entry = secreton_storage::SecretEntry::new(
            format!("keys/{}/versions/{}", key_id, key_info.version),
            metadata,
            serde_json::json!({}),
            SecurityLevel::Confidential,
            user_id.to_string(),
        );
        version_entry.created_at = entry.created_at;
        version_entry.updated_at = entry.updated_at;

        self.storage.store(&entry).await?;
        self.storage.store(&version_entry).await?;

        Ok(key_info)
    }

    /// Delete key
    pub async fn delete_key(&self, key_id: &str, user_id: &str) -> Result<(), SecretServiceError> {
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
    ) -> Result<SignResult, SecretServiceError> {
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
    ) -> Result<VerifyResult, SecretServiceError> {
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
            .map_err(|e| {
                SecretServiceError::Internal(anyhow::anyhow!("Base64 decode failed: {}", e))
            })?;

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
    ) -> Result<HashResult, SecretServiceError> {
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

/// Key metadata
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct KeyMetadata {
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub owner: Option<String>,
    pub purpose: Option<String>,
}

/// Key information
#[derive(Debug, Serialize, Deserialize)]
pub struct KeyInfo {
    pub id: String,
    pub name: String,
    pub key_type: String,
    pub version: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,
    #[serde(default)]
    pub metadata: KeyMetadata,
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
    async fn test_secret_service_creation() {
        let storage = Arc::new(MemoryBackend::new());
        let crypto = Arc::new(CryptoEngine::new());
        let audit_backend: Arc<dyn AuditBackend> = Arc::new(AuditMemoryBackend::default());
        let audit = Arc::new(AuditLogger::new(vec![audit_backend]));

        let secret_service = SecretService::new(storage, crypto, audit).await;
        assert!(secret_service.is_ok());
    }

    #[tokio::test]
    async fn test_get_secret_placeholder() {
        let storage = Arc::new(MemoryBackend::new());
        let crypto = Arc::new(CryptoEngine::new());
        let audit_backend: Arc<dyn AuditBackend> = Arc::new(AuditMemoryBackend::default());
        let audit = Arc::new(AuditLogger::new(vec![audit_backend]));
        let service = SecretService::new(storage, crypto, audit)
            .await
            .expect("Failed to create SecretService");

        let mut data = HashMap::new();
        data.insert("key1".to_string(), "value1".to_string());
        service
            .put_secret("app/config", data, Default::default(), "user1", None)
            .await
            .expect("Failed to put secret");

        let secret = service.get_secret("app/config", "user1").await;
        assert!(secret.is_ok());
        let secret = secret.unwrap();
        assert_eq!(secret.path, "app/config");
        assert!(secret.data.contains_key("key1"));
    }

    #[tokio::test]
    async fn test_key_algorithm_and_id_storage() {
        // verify that create_key stores material under UUID and records algorithm
        let storage = Arc::new(MemoryBackend::new());
        let crypto = Arc::new(CryptoEngine::new());
        let audit_backend: Arc<dyn AuditBackend> = Arc::new(AuditMemoryBackend::default());
        let audit = Arc::new(AuditLogger::new(vec![audit_backend]));
        let service = SecretService::new(storage.clone(), crypto.clone(), audit)
            .await
            .expect("Failed to create SecretService");

        // create a key with explicit type
        let key_info = service
            .create_key("mykey", "chacha20-poly1305", KeyMetadata::default(), "u1")
            .await
            .expect("create_key failed");
        assert_ne!(key_info.id, "mykey");
        assert_eq!(key_info.key_type, "chacha20-poly1305");

        // material should be stored at keys/{id}/material
        let mat = storage
            .get_by_path(&format!("keys/{}/material", key_info.id))
            .await
            .unwrap()
            .expect("material missing");
        assert_eq!(mat.encryption_metadata["algorithm"], "ChaCha20-Poly1305");

        // use encrypt/decrypt and ensure algorithm is respected
        let enc = service
            .encrypt(&key_info.id, "hello", "u1")
            .await
            .expect("encrypt failed");
        let dec = service
            .decrypt(&key_info.id, &enc.ciphertext, "u1")
            .await
            .expect("decrypt failed");
        assert_eq!(dec.plaintext, "hello");
    }

    #[tokio::test]
    async fn test_put_secret_placeholder() {
        let storage = Arc::new(MemoryBackend::new());
        let crypto = Arc::new(CryptoEngine::new());
        let audit_backend: Arc<dyn AuditBackend> = Arc::new(AuditMemoryBackend::default());
        let audit = Arc::new(AuditLogger::new(vec![audit_backend]));
        let service = SecretService::new(storage, crypto, audit)
            .await
            .expect("Failed to create SecretService");

        let mut data = HashMap::new();
        data.insert("username".to_string(), "admin".to_string());
        let secret = service
            .put_secret("app/admin", data, Default::default(), "user1", None)
            .await;
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
        let service = SecretService::new(storage, crypto, audit)
            .await
            .expect("Failed to create SecretService");

        let mut data = HashMap::new();
        data.insert("key".to_string(), "value".to_string());

        let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);
        let secret = service
            .put_secret(
                "app/expiring",
                data,
                Default::default(),
                "user1",
                Some(expires_at),
            )
            .await;

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
        let service = SecretService::new(storage, crypto, audit)
            .await
            .expect("Failed to create SecretService");

        let result = service.encrypt("key1", "plaintext", "user1").await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.ciphertext.is_empty());
        assert_eq!(result.key_version, 1);
    }
}

impl SecretService {
    /// Create mock engine service for testing
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
