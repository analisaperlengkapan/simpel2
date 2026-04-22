//! Integration tests for Client Management API
//!
//! Tests verify that client management types and handler exports
//! are properly accessible. Full HTTP-level integration tests
//! require a running database and are tested separately.

use authenc_types::domain::OidcClient;
use chrono::Utc;

// ============================================================================
// Client Type Tests
// ============================================================================

#[test]
fn test_oidc_client_creation() {
    let now = Utc::now();
    let client = OidcClient {
        id: uuid::Uuid::new_v4().to_string(),
        client_id: "test-client".to_string(),
        client_secret: String::new(),
        redirect_uris: vec!["http://localhost:3000/callback".to_string()],
        name: "Test Client".to_string(),
        enabled: true,
        created_at: now,
        updated_at: now,
    };

    assert_eq!(client.client_id, "test-client");
    assert!(client.enabled);
    assert!(client.client_secret.is_empty());
}

#[test]
fn test_oidc_client_confidential() {
    let now = Utc::now();
    let client = OidcClient {
        id: uuid::Uuid::new_v4().to_string(),
        client_id: "confidential-client".to_string(),
        client_secret: "hashed_secret_value".to_string(),
        redirect_uris: vec!["https://app.example.com/callback".to_string()],
        name: "Confidential Client".to_string(),
        enabled: true,
        created_at: now,
        updated_at: now,
    };

    assert!(!client.client_secret.is_empty());
    assert_eq!(client.redirect_uris.len(), 1);
}

#[test]
fn test_oidc_client_multiple_redirect_uris() {
    let now = Utc::now();
    let client = OidcClient {
        id: uuid::Uuid::new_v4().to_string(),
        client_id: "multi-uri-client".to_string(),
        client_secret: String::new(),
        redirect_uris: vec![
            "http://localhost:3000/callback".to_string(),
            "http://localhost:3000/silent-callback".to_string(),
            "https://app.example.com/callback".to_string(),
        ],
        name: "Multi URI Client".to_string(),
        enabled: true,
        created_at: now,
        updated_at: now,
    };

    assert_eq!(client.redirect_uris.len(), 3);
}

#[test]
fn test_oidc_client_enabled_toggle() {
    let now = Utc::now();
    let client = OidcClient {
        id: uuid::Uuid::new_v4().to_string(),
        client_id: "toggle-client".to_string(),
        client_secret: String::new(),
        redirect_uris: vec![],
        name: "Toggle Client".to_string(),
        enabled: true,
        created_at: now,
        updated_at: now,
    };

    assert!(client.enabled);

    let disabled = OidcClient {
        enabled: false,
        ..client
    };

    assert!(!disabled.enabled);
}

#[test]
fn test_oidc_client_secret_rotation() {
    let now = Utc::now();
    let mut client = OidcClient {
        id: uuid::Uuid::new_v4().to_string(),
        client_id: "rotating-client".to_string(),
        client_secret: "old_secret".to_string(),
        redirect_uris: vec![],
        name: "Rotating Client".to_string(),
        enabled: true,
        created_at: now,
        updated_at: now,
    };

    let old_secret = client.client_secret.clone();
    client.client_secret = "new_secret".to_string();
    client.updated_at = Utc::now();

    assert_ne!(client.client_secret, old_secret);
    assert!(client.updated_at >= now);
}

// ============================================================================
// Handler Export Verification
// ============================================================================

#[test]
fn test_client_handlers_exist() {
    // Verify client management handler module is accessible
    #[allow(unused_imports)]
    use authenc_api::handlers::client;
}

#[test]
fn test_client_registration_handlers_exist() {
    // Verify DCR handler module is accessible
    #[allow(unused_imports)]
    use authenc_api::handlers::client_registration;
}
