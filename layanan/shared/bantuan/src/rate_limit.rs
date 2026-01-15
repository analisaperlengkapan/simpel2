use crate::error::AppError;
use axum::{extract::State, http::Request, middleware::Next, response::Response};
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[allow(dead_code)]
pub struct RateLimitInner {
    pub map: DashMap<String, (u32, u64)>,
    pub last_cleanup: AtomicU64,
}

#[allow(dead_code)]
pub type RateLimitState = Arc<RateLimitInner>;

#[allow(dead_code)]
impl RateLimitInner {
    pub fn new() -> Self {
        Self {
            map: DashMap::new(),
            last_cleanup: AtomicU64::new(0),
        }
    }
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
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Cleanup logic: Run at most once every 60 seconds
    // This prevents the map from growing indefinitely (memory leak protection)
    let last = state.last_cleanup.load(Ordering::Relaxed);
    if now > last + 60 {
        // Try to update last_cleanup. Only one thread will succeed.
        if state
            .last_cleanup
            .compare_exchange(last, now, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
        {
            // Remove entries older than 60 seconds
            // retain is efficient in DashMap as it works on shards
            state.map.retain(|_, (_, timestamp)| now - *timestamp <= 60);
        }
    }

    // Use DashMap's entry API which locks only the specific bucket/entry
    let mut entry = state.map.entry(ip).or_insert((0, now));
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
