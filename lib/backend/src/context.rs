//! Axum-dependent extensions for RequestContext

use lib_core::context::RequestContext;
use lib_core::correlation::{CorrelationId, REQUEST_ID_HEADER};

/// Extract RequestContext from HTTP headers
pub fn request_context_from_headers(headers: &axum::http::HeaderMap) -> RequestContext {
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

    RequestContext::new(ip_address, user_agent, request_id)
}
