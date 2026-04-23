#![cfg(feature = "axum")]

use crate::CommonError;
use axum::{
    body::Body,
    extract::State,
    http::{Request, Response},
    middleware::Next,
};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::time::{Duration, Instant};

/// State for Rate Limiting middleware
pub type RateLimitState = Arc<DashMap<String, (u32, Instant)>>;

/// Rate limiting middleware
///
/// Generic rate limiter based on IP address.
pub async fn rate_limit_middleware(
    state: State<RateLimitState>,
    req: Request<Body>,
    next: Next,
    limit: u32,
    window_secs: u64,
) -> Result<Response<Body>, CommonError> {
    let ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v: &axum::http::HeaderValue| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim())
        .unwrap_or("unknown")
        .to_string();

    let now = Instant::now();
    let window = Duration::from_secs(window_secs);

    let remaining;
    let reset_secs;

    {
        let mut entry = state.entry(ip).or_insert((0, now));
        let (count, start_time) = entry.value_mut();

        if now.duration_since(*start_time) > window {
            *count = 1;
            *start_time = now;
            remaining = limit - 1;
            reset_secs = window_secs;
        } else {
            if *count >= limit {
                return Err(CommonError::Internal("Rate limit exceeded".to_string()));
            }
            *count += 1;
            remaining = limit - *count;
            reset_secs = window
                .checked_sub(now.duration_since(*start_time))
                .unwrap_or_else(|| Duration::from_secs(0))
                .as_secs();
        }
    }

    let mut response: Response<Body> = next.run(req).await;

    // Add Rate Limit Headers
    response
        .headers_mut()
        .insert("X-RateLimit-Limit", limit.to_string().parse().unwrap());
    response.headers_mut().insert(
        "X-RateLimit-Remaining",
        remaining.to_string().parse().unwrap(),
    );
    response
        .headers_mut()
        .insert("X-RateLimit-Reset", reset_secs.to_string().parse().unwrap());

    Ok(response)
}
