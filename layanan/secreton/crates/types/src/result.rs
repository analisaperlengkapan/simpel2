//! Common result types

/// Generic result type for operations that can fail
pub type Result<T, E = Box<dyn std::error::Error + Send + Sync>> = std::result::Result<T, E>;
