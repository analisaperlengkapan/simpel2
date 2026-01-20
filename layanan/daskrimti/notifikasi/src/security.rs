use crate::config::AppConfig;
use crate::error::AppError;
use axum::{extract::State, http::Request, middleware::Next, response::Response};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::time::{Duration, Instant};

pub type RateLimitState = Arc<DashMap<String, (u32, Instant)>>;

#[allow(dead_code)]
pub async fn api_key_middleware(
    State(config): State<AppConfig>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, AppError> {
    let key = req.headers().get("x-api-key").and_then(|v| v.to_str().ok());
    if key != Some(&config.api_key) {
        return Err(AppError::Unauthorized);
    }
    Ok(next.run(req).await)
}

#[allow(dead_code)]
pub async fn rate_limit_middleware(
    State(state): State<RateLimitState>,
    req: Request<axum::body::Body>,
    next: Next,
    limit: u32,
) -> Result<Response, AppError> {
    let ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let now = Instant::now();
    let window = Duration::from_secs(60);

    // Variables to store rate limit status for headers
    let remaining;
    let reset_secs;

    {
        let mut entry = state.entry(ip).or_insert((0, now));
        let (count, start_time) = entry.value_mut();

        if now.duration_since(*start_time) > window {
            *count = 1;
            *start_time = now;
            remaining = limit - 1;
            reset_secs = 60;
        } else {
            if *count >= limit {
                return Err(AppError::RateLimit);
            }
            *count += 1;
            remaining = limit - *count;
            reset_secs = window
                .checked_sub(now.duration_since(*start_time))
                .unwrap_or_else(|| Duration::from_secs(0))
                .as_secs();
        }
    } // Entry lock dropped here

    let mut response = next.run(req).await;

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
