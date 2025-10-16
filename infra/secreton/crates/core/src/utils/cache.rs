use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::RwLock as AsyncRwLock;
use tracing::{debug, warn};

use crate::error::{SecretonError, Result};

/// Cache entry with TTL support for secrets
#[derive(Debug, Clone)]
struct CacheEntry<V> {
    value: V,
    created_at: Instant,
    ttl: Duration,
    access_count: u64,
    last_accessed: Instant,
    sensitivity_level: SensitivityLevel,
}

/// Sensitivity level for cached data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensitivityLevel {
    /// Low sensitivity - can be cached longer
    Low,
    /// Medium sensitivity - standard caching
    Medium,
    /// High sensitivity - short cache duration
    High,
    /// Critical sensitivity - minimal or no caching
    Critical,
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
        // Evict high sensitivity items earlier under load
        match self.sensitivity_level {
            SensitivityLevel::Critical => true, // Always evict critical items quickly
            SensitivityLevel::High => load_factor > 0.7,
            SensitivityLevel::Medium => load_factor > 0.9,
            SensitivityLevel::Low => false,
        }
    }
}

/// LRU Cache with TTL support optimized for secret storage
pub struct SecretLruCache<K, V> {
    data: HashMap<K, CacheEntry<V>>,
    capacity: usize,
    access_order: Vec<K>,
    hit_count: u64,
    miss_count: u64,
    eviction_count: u64,
}

impl<K, V> SecretLruCache<K, V>
where
    K: Clone + Eq + Hash,
    V: Clone,
{
    /// Create a new secret LRU cache with specified capacity
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

    /// Insert a value with TTL and sensitivity level
    pub fn insert(&mut self, key: K, value: V, ttl: Duration, sensitivity_level: SensitivityLevel) {
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
        self.data.insert(key, CacheEntry::new(value, ttl, sensitivity_level));
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
        self.eviction_count = 0;
    }

    /// Evict entries based on sensitivity and load
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

    /// Get cache statistics
    pub fn stats(&self) -> SecretCacheStats {
        let total_requests = self.hit_count + self.miss_count;
        let hit_ratio = if total_requests > 0 {
            self.hit_count as f64 / total_requests as f64
        } else {
            0.0
        };

        // Calculate sensitivity distribution
        let mut sensitivity_counts = [0; 4]; // Low, Medium, High, Critical
        for entry in self.data.values() {
            let index = match entry.sensitivity_level {
                SensitivityLevel::Low => 0,
                SensitivityLevel::Medium => 1,
                SensitivityLevel::High => 2,
                SensitivityLevel::Critical => 3,
            };
            sensitivity_counts[index] += 1;
        }

        SecretCacheStats {
            size: self.data.len(),
            capacity: self.capacity,
            hit_count: self.hit_count,
            miss_count: self.miss_count,
            eviction_count: self.eviction_count,
            hit_ratio,
            sensitivity_distribution: sensitivity_counts,
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
            self.eviction_count += 1;
        }
    }
}

/// Cache statistics for secrets
#[derive(Debug, Clone)]
pub struct SecretCacheStats {
    pub size: usize,
    pub capacity: usize,
    pub hit_count: u64,
    pub miss_count: u64,
    pub eviction_count: u64,
    pub hit_ratio: f64,
    pub sensitivity_distribution: [usize; 4], // [Low, Medium, High, Critical]
}

/// Thread-safe secret LRU cache wrapper
pub struct ThreadSafeSecretCache<K, V> {
    cache: Arc<RwLock<SecretLruCache<K, V>>>,
}

impl<K, V> ThreadSafeSecretCache<K, V>
where
    K: Clone + Eq + Hash,
    V: Clone,
{
    /// Create a new thread-safe secret cache
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(SecretLruCache::new(capacity))),
        }
    }

/// Insertth TTL and sensitivity level
    pub fn insert(&self, key: K, value: V, ttl: Duration, sensitivity_level: SensitivityLevel) -> Result<()> {
        let mut cache = self.cache
            .write()
            .map_err(|_| SecretonError::from(CoreError::Internal(anyhow::anyhow!("Failed to acquire cache write lock".to_string()))))?;
        cache.insert(key, value, ttl, sensitivity_level);
        Ok(())
    }

    /// Get a value from cache
    pub fn get(&self, key: &K) -> Result<Option<V>> {
        let mut cache = self.cache
            .write()
            .map_err(|_| SecretonError::from(CoreError::Internal(anyhow::anyhow!("Failed to acquire cache write lock".to_string()))))?;
        Ok(cache.get(key))
    }

    /// Remove a key from cache
    pub fn remove(&self, key: &K) -> Result<Option<V>> {
        let mut cache = self.cache
            .write()
            .map_err(|_| SecretonError::from(CoreError::Internal(anyhow::anyhow!("Failed to acquire cache write lock".to_string()))))?;
        Ok(cache.remove(key))
    }

    /// Clear all entries
    pub fn clear(&self) -> Result<()> {
        let mut cache = self.cache
            .write()
            .map_err(|_| SecretonError::from(CoreError::Internal(anyhow::anyhow!("Failed to acquire cache write lock".to_string()))))?;
        cache.clear();
        Ok(())
    }

    /// Evict entries based on sensitivity and load
    pub fn evict_by_sensitivity(&self, load_factor: f64) -> Result<()> {
        let mut cache = self.cache
            .write()
            .map_err(|_| SecretonError::from(CoreError::Internal(anyhow::anyhow!("Failed to acquire cache write lock".to_string()))))?;
        cache.evict_by_sensitivity(load_factor);
        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> Result<SecretCacheStats> {
        let cache = self.cache
            .read()
            .map_err(|_| SecretonError::from(CoreError::Internal(anyhow::anyhow!("Failed to acquire cache read lock".to_string()))))?;
        Ok(cache.stats())
    }
}

impl<K, V> Clone for ThreadSafeSecretCache<K, V> {
    fn clone(&self) -> Self {
        Self {
            cache: Arc::clone(&self.cache),
        }
    }
}

/// Async secret LRU cache for use in async contexts
pub struct AsyncSecretCache<K, V> {
    cache: Arc<AsyncRwLock<SecretLruCache<K, V>>>,
}

impl<K, V> AsyncSecretCache<K, V>
where
    K: Clone + Eq + Hash,
    V: Clone,
{
    /// Create a new async secret cache
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Arc::new(AsyncRwLock::new(SecretLruCache::new(capacity))),
        }
    }

    /// Insert a value with TTL and sensitivity level
    pub async fn insert(&self, key: K, value: V, ttl: Duration, sensitivity_level: SensitivityLevel) {
        let mut cache = self.cache.write().await;
        cache.insert(key, value, ttl, sensitivity_level);
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

    /// Evict entries based on sensitivity and load
    pub async fn evict_by_sensitivity(&self, load_factor: f64) {
        let mut cache = self.cache.write().await;
        cache.evict_by_sensitivity(load_factor);
    }

    /// Get cache statistics
    pub async fn stats(&self) -> SecretCacheStats {
        let cache = self.cache.read().await;
        cache.stats()
    }
}

impl<K, V> Clone for AsyncSecretCache<K, V> {
    fn clone(&self) -> Self {
        Self {
            cache: Arc::clone(&self.cache),
        }
    }
}

/// Specialized cache types for different secret operations
pub type SecretValueCache = AsyncSecretCache<String, Vec<u8>>;
pub type TokenValidationCache = AsyncSecretCache<String, bool>;
pub type UserPermissionCache = AsyncSecretCache<String, Vec<String>>;
pub type EncryptionKeyCache = AsyncSecretCache<String, Vec<u8>>;

/// Secret cache manager for coordinating multiple caches
pub struct SecretCacheManager {
    secret_cache: SecretValueCache,
    token_cache: TokenValidationCache,
    permission_cache: UserPermissionCache,
    key_cache: EncryptionKeyCache,
}

impl SecretCacheManager {
    /// Create a new secret cache manager with default capacities
    pub fn new() -> Self {
        Self {
            secret_cache: SecretValueCache::new(20000),
            token_cache: TokenValidationCache::new(50000),
            permission_cache: UserPermissionCache::new(10000),
            key_cache: EncryptionKeyCache::new(5000),
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
        }
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
    }

    /// Evict sensitive entries based on load
    pub async fn evict_sensitive(&self, load_factor: f64) {
        self.secret_cache.evict_by_sensitivity(load_factor).await;
        self.token_cache.evict_by_sensitivity(load_factor).await;
        self.permission_cache.evict_by_sensitivity(load_factor).await;
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

/// Combined cache statistics for secret operations
#[derive(Debug, Clone)]
pub struct SecretCacheManagerStats {
    pub secret_cache: SecretCacheStats,
    pub token_cache: SecretCacheStats,
    pub permission_cache: SecretCacheStats,
    pub key_cache: SecretCacheStats,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_cache_basic_operations() {
        let mut cache = SecretLruCache::new(3);

        cache.insert("key1", vec![1, 2, 3], Duration::from_secs(60), SensitivityLevel::Low);
        cache.insert("key2", vec![4, 5, 6], Duration::from_secs(60), SensitivityLevel::Medium);
        cache.insert("key3", vec![7, 8, 9], Duration::from_secs(60), SensitivityLevel::High);

        assert_eq!(cache.get(&"key1"), Some(vec![1, 2, 3]));
        assert_eq!(cache.get(&"key2"), Some(vec![4, 5, 6]));
        assert_eq!(cache.get(&"key3"), Some(vec![7, 8, 9]));

        // Insert one more to trigger LRU eviction
        cache.insert("key4", vec![10, 11, 12], Duration::from_secs(60), SensitivityLevel::Low);

        // key1 should be evicted (least recently used)
        assert_eq!(cache.get(&"key1"), None);
        assert_eq!(cache.get(&"key4"), Some(vec![10, 11, 12]));
    }

    #[test]
    fn test_sensitivity_eviction() {
        let mut cache = SecretLruCache::new(5);

        cache.insert("low", vec![1], Duration::from_secs(60), SensitivityLevel::Low);
        cache.insert("medium", vec![2], Duration::from_secs(60), SensitivityLevel::Medium);
        cache.insert("high", vec![3], Duration::from_secs(60), SensitivityLevel::High);
        cache.insert("critical", vec![4], Duration::from_secs(60), SensitivityLevel::Critical);

        // High load should evict high and critical sensitivity items
        cache.evict_by_sensitivity(0.8);

        assert_eq!(cache.get(&"low"), Some(vec![1]));
        assert_eq!(cache.get(&"medium"), Some(vec![2]));
        assert_eq!(cache.get(&"high"), None);
        assert_eq!(cache.get(&"critical"), None);
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = SecretLruCache::new(3);

        cache.insert("key1", vec![1], Duration::from_secs(60), SensitivityLevel::Low);
        cache.insert("key2", vec![2], Duration::from_secs(60), SensitivityLevel::High);

        cache.get(&"key1"); // hit
        cache.get(&"key3"); // miss

        let stats = cache.stats();
        assert_eq!(stats.hit_count, 1);
        assert_eq!(stats.miss_count, 1);
        assert_eq!(stats.hit_ratio, 0.5);
        assert_eq!(stats.sensitivity_distribution[0], 1); // Low
        assert_eq!(stats.sensitivity_distribution[2], 1); // High
    }

    #[tokio::test]
    async fn test_async_secret_cache() {
        let cache = AsyncSecretCache::new(3);

        cache.insert("key1".to_string(), vec![1, 2, 3], Duration::from_secs(60), SensitivityLevel::Medium).await;
        cache.insert("key2".to_string(), vec![4, 5, 6], Duration::from_secs(60), SensitivityLevel::High).await;

        assert_eq!(cache.get(&"key1".to_string()).await, Some(vec![1, 2, 3]));
        assert_eq!(cache.get(&"key2".to_string()).await, Some(vec![4, 5, 6]));
        assert_eq!(cache.get(&"key3".to_string()).await, None);
    }

    #[tokio::test]
    async fn test_secret_cache_manager() {
        let manager = SecretCacheManager::new();

        manager.secret_cache().insert("secret1".to_string(), vec![1, 2, 3], Duration::from_secs(60), SensitivityLevel::Medium).await;
        manager.token_cache().insert("token1".to_string(), true, Duration::from_secs(60), SensitivityLevel::Low).await;

        assert_eq!(manager.secret_cache().get(&"secret1".to_string()).await, Some(vec![1, 2, 3]));
        assert_eq!(manager.token_cache().get(&"token1".to_string()).await, Some(true));

        let stats = manager.get_stats().await;
        assert_eq!(stats.secret_cache.size, 1);
        assert_eq!(stats.token_cache.size, 1);
    }
}
