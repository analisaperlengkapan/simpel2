use super::rate_limit::{RateLimitConfig, RateLimiterState};
use axum::{Router, body::Body, http::Request, routing::get};
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn test_rate_limiting() {
    // Configure rate limiting to allow 2 requests per minute for testing
    let config = RateLimitConfig {
        requests_per_minute: 2,
        excluded_paths: vec!["/health".to_string()],
        enabled: true,
        base_delay_ms: 0,
        max_delay_ms: 0,
        progressive_delays: false,
    };

    let state = Arc::new(RateLimiterState::new(config));

    let app = Router::new()
        .route("/test", get(|| async { "Hello, world!" }))
        .route("/health", get(|| async { "OK" }))
        .layer(super::rate_limit::RateLimitLayer::new((*state).clone()));

    // First request should succeed
    let response = app
        .clone()
        .oneshot(Request::get("/test").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), 200);

    // Second request should succeed
    let response = app
        .clone()
        .oneshot(Request::get("/test").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), 200);

    // Third request should be rate limited
    let response = app
        .clone()
        .oneshot(Request::get("/test").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), 429);

    // Health check should not be rate limited
    let response = app
        .clone()
        .oneshot(Request::get("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}
