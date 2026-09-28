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
/// - The revocation list is consulted after signature/expiry checks, so tokens
///   invalidated before their `exp` (logout, role change) are rejected.
pub async fn validate_token_handler(
    State(state): State<Arc<ApiState>>,
    Json(request): Json<ValidateTokenRequest>,
) -> Result<Json<ValidateTokenResponse>, ErrorResponse> {
    match state.jwt_service.verify_token(&request.token) {
        Ok(claims) => {
            // Revocation check — a signature-valid, unexpired token may still
            // have been revoked. Treat a revoked token as invalid; on a store
            // error fail closed (invalid) rather than leaking an authenticated
            // verdict on incomplete information.
            match state
                .revocation_store
                .is_revoked(&claims.jti, claims.sid.as_deref(), &claims.sub, claims.iat)
                .await
            {
                Ok(true) => return Ok(Json(invalid_response("Token has been revoked"))),
                Ok(false) => {}
                Err(e) => {
                    tracing::error!("Revocation check failed: {}", e);
                    return Ok(Json(invalid_response("Token validation failed")));
                }
            }

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
        Err(e) => {
            // Return a generic error category instead of the raw JWT library
            // message to avoid leaking internal details (e.g. "Invalid issuer:
            // expected X, got Y") to unauthenticated callers.
            let error_msg = match &e {
                authenc_types::error::AuthencError::TokenExpired => "Token has expired",
                authenc_types::error::AuthencError::InvalidToken(_) => "Token is invalid",
                _ => "Token validation failed",
            };
            Ok(Json(invalid_response(error_msg)))
        }
    }
}

/// Build an `invalid` validation response with a generic error message.
fn invalid_response(error: &str) -> ValidateTokenResponse {
    ValidateTokenResponse {
        valid: false,
        user_id: None,
        username: None,
        email: None,
        realm_id: None,
        scope: None,
        exp: None,
        iat: None,
        error: Some(error.to_string()),
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
    State(state): State<Arc<ApiState>>,
    Json(request): Json<IntrospectRequest>,
) -> Result<Json<IntrospectResponse>, ErrorResponse> {
    // RFC 7662: an `active` token is one that was issued by this server, has not
    // expired, and has not been revoked. Any failure (bad signature, expired,
    // revoked) is reported uniformly as `active: false` with no other claims —
    // the spec mandates not distinguishing the reason to avoid token probing.
    let claims = match state.jwt_service.verify_token(&request.token) {
        Ok(c) => c,
        Err(_) => return Ok(Json(IntrospectResponse::inactive())),
    };

    // Revoked tokens are inactive. Fail closed on a store error.
    match state
        .revocation_store
        .is_revoked(&claims.jti, claims.sid.as_deref(), &claims.sub, claims.iat)
        .await
    {
        Ok(false) => {}
        Ok(true) => return Ok(Json(IntrospectResponse::inactive())),
        Err(e) => {
            tracing::error!("Revocation check failed during introspection: {}", e);
            return Ok(Json(IntrospectResponse::inactive()));
        }
    }

    let username = claims
        .custom
        .get("preferred_username")
        .and_then(|v| v.as_str().map(|s| s.to_string()));

    Ok(Json(IntrospectResponse {
        active: true,
        scope: claims.scope,
        client_id: claims
            .custom
            .get("client_id")
            .and_then(|v| v.as_str().map(|s| s.to_string())),
        username,
        token_type: Some("Bearer".to_string()),
        exp: Some(claims.exp),
        iat: Some(claims.iat),
        sub: Some(claims.sub),
    }))
}

impl IntrospectResponse {
    /// RFC 7662 inactive response: only `active: false`, no other claims.
    fn inactive() -> Self {
        Self {
            active: false,
            scope: None,
            client_id: None,
            username: None,
            token_type: None,
            exp: None,
            iat: None,
            sub: None,
        }
    }
}

/// Token revocation request.
///
/// Two modes:
/// - **RFC 7009 (self/client):** supply `token` — the presented token is
///   revoked by its `jti`. Possession of the token is sufficient authorization.
/// - **Admin:** supply `session_id` (revoke a whole session) or `user_id`
///   (revoke all of a user's tokens issued before now, e.g. role change /
///   deactivation). These require an admin bearer in the Authorization header.
#[derive(Debug, Deserialize)]
pub struct RevokeRequest {
    /// A specific token to revoke (RFC 7009). Revoked by its `jti`.
    #[serde(default)]
    pub token: Option<String>,
    /// RFC 7009 hint (`access_token` / `refresh_token`). Accepted, not required.
    #[serde(default)]
    pub token_type_hint: Option<String>,
    /// Admin: revoke an entire session by its `sid`.
    #[serde(default)]
    pub session_id: Option<String>,
    /// Admin: revoke all tokens for a user issued before now.
    #[serde(default)]
    pub user_id: Option<String>,
    /// Optional human-readable reason, stored for audit.
    #[serde(default)]
    pub reason: Option<String>,
}

/// Token revocation response.
#[derive(Debug, Serialize)]
pub struct RevokeResponse {
    /// Always `true` on a 2xx — revocation is idempotent (RFC 7009 returns 200
    /// even for already-invalid tokens).
    pub revoked: bool,
}

/// POST /api/v1/auth/revoke — revoke a token, session, or all of a user's tokens.
///
/// Frontends call this on logout (with the access token). Admin tooling calls it
/// with `user_id`/`session_id` on role change or deactivation. The matching
/// `is_revoked` check in `validate`/`introspect` (REST) and the gRPC
/// `validate_token` then rejects affected tokens before their `exp`.
pub async fn revoke_handler(
    State(state): State<Arc<ApiState>>,
    headers: axum::http::HeaderMap,
    Json(request): Json<RevokeRequest>,
) -> Result<Json<RevokeResponse>, ErrorResponse> {
    let reason = request.reason.as_deref();

    // RFC 7009: revoke a specific presented token by its jti. Invalid/expired
    // tokens still yield 200 (nothing to revoke) — never leak token state here.
    if let Some(token) = &request.token
        && let Ok(claims) = state.jwt_service.verify_token(token)
    {
        let expires_at = chrono::DateTime::<chrono::Utc>::from_timestamp(claims.exp, 0)
            .unwrap_or_else(|| chrono::Utc::now() + state.jwt_service.refresh_token_ttl());
        state
            .revocation_store
            .revoke_jti(&claims.jti, expires_at, reason)
            .await
            .map_err(internal_error)?;
    }

    // Admin-scoped revocations require an admin bearer token.
    if request.session_id.is_some() || request.user_id.is_some() {
        require_admin(&state, &headers)?;

        let expires_at = chrono::Utc::now() + state.jwt_service.refresh_token_ttl();
        if let Some(sid) = &request.session_id {
            state
                .revocation_store
                .revoke_session(sid, expires_at, reason)
                .await
                .map_err(internal_error)?;
        }
        if let Some(user_id) = &request.user_id {
            state
                .revocation_store
                .revoke_user(user_id, expires_at, reason)
                .await
                .map_err(internal_error)?;
        }
    } else if request.token.is_none() {
        return Err(ErrorResponse {
            status_code: axum::http::StatusCode::BAD_REQUEST,
            error: "invalid_request".to_string(),
            message: "Provide one of: token, session_id, user_id".to_string(),
        });
    }

    Ok(Json(RevokeResponse { revoked: true }))
}

/// Require that the Authorization bearer is a valid token carrying an
/// administrative realm role. Returns 401 if missing/invalid, 403 if not an admin.
fn require_admin(
    state: &Arc<ApiState>,
    headers: &axum::http::HeaderMap,
) -> Result<(), ErrorResponse> {
    let token = crate::handlers::auth_helpers::extract_bearer_token(headers).map_err(|_| {
        ErrorResponse {
            status_code: axum::http::StatusCode::UNAUTHORIZED,
            error: "unauthorized".to_string(),
            message: "Admin revocation requires a bearer token".to_string(),
        }
    })?;
    let claims = state
        .jwt_service
        .verify_token(&token)
        .map_err(|_| ErrorResponse {
            status_code: axum::http::StatusCode::UNAUTHORIZED,
            error: "unauthorized".to_string(),
            message: "Invalid or expired token".to_string(),
        })?;

    // The allowlist is `lib_core::authz::ADMIN_ROLES` — the same constant the
    // IAM middleware, the perlengkapan backend and both microfrontends gate
    // their admin surfaces on. This used to accept only the exact string
    // "admin", so `admin_pusat` and `superadmin` could revoke their own token
    // (the RFC 7009 path above needs no role) but were refused the admin-scoped
    // `user_id`/`session_id` revocation they are the intended callers of.
    let is_admin = claims
        .custom
        .get("realm_access")
        .and_then(|v| v.get("roles"))
        .and_then(|v| v.as_array())
        .is_some_and(|roles| {
            roles
                .iter()
                .filter_map(|r| r.as_str())
                .any(lib_core::authz::is_admin_role)
        });

    if is_admin {
        Ok(())
    } else {
        Err(ErrorResponse {
            status_code: axum::http::StatusCode::FORBIDDEN,
            error: "forbidden".to_string(),
            message: "Admin role required".to_string(),
        })
    }
}

/// Map an internal store error to a 500 without leaking details.
fn internal_error(e: authenc_types::error::AuthencError) -> ErrorResponse {
    tracing::error!("Revocation store error: {}", e);
    ErrorResponse {
        status_code: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        error: "internal_error".to_string(),
        message: "Failed to process revocation".to_string(),
    }
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

    /// The admin predicate behind `/api/v1/auth/revoke`'s `user_id`/`session_id`
    /// path must accept every role `ADMIN_ROLES` lists, not just the literal
    /// `"admin"`.
    ///
    /// The regression: the guard read `r.as_str() == Some("admin")`, so
    /// `admin_pusat` and `superadmin` — administrators everywhere else in the
    /// system — were refused the cross-user revocation they exist to perform.
    /// The assertion is over the shared allowlist, so adding a role there
    /// cannot silently leave this guard behind.
    #[test]
    fn admin_predicate_accepts_every_shared_admin_role() {
        let accepts = |roles: &[&str]| roles.iter().any(|r| lib_core::authz::is_admin_role(r));

        for admin in lib_core::authz::ADMIN_ROLES {
            assert!(
                accepts(&[admin]),
                "'{admin}' is an admin role and must be accepted"
            );
        }
    }

    /// Non-administrative roles must still be refused — the widened allowlist
    /// must not have widened the *boundary*.
    #[test]
    fn admin_predicate_refuses_non_admin_roles() {
        let accepts = |roles: &[&str]| roles.iter().any(|r| lib_core::authz::is_admin_role(r));

        for role in [
            "operator_satker",
            "validator_wilayah",
            "validator_pusat",
            "validator_satker",
            "approver_satker",
            "user",
            // Lookalikes: a prefix check would admit all of these.
            "admin_master_read_only",
            "administrator",
            "super_admin",
            "superuser",
        ] {
            assert!(
                !accepts(&[role]),
                "'{role}' must not be treated as an admin"
            );
        }
    }

    /// A satker-bound principal that also holds an unrelated role must not be
    /// promoted by the presence of extra roles.
    #[test]
    fn admin_predicate_requires_an_actual_admin_among_roles() {
        let accepts = |roles: &[&str]| roles.iter().any(|r| lib_core::authz::is_admin_role(r));

        assert!(!accepts(&["operator_satker", "validator_wilayah"]));
        assert!(accepts(&["operator_satker", "admin_pusat"]));
    }
}
