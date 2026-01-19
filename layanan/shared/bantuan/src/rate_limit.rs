use crate::error::AppError;
use crate::AppState;
use axum::{extract::State, http::Request, middleware::Next, response::Response};
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

pub type RateLimitState = Arc<DashMap<String, (u32, u64)>>;

pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, AppError> {
    let limit = state.config.rate_limit_ticket; // Use a default from config (or similar)
    let rate_limit_map = &state.rate_limit;

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
        let map_clone = rate_limit_map.clone();
        tokio::spawn(async move {
            map_clone.retain(|_, (_, ts)| now - *ts <= 60);
        });
    }

    // Use DashMap's entry API which locks only the specific bucket/entry
    let mut entry = rate_limit_map.entry(ip).or_insert((0, now));
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
