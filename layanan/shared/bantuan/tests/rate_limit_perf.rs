use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::Mutex;
use dashmap::DashMap;
use std::time::{SystemTime, UNIX_EPOCH, Instant};
use uuid::Uuid;

// Baseline implementation: Mutex<HashMap>
type MutexState = Arc<Mutex<HashMap<String, (u32, u64)>>>;

async fn mutex_rate_limit(state: MutexState, ip: String, limit: u32) -> bool {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let mut lock = state.lock().await;
    let val = lock.entry(ip).or_insert((0, now));

    if now - val.1 > 60 {
        *val = (1, now);
        true
    } else {
        if val.0 >= limit {
            false
        } else {
            val.0 += 1;
            true
        }
    }
}

// Optimized implementation: DashMap
// This matches the existing code in src/rate_limit.rs (before my proposed cleanup changes)
type DashMapState = Arc<DashMap<String, (u32, u64)>>;

fn dashmap_rate_limit(state: DashMapState, ip: String, limit: u32) -> bool {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    // Use DashMap's entry API which locks only the specific bucket/entry
    let mut entry = state.entry(ip).or_insert((0, now));
    let val = entry.value_mut();

    if now - val.1 > 60 {
        *val = (1, now);
        true
    } else {
        if val.0 >= limit {
            false
        } else {
            val.0 += 1;
            true
        }
    }
}

#[tokio::test]
async fn benchmark_rate_limiters() {
    // Parameters for the benchmark
    let num_requests = 50_000;
    let num_ips = 500; // Simulate some contention
    let limit = 1000;

    println!("Benchmarking with {} requests, {} unique IPs...", num_requests, num_ips);

    // Generate random IPs
    let mut ips = Vec::with_capacity(num_requests);
    for _ in 0..num_requests {
        // Simple IP generation
        let ip = format!("192.168.1.{}", (Uuid::new_v4().as_u128() % num_ips as u128));
        ips.push(ip);
    }
    let ips = Arc::new(ips);

    // --- Benchmark Mutex ---
    let state = Arc::new(Mutex::new(HashMap::new()));
    let start = Instant::now();
    let mut handles = Vec::new();

    for i in 0..num_requests {
        let state = state.clone();
        let ip = ips[i].clone();
        handles.push(tokio::spawn(async move {
            mutex_rate_limit(state, ip, limit).await
        }));
    }

    for h in handles {
        h.await.unwrap();
    }
    let duration_mutex = start.elapsed();
    println!("Mutex Duration: {:.2?}", duration_mutex);

    // --- Benchmark DashMap ---
    let state = Arc::new(DashMap::new());
    let start = Instant::now();
    let mut handles = Vec::new();

    for i in 0..num_requests {
        let state = state.clone();
        let ip = ips[i].clone();
        handles.push(tokio::spawn(async move {
            dashmap_rate_limit(state, ip, limit);
        }));
    }

    for h in handles {
        h.await.unwrap();
    }
    let duration_dashmap = start.elapsed();
    println!("DashMap Duration: {:.2?}", duration_dashmap);

    // Check for improvement
    if duration_dashmap < duration_mutex {
        let improvement = duration_mutex.as_secs_f64() / duration_dashmap.as_secs_f64();
        println!("DashMap is {:.2}x faster", improvement);
    } else {
        println!("DashMap is NOT faster (Mutex: {:?}, DashMap: {:?})", duration_mutex, duration_dashmap);
    }
}
