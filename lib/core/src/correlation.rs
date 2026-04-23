use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Header used for request correlation
pub const REQUEST_ID_HEADER: &str = "X-Request-ID";
/// Header used for correlation ID
pub const CORRELATION_ID_HEADER: &str = "X-Correlation-ID";

/// Correlation ID for tracking requests across services
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CorrelationId(String);

impl CorrelationId {
    /// Generate a new random correlation ID
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Create from an existing string
    pub fn new_from_string(id: String) -> Self {
        Self(id)
    }

    /// Get the underlying string representation
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for CorrelationId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CorrelationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
