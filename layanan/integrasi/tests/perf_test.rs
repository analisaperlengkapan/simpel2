use layanan_perlengkapan_integrasi::client::MonsaktiClient;
use layanan_perlengkapan_integrasi::config::Config;
use layanan_perlengkapan_integrasi::siman::{SimanAssetCategory, fetch_all_aset_paginated};
use std::collections::HashMap;
use std::time::Instant;
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_siman_pagination_performance() {
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
    Mock::given(method("GET"))
        .and(path_regex(
            r"^/gateway/SLDKSimanKL/2.0/getRowCount/TEST_BA/SIMAN2_M_ASET_.*",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            { "row_count": 5000 }
        ])))
        .mount(&mock_server)
        .await;

    // Mock Data Endpoint
    let response_delay = std::time::Duration::from_millis(50);
    // Return a few items per request
    let items = vec![serde_json::json!({"id": 1}), serde_json::json!({"id": 2})];

    Mock::given(method("POST"))
        .and(path_regex(r"^/gateway/SLDKSimanKL/2.0/getAset.*"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!(items))
                .set_delay(response_delay),
        )
        .mount(&mock_server)
        .await;

    // Manual Config
    let config = Config {
        base: lib_common::config::BaseServiceConfig::default(),
        base_url: "http://localhost".to_string(),
        mysimkari_base_url: "http://localhost".to_string(),
        siman_base_url: mock_server.uri(),
        siman_token_url: format!("{}/connect/token", mock_server.uri()),
        siman_client_id: Some("mock_client".to_string()),
        siman_client_secret: Some("mock_secret".to_string()),
        siman_ba_key: Some("TEST_BA".to_string()),
        tokens: HashMap::new(),
        output_dir: "./output".to_string(),
        db_config: None,
        siman_concurrency_limit: 20,
        ..Default::default()
    };

    let mut client: MonsaktiClient = MonsaktiClient::new(config)
        .await
        .expect("Failed to create client");

    println!("Starting performance test...");
    let start = Instant::now();
    // chunk_size = 1000. Total = 5000. => 5 requests.
    // Each request takes 50ms.
    // Serial: 5 * 50ms = 250ms + overhead.
    // Concurrent: ~50ms + overhead.
    let result: Result<Vec<serde_json::Value>, _> =
        fetch_all_aset_paginated(&mut client, SimanAssetCategory::AlatBesar, 1000).await;
    let duration = start.elapsed();

    assert!(result.is_ok(), "Fetching failed: {:?}", result.err());
    let data = result.unwrap();
    println!("Fetched {} chunks (mocked) in {:?}", data.len(), duration);
    println!("PERFORMANCE_RESULT: {} ms", duration.as_millis());
}
