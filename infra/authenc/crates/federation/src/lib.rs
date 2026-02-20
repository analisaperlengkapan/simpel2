//! # authenc-federation
//!
//! SSO and external identity provider integration for Authenc.
//!
//! This crate provides:
//! - External IdP integration (OIDC, SAML, LDAP)
//! - SSO flow orchestration
//! - User account linking
//! - Attribute mapping
//! - Just-in-time provisioning

// Re-export types from authenc-types
pub use authenc_types::*;

pub mod service;
pub mod providers;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
