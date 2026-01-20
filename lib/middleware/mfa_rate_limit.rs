//! MFA-specific rate limiting middleware with progressive delays and account lockout protection

use std::{
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    body::Body,
    extract::{ConnectInfo, Request, State},
    http::{Response, StatusCode},
    middleware::Next,
};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use tokio::sync::RwLock;
use tracing::{debug, error, warn};
use uuid::Uuid;

use super::error::AuthencError;

/// Configuration for MFA-specific rate limiting
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MfaRateLimitConfig {
    /// Maximum MFA verification attempts per minute per IP
    pub max_attempts_per_minute_per_ip: u32,
    /// Maximum MFA verification attempts per minute per user
    pub max_attempts_per_minute_per_user: u32,
    /// Maximum MFA setup attempts per hour per IP
    pub max_setup_attempts_per_hour_per_ip: u32,
    /// Progressive delay base in milliseconds
    pub progressive_delay_base_ms: u64,
    /// Maximum progressive delay in milliseconds
    pub progressive_delay_max_ms: u64,
    /// Account lockout threshold (failed attempts)
    pub account_lockout_threshold: u32,
    /// Account lockout duration in minutes
    pub account_lockout_duration_minutes: u32,
    /// Whether to enable progressive delays
    pub enable_progressive_delays: bool,
    /// Whether to enable account lockout
    pub enable_account_lockout: bool,
}

impl Default for MfaRateLimitConfig {
    fn default() -> Self {
        Self {
            max_attempts_per_minute_per_ip: 10,
            max_attempts_per_minute_per_user: 5,
            max_setup_attempts_per_hour_per_ip: 3,
            progressive_delay_base_ms: 2000, // 2 seconds base delay
            progressive_delay_max_ms: 30000, // 30 seconds max delay
            account_lockout_threshold: 5,
            account_lockout_duration_minutes: 15,
            enable_progressive_delays: true,
            enable_account_lockout: true,
        }
    }
}

/// Rate limit counter with timestamp
#[derive(Debug)]
struct RateLimitCounter {
    count: AtomicU32,
    window_start: Instant,
    failed_attempts: AtomicU32,
}

impl RateLimitCounter {
    fn new() -> Self {
        Self {
            count: AtomicU32::new(0),
            window_start: Instant::now(),
            failed_attempts: AtomicU32::new(0),
        }
    }

    fn reset_if_expired(&mut self, window_duration: Duration) {
        let now = Instant::now();
        if now.duration_since(self.window_start) >= window_duration {
            self.count.store(0, Ordering::Relaxed);
            self.window_start = now;
        }
    }

    fn increment(&self) -> u32 {
        self.count.fetch_add(1, Ordering::Relaxed) + 1
    }

    fn increment_failed(&self) -> u32 {
        self.failed_attempts.fetch_add(1, Ordering::Relaxed) + 1
    }

    fn get_count(&self) -> u32 {
        self.count.load(Ordering::Relaxed)
    }

    fn get_failed_count(&self) -> u32 {
        self.failed_attempts.load(Ordering::Relaxed)
    }
}

/// Account lockout information
#[derive(Debug, Clone)]
pub struct AccountLockout {
    /// Time until the account lockout expires
    pub locked_until: Instant,
    /// Number of failed attempts that triggered the lockout
    pub failed_attempts: u32,
    /// Reason for the account lockout
    pub lockout_reason: String,
}

/// MFA rate limiter state
#[derive(Clone)]
pub struct MfaRateLimiterState {
    config: MfaRateLimitConfig,
    // IP-based counters for different MFA operations
    ip_verify_counters: Arc<DashMap<String, RwLock<RateLimitCounter>>>,
    ip_setup_counters: Arc<DashMap<String, RwLock<RateLimitCounter>>>,
    // User-based counters
    user_verify_counters: Arc<DashMap<Uuid, RwLock<RateLimitCounter>>>,
    // Account lockout tracking
    locked_accounts: Arc<DashMap<Uuid, AccountLockout>>,
    // Progressive delay tracking per IP
    progressive_delays: Arc<DashMap<String, AtomicU64>>,
}

impl MfaRateLimiterState {
    /// Create a new MFA rate limiter state
    pub fn new(config: MfaRateLimitConfig) -> Self {
        let state = Self {
            config,
            ip_verify_counters: Arc::new(DashMap::new()),
            ip_setup_counters: Arc::new(DashMap::new()),
            user_verify_counters: Arc::new(DashMap::new()),
            locked_accounts: Arc::new(DashMap::new()),
            progressive_delays: Arc::new(DashMap::new()),
        };

        // Spawn cleanup task
        let cleanup_state = state.clone();
        tokio::spawn(async move {
            cleanup_state.cleanup_task().await;
        });

        state
    }

    /// Background cleanup task to remove expired entries
    async fn cleanup_task(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes
        loop {
            interval.tick().await;
            self.cleanup_expired_entries().await;
        }
    }

    /// Clean up expired rate limit counters and lockouts
    async fn cleanup_expired_entries(&self) {
        let now = Instant::now();

        // Clean up expired account lockouts
        self.locked_accounts
            .retain(|_, lockout| now < lockout.locked_until);

        // Clean up old progressive delay entries (older than 1 hour)
        self.progressive_delays.retain(|_, delay_entry| {
            // Keep entries that have been accessed recently
            delay_entry.load(Ordering::Relaxed) > 0
        });

        debug!("Cleaned up expired MFA rate limit entries");
    }

    /// Check if an account is currently locked
    pub fn is_account_locked(&self, user_id: Uuid) -> Option<AccountLockout> {
        self.locked_accounts
            .get(&user_id)
            .map(|entry| entry.clone())
    }

    /// Lock an account due to excessive failed MFA attempts
    pub fn lock_account(&self, user_id: Uuid, reason: String) {
        if !self.config.enable_account_lockout {
            return;
        }

        let lockout_duration =
            Duration::from_secs(self.config.account_lockout_duration_minutes as u64 * 60);
        let locked_until = Instant::now() + lockout_duration;

        let lockout = AccountLockout {
            locked_until,
            failed_attempts: self.config.account_lockout_threshold,
            lockout_reason: reason,
        };

        self.locked_accounts.insert(user_id, lockout);
        warn!(%user_id, "Account locked due to excessive MFA failures");
    }

    /// Unlock an account (admin override)
    pub fn unlock_account(&self, user_id: Uuid) -> bool {
        self.locked_accounts.remove(&user_id).is_some()
    }

    /// Check MFA verification rate limits
    pub async fn check_mfa_verify_rate_limit(
        &self,
        ip: &str,
        user_id: Option<Uuid>,
    ) -> Result<(), AuthencError> {
        // Check IP-based rate limit
        self.check_ip_verify_rate_limit(ip).await?;

        // Check user-based rate limit if user_id is provided
        if let Some(user_id) = user_id {
            self.check_user_verify_rate_limit(user_id).await?;

            // Check if account is locked
            if let Some(lockout) = self.is_account_locked(user_id) {
                if Instant::now() < lockout.locked_until {
                    return Err(AuthencError::AccountLocked {
                        reason: lockout.lockout_reason,
                        locked_until: lockout.locked_until,
                    });
                } else {
                    // Lockout expired, remove it
                    self.locked_accounts.remove(&user_id);
                }
            }
        }

        Ok(())
    }

    /// Check MFA setup rate limits
    pub async fn check_mfa_setup_rate_limit(&self, ip: &str) -> Result<(), AuthencError> {
        let counter_ref = self
            .ip_setup_counters
            .entry(ip.to_string())
            .or_insert_with(|| RwLock::new(RateLimitCounter::new()));

        let mut counter = counter_ref.write().await;
        counter.reset_if_expired(Duration::from_secs(3600)); // 1 hour window

        let count = counter.increment();
        if count > self.config.max_setup_attempts_per_hour_per_ip {
            warn!(%ip, count, "MFA setup rate limit exceeded");
            return Err(AuthencError::RateLimitExceeded);
        }

        debug!(%ip, count, "MFA setup request within rate limit");
        Ok(())
    }

    /// Record a failed MFA attempt and check for account lockout
    pub async fn record_failed_mfa_attempt(
        &self,
        ip: &str,
        user_id: Uuid,
    ) -> Result<(), AuthencError> {
        // Record failed attempt for user
        let user_counter_ref = self
            .user_verify_counters
            .entry(user_id)
            .or_insert_with(|| RwLock::new(RateLimitCounter::new()));

        let user_counter = user_counter_ref.write().await;
        let failed_count = user_counter.increment_failed();

        // Check if we should lock the account
        if self.config.enable_account_lockout
            && failed_count >= self.config.account_lockout_threshold
        {
            drop(user_counter); // Release the lock before calling lock_account
            self.lock_account(
                user_id,
                format!("Excessive failed MFA attempts: {}", failed_count),
            );
            return Err(AuthencError::AccountLocked {
                reason: "Too many failed MFA attempts".to_string(),
                locked_until: Instant::now()
                    + Duration::from_secs(self.config.account_lockout_duration_minutes as u64 * 60),
            });
        }

        // Update progressive delay for IP
        if self.config.enable_progressive_delays {
            self.update_progressive_delay(ip, failed_count);
        }

        Ok(())
    }

    /// Calculate and apply progressive delay
    pub async fn apply_progressive_delay(&self, ip: &str) {
        if !self.config.enable_progressive_delays {
            return;
        }

        let delay_ms = self
            .progressive_delays
            .get(ip)
            .map(|entry| entry.load(Ordering::Relaxed))
            .unwrap_or(0);

        if delay_ms > 0 {
            let delay = Duration::from_millis(delay_ms.min(self.config.progressive_delay_max_ms));
            debug!(%ip, delay_ms, "Applying progressive delay for MFA request");
            tokio::time::sleep(delay).await;
        }
    }

    /// Update progressive delay for an IP based on failed attempts
    fn update_progressive_delay(&self, ip: &str, failed_attempts: u32) {
        let delay_entry = self
            .progressive_delays
            .entry(ip.to_string())
            .or_insert_with(|| AtomicU64::new(0));

        // Calculate exponential backoff: base_delay * 2^(failed_attempts - 1)
        let multiplier = 1u64 << (failed_attempts.saturating_sub(1).min(10)); // Cap at 2^10
        let delay_ms = self
            .config
            .progressive_delay_base_ms
            .saturating_mul(multiplier)
            .min(self.config.progressive_delay_max_ms);

        delay_entry.store(delay_ms, Ordering::Relaxed);
    }

    /// Reset progressive delay for successful MFA verification
    pub fn reset_progressive_delay(&self, ip: &str) {
        if let Some(delay_entry) = self.progressive_delays.get(ip) {
            delay_entry.store(0, Ordering::Relaxed);
        }
    }

    /// Check IP-based verification rate limit
    async fn check_ip_verify_rate_limit(&self, ip: &str) -> Result<(), AuthencError> {
        let counter_ref = self
            .ip_verify_counters
            .entry(ip.to_string())
            .or_insert_with(|| RwLock::new(RateLimitCounter::new()));

        let mut counter = counter_ref.write().await;
        counter.reset_if_expired(Duration::from_secs(60)); // 1 minute window

        let count = counter.increment();
        if count > self.config.max_attempts_per_minute_per_ip {
            warn!(%ip, count, "MFA verification IP rate limit exceeded");
            return Err(AuthencError::RateLimitExceeded);
        }

        debug!(%ip, count, "MFA verification request within IP rate limit");
        Ok(())
    }

    /// Check user-based verification rate limit
    async fn check_user_verify_rate_limit(&self, user_id: Uuid) -> Result<(), AuthencError> {
        let counter_ref = self
            .user_verify_counters
            .entry(user_id)
            .or_insert_with(|| RwLock::new(RateLimitCounter::new()));

        let mut counter = counter_ref.write().await;
        counter.reset_if_expired(Duration::from_secs(60)); // 1 minute window

        let count = counter.increment();
        if count > self.config.max_attempts_per_minute_per_user {
            warn!(%user_id, count, "MFA verification user rate limit exceeded");
            return Err(AuthencError::RateLimitExceeded);
        }

        debug!(%user_id, count, "MFA verification request within user rate limit");
        Ok(())
    }

    /// Get a reference to the locked accounts map (for admin access)
    pub fn get_locked_accounts(&self) -> &Arc<DashMap<Uuid, AccountLockout>> {
        &self.locked_accounts
    }

    /// Get the rate limit configuration
    pub fn get_config(&self) -> &MfaRateLimitConfig {
        &self.config
    }
}

/// Middleware for MFA-specific rate limiting
pub async fn mfa_rate_limit_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<MfaRateLimiterState>>,
    request: Request,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    let path = request.uri().path();
    let ip = addr.ip().to_string();

    // Only apply MFA rate limiting to MFA endpoints
    if !path.contains("/mfa/") {
        return Ok(next.run(request).await);
    }

    // Apply progressive delay before processing request
    state.apply_progressive_delay(&ip).await;

    // Check rate limits based on endpoint type
    let rate_limit_result = if path.contains("/mfa/setup") {
        state.check_mfa_setup_rate_limit(&ip).await
    } else if path.contains("/mfa/verify") || path.contains("/mfa/verify-setup") {
        // For verification endpoints, we don't have user_id yet, so check IP only
        state.check_mfa_verify_rate_limit(&ip, None).await
    } else {
        Ok(()) // Other MFA endpoints use default rate limiting
    };

    match rate_limit_result {
        Ok(_) => Ok(next.run(request).await),
        Err(AuthencError::RateLimitExceeded) => {
            warn!(%ip, %path, "MFA rate limit exceeded");
            Err(StatusCode::TOO_MANY_REQUESTS)
        }
        Err(AuthencError::AccountLocked {
            reason,
            locked_until: _,
        }) => {
            warn!(%ip, %path, %reason, "Account locked");
            Err(StatusCode::LOCKED)
        }
        Err(_) => {
            error!(%ip, %path, "Unexpected error in MFA rate limiting");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mfa_verify_rate_limiting() {
        let config = MfaRateLimitConfig {
            max_attempts_per_minute_per_ip: 2,
            max_attempts_per_minute_per_user: 2,
            enable_progressive_delays: false,
            enable_account_lockout: false,
            ..Default::default()
        };

        let state = MfaRateLimiterState::new(config);
        let user_id = Uuid::new_v4();

        // First two requests should succeed
        assert!(
            state
                .check_mfa_verify_rate_limit("127.0.0.1", Some(user_id))
                .await
                .is_ok()
        );
        assert!(
            state
                .check_mfa_verify_rate_limit("127.0.0.1", Some(user_id))
                .await
                .is_ok()
        );

        // Third request should be rate limited
        assert!(
            state
                .check_mfa_verify_rate_limit("127.0.0.1", Some(user_id))
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn test_account_lockout() {
        let config = MfaRateLimitConfig {
            account_lockout_threshold: 2,
            enable_account_lockout: true,
            ..Default::default()
        };

        let state = MfaRateLimiterState::new(config);
        let user_id = Uuid::new_v4();

        // Record failed attempts
        assert!(
            state
                .record_failed_mfa_attempt("127.0.0.1", user_id)
                .await
                .is_ok()
        );

        // Second failed attempt should trigger lockout
        assert!(
            state
                .record_failed_mfa_attempt("127.0.0.1", user_id)
                .await
                .is_err()
        );

        // Account should be locked
        assert!(state.is_account_locked(user_id).is_some());
    }

    #[tokio::test]
    async fn test_mfa_setup_rate_limiting() {
        let config = MfaRateLimitConfig {
            max_setup_attempts_per_hour_per_ip: 1,
            ..Default::default()
        };

        let state = MfaRateLimiterState::new(config);

        // First request should succeed
        assert!(state.check_mfa_setup_rate_limit("127.0.0.1").await.is_ok());

        // Second request should be rate limited
        assert!(state.check_mfa_setup_rate_limit("127.0.0.1").await.is_err());
    }
}
