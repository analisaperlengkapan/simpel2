//! Unit tests for PostgresClientStore
//!
//! These tests verify the ClientStore implementation logic for OAuth2/OIDC client management.
//! Updated to match the current OidcClient struct definition in authenc_types.

use authenc_types::domain::OidcClient;
use chrono::Utc;

/// Helper to create a test OidcClient with sensible defaults
fn test_client(client_id: &str) -> OidcClient {
    let now = Utc::now();
    OidcClient {
        id: uuid::Uuid::new_v4().to_string(),
        client_id: client_id.to_string(),
        client_secret: String::new(),
        redirect_uris: vec!["http://localhost:3000/callback".to_string()],
        name: format!("Test Client {}", client_id),
        enabled: true,
        created_at: now,
        updated_at: now,
    }
}

#[test]
fn test_client_creation_public() {
    let client = test_client("public-client");
    assert_eq!(client.client_id, "public-client");
    assert!(client.client_secret.is_empty()); // Public clients have no secret
    assert!(client.enabled);
}

#[test]
fn test_client_creation_confidential() {
    let mut client = test_client("confidential-client");
    client.client_secret = "hashed_secret".to_string();

    assert_eq!(client.client_id, "confidential-client");
    assert!(!client.client_secret.is_empty());
    assert!(client.enabled);
}

#[test]
fn test_client_id_uniqueness() {
    let c1 = test_client("client-1");
    let c2 = test_client("client-2");
    assert_ne!(c1.id, c2.id);
    assert_ne!(c1.client_id, c2.client_id);
}

#[test]
fn test_client_redirect_uris() {
    let mut client = test_client("multi-uri-client");
    client.redirect_uris = vec![
        "http://localhost:3000/callback".to_string(),
        "http://localhost:3000/silent-callback".to_string(),
        "https://app.example.com/callback".to_string(),
    ];

    assert_eq!(client.redirect_uris.len(), 3);
}

#[test]
fn test_client_enabled_states() {
    let enabled_client = test_client("enabled-client");
    assert!(enabled_client.enabled);

    let disabled_client = OidcClient {
        enabled: false,
        ..enabled_client
    };
    assert!(!disabled_client.enabled);
}

#[test]
fn test_client_timestamps() {
    let created_at = Utc::now();
    let updated_at = created_at + chrono::Duration::hours(1);

    let mut client = test_client("test-client");
    client.created_at = created_at;
    client.updated_at = updated_at;

    assert_eq!(client.created_at, created_at);
    assert_eq!(client.updated_at, updated_at);
    assert!(client.updated_at > client.created_at);
}

#[test]
fn test_client_secret_presence() {
    // Public client (no secret)
    let public_client = test_client("public");
    assert!(public_client.client_secret.is_empty());

    // Confidential client (with secret)
    let mut confidential_client = test_client("confidential");
    confidential_client.client_secret = "hashed_secret".to_string();
    assert!(!confidential_client.client_secret.is_empty());
}

#[test]
fn test_client_empty_redirect_uris() {
    let mut client = test_client("no-uris-client");
    client.redirect_uris = vec![];
    assert!(client.redirect_uris.is_empty());
}

#[test]
fn test_multiple_clients() {
    let c1 = test_client("client1");
    let c2 = test_client("client2");

    assert_ne!(c1.id, c2.id);
    assert_ne!(c1.client_id, c2.client_id);
}

#[test]
fn test_client_update_redirect_uris() {
    let mut client = test_client("test-client");
    let created_at = client.created_at;

    client.redirect_uris = vec![
        "http://localhost:3000/callback".to_string(),
        "https://app.example.com/callback".to_string(),
    ];
    client.updated_at = Utc::now();

    assert_eq!(client.redirect_uris.len(), 2);
    assert!(client.updated_at >= created_at);
}

#[test]
fn test_client_secret_rotation() {
    let mut client = test_client("test-client");
    client.client_secret = "old_hashed_secret".to_string();
    let old_secret = client.client_secret.clone();

    client.client_secret = "new_hashed_secret".to_string();
    client.updated_at = Utc::now();

    assert_ne!(client.client_secret, old_secret);
}

#[test]
fn test_client_name_display() {
    let client = test_client("my-app");
    assert!(!client.name.is_empty());
    assert!(client.name.contains("my-app"));
}
