//! # authenc-mfa
//!
//! Multi-factor authentication for Authenc identity provider.
//!
//! This crate provides:
//! - TOTP setup and verification
//! - Backup codes generation and validation
//! - MFA policy enforcement
//! - Integration with Secreton for secret storage
//! - MFA service with fallback support
//! - MFA administration and monitoring
//! - Security monitoring and audit logging
//! - Performance monitoring and metrics

// Re-export types from authenc-types
pub use authenc_types::*;

// Core MFA modules
pub mod backup_codes;
pub mod policy;
pub mod totp;

// MFA service modules
pub mod admin_service;
pub mod cache;
pub mod fallback_client;
pub mod local_storage;
pub mod rate_limiter;
pub mod service;
pub mod totp_store;

// Monitoring and logging modules
pub mod audit_logger;
pub mod performance_monitor;
pub mod security_monitor;

// Re-export main types and services
pub use backup_codes::{
    BackupCode, BackupCodesConfig, BackupCodesService, BackupCodesSet, BackupCodesStore,
};
pub use policy::{MfaPolicy, MfaPolicyService, MfaPolicyStore, MfaRequirement, MfaStatus};
pub use totp::{TotpConfig, TotpService, TotpSetupResponse};

// Re-export MFA service types
pub use admin_service::{
    AccountLockoutInfo, AutoUnlockService, MfaAdminResult, MfaAdminService, ResetMfaRequest,
    UnlockAccountRequest,
};
pub use fallback_client::MfaFallbackClient;
pub use local_storage::{DegradedMode, LocalMfaSecret, LocalStorageMetadata, MfaLocalStorage};
pub use rate_limiter::{AccountLockout, MfaRateLimitConfig, MfaRateLimiterState};
pub use service::{MfaClient, MfaService, MfaSetupResponse, MfaStatistics};
pub use totp_store::TotpStore;

// Re-export monitoring types
pub use audit_logger::{MfaAuditContext, MfaAuditLogger, MfaOperation, create_mfa_audit_logger};
pub use performance_monitor::{
    AlertType, MfaAlertConfig, MfaDashboardData, MfaMetrics, MfaPerformanceMonitor,
    PerformanceAlert,
};
pub use security_monitor::{
    AlertHandler, AlertSeverity, LoggingAlertHandler, MfaSecurityEventType, MfaSecurityMonitor,
    MfaSecurityMonitorConfig, SecurityAlert,
};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
