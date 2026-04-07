//! OAuth2/OIDC public endpoint handlers

use std::sync::Arc;

use authenc_types::{ClientStore, OAuth2Service};
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
use axum::response::IntoResponse;

/// Build an HTTP 302 (Found) redirect response.
///
/// RFC 6749 §4.1.2 mandates 302 for the authorization endpoint.  Axum's
/// `Redirect::temporary()` returns 307, which preserves the HTTP method and
/// may confuse strict OAuth2 client libraries.
fn redirect_found(uri: &str) -> Response {
    (
        axum::http::StatusCode::FOUND,
        [(axum::http::header::LOCATION, uri)],
    )
        .into_response()
}

pub async fn authorize_handler(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Query(request): Query<AuthorizeRequest>,
) -> Result<Response, ErrorResponse> {
    use authenc_types::domain::Realm;
    use authenc_types::domain_types::{
        AuthorizationRequest, AuthorizationResponse, RealmId, UserId,
    };

    // 1. Basic validation of required parameters
    if request.client_id.is_empty() || request.redirect_uri.is_empty() {
        return Err(ErrorResponse {
            status_code: axum::http::StatusCode::BAD_REQUEST,
            error: "invalid_request".to_string(),
            message: "client_id and redirect_uri are required".to_string(),
        });
    }

    // 1.5. Pre-validate client_id and redirect_uri BEFORE any redirect.
    // Per RFC 6749 §4.1.2.1, if the redirect_uri is invalid/unregistered or
    // the client_id is unrecognized, the authorization server MUST NOT
    // redirect the user-agent to the invalid URI.  We therefore look up the
    // client and check the redirect_uri here; failures are returned as plain
    // HTTP errors (not redirects).
    let realm_id_for_lookup =
        authenc_types::domain_types::RealmId::from_uuid(authenc_types::domain::Realm::MASTER_ID);
    let client = state
        .oauth2_service
        .client_store()
        .get_client_by_client_id(&request.client_id, realm_id_for_lookup)
        .await
        .map_err(|_| ErrorResponse {
            status_code: axum::http::StatusCode::BAD_REQUEST,
            error: "invalid_request".to_string(),
            message: "Unknown client_id".to_string(),
        })?;

    if !client.enabled {
        return Err(ErrorResponse {
            status_code: axum::http::StatusCode::BAD_REQUEST,
            error: "invalid_request".to_string(),
            message: "Client is disabled".to_string(),
        });
    }

    state
        .oauth2_service
        .validate_redirect_uri(&client, &request.redirect_uri)
        .map_err(|_| ErrorResponse {
            status_code: axum::http::StatusCode::BAD_REQUEST,
            error: "invalid_request".to_string(),
            message: "Invalid redirect_uri for this client".to_string(),
        })?;

    // 2. Check if user is authenticated via session cookie
    let user_uuid = match auth_helpers::extract_user_from_token(&state, &headers).await {
        Ok(uid) => uid,
        Err(_) => {
            // 4. If not authenticated, redirect to login page
            // Build return URL to this same endpoint after login
            let mut return_url = format!(
                "/api/v1/oauth2/authorize?client_id={}&redirect_uri={}&scope={}&response_type={}",
                urlencoding::encode(&request.client_id),
                urlencoding::encode(&request.redirect_uri),
                urlencoding::encode(&request.scope),
                urlencoding::encode(&request.response_type)
            );

            if let Some(state) = &request.state {
                return_url.push_str(&format!("&state={}", urlencoding::encode(state)));
            }
            if let Some(cc) = &request.code_challenge {
                return_url.push_str(&format!("&code_challenge={}", urlencoding::encode(cc)));
            }
            if let Some(ccm) = &request.code_challenge_method {
                return_url.push_str(&format!(
                    "&code_challenge_method={}",
                    urlencoding::encode(ccm)
                ));
            }
            if let Some(nonce) = &request.nonce {
                return_url.push_str(&format!("&nonce={}", urlencoding::encode(nonce)));
            }

            let login_redirect = format!("/login?return_to={}", urlencoding::encode(&return_url));
            return Ok(redirect_found(&login_redirect));
        }
    };

    // 5. If authenticated, check consent (assume auto-consent for now)

    // 5.5. Validate PKCE parameters (required per OAuth 2.1)
    // Since the redirect_uri has already been validated (step 1.5), errors
    // from this point onward SHOULD be communicated by redirecting to the
    // client's redirect_uri per RFC 6749 §4.1.2.1.
    let code_challenge = match request.code_challenge {
        Some(cc) if !cc.is_empty() => cc,
        _ => {
            let separator = if request.redirect_uri.contains('?') {
                '&'
            } else {
                '?'
            };
            let mut target = format!(
                "{}{}error=invalid_request&error_description={}",
                request.redirect_uri,
                separator,
                urlencoding::encode("code_challenge is required (OAuth 2.1)")
            );
            if let Some(ref state_param) = request.state {
                target.push_str(&format!("&state={}", urlencoding::encode(state_param)));
            }
            return Ok(redirect_found(&target));
        }
    };
    let code_challenge_method = match request.code_challenge_method {
        Some(ccm) if ccm == "S256" || ccm == "plain" => ccm,
        Some(ccm) => {
            // Unrecognized method — redirect error to client
            let separator = if request.redirect_uri.contains('?') {
                '&'
            } else {
                '?'
            };
            let mut target = format!(
                "{}{}error=invalid_request&error_description={}",
                request.redirect_uri,
                separator,
                urlencoding::encode(&format!(
                    "Unsupported code_challenge_method '{}', must be S256 or plain",
                    ccm
                ))
            );
            if let Some(ref state_param) = request.state {
                target.push_str(&format!("&state={}", urlencoding::encode(state_param)));
            }
            return Ok(redirect_found(&target));
        }
        // Per RFC 7636 §4.3, the default when code_challenge_method is absent
        // is "plain".
        None => "plain".to_string(),
    };

    // 6. Generate and persist authorization code via OAuth2 service
    // Save state and redirect_uri before they are moved into domain_request,
    // because we need them for both the success and error redirect paths.
    let redirect_uri = request.redirect_uri.clone();
    let client_state = request.state.clone();

    let domain_request = AuthorizationRequest {
        response_type: request.response_type,
        client_id: request.client_id,
        redirect_uri: request.redirect_uri,
        scope: request.scope,
        state: request.state,
        code_challenge,
        code_challenge_method,
        user_id: UserId::from_uuid(user_uuid),
        realm_id: RealmId::from_uuid(Realm::MASTER_ID),
        nonce: request.nonce,
    };

    match state.oauth2_service.authorize(domain_request).await {
        Ok(resp) => {
            let resp: AuthorizationResponse = resp;
            // 7. Redirect back to client
            // Use '&' if redirect_uri already contains a query string, '?' otherwise
            let separator = if redirect_uri.contains('?') { '&' } else { '?' };
            let mut target = format!(
                "{}{}code={}",
                redirect_uri,
                separator,
                urlencoding::encode(&resp.code)
            );
            if let Some(ref state_param) = resp.state {
                target.push_str(&format!("&state={}", urlencoding::encode(state_param)));
            }
            Ok(redirect_found(&target))
        }
        Err(e) => {
            // Per RFC 6749 §4.1.2.1, errors (other than invalid redirect_uri
            // / unknown client_id) SHOULD be communicated by redirecting to the
            // client's redirect_uri.  The redirect_uri was already validated
            // against the registered client above (step 1.5), so it is safe to
            // redirect here.
            //
            // Use a generic error description to avoid leaking internal
            // details (e.g. database errors, stack traces) to the client.
            let error_code = "server_error";
            let error_description =
                urlencoding::encode("Authorization request could not be processed");
            tracing::warn!("OAuth2 authorize error (redirecting to client): {}", e);
            let separator = if redirect_uri.contains('?') { '&' } else { '?' };
            let mut target = format!(
                "{}{}error={}&error_description={}",
                redirect_uri, separator, error_code, error_description
            );
            if let Some(ref state_param) = client_state {
                target.push_str(&format!("&state={}", urlencoding::encode(state_param)));
            }
            Ok(redirect_found(&target))
        }
    }
}

/// POST /api/v1/oauth2/token - Token endpoint
///
/// OAuth2 token endpoint for exchanging authorization codes for tokens.
/// Supports authorization_code, refresh_token, and client_credentials grants.
pub async fn token_handler(
    State(state): State<Arc<ApiState>>,
    Json(request): Json<TokenRequest>,
) -> Result<Json<TokenResponse>, ErrorResponse> {
    use authenc_types::domain_types::{RealmId, TokenRequest as DomainTokenRequest};
    use uuid::Uuid;

    // Default to master realm
    let realm_id = RealmId::from_uuid(authenc_types::domain::Realm::MASTER_ID);

    let client_id = request.client_id.clone();
    let grant_type = request.grant_type.clone();
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
        Ok(resp) => {
            let mut id_token = None;

            // Generate ID token if 'openid' scope was granted.
            // Per OIDC Core §3.1.3.3, the ID token MUST be present when
            // openid scope is granted — errors must not be silently dropped.
            //
            // However, client_credentials grants have no end-user context
            // (the `sub` is a synthetic service-account UUID that does not
            // exist in the user store), so attempting to generate an ID
            // token would always fail.  OIDC Core does not define ID token
            // semantics for client_credentials, so we skip generation here.
            let is_client_credentials = grant_type == "client_credentials";
            if !is_client_credentials && resp.scope.split_whitespace().any(|s| s == "openid") {
                let claims = state
                    .jwt_service
                    .verify_token(&resp.access_token)
                    .map_err(|e| ErrorResponse {
                        status_code: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        error: "server_error".to_string(),
                        message: format!(
                            "Failed to verify access token for ID token generation: {}",
                            e
                        ),
                    })?;

                let user_uuid = uuid::Uuid::parse_str(&claims.sub).map_err(|e| ErrorResponse {
                    status_code: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    error: "server_error".to_string(),
                    message: format!("Invalid user ID in access token: {}", e),
                })?;

                let user_id = authenc_types::UserId::from_uuid(user_uuid);
                let user =
                    state
                        .user_service
                        .get_user(user_id)
                        .await
                        .map_err(|e| ErrorResponse {
                            status_code: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            error: "server_error".to_string(),
                            message: format!("Failed to fetch user for ID token generation: {}", e),
                        })?;

                id_token = Some(
                    super::oidc_jwt::generate_id_token(
                        &user,
                        &client_id,
                        resp.nonce.clone(),
                        state.jwt_service.issuer(),
                        state.jwt_service.signing_key(),
                    )
                    .map_err(|e| ErrorResponse {
                        status_code: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        error: "server_error".to_string(),
                        message: format!("Failed to generate ID token: {}", e),
                    })?,
                );
            }

            Ok(Json(TokenResponse {
                access_token: resp.access_token,
                token_type: resp.token_type,
                expires_in: resp.expires_in.max(0) as u64,
                refresh_token: resp.refresh_token,
                id_token,
                scope: Some(resp.scope),
            }))
        }
        Err(e) => {
            // Map domain OAuth2 errors to RFC 6749 §5.2 compliant responses.
            // AuthencError::OAuth2Error(inner) carries "{error_code}: {description}"
            // (see From<OAuth2Error> for AuthencError in error.rs).
            let error_string = e.to_string();
            let (status_code, error_code, message) = match &e {
                authenc_types::error::AuthencError::OAuth2Error(msg) => {
                    let inner = msg.as_str();
                    if let Some((code, desc)) = inner.split_once(": ") {
                        let status = if code == "invalid_client" {
                            axum::http::StatusCode::UNAUTHORIZED
                        } else {
                            axum::http::StatusCode::BAD_REQUEST
                        };
                        (status, code.to_string(), desc.to_string())
                    } else {
                        (
                            axum::http::StatusCode::BAD_REQUEST,
                            inner.to_string(),
                            error_string,
                        )
                    }
                }
                _ => (
                    axum::http::StatusCode::BAD_REQUEST,
                    "invalid_request".to_string(),
                    error_string,
                ),
            };
            Err(ErrorResponse {
                status_code,
                error: error_code,
                message,
            })
        }
    }
}

/// GET /api/v1/oauth2/.well-known/openid-configuration - Discovery endpoint
///
/// OIDC discovery endpoint returning server metadata.
pub async fn discovery_handler(
    State(state): State<Arc<ApiState>>,
) -> Result<Json<OidcDiscoveryResponse>, ErrorResponse> {
    let issuer = state.jwt_service.issuer().to_string();
    // Derive base URL by stripping the known auth path suffix.
    // Uses strip_suffix (single match) instead of trim_end_matches (repeated)
    // to avoid accidentally stripping more than intended.
    let base_url = issuer
        .strip_suffix("/api/v1/auth")
        .unwrap_or(&issuer)
        .trim_end_matches('/');

    let discovery = OidcDiscoveryResponse {
        issuer: issuer.clone(),
        authorization_endpoint: format!("{}/api/v1/oauth2/authorize", base_url),
        token_endpoint: format!("{}/api/v1/oauth2/token", base_url),
        userinfo_endpoint: format!("{}/api/v1/oauth2/userinfo", base_url),
        jwks_uri: format!("{}/api/v1/oauth2/jwks", base_url),
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
/// Requires valid access token with openid scope (OIDC Core §5.3).
pub async fn userinfo_handler(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<UserInfoResponse>, ErrorResponse> {
    // Extract and verify bearer token
    let token = auth_helpers::extract_bearer_token(&headers).map_err(|e| ErrorResponse {
        status_code: axum::http::StatusCode::UNAUTHORIZED,
        error: "unauthorized".to_string(),
        message: e.message,
    })?;

    let claims = state
        .jwt_service
        .verify_token(&token)
        .map_err(|e| ErrorResponse {
            status_code: axum::http::StatusCode::UNAUTHORIZED,
            error: "unauthorized".to_string(),
            message: format!("Invalid or expired token: {}", e),
        })?;

    // Validate openid scope per OIDC Core §5.3
    let has_openid = claims
        .scope
        .as_ref()
        .map(|s| s.split_whitespace().any(|t| t == "openid"))
        .unwrap_or(false);
    if !has_openid {
        return Err(ErrorResponse {
            status_code: axum::http::StatusCode::FORBIDDEN,
            error: "insufficient_scope".to_string(),
            message: "Access token must have 'openid' scope to access UserInfo endpoint"
                .to_string(),
        });
    }

    let user_uuid = uuid::Uuid::parse_str(&claims.sub).map_err(|_| ErrorResponse {
        status_code: axum::http::StatusCode::UNAUTHORIZED,
        error: "unauthorized".to_string(),
        message: "Invalid user ID in token".to_string(),
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
