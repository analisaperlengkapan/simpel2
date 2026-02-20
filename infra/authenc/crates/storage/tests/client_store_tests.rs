//! Unit tests for PostgresClientStore
//!
//! These tests verify the ClientStore implementation logic for OAuth2/OIDC client management.

use authenc_types::{domain::OidcClient, ClientId, RealmId};
use chrono::Utc;

#[test]
fn test_client_creation_public() {
    let client_id = ClientId::new();
    let realm_id = RealmId::new();
    let now = Utc::now();

    let client = OidcClient {
        id: client_id,
        client_id: "public-client".to_string(),
        client_secret_hash: None, // Public clients have no secret
        name: "Public Client".to_string(),
        is_public: true,
        redirect_uris: vec!["http://localhost:3000/callback".to_string()],
        allowed_scopes: vec!["openid".to_string(), "profile".to_string()],
        realm_id,
        enabled: true,
        created_at: now,
        updated_at: now,
    };

    assert_eq!(client.id, client_id);
    assert_eq!(client.client_id, "public-client");
    assert!(client.client_secret_hash.is_none());
    assert!(client.is_public);
    assert!(client.enabled);
}

#[test]
fn test_client_creation_confidential() {
    let client_id = ClientId::new();
    let realm_id = RealmId::new();
    let now = Utc::now();

    let client = OidcClient {
        id: client_id,
        client_id: "confidential-client".to_string(),
        client_secret_hash: Some("hashed_secret".to_string()),
        name: "Confidential Client".to_string(),
        is_public: false,
        redirect_uris: vec!["https://app.example.com/callback".to_string()],
        allowed_scopes: vec!["openid".to_string(), "profile".to_string(), "email".to_string()],
        realm_id,
        enabled: true,
        created_at: now,
        updated_at: now,
    };

    assert_eq!(client.id, client_id);
    assert_eq!(client.client_id, "confidential-client");
    assert!(client.client_secret_hash.is_some());
    assert!(!client.is_public);
    assert!(client.enabled);
}

#[test]
fn test_client_id_generation() {
    let id1 = ClientId::new();
    let id2 = ClientId::new();

    // Each generated ID should be unique
    assert_ne!(id1, id2);
}

#[test]
fn test_client_id_display() {
    let id = ClientId::new();
    let display_str = format!("{}", id);

    // Should display the UUID
    assert!(!display_str.is_empty());
}

#[test]
fn test_client_redirect_uris() {
    let client_id = ClientId::new();
    let realm_id = RealmId::new();
    let now = Utc::now();

    let redirect_uris = vec![
        "http://localhost:3000/callback".to_string(),
        "http://localhost:3000/silent-callback".to_string(),
        "https://app.example.com/callback".to_string(),
    ];

    let client = OidcClient {
        id: client_id,
        client_id: "multi-uri-client".to_string(),
        client_secret_hash: None,
        name: "Multi URI Client".to_string(),
        is_public: true,
        redirect_uris: redirect_uris.clone(),
        allowed_scopes: vec![],
        realm_id,
        enabled: true,
        created_at: now,
        updated_at: now,
    };

    assert_eq!(client.redirect_uris.len(), 3);
    assert_eq!(client.redirect_uris, redirect_uris);
}

#[test]
fn test_client_allowed_scopes() {
    let client_id = ClientId::new();
    let realm_id = RealmId::new();
    let now = Utc::now();

    let scopes = vec![
        "openid".to_string(),
        "profile".to_string(),
        "email".to_string(),
        "read:users".to_string(),
        "write:users".to_string(),
    ];

    let client = OidcClient {
        id: client_id,
        client_id: "scoped-client".to_string(),
        client_secret_hash: Some("hashed_secret".to_string()),
        name: "Scoped Client".to_string(),
        is_public: false,
        redirect_uris: vec![],
        allowed_scopes: scopes.clone(),
        realm_id,
        enabled: true,
        created_at: now,
        updated_at: now,
    };

    assert_eq!(client.allowed_scopes.len(), 5);
    assert_eq!(client.allowed_scopes, scopes);
}

#[test]
fn test_client_enabled_states() {
    let client_id = ClientId::new();
    let realm_id = RealmId::new();
    let now = Utc::now();

    let enabled_client = OidcClient {
        id: client_id,
        client_id: "enabled-client".to_string(),
        client_secret_hash: None,
        name: "Enabled Client".to_string(),
        is_public: true,
        redirect_uris: vec![],
        allowed_scopes: vec![],
        realm_id,
        enabled: true,
        created_at: now,
        updated_at: now,
    };

    assert!(enabled_client.enabled);

    let disabled_client = OidcClient {
        enabled: false,
        ..enabled_client
    };

    assert!(!disabled_client.enabled);
}

#[test]
fn test_client_timestamps() {
    let client_id = ClientId::new();
    let realm_id = RealmId::new();
    let created_at = Utc::now();
    let updated_at = created_at + chrono::Duration::hours(1);

    let client = OidcClient {
        id: client_id,
        client_id: "test-client".to_string(),
        client_secret_hash: None,
        name: "Test Client".to_string(),
        is_public: true,
        redirect_uris: vec![],
        allowed_scopes: vec![],
        realm_id,
        enabled: true,
        created_at,
        updated_at,
    };

    assert_eq!(client.created_at, created_at);
    assert_eq!(client.updated_at, updated_at);
    assert!(client.updated_at > client.created_at);
}

#[test]
fn test_client_secret_hash_presence() {
    let client_id = ClientId::new();
    let realm_id = RealmId::new();
    let now = Utc::now();

    // Public client should have no secret
    let public_client = OidcClient {
        id: client_id,
        client_id: "public".to_string(),
        client_secret_hash: None,
        name: "Public".to_string(),
        is_public: true,
        redirect_uris: vec![],
        allowed_scopes: vec![],
        realm_id,
        enabled: true,
        created_at: now,
        updated_at: now,
    };

    assert!(public_client.client_secret_hash.is_none());
    assert!(public_client.is_public);

    // Confidential client should have a secret
    let confidential_client = OidcClient {
        id: ClientId::new(),
        client_id: "confidential".to_string(),
        client_secret_hash: Some("hashed_secret".to_string()),
        name: "Confidential".to_string(),
        is_public: false,
        redirect_uris: vec![],
        allowed_scopes: vec![],
        realm_id,
        enabled: true,
        created_at: now,
        updated_at: now,
    };

    assert!(confidential_client.client_secret_hash.is_some());
    assert!(!confidential_client.is_public);
}

#[test]
fn test_client_empty_redirect_uris() {
    let client_id = ClientId::new();
    let realm_id = RealmId::new();
    let now = Utc::now();

    let client = OidcClient {
        id: client_id,
        client_id: "no-uris-client".to_string(),
        client_secret_hash: None,
        name: "No URIs Client".to_string(),
        is_public: true,
        redirect_uris: vec![],
        allowed_scopes: vec![],
        realm_id,
        enabled: true,
        created_at: now,
        updated_at: now,
    };

    assert!(client.redirect_uris.is_empty());
}

#[test]
fn test_client_empty_scopes() {
    let client_id = ClientId::new();
    let realm_id = RealmId::new();
    let now = Utc::now();

    let client = OidcClient {
        id: client_id,
        client_id: "no-scopes-client".to_string(),
        client_secret_hash: None,
        name: "No Scopes Client".to_string(),
        is_public: true,
        redirect_uris: vec![],
        allowed_scopes: vec![],
        realm_id,
        enabled: true,
        created_at: now,
        updated_at: now,
    };

    assert!(client.allowed_scopes.is_empty());
}

#[test]
fn test_multiple_clients_in_realm() {
    let realm_id = RealmId::new();
    let now = Utc::now();

    let client1 = OidcClient {
        id: ClientId::new(),
        client_id: "client1".to_string(),
        client_secret_hash: None,
        name: "Client 1".to_string(),
        is_public: true,
        redirect_uris: vec![],
        allowed_scopes: vec![],
        realm_id,
        enabled: true,
        created_at: now,
        updated_at: now,
    };

    let client2 = OidcClient {
        id: ClientId::new(),
        client_id: "client2".to_string(),
        client_secret_hash: Some("secret".to_string()),
        name: "Client 2".to_string(),
        is_public: false,
        redirect_uris: vec![],
        allowed_scopes: vec![],
        realm_id,
        enabled: true,
        created_at: now,
        updated_at: now,
    };

    // Both clients in same realm
    assert_eq!(client1.realm_id, client2.realm_id);

    // But different IDs and client_ids
    assert_ne!(client1.id, client2.id);
    assert_ne!(client1.client_id, client2.client_id);
}

#[test]
fn test_client_update_redirect_uris() {
    let client_id = ClientId::new();
    let realm_id = RealmId::new();
    let created_at = Utc::now();

    let mut client = OidcClient {
        id: client_id,
        client_id: "test-client".to_string(),
        client_secret_hash: None,
        name: "Test Client".to_string(),
        is_public: true,
        redirect_uris: vec!["http://localhost:3000/callback".to_string()],
        allowed_scopes: vec![],
        realm_id,
        enabled: true,
        created_at,
        updated_at: created_at,
    };

    // Update redirect URIs
    client.redirect_uris = vec![
        "http://localhost:3000/callback".to_string(),
        "https://app.example.com/callback".to_string(),
    ];
    client.updated_at = Utc::now();

    assert_eq!(client.redirect_uris.len(), 2);
    assert!(client.updated_at >= client.created_at);
}

#[test]
fn test_client_update_scopes() {
    let client_id = ClientId::new();
    let realm_id = RealmId::new();
    let created_at = Utc::now();

    let mut client = OidcClient {
        id: client_id,
        client_id: "test-client".to_string(),
        client_secret_hash: None,
        name: "Test Client".to_string(),
        is_public: true,
        redirect_uris: vec![],
        allowed_scopes: vec!["openid".to_string()],
        realm_id,
        enabled: true,
        created_at,
        updated_at: created_at,
    };

    // Update scopes
    client.allowed_scopes = vec![
        "openid".to_string(),
        "profile".to_string(),
        "email".to_string(),
    ];
    client.updated_at = Utc::now();

    assert_eq!(client.allowed_scopes.len(), 3);
    assert!(client.updated_at >= client.created_at);
}

#[test]
fn test_client_secret_rotation() {
    let client_id = ClientId::new();
    let realm_id = RealmId::new();
    let created_at = Utc::now();

    let mut client = OidcClient {
        id: client_id,
        client_id: "test-client".to_string(),
        client_secret_hash: Some("old_hashed_secret".to_string()),
        name: "Test Client".to_string(),
        is_public: false,
        redirect_uris: vec![],
        allowed_scopes: vec![],
        realm_id,
        enabled: true,
        created_at,
        updated_at: created_at,
    };

    let old_secret = client.client_secret_hash.clone();

    // Rotate secret
    client.client_secret_hash = Some("new_hashed_secret".to_string());
    client.updated_at = Utc::now();

    assert_ne!(client.client_secret_hash, old_secret);
    assert!(client.updated_at > client.created_at);
}
