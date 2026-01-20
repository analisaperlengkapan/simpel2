use layanan_integrasi::client::MonsaktiClient;
use layanan_integrasi::config::Config;
use layanan_integrasi::siman::SimanAssetCategory;
use layanan_integrasi::siman::endpoints::fetch_all_assets_with_pagination;
use layanan_integrasi::StorageStrategy;
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};
use std::time::Instant;
use std::collections::HashMap;

#[tokio::test]
async fn test_siman_fetch_parallel_performance() {
    let mock_server = MockServer::start().await;

    // Mock Token Endpoint (SSO Kemenkeu)
    Mock::given(method("POST"))
        // Siman token URL usually ends with /connect/token or similar, config sets it.
        // We set it to /token in config below.
        .and(path_regex("/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "mock_siman_token",
            "expires_in": 3600,
            "token_type": "Bearer"
        })))
        .mount(&mock_server)
        .await;

    // Mock Row Count Endpoint
    // fetch_all_assets_with_pagination expects {"results": [{"RCOUNT": ...}]}
    Mock::given(method("GET"))
        .and(path_regex(r"^/gateway/SLDKSimanKL/2.0/getRowCount/TEST_BA/SIMAN2_M_ASET_.*"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": [{ "RCOUNT": 5000 }]
        })))
        .mount(&mock_server)
        .await;

    // Mock Data Endpoint
    let response_delay = std::time::Duration::from_millis(100);
    let items = vec![serde_json::json!({"id": 1}), serde_json::json!({"id": 2})];

    // fetch_all_assets_with_pagination handles {"results": [...]} or [...]
    // We'll return [...] to be simple, or maybe wrapped. Let's return wrapped to be consistent with row count.
    Mock::given(method("POST"))
        .and(path_regex(r"^/gateway/SLDKSimanKL/2.0/.*"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(serde_json::json!({
                "results": items
            }))
            .set_delay(response_delay)
        )
        .mount(&mock_server)
        .await;

    // Manual Config
    let config = Config {
        base_url: "http://localhost".to_string(), // Unused for SIMAN
        mysimkari_base_url: "http://localhost".to_string(),
        siman_base_url: mock_server.uri(),
        siman_token_url: format!("{}/token", mock_server.uri()),
        siman_client_id: Some("mock_client".to_string()),
        siman_client_secret: Some("mock_secret".to_string()),
        siman_ba_key: Some("TEST_BA".to_string()),
        tokens: HashMap::new(),
        output_dir: "./output_test".to_string(),
        db_config: None,
    };

    let mut client = MonsaktiClient::new(config).await.expect("Failed to create client");

    // Use JsonFile storage strategy to avoid database requirement
    // Use a temp dir
    let temp_dir = std::env::temp_dir().join("siman_perf_test");
    if temp_dir.exists() {
        std::fs::remove_dir_all(&temp_dir).unwrap_or_default();
    }
    std::fs::create_dir_all(&temp_dir).unwrap();

    let storage = StorageStrategy::JsonFile {
        base_dir: temp_dir.to_string_lossy().to_string(),
    };

    println!("Starting performance test for fetch_all_assets_with_pagination...");
    let start = Instant::now();

    // Total 5000 items. Chunk size 1000. => 5 requests.
    // Each request takes 100ms.
    // If fully serial (fetch, then save): 5 * (100ms fetch + X ms save).
    // If concurrent fetch (5 concurrent):
    //    We expect fetches to start immediately.
    //    Fetch 1-5 start. All take 100ms.
    //    After 100ms, all fetches ready (approx).
    //    Then we save 1, 2, 3, 4, 5.
    //    Total time ~ 100ms + 5 * SaveTime.
    //
    // If fetch is blocked by save loop:
    //    If buffer_unordered(5) is used but loop blocks on save.
    //    If save is fast (writing to JSON), it might be hard to see difference.
    //    But we verify correctness first.

    let result = fetch_all_assets_with_pagination(
        &mut client,
        &storage,
        SimanAssetCategory::AlatBesar
    ).await;

    let duration = start.elapsed();

    assert!(result.is_ok(), "Fetching failed: {:?}", result.err());
    let (success, failed) = result.unwrap();

    // 5 chunks * 2 items/chunk = 10 items saved.
    // But success count in fetch_all_assets_with_pagination returns number of records saved.
    // Each chunk has 2 records. So 10 records total.
    println!("Saved {} records. Failed {}. Duration: {:?}", success, failed, duration);

    // Cleanup
    std::fs::remove_dir_all(&temp_dir).unwrap_or_default();

    println!("PERFORMANCE_RESULT: {} ms", duration.as_millis());
}
