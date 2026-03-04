//! MFA Rate Limiter State
//!
//! Provides rate limiting for MFA operations to prevent brute-force attacks.
//! This is the canonical implementation - authenc-api should use this crate's
//! rate limiter rather than defining its own.

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::debug;
use uuid::Uuid;

/// Configuration for MFA rate limiting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaRateLimitConfig {
    /// Maximum verification attempts per user per window
    pub max_verify_attempts: u32,
    /// Window duration in seconds
    pub window_seconds: u64,
    /// Whether to enable account lockout after repeated failures
    pub enable_account_lockout: bool,
    /// Number of failures before account lockout
    pub account_lockout_threshold: u32,
    /// Duration of account lockout in minutes
    pub account_lockout_duration_minutes: u32,
}

impl Default for MfaRateLimitConfig {
    fn default() -> Self {
        Self {
            max_verify_attempts: 5,
            window_seconds: 300,
            enable_account_lockout: true,
            account_lockout_threshold: 10,
            account_lockout_duration_minutes: 30,
        }
    }
}

/// Account lockout tracking entry
#[derive(Debug, Clone)]
pub struct AccountLockout {
    /// Time until the account lockout expires
    pub locked_until: Instant,
    /// Number of failed attempts that triggered the lockout
    pub failed_attempts: u32,
    /// Reason for the account lockout
    pub lockout_reason: String,
}

/// MFA rate limiter state shared across the application
#[derive(Clone)]
pub struct MfaRateLimiterState {
    config: MfaRateLimitConfig,
    locked_accounts: Arc<DashMap<Uuid, AccountLockout>>,
}

impl MfaRateLimiterState {
    /// Create a new MFA rate limiter state
    pub fn new(config: MfaRateLimitConfig) -> Self {
        let state = Self {
            config,
            locked_accounts: Arc::new(DashMap::new()),
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
        let mut interval = tokio::time::interval(Duration::from_secs(300));
        loop {
            interval.tick().await;
            let now = Instant::now();
            self.locked_accounts
                .retain(|_, lockout| now < lockout.locked_until);
            debug!("Cleaned up expired MFA rate limit entries");
        }
    }

    /// Check if an account is currently locked
    pub fn is_account_locked(&self, user_id: Uuid) -> Option<AccountLockout> {
        let now = Instant::now();
        self.locked_accounts.get(&user_id).and_then(|entry| {
            if now < entry.locked_until {
                Some(entry.clone())
            } else {
                None
            }
        })
    }

    /// Lock an account due to excessive failed MFA attempts
    pub fn lock_account(&self, user_id: Uuid, reason: String) {
        if !self.config.enable_account_lockout {
            return;
        }

        let lockout_duration =
            Duration::from_secs(self.config.account_lockout_duration_minutes as u64 * 60);

        self.locked_accounts.insert(
            user_id,
            AccountLockout {
                locked_until: Instant::now() + lockout_duration,
                failed_attempts: self.config.account_lockout_threshold,
                lockout_reason: reason,
            },
        );
    }

    /// Unlock an account
    pub fn unlock_account(&self, user_id: Uuid) -> bool {
        self.locked_accounts.remove(&user_id).is_some()
    }

    /// Get all currently locked accounts
    pub fn get_locked_accounts(&self) -> Vec<(Uuid, AccountLockout)> {
        let now = Instant::now();
        self.locked_accounts
            .iter()
            .filter(|entry| now < entry.locked_until)
            .map(|entry| (*entry.key(), entry.value().clone()))
            .collect()
    }

    /// Get the rate limit configuration
    pub fn get_config(&self) -> &MfaRateLimitConfig {
        &self.config
    }
}
