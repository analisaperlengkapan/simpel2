//! Generic LRU Cache implementation
//!
//! Provides in-memory LRU caching with TTL support and optional Redis/Hybrid backends.

use crate::error::{CommonError, Result};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::RwLock as AsyncRwLock;

#[cfg(feature = "redis-cache")]
use redis::{AsyncCommands, Client as RedisClient};

/// Sensitivity level for cached data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SensitivityLevel {
    /// Low sensitivity - can be cached longer
    Low,
    /// Medium sensitivity - standard caching (default)
    #[default]
    Medium,
    /// High sensitivity - short cache duration
    High,
    /// Critical sensitivity - minimal or no caching
    Critical,
}

/// Cache entry with TTL support
#[derive(Debug, Clone)]
struct CacheEntry<V> {
    value: V,
    created_at: Instant,
    ttl: Duration,
    access_count: u64,
    last_accessed: Instant,
    sensitivity_level: SensitivityLevel,
}

impl<V> CacheEntry<V> {
    fn new(value: V, ttl: Duration, sensitivity_level: SensitivityLevel) -> Self {
        let now = Instant::now();
        Self {
            value,
            created_at: now,
            ttl,
            access_count: 1,
            last_accessed: now,
            sensitivity_level,
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

    fn should_evict_early(&self, load_factor: f64) -> bool {
        match self.sensitivity_level {
            SensitivityLevel::Critical => true,
            SensitivityLevel::High => load_factor > 0.7,
            SensitivityLevel::Medium => load_factor > 0.9,
            SensitivityLevel::Low => false,
        }
    }
}

/// LRU Cache with TTL and sensitivity support
pub struct LruCache<K, V> {
    data: HashMap<K, CacheEntry<V>>,
    capacity: usize,
    access_order: Vec<K>,
    hit_count: u64,
    miss_count: u64,
    eviction_count: u64,
}

impl<K, V> LruCache<K, V>
where
    K: Clone + Eq + Hash,
    V: Clone,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            data: HashMap::with_capacity(capacity),
            capacity,
            access_order: Vec::with_capacity(capacity),
            hit_count: 0,
            miss_count: 0,
            eviction_count: 0,
        }
    }

    pub fn insert(&mut self, key: K, value: V, ttl: Duration) {
        self.insert_with_sensitivity(key, value, ttl, SensitivityLevel::default())
    }

    pub fn insert_with_sensitivity(
        &mut self,
        key: K,
        value: V,
        ttl: Duration,
        sensitivity_level: SensitivityLevel,
    ) {
        self.cleanup_expired();

        if self.data.len() >= self.capacity && !self.data.contains_key(&key) {
            self.evict_lru();
        }

        if let Some(pos) = self.access_order.iter().position(|k| k == &key) {
            self.access_order.remove(pos);
        }
        self.access_order.push(key.clone());
        self.data
            .insert(key, CacheEntry::new(value, ttl, sensitivity_level));
    }

    pub fn get(&mut self, key: &K) -> Option<V> {
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

    pub fn clear(&mut self) {
        self.data.clear();
        self.access_order.clear();
        self.hit_count = 0;
        self.miss_count = 0;
        self.eviction_count = 0;
    }

    pub fn evict_by_sensitivity(&mut self, load_factor: f64) {
        let keys_to_evict: Vec<K> = self
            .data
            .iter()
            .filter(|(_, entry)| entry.should_evict_early(load_factor))
            .map(|(key, _)| key.clone())
            .collect();

        for key in keys_to_evict {
            self.remove(&key);
            self.eviction_count += 1;
        }
    }

    pub fn stats(&self) -> CacheStats {
        let total_requests = self.hit_count + self.miss_count;
        let hit_ratio = if total_requests > 0 {
            self.hit_count as f64 / total_requests as f64
        } else {
            0.0
        };

        // Calculate sensitivity distribution
        let mut sensitivity_counts = [0; 4];
        for entry in self.data.values() {
            let index = match entry.sensitivity_level {
                SensitivityLevel::Low => 0,
                SensitivityLevel::Medium => 1,
                SensitivityLevel::High => 2,
                SensitivityLevel::Critical => 3,
            };
            sensitivity_counts[index] += 1;
        }

        CacheStats {
            size: self.data.len(),
            capacity: self.capacity,
            hit_count: self.hit_count,
            miss_count: self.miss_count,
            eviction_count: self.eviction_count,
            hit_ratio,
            sensitivity_distribution: sensitivity_counts,
        }
    }

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

    fn evict_lru(&mut self) {
        if let Some(lru_key) = self.access_order.first().cloned() {
            self.data.remove(&lru_key);
            self.access_order.remove(0);
            self.eviction_count += 1;
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub size: usize,
    pub capacity: usize,
    pub hit_count: u64,
    pub miss_count: u64,
    pub eviction_count: u64,
    pub hit_ratio: f64,
    pub sensitivity_distribution: [usize; 4],
}

pub struct ThreadSafeLruCache<K, V> {
    cache: Arc<RwLock<LruCache<K, V>>>,
}

impl<K, V> ThreadSafeLruCache<K, V>
where
    K: Clone + Eq + Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(LruCache::new(capacity))),
        }
    }

    pub fn insert(&self, key: K, value: V, ttl: Duration) -> Result<()> {
        self.insert_with_sensitivity(key, value, ttl, SensitivityLevel::default())
    }

    pub fn insert_with_sensitivity(
        &self,
        key: K,
        value: V,
        ttl: Duration,
        sensitivity_level: SensitivityLevel,
    ) -> Result<()> {
        let mut cache = self
            .cache
            .write()
            .map_err(|_| CommonError::Cache("Failed to acquire cache write lock".to_string()))?;
        cache.insert_with_sensitivity(key, value, ttl, sensitivity_level);
        Ok(())
    }

    pub fn get(&self, key: &K) -> Result<Option<V>> {
        let mut cache = self
            .cache
            .write()
            .map_err(|_| CommonError::Cache("Failed to acquire cache write lock".to_string()))?;
        Ok(cache.get(key))
    }

    pub fn remove(&self, key: &K) -> Result<Option<V>> {
        let mut cache = self
            .cache
            .write()
            .map_err(|_| CommonError::Cache("Failed to acquire cache write lock".to_string()))?;
        Ok(cache.remove(key))
    }

    pub fn clear(&self) -> Result<()> {
        let mut cache = self
            .cache
            .write()
            .map_err(|_| CommonError::Cache("Failed to acquire cache write lock".to_string()))?;
        cache.clear();
        Ok(())
    }

    pub fn stats(&self) -> Result<CacheStats> {
        let cache = self
            .cache
            .read()
            .map_err(|_| CommonError::Cache("Failed to acquire cache read lock".to_string()))?;
        Ok(cache.stats())
    }
}

pub struct AsyncLruCache<K, V> {
    cache: Arc<AsyncRwLock<LruCache<K, V>>>,
}

impl<K, V> AsyncLruCache<K, V>
where
    K: Clone + Eq + Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Arc::new(AsyncRwLock::new(LruCache::new(capacity))),
        }
    }

    pub async fn insert(&self, key: K, value: V, ttl: Duration) {
        self.insert_with_sensitivity(key, value, ttl, SensitivityLevel::default()).await
    }

    pub async fn insert_with_sensitivity(
        &self,
        key: K,
        value: V,
        ttl: Duration,
        sensitivity_level: SensitivityLevel,
    ) {
        let mut cache = self.cache.write().await;
        cache.insert_with_sensitivity(key, value, ttl, sensitivity_level);
    }

    pub async fn get(&self, key: &K) -> Option<V> {
        let mut cache = self.cache.write().await;
        cache.get(key)
    }

    pub async fn remove(&self, key: &K) -> Option<V> {
        let mut cache = self.cache.write().await;
        cache.remove(key)
    }

    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    pub async fn evict_by_sensitivity(&self, load_factor: f64) {
        let mut cache = self.cache.write().await;
        cache.evict_by_sensitivity(load_factor);
    }

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

// Redis Cache Implementation
#[cfg(feature = "redis-cache")]
pub struct RedisCache {
    client: RedisClient,
    key_prefix: String,
}

#[cfg(feature = "redis-cache")]
impl RedisCache {
    pub fn new(url: &str, key_prefix: &str) -> Result<Self> {
        let client = RedisClient::open(url)
            .map_err(|e| CommonError::Cache(format!("Failed to connect to Redis: {}", e)))?;

        Ok(Self {
            client,
            key_prefix: key_prefix.to_string(),
        })
    }

    async fn get_connection(&self) -> Result<redis::aio::MultiplexedConnection> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| CommonError::Cache(format!("Failed to get Redis connection: {}", e)))
    }

    fn build_key(&self, key: &str) -> String {
        format!("{}:{}", self.key_prefix, key)
    }

    pub async fn set(&self, key: &str, value: &[u8], ttl: Duration) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let full_key = self.build_key(key);

        let _: () = conn
            .set_ex(&full_key, value, ttl.as_secs())
            .await
            .map_err(|e| CommonError::Cache(format!("Redis SET failed: {}", e)))?;

        Ok(())
    }

    pub async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let mut conn = self.get_connection().await?;
        let full_key = self.build_key(key);

        let result: Option<Vec<u8>> = conn
            .get(&full_key)
            .await
            .map_err(|e| CommonError::Cache(format!("Redis GET failed: {}", e)))?;

        Ok(result)
    }

    pub async fn delete(&self, key: &str) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let full_key = self.build_key(key);

        let _: () = conn
            .del(&full_key)
            .await
            .map_err(|e| CommonError::Cache(format!("Redis DEL failed: {}", e)))?;

        Ok(())
    }

    pub async fn clear_all(&self) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let pattern = format!("{}:*", self.key_prefix);

        let keys: Vec<String> = conn
            .keys(&pattern)
            .await
            .map_err(|e| CommonError::Cache(format!("Redis KEYS failed: {}", e)))?;

        if !keys.is_empty() {
            let _: () = conn
                .del(&keys)
                .await
                .map_err(|e| CommonError::Cache(format!("Redis DEL failed: {}", e)))?;
        }

        Ok(())
    }
}

// Hybrid Cache (Memory + Redis)
#[cfg(feature = "redis-cache")]
pub struct HybridCache<K, V>
where
    K: Clone + Eq + Hash + ToString + Send + Sync + 'static,
    V: Clone + serde::Serialize + serde::de::DeserializeOwned + Send + Sync + 'static,
{
    l1_cache: AsyncLruCache<K, V>,
    l2_cache: RedisCache,
}

#[cfg(feature = "redis-cache")]
impl<K, V> HybridCache<K, V>
where
    K: Clone + Eq + Hash + ToString + Send + Sync + 'static,
    V: Clone + serde::Serialize + serde::de::DeserializeOwned + Send + Sync + 'static,
{
    pub fn new(l1_capacity: usize, redis_url: &str, key_prefix: &str) -> Result<Self> {
        Ok(Self {
            l1_cache: AsyncLruCache::new(l1_capacity),
            l2_cache: RedisCache::new(redis_url, key_prefix)?,
        })
    }

    pub async fn get(&self, key: &K) -> Result<Option<V>> {
        // Check L1 first
        if let Some(value) = self.l1_cache.get(key).await {
            return Ok(Some(value));
        }

        // Check L2
        let key_str = key.to_string();
        if let Some(bytes) = self.l2_cache.get(&key_str).await? {
            match serde_json::from_slice::<V>(&bytes) {
                Ok(value) => {
                    // Populate L1 cache
                    self.l1_cache
                        .insert_with_sensitivity(
                            key.clone(),
                            value.clone(),
                            Duration::from_secs(300),
                            SensitivityLevel::Medium,
                        )
                        .await;
                    Ok(Some(value))
                }
                Err(e) => Err(CommonError::Cache(format!(
                    "Failed to deserialize from Redis: {}",
                    e
                ))),
            }
        } else {
            Ok(None)
        }
    }

    pub async fn set(
        &self,
        key: K,
        value: V,
        ttl: Duration,
        sensitivity: SensitivityLevel,
    ) -> Result<()> {
        // Write to L1
        self.l1_cache
            .insert_with_sensitivity(key.clone(), value.clone(), ttl, sensitivity)
            .await;

        // Write to L2
        let key_str = key.to_string();
        let bytes = serde_json::to_vec(&value)
            .map_err(|e| CommonError::Cache(format!("Failed to serialize for Redis: {}", e)))?;

        self.l2_cache.set(&key_str, &bytes, ttl).await?;

        Ok(())
    }

    pub async fn remove(&self, key: &K) -> Result<()> {
        self.l1_cache.remove(key).await;
        let key_str = key.to_string();
        self.l2_cache.delete(&key_str).await?;
        Ok(())
    }

    pub async fn clear(&self) -> Result<()> {
        self.l1_cache.clear().await;
        self.l2_cache.clear_all().await?;
        Ok(())
    }
}
