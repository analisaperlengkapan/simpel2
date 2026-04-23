//! Correlation utilities for distributed tracing
//!
//! Provides utilities for managing correlation IDs across service boundaries
//! to enable end-to-end request tracing.

use lib_core::correlation::{CORRELATION_ID_HEADER, CorrelationId, REQUEST_ID_HEADER};

/// HTTP header name for correlation ID
// pub const CORRELATION_ID_HEADER: &str = "X-Correlation-ID"; // Imported from lib_core

/// HTTP header name for request ID
// pub const REQUEST_ID_HEADER: &str = "X-Request-ID"; // Imported from lib_core

/// gRPC metadata key for correlation ID
pub const GRPC_CORRELATION_ID_KEY: &str = "x-correlation-id";

/// Generate a new correlation ID
pub fn generate_correlation_id() -> String {
    CorrelationId::new().to_string()
}

/// Extract correlation ID from HTTP headers
pub fn extract_correlation_id_from_headers(headers: &reqwest::header::HeaderMap) -> Option<String> {
    headers
        .get(CORRELATION_ID_HEADER)
        .or_else(|| headers.get(REQUEST_ID_HEADER))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// Correlation context for request tracking
#[derive(Debug, Clone)]
pub struct CorrelationContext {
    /// Correlation ID for the entire flow
    pub correlation_id: String,
    /// Request ID for this specific request
    pub request_id: String,
    /// Parent request ID (if this is a child request)
    pub parent_request_id: Option<String>,
}

impl CorrelationContext {
    /// Create a new correlation context with generated IDs
    pub fn new() -> Self {
        Self {
            correlation_id: generate_correlation_id(),
            request_id: generate_correlation_id(),
            parent_request_id: None,
        }
    }

    /// Create from existing correlation ID
    pub fn from_correlation_id(correlation_id: String) -> Self {
        Self {
            correlation_id: correlation_id.clone(),
            request_id: generate_correlation_id(),
            parent_request_id: None,
        }
    }

    /// Create with parent request
    pub fn with_parent(correlation_id: String, parent_request_id: String) -> Self {
        Self {
            correlation_id,
            request_id: generate_correlation_id(),
            parent_request_id: Some(parent_request_id),
        }
    }

    /// Add correlation headers to HTTP request builder
    pub fn add_to_headers(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        builder
            .header(CORRELATION_ID_HEADER, &self.correlation_id)
            .header(REQUEST_ID_HEADER, &self.request_id)
    }

    /// Convert to gRPC metadata entries
    #[cfg(feature = "grpc")]
    pub fn to_grpc_metadata(&self) -> Vec<(String, String)> {
        vec![
            (
                GRPC_CORRELATION_ID_KEY.to_string(),
                self.correlation_id.clone(),
            ),
            ("x-request-id".to_string(), self.request_id.clone()),
        ]
    }
}

impl Default for CorrelationContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_generate_correlation_id() {
        let id1 = generate_correlation_id();
        let id2 = generate_correlation_id();

        assert_ne!(id1, id2);
        assert!(Uuid::parse_str(&id1).is_ok());
    }

    #[test]
    fn test_correlation_context_creation() {
        let ctx = CorrelationContext::new();

        assert!(!ctx.correlation_id.is_empty());
        assert!(!ctx.request_id.is_empty());
        assert!(ctx.parent_request_id.is_none());
    }

    #[test]
    fn test_from_correlation_id() {
        let corr_id = "test-correlation-123";
        let ctx = CorrelationContext::from_correlation_id(corr_id.to_string());

        assert_eq!(ctx.correlation_id, corr_id);
        assert_ne!(ctx.request_id, corr_id);
    }

    #[test]
    fn test_with_parent() {
        let corr_id = "test-correlation-123";
        let parent_id = "parent-request-456";
        let ctx = CorrelationContext::with_parent(corr_id.to_string(), parent_id.to_string());

        assert_eq!(ctx.correlation_id, corr_id);
        assert_eq!(ctx.parent_request_id, Some(parent_id.to_string()));
    }

    #[test]
    fn test_extract_from_headers() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(CORRELATION_ID_HEADER, "test-123".parse().unwrap());

        let extracted = extract_correlation_id_from_headers(&headers);
        assert_eq!(extracted, Some("test-123".to_string()));
    }

    #[test]
    fn test_extract_from_request_id_fallback() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(REQUEST_ID_HEADER, "request-456".parse().unwrap());

        let extracted = extract_correlation_id_from_headers(&headers);
        assert_eq!(extracted, Some("request-456".to_string()));
    }
}
