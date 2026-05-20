//! Cache utilities
//!
//! This module re-exports cache types from the common library and provides
//! a secret-specific cache manager.

use crate::CoreError;
use crate::storage::CacheBackend;
use lib_backend::cache::{CacheStats, RedisCache};

// Re-export common cache types
pub use lib_backend::cache::{
    AsyncLruCache as AsyncSecretCache, LruCache as SecretLruCache, SensitivityLevel,
    ThreadSafeLruCache as ThreadSafeSecretCache,
};

/// Specialized cache types for different secret operations
pub type SecretValueCache = AsyncSecretCache<String, Vec<u8>>;
pub type TokenValidationCache = AsyncSecretCache<String, bool>;
pub type UserPermissionCache = AsyncSecretCache<String, Vec<String>>;
pub type EncryptionKeyCache = AsyncSecretCache<String, Vec<u8>>;

/// Secret cache stats (alias for CacheStats)
pub type SecretCacheStats = CacheStats;

/// Secret cache manager for coordinating multiple caches
pub struct SecretCacheManager {
    secret_cache: SecretValueCache,
    token_cache: TokenValidationCache,
    permission_cache: UserPermissionCache,
    key_cache: EncryptionKeyCache,
    #[allow(dead_code)]
    backend: Option<Box<dyn CacheBackend + Send + Sync>>, // Kept for reference, though logic is now in types
    redis_cache: Option<RedisCache>,
}

impl SecretCacheManager {
    /// Create a new secret cache manager with default capacities (memory-only)
    pub fn new() -> Self {
        Self {
            secret_cache: SecretValueCache::new(20000),
            token_cache: TokenValidationCache::new(50000),
            permission_cache: UserPermissionCache::new(10000),
            key_cache: EncryptionKeyCache::new(5000),
            backend: None,
            redis_cache: None,
        }
    }

    /// Create a new cache manager with custom capacities
    pub fn with_capacities(
        secret_capacity: usize,
        token_capacity: usize,
        permission_capacity: usize,
        key_capacity: usize,
    ) -> Self {
        Self {
            secret_cache: SecretValueCache::new(secret_capacity),
            token_cache: TokenValidationCache::new(token_capacity),
            permission_cache: UserPermissionCache::new(permission_capacity),
            key_cache: EncryptionKeyCache::new(key_capacity),
            backend: None,
            redis_cache: None,
        }
    }

    /// Create a new cache manager with Redis backend
    #[cfg(feature = "redis")]
    pub fn with_redis(
        secret_capacity: usize,
        token_capacity: usize,
        permission_capacity: usize,
        key_capacity: usize,
        redis_url: &str,
    ) -> Result<Self, CoreError> {
        let redis_cache = RedisCache::new(redis_url, "secreton")
            .map_err(|e| CoreError::internal(e.to_string()))?;

        Ok(Self {
            secret_cache: SecretValueCache::new(secret_capacity),
            token_cache: TokenValidationCache::new(token_capacity),
            permission_cache: UserPermissionCache::new(permission_capacity),
            key_cache: EncryptionKeyCache::new(key_capacity),
            backend: None, // Simplified
            redis_cache: Some(redis_cache),
        })
    }

    /// Get secret value cache
    pub fn secret_cache(&self) -> &SecretValueCache {
        &self.secret_cache
    }

    /// Get token validation cache
    pub fn token_cache(&self) -> &TokenValidationCache {
        &self.token_cache
    }

    /// Get user permission cache
    pub fn permission_cache(&self) -> &UserPermissionCache {
        &self.permission_cache
    }

    /// Get encryption key cache
    pub fn key_cache(&self) -> &EncryptionKeyCache {
        &self.key_cache
    }

    /// Clear all caches
    pub async fn clear_all(&self) {
        self.secret_cache.clear().await;
        self.token_cache.clear().await;
        self.permission_cache.clear().await;
        self.key_cache.clear().await;

        if let Some(redis) = &self.redis_cache {
            let _ = redis.clear_all().await;
        }
    }

    /// Evict sensitive entries based on load
    pub async fn evict_sensitive(&self, load_factor: f64) {
        self.secret_cache.evict_by_sensitivity(load_factor).await;
        self.token_cache.evict_by_sensitivity(load_factor).await;
        self.permission_cache
            .evict_by_sensitivity(load_factor)
            .await;
        self.key_cache.evict_by_sensitivity(load_factor).await;
    }

    /// Get combined cache statistics
    pub async fn get_stats(&self) -> SecretCacheManagerStats {
        let secret_stats = self.secret_cache.stats().await;
        let token_stats = self.token_cache.stats().await;
        let permission_stats = self.permission_cache.stats().await;
        let key_stats = self.key_cache.stats().await;

        SecretCacheManagerStats {
            secret_cache: secret_stats,
            token_cache: token_stats,
            permission_cache: permission_stats,
            key_cache: key_stats,
        }
    }
}

impl Default for SecretCacheManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Combined cache statistics
#[derive(Debug, Clone)]
pub struct SecretCacheManagerStats {
    pub secret_cache: SecretCacheStats,
    pub token_cache: SecretCacheStats,
    pub permission_cache: SecretCacheStats,
    pub key_cache: SecretCacheStats,
}
