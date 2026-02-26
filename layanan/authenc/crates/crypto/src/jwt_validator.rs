//! JWT Validation Service with Caching
//!
//! This module provides an optimized JWT validation service with multi-layer caching
//! to achieve fast token validation (< 10ms for cached tokens).
//!
//! # Features
//! - JWT signature verification result caching (TTL: 5 minutes)
//! - Token blacklist checking with cache integration
//! - Fast-path validation for cached results
//! - Automatic cache invalidation on token revocation
//!
//! # Performance
//! - Cached validation: < 10ms (target)
//! - Uncached validation: < 50ms (includes signature verification)
//! - Cache hit ratio target: > 80%

use crate::Result;
use crate::jwt::{JwtService, TokenClaims};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, warn};

/// JWT validation result that can be cached
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the token is valid
    pub valid: bool,
    /// User ID from token claims
    pub user_id: Option<String>,
    /// Token expiration timestamp
    pub expires_at: Option<i64>,
    /// Error message if validation failed
    pub error: Option<String>,
    /// Timestamp when this result was cached
    pub cached_at: i64,
}

/// Cache trait for JWT validation
///
/// This trait abstracts the cache implementation to allow for testing and flexibility.
#[async_trait::async_trait]
pub trait ValidationCache: Send + Sync {
    /// Get a value from cache
    async fn get(&self, key: &str) -> Result<Option<serde_json::Value>>;

    /// Set a value in cache with TTL
    async fn set(&self, key: &str, value: &serde_json::Value, ttl: Duration) -> Result<()>;

    /// Check if a key exists in cache
    async fn exists(&self, key: &str) -> Result<bool>;

    /// Delete a key from cache
    async fn delete(&self, key: &str) -> Result<()>;
}

/// JWT Validator with caching support
pub struct JwtValidator {
    /// JWT service for token verification
    jwt_service: Arc<JwtService>,
    /// Cache for validation results
    cache: Option<Arc<dyn ValidationCache>>,
    /// Cache TTL for validation results (default: 5 minutes)
    cache_ttl: Duration,
}

impl JwtValidator {
    /// Create a new JWT validator
    ///
    /// # Arguments
    /// * `jwt_service` - JWT service for token verification
    /// * `cache` - Optional cache implementation for storing validation results
    ///
    /// # Returns
    /// A new `JwtValidator` instance
    pub fn new(jwt_service: Arc<JwtService>, cache: Option<Arc<dyn ValidationCache>>) -> Self {
        Self {
            jwt_service,
            cache,
            cache_ttl: Duration::from_secs(300), // 5 minutes
        }
    }

    /// Create a new JWT validator with custom cache TTL
    ///
    /// # Arguments
    /// * `jwt_service` - JWT service for token verification
    /// * `cache` - Optional cache implementation
    /// * `cache_ttl` - Custom TTL for cached validation results
    ///
    /// # Returns
    /// A new `JwtValidator` instance with custom TTL
    pub fn with_ttl(
        jwt_service: Arc<JwtService>,
        cache: Option<Arc<dyn ValidationCache>>,
        cache_ttl: Duration,
    ) -> Self {
        Self {
            jwt_service,
            cache,
            cache_ttl,
        }
    }

    /// Validate a JWT token with caching support
    ///
    /// This method implements a fast-path for cached validation results:
    /// 1. Check cache for previous validation result
    /// 2. If cache hit and not expired, return cached result (< 10ms)
    /// 3. If cache miss, perform full validation and cache result
    ///
    /// # Arguments
    /// * `token` - The JWT token string to validate
    ///
    /// # Returns
    /// A `Result` containing the validation result
    ///
    /// # Performance
    /// - Cache hit: < 10ms (target)
    /// - Cache miss: < 50ms (includes signature verification)
    pub async fn validate_token(&self, token: &str) -> Result<ValidationResult> {
        // Fast path: Check cache first
        if let Some(cache) = &self.cache {
            let cache_key = self.get_cache_key(token);

            // Try to get cached validation result
            match cache.get(&cache_key).await {
                Ok(Some(cached_value)) => {
                    if let Ok(result) = serde_json::from_value::<ValidationResult>(cached_value) {
                        // Verify cached result is still valid (not expired)
                        if self.is_cached_result_valid(&result) {
                            debug!("JWT validation cache hit for token");
                            return Ok(result);
                        } else {
                            debug!("Cached JWT validation result expired, re-validating");
                        }
                    }
                }
                Ok(None) => {
                    debug!("JWT validation cache miss");
                }
                Err(e) => {
                    warn!("Cache get failed during JWT validation: {}", e);
                    // Continue with validation even if cache fails
                }
            }
        }

        // Slow path: Perform full JWT validation
        let result = self.validate_token_full(token).await?;

        // Cache the validation result (non-blocking)
        if let Some(cache) = &self.cache {
            let cache_key = self.get_cache_key(token);
            let cache_value = serde_json::to_value(&result).unwrap_or_default();

            // Spawn cache write in background to avoid blocking
            let cache_clone = Arc::clone(cache);
            let cache_ttl = self.cache_ttl;
            tokio::spawn(async move {
                if let Err(e) = cache_clone.set(&cache_key, &cache_value, cache_ttl).await {
                    warn!("Failed to cache JWT validation result: {}", e);
                }
            });
        }

        Ok(result)
    }

    /// Perform full JWT validation without cache
    ///
    /// This method performs complete JWT validation including:
    /// 1. Signature verification
    /// 2. Expiration checking
    /// 3. Blacklist checking
    ///
    /// # Arguments
    /// * `token` - The JWT token string to validate
    ///
    /// # Returns
    /// A `Result` containing the validation result
    async fn validate_token_full(&self, token: &str) -> Result<ValidationResult> {
        // Check blacklist first (fast check)
        if self.is_token_blacklisted(token).await? {
            return Ok(ValidationResult {
                valid: false,
                user_id: None,
                expires_at: None,
                error: Some("Token has been revoked".to_string()),
                cached_at: chrono::Utc::now().timestamp(),
            });
        }

        // Verify JWT signature and claims
        match self.jwt_service.verify_token(token) {
            Ok(claims) => Ok(ValidationResult {
                valid: true,
                user_id: Some(claims.sub.clone()),
                expires_at: Some(claims.exp),
                error: None,
                cached_at: chrono::Utc::now().timestamp(),
            }),
            Err(e) => Ok(ValidationResult {
                valid: false,
                user_id: None,
                expires_at: None,
                error: Some(e.to_string()),
                cached_at: chrono::Utc::now().timestamp(),
            }),
        }
    }

    /// Check if a token is blacklisted
    ///
    /// Checks the cache for a blacklist entry for the given token.
    /// Blacklisted tokens are those that have been explicitly revoked.
    ///
    /// # Arguments
    /// * `token` - The JWT token string to check
    ///
    /// # Returns
    /// `true` if the token is blacklisted, `false` otherwise
    async fn is_token_blacklisted(&self, token: &str) -> Result<bool> {
        if let Some(cache) = &self.cache {
            let blacklist_key = self.get_blacklist_key(token);

            match cache.exists(&blacklist_key).await {
                Ok(exists) => Ok(exists),
                Err(e) => {
                    warn!("Failed to check token blacklist: {}", e);
                    // Fail open: if we can't check blacklist, allow the token
                    // (signature verification will still catch invalid tokens)
                    Ok(false)
                }
            }
        } else {
            // No cache available, can't check blacklist
            Ok(false)
        }
    }

    /// Check if a cached validation result is still valid
    ///
    /// Validates that:
    /// 1. The cached result hasn't expired based on token expiration
    /// 2. The cached result is recent enough (within cache TTL)
    ///
    /// # Arguments
    /// * `result` - The cached validation result to check
    ///
    /// # Returns
    /// `true` if the cached result is still valid, `false` otherwise
    fn is_cached_result_valid(&self, result: &ValidationResult) -> bool {
        if !result.valid {
            // Invalid results can be cached to prevent repeated validation attempts
            return true;
        }

        // Check if token has expired since caching
        if let Some(expires_at) = result.expires_at {
            let now = chrono::Utc::now().timestamp();

            if expires_at < now {
                debug!("Cached token has expired");
                return false;
            }
        }

        // Check if cached result is too old
        let now = chrono::Utc::now().timestamp();
        let age = now - result.cached_at;
        if age > self.cache_ttl.as_secs() as i64 {
            debug!("Cached validation result is too old");
            return false;
        }

        true
    }

    /// Revoke a token by adding it to the blacklist
    ///
    /// Adds the token to the blacklist cache and invalidates any cached
    /// validation results for this token.
    ///
    /// # Arguments
    /// * `token` - The JWT token string to revoke
    /// * `claims` - The token claims (for determining TTL)
    ///
    /// # Returns
    /// A `Result` indicating success or failure
    pub async fn revoke_token(&self, token: &str, claims: &TokenClaims) -> Result<()> {
        if let Some(cache) = &self.cache {
            // Add to blacklist with TTL equal to token expiration
            let blacklist_key = self.get_blacklist_key(token);
            let now = chrono::Utc::now().timestamp();

            let ttl = if claims.exp > now {
                Duration::from_secs((claims.exp - now) as u64)
            } else {
                Duration::from_secs(60) // Already expired, short TTL
            };

            cache
                .set(&blacklist_key, &serde_json::json!(true), ttl)
                .await?;

            // Invalidate cached validation result
            let cache_key = self.get_cache_key(token);
            cache.delete(&cache_key).await?;

            debug!("Token revoked and added to blacklist");
        }

        Ok(())
    }

    /// Invalidate cached validation result for a token
    ///
    /// Removes the cached validation result, forcing re-validation on next check.
    /// This is useful when token permissions change or other token metadata is updated.
    ///
    /// # Arguments
    /// * `token` - The JWT token string to invalidate
    ///
    /// # Returns
    /// A `Result` indicating success or failure
    pub async fn invalidate_cache(&self, token: &str) -> Result<()> {
        if let Some(cache) = &self.cache {
            let cache_key = self.get_cache_key(token);
            cache.delete(&cache_key).await?;
            debug!("JWT validation cache invalidated for token");
        }

        Ok(())
    }

    /// Get cache key for token validation result
    ///
    /// Generates a cache key based on the token hash to avoid storing
    /// the full token in the cache key.
    ///
    /// # Arguments
    /// * `token` - The JWT token string
    ///
    /// # Returns
    /// A cache key string
    fn get_cache_key(&self, token: &str) -> String {
        format!("jwt:validation:{}", hash_token(token))
    }

    /// Get blacklist cache key for a token
    ///
    /// Generates a blacklist cache key based on the token hash.
    ///
    /// # Arguments
    /// * `token` - The JWT token string
    ///
    /// # Returns
    /// A blacklist cache key string
    fn get_blacklist_key(&self, token: &str) -> String {
        format!("jwt:blacklist:{}", hash_token(token))
    }
}

/// Hash a token for use in cache keys
///
/// Uses SHA-256 to create a fixed-length hash of the token.
fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jwt::JwtService;
    use chrono::Duration as ChronoDuration;

    // Mock cache for testing
    struct MockCache {
        data: Arc<tokio::sync::RwLock<std::collections::HashMap<String, serde_json::Value>>>,
    }

    impl MockCache {
        fn new() -> Self {
            Self {
                data: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            }
        }
    }

    #[async_trait::async_trait]
    impl ValidationCache for MockCache {
        async fn get(&self, key: &str) -> Result<Option<serde_json::Value>> {
            Ok(self.data.read().await.get(key).cloned())
        }

        async fn set(&self, key: &str, value: &serde_json::Value, _ttl: Duration) -> Result<()> {
            self.data
                .write()
                .await
                .insert(key.to_string(), value.clone());
            Ok(())
        }

        async fn exists(&self, key: &str) -> Result<bool> {
            Ok(self.data.read().await.contains_key(key))
        }

        async fn delete(&self, key: &str) -> Result<()> {
            self.data.write().await.remove(key);
            Ok(())
        }
    }

    fn create_test_jwt_service() -> Arc<JwtService> {
        let key_bytes = JwtService::generate_signing_key();
        Arc::new(
            JwtService::new(
                &key_bytes,
                "https://test.example.com".to_string(),
                ChronoDuration::minutes(15),
                ChronoDuration::days(7),
            )
            .unwrap(),
        )
    }

    #[tokio::test]
    async fn test_jwt_validation_without_cache() {
        let jwt_service = create_test_jwt_service();
        let validator = JwtValidator::new(jwt_service.clone(), None);

        // Generate a valid token
        let token = jwt_service
            .generate_access_token("test_user", None, None, None)
            .unwrap();

        // Validate token
        let result = validator.validate_token(&token).await.unwrap();

        assert!(result.valid);
        assert_eq!(result.user_id, Some("test_user".to_string()));
        assert!(result.expires_at.is_some());
        assert!(result.error.is_none());
    }

    #[tokio::test]
    async fn test_jwt_validation_with_cache() {
        let jwt_service = create_test_jwt_service();
        let cache = Arc::new(MockCache::new());
        let validator = JwtValidator::new(jwt_service.clone(), Some(cache.clone()));

        // Generate a valid token
        let token = jwt_service
            .generate_access_token("test_user", None, None, None)
            .unwrap();

        // First validation (cache miss)
        let result1 = validator.validate_token(&token).await.unwrap();
        assert!(result1.valid);

        // Wait for cache write to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Second validation (cache hit)
        let result2 = validator.validate_token(&token).await.unwrap();
        assert!(result2.valid);
        assert_eq!(result1.user_id, result2.user_id);
    }

    #[tokio::test]
    async fn test_jwt_validation_with_invalid_token() {
        let jwt_service = create_test_jwt_service();
        let validator = JwtValidator::new(jwt_service, None);

        // Invalid token
        let token = "invalid.jwt.token";

        // Validate token
        let result = validator.validate_token(token).await.unwrap();

        assert!(!result.valid);
        assert!(result.user_id.is_none());
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn test_token_revocation() {
        let jwt_service = create_test_jwt_service();
        let cache = Arc::new(MockCache::new());
        let validator = JwtValidator::new(jwt_service.clone(), Some(cache.clone()));

        // Generate a valid token
        let token = jwt_service
            .generate_access_token("test_user", None, None, None)
            .unwrap();

        // Validate token (should be valid)
        let result1 = validator.validate_token(&token).await.unwrap();
        assert!(result1.valid);

        // Revoke token
        let claims = jwt_service.verify_token(&token).unwrap();
        validator.revoke_token(&token, &claims).await.unwrap();

        // Validate token again (should be invalid)
        let result2 = validator.validate_token(&token).await.unwrap();
        assert!(!result2.valid);
        assert_eq!(result2.error, Some("Token has been revoked".to_string()));
    }

    #[test]
    fn test_hash_token() {
        let token1 = "test.jwt.token";
        let token2 = "test.jwt.token";
        let token3 = "different.jwt.token";

        let hash1 = hash_token(token1);
        let hash2 = hash_token(token2);
        let hash3 = hash_token(token3);

        // Same token should generate same hash
        assert_eq!(hash1, hash2);

        // Different tokens should generate different hashes
        assert_ne!(hash1, hash3);
    }
}
