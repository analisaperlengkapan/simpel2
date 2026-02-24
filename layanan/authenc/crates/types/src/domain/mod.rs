//! Domain models for Authenc
//!
//! This module contains the core domain models used throughout the Authenc system.
//! These models represent the fundamental entities and their relationships.

// Core domain models
pub mod consent;
pub mod group;
pub mod organization;
pub mod permission;
pub mod realm;
pub mod role;
pub mod satker;
pub mod session;
pub mod social_account;
pub mod user;

// OAuth2/OIDC domain models
pub mod client_policy;
pub mod client_registration;
pub mod client_scope;
pub mod device;
pub mod oauth2;
pub mod oidc_client;
pub mod protocol_mapper;
pub mod scope;
pub mod service_account;
pub mod token;

// UMA 2.0 domain models
pub mod permission_ticket;
pub mod resource;
pub mod resource_server;

// Audit and event models
pub mod audit;
pub mod audit_log;
pub mod compliance;
pub mod events;

// Advanced authentication models
pub mod dynamic_role;
pub mod mfa;
pub mod saml;
pub mod webauthn;

// Legacy models (simplified versions for backward compatibility)
pub mod legacy_permission;
pub mod legacy_realm;
pub mod legacy_role;
pub mod legacy_user;

// Re-export commonly used types
pub use consent::*;
pub use group::*;
pub use organization::*;
pub use permission::*;
pub use realm::*;
pub use role::*;
pub use satker::*;
pub use session::*;
pub use social_account::*;
pub use user::*;

// Re-export OAuth2/OIDC types
pub use client_policy::*;
pub use client_registration::*;
pub use client_scope::*;
pub use device::*;
pub use oauth2::*;
pub use oidc_client::*;
pub use protocol_mapper::*;
pub use scope::*;
pub use service_account::*;
pub use token::*;

// Re-export UMA 2.0 types
pub use permission_ticket::*;
pub use resource::*;
pub use resource_server::*;

// Re-export audit and event types
pub use audit::*;
pub use audit_log::*;
pub use compliance::*;
pub use events::*;

// Re-export advanced authentication types
pub use dynamic_role::*;
pub use mfa::*;
pub use saml::*;
pub use webauthn::*;

// Re-export legacy types (for backward compatibility) with aliases to avoid name conflicts
// with the primary domain types (User, Realm, Role, Permission)
pub use legacy_permission::Permission as LegacyPermission;
pub use legacy_realm::Realm as LegacyRealm;
pub use legacy_role::Role as LegacyRole;
pub use legacy_user::User as LegacyUser;
