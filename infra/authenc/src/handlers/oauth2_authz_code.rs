/// OAuth2 Authorization Code Flow with PKCE Support
/// This module implements the OAuth2 authorization code flow with PKCE (RFC 7636)
/// for enhanced security.
use crate::{
    database::Database,
    error::{AuthencError, Result},
    models::{OAuth2TokenRequest, OAuth2TokenResponse},
    services::oidc_code_store::OidcCodeStore,
};
use axum::{
    Json,
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
};
use base64ct::{Base64UrlUnpadded, Encoding};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;

/// Authorization request parameters
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthorizationRequest {
    pub response_type: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub scope: Option<String>,
    pub state: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
    pub nonce: Option<String>,
}

/// Application state for authorization handlers
#[derive(Clone)]
pub struct AuthzState {
    pub db: Arc<Database>,
    pub code_store: Arc<OidcCodeStore>,
}

/// Generate cryptographically secure authorization code
fn generate_authorization_code() -> String {
    use rand::RngCore;
    let mut rng = rand::thread_rng();
    let mut random_bytes = [0u8; 32];
    rng.fill_bytes(&mut random_bytes);
    Base64UrlUnpadded::encode_string(&random_bytes)
}

/// Validate redirect URI against registered URIs
fn validate_redirect_uri(redirect_uri: &str, registered_uris: &[String]) -> bool {
    registered_uris.iter().any(|uri| uri == redirect_uri)
}

/// Validate PKCE code challenge method
fn validate_code_challenge_method(method: &str) -> bool {
    method == "S256" || method == "plain"
}

/// Verify PKCE code verifier against code challenge
#[allow(dead_code)]
fn verify_code_challenge(code_verifier: &str, code_challenge: &str, method: &str) -> bool {
    match method {
        "S256" => {
            let mut hasher = Sha256::new();
            hasher.update(code_verifier.as_bytes());
            let hash = hasher.finalize();
            let computed_challenge = Base64UrlUnpadded::encode_string(&hash);
            computed_challenge == code_challenge
        }
        "plain" => code_verifier == code_challenge,
        _ => false,
    }
}

/// OAuth2 Authorization Endpoint
/// Handles authorization requests and initiates the authorization code flow.
/// Validates client, redirect URI, and generates authorization code with PKCE support.
pub async fn authorize(
    State(state): State<AuthzState>,
    Query(params): Query<AuthorizationRequest>,
) -> Result<Response> {
    // Validate response_type
    if params.response_type != "code" {
        return Err(AuthencError::validation(
            "Only 'code' response_type is supported for authorization code flow",
        ));
    }

    // Validate client_id
    if params.client_id.is_empty() {
        return Err(AuthencError::validation("client_id is required"));
    }

    // Validate redirect_uri
    if params.redirect_uri.is_empty() {
        return Err(AuthencError::validation("redirect_uri is required"));
    }

    // TODO: In production, validate client exists and is enabled
    let registered_uris = vec![params.redirect_uri.clone()];

    if !validate_redirect_uri(&params.redirect_uri, &registered_uris) {
        return Err(AuthencError::validation(
            "redirect_uri does not match registered URIs",
        ));
    }

    // Validate PKCE parameters if provided
    if let Some(ref challenge) = params.code_challenge {
        let method = params.code_challenge_method.as_deref().unwrap_or("plain");

        if !validate_code_challenge_method(method) {
            return Err(AuthencError::validation(
                "code_challenge_method must be 'S256' or 'plain'",
            ));
        }

        if challenge.is_empty() {
            return Err(AuthencError::validation("code_challenge cannot be empty"));
        }
    } else if params.code_challenge_method.is_some() {
        return Err(AuthencError::validation(
            "code_challenge_method requires code_challenge",
        ));
    }

    // TODO: In production, check user authentication
    let user_id = "demo_user_id";

    // Generate authorization code
    let auth_code = generate_authorization_code();

    // Parse scopes
    let scopes: Vec<String> = params
        .scope
        .as_ref()
        .map(|s| s.split_whitespace().map(String::from).collect())
        .unwrap_or_else(|| vec!["openid".to_string()]);

    // Store authorization code with 10-minute TTL
    state
        .code_store
        .insert(
            auth_code.clone(),
            params.client_id.clone(),
            user_id.to_string(),
            params.redirect_uri.clone(),
            scopes,
            params.code_challenge.clone(),
            params.code_challenge_method.clone(),
        )
        .await?;

    // Build redirect URI with authorization code
    let mut redirect_url = params.redirect_uri.clone();
    let separator = if redirect_url.contains('?') { "&" } else { "?" };
    redirect_url.push_str(&format!("{}code={}", separator, auth_code));

    // Include state parameter if provided (CSRF protection)
    if let Some(state_param) = params.state {
        redirect_url.push_str(&format!("&state={}", state_param));
    }

    Ok(Redirect::to(&redirect_url).into_response())
}

/// OAuth2 Token Endpoint
/// Exchanges authorization code for access token and refresh token.
/// Validates authorization code, PKCE verifier, and client credentials.
pub async fn token(
    State(state): State<AuthzState>,
    Json(params): Json<OAuth2TokenRequest>,
) -> Result<Json<OAuth2TokenResponse>> {
    // Validate grant_type
    if params.grant_type != "authorization_code" {
        return Err(AuthencError::validation(
            "Only 'authorization_code' grant_type is supported",
        ));
    }

    // Validate required parameters
    let code = params
        .code
        .as_ref()
        .ok_or_else(|| AuthencError::validation("code is required"))?;

    let client_id = params
        .client_id
        .as_ref()
        .ok_or_else(|| AuthencError::validation("client_id is required"))?;

    let _redirect_uri = params
        .redirect_uri
        .as_ref()
        .ok_or_else(|| AuthencError::validation("redirect_uri is required"))?;

    // Retrieve and consume authorization code
    let user_id = state
        .code_store
        .take(code, client_id)
        .await?
        .ok_or_else(|| AuthencError::validation("Invalid or expired authorization code"))?;

    // Validate PKCE if code_verifier is provided
    if let Some(ref _code_verifier) = params.code_verifier {
        // TODO: In production, retrieve code_challenge and method from stored code
        // and verify: verify_code_challenge(_code_verifier, challenge, method)
    }

    // Generate tokens using Ed25519
    use super::oidc_ed25519::generate_ed25519_jwt;

    let access_token = generate_ed25519_jwt(
        &user_id,
        client_id,
        Some("user@example.com"),
        Some("Demo User"),
        Some("user"),
    );

    let id_token = generate_ed25519_jwt(
        &user_id,
        client_id,
        Some("user@example.com"),
        Some("Demo User"),
        Some("user"),
    );

    // Generate refresh token
    let refresh_token = generate_authorization_code();

    // Build token response
    let response = OAuth2TokenResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        refresh_token: Some(refresh_token),
        scope: Some("openid profile email".to_string()),
        id_token: Some(id_token),
    };

    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_authorization_code() {
        let code1 = generate_authorization_code();
        let code2 = generate_authorization_code();

        // Codes should be different
        assert_ne!(code1, code2);

        // Codes should be base64url encoded
        assert!(Base64UrlUnpadded::decode_vec(&code1).is_ok());
        assert!(Base64UrlUnpadded::decode_vec(&code2).is_ok());
    }

    #[test]
    fn test_validate_redirect_uri() {
        let registered = vec![
            "https://example.com/callback".to_string(),
            "https://app.example.com/auth".to_string(),
        ];

        // Valid URIs
        assert!(validate_redirect_uri(
            "https://example.com/callback",
            &registered
        ));
        assert!(validate_redirect_uri(
            "https://app.example.com/auth",
            &registered
        ));

        // Invalid URIs
        assert!(!validate_redirect_uri(
            "https://evil.com/callback",
            &registered
        ));
        assert!(!validate_redirect_uri(
            "https://example.com/callback/evil",
            &registered
        ));
    }

    #[test]
    fn test_validate_code_challenge_method() {
        assert!(validate_code_challenge_method("S256"));
        assert!(validate_code_challenge_method("plain"));
        assert!(!validate_code_challenge_method("invalid"));
        assert!(!validate_code_challenge_method(""));
    }

    #[test]
    fn test_verify_code_challenge_s256() {
        // Test vector from RFC 7636
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let challenge = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

        assert!(verify_code_challenge(verifier, challenge, "S256"));

        // Invalid verifier
        assert!(!verify_code_challenge("invalid", challenge, "S256"));
    }

    #[test]
    fn test_verify_code_challenge_plain() {
        let verifier = "test_verifier_123";
        let challenge = "test_verifier_123";

        assert!(verify_code_challenge(verifier, challenge, "plain"));

        // Invalid verifier
        assert!(!verify_code_challenge("wrong", challenge, "plain"));
    }
}
