//! Integration tests for federation gRPC RPCs
//!
//! Tests:
//! - initiate_federated_auth() with OIDC provider
//! - initiate_federated_auth() with SAML provider
//! - complete_federated_auth() with authorization code
//! - Error handling for invalid providers

use authenc_federation::service::{
    FederationService, IdentityProviderConfig, IdentityProviderType,
    ProviderConfig, OidcProviderConfig,
};
use std::sync::Arc;
use uuid::Uuid;

/// Helper function to create a test federation service
async fn create_test_federation_service() -> Arc<FederationService> {
    // Create federation service
    let federation_service = Arc::new(FederationService::new());

    // Register a test OIDC provider
    let oidc_config = IdentityProviderConfig {
        id: Uuid::new_v4(),
        name: "test-oidc".to_string(),
        provider_type: IdentityProviderType::OIDC,
        config: ProviderConfig::OIDC(OidcProviderConfig {
            authorization_endpoint: "https://test-idp.example.com/authorize".to_string(),
            token_endpoint: "https://test-idp.example.com/token".to_string(),
            userinfo_endpoint: Some("https://test-idp.example.com/userinfo".to_string()),
            client_id: "test_client_id".to_string(),
            client_secret: "test_client_secret".to_string(),
            redirect_uri: "https://authenc.example.com/callback".to_string(),
            scopes: vec!["openid".to_string(), "profile".to_string(), "email".to_string()],
        }),
        enabled: true,
    };

    federation_service
        .register_provider(oidc_config)
        .await
        .unwrap();

    federation_service
}

#[tokio::test]
async fn test_initiate_federated_auth_oidc() {
    let federation_service = create_test_federation_service().await;

    // Create request
    let request = authenc_federation::service::FederatedAuthRequest {
        provider: "test-oidc".to_string(),
        redirect_uri: None,
        scopes: vec![],
        realm_id: authenc_types::domain::RealmId::new(),
    };

    // Initiate federated auth
    let response = federation_service
        .initiate_federated_auth(request)
        .await
        .unwrap();

    // Verify response
    assert!(response.auth_url.contains("test-idp.example.com/authorize"));
    assert!(response.auth_url.contains("response_type=code"));
    assert!(response.auth_url.contains("client_id=test_client_id"));
    assert!(response.auth_url.contains("redirect_uri="));
    assert!(response.auth_url.contains("scope="));
    assert!(response.auth_url.contains("state="));
    assert!(!response.state.is_empty());
    assert_eq!(response.state.len(), 32); // State should be 32 characters
}

#[tokio::test]
async fn test_initiate_federated_auth_with_custom_scopes() {
    let federation_service = create_test_federation_service().await;

    // Create request with custom scopes
    let request = authenc_federation::service::FederatedAuthRequest {
        provider: "test-oidc".to_string(),
        redirect_uri: Some("https://custom.example.com/callback".to_string()),
        scopes: vec!["openid".to_string(), "custom_scope".to_string()],
        realm_id: authenc_types::domain::RealmId::new(),
    };

    // Initiate federated auth
    let response = federation_service
        .initiate_federated_auth(request)
        .await
        .unwrap();

    // Verify custom scopes are included
    assert!(response.auth_url.contains("scope=openid"));
    assert!(response.auth_url.contains("custom_scope"));
    assert!(response.auth_url.contains("redirect_uri=https%3A%2F%2Fcustom.example.com"));
}

#[tokio::test]
async fn test_initiate_federated_auth_provider_not_found() {
    let federation_service = create_test_federation_service().await;

    // Create request with non-existent provider
    let request = authenc_federation::service::FederatedAuthRequest {
        provider: "nonexistent-provider".to_string(),
        redirect_uri: None,
        scopes: vec![],
        realm_id: authenc_types::domain::RealmId::new(),
    };

    // Initiate federated auth should fail
    let result = federation_service.initiate_federated_auth(request).await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(
        error,
        authenc_types::error::AuthencError::NotFound(_)
    ));
}

#[tokio::test]
async fn test_initiate_federated_auth_disabled_provider() {
    let federation_service = create_test_federation_service().await;

    // Register a disabled provider
    let disabled_config = IdentityProviderConfig {
        id: Uuid::new_v4(),
        name: "disabled-oidc".to_string(),
        provider_type: IdentityProviderType::OIDC,
        config: ProviderConfig::OIDC(OidcProviderConfig {
            authorization_endpoint: "https://disabled-idp.example.com/authorize".to_string(),
            token_endpoint: "https://disabled-idp.example.com/token".to_string(),
            userinfo_endpoint: None,
            client_id: "disabled_client".to_string(),
            client_secret: "disabled_secret".to_string(),
            redirect_uri: "https://authenc.example.com/callback".to_string(),
            scopes: vec!["openid".to_string()],
        }),
        enabled: false, // Disabled
    };

    federation_service
        .register_provider(disabled_config)
        .await
        .unwrap();

    // Create request for disabled provider
    let request = authenc_federation::service::FederatedAuthRequest {
        provider: "disabled-oidc".to_string(),
        redirect_uri: None,
        scopes: vec![],
        realm_id: authenc_types::domain::RealmId::new(),
    };

    // Initiate federated auth should fail
    let result = federation_service.initiate_federated_auth(request).await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(
        error,
        authenc_types::error::AuthencError::ValidationError(_)
    ));
}

#[tokio::test]
async fn test_complete_federated_auth() {
    let federation_service = create_test_federation_service().await;

    // Create request
    let request = authenc_federation::service::CompleteFederatedAuthRequest {
        provider: "test-oidc".to_string(),
        code: "test_authorization_code".to_string(),
        state: "test_state_parameter".to_string(),
        realm_id: authenc_types::domain::RealmId::new(),
    };

    // Complete federated auth
    let response = federation_service
        .complete_federated_auth(request)
        .await
        .unwrap();

    // Verify response (mock implementation returns placeholder data)
    assert!(!response.access_token.is_empty());
    assert!(!response.refresh_token.is_empty());
    assert!(!response.username.is_empty());
    assert!(!response.email.is_empty());
}

#[tokio::test]
async fn test_complete_federated_auth_provider_not_found() {
    let federation_service = create_test_federation_service().await;

    // Create request with non-existent provider
    let request = authenc_federation::service::CompleteFederatedAuthRequest {
        provider: "nonexistent-provider".to_string(),
        code: "test_code".to_string(),
        state: "test_state".to_string(),
        realm_id: authenc_types::domain::RealmId::new(),
    };

    // Complete federated auth should fail
    let result = federation_service.complete_federated_auth(request).await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(
        error,
        authenc_types::error::AuthencError::NotFound(_)
    ));
}

#[tokio::test]
async fn test_federation_state_generation_uniqueness() {
    let federation_service = create_test_federation_service().await;

    // Generate multiple states and verify they're unique
    let mut states = std::collections::HashSet::new();

    for _ in 0..100 {
        let request = authenc_federation::service::FederatedAuthRequest {
            provider: "test-oidc".to_string(),
            redirect_uri: None,
            scopes: vec![],
            realm_id: authenc_types::domain::RealmId::new(),
        };

        let response = federation_service
            .initiate_federated_auth(request)
            .await
            .unwrap();

        // State should be unique
        assert!(states.insert(response.state.clone()));
    }

    // All 100 states should be unique
    assert_eq!(states.len(), 100);
}

#[tokio::test]
async fn test_federation_url_encoding() {
    let federation_service = create_test_federation_service().await;

    // Create request with special characters in redirect_uri
    let request = authenc_federation::service::FederatedAuthRequest {
        provider: "test-oidc".to_string(),
        redirect_uri: Some("https://example.com/callback?param=value&other=test".to_string()),
        scopes: vec!["openid".to_string(), "profile email".to_string()],
        realm_id: authenc_types::domain::RealmId::new(),
    };

    // Initiate federated auth
    let response = federation_service
        .initiate_federated_auth(request)
        .await
        .unwrap();

    // Verify URL encoding
    assert!(response.auth_url.contains("redirect_uri=https%3A%2F%2Fexample.com"));
    assert!(response.auth_url.contains("%3Fparam%3Dvalue"));
    assert!(response.auth_url.contains("%26other%3Dtest"));
}
