//! Axum-dependent extensions for CorrelationId

use lib_core::context::RequestContext;
use lib_core::correlation::{CorrelationId, REQUEST_ID_HEADER};

/// Try to extract CorrelationId from HTTP headers
pub fn correlation_from_headers(headers: &axum::http::HeaderMap) -> Option<CorrelationId> {
    headers
        .get(REQUEST_ID_HEADER)
        .and_then(|h| h.to_str().ok())
        .map(|s| CorrelationId::new_from_string(s.to_string()))
}

/// Axum middleware for correlation ID injection
pub async fn correlation_id_middleware(
    mut request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let headers = request.headers();
    let correlation_id = correlation_from_headers(headers).unwrap_or_default();

    // Add RequestContext to request extensions
    let context = request_context_from_headers(headers);
    request.extensions_mut().insert(context);

    let mut response = next.run(request).await;

    // Inject into response headers
    if let Ok(value) = correlation_id.as_str().parse::<axum::http::HeaderValue>() {
        response.headers_mut().insert(REQUEST_ID_HEADER, value);
    }

    response
}

/// Extract RequestContext from HTTP headers (used internally)
fn request_context_from_headers(headers: &axum::http::HeaderMap) -> RequestContext {
    crate::context::request_context_from_headers(headers)
}
