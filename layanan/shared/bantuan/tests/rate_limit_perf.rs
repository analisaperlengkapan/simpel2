use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::Mutex;
use dashmap::DashMap;
use std::time::{SystemTime, UNIX_EPOCH, Instant};
use uuid::Uuid;

// Baseline: Mutex<HashMap>
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

// Optimization: DashMap
type DashMapState = Arc<DashMap<String, (u32, u64)>>;

fn dashmap_rate_limit(state: DashMapState, ip: String, limit: u32) -> bool {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    // Use DashMap's entry API
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
async fn benchmark_rate_limit_contention() {
    let num_requests = 50_000;
    // We spawn all tasks at once to let tokio scheduler handle concurrency
    let num_ips = 50; // Fewer IPs = Higher Contention on specific buckets/mutex
    let limit = 10000;

    // Generate IPs
    let mut ips = Vec::with_capacity(num_requests);
    for _ in 0..num_requests {
        let ip = format!("192.168.1.{}", (Uuid::new_v4().as_u128() % num_ips as u128));
        ips.push(ip);
    }
    let ips = Arc::new(ips);

    println!("Starting benchmark with {} requests, {} IPs...", num_requests, num_ips);

    // --- Benchmark Mutex ---
    let state_mutex = Arc::new(Mutex::new(HashMap::new()));
    let start_mutex = Instant::now();

    let mut handles = Vec::new();
    for i in 0..num_requests {
        let state = state_mutex.clone();
        let ip = ips[i].clone();
        handles.push(tokio::spawn(async move {
            mutex_rate_limit(state, ip, limit).await
        }));
    }

    for h in handles {
        let _ = h.await;
    }
    let duration_mutex = start_mutex.elapsed();
    println!("Mutex Duration: {:?}", duration_mutex);

    // --- Benchmark DashMap ---
    let state_dashmap = Arc::new(DashMap::new());
    let start_dashmap = Instant::now();

    let mut handles = Vec::new();
    for i in 0..num_requests {
        let state = state_dashmap.clone();
        let ip = ips[i].clone();
        handles.push(tokio::spawn(async move {
            dashmap_rate_limit(state, ip, limit);
        }));
    }

    for h in handles {
        let _ = h.await;
    }
    let duration_dashmap = start_dashmap.elapsed();
    println!("DashMap Duration: {:?}", duration_dashmap);

    let improvement = duration_mutex.as_secs_f64() / duration_dashmap.as_secs_f64();
    println!("Improvement: {:.2}x", improvement);

    assert!(duration_dashmap < duration_mutex, "DashMap should be faster than Mutex under contention");
}
