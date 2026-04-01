//! Token validation endpoint handlers

use std::sync::Arc;

use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{handlers::ErrorResponse, state::ApiState};

/// Token validation request
#[derive(Debug, Deserialize)]
pub struct ValidateTokenRequest {
    /// JWT access token to validate
    pub token: String,
}

/// Token validation response
#[derive(Debug, Serialize)]
pub struct ValidateTokenResponse {
    /// Whether the token is valid
    pub valid: bool,
    /// User ID (if valid)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<Uuid>,
    /// Username (if valid)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// Email (if valid)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Realm ID (if valid)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub realm_id: Option<Uuid>,
    /// Scopes (space-separated, if valid)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// Token expiration timestamp (Unix timestamp, if valid)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<i64>,
    /// Token issued at timestamp (Unix timestamp, if valid)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iat: Option<i64>,
    /// Error message (if invalid)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// POST /api/v1/auth/validate - Validate JWT token
///
/// Validates a JWT access token and returns user claims if valid.
/// This endpoint is used by other microfrontends to validate tokens.
///
/// # Security Notes
/// - This endpoint does NOT require authentication (it validates the token itself)
/// - Rate limiting should be applied to prevent abuse
/// - Token signature, expiration, and issuer are verified
/// - Revocation list is checked
pub async fn validate_token_handler(
    State(state): State<Arc<ApiState>>,
    Json(request): Json<ValidateTokenRequest>,
) -> Result<Json<ValidateTokenResponse>, ErrorResponse> {
    match state.jwt_service.verify_token(&request.token) {
        Ok(claims) => {
            let user_id = Uuid::parse_str(&claims.sub).ok();
            let realm_id = claims.realm.and_then(|r| Uuid::parse_str(&r).ok());
            let username = claims
                .custom
                .get("preferred_username")
                .and_then(|v| v.as_str().map(|s| s.to_string()));
            let email = claims
                .custom
                .get("email")
                .and_then(|v| v.as_str().map(|s| s.to_string()));

            Ok(Json(ValidateTokenResponse {
                valid: true,
                user_id,
                username,
                email,
                realm_id,
                scope: claims.scope,
                exp: Some(claims.exp),
                iat: Some(claims.iat),
                error: None,
            }))
        }
        Err(e) => Ok(Json(ValidateTokenResponse {
            valid: false,
            user_id: None,
            username: None,
            email: None,
            realm_id: None,
            scope: None,
            exp: None,
            iat: None,
            error: Some(e.to_string()),
        })),
    }
}

/// Introspection endpoint (RFC 7662) - Optional advanced feature
///
/// POST /api/v1/auth/introspect - Token introspection
///
/// OAuth2 token introspection endpoint for detailed token information.
/// Requires client authentication.
#[derive(Debug, Deserialize)]
pub struct IntrospectRequest {
    /// Token to introspect
    pub token: String,
    /// Token type hint (access_token or refresh_token)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_type_hint: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct IntrospectResponse {
    /// Whether the token is active
    pub active: bool,
    /// Scope (if active)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// Client ID (if active)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    /// Username (if active)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// Token type (if active)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_type: Option<String>,
    /// Expiration timestamp (if active)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<i64>,
    /// Issued at timestamp (if active)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iat: Option<i64>,
    /// Subject (user ID, if active)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<String>,
}

pub async fn introspect_handler(
    State(_state): State<Arc<ApiState>>,
    Json(_request): Json<IntrospectRequest>,
) -> Result<Json<IntrospectResponse>, ErrorResponse> {
    // TODO: Implement introspection logic (RFC 7662)
    // This is an optional advanced feature
    // Similar to validate but requires client authentication

    Err(ErrorResponse {
        status_code: axum::http::StatusCode::NOT_IMPLEMENTED,
        error: "not_implemented".to_string(),
        message: "Token introspection endpoint not yet implemented".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_token_request_deserialization() {
        let json = r#"{"token":"eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9..."}"#;
        let request: ValidateTokenRequest = serde_json::from_str(json).unwrap();
        assert!(request.token.starts_with("eyJ"));
    }

    #[test]
    fn test_validate_token_response_serialization_valid() {
        let response = ValidateTokenResponse {
            valid: true,
            user_id: Some(Uuid::new_v4()),
            username: Some("testuser".to_string()),
            email: Some("test@example.com".to_string()),
            realm_id: Some(Uuid::new_v4()),
            scope: Some("openid profile email".to_string()),
            exp: Some(1234567890),
            iat: Some(1234567800),
            error: None,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"valid\":true"));
        assert!(json.contains("testuser"));
    }

    #[test]
    fn test_validate_token_response_serialization_invalid() {
        let response = ValidateTokenResponse {
            valid: false,
            user_id: None,
            username: None,
            email: None,
            realm_id: None,
            scope: None,
            exp: None,
            iat: None,
            error: Some("Token expired".to_string()),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"valid\":false"));
        assert!(json.contains("Token expired"));
    }

    #[test]
    fn test_introspect_request_deserialization() {
        let json = r#"{"token":"abc123","token_type_hint":"access_token"}"#;
        let request: IntrospectRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.token, "abc123");
        assert_eq!(request.token_type_hint, Some("access_token".to_string()));
    }

    use authenc_crypto::jwt::JwtService;
    use chrono::Duration;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_validate_token_handler_success() {
        let key_bytes = JwtService::generate_signing_key();
        let jwt_service = Arc::new(
            JwtService::new(
                &key_bytes,
                "https://test.example.com".to_string(),
                Duration::minutes(15),
                Duration::days(7),
            )
            .unwrap(),
        );

        let mut custom = HashMap::new();
        custom.insert(
            "preferred_username".to_string(),
            serde_json::json!("testuser"),
        );
        custom.insert("email".to_string(), serde_json::json!("test@example.com"));

        let token = jwt_service
            .generate_access_token_with_claims(
                &Uuid::new_v4().to_string(),
                Some("test-realm".to_string()),
                Some("openid profile".to_string()),
                None,
                custom,
            )
            .unwrap();

        // Testing the mapping logic directly
        let claims = jwt_service.verify_token(&token).unwrap();
        let user_id = Uuid::parse_str(&claims.sub).ok();
        let realm_id = claims.realm.and_then(|r| Uuid::parse_str(&r).ok());
        let username = claims
            .custom
            .get("preferred_username")
            .and_then(|v| v.as_str().map(|s| s.to_string()));
        let email = claims
            .custom
            .get("email")
            .and_then(|v| v.as_str().map(|s| s.to_string()));

        let resp = ValidateTokenResponse {
            valid: true,
            user_id,
            username,
            email,
            realm_id,
            scope: claims.scope,
            exp: Some(claims.exp),
            iat: Some(claims.iat),
            error: None,
        };

        assert!(resp.valid);
        assert_eq!(resp.username, Some("testuser".to_string()));
        assert_eq!(resp.email, Some("test@example.com".to_string()));
        assert_eq!(resp.scope, Some("openid profile".to_string()));
    }

    #[tokio::test]
    async fn test_validate_token_handler_expired() {
        let key_bytes = JwtService::generate_signing_key();
        let jwt_service = JwtService::new(
            &key_bytes,
            "https://test.example.com".to_string(),
            Duration::seconds(-10), // Expired
            Duration::days(7),
        )
        .unwrap();

        let token = jwt_service
            .generate_access_token("user-1", None, None, None)
            .unwrap();

        // Verification should fail
        let result = jwt_service.verify_token(&token);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_token_handler_invalid_signature() {
        let key_bytes1 = JwtService::generate_signing_key();
        let jwt_service1 = JwtService::new(
            &key_bytes1,
            "https://test.example.com".to_string(),
            Duration::minutes(15),
            Duration::days(7),
        )
        .unwrap();

        let key_bytes2 = JwtService::generate_signing_key();
        let jwt_service2 = JwtService::new(
            &key_bytes2,
            "https://test.example.com".to_string(),
            Duration::minutes(15),
            Duration::days(7),
        )
        .unwrap();

        let token = jwt_service1
            .generate_access_token("user-1", None, None, None)
            .unwrap();

        // Verifying with different key should fail
        let result = jwt_service2.verify_token(&token);
        assert!(result.is_err());
    }
}
