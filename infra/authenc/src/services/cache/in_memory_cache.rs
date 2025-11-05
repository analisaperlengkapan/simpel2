//! In-memory cache implementation (L1 cache)
//!
//! This module provides a fast in-memory cache using DashMap with TTL support
//! and LRU eviction for the L1 cache layer in the multi-layer caching strategy.

use super::{Cache, CacheMetrics};
use crate::error::Result;
use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::debug;

/// Entry in the in-memory cache with TTL
#[derive(Debug, Clone)]
struct CacheEntry {
    /// The cached value
    value: serde_json::Value,
    /// When this entry expires
    expires_at: Instant,
    /// When this entry was last accessed (for LRU)
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

/// In-memory cache implementation using DashMap
pub struct InMemoryCache {
    /// The cache storage
    cache: Arc<DashMap<String, CacheEntry>>,
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

        Self {
            cache: Arc::new(DashMap::with_capacity(max_size)),
            max_size,
            default_ttl,
            metrics: Arc::new(CacheMetrics::new()),
        }
    }

    /// Evict expired entries
    fn evict_expired(&self) {
        let now = Instant::now();
        let mut evicted = 0;

        self.cache.retain(|_key, entry| {
            if entry.is_expired() {
                evicted += 1;
                false
            } else {
                true
            }
        });

        if evicted > 0 {
            debug!("Evicted {} expired entries from L1 cache", evicted);
            for _ in 0..evicted {
                self.metrics.record_eviction();
                self.metrics.decrement_cache_size();
            }
        }
    }

    /// Evict least recently used entries if cache is full
    fn evict_lru(&self) {
        if self.cache.len() < self.max_size {
            return;
        }

        // Find the LRU entry
        let mut lru_key: Option<String> = None;
        let mut lru_time = Instant::now();

        for entry in self.cache.iter() {
            if entry.value().last_accessed < lru_time {
                lru_time = entry.value().last_accessed;
                lru_key = Some(entry.key().clone());
            }
        }

        // Remove the LRU entry
        if let Some(key) = lru_key {
            self.cache.remove(&key);
            self.metrics.record_eviction();
            self.metrics.decrement_cache_size();
            debug!("Evicted LRU entry from L1 cache: {}", key);
        }
    }

    /// Get cache metrics
    pub fn metrics(&self) -> Arc<CacheMetrics> {
        Arc::clone(&self.metrics)
    }

    /// Get current cache size
    pub fn size(&self) -> usize {
        self.cache.len()
    }

    /// Clear all entries from the cache
    pub fn clear(&self) {
        let size = self.cache.len();
        self.cache.clear();
        self.metrics.update_cache_size(0);
        debug!("Cleared {} entries from L1 cache", size);
    }
}

#[async_trait]
impl Cache for InMemoryCache {
    async fn get(&self, key: &str) -> Result<Option<serde_json::Value>> {
        let start = Instant::now();

        // Evict expired entries periodically
        if self.cache.len() % 100 == 0 {
            self.evict_expired();
        }

        let result = self.cache.get_mut(key).and_then(|mut entry| {
            if entry.is_expired() {
                None
            } else {
                entry.touch();
                Some(entry.value.clone())
            }
        });

        let elapsed = start.elapsed();
        self.metrics.record_get(elapsed);

        match result {
            Some(value) => {
                debug!("L1 cache hit for key: {}", key);
                self.metrics.record_hit();
                Ok(Some(value))
            }
            None => {
                debug!("L1 cache miss for key: {}", key);
                self.metrics.record_miss();
                Ok(None)
            }
        }
    }

    async fn set(&self, key: &str, value: &serde_json::Value, ttl: Duration) -> Result<()> {
        let start = Instant::now();

        // Evict LRU if cache is full
        self.evict_lru();

        let entry = CacheEntry::new(value.clone(), ttl);
        self.cache.insert(key.to_string(), entry);

        let elapsed = start.elapsed();
        self.metrics.record_set(elapsed);
        self.metrics.increment_cache_size();

        debug!("L1 cache set for key: {} with TTL: {:?}", key, ttl);
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let start = Instant::now();

        if self.cache.remove(key).is_some() {
            self.metrics.decrement_cache_size();
            debug!("L1 cache delete for key: {}", key);
        }

        let elapsed = start.elapsed();
        self.metrics.record_delete(elapsed);

        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let exists = self
            .cache
            .get(key)
            .map(|entry| !entry.is_expired())
            .unwrap_or(false);

        debug!("L1 cache exists check for key {}: {}", key, exists);
        Ok(exists)
    }

    async fn expire(&self, key: &str, ttl: Duration) -> Result<()> {
        if let Some(mut entry) = self.cache.get_mut(key) {
            entry.expires_at = Instant::now() + ttl;
            debug!("L1 cache expire set for key: {} with TTL: {:?}", key, ttl);
        }
        Ok(())
    }

    async fn increment(&self, key: &str, delta: i64) -> Result<i64> {
        let mut new_value = delta;

        self.cache
            .entry(key.to_string())
            .and_modify(|entry| {
                if let Some(num) = entry.value.as_i64() {
                    new_value = num + delta;
                    entry.value = serde_json::Value::Number(new_value.into());
                    entry.touch();
                }
            })
            .or_insert_with(|| {
                CacheEntry::new(serde_json::Value::Number(delta.into()), self.default_ttl)
            });

        debug!(
            "L1 cache increment for key: {} by {} = {}",
            key, delta, new_value
        );
        Ok(new_value)
    }

    async fn set_nx(&self, key: &str, value: &serde_json::Value, ttl: Duration) -> Result<bool> {
        let start = Instant::now();

        // Check if key exists and is not expired
        if let Some(entry) = self.cache.get(key) {
            if !entry.is_expired() {
                debug!("L1 cache set_nx failed (key exists) for key: {}", key);
                return Ok(false);
            }
        }

        // Evict LRU if cache is full
        self.evict_lru();

        let entry = CacheEntry::new(value.clone(), ttl);
        self.cache.insert(key.to_string(), entry);

        let elapsed = start.elapsed();
        self.metrics.record_set(elapsed);
        self.metrics.increment_cache_size();

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

        // Add a new key, should evict key2 (LRU)
        cache
            .set("key4", &serde_json::json!(4), Duration::from_secs(60))
            .await
            .unwrap();

        // key1 and key3 should still exist
        assert!(cache.get("key1").await.unwrap().is_some());
        assert!(cache.get("key3").await.unwrap().is_some());
        assert!(cache.get("key4").await.unwrap().is_some());
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
