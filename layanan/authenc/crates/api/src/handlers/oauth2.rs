//! OAuth2/OIDC public endpoint handlers

use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
    response::Response,
};
use serde::{Deserialize, Serialize};

use crate::{handlers::ErrorResponse, state::ApiState};

/// OAuth2 authorization request parameters
#[derive(Debug, Deserialize)]
pub struct AuthorizeRequest {
    /// Response type (code, token, id_token)
    pub response_type: String,
    /// Client ID
    pub client_id: String,
    /// Redirect URI
    pub redirect_uri: String,
    /// Requested scopes (space-separated)
    pub scope: String,
    /// State parameter for CSRF protection
    pub state: Option<String>,
    /// PKCE code challenge
    pub code_challenge: Option<String>,
    /// PKCE code challenge method (S256 or plain)
    pub code_challenge_method: Option<String>,
    /// Nonce for OIDC
    pub nonce: Option<String>,
}

/// OAuth2 authorization response (redirect)
#[derive(Debug, Serialize)]
pub struct AuthorizeResponse {
    /// Authorization code
    pub code: String,
    /// State parameter (echoed back)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

/// OAuth2 token request
#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    /// Grant type (authorization_code, refresh_token, client_credentials)
    pub grant_type: String,
    /// Authorization code (for authorization_code grant)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// Redirect URI (must match authorization request)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_uri: Option<String>,
    /// Client ID
    pub client_id: String,
    /// Client secret (for confidential clients)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
    /// PKCE code verifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_verifier: Option<String>,
    /// Refresh token (for refresh_token grant)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    /// Scope (for client_credentials grant)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

/// OAuth2 token response
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    /// Access token
    pub access_token: String,
    /// Token type (always "Bearer")
    pub token_type: String,
    /// Token expiration in seconds
    pub expires_in: u64,
    /// Refresh token (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    /// ID token (for OIDC)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_token: Option<String>,
    /// Scope (space-separated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

/// OIDC Discovery document
#[derive(Debug, Serialize)]
pub struct OidcDiscoveryResponse {
    /// Issuer URL
    pub issuer: String,
    /// Authorization endpoint
    pub authorization_endpoint: String,
    /// Token endpoint
    pub token_endpoint: String,
    /// UserInfo endpoint
    pub userinfo_endpoint: String,
    /// JWKS URI
    pub jwks_uri: String,
    /// Supported response types
    pub response_types_supported: Vec<String>,
    /// Supported grant types
    pub grant_types_supported: Vec<String>,
    /// Supported subject types
    pub subject_types_supported: Vec<String>,
    /// Supported ID token signing algorithms
    pub id_token_signing_alg_values_supported: Vec<String>,
    /// Supported scopes
    pub scopes_supported: Vec<String>,
    /// Supported token endpoint auth methods
    pub token_endpoint_auth_methods_supported: Vec<String>,
    /// Supported claims
    pub claims_supported: Vec<String>,
    /// PKCE code challenge methods supported
    pub code_challenge_methods_supported: Vec<String>,
}

/// OIDC UserInfo response
#[derive(Debug, Serialize)]
pub struct UserInfoResponse {
    /// Subject (user ID)
    pub sub: String,
    /// Username
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_username: Option<String>,
    /// Email
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Email verified
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_verified: Option<bool>,
    /// Full name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Given name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub given_name: Option<String>,
    /// Family name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family_name: Option<String>,
}

/// GET /api/v1/oauth2/authorize - Authorization endpoint
///
/// OAuth2 authorization endpoint for initiating authorization code flow.
/// Validates client, redirect URI, and scopes, then redirects to login if needed.
pub async fn authorize_handler(
    State(_state): State<Arc<ApiState>>,
    Query(_request): Query<AuthorizeRequest>,
) -> Result<Response, ErrorResponse> {
    // TODO: Implement authorization logic
    // 1. Validate client_id and redirect_uri
    // 2. Validate scopes
    // 3. Check if user is authenticated (session cookie)
    // 4. If not authenticated, redirect to login page with return URL
    // 5. If authenticated, check consent
    // 6. Generate authorization code
    // 7. Redirect to redirect_uri with code and state

    Err(ErrorResponse {
        status_code: axum::http::StatusCode::NOT_IMPLEMENTED,
        error: "not_implemented".to_string(),
        message: "OAuth2 authorize endpoint not yet implemented".to_string(),
    })
}

/// POST /api/v1/oauth2/token - Token endpoint
///
/// OAuth2 token endpoint for exchanging authorization codes for tokens.
/// Supports authorization_code, refresh_token, and client_credentials grants.
pub async fn token_handler(
    State(state): State<Arc<ApiState>>,
    Json(request): Json<TokenRequest>,
) -> Result<Json<TokenResponse>, ErrorResponse> {
    use authenc_types::OAuth2Service;
    use authenc_types::domain_types::{RealmId, TokenRequest as DomainTokenRequest};
    use uuid::Uuid;

    // Default to master realm (all-zeros UUID)
    let realm_id = RealmId::from_uuid(
        Uuid::parse_str("00000000-0000-0000-0000-000000000000").expect("Invalid master realm UUID"),
    );

    let domain_request = DomainTokenRequest {
        grant_type: request.grant_type,
        code: request.code,
        redirect_uri: request.redirect_uri,
        code_verifier: request.code_verifier,
        client_id: request.client_id,
        client_secret: request.client_secret,
        refresh_token: request.refresh_token,
        scope: request.scope,
        realm_id,
    };

    match state.oauth2_service.token(domain_request).await {
        Ok(resp) => Ok(Json(TokenResponse {
            access_token: resp.access_token,
            token_type: resp.token_type,
            expires_in: resp.expires_in as u64,
            refresh_token: resp.refresh_token,
            id_token: None, // TODO: Generate ID token if openid scope requested
            scope: Some(resp.scope),
        })),
        Err(e) => Err(ErrorResponse {
            status_code: axum::http::StatusCode::BAD_REQUEST,
            error: "invalid_request".to_string(),
            message: e.to_string(),
        }),
    }
}

/// GET /api/v1/oauth2/.well-known/openid-configuration - Discovery endpoint
///
/// OIDC discovery endpoint returning server metadata.
pub async fn discovery_handler(
    State(_state): State<Arc<ApiState>>,
) -> Result<Json<OidcDiscoveryResponse>, ErrorResponse> {
    // TODO: Implement discovery document
    // Return static configuration with endpoint URLs

    let discovery = OidcDiscoveryResponse {
        issuer: "https://authenc.kejaksaan.go.id".to_string(),
        authorization_endpoint: "https://authenc.kejaksaan.go.id/api/v1/oauth2/authorize"
            .to_string(),
        token_endpoint: "https://authenc.kejaksaan.go.id/api/v1/oauth2/token".to_string(),
        userinfo_endpoint: "https://authenc.kejaksaan.go.id/api/v1/oauth2/userinfo".to_string(),
        jwks_uri: "https://authenc.kejaksaan.go.id/api/v1/oauth2/jwks".to_string(),
        response_types_supported: vec!["code".to_string(), "id_token".to_string()],
        grant_types_supported: vec![
            "authorization_code".to_string(),
            "refresh_token".to_string(),
            "client_credentials".to_string(),
        ],
        subject_types_supported: vec!["public".to_string()],
        id_token_signing_alg_values_supported: vec!["EdDSA".to_string()],
        scopes_supported: vec![
            "openid".to_string(),
            "profile".to_string(),
            "email".to_string(),
        ],
        token_endpoint_auth_methods_supported: vec![
            "client_secret_post".to_string(),
            "client_secret_basic".to_string(),
            "none".to_string(),
        ],
        claims_supported: vec![
            "sub".to_string(),
            "email".to_string(),
            "email_verified".to_string(),
            "name".to_string(),
            "preferred_username".to_string(),
        ],
        code_challenge_methods_supported: vec!["S256".to_string(), "plain".to_string()],
    };

    Ok(Json(discovery))
}

use crate::handlers::auth_helpers;
use axum::http::HeaderMap;

/// GET /api/v1/oauth2/userinfo - UserInfo endpoint
///
/// OIDC UserInfo endpoint returning claims about the authenticated user.
/// Requires valid access token with openid scope.
pub async fn userinfo_handler(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<UserInfoResponse>, ErrorResponse> {
    let user_uuid = auth_helpers::extract_user_from_token(&state, &headers)
        .await
        .map_err(|e| ErrorResponse {
            status_code: axum::http::StatusCode::UNAUTHORIZED,
            error: "unauthorized".to_string(),
            message: e.message,
        })?;

    let user_id = authenc_types::UserId::from_uuid(user_uuid);
    match state.user_service.get_user(user_id).await {
        Ok(user) => {
            let name = match (&user.first_name, &user.last_name) {
                (Some(f), Some(l)) => Some(format!("{} {}", f, l)),
                (Some(f), None) => Some(f.clone()),
                (None, Some(l)) => Some(l.clone()),
                _ => user.nama.clone(),
            };

            Ok(Json(UserInfoResponse {
                sub: user.id.to_string(),
                preferred_username: Some(user.username),
                email: Some(user.email),
                email_verified: Some(user.email_verified),
                name,
                given_name: user.first_name,
                family_name: user.last_name,
            }))
        }
        Err(e) => Err(ErrorResponse {
            status_code: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            error: "internal_error".to_string(),
            message: e.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authorize_request_deserialization() {
        let json = r#"{
            "response_type": "code",
            "client_id": "portal-client",
            "redirect_uri": "https://portal.kejaksaan.go.id/callback",
            "scope": "openid profile email",
            "state": "random-state"
        }"#;
        let request: AuthorizeRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.response_type, "code");
        assert_eq!(request.client_id, "portal-client");
    }

    #[test]
    fn test_token_response_serialization() {
        let response = TokenResponse {
            access_token: "token123".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: 900,
            refresh_token: Some("refresh123".to_string()),
            id_token: None,
            scope: Some("openid profile".to_string()),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("access_token"));
        assert!(json.contains("Bearer"));
    }

    #[test]
    fn test_discovery_response_serialization() {
        let discovery = OidcDiscoveryResponse {
            issuer: "https://authenc.test".to_string(),
            authorization_endpoint: "https://authenc.test/authorize".to_string(),
            token_endpoint: "https://authenc.test/token".to_string(),
            userinfo_endpoint: "https://authenc.test/userinfo".to_string(),
            jwks_uri: "https://authenc.test/jwks".to_string(),
            response_types_supported: vec!["code".to_string()],
            grant_types_supported: vec!["authorization_code".to_string()],
            subject_types_supported: vec!["public".to_string()],
            id_token_signing_alg_values_supported: vec!["EdDSA".to_string()],
            scopes_supported: vec!["openid".to_string()],
            token_endpoint_auth_methods_supported: vec!["client_secret_post".to_string()],
            claims_supported: vec!["sub".to_string()],
            code_challenge_methods_supported: vec!["S256".to_string()],
        };
        let json = serde_json::to_string(&discovery).unwrap();
        assert!(json.contains("issuer"));
        assert!(json.contains("authorization_endpoint"));
    }

    use authenc_types::domain::user::User;
    use uuid::Uuid;

    #[test]
    fn test_user_info_mapping() {
        let user_id = Uuid::new_v4();
        let user = User {
            id: user_id,
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            email_verified: true,
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            nip: Some("12345".to_string()),
            nama: Some("Test User Full".to_string()),
            jabatan: Some("Developer".to_string()),
            satker_code: "001".to_string(),
            phone_number: None,
            phone_verified: false,
            password_hash: None,
            totp_secret: None,
            totp_backup_codes: None,
            mfa_enabled: false,
            mfa_setup_at: None,
            mfa_last_used: None,
            webauthn_enabled: false,
            account_locked: false,
            account_locked_until: None,
            failed_login_attempts: 0,
            last_login_at: None,
            last_failed_login_at: None,
            password_changed_at: None,
            password_expires_at: None,
            require_password_change: false,
            realm_id: None,
            organization_id: None,
            roles: Vec::new(),
            permissions: Vec::new(),
            session_data: None,
            security_context: Default::default(),
            attributes: None,
            enabled: true,
            federated: false,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deleted_at: None,
            login_count: 0,
        };

        let name = match (&user.first_name, &user.last_name) {
            (Some(f), Some(l)) => Some(format!("{} {}", f, l)),
            (Some(f), None) => Some(f.clone()),
            (None, Some(l)) => Some(l.clone()),
            _ => user.nama.clone(),
        };

        let resp = UserInfoResponse {
            sub: user.id.to_string(),
            preferred_username: Some(user.username),
            email: Some(user.email),
            email_verified: Some(user.email_verified),
            name,
            given_name: user.first_name,
            family_name: user.last_name,
        };

        assert_eq!(resp.sub, user_id.to_string());
        assert_eq!(resp.preferred_username, Some("testuser".to_string()));
        assert_eq!(resp.name, Some("Test User".to_string()));
        assert_eq!(resp.given_name, Some("Test".to_string()));
        assert_eq!(resp.family_name, Some("User".to_string()));
    }
}
