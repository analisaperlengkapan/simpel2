//! Error shim - re-exports error types from authenc_types for legacy operations
//!
//! This module provides backward-compatible `crate::error::Result` and
//! `crate::error::AuthencError` import patterns used by legacy operation files.

pub use authenc_types::error::AuthencError;

pub type Result<T, E = AuthencError> = std::result::Result<T, E>;
