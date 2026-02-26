//! # authenc-types
//!
//! Shared types, traits, and interfaces for the Authenc authentication system.
//!
//! This crate provides the foundational types and trait definitions used across
//! all Authenc crates, enabling dependency injection and modular architecture.

// Re-export commonly used types
pub use chrono::{DateTime, Utc};
pub use uuid::Uuid;

// Module declarations
pub mod config;
pub mod domain; // Domain models (user, realm, role, etc.)
pub mod domain_types; // Strongly-typed IDs and auth results
pub mod error;
pub mod result;
pub mod traits;

#[cfg(test)]
mod domain_tests;

// Re-export key types for convenience
pub use config::*;
pub use domain::*; // Re-export all domain models
// Re-export strongly-typed IDs and auth result types from domain_types
// Note: OidcClient, AuthorizationRequest, TokenResponse are NOT re-exported here
// because they also exist in domain/ modules (would cause ambiguous glob re-exports).
// Use authenc_types::domain_types::OidcClient etc. for the domain_types versions,
// or authenc_types::OidcClient (resolved from domain::* which takes precedence).
pub use domain_types::{
    AuthFailureReason, AuthResult, AuthorizationCode, AuthorizationResponse, ClientId, Credentials,
    OAuth2Error, RealmId, RefreshToken, RoleId, SessionId, TokenRequest, UserId,
};
pub use error::AuthencError;
pub use result::Result;
pub use traits::*;
