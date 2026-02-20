//! # authenc-mfa
//!
//! Multi-factor authentication for Authenc identity provider.
//!
//! This crate provides:
//! - TOTP setup and verification
//! - Backup codes generation and validation
//! - MFA policy enforcement
//! - Integration with Secreton for secret storage

// Re-export types from authenc-types
pub use authenc_types::*;

pub mod totp;
pub mod backup_codes;
pub mod policy;

// Re-export main types and services
pub use totp::{TotpConfig, TotpService, TotpSetupResponse, TotpStore};
pub use backup_codes::{BackupCode, BackupCodesConfig, BackupCodesService, BackupCodesSet, BackupCodesStore};
pub use policy::{MfaPolicy, MfaPolicyService, MfaPolicyStore, MfaRequirement, MfaStatus};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
