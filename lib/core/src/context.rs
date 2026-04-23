use crate::correlation::CorrelationId;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Unified request context for infrastructure metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    /// IP address of the client
    pub ip_address: Option<String>,
    /// User agent string
    pub user_agent: Option<String>,
    /// Correlation/Request ID
    pub request_id: CorrelationId,
    /// Time when the request started
    #[serde(skip, default = "Instant::now")]
    pub start_time: Instant,
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
            start_time: Instant::now(),
        }
    }
}

impl Default for RequestContext {
    fn default() -> Self {
        Self::new(None, None, CorrelationId::default())
    }
}
