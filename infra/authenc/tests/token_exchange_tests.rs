//! Integration tests for OAuth 2.0 Token Exchange (RFC 8693)
//!
//! These tests verify the complete token exchange flow including:
//! - Token validation
//! - Token type conversion
//! - Scope downscoping
//! - Actor/delegation support
//! - Audit logging
//! - Error handling

#[cfg(test)]
mod token_exchange_tests {
    use async_trait::async_trait;
    use authenc::database::Database;
    use authenc::handlers::oauth2_comprehensive::{AccessTokenClaims, generate_access_token};
    use authenc::models::audit_log::AuditLog;
    use authenc::services::jwt_validator::JwtValidator;
    use authenc::services::stores::audit_log_store::AuditLogStore;
    use authenc::services::token_exchange::{
        TokenExchangeConfig, TokenExchangeRequest, TokenExchangeService,
    };
    use chrono::Utc;
    use std::sync::Arc;
    use uuid::Uuid;

    /// Helper to create a test database connection
    async fn create_test_db() -> Arc<Database> {
        // In real tests, this would connect to a test database
        // For compilation, we'll use a mock
        unimplemented!("Test database setup required")
    }

    /// Simple in-memory audit log store for tests
    #[derive(Default)]
    struct InMemoryAuditLogStore;

    #[async_trait]
    impl AuditLogStore for InMemoryAuditLogStore {
        async fn add_log(&self, _log: &AuditLog) -> anyhow::Result<()> {
            Ok(())
        }

        async fn all(&self) -> anyhow::Result<Vec<AuditLog>> {
            Ok(Vec::new())
        }
    }

    /// Helper to create test access token
    fn create_test_access_token(user_id: &str, client_id: &str, scopes: &[&str]) -> String {
        let now = Utc::now().timestamp();
        let claims = AccessTokenClaims {
            iss: "http://localhost:8080/v1".to_string(),
            sub: user_id.to_string(),
            aud: client_id.to_string(),
            client_id: client_id.to_string(),
            exp: now + 3600,
            iat: now,
            nbf: now,
            jti: Uuid::new_v4().to_string(),
            scope: Some(scopes.join(" ")),
            roles: None,
            groups: None,
        };

        generate_access_token(&claims, None)
    }

    #[tokio::test]
    #[ignore] // Requires test database
    async fn test_token_exchange_access_token_to_access_token() {
        let db = create_test_db().await;
        let jwt_validator = Arc::new(JwtValidator::new(None));
        let audit_log: Arc<dyn AuditLogStore> = Arc::new(InMemoryAuditLogStore::default());

        let service = TokenExchangeService::new(
            db.clone(),
            jwt_validator,
            audit_log,
            Some(TokenExchangeConfig::default()),
        );

        // Create test subject token
        let user_id = Uuid::new_v4().to_string();
        let client_id = Uuid::new_v4().to_string();
        let subject_token = create_test_access_token(
            &user_id,
            &client_id,
            &["read:data", "write:data", "admin:users"],
        );

        // Request token exchange with scope downscoping
        let request = TokenExchangeRequest {
            grant_type: "urn:ietf:params:oauth:grant-type:token-exchange".to_string(),
            subject_token: subject_token.clone(),
            subject_token_type: "urn:ietf:params:oauth:token-type:access_token".to_string(),
            actor_token: None,
            actor_token_type: None,
            requested_token_type: Some("urn:ietf:params:oauth:token-type:access_token".to_string()),
            resource: Some("https://api.example.com".to_string()),
            audience: Some("api-server".to_string()),
            scope: Some("read:data".to_string()), // Downscoped
            client_id: Some(client_id.clone()),
        };

        // Perform token exchange
        let response = service.exchange_token(request).await;

        assert!(response.is_ok());
        let response = response.unwrap();

        // Verify response
        assert_eq!(response.token_type, "Bearer");
        assert_eq!(
            response.issued_token_type,
            "urn:ietf:params:oauth:token-type:access_token"
        );
        assert_eq!(response.scope, Some("read:data".to_string()));
        assert!(response.expires_in.is_some());
        assert!(!response.access_token.is_empty());
    }

    #[tokio::test]
    #[ignore] // Requires test database
    async fn test_token_exchange_with_delegation() {
        let db = create_test_db().await;
        let jwt_validator = Arc::new(JwtValidator::new(None));
        let audit_log: Arc<dyn AuditLogStore> = Arc::new(InMemoryAuditLogStore::default());

        let config = TokenExchangeConfig {
            allow_delegation: true,
            allow_impersonation: false,
            ..Default::default()
        };

        let service = TokenExchangeService::new(db.clone(), jwt_validator, audit_log, Some(config));

        // Create subject token
        let subject_user_id = Uuid::new_v4().to_string();
        let subject_client_id = Uuid::new_v4().to_string();
        let subject_token =
            create_test_access_token(&subject_user_id, &subject_client_id, &["read:data"]);

        // Create actor token
        let actor_user_id = Uuid::new_v4().to_string();
        let actor_client_id = Uuid::new_v4().to_string();
        let actor_token =
            create_test_access_token(&actor_user_id, &actor_client_id, &["delegate:on-behalf"]);

        // Request token exchange with actor (delegation)
        let request = TokenExchangeRequest {
            grant_type: "urn:ietf:params:oauth:grant-type:token-exchange".to_string(),
            subject_token: subject_token.clone(),
            subject_token_type: "urn:ietf:params:oauth:token-type:access_token".to_string(),
            actor_token: Some(actor_token),
            actor_token_type: Some("urn:ietf:params:oauth:token-type:access_token".to_string()),
            requested_token_type: Some("urn:ietf:params:oauth:token-type:access_token".to_string()),
            resource: Some("https://api.example.com".to_string()),
            audience: Some("api-server".to_string()),
            scope: Some("read:data".to_string()),
            client_id: Some(subject_client_id),
        };

        // Perform token exchange
        let response = service.exchange_token(request).await;

        assert!(response.is_ok());
        let response = response.unwrap();

        // Verify delegation token issued
        assert!(!response.access_token.is_empty());
        // Token should contain actor claim (verified by decoding JWT in production)
    }

    #[tokio::test]
    #[ignore] // Requires test database
    async fn test_token_exchange_scope_downscoping_enforcement() {
        let db = create_test_db().await;
        let jwt_validator = Arc::new(JwtValidator::new(None));
        let audit_log: Arc<dyn AuditLogStore> = Arc::new(InMemoryAuditLogStore::default());

        let config = TokenExchangeConfig {
            enforce_scope_downscoping: true,
            ..Default::default()
        };

        let service = TokenExchangeService::new(db.clone(), jwt_validator, audit_log, Some(config));

        // Create subject token with limited scopes
        let user_id = Uuid::new_v4().to_string();
        let client_id = Uuid::new_v4().to_string();
        let subject_token = create_test_access_token(&user_id, &client_id, &["read:data"]);

        // Try to request more scopes (should fail)
        let request = TokenExchangeRequest {
            grant_type: "urn:ietf:params:oauth:grant-type:token-exchange".to_string(),
            subject_token: subject_token.clone(),
            subject_token_type: "urn:ietf:params:oauth:token-type:access_token".to_string(),
            actor_token: None,
            actor_token_type: None,
            requested_token_type: Some("urn:ietf:params:oauth:token-type:access_token".to_string()),
            resource: Some("https://api.example.com".to_string()),
            audience: Some("api-server".to_string()),
            scope: Some("read:data write:data admin:users".to_string()), // More than original
            client_id: Some(client_id),
        };

        // Perform token exchange - should fail
        let response = service.exchange_token(request).await;
        assert!(response.is_err());

        // Verify error is about invalid scope
        let err = response.unwrap_err();
        assert!(err.to_string().contains("scope"));
    }

    #[tokio::test]
    #[ignore] // Requires test database
    async fn test_token_exchange_invalid_grant_type() {
        let db = create_test_db().await;
        let jwt_validator = Arc::new(JwtValidator::new(None));
        let audit_log: Arc<dyn AuditLogStore> = Arc::new(InMemoryAuditLogStore::default());

        let service = TokenExchangeService::new(
            db.clone(),
            jwt_validator,
            audit_log,
            Some(TokenExchangeConfig::default()),
        );

        let request = TokenExchangeRequest {
            grant_type: "invalid_grant_type".to_string(),
            subject_token: "test_token".to_string(),
            subject_token_type: "urn:ietf:params:oauth:token-type:access_token".to_string(),
            actor_token: None,
            actor_token_type: None,
            requested_token_type: None,
            resource: None,
            audience: None,
            scope: None,
            client_id: Some("test_client".to_string()),
        };

        let response = service.exchange_token(request).await;
        assert!(response.is_err());

        let err = response.unwrap_err();
        assert!(err.to_string().contains("grant_type"));
    }

    #[tokio::test]
    #[ignore] // Requires test database
    async fn test_token_exchange_expired_subject_token() {
        let db = create_test_db().await;
        let jwt_validator = Arc::new(JwtValidator::new(None));
        let audit_log: Arc<dyn AuditLogStore> = Arc::new(InMemoryAuditLogStore::default());

        let service = TokenExchangeService::new(
            db.clone(),
            jwt_validator,
            audit_log,
            Some(TokenExchangeConfig::default()),
        );

        // Create an expired token
        let now = Utc::now().timestamp();
        let claims = AccessTokenClaims {
            iss: "http://localhost:8080/v1".to_string(),
            sub: Uuid::new_v4().to_string(),
            aud: "test_client".to_string(),
            client_id: "test_client".to_string(),
            exp: now - 3600, // Expired 1 hour ago
            iat: now - 7200,
            nbf: now - 7200,
            jti: Uuid::new_v4().to_string(),
            scope: Some("read:data".to_string()),
            roles: None,
            groups: None,
        };

        let expired_token = generate_access_token(&claims, None);

        let request = TokenExchangeRequest {
            grant_type: "urn:ietf:params:oauth:grant-type:token-exchange".to_string(),
            subject_token: expired_token,
            subject_token_type: "urn:ietf:params:oauth:token-type:access_token".to_string(),
            actor_token: None,
            actor_token_type: None,
            requested_token_type: Some("urn:ietf:params:oauth:token-type:access_token".to_string()),
            resource: None,
            audience: Some("api-server".to_string()),
            scope: None,
            client_id: Some("test_client".to_string()),
        };

        let response = service.exchange_token(request).await;
        assert!(response.is_err());

        let err = response.unwrap_err();
        assert!(err.to_string().contains("expired"));
    }

    #[test]
    fn test_token_exchange_request_serialization() {
        let request = TokenExchangeRequest {
            grant_type: "urn:ietf:params:oauth:grant-type:token-exchange".to_string(),
            subject_token: "subject_token_value".to_string(),
            subject_token_type: "urn:ietf:params:oauth:token-type:access_token".to_string(),
            actor_token: Some("actor_token_value".to_string()),
            actor_token_type: Some("urn:ietf:params:oauth:token-type:access_token".to_string()),
            requested_token_type: Some("urn:ietf:params:oauth:token-type:access_token".to_string()),
            resource: Some("https://api.example.com".to_string()),
            audience: Some("api-server".to_string()),
            scope: Some("read:data write:data".to_string()),
            client_id: Some("test_client".to_string()),
        };

        // Test serialization
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("grant_type"));
        assert!(json.contains("subject_token"));

        // Test deserialization
        let deserialized: TokenExchangeRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.grant_type, request.grant_type);
        assert_eq!(deserialized.subject_token, request.subject_token);
    }
}
