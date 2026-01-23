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

    /// Extract request context from headers
    #[cfg(feature = "axum")]
    pub fn from_headers(headers: &axum::http::HeaderMap) -> Self {
        use crate::correlation::REQUEST_ID_HEADER;

        let ip_address = headers
            .get("x-forwarded-for")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.split(',').next())
            .map(|s| s.trim().to_string())
            .or_else(|| {
                headers
                    .get("x-real-ip")
                    .and_then(|h| h.to_str().ok())
                    .map(|s| s.trim().to_string())
            })
            .or_else(|| {
                headers
                    .get("cf-connecting-ip")
                    .and_then(|h| h.to_str().ok())
                    .map(|s| s.trim().to_string())
            })
            .or_else(|| {
                headers
                    .get("true-client-ip")
                    .and_then(|h| h.to_str().ok())
                    .map(|s| s.trim().to_string())
            });

        let user_agent = headers
            .get("user-agent")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        let request_id = headers
            .get(REQUEST_ID_HEADER)
            .and_then(|h| h.to_str().ok())
            .map(|s| CorrelationId::new_from_string(s.to_string()))
            .unwrap_or_default();

        Self::new(ip_address, user_agent, request_id)
    }
}

impl Default for RequestContext {
    fn default() -> Self {
        Self::new(None, None, CorrelationId::default())
    }
}
