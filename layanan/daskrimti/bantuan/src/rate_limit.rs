use crate::AppState;
use crate::error::AppError;
use axum::{extract::State, http::Request, middleware::Next, response::Response};
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

pub type RateLimitState = Arc<DashMap<String, (u32, u64)>>;

pub async fn check_rate_limit(
    rate_limit_map: &RateLimitState,
    limit: u32,
    ip: String,
) -> Result<(), AppError> {
    let now_duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let now = now_duration.as_secs();

    // Probabilistic cleanup (approx 1 in 100 requests) to prevent memory leaks
    if now_duration.subsec_nanos().is_multiple_of(100) {
        let map_clone = rate_limit_map.clone();
        tokio::spawn(async move {
            map_clone.retain(|_, (_, ts)| now - *ts <= 60);
        });
    }

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

    Ok(())
}

pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, AppError> {
    let limit = state.config.rate_limit_ticket;

    let ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    check_rate_limit(&state.rate_limit, limit, ip).await?;

    Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Instant;

    #[tokio::test]
    async fn benchmark_rate_limit_contention() {
        let state: RateLimitState = Arc::new(DashMap::new());
        let limit = 1000000;

        let concurrency = 50;
        let requests_per_task = 2000;
        let total_requests = concurrency * requests_per_task;

        let start = Instant::now();

        let mut tasks = Vec::new();
        for _ in 0..concurrency {
            let state = state.clone();
            tasks.push(tokio::spawn(async move {
                for j in 0..requests_per_task {
                    let ip = format!("192.168.1.{}", j % 100);
                    let _ = check_rate_limit(&state, limit, ip).await;
                }
            }));
        }

        for task in tasks {
            task.await.unwrap();
        }

        let duration = start.elapsed();
        println!(
            "Benchmark Result: {} requests in {:?} (Concurrency: {})",
            total_requests, duration, concurrency
        );
        println!(
            "Throughput: {:.2} req/s",
            total_requests as f64 / duration.as_secs_f64()
        );
    }
}
