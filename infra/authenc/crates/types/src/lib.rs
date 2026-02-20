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
pub mod domain;          // Domain models (user, realm, role, etc.)
pub mod domain_types;    // Strongly-typed IDs and auth results
pub mod error;
pub mod result;
pub mod traits;

#[cfg(test)]
mod domain_tests;

// Re-export key types for convenience
pub use config::*;
pub use domain::*;       // Re-export all domain models
pub use domain_types::*; // Re-export strongly-typed IDs
pub use error::AuthencError;
pub use result::Result;
pub use traits::*;
