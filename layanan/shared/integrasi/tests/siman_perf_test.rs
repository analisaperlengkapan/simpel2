use layanan_integrasi::StorageStrategy;
use layanan_integrasi::client::MonsaktiClient;
use layanan_integrasi::config::Config;
use layanan_integrasi::siman::SimanAssetCategory;
use layanan_integrasi::siman::endpoints::fetch_all_assets_with_pagination;
use std::collections::HashMap;
use std::fs;
use std::time::Instant;
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_siman_pagination_pipeline_perf() {
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

    // Mock Row Count Endpoint
    // Return 5000 items. Chunk size is 1000, so 5 batches.
    Mock::given(method("GET"))
        .and(path_regex(
            r"^/gateway/SLDKSimanKL/2.0/getRowCount/TEST_BA/SIMAN2_M_ASET_.*",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
             "results": [
                { "RCOUNT": 5000 }
             ]
        })))
        .mount(&mock_server)
        .await;

    // Mock Data Endpoint
    let response_delay = std::time::Duration::from_millis(100);
    // Return a few items per request
    let items = vec![serde_json::json!({"id": 1}), serde_json::json!({"id": 2})];
    let response_body = serde_json::json!({
        "results": items
    });

    Mock::given(method("POST"))
        .and(path_regex(r"^/gateway/SLDKSimanKL/2.0/getAset.*"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(response_body)
                .set_delay(response_delay),
        )
        .mount(&mock_server)
        .await;

    // Prepare temp output dir
    let output_dir = format!("/tmp/siman_perf_test_{}", uuid::Uuid::new_v4());
    fs::create_dir_all(&output_dir).unwrap();

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
        output_dir: output_dir.clone(),
        db_config: None,
    };

    let mut client = MonsaktiClient::new(config)
        .await
        .expect("Failed to create client");

    // Use JsonFile storage strategy
    let storage = StorageStrategy::JsonFile {
        base_dir: output_dir.clone(),
    };

    println!("Starting performance test (pipeline)...");
    let start = Instant::now();

    // This will fetch 5000 records in chunks of 1000.
    // 5 requests. Each 100ms.
    // Concurrency 5.
    // If save is fast: time should be around 100ms + overhead.
    // If save is slow (implicit in file IO, but might be fast on tmpfs): we will see.
    let result =
        fetch_all_assets_with_pagination(&mut client, &storage, SimanAssetCategory::AlatBesar)
            .await;
    let duration = start.elapsed();

    assert!(result.is_ok(), "Fetching failed: {:?}", result.err());
    let (success, failed) = result.unwrap();
    println!(
        "Fetched: {} success, {} failed in {:?}",
        success, failed, duration
    );
    println!("PERFORMANCE_RESULT: {} ms", duration.as_millis());

    // Cleanup
    let _ = fs::remove_dir_all(&output_dir);
}
