//! Brute Force Protection Service
//!
//! This module provides protection against brute force attacks by:
//! - Tracking failed login attempts per username
//! - Implementing account lockout after configurable failures
//! - Adding CAPTCHA requirement after threshold
//! - Automatic lockout expiration

use async_trait::async_trait;
use authenc_types::{result::Result, traits::BruteForceProtector as BruteForceProtectorTrait};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Configuration for brute force protection
#[derive(Debug, Clone)]
pub struct BruteForceConfig {
    /// Maximum failed attempts before lockout
    pub max_attempts: usize,
    /// Lockout duration in seconds
    pub lockout_duration_secs: u64,
    /// Failed attempts before CAPTCHA is required
    pub captcha_threshold: usize,
    /// Time window for counting attempts (in seconds)
    pub window_secs: u64,
}

impl Default for BruteForceConfig {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            lockout_duration_secs: 900, // 15 minutes
            captcha_threshold: 3,
            window_secs: 300, // 5 minutes
        }
    }
}

/// Attempt record for a username
#[derive(Debug, Clone)]
struct AttemptRecord {
    /// Timestamps of failed attempts
    attempts: Vec<Instant>,
    /// Lockout expiration time (if locked)
    locked_until: Option<Instant>,
    /// Whether CAPTCHA is required
    captcha_required: bool,
}

impl AttemptRecord {
    fn new() -> Self {
        Self {
            attempts: Vec::new(),
            locked_until: None,
            captcha_required: false,
        }
    }

    /// Check if the account is currently locked
    fn is_locked(&self) -> bool {
        if let Some(locked_until) = self.locked_until {
            Instant::now() < locked_until
        } else {
            false
        }
    }

    /// Check if CAPTCHA is required
    fn requires_captcha(&self) -> bool {
        self.captcha_required
    }

    /// Remove expired attempts based on time window
    fn cleanup_expired(&mut self, window: Duration) {
        let now = Instant::now();
        self.attempts.retain(|&t| now.duration_since(t) < window);
    }
}

/// Brute force protection service implementation
pub struct BruteForceProtectorImpl {
    /// Configuration
    config: BruteForceConfig,
    /// Map of username to attempt records
    records: Arc<RwLock<HashMap<String, AttemptRecord>>>,
}

impl BruteForceProtectorImpl {
    /// Create a new brute force protector with default configuration
    pub fn new() -> Self {
        Self::with_config(BruteForceConfig::default())
    }

    /// Create a new brute force protector with custom configuration
    pub fn with_config(config: BruteForceConfig) -> Self {
        Self {
            config,
            records: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get the current configuration
    pub fn config(&self) -> &BruteForceConfig {
        &self.config
    }

    /// Check if CAPTCHA is required for a username
    pub async fn is_captcha_required(&self, username: &str) -> bool {
        let records = self.records.read().await;
        records
            .get(username)
            .map(|r| r.requires_captcha())
            .unwrap_or(false)
    }

    /// Get the number of failed attempts for a username
    pub async fn get_attempt_count(&self, username: &str) -> usize {
        let records = self.records.read().await;
        records.get(username).map(|r| r.attempts.len()).unwrap_or(0)
    }

    /// Get the lockout expiration time for a username
    pub async fn get_lockout_expiration(&self, username: &str) -> Option<Instant> {
        let records = self.records.read().await;
        records.get(username).and_then(|r| r.locked_until)
    }
}

impl Default for BruteForceProtectorImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BruteForceProtectorTrait for BruteForceProtectorImpl {
    /// Check if a username is locked
    async fn check(&self, username: &str) -> Result<()> {
        let mut records = self.records.write().await;

        // Get or create record
        let record = records
            .entry(username.to_string())
            .or_insert_with(AttemptRecord::new);

        // Clean up expired attempts
        record.cleanup_expired(Duration::from_secs(self.config.window_secs));

        // Check if locked
        if record.is_locked() {
            return Err(authenc_types::error::AuthencError::AccountLocked {
                username: username.to_string(),
                locked_until: record.locked_until,
            });
        }

        Ok(())
    }

    /// Record a failed login attempt
    async fn record_failure(&self, username: &str) -> Result<()> {
        let mut records = self.records.write().await;

        // Get or create record
        let record = records
            .entry(username.to_string())
            .or_insert_with(AttemptRecord::new);

        // Clean up expired attempts
        record.cleanup_expired(Duration::from_secs(self.config.window_secs));

        // Add new attempt
        record.attempts.push(Instant::now());

        // Check if CAPTCHA should be required
        if record.attempts.len() >= self.config.captcha_threshold {
            record.captcha_required = true;
        }

        // Check if account should be locked
        if record.attempts.len() >= self.config.max_attempts {
            record.locked_until =
                Some(Instant::now() + Duration::from_secs(self.config.lockout_duration_secs));
        }

        Ok(())
    }

    /// Record a successful login (reset failure count)
    async fn record_success(&self, username: &str) -> Result<()> {
        let mut records = self.records.write().await;
        records.remove(username);
        Ok(())
    }

    /// Unlock a username (admin action)
    async fn unlock(&self, username: &str) -> Result<()> {
        let mut records = self.records.write().await;
        records.remove(username);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_new_protector() {
        let protector = BruteForceProtectorImpl::new();
        assert_eq!(protector.config.max_attempts, 5);
        assert_eq!(protector.config.lockout_duration_secs, 900);
        assert_eq!(protector.config.captcha_threshold, 3);
    }

    #[tokio::test]
    async fn test_check_unlocked_user() {
        let protector = BruteForceProtectorImpl::new();
        let result = protector.check("testuser").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_record_failure_under_threshold() {
        let protector = BruteForceProtectorImpl::new();

        // Record 2 failures (under captcha threshold of 3)
        protector.record_failure("testuser").await.unwrap();
        protector.record_failure("testuser").await.unwrap();

        // Should not be locked
        assert!(protector.check("testuser").await.is_ok());

        // Should not require CAPTCHA yet
        assert!(!protector.is_captcha_required("testuser").await);

        // Should have 2 attempts
        assert_eq!(protector.get_attempt_count("testuser").await, 2);
    }

    #[tokio::test]
    async fn test_captcha_required_after_threshold() {
        let protector = BruteForceProtectorImpl::new();

        // Record 3 failures (at captcha threshold)
        for _ in 0..3 {
            protector.record_failure("testuser").await.unwrap();
        }

        // Should require CAPTCHA
        assert!(protector.is_captcha_required("testuser").await);

        // Should not be locked yet (max_attempts is 5)
        assert!(protector.check("testuser").await.is_ok());
    }

    #[tokio::test]
    async fn test_lockout_after_max_attempts() {
        let protector = BruteForceProtectorImpl::new();

        // Record 5 failures (at max_attempts)
        for _ in 0..5 {
            protector.record_failure("testuser").await.unwrap();
        }

        // Should be locked
        let result = protector.check("testuser").await;
        assert!(result.is_err());

        // Should have lockout expiration
        assert!(protector.get_lockout_expiration("testuser").await.is_some());
    }

    #[tokio::test]
    async fn test_record_success_clears_attempts() {
        let protector = BruteForceProtectorImpl::new();

        // Record some failures
        for _ in 0..3 {
            protector.record_failure("testuser").await.unwrap();
        }

        // Verify attempts recorded
        assert_eq!(protector.get_attempt_count("testuser").await, 3);

        // Record success
        protector.record_success("testuser").await.unwrap();

        // Attempts should be cleared
        assert_eq!(protector.get_attempt_count("testuser").await, 0);
        assert!(!protector.is_captcha_required("testuser").await);
    }

    #[tokio::test]
    async fn test_unlock_clears_lockout() {
        let protector = BruteForceProtectorImpl::new();

        // Lock the account
        for _ in 0..5 {
            protector.record_failure("testuser").await.unwrap();
        }

        // Verify locked
        assert!(protector.check("testuser").await.is_err());

        // Unlock
        protector.unlock("testuser").await.unwrap();

        // Should be unlocked
        assert!(protector.check("testuser").await.is_ok());
        assert_eq!(protector.get_attempt_count("testuser").await, 0);
    }

    #[tokio::test]
    async fn test_different_users_tracked_separately() {
        let protector = BruteForceProtectorImpl::new();

        // Record failures for user1
        for _ in 0..3 {
            protector.record_failure("user1").await.unwrap();
        }

        // Record failures for user2
        for _ in 0..2 {
            protector.record_failure("user2").await.unwrap();
        }

        // Verify separate tracking
        assert_eq!(protector.get_attempt_count("user1").await, 3);
        assert_eq!(protector.get_attempt_count("user2").await, 2);
        assert!(protector.is_captcha_required("user1").await);
        assert!(!protector.is_captcha_required("user2").await);
    }

    #[tokio::test]
    async fn test_custom_config() {
        let config = BruteForceConfig {
            max_attempts: 3,
            lockout_duration_secs: 60,
            captcha_threshold: 2,
            window_secs: 120,
        };

        let protector = BruteForceProtectorImpl::with_config(config);

        // Record 2 failures (at captcha threshold)
        for _ in 0..2 {
            protector.record_failure("testuser").await.unwrap();
        }

        // Should require CAPTCHA
        assert!(protector.is_captcha_required("testuser").await);

        // Record 1 more failure (at max_attempts)
        protector.record_failure("testuser").await.unwrap();

        // Should be locked
        assert!(protector.check("testuser").await.is_err());
    }

    #[tokio::test]
    async fn test_expired_attempts_cleanup() {
        let config = BruteForceConfig {
            max_attempts: 5,
            lockout_duration_secs: 900,
            captcha_threshold: 3,
            window_secs: 1, // 1 second window
        };

        let protector = BruteForceProtectorImpl::with_config(config);

        // Record 2 failures
        for _ in 0..2 {
            protector.record_failure("testuser").await.unwrap();
        }

        // Wait for window to expire
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Check should trigger cleanup
        protector.check("testuser").await.unwrap();

        // Attempts should be cleaned up
        assert_eq!(protector.get_attempt_count("testuser").await, 0);
    }
}
