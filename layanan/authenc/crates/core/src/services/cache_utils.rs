//! Cache utilities
//!
//! DashMap-based in-memory caches with TTL support for JWT tokens,
//! user sessions, and validation results.

use authenc_storage::CacheStats;
use dashmap::DashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// A cached entry with TTL tracking
#[derive(Clone)]
struct CachedEntry<V> {
    value: V,
    inserted_at: Instant,
    ttl: Duration,
}

impl<V> CachedEntry<V> {
    fn is_expired(&self) -> bool {
        self.inserted_at.elapsed() > self.ttl
    }
}

/// Generic in-memory cache backed by DashMap with per-entry TTL
pub struct TtlCache<K: Eq + std::hash::Hash, V: Clone> {
    entries: DashMap<K, CachedEntry<V>>,
    capacity: usize,
    default_ttl: Duration,
    hit_count: AtomicUsize,
    miss_count: AtomicUsize,
}

impl<K: Eq + std::hash::Hash, V: Clone> TtlCache<K, V> {
    /// Create a new cache with given capacity and default TTL
    pub fn new(capacity: usize, default_ttl: Duration) -> Self {
        Self {
            entries: DashMap::with_capacity(capacity),
            capacity,
            default_ttl,
            hit_count: AtomicUsize::new(0),
            miss_count: AtomicUsize::new(0),
        }
    }

    /// Get a value from the cache, returning None if expired or absent
    pub fn get(&self, key: &K) -> Option<V> {
        if let Some(entry) = self.entries.get(key) {
            if entry.is_expired() {
                drop(entry);
                self.entries.remove(key);
                self.miss_count.fetch_add(1, Ordering::Relaxed);
                None
            } else {
                self.hit_count.fetch_add(1, Ordering::Relaxed);
                Some(entry.value.clone())
            }
        } else {
            self.miss_count.fetch_add(1, Ordering::Relaxed);
            None
        }
    }

    /// Insert a value with default TTL, evicting oldest if at capacity
    pub fn insert(&self, key: K, value: V) {
        self.insert_with_ttl(key, value, self.default_ttl);
    }

    /// Insert a value with a custom TTL
    pub fn insert_with_ttl(&self, key: K, value: V, ttl: Duration) {
        // Evict expired entries if we're at capacity
        if self.entries.len() >= self.capacity {
            self.evict_expired();
        }

        self.entries.insert(
            key,
            CachedEntry {
                value,
                inserted_at: Instant::now(),
                ttl,
            },
        );
    }

    /// Remove a specific entry
    pub fn remove(&self, key: &K) -> Option<V> {
        self.entries.remove(key).map(|(_, e)| e.value)
    }

    /// Clear all entries
    pub fn clear(&self) {
        self.entries.clear();
        self.hit_count.store(0, Ordering::Relaxed);
        self.miss_count.store(0, Ordering::Relaxed);
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            size: self.entries.len(),
            max_size: self.capacity,
        }
    }

    /// Evict all expired entries
    fn evict_expired(&self) {
        self.entries.retain(|_, entry| !entry.is_expired());
    }

    /// Current number of entries (including potentially expired)
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Cache for validated JWT tokens (short TTL, high throughput)
pub struct TokenCache {
    inner: TtlCache<String, serde_json::Value>,
}

impl TokenCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: TtlCache::new(capacity, Duration::from_secs(300)), // 5 min TTL
        }
    }

    /// Cache a validated token's claims
    pub fn cache_token(&self, token_hash: String, claims: serde_json::Value) {
        self.inner.insert(token_hash, claims);
    }

    /// Look up cached token claims
    pub fn get_claims(&self, token_hash: &str) -> Option<serde_json::Value> {
        self.inner.get(&token_hash.to_string())
    }

    /// Invalidate a specific token
    pub fn invalidate(&self, token_hash: &str) {
        self.inner.remove(&token_hash.to_string());
    }

    pub async fn clear(&self) {
        self.inner.clear();
    }

    pub async fn stats(&self) -> CacheStats {
        self.inner.stats()
    }
}

/// Cache for user sessions (medium TTL)
pub struct SessionCache {
    inner: TtlCache<String, serde_json::Value>,
}

impl SessionCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: TtlCache::new(capacity, Duration::from_secs(1800)), // 30 min TTL
        }
    }

    /// Cache session data
    pub fn cache_session(&self, session_id: String, session_data: serde_json::Value) {
        self.inner.insert(session_id, session_data);
    }

    /// Look up cached session
    pub fn get_session(&self, session_id: &str) -> Option<serde_json::Value> {
        self.inner.get(&session_id.to_string())
    }

    /// Remove a session from cache
    pub fn remove_session(&self, session_id: &str) {
        self.inner.remove(&session_id.to_string());
    }

    pub async fn clear(&self) {
        self.inner.clear();
    }

    pub async fn stats(&self) -> CacheStats {
        self.inner.stats()
    }
}

/// Cache for validation results (short TTL to avoid repeated DB lookups)
pub struct ValidationCache {
    inner: TtlCache<String, bool>,
}

impl ValidationCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: TtlCache::new(capacity, Duration::from_secs(60)), // 1 min TTL
        }
    }

    /// Cache a validation result
    pub fn cache_result(&self, key: String, valid: bool) {
        self.inner.insert(key, valid);
    }

    /// Look up a cached validation result
    pub fn get_result(&self, key: &str) -> Option<bool> {
        self.inner.get(&key.to_string())
    }

    pub async fn clear(&self) {
        self.inner.clear();
    }

    pub async fn stats(&self) -> CacheStats {
        self.inner.stats()
    }
}

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
        CacheManagerStats {
            token_cache: self.token_cache.stats().await,
            session_cache: self.session_cache.stats().await,
            validation_cache: self.validation_cache.stats().await,
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
