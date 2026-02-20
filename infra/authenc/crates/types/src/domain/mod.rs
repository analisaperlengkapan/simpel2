//! Domain models for Authenc
//!
//! This module contains the core domain models used throughout the Authenc system.
//! These models represent the fundamental entities and their relationships.

// Core domain models
pub mod user;
pub mod realm;
pub mod role;
pub mod permission;
pub mod consent;
pub mod social_account;
pub mod session;
pub mod group;
pub mod organization;
pub mod satker;

// OAuth2/OIDC domain models
pub mod oauth2;
pub mod oidc_client;
pub mod client_scope;
pub mod scope;
pub mod token;
pub mod device;
pub mod service_account;
pub mod client_registration;
pub mod client_policy;
pub mod protocol_mapper;

// UMA 2.0 domain models
pub mod resource;
pub mod resource_server;
pub mod permission_ticket;

// Audit and event models
pub mod audit;
pub mod audit_log;
pub mod events;
pub mod compliance;

// Advanced authentication models
pub mod webauthn;
pub mod saml;
pub mod dynamic_role;

// Legacy models (simplified versions for backward compatibility)
pub mod legacy_user;
pub mod legacy_realm;
pub mod legacy_role;
pub mod legacy_permission;

// Re-export commonly used types
pub use user::*;
pub use realm::*;
pub use role::*;
pub use permission::*;
pub use consent::*;
pub use social_account::*;
pub use session::*;
pub use group::*;
pub use organization::*;
pub use satker::*;

// Re-export OAuth2/OIDC types
pub use oauth2::*;
pub use oidc_client::*;
pub use client_scope::*;
pub use scope::*;
pub use token::*;
pub use device::*;
pub use service_account::*;
pub use client_registration::*;
pub use client_policy::*;
pub use protocol_mapper::*;

// Re-export UMA 2.0 types
pub use resource::*;
pub use resource_server::*;
pub use permission_ticket::*;

// Re-export audit and event types
pub use audit::*;
pub use audit_log::*;
pub use events::*;
pub use compliance::*;

// Re-export advanced authentication types
pub use webauthn::*;
pub use saml::*;
pub use dynamic_role::*;

// Re-export legacy types (for backward compatibility)
pub use legacy_user::*;
pub use legacy_realm::*;
pub use legacy_role::*;
pub use legacy_permission::*;
