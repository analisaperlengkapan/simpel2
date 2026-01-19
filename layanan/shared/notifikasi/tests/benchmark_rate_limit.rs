use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use dashmap::DashMap;
use std::time::Instant;

#[tokio::test]
async fn bench_mutex_vs_dashmap() {
    let iterations = 5000;
    let concurrency = 100;

    // Mutex Benchmark
    let mutex_map: Arc<Mutex<HashMap<String, u32>>> = Arc::new(Mutex::new(HashMap::new()));
    let start = Instant::now();
    let mut handles = vec![];

    for i in 0..concurrency {
        let map = mutex_map.clone();
        handles.push(tokio::spawn(async move {
            for _ in 0..iterations {
                // High contention on a small set of keys
                let key = format!("key-{}", i % 5);
                let mut guard = map.lock().await;
                *guard.entry(key).or_insert(0) += 1;
            }
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }
    let duration_mutex = start.elapsed();
    println!("Mutex<HashMap> duration: {:?}", duration_mutex);


    // DashMap Benchmark
    let dash_map: Arc<DashMap<String, u32>> = Arc::new(DashMap::new());
    let start = Instant::now();
    let mut handles = vec![];

    for i in 0..concurrency {
        let map = dash_map.clone();
        handles.push(tokio::spawn(async move {
            for _ in 0..iterations {
                let key = format!("key-{}", i % 5);
                *map.entry(key).or_insert(0) += 1;
            }
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }
    let duration_dashmap = start.elapsed();
    println!("DashMap duration: {:?}", duration_dashmap);

    if duration_dashmap < duration_mutex {
        println!("Success: DashMap is {:.2}x faster", duration_mutex.as_secs_f64() / duration_dashmap.as_secs_f64());
    } else {
        println!("Warning: DashMap was not faster (Mutex: {:?}, DashMap: {:?})", duration_mutex, duration_dashmap);
    }
}
