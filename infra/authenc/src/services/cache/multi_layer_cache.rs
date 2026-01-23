//! Multi-layer cache implementation
//!
//! This module provides a multi-layer caching strategy with:
//! - L1: Fast in-memory cache (DashMap) with 60s TTL and 10k entries
//! - L2: Distributed Redis cache with configurable TTL
//! - Cache-aside pattern with automatic L1 population from L2
//! - LRU eviction for L1 cache

use super::{Cache, CacheMetrics, InMemoryCache, RedisCache};
use crate::error::Result;
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, warn};

/// Multi-layer cache configuration
#[derive(Debug, Clone)]
pub struct MultiLayerCacheConfig {
    /// L1 cache maximum size (number of entries)
    pub l1_max_size: usize,
    /// L1 cache default TTL
    pub l1_ttl: Duration,
    /// Whether to enable L1 cache
    pub l1_enabled: bool,
    /// Whether to enable L2 cache
    pub l2_enabled: bool,
}

impl Default for MultiLayerCacheConfig {
    fn default() -> Self {
        Self {
            l1_max_size: 10_000,
            l1_ttl: Duration::from_secs(60),
            l1_enabled: true,
            l2_enabled: true,
        }
    }
}

/// Multi-layer cache wrapper
/// Implements a two-tier caching strategy:
/// - L1: Fast in-memory cache for hot data
/// - L2: Distributed Redis cache for shared data
/// # Cache-aside Pattern
/// On GET:
/// 1. Check L1 cache (in-memory)
/// 2. If miss, check L2 cache (Redis)
/// 3. If hit in L2, populate L1 and return
/// 4. If miss in both, return None
/// On SET:
/// 1. Write to both L1 and L2
/// 2. Use shorter TTL for L1 (60s) vs L2 (configurable)
/// On DELETE:
/// 1. Delete from both L1 and L2
pub struct MultiLayerCache {
    /// L1 in-memory cache
    l1: Arc<InMemoryCache>,
    /// L2 Redis cache
    l2: Arc<RedisCache>,
    /// Configuration
    config: MultiLayerCacheConfig,
    /// Combined metrics
    metrics: Arc<CacheMetrics>,
}

impl MultiLayerCache {
    /// Create a new multi-layer cache
    ///
    /// # Arguments
    /// * `l2_cache` - Redis cache instance for L2
    /// * `config` - Multi-layer cache configuration
    pub fn new(l2_cache: Arc<RedisCache>, config: MultiLayerCacheConfig) -> Self {
        let l1 = Arc::new(InMemoryCache::new(config.l1_max_size, config.l1_ttl));

        debug!(
            "Created multi-layer cache with L1 (max_size={}, ttl={:?}) and L2 (Redis)",
            config.l1_max_size, config.l1_ttl
        );

        Self {
            l1,
            l2: l2_cache,
            config,
            metrics: Arc::new(CacheMetrics::new()),
        }
    }

    /// Create with default configuration
    pub fn with_defaults(l2_cache: Arc<RedisCache>) -> Self {
        Self::new(l2_cache, MultiLayerCacheConfig::default())
    }

    /// Get L1 cache metrics
    pub fn l1_metrics(&self) -> Arc<CacheMetrics> {
        self.l1.metrics()
    }

    /// Get L2 cache metrics
    pub fn l2_metrics(&self) -> Arc<CacheMetrics> {
        self.l2.metrics()
    }

    /// Get combined cache metrics
    pub fn metrics(&self) -> Arc<CacheMetrics> {
        Arc::clone(&self.metrics)
    }

    /// Get L1 cache size
    pub fn l1_size(&self) -> usize {
        self.l1.size()
    }

    /// Clear L1 cache
    pub fn clear_l1(&self) {
        self.l1.clear();
    }

    /// Get cache hit ratio (combined L1 + L2)
    pub fn hit_ratio(&self) -> f64 {
        self.metrics.hit_ratio()
    }

    /// Get L1 cache hit ratio
    pub fn l1_hit_ratio(&self) -> f64 {
        self.l1.metrics().hit_ratio()
    }

    /// Get L2 cache hit ratio
    pub fn l2_hit_ratio(&self) -> f64 {
        self.l2.metrics().hit_ratio()
    }
}

#[async_trait]
impl Cache for MultiLayerCache {
    async fn get(&self, key: &str) -> Result<Option<serde_json::Value>> {
        let start = std::time::Instant::now();

        // Try L1 cache first (fast path)
        if self.config.l1_enabled {
            if let Some(value) = self.l1.get(key).await? {
                debug!("Multi-layer cache: L1 hit for key: {}", key);
                self.metrics.record_hit();
                self.metrics.record_get(start.elapsed());
                return Ok(Some(value));
            }
        }

        // Try L2 cache (Redis)
        if self.config.l2_enabled {
            match self.l2.get(key).await {
                Ok(Some(value)) => {
                    debug!("Multi-layer cache: L2 hit for key: {}", key);

                    // Populate L1 cache with shorter TTL
                    if self.config.l1_enabled {
                        if let Err(e) = self.l1.set(key, &value, self.config.l1_ttl).await {
                            warn!("Failed to populate L1 cache from L2: {}", e);
                        }
                    }

                    self.metrics.record_hit();
                    self.metrics.record_get(start.elapsed());
                    return Ok(Some(value));
                }
                Ok(None) => {
                    debug!("Multi-layer cache: L2 miss for key: {}", key);
                }
                Err(e) => {
                    warn!("L2 cache error for key {}: {}", key, e);
                    self.metrics.record_error();
                }
            }
        }

        // Cache miss in both layers
        debug!("Multi-layer cache: miss for key: {}", key);
        self.metrics.record_miss();
        self.metrics.record_get(start.elapsed());
        Ok(None)
    }

    async fn set(&self, key: &str, value: &serde_json::Value, ttl: Duration) -> Result<()> {
        let start = std::time::Instant::now();

        // Write to L2 first (source of truth)
        if self.config.l2_enabled {
            if let Err(e) = self.l2.set(key, value, ttl).await {
                warn!("Failed to set L2 cache for key {}: {}", key, e);
                self.metrics.record_error();
                // Continue to L1 even if L2 fails
            }
        }

        // Write to L1 with shorter TTL
        if self.config.l1_enabled {
            let l1_ttl = std::cmp::min(self.config.l1_ttl, ttl);
            if let Err(e) = self.l1.set(key, value, l1_ttl).await {
                warn!("Failed to set L1 cache for key {}: {}", key, e);
                self.metrics.record_error();
            }
        }

        debug!(
            "Multi-layer cache: set for key: {} with TTL: {:?}",
            key, ttl
        );
        self.metrics.record_set(start.elapsed());
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let start = std::time::Instant::now();

        // Delete from both layers
        if self.config.l1_enabled {
            if let Err(e) = self.l1.delete(key).await {
                warn!("Failed to delete from L1 cache for key {}: {}", key, e);
            }
        }

        if self.config.l2_enabled {
            if let Err(e) = self.l2.delete(key).await {
                warn!("Failed to delete from L2 cache for key {}: {}", key, e);
                self.metrics.record_error();
            }
        }

        debug!("Multi-layer cache: delete for key: {}", key);
        self.metrics.record_delete(start.elapsed());
        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        // Check L1 first
        if self.config.l1_enabled && self.l1.exists(key).await? {
            return Ok(true);
        }

        // Check L2
        if self.config.l2_enabled {
            return self.l2.exists(key).await;
        }

        Ok(false)
    }

    async fn expire(&self, key: &str, ttl: Duration) -> Result<()> {
        // Update expiration in both layers
        if self.config.l1_enabled {
            let l1_ttl = std::cmp::min(self.config.l1_ttl, ttl);
            if let Err(e) = self.l1.expire(key, l1_ttl).await {
                warn!("Failed to update L1 expiration for key {}: {}", key, e);
            }
        }

        if self.config.l2_enabled {
            if let Err(e) = self.l2.expire(key, ttl).await {
                warn!("Failed to update L2 expiration for key {}: {}", key, e);
                self.metrics.record_error();
            }
        }

        Ok(())
    }

    async fn increment(&self, key: &str, delta: i64) -> Result<i64> {
        // Increment in L2 (source of truth)
        let new_value = if self.config.l2_enabled {
            self.l2.increment(key, delta).await?
        } else if self.config.l1_enabled {
            self.l1.increment(key, delta).await?
        } else {
            return Ok(delta);
        };

        // Invalidate L1 cache to ensure consistency
        if self.config.l1_enabled {
            if let Err(e) = self.l1.delete(key).await {
                warn!("Failed to invalidate L1 cache after increment: {}", e);
            }
        }

        Ok(new_value)
    }

    async fn set_nx(&self, key: &str, value: &serde_json::Value, ttl: Duration) -> Result<bool> {
        // Try L2 first (source of truth)
        let success = if self.config.l2_enabled {
            self.l2.set_nx(key, value, ttl).await?
        } else if self.config.l1_enabled {
            self.l1.set_nx(key, value, ttl).await?
        } else {
            return Ok(false);
        };

        // If successful, also set in L1
        if success && self.config.l1_enabled && self.config.l2_enabled {
            let l1_ttl = std::cmp::min(self.config.l1_ttl, ttl);
            if let Err(e) = self.l1.set(key, value, l1_ttl).await {
                warn!("Failed to populate L1 after set_nx: {}", e);
            }
        }

        Ok(success)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RedisConfig;

    /// Get Redis URL from environment or use default with password for docker
    fn get_test_redis_url() -> String {
        std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://:redis_password@localhost:6379/15".to_string())
    }

    // Helper to create a test Redis cache
    async fn create_test_redis_cache()
    -> std::result::Result<Arc<RedisCache>, crate::error::AuthencError> {
        let config = RedisConfig {
            enabled: true,
            url: get_test_redis_url(),
            ..Default::default()
        };

        Ok(Arc::new(RedisCache::new(&config).await?))
    }

    #[tokio::test]
    async fn test_multi_layer_cache_l1_only() {
        let redis_cache = match create_test_redis_cache().await {
            Ok(cache) => cache,
            Err(_) => {
                eprintln!("Skipping test: Redis not available");
                return;
            }
        };
        let config = MultiLayerCacheConfig {
            l1_enabled: true,
            l2_enabled: false,
            ..Default::default()
        };

        let cache = MultiLayerCache::new(redis_cache, config);

        let key = "test_l1_only";
        let value = serde_json::json!({"data": "test"});

        // Set and get
        cache
            .set(key, &value, Duration::from_secs(60))
            .await
            .unwrap();
        let retrieved = cache.get(key).await.unwrap();
        assert_eq!(retrieved, Some(value));

        // Delete
        cache.delete(key).await.unwrap();
        let retrieved = cache.get(key).await.unwrap();
        assert_eq!(retrieved, None);
    }

    #[tokio::test]
    async fn test_multi_layer_cache_both_layers() {
        let redis_cache = match create_test_redis_cache().await {
            Ok(cache) => cache,
            Err(_) => {
                eprintln!("Skipping test: Redis not available");
                return;
            }
        };
        let cache = MultiLayerCache::with_defaults(redis_cache);

        let key = "test_both_layers";
        let value = serde_json::json!({"data": "test"});

        // Set in both layers
        cache
            .set(key, &value, Duration::from_secs(60))
            .await
            .unwrap();

        // Should hit L1
        let retrieved = cache.get(key).await.unwrap();
        assert_eq!(retrieved, Some(value.clone()));

        // Clear L1
        cache.clear_l1();

        // Should hit L2 and populate L1
        let retrieved = cache.get(key).await.unwrap();
        assert_eq!(retrieved, Some(value.clone()));

        // Verify L1 was populated
        assert_eq!(cache.l1_size(), 1);

        // Clean up
        cache.delete(key).await.unwrap();
    }

    #[tokio::test]
    async fn test_multi_layer_cache_l2_fallback() {
        let redis_cache = match create_test_redis_cache().await {
            Ok(cache) => cache,
            Err(_) => {
                eprintln!("Skipping test: Redis not available");
                return;
            }
        };
        let cache = MultiLayerCache::with_defaults(redis_cache);

        let key = "test_l2_fallback";
        let value = serde_json::json!({"data": "test"});

        // Set in both layers
        cache
            .set(key, &value, Duration::from_secs(60))
            .await
            .unwrap();

        // Clear L1 to force L2 lookup
        cache.clear_l1();

        // Should fallback to L2
        let retrieved = cache.get(key).await.unwrap();
        assert_eq!(retrieved, Some(value));

        // Clean up
        cache.delete(key).await.unwrap();
    }

    #[tokio::test]
    async fn test_multi_layer_cache_set_nx() {
        let redis_cache = match create_test_redis_cache().await {
            Ok(cache) => cache,
            Err(_) => {
                eprintln!("Skipping test: Redis not available");
                return;
            }
        };
        let cache = MultiLayerCache::with_defaults(redis_cache);

        let key = "test_set_nx_multi";
        let value = serde_json::json!({"data": "test"});

        // Clean up first
        let _ = cache.delete(key).await;

        // First set_nx should succeed
        let result = cache
            .set_nx(key, &value, Duration::from_secs(60))
            .await
            .unwrap();
        assert!(result);

        // Second set_nx should fail
        let result = cache
            .set_nx(key, &value, Duration::from_secs(60))
            .await
            .unwrap();
        assert!(!result);

        // Clean up
        cache.delete(key).await.unwrap();
    }

    #[tokio::test]
    async fn test_multi_layer_cache_increment() {
        let redis_cache = match create_test_redis_cache().await {
            Ok(cache) => cache,
            Err(_) => {
                eprintln!("Skipping test: Redis not available");
                return;
            }
        };
        let cache = MultiLayerCache::with_defaults(redis_cache);

        let key = "test_increment_multi";

        // Clean up first
        let _ = cache.delete(key).await;

        // Increment
        let result = cache.increment(key, 1).await.unwrap();
        assert_eq!(result, 1);

        let result = cache.increment(key, 5).await.unwrap();
        assert_eq!(result, 6);

        // Clean up
        cache.delete(key).await.unwrap();
    }

    #[tokio::test]
    async fn test_multi_layer_cache_metrics() {
        let redis_cache = match create_test_redis_cache().await {
            Ok(cache) => cache,
            Err(_) => {
                eprintln!("Skipping test: Redis not available");
                return;
            }
        };
        let cache = MultiLayerCache::with_defaults(redis_cache);

        let key = "test_metrics_multi";
        let value = serde_json::json!({"data": "test"});

        // Generate some cache activity
        cache
            .set(key, &value, Duration::from_secs(60))
            .await
            .unwrap();
        let _ = cache.get(key).await;
        let _ = cache.get("nonexistent").await;

        // Check metrics
        let metrics = cache.metrics();
        assert!(metrics.hits() > 0);
        assert!(metrics.misses() > 0);

        // Check L1 metrics
        let l1_metrics = cache.l1_metrics();
        assert!(l1_metrics.set_operations() > 0);
    }

    #[tokio::test]
    async fn test_multi_layer_cache_l1_ttl() {
        let redis_cache = match create_test_redis_cache().await {
            Ok(cache) => cache,
            Err(_) => {
                eprintln!("Skipping test: Redis not available");
                return;
            }
        };
        let config = MultiLayerCacheConfig {
            l1_ttl: Duration::from_millis(100),
            ..Default::default()
        };

        let cache = MultiLayerCache::new(redis_cache, config);

        let key = "test_l1_ttl";
        let value = serde_json::json!({"data": "test"});

        // Set with longer TTL
        cache
            .set(key, &value, Duration::from_secs(60))
            .await
            .unwrap();

        // Should be in L1
        assert_eq!(cache.l1_size(), 1);

        // Wait for L1 to expire
        std::thread::sleep(Duration::from_millis(150));

        // L1 should be expired, but L2 should still have it (if Redis is available)
        let _ = cache.get(key).await;
    }
}
