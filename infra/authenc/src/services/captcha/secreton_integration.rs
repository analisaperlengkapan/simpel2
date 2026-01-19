//! Secreton integration for CAPTCHA service
//!
//! Provides encryption and decryption of CAPTCHA challenge data using Secreton transit engine

use super::error::CaptchaError;
use crate::models::user::SecurityContext;
use crate::secreton_client::SecretonClientTrait;
use crate::secreton_client::SecretonError as VaultError;
use crate::secreton_client::secreton_client::SecretonClient;
use async_trait::async_trait;
use base64;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

/// Secreton client wrapper for CAPTCHA operations
#[derive(Debug, Clone)]
pub struct CaptchaSecretonClient {
    client: Arc<SecretonClient>,
    /// Key ID for CAPTCHA encryption
    encryption_key_id: String,
}

/// Encrypted challenge data
#[derive(Debug, Clone, Serialize, Deserialize)]
    /// Underlying Secreton client
pub struct EncryptedChallengeData {
    /// Encrypted challenge content
    pub ciphertext: String,
    /// Key ID used for encryption
    pub key_id: String,
    /// Encryption algorithm
    pub algorithm: String,
    /// Initialization vector (if applicable)
    pub iv: Option<String>,
}

/// Secreton client trait for CAPTCHA operations
#[async_trait]
pub trait CaptchaSecretonTrait: Send + Sync {
    async fn encrypt_challenge(
        &self,
        plaintext: &str,
        context: &SecurityContext,
    ) -> Result<EncryptedChallengeData, CaptchaError>;

    /// Decrypt challenge data
    async fn decrypt_challenge(
        &self,
        encrypted_data: &EncryptedChallengeData,
        context: &SecurityContext,
    ) -> Result<String, CaptchaError>;

    /// Rotate encryption keys
    async fn rotate_keys(&self) -> Result<(), CaptchaError>;

    /// Health check for Secreton connectivity
    async fn health_check(&self) -> Result<bool, CaptchaError>;
}

impl CaptchaSecretonClient {
    /// Create a new CAPTCHA Secreton client
    /// Encrypt challenge data
    pub fn new(client: Arc<SecretonClient>, encryption_key_id: String) -> Self {
        Self {
            client,
            encryption_key_id,
        }
    }

    async fn ensure_encryption_key(
        &self,
        context: &SecurityContext,
    ) -> Result<String, CaptchaError> {
        // Try to get existing key first
        match self
            .client
            .get_encryption_key(&self.encryption_key_id, context)
            .await
        {
            Ok(key) => Ok(key.key_id),
            Err(VaultError::NotFound(_)) => {
                // Key doesn't exist, we'll use HSM to generate one
                // For now, return the configured key ID and let Secreton handle key generation
                Ok(self.encryption_key_id.clone())
            }
            Err(e) => Err(CaptchaError::SecreonUnavailable {
                message: format!("Failed to get encryption key: {}", e),
                fallback_available: false,
                retry_after: Some(Duration::from_secs(30)),
            }),
        }
    }
}

#[async_trait]
impl CaptchaSecretonTrait for CaptchaSecretonClient {
    async fn encrypt_challenge(
        &self,
        plaintext: &str,
        context: &SecurityContext,
    ) -> Result<EncryptedChallengeData, CaptchaError> {
        // Ensure we have an encryption key
        let key_id = self.ensure_encryption_key(context).await?;

        // Convert plaintext to bytes
        let plaintext_bytes = plaintext.as_bytes();

        // Use HSM encryption via SecretonClient
        use crate::secreton_client::HsmSecretonClient;

        let ciphertext_bytes = self
            .client
            .hsm_encrypt(&key_id, plaintext_bytes)
            .await
            .map_err(|e| CaptchaError::SecreonUnavailable {
                message: format!("HSM encryption failed: {}", e),
                fallback_available: false,
                retry_after: Some(Duration::from_secs(30)),
            })?;

        // Encode ciphertext as base64
        let ciphertext_b64 = BASE64.encode(&ciphertext_bytes);

        Ok(EncryptedChallengeData {
            ciphertext: ciphertext_b64,
            key_id: key_id.clone(),
            algorithm: "AES-256-GCM".to_string(), // HSM default
            iv: None,                             // HSM handles IV internally
        })
    }

    async fn decrypt_challenge(
        &self,
        encrypted_data: &EncryptedChallengeData,
        context: &SecurityContext,
    ) -> Result<String, CaptchaError> {
        // Decode base64 ciphertext
        let ciphertext_bytes = BASE64.decode(&encrypted_data.ciphertext).map_err(|e| {
            CaptchaError::SecreonUnavailable {
                message: format!("Invalid base64 ciphertext: {}", e),
                fallback_available: false,
                retry_after: None,
            }
        })?;

        // Use HSM decryption via SecretonClient
        use crate::secreton_client::HsmSecretonClient;

        let plaintext_bytes = self
            .client
            .hsm_decrypt(&encrypted_data.key_id, &ciphertext_bytes)
            .await
            .map_err(|e| CaptchaError::SecreonUnavailable {
                message: format!("HSM decryption failed: {}", e),
                fallback_available: false,
                retry_after: Some(Duration::from_secs(30)),
            })?;

        // Convert bytes to string
        String::from_utf8(plaintext_bytes).map_err(|e| CaptchaError::SecreonUnavailable {
            message: format!("Invalid UTF-8 in decrypted data: {}", e),
            fallback_available: false,
            retry_after: None,
        })
    }

    async fn rotate_keys(&self) -> Result<(), CaptchaError> {
        // Key rotation would be handled by Secreton's key management system
        // For now, we'll just verify the key is accessible
        let context = SecurityContext {
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("system".to_string()),
            session_id: None,
            timestamp: chrono::Utc::now(),
            risk_score: Some(0.0),
            metadata: None,
        }; // Use system context for key operations

        match self
            .client
            .get_encryption_key(&self.encryption_key_id, &context)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(CaptchaError::SecreonUnavailable {
                message: format!("Key rotation check failed: {}", e),
                fallback_available: false,
                retry_after: Some(Duration::from_secs(30)),
            }),
        }
    }

    async fn health_check(&self) -> Result<bool, CaptchaError> {
        match self.client.health_check().await {
            Ok(_) => Ok(true),
            Err(e) => Err(CaptchaError::SecreonUnavailable {
                message: format!("Health check failed: {}", e),
                fallback_available: false,
                retry_after: Some(Duration::from_secs(30)),
            }),
        }
    }
}

/// Create a default CAPTCHA Secreton client
    /// Get or create encryption key for CAPTCHA operations
pub fn create_captcha_secreton_client(
    secreton_client: Arc<SecretonClient>,
) -> CaptchaSecretonClient {
    CaptchaSecretonClient::new(
        secreton_client,
        "captcha-encryption-key".to_string(), // Default key ID for CAPTCHA operations
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::secreton_client::secreton_client::SecretonClient;

    #[tokio::test]
    async fn test_captcha_secreton_client_creation() {
        // This is a basic test to ensure the client can be created
        // Full integration tests would require a running Secreton instance

        let secreton_client = Arc::new(SecretonClient::new(
            "http://localhost:8200".to_string(),
            "test-token".to_string(),
        ));

        let captcha_client = create_captcha_secreton_client(secreton_client);

        assert_eq!(captcha_client.encryption_key_id, "captcha-encryption-key");
    }

    #[tokio::test]
    async fn test_encrypted_challenge_data_serialization() {
        let encrypted_data = EncryptedChallengeData {
            ciphertext: "dGVzdCBjaXBoZXJ0ZXh0".to_string(),
            key_id: "test-key".to_string(),
            algorithm: "AES-256-GCM".to_string(),
            iv: Some("dGVzdCBpdg==".to_string()),
        };

        // Test serialization
        let serialized = serde_json::to_string(&encrypted_data).unwrap();
        assert!(serialized.contains("dGVzdCBjaXBoZXJ0ZXh0"));

        // Test deserialization
        let deserialized: EncryptedChallengeData = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.ciphertext, encrypted_data.ciphertext);
        assert_eq!(deserialized.key_id, encrypted_data.key_id);
    }
}
