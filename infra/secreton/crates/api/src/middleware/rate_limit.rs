// Rate Limiting Middleware for REST API
//
// Implements per-client rate limiting with HTTP 429 and Retry-After headers
// as specified in Requirements 13.4

use axum::{
    extract::{ConnectInfo, Request, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use secreton_core::services::rate_limit::{RateLimitConfig, RateLimitError, RateLimiter};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{debug, warn};

/// Rate limit middleware state
#[derive(Clone)]
pub struct RateLimitMiddleware {
    limiter: Arc<RateLimiter>,
}

impl RateLimitMiddleware {
    /// Create new rate limit middleware
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            limiter: Arc::new(RateLimiter::new(config)),
        }
    }

    /// Get the underlying rate limiter
    pub fn limiter(&self) -> &RateLimiter {
        &self.limiter
    }
}

/// Rate limit response
#[derive(Debug, Serialize, Deserialize)]
pub struct RateLimitResponse {
    pub error: String,
    pub retry_after: u64,
    pub limit: u32,
    pub remaining: u32,
}

impl IntoResponse for RateLimitResponse {
    fn into_response(self) -> Response {
        let mut headers = HeaderMap::new();

        // Add Retry-After header (in seconds)
        if let Ok(value) = HeaderValue::from_str(&self.retry_after.to_string()) {
            headers.insert("Retry-After", value);
        }

        // Add rate limit headers
        if let Ok(value) = HeaderValue::from_str(&self.limit.to_string()) {
            headers.insert("X-RateLimit-Limit", value);
        }
        if let Ok(value) = HeaderValue::from_str(&self.remaining.to_string()) {
            headers.insert("X-RateLimit-Remaining", value);
        }

        let body = serde_json::to_string(&self)
            .unwrap_or_else(|_| r#"{"error":"Rate limit exceeded"}"#.to_string());

        (StatusCode::TOO_MANY_REQUESTS, headers, body).into_response()
    }
}

/// Extract client identifier from request
fn extract_client_id(
    headers: &HeaderMap,
    connect_info: Option<&ConnectInfo<SocketAddr>>,
) -> String {
    // Try to get from X-Forwarded-For header first
    if let Some(forwarded) = headers.get("X-Forwarded-For")
        && let Ok(value) = forwarded.to_str()
    {
        // Take the first IP in the chain
        if let Some(ip) = value.split(',').next() {
            return ip.trim().to_string();
        }
    }

    // Try X-Real-IP header
    if let Some(real_ip) = headers.get("X-Real-IP")
        && let Ok(value) = real_ip.to_str()
    {
        return value.to_string();
    }

    // Fall back to connection IP
    if let Some(ConnectInfo(addr)) = connect_info {
        return addr.ip().to_string();
    }

    // Default fallback
    "unknown".to_string()
}

/// Calculate retry-after duration based on rate limit strategy
fn calculate_retry_after(config: &RateLimitConfig) -> u64 {
    use secreton_core::services::rate_limit::RateLimitStrategy;

    match &config.strategy {
        RateLimitStrategy::TokenBucket { refill_rate, .. } => {
            // Time to refill one token
            if *refill_rate > 0 {
                1 // At least 1 second
            } else {
                60 // Default to 60 seconds
            }
        }
        RateLimitStrategy::SlidingWindow { window_seconds, .. }
        | RateLimitStrategy::FixedWindow { window_seconds, .. } => {
            // Suggest waiting for window to slide
            (*window_seconds).min(60) // Cap at 60 seconds
        }
    }
}

/// Rate limiting middleware handler
pub async fn rate_limit_middleware(
    State(middleware): State<RateLimitMiddleware>,
    request: Request,
    next: Next,
) -> Response {
    let client_id = extract_client_id(request.headers(), None);

    debug!("Rate limit check for client: {}", client_id);

    // Check rate limit
    match middleware.limiter.check(&client_id).await {
        Ok(true) => {
            // Request allowed
            next.run(request).await
        }
        Ok(false) | Err(RateLimitError::LimitExceeded(_)) => {
            // Rate limit exceeded
            warn!("Rate limit exceeded for client: {}", client_id);

            let config = middleware.limiter.get_config().await;
            let retry_after = calculate_retry_after(&config);

            // Extract limit from config
            let limit = match config.strategy {
                secreton_core::services::rate_limit::RateLimitStrategy::TokenBucket {
                    capacity,
                    ..
                } => capacity,
                secreton_core::services::rate_limit::RateLimitStrategy::SlidingWindow {
                    max_requests,
                    ..
                } => max_requests,
                secreton_core::services::rate_limit::RateLimitStrategy::FixedWindow {
                    max_requests,
                    ..
                } => max_requests,
            };

            RateLimitResponse {
                error: "Rate limit exceeded. Please retry after the specified duration."
                    .to_string(),
                retry_after,
                limit,
                remaining: 0,
            }
            .into_response()
        }
        Err(e) => {
            warn!("Rate limit check error: {}", e);
            // On error, allow the request but log the issue
            next.run(request).await
        }
    }
}

/// Rate limit configuration endpoint response
#[derive(Debug, Serialize, Deserialize)]
pub struct RateLimitConfigResponse {
    pub enabled: bool,
    pub strategy: String,
    pub limit: u32,
    pub window_or_refill: String,
}

impl From<RateLimitConfig> for RateLimitConfigResponse {
    fn from(config: RateLimitConfig) -> Self {
        use secreton_core::services::rate_limit::RateLimitStrategy;

        let (strategy, limit, window_or_refill) = match config.strategy {
            RateLimitStrategy::TokenBucket {
                capacity,
                refill_rate,
            } => (
                "token_bucket".to_string(),
                capacity,
                format!("{} tokens/sec", refill_rate),
            ),
            RateLimitStrategy::SlidingWindow {
                max_requests,
                window_seconds,
            } => (
                "sliding_window".to_string(),
                max_requests,
                format!("{} seconds", window_seconds),
            ),
            RateLimitStrategy::FixedWindow {
                max_requests,
                window_seconds,
            } => (
                "fixed_window".to_string(),
                max_requests,
                format!("{} seconds", window_seconds),
            ),
        };

        Self {
            enabled: config.enabled,
            strategy,
            limit,
            window_or_refill,
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
        routing::get,
    };
    use secreton_core::services::rate_limit::RateLimitStrategy;
    use tower::ServiceExt;

    async fn test_handler() -> &'static str {
        "OK"
    }

    #[tokio::test]
    async fn test_rate_limit_allows_within_limit() {
        let config = RateLimitConfig {
            strategy: RateLimitStrategy::TokenBucket {
                capacity: 10,
                refill_rate: 1,
            },
            enabled: true,
        };

        let middleware_state = RateLimitMiddleware::new(config);

        let app =
            Router::new()
                .route("/test", get(test_handler))
                .layer(middleware::from_fn_with_state(
                    middleware_state,
                    rate_limit_middleware,
                ));

        // First request should succeed
        let response = app
            .clone()
            .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_rate_limit_blocks_over_limit() {
        let config = RateLimitConfig {
            strategy: RateLimitStrategy::TokenBucket {
                capacity: 2,
                refill_rate: 1,
            },
            enabled: true,
        };

        let middleware_state = RateLimitMiddleware::new(config);

        let app =
            Router::new()
                .route("/test", get(test_handler))
                .layer(middleware::from_fn_with_state(
                    middleware_state,
                    rate_limit_middleware,
                ));

        // First two requests should succeed
        for _ in 0..2 {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/test")
                        .header("X-Real-IP", "192.168.1.1")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::OK);
        }

        // Third request should be rate limited
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("X-Real-IP", "192.168.1.1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);

        // Check for Retry-After header
        assert!(response.headers().contains_key("Retry-After"));
    }

    #[test]
    fn test_extract_client_id_from_forwarded() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Forwarded-For",
            HeaderValue::from_static("203.0.113.1, 198.51.100.1"),
        );

        let client_id = extract_client_id(&headers, None);
        assert_eq!(client_id, "203.0.113.1");
    }

    #[test]
    fn test_extract_client_id_from_real_ip() {
        let mut headers = HeaderMap::new();
        headers.insert("X-Real-IP", HeaderValue::from_static("203.0.113.1"));

        let client_id = extract_client_id(&headers, None);
        assert_eq!(client_id, "203.0.113.1");
    }

    #[test]
    fn test_calculate_retry_after_token_bucket() {
        let config = RateLimitConfig {
            strategy: RateLimitStrategy::TokenBucket {
                capacity: 100,
                refill_rate: 10,
            },
            enabled: true,
        };

        let retry_after = calculate_retry_after(&config);
        assert_eq!(retry_after, 1);
    }

    #[test]
    fn test_calculate_retry_after_sliding_window() {
        let config = RateLimitConfig {
            strategy: RateLimitStrategy::SlidingWindow {
                max_requests: 100,
                window_seconds: 30,
            },
            enabled: true,
        };

        let retry_after = calculate_retry_after(&config);
        assert_eq!(retry_after, 30);
    }
}
