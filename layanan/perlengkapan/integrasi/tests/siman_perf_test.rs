use layanan_integrasi::siman::{SimanAssetCategory, fetch_all_assets_with_pagination};
use layanan_integrasi::{Config, MonsaktiClient, StorageStrategy};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_parallel_fetching_performance() {
    // 1. Start a mock server
    let mock_server = MockServer::start().await;

    // 2. Setup Mock Responses

    // Mock Authentication (SIMAN Token)
    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "mock_token",
            "expires_in": 3600,
            "token_type": "Bearer"
        })))
        .mount(&mock_server)
        .await;

    // Mock Row Count
    // Return 50,000 records to generate 50 chunks (chunk_size=1000)
    Mock::given(method("GET"))
        .and(path_regex(r"^/gateway/SLDKSimanKL/2.0/getRowCount/.*"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": [
                {
                    "RCOUNT": 50000
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    // Mock Data Endpoint
    // Simulate 100ms latency per request
    Mock::given(method("POST"))
        .and(path_regex(r"^/gateway/SLDKSimanKL/2.0/.*"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({
                    "results": [
                        {"id_aset": "1", "nama_aset": "Dummy Asset 1"},
                        {"id_aset": "2", "nama_aset": "Dummy Asset 2"}
                    ]
                }))
                .set_delay(Duration::from_millis(100)),
        )
        .mount(&mock_server)
        .await;

    // 3. Configure Client
    let config = Config {
        base: lib_common::config::BaseServiceConfig::default(),
        base_url: "http://mock".to_string(),
        mysimkari_base_url: "http://mock".to_string(),
        siman_base_url: mock_server.uri(),
        siman_token_url: format!("{}/token", mock_server.uri()),
        siman_client_id: Some("mock_client".to_string()),
        siman_client_secret: Some("mock_secret".to_string()),
        siman_ba_key: Some("mock_ba_key".to_string()),
        tokens: HashMap::new(),
        output_dir: std::env::temp_dir().to_string_lossy().to_string(),
        db_config: None,
        siman_concurrency_limit: 20,
    };

    let mut client = MonsaktiClient::new(config)
        .await
        .expect("Failed to create client");

    // 4. Run Benchmark
    let start_time = Instant::now();
    let category = SimanAssetCategory::Tanah;

    let storage = StorageStrategy::JsonFile {
        base_dir: std::env::temp_dir()
            .join("siman_test")
            .to_string_lossy()
            .to_string(),
    };

    let result = fetch_all_assets_with_pagination(&mut client, &storage, category).await;

    let elapsed = start_time.elapsed();

    println!("Total execution time: {:?}", elapsed);

    assert!(result.is_ok());
    let (success, failed) = result.unwrap();
    println!("Success: {}, Failed: {}", success, failed);

    // 50 chunks * 100ms = 5000ms (Serial)
    // 50 chunks * 100ms / 5 = 1000ms (Current Parallel)
    // 50 chunks * 100ms / 20 = 250ms (Target Parallel)

    // Assert that it is at least faster than serial
    assert!(
        elapsed < Duration::from_secs(4),
        "Should be faster than serial execution (5s)"
    );
}
