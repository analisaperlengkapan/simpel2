//! In-memory cache implementation (L1 cache) - requires `cache` feature
//!
//! This module provides a fast in-memory cache using LruCache with TTL support
//! and O(1) LRU eviction for the L1 cache layer in the multi-layer caching strategy.
#![cfg(feature = "cache")]

use super::{Cache, CacheMetrics};
use async_trait::async_trait;
use authenc_types::Result;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// Entry in the in-memory cache with TTL
#[derive(Debug, Clone)]
struct CacheEntry {
    /// The cached value
    value: serde_json::Value,
    /// When this entry expires
    expires_at: Instant,
    /// When this entry was last accessed (for LRU)
    // Note: LruCache maintains access order internally, but we might keep this for debugging or if we switch back
    last_accessed: Instant,
}

impl CacheEntry {
    /// Create a new cache entry
    fn new(value: serde_json::Value, ttl: Duration) -> Self {
        let now = Instant::now();
        Self {
            value,
            expires_at: now + ttl,
            last_accessed: now,
        }
    }

    /// Check if this entry has expired
    fn is_expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }

    /// Update last accessed time
    fn touch(&mut self) {
        self.last_accessed = Instant::now();
    }
}

/// In-memory cache implementation using LruCache
pub struct InMemoryCache {
    /// The cache storage
    cache: Arc<Mutex<LruCache<String, CacheEntry>>>,
    /// Maximum number of entries
    max_size: usize,
    /// Default TTL for entries
    default_ttl: Duration,
    /// Metrics collector
    metrics: Arc<CacheMetrics>,
}

impl InMemoryCache {
    /// Create a new in-memory cache
    ///
    /// # Arguments
    /// * `max_size` - Maximum number of entries (default: 10,000)
    /// * `default_ttl` - Default TTL for entries (default: 60 seconds)
    pub fn new(max_size: usize, default_ttl: Duration) -> Self {
        debug!(
            "Creating in-memory cache with max_size={}, default_ttl={:?}",
            max_size, default_ttl
        );

        let capacity = NonZeroUsize::new(max_size).unwrap_or(NonZeroUsize::new(10000).unwrap());

        Self {
            cache: Arc::new(Mutex::new(LruCache::new(capacity))),
            max_size,
            default_ttl,
            metrics: Arc::new(CacheMetrics::new()),
        }
    }

    /// Evict expired entries
    fn evict_expired(&self) {
        // We collect keys first to avoid holding the lock while iterating if we were to do complex logic,
        // but LruCache iter returns references so we can't mutate anyway.
        // We need to find expired keys and then remove them.

        // This operation is O(N) but it is only called periodically.

        let mut cache = match self.cache.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Cache mutex poisoned, recovering");
                poisoned.into_inner()
            }
        };

        let keys_to_remove: Vec<String> = cache
            .iter()
            .filter(|(_, entry)| entry.is_expired())
            .map(|(k, _)| k.clone())
            .collect();

        let evicted = keys_to_remove.len();
        if evicted > 0 {
            for key in keys_to_remove {
                cache.pop(&key);
                self.metrics.record_eviction();
                self.metrics.decrement_cache_size();
            }
            debug!("Evicted {} expired entries from L1 cache", evicted);
        }
    }

    /// Get cache metrics
    pub fn metrics(&self) -> Arc<CacheMetrics> {
        Arc::clone(&self.metrics)
    }

    /// Get current cache size
    pub fn size(&self) -> usize {
        let cache = self.cache.lock().unwrap();
        cache.len()
    }

    /// Clear all entries from the cache
    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        let size = cache.len();
        cache.clear();
        self.metrics.update_cache_size(0);
        debug!("Cleared {} entries from L1 cache", size);
    }
}

#[async_trait]
impl Cache for InMemoryCache {
    async fn get(&self, key: &str) -> Result<Option<serde_json::Value>> {
        let start = Instant::now();

        // Evict expired entries periodically (every 100 operations approx)
        // Note: we check size first to avoid locking if empty?
        // We need random sampling or just a counter.
        // Since we are inside `get`, we can't easily maintain a counter without mutability.
        // The original code used `self.cache.len() % 100 == 0`.
        // We can do checking cache size with lock.
        {
            // Acquire lock just to check len? Or do we skip this?
            // Checking expired every time is expensive.
            // But doing it every 100 times requires shared state counter or random.
            // Using random is good.
            if rand::random::<u8>() % 100 == 0 {
                self.evict_expired();
            }
        }

        let mut cache = self.cache.lock().unwrap();

        // get() updates LRU
        let result = if let Some(entry) = cache.get(key) {
            if entry.is_expired() {
                // Remove expired entry
                // We must drop the reference `entry` before borrowing `cache` mutably again for pop
                // But `get` returns `Option<&mut V>`? No `lru` `get` returns `Option<&V>`.
                // Actually `lru` `get` requires `&mut self`.
                // So `entry` borrows `cache` mutably.
                // We cannot call `pop` while `entry` is alive.
                // So we check expiration, verify result, then act.
                true
            } else {
                false
            }
        } else {
            false
        };

        // If we found it was expired, we pop it.
        // If it was valid, we need to retrieve it again?
        // Or we could have cloned it?

        // Efficient way:
        let value = if result {
            // It was expired
            cache.pop(key);
            self.metrics.decrement_cache_size(); // Explicitly decrement as it was removed
            debug!("L1 cache miss (expired) for key: {}", key);
            self.metrics.record_miss();
            None
        } else {
            // Check if exists (valid)
            // We need to get it again to clone value. This updates LRU again (harmless).
            // Or we could have cloned inside the first block if we handled scopes correctly.
            if let Some(entry) = cache.get(key) {
                // entry.touch(); // LruCache updates access time automatically
                debug!("L1 cache hit for key: {}", key);
                self.metrics.record_hit();
                Some(entry.value.clone())
            } else {
                debug!("L1 cache miss for key: {}", key);
                self.metrics.record_miss();
                None
            }
        };

        let elapsed = start.elapsed();
        self.metrics.record_get(elapsed);

        Ok(value)
    }

    async fn set(&self, key: &str, value: &serde_json::Value, ttl: Duration) -> Result<()> {
        let start = Instant::now();

        let entry = CacheEntry::new(value.clone(), ttl);

        let mut cache = self.cache.lock().unwrap();
        let cap = cache.cap().get();
        let len_before = cache.len();
        let exists = cache.contains(key);

        // put returns the old value if key existed
        // If cache was full and key didn't exist, it returns (K, V) of evicted item?
        // lru::LruCache::put returns Option<V> (old value of key).
        // It does NOT return evicted item.
        cache.put(key.to_string(), entry);

        // Metrics logic
        let len_after = cache.len();

        if !exists {
            if len_before == cap && len_after == cap {
                // Must have evicted something
                self.metrics.record_eviction();
                // size stays same
            } else {
                self.metrics.increment_cache_size();
            }
        }

        let elapsed = start.elapsed();
        self.metrics.record_set(elapsed);

        debug!("L1 cache set for key: {} with TTL: {:?}", key, ttl);
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let start = Instant::now();

        let mut cache = self.cache.lock().unwrap();
        if cache.pop(key).is_some() {
            self.metrics.decrement_cache_size();
            debug!("L1 cache delete for key: {}", key);
        }

        let elapsed = start.elapsed();
        self.metrics.record_delete(elapsed);

        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let mut cache = self.cache.lock().unwrap();
        // peek() does not update LRU
        let exists = cache
            .peek(key)
            .map(|entry| !entry.is_expired())
            .unwrap_or(false);

        debug!("L1 cache exists check for key {}: {}", key, exists);
        Ok(exists)
    }

    async fn expire(&self, key: &str, ttl: Duration) -> Result<()> {
        let mut cache = self.cache.lock().unwrap();
        if let Some(entry) = cache.get_mut(key) {
            entry.expires_at = Instant::now() + ttl;
            debug!("L1 cache expire set for key: {} with TTL: {:?}", key, ttl);
        }
        Ok(())
    }

    async fn increment(&self, key: &str, delta: i64) -> Result<i64> {
        let mut cache = self.cache.lock().unwrap();
        let mut new_value = delta;

        // Check if exists
        let entry_exists = cache.contains(key);

        if entry_exists {
            if let Some(entry) = cache.get_mut(key) {
                if let Some(num) = entry.value.as_i64() {
                    new_value = num + delta;
                    entry.value = serde_json::Value::Number(new_value.into());
                    entry.touch();
                } else {
                    // Not a number, overwrite with delta as suggested
                    new_value = delta;
                    entry.value = serde_json::Value::Number(new_value.into());
                    entry.touch();
                }
            }
        } else {
            // Insert new
            cache.put(
                key.to_string(),
                CacheEntry::new(serde_json::Value::Number(delta.into()), self.default_ttl),
            );
            self.metrics.increment_cache_size(); // Assuming not full
        }

        debug!(
            "L1 cache increment for key: {} by {} = {}",
            key, delta, new_value
        );
        Ok(new_value)
    }

    async fn set_nx(&self, key: &str, value: &serde_json::Value, ttl: Duration) -> Result<bool> {
        let start = Instant::now();

        let mut cache = self.cache.lock().unwrap();

        // Check if key exists and is not expired
        if let Some(entry) = cache.get(key) {
            if !entry.is_expired() {
                debug!("L1 cache set_nx failed (key exists) for key: {}", key);
                return Ok(false);
            }
            // If expired, we proceed to overwrite it.
        }

        // Put overwrites
        // Metrics logic similar to set
        let cap = cache.cap().get();
        let len_before = cache.len();

        cache.put(key.to_string(), CacheEntry::new(value.clone(), ttl));

        let len_after = cache.len();
        if len_before == cap && len_after == cap {
            self.metrics.record_eviction();
        } else {
            // If we overwrote an expired item, size stays same?
            // `put` replaces.
            // If we replaced, size same.
            // If we added new, size +1.
            if len_after > len_before {
                self.metrics.increment_cache_size();
            }
        }

        let elapsed = start.elapsed();
        self.metrics.record_set(elapsed);

        debug!(
            "L1 cache set_nx succeeded for key: {} with TTL: {:?}",
            key, ttl
        );
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[tokio::test]
    async fn test_in_memory_cache_basic_operations() {
        let cache = InMemoryCache::new(100, Duration::from_secs(60));

        // Test set and get
        let key = "test_key";
        let value = serde_json::json!({"id": 123, "name": "test"});

        cache
            .set(key, &value, Duration::from_secs(60))
            .await
            .unwrap();
        let retrieved = cache.get(key).await.unwrap();
        assert_eq!(retrieved, Some(value));

        // Test exists
        assert!(cache.exists(key).await.unwrap());

        // Test delete
        cache.delete(key).await.unwrap();
        let retrieved = cache.get(key).await.unwrap();
        assert_eq!(retrieved, None);
    }

    #[tokio::test]
    async fn test_in_memory_cache_ttl() {
        let cache = InMemoryCache::new(100, Duration::from_secs(60));

        let key = "ttl_test";
        let value = serde_json::json!({"data": "test"});

        // Set with short TTL
        cache
            .set(key, &value, Duration::from_millis(100))
            .await
            .unwrap();

        // Should exist immediately
        assert!(cache.get(key).await.unwrap().is_some());

        // Wait for expiration
        thread::sleep(Duration::from_millis(150));

        // Should be expired
        assert!(cache.get(key).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_in_memory_cache_lru_eviction() {
        let cache = InMemoryCache::new(3, Duration::from_secs(60));

        // Fill cache to capacity
        cache
            .set("key1", &serde_json::json!(1), Duration::from_secs(60))
            .await
            .unwrap();
        cache
            .set("key2", &serde_json::json!(2), Duration::from_secs(60))
            .await
            .unwrap();
        cache
            .set("key3", &serde_json::json!(3), Duration::from_secs(60))
            .await
            .unwrap();

        // Access key1 to make it more recently used
        let _ = cache.get("key1").await;

        // Add a new key, should evict key2 (LRU) - key1 accessed, key3 added last, key2 added 2nd last but never accessed?
        // insert order: 1, 2, 3. LRU: 1, 2, 3 (3 is MRU).
        // access 1: LRU: 2, 3, 1 (1 is MRU).
        // set 4: evict 2.
        cache
            .set("key4", &serde_json::json!(4), Duration::from_secs(60))
            .await
            .unwrap();

        // key1 and key3 should still exist
        assert!(cache.get("key1").await.unwrap().is_some());
        assert!(cache.get("key3").await.unwrap().is_some());
        assert!(cache.get("key4").await.unwrap().is_some());

        // key2 should be gone
        assert!(cache.get("key2").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_in_memory_cache_set_nx() {
        let cache = InMemoryCache::new(100, Duration::from_secs(60));

        let key = "set_nx_test";
        let value = serde_json::json!({"data": "test"});

        // First set_nx should succeed
        let result = cache
            .set_nx(key, &value, Duration::from_secs(60))
            .await
            .unwrap();
        assert!(result);

        // Second set_nx should fail (key exists)
        let result = cache
            .set_nx(key, &value, Duration::from_secs(60))
            .await
            .unwrap();
        assert!(!result);

        // After deletion, set_nx should succeed again
        cache.delete(key).await.unwrap();
        let result = cache
            .set_nx(key, &value, Duration::from_secs(60))
            .await
            .unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_in_memory_cache_increment() {
        let cache = InMemoryCache::new(100, Duration::from_secs(60));

        let key = "counter";

        // First increment
        let result = cache.increment(key, 1).await.unwrap();
        assert_eq!(result, 1);

        // Second increment
        let result = cache.increment(key, 5).await.unwrap();
        assert_eq!(result, 6);

        // Negative increment
        let result = cache.increment(key, -2).await.unwrap();
        assert_eq!(result, 4);
    }

    #[tokio::test]
    async fn test_in_memory_cache_increment_non_numeric() {
        let cache = InMemoryCache::new(100, Duration::from_secs(60));
        let key = "not_a_number";

        // Set a non-numeric value
        cache
            .set(
                key,
                &serde_json::json!("string_value"),
                Duration::from_secs(60),
            )
            .await
            .unwrap();

        // Increment should overwrite and return delta
        let result = cache.increment(key, 10).await.unwrap();
        assert_eq!(result, 10);

        // Verify it was actually stored
        let stored = cache.get(key).await.unwrap();
        assert_eq!(stored, Some(serde_json::json!(10)));
    }

    #[tokio::test]
    async fn test_in_memory_cache_clear() {
        let cache = InMemoryCache::new(100, Duration::from_secs(60));

        // Add some entries
        cache
            .set("key1", &serde_json::json!(1), Duration::from_secs(60))
            .await
            .unwrap();
        cache
            .set("key2", &serde_json::json!(2), Duration::from_secs(60))
            .await
            .unwrap();

        assert_eq!(cache.size(), 2);

        // Clear cache
        cache.clear();

        assert_eq!(cache.size(), 0);
        assert!(cache.get("key1").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_in_memory_cache_metrics() {
        let cache = InMemoryCache::new(100, Duration::from_secs(60));

        let key = "metrics_test";
        let value = serde_json::json!({"data": "test"});

        // Set and get to generate metrics
        cache
            .set(key, &value, Duration::from_secs(60))
            .await
            .unwrap();
        let _ = cache.get(key).await;
        let _ = cache.get("nonexistent").await;

        let metrics = cache.metrics();
        assert_eq!(metrics.hits(), 1);
        assert_eq!(metrics.misses(), 1);
        assert_eq!(metrics.set_operations(), 1);
        assert_eq!(metrics.get_operations(), 2);
    }
}
