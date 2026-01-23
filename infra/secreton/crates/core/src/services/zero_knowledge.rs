//! Zero-Knowledge Service
//!
//! Implements end-to-end encryption (E2EE) with zero-knowledge architecture where
//! the server cannot read client secrets. Clients encrypt data before sending to
//! the server, and the server stores only encrypted blobs with metadata.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────┐                    ┌─────────────┐
//! │   Client    │                    │   Server    │
//! │             │                    │  (Secreton) │
//! │  1. Derive  │                    │             │
//! │     Key     │◄───────────────────┤ 2. Provide  │
//! │             │  Derivation Params │    Params   │
//! │             │                    │             │
//! │  3. Encrypt │                    │             │
//! │     Data    │                    │             │
//! │             │                    │             │
//! │  4. Send    │───────────────────►│ 5. Store    │
//! │  Encrypted  │   Encrypted Blob   │   Blob +    │
//! │     Blob    │                    │   Metadata  │
//! └─────────────┘                    └─────────────┘
//! ```
//!
//! # Key Features
//!
//! - **Client-Side Encryption**: Data encrypted before leaving client
//! - **Server Blindness**: Server never sees plaintext or encryption keys
//! - **Key Derivation**: HKDF-based key derivation with client entropy
//! - **Metadata Storage**: Server stores encryption metadata for client reference
//! - **Audit Privacy**: Audit logs contain no secret content
//!
//! # Use Cases
//!
//! - Storing highly sensitive secrets (passwords, API keys)
//! - Compliance with data residency requirements
//! - Protection against server compromise
//! - End-user controlled encryption
//!
//! # Example
//!
//! ```rust,no_run
//! use secreton_core::services::zero_knowledge::{ZeroKnowledgeService, ZeroKnowledgeMetadata};
//!
//! # async fn example(service: impl ZeroKnowledgeService) -> Result<(), Box<dyn std::error::Error>> {
//! // Client derives encryption key
//! let client_entropy = b"user-password-or-key";
//! let params = service.derive_params(client_entropy).await?;
//!
//! // Client encrypts data (not shown - done client-side)
//! let encrypted_data = vec![/* encrypted bytes */];
//!
//! // Store encrypted data
//! let metadata = ZeroKnowledgeMetadata {
//!     path: "secret/api-key".to_string(),
//!     encryption_algorithm: "aes-256-gcm".to_string(),
//!     key_derivation_params: params,
//!     created_at: chrono::Utc::now(),
//! ;
//!
//! service.store("secret/api-key", encrypted_data, metadata).await?;
//! # Ok(())
//! # }
//! ```

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rand::RngCore;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Zero-knowledge service errors
#[derive(Debug, thiserror::Error)]
pub enum ZkError {
    #[error("Secret not found: {0}")]
    SecretNotFound(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Serialization failed: {0}")]
    SerializationFailed(String),

    #[error("Invalid encryption algorithm: {0}")]
    InvalidAlgorithm(String),

    #[error("Path not zero-knowledge enabled: {0}")]
    NotZkEnabled(String),
}

/// Key derivation parameters for client-side key derivation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeyDerivationParams {
    /// Algorithm used (e.g., "hkdf-sha256")
    pub algorithm: String,
    /// Random salt for key derivation
    pub salt: Vec<u8>,
    /// Context information for HKDF
    pub info: Vec<u8>,
    /// Desired key length in bytes
    pub key_length: usize,
}

impl KeyDerivationParams {
    /// Create new key derivation parameters
    pub fn new(algorithm: String, salt: Vec<u8>, info: Vec<u8>, key_length: usize) -> Self {
        Self {
            algorithm,
            salt,
            info,
            key_length,
        }
    }

    /// Generate default HKDF-SHA256 parameters
    pub fn default_hkdf() -> Self {
        let mut salt = vec![0u8; 32];
        OsRng.fill_bytes(&mut salt);

        Self {
            algorithm: "hkdf-sha256".to_string(),
            salt,
            info: b"secreton-zk-v1".to_vec(),
            key_length: 32, // AES-256
        }
    }
}

/// Zero-knowledge secret metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZeroKnowledgeMetadata {
    /// Path where secret is stored
    pub path: String,
    /// Encryption algorithm used by client (e.g., "aes-256-gcm")
    pub encryption_algorithm: String,
    /// Key derivation parameters
    pub key_derivation_params: KeyDerivationParams,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

impl ZeroKnowledgeMetadata {
    /// Create new zero-knowledge metadata
    pub fn new(
        path: String,
        encryption_algorithm: String,
        key_derivation_params: KeyDerivationParams,
    ) -> Self {
        Self {
            path,
            encryption_algorithm,
            key_derivation_params,
            created_at: Utc::now(),
        }
    }
}

/// Stored zero-knowledge secret
#[derive(Debug, Clone)]
struct ZkSecret {
    /// Encrypted data blob
    encrypted_data: Vec<u8>,
    /// Metadata about encryption
    metadata: ZeroKnowledgeMetadata,
}

/// Zero-Knowledge Service trait
#[async_trait]
pub trait ZeroKnowledgeService: Send + Sync {
    /// Store pre-encrypted secret
    ///
    /// # Arguments
    /// * `path` - Secret path (e.g., "secret/api-key")
    /// * `encrypted_data` - Pre-encrypted data from client
    /// * `metadata` - Encryption metadata
    async fn store(
        &self,
        path: &str,
        encrypted_data: Vec<u8>,
        metadata: ZeroKnowledgeMetadata,
    ) -> Result<(), ZkError>;

    /// Retrieve encrypted secret
    ///
    /// Returns the encrypted blob and metadata without server-side decryption
    async fn retrieve(&self, path: &str) -> Result<(Vec<u8>, ZeroKnowledgeMetadata), ZkError>;

    /// Derive key parameters for client
    ///
    /// Generates derivation parameters that client can use to derive encryption key
    async fn derive_params(&self, client_entropy: &[u8]) -> Result<KeyDerivationParams, ZkError>;

    /// Check if path is zero-knowledge enabled
    async fn is_zk_enabled(&self, path: &str) -> Result<bool, ZkError>;

    /// Delete zero-knowledge secret
    async fn delete(&self, path: &str) -> Result<(), ZkError>;

    /// List all zero-knowledge secret paths
    async fn list_paths(&self) -> Result<Vec<String>, ZkError>;
}

/// In-memory implementation of Zero-Knowledge Service
///
/// This is a reference implementation for testing and development.
/// Production deployments should use a persistent storage backend.
pub struct ZeroKnowledgeServiceImpl {
    /// In-memory storage of encrypted secrets
    secrets: Arc<RwLock<HashMap<String, ZkSecret>>>,
}

impl ZeroKnowledgeServiceImpl {
    /// Create new zero-knowledge service
    pub fn new() -> Self {
        Self {
            secrets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Validate path format
    fn validate_path(path: &str) -> Result<(), ZkError> {
        if path.is_empty() {
            return Err(ZkError::InvalidPath("Path cannot be empty".to_string()));
        }

        if path.contains("..") {
            return Err(ZkError::InvalidPath("Path cannot contain '..'".to_string()));
        }

        if !path.starts_with('/') && !path.contains('/') {
            return Err(ZkError::InvalidPath(
                "Path must contain at least one '/'".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate encryption algorithm
    fn validate_algorithm(algorithm: &str) -> Result<(), ZkError> {
        match algorithm {
            "aes-256-gcm" | "chacha20-poly1305" | "aes-128-gcm" => Ok(()),
            _ => Err(ZkError::InvalidAlgorithm(format!(
                "Unsupported algorithm: {}",
                algorithm
            ))),
        }
    }
}

impl Default for ZeroKnowledgeServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ZeroKnowledgeService for ZeroKnowledgeServiceImpl {
    async fn store(
        &self,
        path: &str,
        encrypted_data: Vec<u8>,
        metadata: ZeroKnowledgeMetadata,
    ) -> Result<(), ZkError> {
        // Validate inputs
        Self::validate_path(path)?;
        Self::validate_algorithm(&metadata.encryption_algorithm)?;

        if encrypted_data.is_empty() {
            return Err(ZkError::StorageError(
                "Encrypted data cannot be empty".to_string(),
            ));
        }

        // Store secret
        let secret = ZkSecret {
            encrypted_data,
            metadata,
        };

        let mut secrets = self.secrets.write().await;
        secrets.insert(path.to_string(), secret);

        Ok(())
    }

    async fn retrieve(&self, path: &str) -> Result<(Vec<u8>, ZeroKnowledgeMetadata), ZkError> {
        Self::validate_path(path)?;

        let secrets = self.secrets.read().await;
        let secret = secrets
            .get(path)
            .ok_or_else(|| ZkError::SecretNotFound(path.to_string()))?;

        Ok((secret.encrypted_data.clone(), secret.metadata.clone()))
    }

    async fn derive_params(&self, client_entropy: &[u8]) -> Result<KeyDerivationParams, ZkError> {
        if client_entropy.is_empty() {
            return Err(ZkError::KeyDerivationFailed(
                "Client entropy cannot be empty".to_string(),
            ));
        }

        // Generate random salt
        let mut salt = vec![0u8; 32];
        OsRng.fill_bytes(&mut salt);

        // Use client entropy as part of info parameter
        let mut info = b"secreton-zk-v1-".to_vec();
        info.extend_from_slice(&client_entropy[..std::cmp::min(client_entropy.len(), 16)]);

        Ok(KeyDerivationParams {
            algorithm: "hkdf-sha256".to_string(),
            salt,
            info,
            key_length: 32, // AES-256
        })
    }

    async fn is_zk_enabled(&self, path: &str) -> Result<bool, ZkError> {
        Self::validate_path(path)?;

        let secrets = self.secrets.read().await;
        Ok(secrets.contains_key(path))
    }

    async fn delete(&self, path: &str) -> Result<(), ZkError> {
        Self::validate_path(path)?;

        let mut secrets = self.secrets.write().await;
        secrets
            .remove(path)
            .ok_or_else(|| ZkError::SecretNotFound(path.to_string()))?;

        Ok(())
    }

    async fn list_paths(&self) -> Result<Vec<String>, ZkError> {
        let secrets = self.secrets.read().await;
        let mut paths: Vec<String> = secrets.keys().cloned().collect();
        paths.sort();
        Ok(paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_store_and_retrieve() {
        let service = ZeroKnowledgeServiceImpl::new();

        let encrypted_data = b"encrypted-secret-data".to_vec();
        let metadata = ZeroKnowledgeMetadata::new(
            "secret/test".to_string(),
            "aes-256-gcm".to_string(),
            KeyDerivationParams::default_hkdf(),
        );

        // Store secret
        service
            .store("secret/test", encrypted_data.clone(), metadata.clone())
            .await
            .unwrap();

        // Retrieve secret
        let (retrieved_data, retrieved_metadata) = service.retrieve("secret/test").await.unwrap();

        assert_eq!(retrieved_data, encrypted_data);
        assert_eq!(retrieved_metadata.path, metadata.path);
        assert_eq!(
            retrieved_metadata.encryption_algorithm,
            metadata.encryption_algorithm
        );
    }

    #[tokio::test]
    async fn test_derive_params() {
        let service = ZeroKnowledgeServiceImpl::new();

        let client_entropy = b"user-password-123";
        let params = service.derive_params(client_entropy).await.unwrap();

        assert_eq!(params.algorithm, "hkdf-sha256");
        assert_eq!(params.salt.len(), 32);
        assert_eq!(params.key_length, 32);
        assert!(!params.info.is_empty());
    }

    #[tokio::test]
    async fn test_is_zk_enabled() {
        let service = ZeroKnowledgeServiceImpl::new();

        // Initially not enabled
        assert!(!service.is_zk_enabled("secret/test").await.unwrap());

        // Store a secret
        let encrypted_data = b"encrypted-data".to_vec();
        let metadata = ZeroKnowledgeMetadata::new(
            "secret/test".to_string(),
            "aes-256-gcm".to_string(),
            KeyDerivationParams::default_hkdf(),
        );

        service
            .store("secret/test", encrypted_data, metadata)
            .await
            .unwrap();

        // Now enabled
        assert!(service.is_zk_enabled("secret/test").await.unwrap());
    }

    #[tokio::test]
    async fn test_delete() {
        let service = ZeroKnowledgeServiceImpl::new();

        let encrypted_data = b"encrypted-data".to_vec();
        let metadata = ZeroKnowledgeMetadata::new(
            "secret/test".to_string(),
            "aes-256-gcm".to_string(),
            KeyDerivationParams::default_hkdf(),
        );

        service
            .store("secret/test", encrypted_data, metadata)
            .await
            .unwrap();

        // Delete secret
        service.delete("secret/test").await.unwrap();

        // Should not be found
        assert!(service.retrieve("secret/test").await.is_err());
    }

    #[tokio::test]
    async fn test_list_paths() {
        let service = ZeroKnowledgeServiceImpl::new();

        // Store multiple secrets
        for i in 1..=3 {
            let path = format!("secret/test{}", i);
            let encrypted_data = format!("encrypted-data-{}", i).into_bytes();
            let metadata = ZeroKnowledgeMetadata::new(
                path.clone(),
                "aes-256-gcm".to_string(),
                KeyDerivationParams::default_hkdf(),
            );

            service
                .store(&path, encrypted_data, metadata)
                .await
                .unwrap();
        }

        let paths = service.list_paths().await.unwrap();
        assert_eq!(paths.len(), 3);
        assert_eq!(paths[0], "secret/test1");
        assert_eq!(paths[1], "secret/test2");
        assert_eq!(paths[2], "secret/test3");
    }

    #[tokio::test]
    async fn test_invalid_path() {
        let service = ZeroKnowledgeServiceImpl::new();

        let encrypted_data = b"data".to_vec();
        let metadata = ZeroKnowledgeMetadata::new(
            "invalid".to_string(),
            "aes-256-gcm".to_string(),
            KeyDerivationParams::default_hkdf(),
        );

        // Empty path
        assert!(
            service
                .store("", encrypted_data.clone(), metadata.clone())
                .await
                .is_err()
        );

        // Path with ..
        assert!(
            service
                .store("secret/../other", encrypted_data.clone(), metadata.clone())
                .await
                .is_err()
        );

        // Path without /
        assert!(
            service
                .store("invalid", encrypted_data, metadata)
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn test_invalid_algorithm() {
        let service = ZeroKnowledgeServiceImpl::new();

        let encrypted_data = b"data".to_vec();
        let metadata = ZeroKnowledgeMetadata::new(
            "secret/test".to_string(),
            "invalid-algorithm".to_string(),
            KeyDerivationParams::default_hkdf(),
        );

        assert!(
            service
                .store("secret/test", encrypted_data, metadata)
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn test_empty_encrypted_data() {
        let service = ZeroKnowledgeServiceImpl::new();

        let encrypted_data = Vec::new();
        let metadata = ZeroKnowledgeMetadata::new(
            "secret/test".to_string(),
            "aes-256-gcm".to_string(),
            KeyDerivationParams::default_hkdf(),
        );

        assert!(
            service
                .store("secret/test", encrypted_data, metadata)
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn test_retrieve_nonexistent() {
        let service = ZeroKnowledgeServiceImpl::new();

        assert!(service.retrieve("secret/nonexistent").await.is_err());
    }

    #[tokio::test]
    async fn test_empty_client_entropy() {
        let service = ZeroKnowledgeServiceImpl::new();

        assert!(service.derive_params(&[]).await.is_err());
    }
}
