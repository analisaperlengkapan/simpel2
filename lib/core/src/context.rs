use crate::correlation::CorrelationId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Unified request context for infrastructure metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    /// IP address of the client
    pub ip_address: Option<String>,
    /// User agent string
    pub user_agent: Option<String>,
    /// Correlation/Request ID
    pub request_id: CorrelationId,
    /// Time when the request started.
    ///
    /// Uses `chrono::DateTime<Utc>` instead of `std::time::Instant` so that
    /// this type is WASM-safe — `Instant::now()` panics on
    /// `wasm32-unknown-unknown`, and `lib-core` is required to compile for
    /// both backend and WASM targets.
    #[serde(skip, default = "Utc::now")]
    pub start_time: DateTime<Utc>,
}

impl RequestContext {
    /// Create a new request context
    pub fn new(
        ip_address: Option<String>,
        user_agent: Option<String>,
        request_id: CorrelationId,
    ) -> Self {
        Self {
            ip_address,
            user_agent,
            request_id,
            start_time: Utc::now(),
        }
    }
}

impl Default for RequestContext {
    fn default() -> Self {
        Self::new(None, None, CorrelationId::default())
    }
}
