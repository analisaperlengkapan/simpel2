use crate::error::AppError;
use axum::{extract::State, http::Request, middleware::Next, response::Response};
use moka::future::Cache;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[allow(dead_code)]
pub type RateLimitState = Cache<String, Arc<AtomicU32>>;

#[allow(dead_code)]
pub fn new_rate_limiter() -> RateLimitState {
    Cache::builder()
        .time_to_live(Duration::from_secs(60))
        .build()
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

    // Use moka's get_with to atomically get or initialize the counter.
    // moka handles the TTL (60s). If expired, it's gone, so get_with creates new.
    let counter = state.get_with(ip, async {
        Arc::new(AtomicU32::new(0))
    }).await;

    // Increment and check
    let count = counter.fetch_add(1, Ordering::SeqCst) + 1;
    if count > limit {
        return Err(AppError::RateLimit);
    }

    Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tokio::sync::Mutex;
    use std::sync::Arc;
    use std::time::{SystemTime, UNIX_EPOCH, Instant};
    use dashmap::DashMap;

    #[tokio::test]
    async fn benchmark_rate_limit_implementations() {
        let concurrency = 50;
        let requests_per_task = 1000;
        let total_requests = concurrency * requests_per_task;

        println!("Benchmarking {} total requests with concurrency {}", total_requests, concurrency);

        // 1. Mutex<HashMap> (Baseline)
        let mutex_map: Arc<Mutex<HashMap<String, (u32, u64)>>> = Arc::new(Mutex::new(HashMap::new()));
        let start = Instant::now();
        let mut handles = vec![];

        for i in 0..concurrency {
            let map = mutex_map.clone();
            handles.push(tokio::spawn(async move {
                for _ in 0..requests_per_task {
                    let ip = format!("192.168.1.{}", i % 10); // Simulate some contention on same IPs
                    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                    let mut lock = map.lock().await;
                    let val = lock.entry(ip).or_insert((0, now));
                    if now - val.1 > 60 {
                        *val = (1, now);
                    } else {
                        val.0 += 1;
                    }
                }
            }));
        }
        for handle in handles {
            handle.await.unwrap();
        }
        let duration = start.elapsed();
        println!("Mutex<HashMap>: {:?}", duration);


        // 2. DashMap (Previous)
        let dash_map: Arc<DashMap<String, (u32, u64)>> = Arc::new(DashMap::new());
        let start = Instant::now();
        let mut handles = vec![];

        for i in 0..concurrency {
            let map = dash_map.clone();
            handles.push(tokio::spawn(async move {
                for _ in 0..requests_per_task {
                    let ip = format!("192.168.1.{}", i % 10);
                    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

                    let mut entry = map.entry(ip).or_insert((0, now));
                    let val = entry.value_mut();
                    if now - val.1 > 60 {
                        *val = (1, now);
                    } else {
                        val.0 += 1;
                    }
                }
            }));
        }
        for handle in handles {
            handle.await.unwrap();
        }
        let duration = start.elapsed();
        println!("DashMap: {:?}", duration);

        // 3. moka (New)
        let moka_cache: RateLimitState = new_rate_limiter();

        let start = Instant::now();
        let mut handles = vec![];

        for i in 0..concurrency {
            let cache = moka_cache.clone();
            handles.push(tokio::spawn(async move {
                for _ in 0..requests_per_task {
                    let ip = format!("192.168.1.{}", i % 10);
                    let counter = cache.get_with(ip, async {
                        Arc::new(AtomicU32::new(0))
                    }).await;
                    counter.fetch_add(1, Ordering::Relaxed);
                }
            }));
        }
        for handle in handles {
            handle.await.unwrap();
        }
        let duration = start.elapsed();
        println!("moka (TTL=60s): {:?}", duration);
    }
}
