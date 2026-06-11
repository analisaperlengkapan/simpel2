use secreton_core::config::api::AuthConfig;
use secreton_core::services::auth_service::{AuthError, AuthService, CreateUserRequest};
use secreton_core::storage::InMemoryStorage;
use secreton_crypto::CryptoEngine;
use secreton_storage::memory::MemoryBackend;
use std::sync::Arc;

/// Helper to setup auth service for tests
async fn setup_auth_service() -> AuthService {
    let storage = Arc::new(InMemoryStorage::new());
    let crypto = Arc::new(CryptoEngine::new());
    let config = AuthConfig::default();
    AuthService::new(storage, crypto, &config).await.unwrap()
}

#[tokio::test]
async fn test_last_login_update() {
    let backend = Arc::new(MemoryBackend::new());
    let crypto = Arc::new(CryptoEngine::new());
    let auth_service = AuthService::new_mock(backend.clone(), crypto.clone());

    let username = "testuser";
    let password = "password123";

    // Create user
    let user = auth_service
        .create_user(CreateUserRequest {
            username,
            email: "test@example.com",
            password,
            full_name: Some("Test User"),
            roles: vec!["user".to_string()],
            policies: vec![],
            metadata: None,
            is_active: true,
        })
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

#[tokio::test]
async fn test_refresh_token() {
    let auth_service = setup_auth_service().await;

    // 1. Create a user
    let user = auth_service
        .create_user(CreateUserRequest {
            username: "testuser",
            email: "test@example.com",
            password: "password123",
            full_name: Some("Test User"),
            roles: vec!["user".to_string()],
            policies: vec![],
            metadata: None,
            is_active: true,
        })
        .await
        .unwrap();

    // 2. Authenticate to get an initial token
    let auth_token = auth_service
        .authenticate("testuser", "password123", None, "127.0.0.1", "test-agent")
        .await
        .unwrap();

    // Ensure time passes so token timestamps are different
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    // 3. Use the refresh token to get a new access token
    let refreshed_token = auth_service
        .refresh_token(&auth_token.refresh_token)
        .await
        .unwrap();

    // 4. Verify the new token
    assert_ne!(auth_token.access_token, refreshed_token.access_token);
    assert_eq!(auth_token.refresh_token, refreshed_token.refresh_token);
    assert_eq!(refreshed_token.user.id, user.id);

    // 5. Validate the new access token
    let validated_user = auth_service
        .validate_token(&refreshed_token.access_token)
        .await
        .unwrap();
    assert_eq!(validated_user.id, user.id);
}

#[tokio::test]
async fn test_refresh_token_with_invalid_token() {
    let auth_service = setup_auth_service().await;
    let result = auth_service.refresh_token("invalid_token").await;
    assert!(matches!(result, Err(AuthError::InvalidToken)));
}

#[tokio::test]
async fn test_refresh_token_with_access_token() {
    let auth_service = setup_auth_service().await;

    auth_service
        .create_user(CreateUserRequest {
            username: "testuser2",
            email: "test2@example.com",
            password: "password123",
            full_name: None,
            roles: vec!["user".to_string()],
            policies: vec![],
            metadata: None,
            is_active: true,
        })
        .await
        .unwrap();
    let auth_token = auth_service
        .authenticate("testuser2", "password123", None, "127.0.0.1", "test-agent")
        .await
        .unwrap();

    // Attempt to refresh with the access token (should fail)
    let result = auth_service.refresh_token(&auth_token.access_token).await;
    assert!(matches!(result, Err(AuthError::InvalidToken)));
}
