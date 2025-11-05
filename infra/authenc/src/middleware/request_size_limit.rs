// SPDX-License-Identifier: Apache-2.0
//! Request size limit middleware
//!
//! Enforces maximum request body size to prevent DoS attacks and resource exhaustion.

use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use http_body_util::Limited;

/// Maximum request body size (1MB)
pub const MAX_REQUEST_BODY_SIZE: usize = 1_048_576;

/// Middleware to enforce request body size limits
///
/// This middleware checks the Content-Length header and rejects requests
/// that exceed the maximum allowed size before reading the body.
pub async fn request_size_limit_middleware(
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    // Check Content-Length header if present
    if let Some(content_length) = request.headers().get(http::header::CONTENT_LENGTH) {
        if let Ok(length_str) = content_length.to_str() {
            if let Ok(length) = length_str.parse::<usize>() {
                if length > MAX_REQUEST_BODY_SIZE {
                    tracing::warn!(
                        "Request body too large: {} bytes (max: {} bytes)",
                        length,
                        MAX_REQUEST_BODY_SIZE
                    );
                    return Err((
                        StatusCode::PAYLOAD_TOO_LARGE,
                        format!(
                            "Request body too large. Maximum size is {} bytes (1MB)",
                            MAX_REQUEST_BODY_SIZE
                        ),
                    )
                        .into_response());
                }
            }
        }
    }

    // For chunked encoding or missing Content-Length, we need to check during body reading
    // This is handled by limiting the body size during extraction
    let (parts, body) = request.into_parts();

    // Limit the body size using Limited wrapper
    let limited_body = Limited::new(body, MAX_REQUEST_BODY_SIZE);

    // Reconstruct the request with the limited body
    let request = Request::from_parts(parts, Body::new(limited_body));

    // Continue with the request
    Ok(next.run(request).await)
}

/// Tower layer for request size limiting
///
/// This can be used with tower's layer system for more flexible middleware composition.
pub mod layer {
    use super::*;
    use std::task::{Context, Poll};
    use tower::{Layer, Service};

    /// Request size limit layer
    #[derive(Clone)]
    pub struct RequestSizeLimitLayer {
        max_size: usize,
    }

    impl RequestSizeLimitLayer {
        /// Create a new request size limit layer with the default maximum size
        pub fn new() -> Self {
            Self {
                max_size: MAX_REQUEST_BODY_SIZE,
            }
        }

        /// Create a new request size limit layer with a custom maximum size
        pub fn with_max_size(max_size: usize) -> Self {
            Self { max_size }
        }
    }

    impl Default for RequestSizeLimitLayer {
        fn default() -> Self {
            Self::new()
        }
    }

    impl<S> Layer<S> for RequestSizeLimitLayer {
        type Service = RequestSizeLimitService<S>;

        fn layer(&self, inner: S) -> Self::Service {
            RequestSizeLimitService {
                inner,
                max_size: self.max_size,
            }
        }
    }

    /// Request size limit service
    #[derive(Clone)]
    pub struct RequestSizeLimitService<S> {
        inner: S,
        max_size: usize,
    }

    impl<S> Service<Request> for RequestSizeLimitService<S>
    where
        S: Service<Request, Response = Response> + Clone + Send + 'static,
        S::Future: Send + 'static,
    {
        type Response = S::Response;
        type Error = S::Error;
        type Future = std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
        >;

        fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.inner.poll_ready(cx)
        }

        fn call(&mut self, request: Request) -> Self::Future {
            let max_size = self.max_size;
            let mut inner = self.inner.clone();

            Box::pin(async move {
                // Check Content-Length header
                if let Some(content_length) = request.headers().get(http::header::CONTENT_LENGTH) {
                    if let Ok(length_str) = content_length.to_str() {
                        if let Ok(length) = length_str.parse::<usize>() {
                            if length > max_size {
                                tracing::warn!(
                                    "Request body too large: {} bytes (max: {} bytes)",
                                    length,
                                    max_size
                                );
                                return Ok((
                                    StatusCode::PAYLOAD_TOO_LARGE,
                                    format!(
                                        "Request body too large. Maximum size is {} bytes",
                                        max_size
                                    ),
                                )
                                    .into_response());
                            }
                        }
                    }
                }

                // Limit body size
                let (parts, body) = request.into_parts();
                let limited_body = Limited::new(body, max_size);
                let request = Request::from_parts(parts, Body::new(limited_body));

                inner.call(request).await
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
        middleware,
        response::IntoResponse,
        routing::post,
    };
    use tower::ServiceExt;

    async fn test_handler() -> impl IntoResponse {
        "OK"
    }

    #[tokio::test]
    async fn test_request_within_limit() {
        let app = Router::new()
            .route("/test", post(test_handler))
            .layer(middleware::from_fn(request_size_limit_middleware));

        let body = "x".repeat(1000); // 1KB
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
    async fn test_request_exceeds_limit() {
        let app = Router::new()
            .route("/test", post(test_handler))
            .layer(middleware::from_fn(request_size_limit_middleware));

        let size = MAX_REQUEST_BODY_SIZE + 1;
        let request = Request::builder()
            .method("POST")
            .uri("/test")
            .header("content-length", size.to_string())
            .body(Body::from("x".repeat(size)))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }
}
