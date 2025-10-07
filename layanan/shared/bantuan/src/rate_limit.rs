use crate::error::AppError;
use axum::{extract::State, http::Request, middleware::Next, response::Response};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

pub type RateLimitState = Arc<Mutex<HashMap<String, (u32, u64)>>>;

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
    let mut map = state.lock().await;
    let entry = map.entry(ip.clone()).or_insert((0, now));
    if now - entry.1 > 60 {
        *entry = (1, now);
    } else {
        if entry.0 >= limit {
            return Err(AppError::RateLimit);
        }
        entry.0 += 1;
    }
    Ok(next.run(req).await)
}
