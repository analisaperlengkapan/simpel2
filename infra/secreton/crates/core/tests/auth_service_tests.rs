
#[cfg(test)]
mod auth_service_tests {
    use secreton_core::config::api::AuthConfig;
    use secreton_core::services::auth_service::{AuthError, AuthService};
    use secreton_crypto::CryptoEngine;
    use secreton_storage::InMemoryStorage;
    use std::sync::Arc;

    async fn setup_auth_service() -> AuthService {
        let storage = Arc::new(InMemoryStorage::new());
        let crypto = Arc::new(CryptoEngine::new().unwrap());
        let config = AuthConfig::default();
        AuthService::new(storage, crypto, &config).await.unwrap()
    }

    #[tokio::test]
    async fn test_refresh_token() {
        let auth_service = setup_auth_service().await;

        // 1. Create a user
        let user = auth_service
            .create_user(
                "testuser",
                "test@example.com",
                "password123",
                Some("Test User"),
                vec!["user".to_string()],
                None,
                true,
            )
            .await
            .unwrap();

        // 2. Authenticate to get an initial token
        let auth_token = auth_service
            .authenticate(
                "testuser",
                "password123",
                None,
                "127.0.0.1",
                "test-agent",
            )
            .await
            .unwrap();

        // 3. Use the refresh token to get a new access token
        let refreshed_token = auth_service
            .refresh_token(&auth_token.refresh_token)
            .await
            .unwrap();

        // 4. Verify the new token
        assert_ne!(
            auth_token.access_token,
            refreshed_token.access_token
        );
        assert_eq!(
            auth_token.refresh_token,
            refreshed_token.refresh_token
        );
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

        // 1. Create a user and authenticate
        auth_service
            .create_user(
                "testuser",
                "test@example.com",
                "password123",
                None,
                vec!["user".to_string()],
                None,
                true,
            )
            .await
            .unwrap();
        let auth_token = auth_service
            .authenticate(
                "testuser",
                "password123",
                None,
                "127.0.0.1",
                "test-agent",
            )
            .await
            .unwrap();

        // 2. Attempt to refresh with the access token
        let result = auth_service
            .refresh_token(&auth_token.access_token)
            .await;

        assert!(matches!(result, Err(AuthError::InvalidToken)));
    }
}
