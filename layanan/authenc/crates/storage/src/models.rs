//! Models shim - re-exports domain types from authenc_types for legacy operations
//!
//! This module provides backward-compatible re-exports so that legacy operation
//! files can continue to use `crate::models::*` import patterns while the
//! underlying types have been migrated to `authenc_types`.

// Re-export top-level domain types used directly as `crate::models::TypeName`
pub use authenc_types::domain::consent::{ConsentGrantRequest, UserConsent};
pub use authenc_types::domain::oauth2::{OAuth2AccessToken, OAuth2AuthorizationCode, OAuth2Client};
pub use authenc_types::domain::organization::{
    Organization, OrganizationInvitation, OrganizationMember,
};
pub use authenc_types::domain::realm::Realm;
pub use authenc_types::domain::role::Role;
pub use authenc_types::domain::user::User;

pub mod oauth2 {
    pub use authenc_types::domain::oauth2::{
        ClientRegistrationPolicy, CreateOAuth2ClientRequest, OAuth2AccessToken,
        OAuth2AuthorizationCode, OAuth2Client,
    };
}

pub mod role {
    pub use authenc_types::domain::role::{CreateRoleRequest, Role, UpdateRoleRequest};
}

pub mod scope {
    pub use authenc_types::domain::scope::{CreateScopeRequest, Scope, UpdateScopeRequest};
}

// Re-export submodule types to support `crate::models::realm::TypeName` patterns
pub mod realm {
    pub use authenc_types::domain::realm::{CreateRealmRequest, UpdateRealmRequest};
}

// Re-export submodule types to support `crate::models::user::TypeName` patterns
pub mod user {
    pub use authenc_types::domain::user::{
        AccessLevel, CreateUserRequest, SecretonAccessPolicy, SecurityContext, UpdateUserRequest,
    };
}

// Re-export submodule types to support `crate::models::resource_server::TypeName` patterns
pub mod resource_server {
    pub use authenc_types::domain::resource_server::{
        CreateResourceServerRequest, ResourceServer, UpdateResourceServerRequest,
    };
}

// Re-export submodule types to support `crate::models::social_account::TypeName` patterns
// Note: social_accounts legacy operations also need `crate::services::social::SocialProvider`
// which is not available in authenc-storage (it lives in authenc-core).
// The social_accounts module remains disabled until that dependency is resolved.
pub mod social_account {
    pub use authenc_types::domain::social_account::{CreateSocialAccountRequest, SocialAccount};
}
