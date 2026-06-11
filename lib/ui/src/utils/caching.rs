//! Caching utilities for performance optimization
//!
//! Provides utilities for:
//! - localStorage caching with TTL
//! - API response caching
//! - Cache invalidation strategies
//! - Memory-based caching

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Cached item with expiration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CachedItem<T> {
    pub data: T,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl<T> CachedItem<T> {
    pub fn new(data: T, ttl_seconds: i64) -> Self {
        let now = Utc::now();
        Self {
            data,
            expires_at: now + Duration::seconds(ttl_seconds),
            created_at: now,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn is_valid(&self) -> bool {
        !self.is_expired()
    }
}

/// LocalStorage cache with TTL support
pub struct LocalStorageCache;

impl LocalStorageCache {
    /// Set item in cache with TTL
    #[cfg(target_arch = "wasm32")]
    pub fn set<T: Serialize>(key: &str, value: T, ttl_seconds: i64) -> Result<(), String> {
        use gloo_storage::{LocalStorage, Storage};

        let cached_item = CachedItem {
            data: value,
            expires_at: chrono::Utc::now() + chrono::Duration::seconds(ttl_seconds),
            created_at: chrono::Utc::now(),
        };
        LocalStorage::set(key, cached_item).map_err(|e| e.to_string())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn set<T: Serialize>(_key: &str, _value: T, _ttl_seconds: i64) -> Result<(), String> {
        Ok(())
    }

    /// Get item from cache if not expired
    #[cfg(target_arch = "wasm32")]
    pub fn get<T: for<'de> Deserialize<'de>>(key: &str) -> Option<T> {
        use gloo_storage::{LocalStorage, Storage};

        let cached_item: CachedItem<T> = LocalStorage::get(key).ok()?;

        if cached_item.is_valid() {
            Some(cached_item.data)
        } else {
            // Remove expired item
            Self::remove(key);
            None
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn get<T: for<'de> Deserialize<'de>>(_key: &str) -> Option<T> {
        None
    }

    /// Remove item from cache
    #[cfg(target_arch = "wasm32")]
    pub fn remove(key: &str) {
        use gloo_storage::{LocalStorage, Storage};
        LocalStorage::delete(key);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn remove(_key: &str) {}

    /// Clear all cached items
    #[cfg(target_arch = "wasm32")]
    pub fn clear() {
        use gloo_storage::{LocalStorage, Storage};
        LocalStorage::clear();
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn clear() {}

    /// Clear expired items
    #[cfg(target_arch = "wasm32")]
    pub fn clear_expired() {
        use gloo_storage::{LocalStorage, Storage};

        if let Some(window) = web_sys::window()
            && let Ok(Some(storage)) = window.local_storage()
        {
            let mut keys_to_remove = Vec::new();

            // Iterate through all keys
            for i in 0..storage.length().unwrap_or(0) {
                if let Ok(Some(key)) = storage.key(i)
                    && let Ok(Some(value)) = storage.get_item(&key)
                {
                    // Try to parse as CachedItem
                    if let Ok(cached_item) = serde_json::from_str::<CachedItem<String>>(&value)
                        && cached_item.is_expired()
                    {
                        keys_to_remove.push(key);
                    }
                }
            }

            // Remove expired keys
            for key in keys_to_remove {
                LocalStorage::delete(&key);
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn clear_expired() {}
}

/// In-memory cache for runtime data
pub struct MemoryCache<T> {
    cache: Arc<Mutex<HashMap<String, CachedItem<T>>>>,
}

impl<T: Clone> MemoryCache<T> {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Set item in cache with TTL
    pub fn set(&self, key: String, value: T, ttl_seconds: i64) {
        let cached_item = CachedItem::new(value, ttl_seconds);
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(key, cached_item);
        }
    }

    /// Get item from cache if not expired
    pub fn get(&self, key: &str) -> Option<T> {
        if let Ok(mut cache) = self.cache.lock()
            && let Some(cached_item) = cache.get(key)
        {
            if cached_item.is_valid() {
                return Some(cached_item.data.clone());
            } else {
                // Remove expired item
                cache.remove(key);
            }
        }
        None
    }

    /// Remove item from cache
    pub fn remove(&self, key: &str) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.remove(key);
        }
    }

    /// Clear all cached items
    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }
    }

    /// Clear expired items
    pub fn clear_expired(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.retain(|_, item| item.is_valid());
        }
    }

    /// Get cache size
    pub fn size(&self) -> usize {
        if let Ok(cache) = self.cache.lock() {
            cache.len()
        } else {
            0
        }
    }
}

impl<T: Clone> Default for MemoryCache<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache key builder for consistent cache keys
pub struct CacheKeyBuilder {
    parts: Vec<String>,
}

impl CacheKeyBuilder {
    pub fn new(prefix: &str) -> Self {
        Self {
            parts: vec![prefix.to_string()],
        }
    }

    pub fn with_part(mut self, part: &str) -> Self {
        self.parts.push(part.to_string());
        self
    }

    pub fn add_param(mut self, key: &str, value: &str) -> Self {
        self.parts.push(format!("{}={}", key, value));
        self
    }

    pub fn build(self) -> String {
        self.parts.join(":")
    }
}

/// Cache invalidation strategies
pub enum CacheInvalidationStrategy {
    /// Time-based expiration (TTL)
    TimeToLive(i64),
    /// Invalidate on specific events
    EventBased(Vec<String>),
    /// Least Recently Used (LRU)
    LeastRecentlyUsed { max_size: usize },
    /// Manual invalidation only
    Manual,
}

/// API response cache configuration
#[derive(Clone, Debug)]
pub struct ApiCacheConfig {
    /// Enable caching
    pub enabled: bool,
    /// Default TTL in seconds
    pub default_ttl: i64,
    /// Maximum cache size (number of items)
    pub max_size: usize,
    /// Cache only GET requests
    pub cache_get_only: bool,
}

impl Default for ApiCacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_ttl: 300, // 5 minutes
            max_size: 100,
            cache_get_only: true,
        }
    }
}

/// API response cache
pub struct ApiCache {
    cache: MemoryCache<String>,
    config: ApiCacheConfig,
}

impl ApiCache {
    pub fn new(config: ApiCacheConfig) -> Self {
        Self {
            cache: MemoryCache::new(),
            config,
        }
    }

    /// Generate cache key for API request
    pub fn generate_key(method: &str, url: &str, params: Option<&str>) -> String {
        let mut builder = CacheKeyBuilder::new("api");
        builder = builder.with_part(method).with_part(url);

        if let Some(params) = params {
            builder = builder.with_part(params);
        }

        builder.build()
    }

    /// Cache API response
    pub fn set(&self, method: &str, url: &str, params: Option<&str>, response: String) {
        if !self.config.enabled {
            return;
        }

        if self.config.cache_get_only && method != "GET" {
            return;
        }

        let key = Self::generate_key(method, url, params);
        self.cache.set(key, response, self.config.default_ttl);

        // Check cache size and evict if necessary
        if self.cache.size() > self.config.max_size {
            self.cache.clear_expired();
        }
    }

    /// Get cached API response
    pub fn get(&self, method: &str, url: &str, params: Option<&str>) -> Option<String> {
        if !self.config.enabled {
            return None;
        }

        let key = Self::generate_key(method, url, params);
        self.cache.get(&key)
    }

    /// Invalidate cache for specific URL pattern
    pub fn invalidate_pattern(&self, _pattern: &str) {
        // This is a simplified implementation
        // In production, you'd want to iterate through keys and match patterns
        self.cache.clear();
    }

    /// Clear all cached responses
    pub fn clear(&self) {
        self.cache.clear();
    }
}

impl Default for ApiCache {
    fn default() -> Self {
        Self::new(ApiCacheConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cached_item_expiration() {
        let item = CachedItem::new("test data", 1);
        assert!(item.is_valid());
        assert!(!item.is_expired());

        // Test with expired item
        let expired_item = CachedItem {
            data: "test",
            expires_at: Utc::now() - Duration::seconds(10),
            created_at: Utc::now() - Duration::seconds(20),
        };
        assert!(!expired_item.is_valid());
        assert!(expired_item.is_expired());
    }

    #[test]
    fn test_memory_cache() {
        let cache = MemoryCache::new();

        // Set and get
        cache.set("key1".to_string(), "value1".to_string(), 60);
        assert_eq!(cache.get("key1"), Some("value1".to_string()));

        // Non-existent key
        assert_eq!(cache.get("key2"), None);

        // Remove
        cache.remove("key1");
        assert_eq!(cache.get("key1"), None);

        // Size
        cache.set("key1".to_string(), "value1".to_string(), 60);
        cache.set("key2".to_string(), "value2".to_string(), 60);
        assert_eq!(cache.size(), 2);

        // Clear
        cache.clear();
        assert_eq!(cache.size(), 0);
    }

    #[test]
    fn test_cache_key_builder() {
        let key = CacheKeyBuilder::new("api")
            .with_part("GET")
            .with_part("/users")
            .add_param("page", "1")
            .add_param("limit", "10")
            .build();

        assert_eq!(key, "api:GET:/users:page=1:limit=10");
    }

    #[test]
    fn test_api_cache() {
        let cache = ApiCache::default();

        // Cache GET request
        cache.set("GET", "/api/users", None, "response data".to_string());
        assert_eq!(
            cache.get("GET", "/api/users", None),
            Some("response data".to_string())
        );

        // POST request should not be cached (with default config)
        cache.set("POST", "/api/users", None, "post response".to_string());
        assert_eq!(cache.get("POST", "/api/users", None), None);
    }

    #[test]
    fn test_api_cache_with_params() {
        let cache = ApiCache::default();

        cache.set(
            "GET",
            "/api/users",
            Some("page=1"),
            "page 1 data".to_string(),
        );
        cache.set(
            "GET",
            "/api/users",
            Some("page=2"),
            "page 2 data".to_string(),
        );

        assert_eq!(
            cache.get("GET", "/api/users", Some("page=1")),
            Some("page 1 data".to_string())
        );
        assert_eq!(
            cache.get("GET", "/api/users", Some("page=2")),
            Some("page 2 data".to_string())
        );
    }
}
