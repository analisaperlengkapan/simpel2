use layanan_integrasi::client::MonsaktiClient;
use layanan_integrasi::config::Config;
use serde_json::json;
use std::collections::HashMap;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_monsakti_fetch_parsing_logic() {
    // 1. Setup WireMock
    let mock_server = MockServer::start().await;

    // Response that mimics MonSAKTI array format: [[{TOKEN:...}], [data...]]
    let response_body = json!([
        [{"TOKEN": "new_token_123"}],
        [{"id": 1, "name": "Item 1"}, {"id": 2, "name": "Item 2"}]
    ]);

    Mock::given(method("GET"))
        .and(path("/API/ADM/data/var1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    // 2. Configure Client
    let mut config = Config::from_env().unwrap_or_else(|_| Config {
        base: lib_common::config::BaseServiceConfig::default(),
        base_url: mock_server.uri(), // Point to mock server
        mysimkari_base_url: "".to_string(),
        siman_base_url: "".to_string(),
        siman_token_url: "".to_string(),
        siman_client_id: None,
        siman_client_secret: None,
        siman_ba_key: None,
        tokens: HashMap::new(),
        output_dir: ".".to_string(),
        db_config: None,
        siman_concurrency_limit: 20,
    });
    // Override base_url to ensure it hits mock
    config.base_url = mock_server.uri();
    config
        .tokens
        .insert("ADM".to_string(), "initial_token".to_string());

    let mut client = MonsaktiClient::new(config)
        .await
        .expect("Failed to create client");

    // 3. Perform Fetch
    let res = client.fetch("ADM", "data", vec!["var1".to_string()]).await;

    // 4. Assertions
    assert!(res.is_ok(), "Fetch failed");
    let resp = res.unwrap();

    // Verify parsing logic works for the specific array format
    assert!(resp.new_token.is_some(), "Expected new_token to be present");
    assert_eq!(resp.new_token.as_deref(), Some("new_token_123"));

    let data = resp.data.expect("Expected data");
    let arr = data.as_array().expect("Expected data to be array");
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["name"], "Item 1");
}
