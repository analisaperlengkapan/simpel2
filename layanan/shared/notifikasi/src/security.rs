use crate::config::AppConfig;
use crate::error::AppError;
use axum::{http::Request, middleware::Next, response::Response, extract::State};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

pub type RateLimitState = Arc<Mutex<HashMap<String, (u32, u64)>>>;

pub async fn api_key_middleware<B>(
    State(config): State<AppConfig>,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, AppError> {
    let key = req.headers().get("x-api-key").and_then(|v| v.to_str().ok());
    if key != Some(&config.api_key) {
        return Err(AppError::Unauthorized);
    }
    Ok(next.run(req).await)
}

pub async fn rate_limit_middleware<B>(
    State(state): State<RateLimitState>,
    req: Request<B>,
    next: Next<B>,
    limit: u32,
) -> Result<Response, AppError> {
    let ip = req.headers().get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown").to_string();
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
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