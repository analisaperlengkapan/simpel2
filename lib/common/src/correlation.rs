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

    /// Try to create from an HTTP header value
    /// Returns None if the header is missing or invalid utf-8
    #[cfg(feature = "axum")]
    pub fn from_headers(headers: &axum::http::HeaderMap) -> Option<Self> {
        headers
            .get(REQUEST_ID_HEADER)
            .and_then(|h| h.to_str().ok())
            .map(|s| Self(s.to_string()))
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

/// Axum middleware for correlation ID
#[cfg(feature = "axum")]
pub async fn correlation_id_middleware(
    mut request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let headers = request.headers();
    let correlation_id = CorrelationId::from_headers(headers).unwrap_or_default();

    // Add to request extensions as RequestContext (or update existing)
    let context = crate::context::RequestContext::from_headers(headers);
    request.extensions_mut().insert(context);

    let mut response = next.run(request).await;

    // Inject into response headers
    if let Ok(value) = correlation_id.as_str().parse::<axum::http::HeaderValue>() {
        response.headers_mut().insert(REQUEST_ID_HEADER, value);
    }

    response
}
