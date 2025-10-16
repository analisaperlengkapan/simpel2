//! MFA-specific caching implementation
//!
//! This module provides specialized caching for MFA operations including
//! user MFA status caching and OTP verification replay protection.

use super::{Cache, CacheKeys};
use crate::error::{AuthencError, Result};
use crate::services::mfa_service::MfaStatus;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// MFA cache entry for user MFA status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaCacheEntry {
    /// User ID
    pub user_id: Uuid,
    /// Whether MFA is enabled
    pub enabled: bool,
    /// When MFA was set up
    pub setup_at: Option<DateTime<Utc>>,
    /// Number of backup codes remaining
    pub backup_codes_remaining: i32,
    /// Last time MFA was used
    pub last_used: Option<DateTime<Utc>>,
    /// Cache timestamp for invalidation
    pub cached_at: DateTime<Utc>,
}

impl From<MfaStatus> for MfaCacheEntry {
    fn from(status: MfaStatus) -> Self {
        Self {
            user_id: Uuid::new_v4(), // Will be set by the caller
            enabled: status.enabled,
            setup_at: status.setup_at,
            backup_codes_remaining: status.backup_codes_remaining,
            last_used: status.last_used,
            cached_at: Utc::now(),
        }
    }
}

impl From<MfaCacheEntry> for MfaStatus {
    fn from(entry: MfaCacheEntry) -> Self {
        Self {
            enabled: entry.enabled,
            setup_at: entry.setup_at,
            backup_codes_remaining: entry.backup_codes_remaining,
            last_used: entry.last_used,
        }
    }
}

/// MFA verification result for caching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaVerificationResult {
    /// User ID
    pub user_id: Uuid,
    /// OTP code that was verified
    pub otp_code: String,
    /// Verification timestamp
    pub verified_at: DateTime<Utc>,
    /// Whether verification was successful
    pub success: bool,
}

/// MFA-specific cache operations
pub struct MfaCache {
    /// Underlying cache implementation
    cache: Arc<dyn Cache>,
    /// MFA cache TTL
    mfa_cache_ttl: Duration,
    /// OTP verification cache TTL (for replay protection)
    otp_verification_ttl: Duration,
}

impl MfaCache {
    /// Create a new MFA cache instance
    pub fn new(
        cache: Arc<dyn Cache>,
        mfa_cache_ttl: Duration,
        otp_verification_ttl: Duration,
    ) -> Self {
        Self {
            cache,
            mfa_cache_ttl,
            otp_verification_ttl,
        }
    }

    /// Cache user MFA status
    pub async fn cache_mfa_status(&self, user_id: Uuid, status: &MfaStatus) -> Result<()> {
        let mut entry = MfaCacheEntry::from(status.clone());
        entry.user_id = user_id;

        let cache_key = CacheKeys::mfa_status(&user_id.to_string());
        let value = serde_json::to_value(&entry)
            .map_err(|e| AuthencError::internal(format!("Failed to serialize MFA cache entry: {}", e)))?;

        self.cache
            .set(&cache_key, &value, self.mfa_cache_ttl)
            .await?;

        debug!("Cached MFA status for user: {}", user_id);
        Ok(())
    }

    /// Get cached MFA status for a user
    pub async fn get_mfa_status(&self, user_id: Uuid) -> Result<Option<MfaStatus>> {
        let cache_key = CacheKeys::mfa_status(&user_id.to_string());

        match self.cache.get(&cache_key).await? {
            Some(value) => {
                let entry: MfaCacheEntry = serde_json::from_value(value)
                    .map_err(|e| AuthencError::internal(format!("Failed to deserialize MFA cache entry: {}", e)))?;
                debug!("Cache hit for MFA status: {}", user_id);
                Ok(Some(entry.into()))
            }
            None => {
                debug!("Cache miss for MFA status: {}", user_id);
                Ok(None)
            }
        }
    }

    /// Invalidate cached MFA status for a user
    pub async fn invalidate_mfa_status(&self, user_id: Uuid) -> Result<()> {
        let cache_key = CacheKeys::mfa_status(&user_id.to_string());

        self.cache.delete(&cache_key).await?;

        info!("Invalidated MFA status cache for user: {}", user_id);
        Ok(())
    }

    /// Check if an OTP code has been recently used (replay protection)
    pub async fn is_otp_recently_used(&self, user_id: Uuid, otp_code: &str) -> Result<bool> {
        let cache_key = CacheKeys::otp_verification(&user_id.to_string(), otp_code);

        let exists = self.cache.exists(&cache_key).await?;

        if exists {
            warn!("OTP replay attempt detected for user: {} with code: {}", user_id, otp_code);
        }

        Ok(exists)
    }

    /// Mark an OTP code as used (for replay protection)
    pub async fn mark_otp_as_used(&self, user_id: Uuid, otp_code: &str) -> Result<()> {
        let verification_result = MfaVerificationResult {
            user_id,
            otp_code: otp_code.to_string(),
            verified_at: Utc::now(),
            success: true,
        };

        let cache_key = CacheKeys::otp_verification(&user_id.to_string(), otp_code);

        // Use set_nx to ensure atomic operation (only set if not exists)
        let value = serde_json::to_value(&verification_result)
            .map_err(|e| AuthencError::internal(format!("Failed to serialize verification result: {}", e)))?;
        let was_set = self.cache
            .set_nx(&cache_key, &value, self.otp_verification_ttl)
            .await?;

        if was_set {
            debug!("Marked OTP as used for user: {} with code: {}", user_id, otp_code);
            Ok(())
        } else {
            warn!("OTP was already marked as used for user: {} with code: {}", user_id, otp_code);
            Err(AuthencError::invalid_otp_code())
        }
    }

    /// Get MFA rate limiting information
    pub async fn get_mfa_rate_limit(&self, user_id: Uuid, operation: &str) -> Result<Option<i64>> {
        let cache_key = CacheKeys::mfa_rate_limit(&user_id.to_string(), operation);

        match self.cache.get(&cache_key).await? {
            Some(value) => {
                let count: i64 = serde_json::from_value(value)
                    .map_err(|e| AuthencError::internal(format!("Failed to deserialize rate limit count: {}", e)))?;
                debug!("MFA rate limit check for user {} operation {}: {}", user_id, operation, count);
                Ok(Some(count))
            }
            None => {
                debug!("No MFA rate limit data for user {} operation {}", user_id, operation);
                Ok(None)
            }
        }
    }

    /// Increment MFA rate limiting counter
    pub async fn increment_mfa_rate_limit(
        &self,
        user_id: Uuid,
        operation: &str,
        window: Duration,
    ) -> Result<i64> {
        let cache_key = CacheKeys::mfa_rate_limit(&user_id.to_string(), operation);

        // Increment counter
        let new_count = self.cache.increment(&cache_key, 1).await?;

        // Set expiration if this is the first increment
        if new_count == 1 {
            self.cache.expire(&cache_key, window).await?;
        }

        debug!("Incremented MFA rate limit for user {} operation {}: {}", user_id, operation, new_count);
        Ok(new_count)
    }

    /// Reset MFA rate limiting counter
    pub async fn reset_mfa_rate_limit(&self, user_id: Uuid, operation: &str) -> Result<()> {
        let cache_key = CacheKeys::mfa_rate_limit(&user_id.to_string(), operation);

        self.cache.delete(&cache_key).await?;

        info!("Reset MFA rate limit for user {} operation {}", user_id, operation);
        Ok(())
    }

    /// Batch get MFA status for multiple users
    pub async fn batch_get_mfa_status(&self, user_ids: &[Uuid]) -> Result<Vec<(Uuid, Option<MfaStatus>)>> {
        if user_ids.is_empty() {
            return Ok(vec![]);
        }

        let mut results = Vec::with_capacity(user_ids.len());
        for &user_id in user_ids {
            let cache_key = CacheKeys::mfa_status(&user_id.to_string());
            let status = match self.cache.get(&cache_key).await? {
                Some(value) => {
                    let entry: MfaCacheEntry = serde_json::from_value(value)
                        .map_err(|e| AuthencError::internal(format!("Failed to deserialize MFA cache entry: {}", e)))?;
                    Some(entry.into())
                }
                None => None,
            };
            results.push((user_id, status));
        }

        debug!("Batch retrieved MFA status for {} users", user_ids.len());
        Ok(results)
    }

    /// Batch cache MFA status for multiple users
    pub async fn batch_cache_mfa_status(&self, statuses: &[(Uuid, MfaStatus)]) -> Result<()> {
        if statuses.is_empty() {
            return Ok(());
        }

        for (user_id, status) in statuses {
            let mut entry = MfaCacheEntry::from(status.clone());
            entry.user_id = *user_id;
            let cache_key = CacheKeys::mfa_status(&user_id.to_string());
            let value = serde_json::to_value(&entry)
                .map_err(|e| AuthencError::internal(format!("Failed to serialize MFA cache entry: {}", e)))?;
            self.cache.set(&cache_key, &value, self.mfa_cache_ttl).await?;
        }

        debug!("Batch cached MFA status for {} users", statuses.len());
        Ok(())
    }

    /// Get cache statistics for monitoring
    pub async fn get_cache_stats(&self) -> Result<MfaCacheStats> {
        // This is a simplified implementation
        // In a real implementation, you might want to track these metrics
        Ok(MfaCacheStats {
            mfa_status_cache_hits: 0,
            mfa_status_cache_misses: 0,
            otp_replay_blocks: 0,
            rate_limit_checks: 0,
        })
    }

    /// Warm up cache with frequently accessed MFA statuses
    pub async fn warm_up_cache(&self, user_ids: &[Uuid]) -> Result<()> {
        // This would typically be called during application startup
        // to pre-populate cache with frequently accessed data
        debug!("Warming up MFA cache for {} users", user_ids.len());

        // Implementation would fetch from database and populate cache
        // For now, this is a placeholder
        Ok(())
    }
}

/// Cache statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaCacheStats {
    /// Number of MFA status cache hits
    pub mfa_status_cache_hits: u64,
    /// Number of MFA status cache misses
    pub mfa_status_cache_misses: u64,
    /// Number of OTP replay attempts blocked
    pub otp_replay_blocks: u64,
    /// Number of rate limit checks performed
    pub rate_limit_checks: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::cache::redis_cache::RedisCache;
    use crate::config::RedisConfig;

    #[tokio::test]
    #[ignore] // Requires Redis
    async fn test_mfa_cache_operations() {
        let redis_config = RedisConfig {
            enabled: true,
            url: "redis://localhost:6379/15".to_string(),
            ..Default::default()
        };

        let redis_cache = Arc::new(RedisCache::new(&redis_config).await.unwrap());
        let mfa_cache = MfaCache::new(
            redis_cache,
            Duration::from_secs(300),
            Duration::from_secs(90),
        );

        let user_id = Uuid::new_v4();
        let mfa_status = MfaStatus {
            enabled: true,
            setup_at: Some(Utc::now()),
            backup_codes_remaining: 8,
            last_used: None,
        };

        // Test caching MFA status
        mfa_cache.cache_mfa_status(user_id, &mfa_status).await.unwrap();

        // Test retrieving cached MFA status
        let cached_status = mfa_cache.get_mfa_status(user_id).await.unwrap();
        assert!(cached_status.is_some());
        assert_eq!(cached_status.unwrap().enabled, true);

        // Test OTP replay protection
        let otp_code = "123456";

        // First check should return false (not used)
        assert!(!mfa_cache.is_otp_recently_used(user_id, otp_code).await.unwrap());

        // Mark as used
        mfa_cache.mark_otp_as_used(user_id, otp_code).await.unwrap();

        // Second check should return true (already used)
        assert!(mfa_cache.is_otp_recently_used(user_id, otp_code).await.unwrap());

        // Test rate limiting
        let operation = "verify";
        let count = mfa_cache.increment_mfa_rate_limit(user_id, operation, Duration::from_secs(60)).await.unwrap();
        assert_eq!(count, 1);

        let count = mfa_cache.increment_mfa_rate_limit(user_id, operation, Duration::from_secs(60)).await.unwrap();
        assert_eq!(count, 2);

        // Clean up
        mfa_cache.invalidate_mfa_status(user_id).await.unwrap();
        mfa_cache.reset_mfa_rate_limit(user_id, operation).await.unwrap();
    }
}
