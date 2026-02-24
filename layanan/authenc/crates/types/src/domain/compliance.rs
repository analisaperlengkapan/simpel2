//! Compliance and audit metrics types
//!
//! This module provides types for tracking compliance metrics and audit statistics.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Compliance metrics for tracking security and audit statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceMetrics {
    /// Total number of login attempts
    pub total_logins: u64,

    /// Number of failed login attempts
    pub failed_logins: u64,

    /// Number of users with MFA enabled
    pub mfa_enabled_users: u64,

    /// Total number of active users
    pub active_users: u64,

    /// Number of locked accounts
    pub locked_accounts: u64,

    /// Number of password changes in period
    pub password_changes: u64,

    /// Number of successful MFA verifications
    pub mfa_verifications: u64,

    /// Number of failed MFA verifications
    pub failed_mfa_verifications: u64,

    /// Number of active sessions
    pub active_sessions: u64,

    /// Number of OAuth2 token grants
    pub token_grants: u64,

    /// Number of OAuth2 token revocations
    pub token_revocations: u64,

    /// Number of audit log entries
    pub audit_log_entries: u64,

    /// Number of security incidents detected
    pub security_incidents: u64,

    /// Number of brute force attempts blocked
    pub brute_force_blocks: u64,

    /// Timestamp when metrics were collected
    pub collected_at: DateTime<Utc>,

    /// Start of the metrics collection period
    pub period_start: DateTime<Utc>,

    /// End of the metrics collection period
    pub period_end: DateTime<Utc>,
}

impl Default for ComplianceMetrics {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            total_logins: 0,
            failed_logins: 0,
            mfa_enabled_users: 0,
            active_users: 0,
            locked_accounts: 0,
            password_changes: 0,
            mfa_verifications: 0,
            failed_mfa_verifications: 0,
            active_sessions: 0,
            token_grants: 0,
            token_revocations: 0,
            audit_log_entries: 0,
            security_incidents: 0,
            brute_force_blocks: 0,
            collected_at: now,
            period_start: now,
            period_end: now,
        }
    }
}

impl ComplianceMetrics {
    /// Create new compliance metrics for a specific time period
    pub fn new(period_start: DateTime<Utc>, period_end: DateTime<Utc>) -> Self {
        Self {
            period_start,
            period_end,
            collected_at: Utc::now(),
            ..Default::default()
        }
    }

    /// Calculate the failure rate for logins
    pub fn login_failure_rate(&self) -> f64 {
        if self.total_logins == 0 {
            0.0
        } else {
            (self.failed_logins as f64 / self.total_logins as f64) * 100.0
        }
    }

    /// Calculate the MFA adoption rate
    pub fn mfa_adoption_rate(&self) -> f64 {
        if self.active_users == 0 {
            0.0
        } else {
            (self.mfa_enabled_users as f64 / self.active_users as f64) * 100.0
        }
    }

    /// Calculate the MFA success rate
    pub fn mfa_success_rate(&self) -> f64 {
        let total_mfa = self.mfa_verifications + self.failed_mfa_verifications;
        if total_mfa == 0 {
            0.0
        } else {
            (self.mfa_verifications as f64 / total_mfa as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_compliance_metrics() {
        let metrics = ComplianceMetrics::default();
        assert_eq!(metrics.total_logins, 0);
        assert_eq!(metrics.failed_logins, 0);
        assert_eq!(metrics.mfa_enabled_users, 0);
    }

    #[test]
    fn test_login_failure_rate() {
        let mut metrics = ComplianceMetrics::default();
        metrics.total_logins = 100;
        metrics.failed_logins = 10;

        assert_eq!(metrics.login_failure_rate(), 10.0);
    }

    #[test]
    fn test_mfa_adoption_rate() {
        let mut metrics = ComplianceMetrics::default();
        metrics.active_users = 100;
        metrics.mfa_enabled_users = 75;

        assert_eq!(metrics.mfa_adoption_rate(), 75.0);
    }

    #[test]
    fn test_mfa_success_rate() {
        let mut metrics = ComplianceMetrics::default();
        metrics.mfa_verifications = 90;
        metrics.failed_mfa_verifications = 10;

        assert_eq!(metrics.mfa_success_rate(), 90.0);
    }
}
