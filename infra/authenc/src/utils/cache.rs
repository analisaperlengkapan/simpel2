use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::RwLock as AsyncRwLock;

use crate::error::{AuthencError, Result};

/// Cache entry with TTL support
#[derive(Debug, Clone)]
struct CacheEntry<V> {
    value: V,
    created_at: Instant,
    ttl: Duration,
    access_count: u64,
    last_accessed: Instant,
}

impl<V> CacheEntry<V> {
    fn new(value: V, ttl: Duration) -> Self {
        let now = Instant::now();
        Self {
            value,
            created_at: now,
            ttl,
            access_count: 1,
            last_accessed: now,
        }
    }

    fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }

    fn access(&mut self) -> &V {
        self.access_count += 1;
        self.last_accessed = Instant::now();
        &self.value
    }
}

/// LRU Cache with TTL support for authentication data
pub struct LruCache<K, V> {
    data: HashMap<K, CacheEntry<V>>,
    capacity: usize,
    access_order: Vec<K>,
    hit_count: u64,
    miss_count: u64,
}

impl<K, V> LruCache<K, V>
where
    K: Clone + Eq + Hash,
    V: Clone,
{
    /// Create a new LRU cache with specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            data: HashMap::with_capacity(capacity),
            capacity,
            access_order: Vec::with_capacity(capacity),
            hit_count: 0,
            miss_count: 0,
        }
    }

    /// Insert a value with TTL
    pub fn insert(&mut self, key: K, value: V, ttl: Duration) {
        // Remove expired entries first
        self.cleanup_expired();

        // If at capacity, remove LRU item
        if self.data.len() >= self.capacity && !self.data.contains_key(&key) {
            self.evict_lru();
        }

        // Update access order
        if let Some(pos) = self.access_order.iter().position(|k| k == &key) {
            self.access_order.remove(pos);
        }
        self.access_order.push(key.clone());

        // Insert the entry
        self.data.insert(key, CacheEntry::new(value, ttl));
    }

    /// Get a value from cache
    pub fn get(&mut self, key: &K) -> Option<V> {
        // Clean up expired entries
        self.cleanup_expired();

        if let Some(entry) = self.data.get_mut(key) {
            if entry.is_expired() {
                self.data.remove(key);
                if let Some(pos) = self.access_order.iter().position(|k| k == key) {
                    self.access_order.remove(pos);
                }
                self.miss_count += 1;
                return None;
            }

            // Update access order
            if let Some(pos) = self.access_order.iter().position(|k| k == key) {
                self.access_order.remove(pos);
                self.access_order.push(key.clone());
            }

            self.hit_count += 1;
            Some(entry.access().clone())
        } else {
            self.miss_count += 1;
            None
        }
    }

    /// Remove a key from cache
    pub fn remove(&mut self, key: &K) -> Option<V> {
        if let Some(entry) = self.data.remove(key) {
            if let Some(pos) = self.access_order.iter().position(|k| k == key) {
                self.access_order.remove(pos);
            }
            Some(entry.value)
        } else {
            None
        }
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.data.clear();
        self.access_order.clear();
        self.hit_count = 0;
        self.miss_count = 0;
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let total_requests = self.hit_count + self.miss_count;
        let hit_ratio = if total_requests > 0 {
            self.hit_count as f64 / total_requests as f64
        } else {
            0.0
        };

        CacheStats {
            size: self.data.len(),
            capacity: self.capacity,
            hit_count: self.hit_count,
            miss_count: self.miss_count,
            hit_ratio,
        }
    }

    /// Cleanup expired entries
    fn cleanup_expired(&mut self) {
        let expired_keys: Vec<K> = self
            .data
            .iter()
            .filter(|(_, entry)| entry.is_expired())
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_keys {
            self.data.remove(&key);
            if let Some(pos) = self.access_order.iter().position(|k| k == &key) {
                self.access_order.remove(pos);
            }
        }
    }

    /// Evict least recently used item
    fn evict_lru(&mut self) {
        if let Some(lru_key) = self.access_order.first().cloned() {
            self.data.remove(&lru_key);
            self.access_order.remove(0);
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Current number of items in cache
    pub size: usize,
    /// Maximum capacity of cache
    pub capacity: usize,
    /// Number of cache hits
    pub hit_count: u64,
    /// Number of cache misses
    pub miss_count: u64,
    /// Cache hit ratio (hits / (hits + misses))
    pub hit_ratio: f64,
}

/// Thread-safe LRU cache wrapper
pub struct ThreadSafeLruCache<K, V> {
    /// Underlying LRU cache protected by RwLock
    cache: Arc<RwLock<LruCache<K, V>>>,
}

impl<K, V> ThreadSafeLruCache<K, V>
where
    K: Clone + Eq + Hash,
    V: Clone,
{
    /// Create a new thread-safe LRU cache
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(LruCache::new(capacity))),
        }
    }

    /// Insert a value with TTL
    pub fn insert(&self, key: K, value: V, ttl: Duration) -> Result<()> {
        let mut cache = self.cache
            .write()
            .map_err(|_| AuthencError::internal("Failed to acquire cache write lock"))?;
        cache.insert(key, value, ttl);
        Ok(())
    }

    /// Get a value from cache
    pub fn get(&self, key: &K) -> Result<Option<V>> {
        let mut cache = self.cache
            .write()
            .map_err(|_| AuthencError::internal("Failed to acquire cache write lock"))?;
        Ok(cache.get(key))
    }

    /// Remove a key from cache
    pub fn remove(&self, key: &K) -> Result<Option<V>> {
        let mut cache = self.cache
            .write()
            .map_err(|_| AuthencError::internal("Failed to acquire cache write lock"))?;
        Ok(cache.remove(key))
    }

    /// Clear all entries
    pub fn clear(&self) -> Result<()> {
        let mut cache = self.cache
            .write()
            .map_err(|_| AuthencError::internal("Failed to acquire cache write lock"))?;
        cache.clear();
        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> Result<CacheStats> {
        let cache = self.cache
            .read()
            .map_err(|_| AuthencError::internal("Failed to acquire cache read lock"))?;
        Ok(cache.stats())
    }
}

impl<K, V> Clone for ThreadSafeLruCache<K, V> {
    fn clone(&self) -> Self {
        Self {
            cache: Arc::clone(&self.cache),
        }
    }
}

/// Async LRU cache for use in async contexts
pub struct AsyncLruCache<K, V> {
    cache: Arc<AsyncRwLock<LruCache<K, V>>>,
}

impl<K, V> AsyncLruCache<K, V>
where
    K: Clone + Eq + Hash,
    V: Clone,
{
    /// Create a new async LRU cache
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Arc::new(AsyncRwLock::new(LruCache::new(capacity))),
        }
    }

    /// Insert a value with TTL
    pub async fn insert(&self, key: K, value: V, ttl: Duration) {
        let mut cache = self.cache.write().await;
        cache.insert(key, value, ttl);
    }

    /// Get a value from cache
    pub async fn get(&self, key: &K) -> Option<V> {
        let mut cache = self.cache.write().await;
        cache.get(key)
    }

    /// Remove a key from cache
    pub async fn remove(&self, key: &K) -> Option<V> {
        let mut cache = self.cache.write().await;
        cache.remove(key)
    }

    /// Clear all entries
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        cache.stats()
    }
}

impl<K, V> Clone for AsyncLruCache<K, V> {
    fn clone(&self) -> Self {
        Self {
            cache: Arc::clone(&self.cache),
        }
    }
}

/// Specialized cache for JWT tokens
pub type TokenCache = AsyncLruCache<String, String>;

/// Specialized cache for user sessions
pub type SessionCache = AsyncLruCache<String, serde_json::Value>;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    #[test]
    fn test_lru_cache_basic_operations() {
        let mut cache = LruCache::new(3);

        cache.insert("key1", "value1", Duration::from_secs(60));
        cache.insert("key2", "value2", Duration::from_secs(60));
        cache.insert("key3", "value3", Duration::from_secs(60));

        assert_eq!(cache.get(&"key1"), Some("value1".to_string()).as_deref());
        assert_eq!(cache.get(&"key2"), Some("value2".to_string()).as_deref());
        assert_eq!(cache.get(&"key3"), Some("value3".to_string()).as_deref());

        // Insert one more to trigger LRU eviction
        cache.insert("key4", "value4", Duration::from_secs(60));

        // key1 should be evicted (least recently used)
        assert_eq!(cache.get(&"key1"), None);
        assert_eq!(cache.get(&"key4"), Some("value4".to_string()).as_deref());
    }

    #[test]
    fn test_lru_cache_ttl() {
        let mut cache = LruCache::new(3);

        cache.insert("key1", "value1", Duration::from_millis(100));
        assert_eq!(cache.get(&"key1"), Some("value1".to_string()).as_deref());

        std::thread::sleep(Duration::from_millis(150));
        assert_eq!(cache.get(&"key1"), None);
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = LruCache::new(3);

        cache.insert("key1", "value1", Duration::from_secs(60));
        cache.get(&"key1"); // hit
        cache.get(&"key2"); // miss

        let stats = cache.stats();
        assert_eq!(stats.hit_count, 1);
        assert_eq!(stats.miss_count, 1);
        assert_eq!(stats.hit_ratio, 0.5);
    }

    #[tokio::test]
    async fn test_async_cache() {
        let cache = AsyncLruCache::new(3);

        cache.insert("key1".to_string(), "value1".to_string(), Duration::from_secs(60)).await;
        cache.insert("key2".to_string(), "value2".to_string(), Duration::from_secs(60)).await;

        assert_eq!(cache.get(&"key1".to_string()).await, Some("value1".to_string()));
        assert_eq!(cache.get(&"key2".to_string()).await, Some("value2".to_string()));
        assert_eq!(cache.get(&"key3".to_string()).await, None);
    }

    #[tokio::test]
    async fn test_cache_manager() {
        let manager = CacheManager::new();

        manager.token_cache().insert("token1".to_string(), "jwt_token".to_string(), Duration::from_secs(60)).await;
        manager.validation_cache().insert("validation1".to_string(), true, Duration::from_secs(60)).await;

        assert_eq!(manager.token_cache().get(&"token1".to_string()).await, Some("jwt_token".to_string()));
        assert_eq!(manager.validation_cache().get(&"validation1".to_string()).await, Some(true));

        let stats = manager.get_stats().await;
        assert_eq!(stats.token_cache.size, 1);
        assert_eq!(stats.validation_cache.size, 1);
    }
}
