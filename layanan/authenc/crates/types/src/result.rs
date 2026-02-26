//! Result type for Authenc operations

use crate::error::AuthencError;

/// Result type alias for Authenc operations
pub type Result<T> = std::result::Result<T, AuthencError>;
