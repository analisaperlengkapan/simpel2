//! Caching services for performance optimization
//!
//! This module provides caching implementations for various data types
//! to improve performance and reduce database load.

#[cfg(feature = "kafka")]
pub mod event_consumer;
#[cfg(feature = "cache")]
pub mod in_memory_cache;
#[cfg(feature = "kafka")]
pub mod invalidation;
pub mod metrics;
pub mod mfa_cache;
#[cfg(all(feature = "cache", feature = "redis-cache"))]
pub mod multi_layer_cache;
#[cfg(feature = "redis-cache")]
pub mod redis_cache;

#[cfg(feature = "kafka")]
pub use event_consumer::{EventConsumerConfig, EventConsumerStats, EventDrivenCacheInvalidator};
#[cfg(feature = "cache")]
pub use in_memory_cache::InMemoryCache;
#[cfg(feature = "kafka")]
pub use invalidation::{
    CacheInvalidationService, CacheWarmingService, InvalidationEvent, InvalidationStats,
};
pub use metrics::{CacheMetrics, CacheMetricsSnapshot, OperationTimer};
pub use mfa_cache::{MfaCache, MfaCacheEntry, MfaVerificationResult};
#[cfg(all(feature = "cache", feature = "redis-cache"))]
pub use multi_layer_cache::{MultiLayerCache, MultiLayerCacheConfig};
#[cfg(feature = "redis-cache")]
pub use redis_cache::RedisCache;

use async_trait::async_trait;
use authenc_types::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Generic cache trait for different cache implementations
#[async_trait]
pub trait Cache: Send + Sync {
    /// Get a value from the cache
    async fn get(&self, key: &str) -> Result<Option<serde_json::Value>>;

    /// Set a value in the cache with TTL
    async fn set(&self, key: &str, value: &serde_json::Value, ttl: Duration) -> Result<()>;

    /// Delete a value from the cache
    async fn delete(&self, key: &str) -> Result<()>;

    /// Check if a key exists in the cache
    async fn exists(&self, key: &str) -> Result<bool>;

    /// Set expiration for a key
    async fn expire(&self, key: &str, ttl: Duration) -> Result<()>;

    /// Increment a counter value
    async fn increment(&self, key: &str, delta: i64) -> Result<i64>;

    /// Set a value only if it doesn't exist (atomic operation)
    async fn set_nx(&self, key: &str, value: &serde_json::Value, ttl: Duration) -> Result<bool>;
}

/// Cache key prefixes for different data types
pub struct CacheKeys;

impl CacheKeys {
    /// MFA status cache key prefix
    pub const MFA_STATUS: &'static str = "mfa:status";

    /// OTP verification cache key prefix (for replay protection)
    pub const OTP_VERIFICATION: &'static str = "mfa:otp_used";

    /// User session cache key prefix
    pub const USER_SESSION: &'static str = "session:user";

    /// Rate limiting cache key prefix
    pub const RATE_LIMIT: &'static str = "rate_limit";

    /// MFA rate limiting cache key prefix
    pub const MFA_RATE_LIMIT: &'static str = "mfa:rate_limit";

    /// Generate MFA status cache key
    pub fn mfa_status(user_id: &str) -> String {
        format!("{}:{}", Self::MFA_STATUS, user_id)
    }

    /// Generate OTP verification cache key
    pub fn otp_verification(user_id: &str, otp_code: &str) -> String {
        format!("{}:{}:{}", Self::OTP_VERIFICATION, user_id, otp_code)
    }

    /// Generate user session cache key
    pub fn user_session(session_id: &str) -> String {
        format!("{}:{}", Self::USER_SESSION, session_id)
    }

    /// Generate rate limit cache key
    pub fn rate_limit(identifier: &str, endpoint: &str) -> String {
        format!("{}:{}:{}", Self::RATE_LIMIT, identifier, endpoint)
    }

    /// Generate MFA rate limit cache key
    pub fn mfa_rate_limit(user_id: &str, operation: &str) -> String {
        format!("{}:{}:{}", Self::MFA_RATE_LIMIT, user_id, operation)
    }
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Default TTL for cached items
    pub default_ttl: Duration,
    /// MFA cache TTL
    pub mfa_cache_ttl: Duration,
    /// OTP verification cache TTL
    pub otp_verification_ttl: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            default_ttl: Duration::from_secs(3600),        // 1 hour
            mfa_cache_ttl: Duration::from_secs(300),       // 5 minutes
            otp_verification_ttl: Duration::from_secs(90), // 1.5 minutes
        }
    }
}
