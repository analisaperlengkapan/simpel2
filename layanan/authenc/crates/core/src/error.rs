//! Error types for authenc-core.
//!
//! Re-exports error types from `authenc-types` to provide a convenient
//! `crate::error::Result` and `crate::error::AuthencError` path.

pub use authenc_types::error::AuthencError;

/// Result type alias using `AuthencError`
pub type Result<T> = std::result::Result<T, AuthencError>;
