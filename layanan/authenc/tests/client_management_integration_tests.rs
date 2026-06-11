//! Integration tests for Client Management API
//!
//! Tests the complete client management flow including:
//! - Client CRUD operations
//! - Dynamic Client Registration (DCR) flow
//! - Protocol mapper operations
//! - Client authentication
//! - Error handling
//!
//! Requirements: REQ-TEST-002, REQ-CLIENT-001

use authenc::AppConfig;
use authenc_storage::operations::client_registration::{
    create_client_with_metadata, create_initial_access_token, create_registration_policy,
    get_client_by_client_id, get_initial_access_token_by_hash,
};
use authenc_storage::operations::entities::oauth2::{
    create_client as db_create_client, get_all_clients, get_client_by_id as db_get_client_by_id,
};
use authenc_storage::operations::protocol_mappers::{
    create_protocol_mapper, get_protocol_mappers_for_client,
};
use authenc_storage::Database;
use authenc_types::domain::{
    ClientRegistrationPolicy, InitialAccessToken, OAuth2Client, ProtocolMapper,
};
use chrono::Utc;
use uuid::Uuid;

/// Test helper to create a test database
async fn setup_test_db() -> Database {
    let config = AppConfig::default();
    Database::new(&config.database)
        .await
        .expect("Failed to connect to test database")
}

/// Test helper to create a basic OAuth2 client
fn create_test_client(client_id: &str, client_type: &str) -> OAuth2Client {
    let now = Utc::now();
    OAuth2Client {
        id: Uuid::new_v4(),
        client_id: client_id.to_string(),
        client_secret_hash: if client_type == "confidential" {
            Some("hashed-secret".to_string())
        } else {
            None
        },
        client_name: Some(format!("Test Client {}", client_id)),
        client_type: client_type.to_string(),
        redirect_uris: vec!["https://example.com/callback".to_string()],
        scopes: vec!["openid".to_string(), "profile".to_string()],
        grant_types: vec!["authorization_code".to_string()],
        response_types: vec!["code".to_string()],
        token_endpoint_auth_method: Some(if client_type == "public" {
            "none".to_string()
        } else {
            "client_secret_basic".to_string()
        }),
        owner_id: None,
        realm_id: None,
        enabled: true,
        created_at: now,
        updated_at: now,
        deleted_at: None,
        logo_uri: None,
        client_uri: None,
        policy_uri: None,
        tos_uri: None,
        jwks_uri: None,
        jwks: None,
        sector_identifier_uri: None,
        subject_type: Some("public".to_string()),
        id_token_signed_response_alg: Some("RS256".to_string()),
        id_token_encrypted_response_alg: None,
        id_token_encrypted_response_enc: None,
        userinfo_signed_response_alg: None,
        userinfo_encrypted_response_alg: None,
        userinfo_encrypted_response_enc: None,
        request_object_signing_alg: None,
        request_object_encryption_alg: None,
        request_object_encryption_enc: None,
        token_endpoint_auth_signing_alg: None,
        default_max_age: None,
        require_auth_time: None,
        default_acr_values: None,
        initiate_login_uri: None,
        request_uris: None,
        application_type: Some("web".to_string()),
        contacts: None,
        client_id_issued_at: Some(now),
        client_secret_expires_at: None,
        software_id: None,
        software_version: None,
        registration_access_token_hash: None,
    }
}

#[tokio::test]
async fn test_client_crud_operations() {
    let db = setup_test_db().await;

    // Test 1: Create client
    let client = create_test_client("test-client-1", "public");
    let created = db_create_client(&db, &client).await.unwrap();

    assert_eq!(created.client_id, "test-client-1");
    assert_eq!(created.client_type, "public");
    assert_eq!(created.redirect_uris.len(), 1);
    assert!(created.enabled);

    // Test 2: Get client by ID
    let fetched = db_get_client_by_id(&db, &created.client_id)
        .await
        .unwrap()
        .expect("Client should exist");

    assert_eq!(fetched.client_id, created.client_id);
    assert_eq!(fetched.client_name, created.client_name);

    // Test 3: List all clients
    let all_clients = get_all_clients(&db).await.unwrap();
    assert!(!all_clients.is_empty());
    assert!(all_clients.iter().any(|c| c.client_id == "test-client-1"));

    // Test 4: Create another client
    let client2 = create_test_client("test-client-2", "confidential");
    let created2 = db_create_client(&db, &client2).await.unwrap();

    assert_eq!(created2.client_id, "test-client-2");
    assert_eq!(created2.client_type, "confidential");
    assert!(created2.client_secret_hash.is_some());

    // Test 5: Verify both clients exist
    let all_clients = get_all_clients(&db).await.unwrap();
    assert!(all_clients.len() >= 2);
}

#[tokio::test]
async fn test_client_types() {
    let db = setup_test_db().await;

    // Test public client (no secret)
    let public_client = create_test_client("public-client", "public");
    let created_public = db_create_client(&db, &public_client).await.unwrap();

    assert_eq!(created_public.client_type, "public");
    assert!(created_public.client_secret_hash.is_none());
    assert_eq!(
        created_public.token_endpoint_auth_method,
        Some("none".to_string())
    );

    // Test confidential client (with secret)
    let confidential_client = create_test_client("confidential-client", "confidential");
    let created_confidential = db_create_client(&db, &confidential_client).await.unwrap();

    assert_eq!(created_confidential.client_type, "confidential");
    assert!(created_confidential.client_secret_hash.is_some());
    assert_eq!(
        created_confidential.token_endpoint_auth_method,
        Some("client_secret_basic".to_string())
    );
}

#[tokio::test]
async fn test_dynamic_client_registration_flow() {
    let db = setup_test_db().await;
    let realm_id = Uuid::new_v4();

    // Step 1: Create registration policy
    let policy = ClientRegistrationPolicy {
        id: Uuid::new_v4(),
        realm_id: Some(realm_id),
        allow_dynamic_registration: true,
        require_initial_access_token: true,
        default_scopes: Some(vec!["openid".to_string()]),
        allowed_scopes: Some(vec![
            "openid".to_string(),
            "profile".to_string(),
            "email".to_string(),
        ]),
        require_software_statement: false,
        trusted_software_statement_issuers: None,
        registration_token_expires_in: Some(3600),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    create_registration_policy(&db, &policy).await.unwrap();

    // Step 2: Create initial access token
    let token_value = "initial-access-token-123";
    let token_hash = format!("hashed-{}", token_value); // In real code, use proper hashing

    let initial_token = InitialAccessToken {
        id: Uuid::new_v4(),
        token_hash: token_hash.clone(),
        realm_id: Some(realm_id),
        max_uses: Some(5),
        uses_count: 0,
        expires_at: Some(Utc::now() + chrono::Duration::hours(24)),
        created_at: Utc::now(),
        created_by: None,
    };

    create_initial_access_token(&db, &initial_token)
        .await
        .unwrap();

    // Step 3: Verify initial access token exists
    let fetched_token = get_initial_access_token_by_hash(&db, &token_hash)
        .await
        .unwrap()
        .expect("Token should exist");

    assert_eq!(fetched_token.token_hash, token_hash);
    assert_eq!(fetched_token.uses_count, 0);
    assert_eq!(fetched_token.max_uses, Some(5));

    // Step 4: Register client using DCR
    let mut dcr_client = create_test_client("dcr-client-1", "public");
    dcr_client.realm_id = Some(realm_id);
    dcr_client.software_id = Some("my-app-v1".to_string());
    dcr_client.software_version = Some("1.0.0".to_string());

    let registered_client = create_client_with_metadata(&db, &dcr_client).await.unwrap();

    assert_eq!(registered_client.client_id, "dcr-client-1");
    assert_eq!(registered_client.realm_id, Some(realm_id));
    assert_eq!(
        registered_client.software_id,
        Some("my-app-v1".to_string())
    );

    // Step 5: Verify client can be retrieved
    let fetched_client = get_client_by_client_id(&db, &registered_client.client_id)
        .await
        .unwrap()
        .expect("Client should exist");

    assert_eq!(fetched_client.client_id, registered_client.client_id);
    assert_eq!(fetched_client.software_version, Some("1.0.0".to_string()));
}

#[tokio::test]
async fn test_protocol_mapper_operations() {
    let db = setup_test_db().await;

    // Create a client first
    let client = create_test_client("mapper-test-client", "public");
    let created_client = db_create_client(&db, &client).await.unwrap();

    // Create protocol mapper for the client
    let mapper = ProtocolMapper {
        id: Uuid::new_v4(),
        name: "username-mapper".to_string(),
        protocol: "openid-connect".to_string(),
        protocol_mapper: "oidc-usermodel-property-mapper".to_string(),
        client_id: Some(created_client.id),
        client_scope_id: None,
        config: serde_json::json!({
            "user.attribute": "username",
            "claim.name": "preferred_username",
            "jsonType.label": "String",
            "id.token.claim": "true",
            "access.token.claim": "true",
            "userinfo.token.claim": "true"
        }),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    create_protocol_mapper(&db, &mapper).await.unwrap();

    // Create another mapper
    let email_mapper = ProtocolMapper {
        id: Uuid::new_v4(),
        name: "email-mapper".to_string(),
        protocol: "openid-connect".to_string(),
        protocol_mapper: "oidc-usermodel-property-mapper".to_string(),
        client_id: Some(created_client.id),
        client_scope_id: None,
        config: serde_json::json!({
            "user.attribute": "email",
            "claim.name": "email",
            "jsonType.label": "String",
            "id.token.claim": "true",
            "access.token.claim": "true",
            "userinfo.token.claim": "true"
        }),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    create_protocol_mapper(&db, &email_mapper).await.unwrap();

    // Retrieve all mappers for the client
    let mappers = get_protocol_mappers_for_client(&db, created_client.id)
        .await
        .unwrap();

    assert_eq!(mappers.len(), 2);
    assert!(mappers.iter().any(|m| m.name == "username-mapper"));
    assert!(mappers.iter().any(|m| m.name == "email-mapper"));

    // Verify mapper configuration
    let username_mapper = mappers
        .iter()
        .find(|m| m.name == "username-mapper")
        .unwrap();
    assert_eq!(username_mapper.protocol, "openid-connect");
    assert_eq!(
        username_mapper.protocol_mapper,
        "oidc-usermodel-property-mapper"
    );
    assert_eq!(
        username_mapper.config["claim.name"],
        "preferred_username"
    );
}

#[tokio::test]
async fn test_client_authentication_methods() {
    let db = setup_test_db().await;

    // Test 1: Public client with "none" authentication
    let public_client = create_test_client("public-auth-test", "public");
    let created_public = db_create_client(&db, &public_client).await.unwrap();

    assert_eq!(
        created_public.token_endpoint_auth_method,
        Some("none".to_string())
    );
    assert!(created_public.client_secret_hash.is_none());

    // Test 2: Confidential client with client_secret_basic
    let mut basic_client = create_test_client("basic-auth-test", "confidential");
    basic_client.token_endpoint_auth_method = Some("client_secret_basic".to_string());
    let created_basic = db_create_client(&db, &basic_client).await.unwrap();

    assert_eq!(
        created_basic.token_endpoint_auth_method,
        Some("client_secret_basic".to_string())
    );
    assert!(created_basic.client_secret_hash.is_some());

    // Test 3: Confidential client with client_secret_post
    let mut post_client = create_test_client("post-auth-test", "confidential");
    post_client.token_endpoint_auth_method = Some("client_secret_post".to_string());
    let created_post = db_create_client(&db, &post_client).await.unwrap();

    assert_eq!(
        created_post.token_endpoint_auth_method,
        Some("client_secret_post".to_string())
    );
    assert!(created_post.client_secret_hash.is_some());

    // Test 4: Confidential client with private_key_jwt
    let mut jwt_client = create_test_client("jwt-auth-test", "confidential");
    jwt_client.token_endpoint_auth_method = Some("private_key_jwt".to_string());
    jwt_client.jwks_uri = Some("https://example.com/.well-known/jwks.json".to_string());
    let created_jwt = db_create_client(&db, &jwt_client).await.unwrap();

    assert_eq!(
        created_jwt.token_endpoint_auth_method,
        Some("private_key_jwt".to_string())
    );
    assert_eq!(
        created_jwt.jwks_uri,
        Some("https://example.com/.well-known/jwks.json".to_string())
    );
}

#[tokio::test]
async fn test_client_grant_types_and_response_types() {
    let db = setup_test_db().await;

    // Test 1: Authorization Code flow
    let mut auth_code_client = create_test_client("auth-code-client", "confidential");
    auth_code_client.grant_types = vec!["authorization_code".to_string()];
    auth_code_client.response_types = vec!["code".to_string()];
    let created_auth_code = db_create_client(&db, &auth_code_client).await.unwrap();

    assert_eq!(created_auth_code.grant_types, vec!["authorization_code"]);
    assert_eq!(created_auth_code.response_types, vec!["code"]);

    // Test 2: Client Credentials flow
    let mut client_creds_client = create_test_client("client-creds-client", "confidential");
    client_creds_client.grant_types = vec!["client_credentials".to_string()];
    client_creds_client.response_types = vec![];
    let created_client_creds = db_create_client(&db, &client_creds_client).await.unwrap();

    assert_eq!(created_client_creds.grant_types, vec!["client_credentials"]);
    assert!(created_client_creds.response_types.is_empty());

    // Test 3: Refresh Token flow
    let mut refresh_client = create_test_client("refresh-client", "confidential");
    refresh_client.grant_types = vec![
        "authorization_code".to_string(),
        "refresh_token".to_string(),
    ];
    refresh_client.response_types = vec!["code".to_string()];
    let created_refresh = db_create_client(&db, &refresh_client).await.unwrap();

    assert_eq!(
        created_refresh.grant_types,
        vec!["authorization_code", "refresh_token"]
    );
    assert_eq!(created_refresh.response_types, vec!["code"]);

    // Test 4: Multiple grant types
    let mut multi_grant_client = create_test_client("multi-grant-client", "confidential");
    multi_grant_client.grant_types = vec![
        "authorization_code".to_string(),
        "client_credentials".to_string(),
        "refresh_token".to_string(),
    ];
    multi_grant_client.response_types = vec!["code".to_string()];
    let created_multi = db_create_client(&db, &multi_grant_client).await.unwrap();

    assert_eq!(created_multi.grant_types.len(), 3);
    assert!(created_multi
        .grant_types
        .contains(&"authorization_code".to_string()));
    assert!(created_multi
        .grant_types
        .contains(&"client_credentials".to_string()));
    assert!(created_multi
        .grant_types
        .contains(&"refresh_token".to_string()));
}

#[tokio::test]
async fn test_client_scopes() {
    let db = setup_test_db().await;

    // Test 1: Basic scopes
    let mut basic_scope_client = create_test_client("basic-scope-client", "public");
    basic_scope_client.scopes = vec!["openid".to_string(), "profile".to_string()];
    let created_basic = db_create_client(&db, &basic_scope_client).await.unwrap();

    assert_eq!(created_basic.scopes, vec!["openid", "profile"]);

    // Test 2: Extended scopes
    let mut extended_scope_client = create_test_client("extended-scope-client", "confidential");
    extended_scope_client.scopes = vec![
        "openid".to_string(),
        "profile".to_string(),
        "email".to_string(),
        "address".to_string(),
        "phone".to_string(),
    ];
    let created_extended = db_create_client(&db, &extended_scope_client).await.unwrap();

    assert_eq!(created_extended.scopes.len(), 5);
    assert!(created_extended.scopes.contains(&"openid".to_string()));
    assert!(created_extended.scopes.contains(&"email".to_string()));
    assert!(created_extended.scopes.contains(&"address".to_string()));

    // Test 3: Custom scopes
    let mut custom_scope_client = create_test_client("custom-scope-client", "confidential");
    custom_scope_client.scopes = vec![
        "openid".to_string(),
        "read:documents".to_string(),
        "write:documents".to_string(),
        "admin:users".to_string(),
    ];
    let created_custom = db_create_client(&db, &custom_scope_client).await.unwrap();

    assert_eq!(created_custom.scopes.len(), 4);
    assert!(created_custom
        .scopes
        .contains(&"read:documents".to_string()));
    assert!(created_custom
        .scopes
        .contains(&"write:documents".to_string()));
    assert!(created_custom.scopes.contains(&"admin:users".to_string()));
}

#[tokio::test]
async fn test_client_redirect_uris() {
    let db = setup_test_db().await;

    // Test 1: Single redirect URI
    let mut single_uri_client = create_test_client("single-uri-client", "public");
    single_uri_client.redirect_uris = vec!["https://example.com/callback".to_string()];
    let created_single = db_create_client(&db, &single_uri_client).await.unwrap();

    assert_eq!(created_single.redirect_uris.len(), 1);
    assert_eq!(
        created_single.redirect_uris[0],
        "https://example.com/callback"
    );

    // Test 2: Multiple redirect URIs
    let mut multi_uri_client = create_test_client("multi-uri-client", "confidential");
    multi_uri_client.redirect_uris = vec![
        "https://example.com/callback".to_string(),
        "https://example.com/callback2".to_string(),
        "https://app.example.com/oauth/callback".to_string(),
    ];
    let created_multi = db_create_client(&db, &multi_uri_client).await.unwrap();

    assert_eq!(created_multi.redirect_uris.len(), 3);
    assert!(created_multi
        .redirect_uris
        .contains(&"https://example.com/callback".to_string()));
    assert!(created_multi
        .redirect_uris
        .contains(&"https://app.example.com/oauth/callback".to_string()));

    // Test 3: Localhost redirect URI (for development)
    let mut localhost_client = create_test_client("localhost-client", "public");
    localhost_client.redirect_uris = vec!["http://localhost:3000/callback".to_string()];
    let created_localhost = db_create_client(&db, &localhost_client).await.unwrap();

    assert_eq!(created_localhost.redirect_uris.len(), 1);
    assert_eq!(
        created_localhost.redirect_uris[0],
        "http://localhost:3000/callback"
    );
}

#[tokio::test]
async fn test_client_metadata_fields() {
    let db = setup_test_db().await;

    // Create client with full metadata
    let mut metadata_client = create_test_client("metadata-client", "confidential");
    metadata_client.client_uri = Some("https://example.com".to_string());
    metadata_client.logo_uri = Some("https://example.com/logo.png".to_string());
    metadata_client.tos_uri = Some("https://example.com/tos".to_string());
    metadata_client.policy_uri = Some("https://example.com/privacy".to_string());
    metadata_client.contacts = Some(vec!["admin@example.com".to_string()]);
    metadata_client.software_id = Some("my-app".to_string());
    metadata_client.software_version = Some("2.0.0".to_string());

    let created = db_create_client(&db, &metadata_client).await.unwrap();

    assert_eq!(
        created.client_uri,
        Some("https://example.com".to_string())
    );
    assert_eq!(
        created.logo_uri,
        Some("https://example.com/logo.png".to_string())
    );
    assert_eq!(
        created.tos_uri,
        Some("https://example.com/tos".to_string())
    );
    assert_eq!(
        created.policy_uri,
        Some("https://example.com/privacy".to_string())
    );
    assert_eq!(
        created.contacts,
        Some(vec!["admin@example.com".to_string()])
    );
    assert_eq!(created.software_id, Some("my-app".to_string()));
    assert_eq!(created.software_version, Some("2.0.0".to_string()));
}

#[tokio::test]
async fn test_error_handling_invalid_client() {
    let db = setup_test_db().await;

    // Test 1: Get non-existent client
    let result = db_get_client_by_id(&db, "non-existent-client").await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());

    // Test 2: Get client by client_id (non-existent)
    let result = get_client_by_client_id(&db, "non-existent-client").await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_error_handling_invalid_token() {
    let db = setup_test_db().await;

    // Test: Get non-existent initial access token
    let result = get_initial_access_token_by_hash(&db, "non-existent-token-hash").await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_client_enabled_disabled() {
    let db = setup_test_db().await;

    // Create enabled client
    let mut enabled_client = create_test_client("enabled-client", "public");
    enabled_client.enabled = true;
    let created_enabled = db_create_client(&db, &enabled_client).await.unwrap();

    assert!(created_enabled.enabled);

    // Create disabled client
    let mut disabled_client = create_test_client("disabled-client", "public");
    disabled_client.enabled = false;
    let created_disabled = db_create_client(&db, &disabled_client).await.unwrap();

    assert!(!created_disabled.enabled);
}

#[tokio::test]
async fn test_client_realm_isolation() {
    let db = setup_test_db().await;
    let realm1 = Uuid::new_v4();
    let realm2 = Uuid::new_v4();

    // Create client in realm 1
    let mut client1 = create_test_client("realm1-client", "public");
    client1.realm_id = Some(realm1);
    let created1 = db_create_client(&db, &client1).await.unwrap();

    assert_eq!(created1.realm_id, Some(realm1));

    // Create client in realm 2
    let mut client2 = create_test_client("realm2-client", "public");
    client2.realm_id = Some(realm2);
    let created2 = db_create_client(&db, &client2).await.unwrap();

    assert_eq!(created2.realm_id, Some(realm2));

    // Verify clients are in different realms
    assert_ne!(created1.realm_id, created2.realm_id);
}

#[tokio::test]
async fn test_client_timestamps() {
    let db = setup_test_db().await;

    let before_creation = Utc::now();
    let client = create_test_client("timestamp-client", "public");
    let created = db_create_client(&db, &client).await.unwrap();
    let after_creation = Utc::now();

    // Verify created_at is within expected range
    assert!(created.created_at >= before_creation);
    assert!(created.created_at <= after_creation);

    // Verify updated_at is set
    assert!(created.updated_at >= before_creation);
    assert!(created.updated_at <= after_creation);

    // Verify client_id_issued_at is set
    assert!(created.client_id_issued_at.is_some());
    let issued_at = created.client_id_issued_at.unwrap();
    assert!(issued_at >= before_creation);
    assert!(issued_at <= after_creation);
}
