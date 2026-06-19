//! # Request Size Limit Middleware
//!
//! Enforces maximum request body size to prevent DoS attacks and resource exhaustion.
//! Uses both Content-Length header checks and streaming-level enforcement.

use axum::{
    body::Body,
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};
use http_body_util::Limited;

use crate::shared::error::AppError;
use crate::state::MaxRequestSize;

/// Default maximum request size: 10MB (suitable for document uploads)
pub const DEFAULT_MAX_SIZE: usize = 10 * 1024 * 1024;

/// Middleware to enforce request body size limits.
///
/// This middleware protects the service by:
/// 1. Rejecting requests immediately if the `Content-Length` header exceeds the limit.
/// 2. Wrapping the body stream with a limit to handle chunked encoding or dishonest headers.
pub async fn request_size_limit_middleware(
    State(MaxRequestSize(max_size)): State<MaxRequestSize>,
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    // 1. Pre-emptive check: Content-Length header
    if let Some(content_length) = request.headers().get(http::header::CONTENT_LENGTH) {
        if let Ok(length_str) = content_length.to_str() {
            if let Ok(length) = length_str.parse::<usize>() {
                if length > max_size {
                    tracing::warn!(
                        "Request body too large (Content-Length): {} bytes (max: {} bytes)",
                        length,
                        max_size
                    );
                    return Err(AppError::PayloadTooLarge(format!(
                        "Request body too large. Maximum size is {} bytes",
                        max_size
                    ))
                    .into_response());
                }
            }
        }
    }

    // 2. Stream-level enforcement: Wrap body with Limited
    // This protects against chunked encoding and missing Content-Length headers.
    let (parts, body) = request.into_parts();
    let limited_body = Body::new(Limited::new(body, max_size));
    request = Request::from_parts(parts, limited_body);

    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
        middleware,
        routing::post,
    };
    use tower::ServiceExt;

    async fn test_handler(_body: String) -> impl IntoResponse {
        "OK"
    }

    #[tokio::test]
    async fn test_size_limit_within_bounds() {
        let max_size = MaxRequestSize(1024 * 1024); // 1MB
        let app = Router::new()
            .route("/test", post(test_handler))
            .layer(middleware::from_fn_with_state(max_size, request_size_limit_middleware));

        let body = "a".repeat(1024); // 1KB
        let request = Request::builder()
            .method("POST")
            .uri("/test")
            .header("content-length", body.len().to_string())
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_size_limit_exceeded_via_header() {
        let max_size = MaxRequestSize(100); // 100 bytes
        let app = Router::new()
            .route("/test", post(test_handler))
            .layer(middleware::from_fn_with_state(max_size, request_size_limit_middleware));

        let body = "x".repeat(101);
        let request = Request::builder()
            .method("POST")
            .uri("/test")
            .header("content-length", body.len().to_string())
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[tokio::test]
    async fn test_size_limit_exceeded_streaming() {
        let max_size = MaxRequestSize(1024); // 1KB
        let app = Router::new()
            .route("/test", post(test_handler))
            .layer(middleware::from_fn_with_state(max_size, request_size_limit_middleware));

        let size = 2048;
        let request = Request::builder()
            .method("POST")
            .uri("/test")
            // No content-length to force streaming check
            .body(Body::from(vec![0u8; size]))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }
}
