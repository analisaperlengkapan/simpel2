//! # authenc-webauthn
//!
//! WebAuthn/Passkeys (FIDO2) authentication for Authenc - PRIMARY AUTHENTICATION METHOD.
//!
//! This crate provides:
//! - Passkey registration flow
//! - Passkey authentication flow (including usernameless)
//! - Credential management
//! - Replay attack prevention (credential counter)
//! - Origin binding enforcement
//! - Multi-device passkey support
//! - Platform authenticator support (Touch ID, Face ID, Windows Hello)
//! - Security key support (YubiKey, Titan Key, etc.)

// Re-export types from authenc-types
pub use authenc_types::*;

// Re-export webauthn-rs types for convenience
pub use webauthn_rs::prelude::*;

pub mod models;
pub mod service;
pub mod store;

// Re-export main types
pub use models::{
    AuthenticationResult, AuthenticationSession, RegistrationSession, StoredCredential,
};
pub use service::{WebAuthnConfig, WebAuthnService};
pub use store::CredentialStore;

#[cfg(test)]
mod tests {
    #[test]
    fn test_crate_compiles() {
        // Basic smoke test to ensure crate compiles
        assert_eq!(2 + 2, 4);
    }
}
