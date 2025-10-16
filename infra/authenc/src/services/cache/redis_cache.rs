//! Redis cache implementation
//!
//! This module provides a Redis-based cache implementation for high-performance
//! caching with support for TTL, atomic operations, and connection pooling.

use super::{Cache, CacheConfig};
use crate::config::RedisConfig;
use crate::error::{AuthencError, Result};
use async_trait::async_trait;
use redis::{aio::ConnectionManager, AsyncCommands, Client, RedisResult};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, warn};

/// Redis cache implementation
pub struct RedisCache {
    /// Redis connection manager for connection pooling
    connection_manager: Arc<ConnectionManager>,
    /// Cache configuration
    config: CacheConfig,
}

impl RedisCache {
    /// Create a new Redis cache instance
    pub async fn new(redis_config: &RedisConfig) -> Result<Self> {
        // Create Redis client
        let client = Client::open(redis_config.url.as_str())
            .map_err(|e| AuthencError::internal(format!("Failed to create Redis client: {}", e)))?;

        // Create connection manager for connection pooling
        let connection_manager = ConnectionManager::new(client)
            .await
            .map_err(|e| AuthencError::internal(format!("Failed to create Redis connection manager: {}", e)))?;

        let config = CacheConfig {
            default_ttl: Duration::from_secs(redis_config.default_ttl),
            mfa_cache_ttl: Duration::from_secs(redis_config.mfa_cache_ttl),
            otp_verification_ttl: Duration::from_secs(redis_config.otp_verification_ttl),
        };

        debug!("Redis cache initialized with URL: {}", redis_config.url);

        Ok(Self {
            connection_manager: Arc::new(connection_manager),
            config,
        })
    }

    /// Get a connection from the pool
    async fn get_connection(&self) -> Result<ConnectionManager> {
        Ok(self.connection_manager.as_ref().clone())
    }

    /// Convert Duration to seconds for Redis TTL
    fn duration_to_seconds(&self, duration: Duration) -> u64 {
        duration.as_secs()
    }

    /// Handle Redis errors and convert to AuthencError
    fn handle_redis_error(&self, error: redis::RedisError) -> AuthencError {
        error!("Redis operation failed: {}", error);
        AuthencError::internal(format!("Cache operation failed: {}", error))
    }
}

#[async_trait]
impl Cache for RedisCache {
    async fn get(&self, key: &str) -> Result<Option<serde_json::Value>> {
        let mut conn = self.get_connection().await?;

        let result: RedisResult<Option<String>> = conn.get(key).await;

        match result {
            Ok(Some(data)) => {
                debug!("Cache hit for key: {}", key);
                let value: serde_json::Value = serde_json::from_str(&data)
                    .map_err(|e| AuthencError::internal(format!("Failed to deserialize cache value: {}", e)))?;
                Ok(Some(value))
            }
            Ok(None) => {
                debug!("Cache miss for key: {}", key);
                Ok(None)
            }
            Err(e) => {
                warn!("Cache get failed for key {}: {}", key, e);
                Err(self.handle_redis_error(e))
            }
        }
    }

    async fn set(&self, key: &str, value: &serde_json::Value, ttl: Duration) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let serialized = serde_json::to_string(value)
            .map_err(|e| AuthencError::internal(format!("Failed to serialize cache value: {}", e)))?;
        let ttl_seconds = self.duration_to_seconds(ttl);

        let result: RedisResult<()> = conn.set_ex(key, serialized, ttl_seconds).await;

        match result {
            Ok(()) => {
                debug!("Cache set for key: {} with TTL: {}s", key, ttl_seconds);
                Ok(())
            }
            Err(e) => {
                warn!("Cache set failed for key {}: {}", key, e);
                Err(self.handle_redis_error(e))
            }
        }
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let mut conn = self.get_connection().await?;

        let result: RedisResult<i32> = conn.del(key).await;

        match result {
            Ok(_) => {
                debug!("Cache delete for key: {}", key);
                Ok(())
            }
            Err(e) => {
                warn!("Cache delete failed for key {}: {}", key, e);
                Err(self.handle_redis_error(e))
            }
        }
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let mut conn = self.get_connection().await?;

        let result: RedisResult<bool> = conn.exists(key).await;

        match result {
            Ok(exists) => {
                debug!("Cache exists check for key {}: {}", key, exists);
                Ok(exists)
            }
            Err(e) => {
                warn!("Cache exists check failed for key {}: {}", key, e);
                Err(self.handle_redis_error(e))
            }
        }
    }

    async fn expire(&self, key: &str, ttl: Duration) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let ttl_seconds = self.duration_to_seconds(ttl);

        let result: RedisResult<bool> = conn.expire(key, ttl_seconds as i64).await;

        match result {
            Ok(_) => {
                debug!("Cache expire set for key: {} with TTL: {}s", key, ttl_seconds);
                Ok(())
            }
            Err(e) => {
                warn!("Cache expire failed for key {}: {}", key, e);
                Err(self.handle_redis_error(e))
            }
        }
    }

    async fn increment(&self, key: &str, delta: i64) -> Result<i64> {
        let mut conn = self.get_connection().await?;

        let result: RedisResult<i64> = conn.incr(key, delta).await;

        match result {
            Ok(new_value) => {
                debug!("Cache increment for key: {} by {} = {}", key, delta, new_value);
                Ok(new_value)
            }
            Err(e) => {
                warn!("Cache increment failed for key {}: {}", key, e);
                Err(self.handle_redis_error(e))
            }
        }
    }

    async fn set_nx(&self, key: &str, value: &serde_json::Value, ttl: Duration) -> Result<bool> {
        let mut conn = self.get_connection().await?;
        let serialized = serde_json::to_string(value)
            .map_err(|e| AuthencError::internal(format!("Failed to serialize cache value: {}", e)))?;
        let ttl_seconds = self.duration_to_seconds(ttl);

        // Use SET with NX (only if not exists) and EX (expiration)
        let result: RedisResult<Option<String>> = redis::cmd("SET")
            .arg(key)
            .arg(serialized)
            .arg("NX")
            .arg("EX")
            .arg(ttl_seconds)
            .query_async(&mut conn)
            .await;

        match result {
            Ok(Some(_)) => {
                debug!("Cache set_nx succeeded for key: {} with TTL: {}s", key, ttl_seconds);
                Ok(true)
            }
            Ok(None) => {
                debug!("Cache set_nx failed (key exists) for key: {}", key);
                Ok(false)
            }
            Err(e) => {
                warn!("Cache set_nx failed for key {}: {}", key, e);
                Err(self.handle_redis_error(e))
            }
        }
    }
}

impl RedisCache {
    /// Get MFA cache TTL
    pub fn mfa_cache_ttl(&self) -> Duration {
        self.config.mfa_cache_ttl
    }

    /// Get OTP verification cache TTL
    pub fn otp_verification_ttl(&self) -> Duration {
        self.config.otp_verification_ttl
    }

    /// Get default cache TTL
    pub fn default_ttl(&self) -> Duration {
        self.config.default_ttl
    }

    /// Batch get multiple keys
    pub async fn mget(&self, keys: &[String]) -> Result<Vec<Option<serde_json::Value>>> {
        if keys.is_empty() {
            return Ok(vec![]);
        }

        let mut conn = self.get_connection().await?;

        let result: RedisResult<Vec<Option<String>>> = conn.get(keys).await;

        match result {
            Ok(values) => {
                let mut results = Vec::with_capacity(values.len());
                for (i, value) in values.into_iter().enumerate() {
                    match value {
                        Some(data) => {
                            match serde_json::from_str(&data) {
                                Ok(deserialized) => results.push(Some(deserialized)),
                                Err(e) => {
                                    warn!("Failed to deserialize value for key {}: {}", keys[i], e);
                                    results.push(None);
                                }
                            }
                        }
                        None => results.push(None),
                    }
                }
                debug!("Batch get completed for {} keys", keys.len());
                Ok(results)
            }
            Err(e) => {
                warn!("Batch get failed for keys: {:?}, error: {}", keys, e);
                Err(self.handle_redis_error(e))
            }
        }
    }

    /// Batch set multiple key-value pairs
    pub async fn mset(&self, items: &[(String, serde_json::Value)], ttl: Duration) -> Result<()> {
        if items.is_empty() {
            return Ok(());
        }

        let mut conn = self.get_connection().await?;
        let ttl_seconds = self.duration_to_seconds(ttl);

        // Use pipeline for batch operations
        let mut pipe = redis::pipe();

        for (key, value) in items {
            let serialized = serde_json::to_string(value)
                .map_err(|e| AuthencError::internal(format!("Failed to serialize cache value: {}", e)))?;
            pipe.set_ex(key, serialized, ttl_seconds);
        }

        let result: RedisResult<()> = pipe.query_async(&mut conn).await;

        match result {
            Ok(()) => {
                debug!("Batch set completed for {} items with TTL: {}s", items.len(), ttl_seconds);
                Ok(())
            }
            Err(e) => {
                warn!("Batch set failed for {} items: {}", items.len(), e);
                Err(self.handle_redis_error(e))
            }
        }
    }

    /// Health check for Redis connection
    pub async fn health_check(&self) -> Result<()> {
        let mut conn = self.get_connection().await?;

        let result: RedisResult<String> = redis::cmd("PING").query_async(&mut conn).await;

        match result {
            Ok(response) if response == "PONG" => {
                debug!("Redis health check passed");
                Ok(())
            }
            Ok(response) => {
                warn!("Redis health check returned unexpected response: {}", response);
                Err(AuthencError::internal("Redis health check failed"))
            }
            Err(e) => {
                error!("Redis health check failed: {}", e);
                Err(self.handle_redis_error(e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestData {
        id: u32,
        name: String,
    }

    // Note: These tests require a running Redis instance
    // They are integration tests and should be run with `cargo test --ignored`

    #[tokio::test]
    #[ignore]
    async fn test_redis_cache_operations() {
        let config = RedisConfig {
            enabled: true,
            url: "redis://localhost:6379/15".to_string(), // Use test database
            ..Default::default()
        };

        let cache = RedisCache::new(&config).await.unwrap();
        let test_key = "test:cache:operations";
        let test_data = TestData {
            id: 123,
            name: "test".to_string(),
        };

        // Test set and get
        let test_value = serde_json::to_value(&test_data).unwrap();
        cache.set(test_key, &test_value, Duration::from_secs(60)).await.unwrap();
        let retrieved = cache.get(test_key).await.unwrap();
        let retrieved_data: Option<TestData> = retrieved.map(|v| serde_json::from_value(v).unwrap()).unwrap();
        assert_eq!(retrieved_data, Some(test_data.clone()));

        // Test exists
        assert!(cache.exists(test_key).await.unwrap());

        // Test delete
        cache.delete(test_key).await.unwrap();
        let retrieved = cache.get(test_key).await.unwrap();
        assert_eq!(retrieved, None);
    }

    #[tokio::test]
    #[ignore]
    async fn test_redis_cache_set_nx() {
        let config = RedisConfig {
            enabled: true,
            url: "redis://localhost:6379/15".to_string(),
            ..Default::default()
        };

        let cache = RedisCache::new(&config).await.unwrap();
        let test_key = "test:cache:set_nx";
        let test_data = TestData {
            id: 456,
            name: "set_nx_test".to_string(),
        };

        // Clean up first
        let _ = cache.delete(test_key).await;

        // First set_nx should succeed
        let test_value = serde_json::to_value(&test_data).unwrap();
        let result = cache.set_nx(test_key, &test_value, Duration::from_secs(60)).await.unwrap();
        assert!(result);

        // Second set_nx should fail (key exists)
        let result = cache.set_nx(test_key, &test_value, Duration::from_secs(60)).await.unwrap();
        assert!(!result);

        // Clean up
        cache.delete(test_key).await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_redis_health_check() {
        let config = RedisConfig {
            enabled: true,
            url: "redis://localhost:6379/15".to_string(),
            ..Default::default()
        };

        let cache = RedisCache::new(&config).await.unwrap();
        cache.health_check().await.unwrap();
    }
}
