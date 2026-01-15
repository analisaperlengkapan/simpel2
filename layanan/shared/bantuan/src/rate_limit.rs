use crate::error::AppError;
use axum::{extract::State, http::Request, middleware::Next, response::Response};
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[allow(dead_code)]
pub type RateLimitState = Arc<DashMap<String, (u32, u64)>>;

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
    let now_duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let now = now_duration.as_secs();

    // Probabilistic cleanup (approx 1 in 100 requests) to prevent memory leaks
    if now_duration.subsec_nanos() % 100 == 0 {
        let state_clone = state.clone();
        tokio::spawn(async move {
            state_clone.retain(|_, (_, ts)| now - *ts <= 60);
        });
    }

    // Use DashMap's entry API which locks only the specific bucket/entry
    let mut entry = state.entry(ip).or_insert((0, now));
    let val = entry.value_mut();

    if now - val.1 > 60 {
        *val = (1, now);
    } else {
        if val.0 >= limit {
            return Err(AppError::RateLimit);
        }
        val.0 += 1;
    }
    drop(entry); // Explicitly drop to release the lock early

    Ok(next.run(req).await)
}
