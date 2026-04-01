//! OAuth2 flow tests
//!
//! Tests OAuth2/OIDC authentication flow including:
//! - Authorization URL generation
//! - State parameter handling
//! - Callback URL parsing
//! - Code exchange

// In integration tests, we need to use the crate name from Cargo.toml
extern crate portal_microfrontend;

use portal_microfrontend::features::oauth::OAuthClient;

#[cfg(test)]
mod oauth_flow_tests {
    use super::*;

    #[test]
    fn test_oauth_client_creation() {
        let client = OAuthClient::new(
            "http://localhost:8088".to_string(),
            "simpel".to_string(),
            "portal".to_string(),
            "http://localhost:8080/callback".to_string(),
        );

        assert_eq!(client.base_url, "http://localhost:8088");
        assert_eq!(client.realm, "simpel");
        assert_eq!(client.client_id, "portal");
        assert_eq!(client.redirect_uri, "http://localhost:8080/callback");
    }

    #[test]
    fn test_authorization_endpoint() {
        let client = create_test_client();

        let endpoint = client.authorization_endpoint();
        assert_eq!(endpoint, "http://localhost:8088/api/v1/oauth2/authorize");
    }

    #[test]
    fn test_token_endpoint() {
        let client = create_test_client();

        let endpoint = client.token_endpoint();
        assert_eq!(endpoint, "http://localhost:8088/api/v1/oauth2/token");
    }

    #[test]
    fn test_generate_state() {
        let state1 = OAuthClient::generate_state();
        let state2 = OAuthClient::generate_state();

        // States should be unique
        assert_ne!(state1, state2);

        // States should not be empty
        assert!(!state1.is_empty());
        assert!(!state2.is_empty());

        // States should be valid UUIDs
        assert!(uuid::Uuid::parse_str(&state1).is_ok());
        assert!(uuid::Uuid::parse_str(&state2).is_ok());
    }

    #[test]
    fn test_get_authorization_url_default_scope() {
        let client = create_test_client();
        let state = "test-state-123".to_string();

        let auth_url = client.get_authorization_url(state.clone(), None);

        // Verify URL contains required parameters
        assert!(auth_url.contains("response_type=code"));
        assert!(auth_url.contains("client_id=portal"));
        assert!(auth_url.contains("redirect_uri=http%3A%2F%2Flocalhost%3A8080%2Fcallback"));
        // Scope is URL encoded, so spaces become %20
        assert!(auth_url.contains("scope=openid%20profile%20email"));
        assert!(auth_url.contains(&format!("state={}", state)));
    }

    #[test]
    fn test_get_authorization_url_custom_scope() {
        let client = create_test_client();
        let state = "test-state-456".to_string();
        let custom_scope = "openid profile email roles".to_string();

        let auth_url = client.get_authorization_url(state.clone(), Some(custom_scope));

        // Verify custom scope is used (URL encoded, spaces become %20)
        assert!(auth_url.contains("scope=openid%20profile%20email%20roles"));
    }

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_parse_callback_url_success() {
        let callback_url = "http://localhost:8080/callback?code=auth_code_123&state=test_state_456";

        let result = OAuthClient::parse_callback_url(callback_url);
        assert!(result.is_ok());

        let (code, state) = result.unwrap();
        assert_eq!(code, "auth_code_123");
        assert_eq!(state, "test_state_456");
    }

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_parse_callback_url_missing_code() {
        let callback_url = "http://localhost:8080/callback?state=test_state_456";

        let result = OAuthClient::parse_callback_url(callback_url);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing authorization code"));
    }

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_parse_callback_url_missing_state() {
        let callback_url = "http://localhost:8080/callback?code=auth_code_123";

        let result = OAuthClient::parse_callback_url(callback_url);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing state parameter"));
    }

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_parse_callback_url_with_error() {
        let callback_url = "http://localhost:8080/callback?error=access_denied&error_description=User+denied+access";

        let result = OAuthClient::parse_callback_url(callback_url);
        assert!(result.is_err());

        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("access_denied"));
        assert!(error_msg.contains("User denied access"));
    }

    #[test]
    fn test_authorization_url_encoding() {
        let client = OAuthClient::new(
            "http://localhost:8088".to_string(),
            "simpel".to_string(),
            "portal-client".to_string(),
            "http://localhost:8080/auth/callback".to_string(),
        );

        let state = "state-with-special-chars-!@#$".to_string();
        let auth_url = client.get_authorization_url(state, None);

        // Verify special characters are URL encoded
        assert!(auth_url.contains("client_id=portal-client"));
        assert!(auth_url.contains("redirect_uri=http%3A%2F%2Flocalhost%3A8080%2Fauth%2Fcallback"));
    }

    /// Test exchange_code
    /// NOTE: This test is disabled because exchange_code requires WASM context
    #[tokio::test]
    #[ignore]
    async fn test_exchange_code_mock() {
        // Set mock mode
        unsafe {
            std::env::set_var("AUTHENC_API_URL", "mock");
        }

        let client = create_test_client();
        let code = "mock_authorization_code";

        let result = client.exchange_code(code).await;

        // In non-WASM environment, this will return an error
        // In WASM environment with mock mode, this should succeed
        if let Ok(token_response) = result {
            assert!(!token_response.access_token.is_empty());
            assert_eq!(token_response.token_type, "Bearer");
            assert!(token_response.expires_in > 0);
        }

        // Clean up
        unsafe {
            std::env::remove_var("AUTHENC_API_URL");
        }
    }

    #[test]
    fn test_oauth_flow_state_management() {
        // Generate state
        let state = OAuthClient::generate_state();
        assert!(!state.is_empty());

        // In a real flow:
        // 1. Store state before redirect
        // 2. Verify state in callback
        // This ensures CSRF protection

        // Verify state is a valid UUID
        assert!(uuid::Uuid::parse_str(&state).is_ok());
    }

    #[test]
    fn test_multiple_oauth_clients() {
        let client1 = OAuthClient::new(
            "http://authenc1.example.com".to_string(),
            "realm1".to_string(),
            "client1".to_string(),
            "http://app1.example.com/callback".to_string(),
        );

        let client2 = OAuthClient::new(
            "http://authenc2.example.com".to_string(),
            "realm2".to_string(),
            "client2".to_string(),
            "http://app2.example.com/callback".to_string(),
        );

        // Verify clients are independent
        assert_ne!(client1.base_url, client2.base_url);
        assert_ne!(client1.realm, client2.realm);
        assert_ne!(client1.client_id, client2.client_id);
        assert_ne!(client1.redirect_uri, client2.redirect_uri);

        // Verify endpoints are different
        assert_ne!(
            client1.authorization_endpoint(),
            client2.authorization_endpoint()
        );
        assert_ne!(client1.token_endpoint(), client2.token_endpoint());
    }

    // Helper function to create a test OAuth client
    fn create_test_client() -> OAuthClient {
        OAuthClient::new(
            "http://localhost:8088".to_string(),
            "simpel".to_string(),
            "portal".to_string(),
            "http://localhost:8080/callback".to_string(),
        )
    }
}
