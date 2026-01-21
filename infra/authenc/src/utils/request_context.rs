//! Request context extraction utilities (Renovated to use lib_common)

pub use lib_common::context::RequestContext;

/// Extract IP address (Legacy helper, now using lib_common logic internally)
pub fn extract_ip_address(headers: &axum::http::HeaderMap) -> Option<String> {
    RequestContext::from_headers(headers).ip_address
}

/// Extract user agent (Legacy helper)
pub fn extract_user_agent(headers: &axum::http::HeaderMap) -> Option<String> {
    RequestContext::from_headers(headers).user_agent
}

/// Extract request ID (Legacy helper)
pub fn extract_request_id(headers: &axum::http::HeaderMap) -> Option<String> {
    Some(RequestContext::from_headers(headers).request_id.to_string())
}
