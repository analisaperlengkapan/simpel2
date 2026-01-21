//! Cache utilities
//!
//! This module re-exports cache types from the common library and provides
//! authentication-specific cache aliases and manager.

use lib_common::cache::{AsyncLruCache, CacheStats};
use serde_json::Value;

// Re-export common cache types
pub use lib_common::cache::{
    LruCache, ThreadSafeLruCache,
};

/// Specialized cache for JWT tokens
pub type TokenCache = AsyncLruCache<String, String>;

/// Specialized cache for user sessions
pub type SessionCache = AsyncLruCache<String, Value>;

/// Specialized cache for validation results
pub type ValidationCache = AsyncLruCache<String, bool>;

/// Cache manager for coordinating multiple caches
pub struct CacheManager {
    token_cache: TokenCache,
    session_cache: SessionCache,
    validation_cache: ValidationCache,
}

impl CacheManager {
    /// Create a new cache manager with default capacities
    pub fn new() -> Self {
        Self {
            token_cache: TokenCache::new(10000),
            session_cache: SessionCache::new(5000),
            validation_cache: ValidationCache::new(20000),
        }
    }

    /// Create a new cache manager with custom capacities
    pub fn with_capacities(
        token_capacity: usize,
        session_capacity: usize,
        validation_capacity: usize,
    ) -> Self {
        Self {
            token_cache: TokenCache::new(token_capacity),
            session_cache: SessionCache::new(session_capacity),
            validation_cache: ValidationCache::new(validation_capacity),
        }
    }

    /// Get token cache
    pub fn token_cache(&self) -> &TokenCache {
        &self.token_cache
    }

    /// Get session cache
    pub fn session_cache(&self) -> &SessionCache {
        &self.session_cache
    }

    /// Get validation cache
    pub fn validation_cache(&self) -> &ValidationCache {
        &self.validation_cache
    }

    /// Clear all caches
    pub async fn clear_all(&self) {
        self.token_cache.clear().await;
        self.session_cache.clear().await;
        self.validation_cache.clear().await;
    }

    /// Get combined cache statistics
    pub async fn get_stats(&self) -> CacheManagerStats {
        let token_stats = self.token_cache.stats().await;
        let session_stats = self.session_cache.stats().await;
        let validation_stats = self.validation_cache.stats().await;

        CacheManagerStats {
            token_cache: token_stats,
            session_cache: session_stats,
            validation_cache: validation_stats,
        }
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Combined cache statistics
#[derive(Debug, Clone)]
pub struct CacheManagerStats {
    /// Statistics for token cache
    pub token_cache: CacheStats,
    /// Statistics for session cache
    pub session_cache: CacheStats,
    /// Statistics for validation cache
    pub validation_cache: CacheStats,
}
