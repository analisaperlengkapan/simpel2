//! # Secreton Types
//!
//! Shared types and primitives used across the Secreton engine system.
//! This crate provides common data structures that are used by multiple crates
//! to avoid duplication and ensure consistency.
//!
//! ## Modules
//!
//! - [`security`] - Security levels and classifications
//! - [`resource`] - Resource identifiers and metadata
//! - [`result`] - Common result types

pub mod resource;
pub mod result;
pub mod security;

// Re-export commonly used types
pub use resource::{Metadata, ResourceId, Tags};
pub use security::SecurityLevel;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::resource::{Metadata, ResourceId, Tags};
    pub use crate::security::SecurityLevel;
}
