use layanan_integrasi::client::MonsaktiClient;
use layanan_integrasi::config::Config;
use serde_json::json;
use std::collections::HashMap;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup config
    let config = Config {
        base: lib_common::config::BaseServiceConfig::default(),
        base_url: "http://localhost".to_string(),
        mysimkari_base_url: "http://localhost".to_string(),
        siman_base_url: "http://localhost".to_string(),
        siman_token_url: "http://localhost".to_string(),
        siman_client_id: None,
        siman_client_secret: None,
        siman_ba_key: None,
        tokens: HashMap::new(),
        output_dir: "./bench_output".to_string(),
        db_config: None,
        siman_concurrency_limit: 20,
    };

    // Initialize client
    // Note: new() connects to DB if db_config is present, here it is None.
    let client = MonsaktiClient::new(config).await?;

    // Generate large data
    let row_count = 100_000;
    println!("Generating {} rows of data...", row_count);
    let mut data = Vec::with_capacity(row_count);
    for i in 0..row_count {
        data.push(json!({
            "id": i,
            "name": format!("Item {}", i),
            "description": "This is a long description to make the row larger and simulate real data to test I/O and allocation overhead.",
            "value": i * 100,
            "is_active": i % 2 == 0,
            "created_at": "2023-01-01T00:00:00Z",
            "nested": {
                "x": i,
                "y": i + 1
            },
            "tags": ["a", "b", "c"]
        }));
    }
    let json_data = serde_json::Value::Array(data);

    // Measure save_to_csv
    println!("Starting benchmark for save_to_csv...");
    let start = Instant::now();
    client.save_to_csv(&json_data, "bench_result.csv").await?;
    let duration = start.elapsed();

    println!("Time taken: {:?}", duration);
    println!("Time taken (ms): {}", duration.as_millis());

    // Cleanup
    let _ = std::fs::remove_dir_all("./bench_output");

    Ok(())
}
