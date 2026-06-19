//! Security middleware for Axum backend services
//!
//! Provides:
//! - Rate limiting
//! - CSRF validation
//! - Security headers
//! - Input validation

use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    middleware::Next,
    response::Response,
};
use http_body_util::Limited;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;

/// Security headers middleware
/// Adds essential security headers to all responses
pub async fn security_headers_middleware(
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    // Strict-Transport-Security (HSTS)
    headers.insert(
        header::STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );

    // X-Frame-Options (prevent clickjacking)
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));

    // X-Content-Type-Options (prevent MIME sniffing)
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );

    // X-XSS-Protection
    headers.insert(
        "X-XSS-Protection",
        HeaderValue::from_static("1; mode=block"),
    );

    // Referrer-Policy
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // Content-Security-Policy
    let csp = "default-src 'self'; \
               script-src 'self' 'wasm-unsafe-eval'; \
               style-src 'self' 'unsafe-inline'; \
               img-src 'self' data: https:; \
               font-src 'self' data:; \
               connect-src 'self' https://api.simpelv2.kejaksaan.go.id; \
               frame-ancestors 'none'; \
               base-uri 'self'; \
               form-action 'self'";

    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(csp),
    );

    // Permissions-Policy
    headers.insert(
        "Permissions-Policy",
        HeaderValue::from_static("geolocation=(), microphone=(), camera=()"),
    );

    Ok(response)
}

/// Rate limiter state
#[derive(Clone)]
pub struct RateLimiter {
    requests: Arc<RwLock<HashMap<String, Vec<SystemTime>>>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
            max_requests,
            window,
        }
    }

    /// Check if request is allowed
    pub async fn check(&self, key: &str) -> bool {
        let mut requests = self.requests.write().await;
        let now = SystemTime::now();

        // Get or create request history for this key
        let history = requests.entry(key.to_string()).or_insert_with(Vec::new);

        // Remove expired requests
        history.retain(|&time| {
            now.duration_since(time).unwrap_or(Duration::from_secs(0)) < self.window
        });

        // Check if limit exceeded
        if history.len() >= self.max_requests {
            return false;
        }

        // Add current request
        history.push(now);
        true
    }

    /// Clean up old entries periodically
    pub async fn cleanup(&self) {
        let mut requests = self.requests.write().await;
        let now = SystemTime::now();

        requests.retain(|_, history| {
            history.retain(|&time| {
                now.duration_since(time).unwrap_or(Duration::from_secs(0)) < self.window
            });
            !history.is_empty()
        });
    }
}

/// Rate limiting middleware
/// Limits requests per IP address
pub async fn rate_limit_middleware(
    State(limiter): State<Arc<RateLimiter>>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract client IP from headers or connection info
    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .unwrap_or("unknown")
        .to_string();

    // Check rate limit
    if !limiter.check(&client_ip).await {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(request).await)
}

/// CSRF token validation middleware
/// Validates CSRF token from X-CSRF-Token header
pub async fn csrf_validation_middleware(
    headers: HeaderMap,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip CSRF validation for GET, HEAD, OPTIONS
    let method = request.method();
    if method == "GET" || method == "HEAD" || method == "OPTIONS" {
        return Ok(next.run(request).await);
    }

    // Extract CSRF token from header
    let token = headers
        .get("X-CSRF-Token")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::FORBIDDEN)?;

    // Extract CSRF token from cookie
    let cookie_token = headers
        .get(header::COOKIE)
        .and_then(|h| h.to_str().ok())
        .and_then(|cookies| {
            cookies
                .split(';')
                .find(|c| c.trim().starts_with("csrf_token="))
                .and_then(|c| c.split('=').nth(1))
        })
        .ok_or(StatusCode::FORBIDDEN)?;

    // Validate tokens match (constant-time comparison)
    if !constant_time_compare(token, cookie_token) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(request).await)
}

/// Constant-time string comparison to prevent timing attacks
fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (byte_a, byte_b) in a.bytes().zip(b.bytes()) {
        result |= byte_a ^ byte_b;
    }

    result == 0
}

/// Maximum request body size (10MB)
pub const MAX_REQUEST_BODY_SIZE: usize = 10 * 1024 * 1024;

/// Input validation middleware
/// Validates request body size and content type robustly
pub async fn input_validation_middleware(
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let method = request.method().clone();
    let headers = request.headers();

    // 1. Pre-emptive check: Check Content-Length header if present
    if let Some(length) = headers
        .get(header::CONTENT_LENGTH)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<usize>().ok())
        && length > MAX_REQUEST_BODY_SIZE
    {
        tracing::warn!(
            "Request body too large (Content-Length): {} bytes (max: {} bytes)",
            length,
            MAX_REQUEST_BODY_SIZE
        );
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }

    // 2. Validate Content-Type for POST/PUT/PATCH
    if method == "POST" || method == "PUT" || method == "PATCH" {
        if let Some(content_type) = headers.get(header::CONTENT_TYPE) {
            let content_type_str = content_type.to_str().unwrap_or("");

            // Allow only JSON and form data
            if !content_type_str.starts_with("application/json")
                && !content_type_str.starts_with("application/x-www-form-urlencoded")
                && !content_type_str.starts_with("multipart/form-data")
            {
                return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE);
            }
        } else {
            return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE);
        }
    }

    // 3. Stream-time check: Wrap body with a limit to handle missing Content-Length/chunked encoding
    // This is the primary defense against bodies that lie about their size or use chunked encoding.
    let (parts, body) = request.into_parts();
    let limited_body = Body::new(Limited::new(body, MAX_REQUEST_BODY_SIZE));
    let request = Request::from_parts(parts, limited_body);

    Ok(next.run(request).await)
}

/// Combine all security middleware
pub fn security_middleware_stack<S>() -> impl tower::Layer<S>
where
    S: Send + 'static,
{
    tower::ServiceBuilder::new()
        .layer(axum::middleware::from_fn::<_, Body>(
            security_headers_middleware,
        ))
        .layer(axum::middleware::from_fn::<_, Body>(
            input_validation_middleware,
        ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_time_compare() {
        assert!(constant_time_compare("abc123", "abc123"));
        assert!(!constant_time_compare("abc123", "abc124"));
        assert!(!constant_time_compare("abc123", "abc12"));
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = RateLimiter::new(5, Duration::from_secs(60));

        // First 5 requests should succeed
        for _ in 0..5 {
            assert!(limiter.check("test_ip").await);
        }

        // 6th request should fail
        assert!(!limiter.check("test_ip").await);

        // Different IP should succeed
        assert!(limiter.check("other_ip").await);
    }

    #[tokio::test]
    async fn test_input_validation_body_limit_exceeded() {
        use axum::{Router, http::Request, middleware, routing::post};
        use tower::ServiceExt;

        let app = Router::new()
            .route("/test", post(|_body: String| async { "OK" }))
            .layer(middleware::from_fn(input_validation_middleware));

        // Create a body that exceeds 10MB
        let large_body = "x".repeat(MAX_REQUEST_BODY_SIZE + 1);

        // Case 1: Exceeds limit via Content-Length header
        let request = Request::builder()
            .method("POST")
            .uri("/test")
            .header("content-type", "application/json")
            .header("content-length", large_body.len())
            .body(Body::from(large_body.clone()))
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);

        // Case 2: Exceeds limit WITHOUT Content-Length header (simulating chunked)
        // Note: Axum/Hyper will still enforce the limit during body reading because of the Limited wrapper.
        // We use a body that doesn't have a content-length header.
        let request_no_len = Request::builder()
            .method("POST")
            .uri("/test")
            .header("content-type", "application/json")
            .body(Body::from(large_body))
            .unwrap();

        let response = app.oneshot(request_no_len).await.unwrap();
        // Since we are reading the whole body into a String in the handler,
        // the Limited body will return an error when it reaches the limit,
        // and Axum's String extractor will return 413 Payload Too Large.
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }
}
