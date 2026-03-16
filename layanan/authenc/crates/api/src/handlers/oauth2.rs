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
        status_code: axum::http::StatusCode::BAD_REQUEST,
        error: "not_implemented".to_string(),
        message: "OAuth2 authorize endpoint not yet implemented".to_string(),
    })
}

/// POST /api/v1/oauth2/token - Token endpoint
///
/// OAuth2 token endpoint for exchanging authorization codes for tokens.
/// Supports authorization_code, refresh_token, and client_credentials grants.
pub async fn token_handler(
    State(_state): State<Arc<ApiState>>,
    Json(_request): Json<TokenRequest>,
) -> Result<Json<TokenResponse>, ErrorResponse> {
    // TODO: Implement token exchange logic
    // 1. Validate grant_type
    // 2. For authorization_code:
    //    - Validate code, client_id, redirect_uri
    //    - Verify PKCE code_verifier if present
    //    - Generate access token and refresh token
    //    - Generate ID token if openid scope requested
    // 3. For refresh_token:
    //    - Validate refresh token
    //    - Generate new access token and refresh token
    // 4. For client_credentials:
    //    - Validate client credentials
    //    - Generate access token (no refresh token)
    // 5. Return token response

    Err(ErrorResponse {
        status_code: axum::http::StatusCode::BAD_REQUEST,
        error: "not_implemented".to_string(),
        message: "OAuth2 token endpoint not yet implemented".to_string(),
    })
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

/// GET /api/v1/oauth2/userinfo - UserInfo endpoint
///
/// OIDC UserInfo endpoint returning claims about the authenticated user.
/// Requires valid access token with openid scope.
pub async fn userinfo_handler(
    State(_state): State<Arc<ApiState>>,
    // TODO: Add JWT claims extractor from middleware
) -> Result<Json<UserInfoResponse>, ErrorResponse> {
    // TODO: Implement UserInfo logic
    // 1. Extract user ID from access token
    // 2. Validate token has openid scope
    // 3. Fetch user from database
    // 4. Return claims based on requested scopes

    Err(ErrorResponse {
        status_code: axum::http::StatusCode::BAD_REQUEST,
        error: "not_implemented".to_string(),
        message: "OIDC UserInfo endpoint not yet implemented".to_string(),
    })
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
}
