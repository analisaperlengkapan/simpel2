//! JWT Key Management with Secreton Integration
//!
//! This module provides secure JWT signing key management by retrieving
//! keys from Secreton instead of using hardcoded values.

use crate::{AuthencError, Result};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use tracing::{info, warn};

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

/// Secreton client trait for key retrieval
///
/// This trait abstracts the Secreton client to allow for testing and flexibility.
#[async_trait::async_trait]
pub trait SecretonClient: Send + Sync {
    /// Get a secret from Secreton
    async fn get_secret(&self, path: &str, realm: Option<&str>) -> Option<String>;
}

/// JWT Key Manager that retrieves signing keys from Secreton
pub struct JwtKeyManager<C: SecretonClient> {
    /// Secreton client for key retrieval
    secreton: Arc<C>,
    /// Cached signing key
    key_cache: Arc<RwLock<Option<JwtKey>>>,
    /// Path to the JWT signing key in Secreton
    key_path: String,
    /// Realm for the key (optional namespace)
    key_realm: Option<String>,
    /// Default TTL for cached keys (1 hour)
    cache_ttl: u64,
}

impl<C: SecretonClient> JwtKeyManager<C> {
    /// Create a new JWT Key Manager
    ///
    /// # Arguments
    /// * `secreton` - Secreton client
    /// * `key_path` - Path to JWT signing key in Secreton (e.g., "auth/jwt-signing-key")
    /// * `key_realm` - Optional namespace/realm for the key
    pub fn new(secreton: Arc<C>, key_path: String, key_realm: Option<String>) -> Self {
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
        secreton: Arc<C>,
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
    pub async fn get_signing_key(&self) -> Result<Vec<u8>> {
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
    pub async fn refresh_key(&self) -> Result<Vec<u8>> {
        info!("Fetching JWT signing key from Secreton: {}", self.key_path);

        let secret_value = self
            .secreton
            .get_secret(&self.key_path, self.key_realm.as_deref())
            .await
            .ok_or_else(|| AuthencError::crypto(format!("JWT key not found: {}", self.key_path)))?;

        // Parse the secret value (assuming it's the raw key or base64)
        let key_data = secret_value;

        // Decode from base64 if needed
        let key_bytes = if let Some(b64) = key_data.strip_prefix("base64:") {
            use base64::{Engine as _, engine::general_purpose::STANDARD};
            STANDARD
                .decode(b64)
                .map_err(|e| AuthencError::crypto(format!("Base64 decode error: {}", e)))?
        } else {
            let bytes = key_data.as_bytes().to_vec();
            if bytes.is_empty() {
                return Err(AuthencError::crypto("Empty key data".to_string()));
            }
            bytes
        };

        // Extract algorithm (defaulting to HS256)
        let algorithm = "HS256".to_string();

        // Create JWT key
        let jwt_key = JwtKey {
            key: key_bytes.clone(),
            retrieved_at: SystemTime::now(),
            ttl: self.cache_ttl,
            algorithm,
            version: None, // Version not returned by simple get_secret
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
    pub async fn initialize_key_if_missing(&self) -> Result<()> {
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
                info!("JWT signing key not found, manual generation required");
                Err(AuthencError::crypto(
                    "JWT signing key not found in Secreton. Please generate and store a key manually.".to_string()
                ))
            }
        }
    }

    /// Get the current cached key metadata (for monitoring/debugging)
    pub async fn get_cached_key_info(&self) -> Option<(String, u64, Option<u32>)> {
        let cache = self.key_cache.read().await;
        cache
            .as_ref()
            .map(|k| (k.algorithm.clone(), k.remaining_ttl(), k.version))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

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

    // Mock Secreton client for testing
    struct MockSecretonClient {
        secret: Option<String>,
    }

    #[async_trait::async_trait]
    impl SecretonClient for MockSecretonClient {
        async fn get_secret(&self, _path: &str, _realm: Option<&str>) -> Option<String> {
            self.secret.clone()
        }
    }

    #[tokio::test]
    async fn test_get_signing_key_from_secreton() {
        let mock_client = Arc::new(MockSecretonClient {
            secret: Some("base64:dGVzdC1rZXktZGF0YQ==".to_string()),
        });

        let manager = JwtKeyManager::new(mock_client, "auth/jwt-signing-key".to_string(), None);

        let key = manager.get_signing_key().await.unwrap();
        assert!(!key.is_empty());
    }

    #[tokio::test]
    async fn test_key_caching() {
        let mock_client = Arc::new(MockSecretonClient {
            secret: Some("base64:dGVzdC1rZXktZGF0YQ==".to_string()),
        });

        let manager = JwtKeyManager::new(mock_client, "auth/jwt-signing-key".to_string(), None);

        // First call should fetch from Secreton
        let key1 = manager.get_signing_key().await.unwrap();

        // Second call should use cache
        let key2 = manager.get_signing_key().await.unwrap();

        assert_eq!(key1, key2);
    }
}
