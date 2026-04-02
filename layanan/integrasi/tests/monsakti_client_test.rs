use layanan_integrasi::client::MonsaktiClient;
use layanan_integrasi::config::Config;
use std::collections::HashMap;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_monsakti_fetch_parsing_logic() {
    // 1. Start mock server
    let mock_server = MockServer::start().await;

    // 2. Setup mock response for ADM/refAdmin
    // Format MonSAKTI: [[{"TOKEN":"new_token"}], [data1, data2]]
    let mock_json = serde_json::json!([
        [{"TOKEN": "token_baru_123"}],
        [
            {"kd_kl": "006", "nm_kl": "KEJAKSAAN REPUBLIK INDONESIA"},
            {"kd_kl": "007", "nm_kl": "OTHER"}
        ]
    ]);

    Mock::given(method("GET"))
        .and(path("/API/ADM/refAdmin/006"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_json))
        .mount(&mock_server)
        .await;

    // 3. Configure client
    let mut config = Config::default();
    config.base_url = mock_server.uri();
    config.tokens.insert("ADM".to_string(), "token_awal".to_string());
    config.db_config = None;

    let mut client = MonsaktiClient::new(config)
        .await
        .expect("Failed to create client");

    // 4. Execute fetch
    let response = client.fetch("ADM", "refAdmin", vec!["006".to_string()]).await;

    // 5. Assertions
    assert!(response.is_ok());
    let res = response.unwrap();

    // Check data
    assert!(res.data.is_some());
    let data = res.data.unwrap();
    assert!(data.is_array());
    assert_eq!(data.as_array().unwrap().len(), 2);
    assert_eq!(data[0]["kd_kl"], "006");

    // Check token update
    assert_eq!(res.new_token, Some("token_baru_123".to_string()));
}
