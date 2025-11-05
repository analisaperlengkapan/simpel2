//! Request context extraction utilities
//!
//! Provides utilities for extracting IP address, user agent, and other
//! contextual information from HTTP requests for audit logging.

use axum::http::HeaderMap;
use std::net::IpAddr;

/// Request context information extracted from HTTP headers
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// IP address of the client
    pub ip_address: Option<String>,
    /// User agent string
    pub user_agent: Option<String>,
    /// Session ID if available
    pub session_id: Option<String>,
    /// Request ID for correlation
    pub request_id: Option<String>,
}

impl RequestContext {
    /// Extract request context from HTTP headers
    pub fn from_headers(headers: &HeaderMap) -> Self {
        Self {
            ip_address: extract_ip_address(headers),
            user_agent: extract_user_agent(headers),
            session_id: None, // Set separately from session management
            request_id: extract_request_id(headers),
        }
    }

    /// Create a new request context with all fields
    pub fn new(
        ip_address: Option<String>,
        user_agent: Option<String>,
        session_id: Option<String>,
        request_id: Option<String>,
    ) -> Self {
        Self {
            ip_address,
            user_agent,
            session_id,
            request_id,
        }
    }
}

/// Extract IP address from request headers
///
/// Checks the following headers in order:
/// 1. X-Forwarded-For (first IP in the list)
/// 2. X-Real-IP
/// 3. CF-Connecting-IP (Cloudflare)
/// 4. True-Client-IP (Akamai)
pub fn extract_ip_address(headers: &HeaderMap) -> Option<String> {
    // Try X-Forwarded-For first (most common)
    if let Some(forwarded) = headers.get("x-forwarded-for") {
        if let Ok(value) = forwarded.to_str() {
            // Take the first IP in the list (original client)
            if let Some(first_ip) = value.split(',').next() {
                let trimmed = first_ip.trim();
                // Validate it's a proper IP address
                if trimmed.parse::<IpAddr>().is_ok() {
                    return Some(trimmed.to_string());
                }
            }
        }
    }

    // Try X-Real-IP
    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(value) = real_ip.to_str() {
            let trimmed = value.trim();
            if trimmed.parse::<IpAddr>().is_ok() {
                return Some(trimmed.to_string());
            }
        }
    }

    // Try CF-Connecting-IP (Cloudflare)
    if let Some(cf_ip) = headers.get("cf-connecting-ip") {
        if let Ok(value) = cf_ip.to_str() {
            let trimmed = value.trim();
            if trimmed.parse::<IpAddr>().is_ok() {
                return Some(trimmed.to_string());
            }
        }
    }

    // Try True-Client-IP (Akamai)
    if let Some(true_ip) = headers.get("true-client-ip") {
        if let Ok(value) = true_ip.to_str() {
            let trimmed = value.trim();
            if trimmed.parse::<IpAddr>().is_ok() {
                return Some(trimmed.to_string());
            }
        }
    }

    None
}

/// Extract user agent from request headers
pub fn extract_user_agent(headers: &HeaderMap) -> Option<String> {
    headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
}

/// Extract request ID from headers for correlation
pub fn extract_request_id(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-request-id")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
        .or_else(|| {
            headers
                .get("x-correlation-id")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string())
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn test_extract_ip_from_x_forwarded_for() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            HeaderValue::from_static("203.0.113.1, 198.51.100.1"),
        );

        let ip = extract_ip_address(&headers);
        assert_eq!(ip, Some("203.0.113.1".to_string()));
    }

    #[test]
    fn test_extract_ip_from_x_real_ip() {
        let mut headers = HeaderMap::new();
        headers.insert("x-real-ip", HeaderValue::from_static("203.0.113.1"));

        let ip = extract_ip_address(&headers);
        assert_eq!(ip, Some("203.0.113.1".to_string()));
    }

    #[test]
    fn test_extract_ip_priority() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", HeaderValue::from_static("203.0.113.1"));
        headers.insert("x-real-ip", HeaderValue::from_static("198.51.100.1"));

        let ip = extract_ip_address(&headers);
        // X-Forwarded-For should take priority
        assert_eq!(ip, Some("203.0.113.1".to_string()));
    }

    #[test]
    fn test_extract_user_agent() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "user-agent",
            HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64)"),
        );

        let ua = extract_user_agent(&headers);
        assert_eq!(
            ua,
            Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64)".to_string())
        );
    }

    #[test]
    fn test_extract_request_id() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-request-id",
            HeaderValue::from_static("550e8400-e29b-41d4-a716-446655440000"),
        );

        let req_id = extract_request_id(&headers);
        assert_eq!(
            req_id,
            Some("550e8400-e29b-41d4-a716-446655440000".to_string())
        );
    }

    #[test]
    fn test_request_context_from_headers() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", HeaderValue::from_static("203.0.113.1"));
        headers.insert("user-agent", HeaderValue::from_static("Mozilla/5.0"));
        headers.insert("x-request-id", HeaderValue::from_static("test-request-id"));

        let context = RequestContext::from_headers(&headers);
        assert_eq!(context.ip_address, Some("203.0.113.1".to_string()));
        assert_eq!(context.user_agent, Some("Mozilla/5.0".to_string()));
        assert_eq!(context.request_id, Some("test-request-id".to_string()));
    }
}
