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
//! - Social login providers (Google, GitHub, etc.)
//! - Identity brokering
//! - User synchronization

// Re-export types from authenc-types
pub use authenc_types::*;

// Core federation modules
pub mod advanced;
pub mod manager;
pub mod provider;
pub mod service;

// Federation providers
pub mod providers;

// SSO (Single Sign-On)
pub mod sso;

// Identity broker
pub mod broker;

// Social login
pub mod social;

// SAML
pub mod saml;

// User synchronization
pub mod mysimkari_sync;
pub mod user_sync;

// Re-export key types and traits
pub use advanced::{
    AdvancedFederationRegistry, GitHubOAuth2Provider, GoogleOAuth2Provider, KerberosConfig,
    KerberosFederationProvider, LdapConfig, LdapFederationProvider, LdapSyncSettings,
    SamlIdentityProvider, SamlIdpConfig, SocialLoginProvider, SocialLoginResult, SyncResult,
    UserFederationProvider, UserInfo,
};
pub use manager::{
    FederatedIdentityLink, FederationAuthResult, FederationManager, FederationProviderType,
    IdentityProviderConfig,
};
pub use provider::{DummyFederationProvider, FederationProvider, FederationRegistry};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
