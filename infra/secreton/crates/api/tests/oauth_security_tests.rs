use axum_test::TestServer;
use secreton_api::{
    config::{ApiConfig, OAuth2Config, OAuth2Provider},
    handlers::auth::create_routes,
    services::ServiceContainer,
    ApiResponse,
};
use secreton_storage::{MemoryBackend, SecretEntry, SecurityLevel, StorageBackend};
use secreton_crypto::CryptoEngine;
use std::sync::Arc;

fn create_dummy_pool() -> deadpool_postgres::Pool {
    let mut cfg = deadpool_postgres::Config::new();
    cfg.dbname = Some("test".to_string());
    cfg.manager = Some(deadpool_postgres::ManagerConfig {
        recycling_method: deadpool_postgres::RecyclingMethod::Verified,
    });
    cfg.create_pool(Some(deadpool_postgres::Runtime::Tokio1), tokio_postgres::NoTls).unwrap()
}

async fn create_test_server(config: ApiConfig, storage: Arc<dyn StorageBackend + Send + Sync>, crypto: Arc<CryptoEngine>) -> TestServer {
    let pool = create_dummy_pool();
    let mut services = ServiceContainer::new_mock(storage, pool);
    services.config = config;
    services.crypto = crypto;

    let services = Arc::new(services);

    let app = create_routes()
        .with_state(services);

    TestServer::new(app).expect("Failed to create test server")
}

#[tokio::test]
async fn test_bug_1_oauth_state_expiration() {
    let mut config = ApiConfig::default();
    config.auth.oauth2 = Some(OAuth2Config {
        providers: vec![OAuth2Provider {
            name: "github".to_string(),
            client_id: "test_client_id".to_string(),
            client_secret: "test_client_secret".to_string(),
            auth_url: "https://github.com/login/oauth/authorize".to_string(),
            token_url: "https://github.com/login/oauth/access_token".to_string(),
            user_info_url: "https://api.github.com/user".to_string(),
        }],
        redirect_url: "http://localhost:8200/auth/callback".to_string(),
        scopes: vec!["read:user".to_string()],
    });

    let storage = Arc::new(MemoryBackend::new());
    let crypto = Arc::new(CryptoEngine::new());
    let server = create_test_server(config, storage.clone(), crypto.clone()).await;

    // 1. Manually insert an expired state
    let state_token = "expired_state";
    let storage_path = format!("sys/oauth/states/{}", state_token);
    let now = chrono::Utc::now();
    let expired_time = now - chrono::Duration::minutes(20);

    let data = serde_json::json!({
        "provider": "github",
    });

    // Encrypt the data so it passes decryption check
    let state_bytes = serde_json::to_vec(&data).unwrap();
    let encrypted_data = crypto.encrypt_simple(&state_bytes).unwrap();

    let entry = SecretEntry::new(
        storage_path.clone(),
        encrypted_data,
        serde_json::json!({"method": "simple"}),
        SecurityLevel::Internal,
        "system".to_string(),
    )
    .with_expiration(expired_time);

    storage.store(&entry).await.unwrap();

    // 2. Call callback with expired state
    let response = server
        .get("/oauth/github/callback")
        .add_query_param("state", state_token)
        .add_query_param("code", "some_code")
        .await;

    let body: ApiResponse<serde_json::Value> = response.json();

    assert!(!body.success);
    let error_msg = body.error.unwrap().message;
    println!("Error message: {}", error_msg);

    // Now we expect "Invalid or expired state parameter"
    // The error message might be prefixed with "Authentication failed: " depending on how ApiError handles it.
    assert!(error_msg.contains("Invalid or expired state parameter"), "Should return expiration error. Got: {}", error_msg);
}

#[tokio::test]
async fn test_bug_2_oauth_state_encryption() {
    let mut config = ApiConfig::default();
    config.auth.oauth2 = Some(OAuth2Config {
        providers: vec![OAuth2Provider {
            name: "github".to_string(),
            client_id: "test_client_id".to_string(),
            client_secret: "test_client_secret".to_string(),
            auth_url: "https://github.com/login/oauth/authorize".to_string(),
            token_url: "https://github.com/login/oauth/access_token".to_string(),
            user_info_url: "https://api.github.com/user".to_string(),
        }],
        redirect_url: "http://localhost:8200/auth/callback".to_string(),
        scopes: vec!["read:user".to_string()],
    });

    let storage = Arc::new(MemoryBackend::new());
    let crypto = Arc::new(CryptoEngine::new());
    let server = create_test_server(config, storage.clone(), crypto.clone()).await;

    // 1. Call oauth login to generate state
    let response = server.get("/oauth/github").await;
    response.assert_status_ok();

    let body: ApiResponse<serde_json::Value> = response.json();
    let state_token = body.data.unwrap()["state"].as_str().unwrap().to_string();

    // 2. Check storage content
    let storage_path = format!("sys/oauth/states/{}", state_token);
    let entry = storage.get_by_path(&storage_path).await.unwrap().expect("State should be stored");

    // Bug 2 Check: Data should NOT be plain JSON
    let data = entry.encrypted_data;

    // Try to parse as JSON directly
    let json_result: Result<serde_json::Value, _> = serde_json::from_slice(&data);

    if let Ok(json) = json_result {
         if json.get("provider") == Some(&serde_json::Value::String("github".to_string())) {
             panic!("Data is still stored as plain JSON!");
         }
    } else {
        println!("Data is not plain JSON (good).");
    }

    // Verify we can decrypt it
    let decrypted = crypto.decrypt_simple(&data).expect("Should be able to decrypt");
    let json: serde_json::Value = serde_json::from_slice(&decrypted).expect("Decrypted data should be JSON");
    assert_eq!(json["provider"], "github");
}
