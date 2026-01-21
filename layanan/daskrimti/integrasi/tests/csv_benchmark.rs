use layanan_integrasi::client::MonsaktiClient;
use layanan_integrasi::config::Config;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use std::sync::{Arc, atomic::{AtomicBool, Ordering, AtomicU64}};

#[tokio::test]
async fn test_save_to_csv_performance() {
    // 1. Setup Client
    let config = Config {
        base: Default::default(),
        base_url: "http://localhost".to_string(),
        mysimkari_base_url: "http://localhost".to_string(),
        siman_base_url: "http://localhost".to_string(),
        siman_token_url: "http://localhost".to_string(),
        siman_client_id: None,
        siman_client_secret: None,
        siman_ba_key: None,
        tokens: HashMap::new(),
        output_dir: "./output_test_perf".to_string(),
        db_config: None,
        siman_concurrency_limit: 20,
    };
    let client = MonsaktiClient::new(config).await.expect("Failed to create client");

    // 2. Generate Large Data (200,000 records)
    let count = 200_000;
    println!("Generating {} records...", count);
    let mut data = Vec::with_capacity(count);
    for i in 0..count {
        data.push(serde_json::json!({
            "id": i,
            "name": format!("Item {}", i),
            "description": "Some long description to make the file bigger and ensure IO takes time",
            "active": true,
            "score": 123.45,
            "extra_field_1": "Just adding more data",
            "extra_field_2": "To make the row larger",
            "nested": {
                "x": 1,
                "y": 2
            }
        }));
    }
    let json_data = serde_json::Value::Array(data);

    // 3. Start Background Monitor
    let running = Arc::new(AtomicBool::new(true));
    let max_lag_micros = Arc::new(AtomicU64::new(0));

    let running_clone = running.clone();
    let max_lag_clone = max_lag_micros.clone();

    let monitor_handle = tokio::spawn(async move {
        while running_clone.load(Ordering::Relaxed) {
            let loop_start = Instant::now();
            sleep(Duration::from_millis(10)).await;
            let elapsed = loop_start.elapsed();

            // Expected sleep is 10ms. Lag is anything beyond that.
            // We use 12ms threshold to account for minor scheduling jitter.
            let lag = if elapsed > Duration::from_millis(12) {
                elapsed - Duration::from_millis(10)
            } else {
                Duration::from_millis(0)
            };

            let current_max = max_lag_clone.load(Ordering::Relaxed);
            if lag.as_micros() as u64 > current_max {
                max_lag_clone.store(lag.as_micros() as u64, Ordering::Relaxed);
            }
        }
    });

    // 4. Run Save to CSV
    println!("Saving to CSV...");
    let start_save = Instant::now();
    let filename = "perf_test.csv";

    client.save_to_csv(&json_data, filename).await.expect("Failed to save");

    let save_duration = start_save.elapsed();

    // Stop monitor
    running.store(false, Ordering::Relaxed);
    let _ = monitor_handle.await;

    let max_lag = Duration::from_micros(max_lag_micros.load(Ordering::Relaxed));

    println!("BENCHMARK_RESULT: Save duration: {:?} ms", save_duration.as_millis());
    println!("BENCHMARK_RESULT: Max event loop lag: {:?} ms", max_lag.as_millis());

    // Cleanup
    let _ = tokio::fs::remove_dir_all("./output_test_perf").await;
}
