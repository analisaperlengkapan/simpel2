use secreton_core::services::auth_service::AuthService;
use secreton_crypto::CryptoEngine;
use secreton_storage::memory::MemoryBackend;
use std::sync::Arc;

#[tokio::test]
async fn test_last_login_update() {
    let backend = Arc::new(MemoryBackend::new());
    let crypto = Arc::new(CryptoEngine::new());
    let auth_service = AuthService::new_mock(backend.clone(), crypto.clone());

    let username = "testuser";
    let password = "password123";

    // Create user
    let user = auth_service
        .create_user(
            username,
            "test@example.com",
            password,
            Some("Test User"),
            vec!["user".to_string()],
            None,
            true,
        )
        .await
        .expect("Failed to create user");

    assert!(user.last_login.is_none());

    // Login (authenticate)
    auth_service
        .authenticate(username, password, None, "127.0.0.1", "test-agent")
        .await
        .expect("Authentication failed");

    // Verify last_login updated
    let updated_user = auth_service
        .get_user(&user.id.to_string())
        .await
        .expect("Failed to get user");

    assert!(updated_user.last_login.is_some());
}
