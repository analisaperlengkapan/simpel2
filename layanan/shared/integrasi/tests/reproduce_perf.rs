use layanan_integrasi::client::MonsaktiClient;
use layanan_integrasi::config::Config;
use layanan_integrasi::siman::{endpoints::fetch_all_assets_with_pagination, SimanAssetCategory};
use layanan_integrasi::StorageStrategy;
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};
use std::time::Instant;
use std::collections::HashMap;

#[tokio::test]
async fn test_siman_assets_pagination_performance() {
    let mock_server = MockServer::start().await;

    // Mock Token Endpoint
    Mock::given(method("POST"))
        .and(path("/connect/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "mock_token",
            "token_type": "Bearer",
            "expires_in": 3600
        })))
        .mount(&mock_server)
        .await;

    // Mock Row Count Endpoint (Using SimanAssetCategory::AlatBesar which maps to SIMAN2_M_ASET_ALAT_BESAR)
    // We set a high count to force multiple chunks.
    // Target: 10,000 items.
    // Current chunk size 1000 => 10 requests.
    // Optimized chunk size 2000 => 5 requests.
    let total_rows = 10000;

    Mock::given(method("GET"))
        .and(path_regex(r"^/gateway/SLDKSimanKL/2.0/getRowCount/TEST_BA/SIMAN2_M_ASET_.*"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": [
                { "RCOUNT": total_rows }
            ]
        })))
        .mount(&mock_server)
        .await;

    // Mock Data Endpoint
    let response_delay = std::time::Duration::from_millis(50);
    // Return dummy items. The number of items returned should theoretically match chunk size
    // but for performance test we just return a small array to simulate "some data"
    // unless the code checks for exact count match which it doesn't seem to strict about (it warns if empty).
    // However, fetch_all_assets_with_pagination checks "if records.is_empty()".
    // We'll return 10 items per request to keep it light.
    let items: Vec<serde_json::Value> = (0..10).map(|i| serde_json::json!({"id": i})).collect();

    // The response structure expected by fetch_all_assets_with_pagination:
    // { "data": { "results": [...] } } OR { "data": [...] }
    let response_body = serde_json::json!({
        "results": items
    });

    Mock::given(method("POST"))
        .and(path_regex(r"^/gateway/SLDKSimanKL/2.0/getAset.*"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(response_body)
            .set_delay(response_delay)
        )
        .mount(&mock_server)
        .await;

    // Manual Config
    let config = Config {
        base_url: "http://localhost".to_string(),
        mysimkari_base_url: "http://localhost".to_string(),
        siman_base_url: mock_server.uri(),
        siman_token_url: format!("{}/connect/token", mock_server.uri()),
        siman_client_id: Some("mock_client".to_string()),
        siman_client_secret: Some("mock_secret".to_string()),
        siman_ba_key: Some("TEST_BA".to_string()),
        tokens: HashMap::new(),
        output_dir: "./output_test".to_string(), // Use a test output dir
        db_config: None,
        siman_concurrency_limit: 20,
    };

    let mut client = MonsaktiClient::new(config).await.expect("Failed to create client");

    // Use JsonFile strategy to avoid DB.
    let storage = StorageStrategy::JsonFile { base_dir: "./output_test".to_string() };

    // Clean up output dir if exists
    let _ = tokio::fs::remove_dir_all("./output_test").await;

    println!("Starting performance test for fetch_all_assets_with_pagination...");
    let start = Instant::now();

    let result = fetch_all_assets_with_pagination(&mut client, &storage, SimanAssetCategory::AlatBesar).await;
    let duration = start.elapsed();

    // Clean up
    let _ = tokio::fs::remove_dir_all("./output_test").await;

    assert!(result.is_ok(), "Fetching failed: {:?}", result.err());
    let (success, failed) = result.unwrap();
    println!("Fetched: {} success, {} failed in {:?}", success, failed, duration);
    println!("PERFORMANCE_RESULT: {} ms", duration.as_millis());
}
