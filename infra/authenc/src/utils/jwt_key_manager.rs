//! JWT Key Management with Secreton Integration
//!
//! This module provides secure JWT signing key management by retrieving
//! keys from Secreton instead of using hardcoded values.

use crate::secreton_client::{GrpcSecretonClient, SecretonClientTrait, SecretonError};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// JWT signing key with metadata
#[derive(Clone, Debug)]
pub struct JwtKey {
    /// The actual signing key bytes
    pub key: Vec<u8>,
    /// When this key was retrieved
    pub retrieved_at: SystemTime,
    /// Time-to-live for this key in seconds
    pub ttl: u64,
    /// Algorithm used (e.g., "HS256", "RS256")
    pub algorithm: String,
    /// Key version for rotation tracking
    pub version: Option<u32>,
}

impl JwtKey {
    /// Check if the key is expired
    pub fn is_expired(&self) -> bool {
        if let Ok(elapsed) = self.retrieved_at.elapsed() {
            elapsed.as_secs() > self.ttl
        } else {
            // If we can't determine elapsed time, consider it expired
            true
        }
    }

    /// Get remaining TTL in seconds
    pub fn remaining_ttl(&self) -> u64 {
        if let Ok(elapsed) = self.retrieved_at.elapsed() {
            self.ttl.saturating_sub(elapsed.as_secs())
        } else {
            0
        }
    }
}

/// JWT Key Manager that retrieves signing keys from Secreton
pub struct JwtKeyManager {
    /// Secreton client for key retrieval
    secreton: Arc<GrpcSecretonClient>,
    /// Cached signing key
    key_cache: Arc<RwLock<Option<JwtKey>>>,
    /// Path to the JWT signing key in Secreton
    key_path: String,
    /// Realm for the key (optional namespace)
    key_realm: Option<String>,
    /// Default TTL for cached keys (1 hour)
    cache_ttl: u64,
}

impl JwtKeyManager {
    /// Create a new JWT Key Manager
    ///
    /// # Arguments
    /// * `secreton` - Secreton gRPC client
    /// * `key_path` - Path to JWT signing key in Secreton (e.g., "auth/jwt-signing-key")
    /// * `key_realm` - Optional namespace/realm for the key
    pub fn new(
        secreton: Arc<GrpcSecretonClient>,
        key_path: String,
        key_realm: Option<String>,
    ) -> Self {
        Self {
            secreton,
            key_cache: Arc::new(RwLock::new(None)),
            key_path,
            key_realm,
            cache_ttl: 3600, // 1 hour default
        }
    }

    /// Create a new JWT Key Manager with custom cache TTL
    pub fn new_with_ttl(
        secreton: Arc<GrpcSecretonClient>,
        key_path: String,
        key_realm: Option<String>,
        cache_ttl: u64,
    ) -> Self {
        Self {
            secreton,
            key_cache: Arc::new(RwLock::new(None)),
            key_path,
            key_realm,
            cache_ttl,
        }
    }

    /// Get the JWT signing key (from cache or Secreton)
    ///
    /// This method checks the cache first. If the cached key is expired
    /// or doesn't exist, it retrieves a fresh key from Secreton.
    pub async fn get_signing_key(&self) -> Result<Vec<u8>, JwtKeyError> {
        // Check cache first
        {
            let cache = self.key_cache.read().await;
            if let Some(cached_key) = cache.as_ref() {
                if !cached_key.is_expired() {
                    info!(
                        "Using cached JWT key (TTL remaining: {}s)",
                        cached_key.remaining_ttl()
                    );
                    return Ok(cached_key.key.clone());
                } else {
                    warn!("Cached JWT key expired, fetching new key from Secreton");
                }
            }
        }

        // Fetch from Secreton
        self.refresh_key().await
    }

    /// Force refresh the signing key from Secreton
    pub async fn refresh_key(&self) -> Result<Vec<u8>, JwtKeyError> {
        info!("Fetching JWT signing key from Secreton: {}", self.key_path);

        let secret = self
            .secreton
            .get_secret(&self.key_path, self.key_realm.as_deref())
            .await
            .ok_or_else(|| JwtKeyError::KeyNotFound(self.key_path.clone()))?;

        // Extract key from metadata
        let key_data = secret
            .metadata
            .as_ref()
            .and_then(|m| m.get("key"))
            .or_else(|| Some(&secret.value))
            .ok_or_else(|| JwtKeyError::InvalidKeyFormat("No key field found".to_string()))?;

        // Decode from base64 if needed
        let key_bytes = if key_data.starts_with("base64:") {
            base64::decode(&key_data[7..])
                .map_err(|e| JwtKeyError::InvalidKeyFormat(format!("Base64 decode error: {}", e)))?
        } else {
            key_data.as_bytes().to_vec()
        };

        // Extract algorithm
        let algorithm = secret
            .metadata
            .as_ref()
            .and_then(|m| m.get("algorithm"))
            .map(|s| s.clone())
            .unwrap_or_else(|| "HS256".to_string());

        // Create JWT key
        let jwt_key = JwtKey {
            key: key_bytes.clone(),
            retrieved_at: SystemTime::now(),
            ttl: self.cache_ttl,
            algorithm,
            version: secret.version,
        };

        // Update cache
        *self.key_cache.write().await = Some(jwt_key.clone());

        info!(
            "Successfully retrieved JWT key (version: {:?}, algorithm: {})",
            jwt_key.version, jwt_key.algorithm
        );

        Ok(jwt_key.key)
    }

    /// Initialize JWT key in Secreton if it doesn't exist
    ///
    /// This should be called on application startup to ensure the key exists
    pub async fn initialize_key_if_missing(&self) -> Result<(), JwtKeyError> {
        // Check if key exists
        match self
            .secreton
            .get_secret(&self.key_path, self.key_realm.as_deref())
            .await
        {
            Some(_) => {
                info!("JWT signing key already exists in Secreton");
                Ok(())
            }
            None => {
                info!("JWT signing key not found, generating new key");
                self.generate_and_store_key().await
            }
        }
    }

    /// Generate a new JWT signing key and store it in Secreton
    async fn generate_and_store_key(&self) -> Result<(), JwtKeyError> {
        // Generate a secure random key (256 bits for HS256)
        let key = self.generate_secure_key(32)?;

        // Encode as base64
        let encoded_key = format!("base64:{}", base64::encode(&key));

        // Create metadata
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("key".to_string(), encoded_key);
        metadata.insert("algorithm".to_string(), "HS256".to_string());
        metadata.insert("created_at".to_string(), chrono::Utc::now().to_rfc3339());
        metadata.insert(
            "description".to_string(),
            "JWT signing key for authentication".to_string(),
        );

        // Store in Secreton
        self.secreton
            .put_secret(
                &self.key_path,
                "", // Value is in metadata
                self.key_realm.as_deref(),
                Some(metadata),
            )
            .await
            .map_err(JwtKeyError::SecretonError)?;

        info!("Successfully generated and stored new JWT signing key");

        Ok(())
    }

    /// Generate a cryptographically secure random key
    fn generate_secure_key(&self, size: usize) -> Result<Vec<u8>, JwtKeyError> {
        use rand::RngCore;

        let mut key = vec![0u8; size];
        rand::thread_rng()
            .try_fill_bytes(&mut key)
            .map_err(|e| JwtKeyError::KeyGenerationFailed(e.to_string()))?;

        Ok(key)
    }

    /// Get the current cached key metadata (for monitoring/debugging)
    pub async fn get_cached_key_info(&self) -> Option<(String, u64, Option<u32>)> {
        let cache = self.key_cache.read().await;
        cache
            .as_ref()
            .map(|k| (k.algorithm.clone(), k.remaining_ttl(), k.version))
    }

    /// Rotate the JWT signing key
    ///
    /// Generates a new key and stores it in Secreton, while keeping the old one
    /// in cache for a grace period to allow in-flight tokens to validate
    pub async fn rotate_key(&self) -> Result<(), JwtKeyError> {
        info!("Rotating JWT signing key");

        // Generate new key
        self.generate_and_store_key().await?;

        // The next get_signing_key() call will fetch the new key
        // Old key remains in cache until it expires naturally

        Ok(())
    }
}

/// JWT Key Manager errors
#[derive(Debug, thiserror::Error)]
pub enum JwtKeyError {
    #[error("JWT key not found in Secreton: {0}")]
    KeyNotFound(String),

    #[error("Invalid key format: {0}")]
    InvalidKeyFormat(String),

    #[error("Failed to generate key: {0}")]
    KeyGenerationFailed(String),

    #[error("Secreton error: {0}")]
    SecretonError(#[from] SecretonError),

    #[error("Key retrieval failed: {0}")]
    RetrievalFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_key_expiration() {
        let key = JwtKey {
            key: vec![1, 2, 3, 4],
            retrieved_at: SystemTime::now() - Duration::from_secs(3700),
            ttl: 3600,
            algorithm: "HS256".to_string(),
            version: Some(1),
        };

        assert!(key.is_expired());
        assert_eq!(key.remaining_ttl(), 0);
    }

    #[test]
    fn test_jwt_key_not_expired() {
        let key = JwtKey {
            key: vec![1, 2, 3, 4],
            retrieved_at: SystemTime::now(),
            ttl: 3600,
            algorithm: "HS256".to_string(),
            version: Some(1),
        };

        assert!(!key.is_expired());
        assert!(key.remaining_ttl() > 3500);
    }
}
