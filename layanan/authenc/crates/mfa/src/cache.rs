//! MFA Cache for performance optimization
//!
//! Provides in-memory caching for MFA status lookups and OTP replay prevention.

use crate::policy::MfaStatus;
use authenc_core::error::Result;
use dashmap::DashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;

/// Cache entry with TTL
struct CacheEntry<T> {
    value: T,
    expires_at: Instant,
}

/// MFA cache for performance optimization
pub struct MfaCache {
    /// Cached MFA status per user
    status_cache: DashMap<Uuid, CacheEntry<MfaStatus>>,
    /// Recently used OTP codes for replay prevention
    used_otps: DashMap<String, Instant>,
    /// TTL for status cache entries
    status_ttl: Duration,
    /// TTL for OTP replay entries
    otp_ttl: Duration,
}

impl MfaCache {
    /// Create a new MFA cache
    pub fn new() -> Self {
        Self {
            status_cache: DashMap::new(),
            used_otps: DashMap::new(),
            status_ttl: Duration::from_secs(300), // 5 minutes
            otp_ttl: Duration::from_secs(90),     // 1.5x TOTP period
        }
    }

    /// Check if an OTP code was recently used (replay prevention)
    pub async fn is_otp_recently_used(&self, user_id: Uuid, code: &str) -> Result<bool> {
        let key = format!("{}:{}", user_id, code);
        if let Some(entry) = self.used_otps.get(&key) {
            Ok(Instant::now() < *entry)
        } else {
            Ok(false)
        }
    }

    /// Mark an OTP code as used
    pub async fn mark_otp_as_used(&self, user_id: Uuid, code: &str) -> Result<()> {
        let key = format!("{}:{}", user_id, code);
        self.used_otps.insert(key, Instant::now() + self.otp_ttl);
        Ok(())
    }

    /// Get cached MFA status for a user
    pub async fn get_mfa_status(&self, user_id: Uuid) -> Result<Option<MfaStatus>> {
        if let Some(entry) = self.status_cache.get(&user_id)
            && Instant::now() < entry.expires_at {
                return Ok(Some(entry.value.clone()));
            }
            // Entry expired, will be cleaned up
        Ok(None)
    }

    /// Cache MFA status for a user
    pub async fn cache_mfa_status(&self, user_id: Uuid, status: &MfaStatus) -> Result<()> {
        self.status_cache.insert(
            user_id,
            CacheEntry {
                value: status.clone(),
                expires_at: Instant::now() + self.status_ttl,
            },
        );
        Ok(())
    }

    /// Invalidate cached MFA status for a user
    pub async fn invalidate_mfa_status(&self, user_id: Uuid) -> Result<()> {
        self.status_cache.remove(&user_id);
        Ok(())
    }

    /// Batch get MFA status for multiple users
    pub async fn batch_get_mfa_status(
        &self,
        user_ids: &[Uuid],
    ) -> Result<Vec<(Uuid, Option<MfaStatus>)>> {
        let now = Instant::now();
        let results = user_ids
            .iter()
            .map(|id| {
                let status = self.status_cache.get(id).and_then(|entry| {
                    if now < entry.expires_at {
                        Some(entry.value.clone())
                    } else {
                        None
                    }
                });
                (*id, status)
            })
            .collect();
        Ok(results)
    }

    /// Batch cache MFA status for multiple users
    pub async fn batch_cache_mfa_status(&self, items: &[(Uuid, MfaStatus)]) -> Result<()> {
        let expires_at = Instant::now() + self.status_ttl;
        for (user_id, status) in items {
            self.status_cache.insert(
                *user_id,
                CacheEntry {
                    value: status.clone(),
                    expires_at,
                },
            );
        }
        Ok(())
    }
}

impl Default for MfaCache {
    fn default() -> Self {
        Self::new()
    }
}
