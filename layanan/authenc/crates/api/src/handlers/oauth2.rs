//! OAuth2/OIDC public endpoint handlers

use std::sync::Arc;

use authenc_types::OAuth2Service;
use axum::{
    Form, Json,
    extract::{Query, State},
    response::Response,
};
use serde::{Deserialize, Serialize};

use crate::{handlers::ErrorResponse, state::ApiState};

/// OAuth2 authorization request parameters
#[derive(Debug, Deserialize)]
pub struct AuthorizeRequest {
    /// Response type (only "code" is supported)
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
///
/// `client_id` and `client_secret` are optional in the form body because
/// they may be supplied via HTTP Basic authentication
/// (`client_secret_basic`) per RFC 6749 §2.3.1.
///
/// Deserialized from `application/x-www-form-urlencoded` per RFC 6749 §4.1.3.
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
    /// Client ID (optional in body; may come from Authorization header)
    #[serde(default)]
    pub client_id: Option<String>,
    /// Client secret (for confidential clients; may come from Authorization header)
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
    //
    // NOTE: This causes a double client lookup — `oauth2_service.authorize()`
    // performs the same lookup as defense-in-depth.  This is intentional for
    // security (RFC compliance), but could be optimized by passing the
    // pre-validated client into the service layer.  See the corresponding
    // TODO in `oauth2_service.rs`.
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

    // 2. Validate response_type BEFORE authentication.
    // Only "code" is supported (Authorization Code flow).  Per RFC 6749
    // §4.1.2.1, errors after redirect_uri validation SHOULD be redirected
    // to the client with the appropriate error code.
    //
    // Checking this before authentication avoids forcing the user through
    // the login flow only to receive an error redirect afterwards.
    if request.response_type != "code" {
        let separator = if request.redirect_uri.contains('?') {
            '&'
        } else {
            '?'
        };
        let mut target = format!(
            "{}{}error=unsupported_response_type&error_description={}",
            request.redirect_uri,
            separator,
            urlencoding::encode(&format!(
                "Unsupported response_type '{}', only 'code' is supported",
                request.response_type
            ))
        );
        if let Some(ref state_param) = request.state {
            target.push_str(&format!("&state={}", urlencoding::encode(state_param)));
        }
        return Ok(redirect_found(&target));
    }

    // 3. Check if user is authenticated via Bearer token
    let user_uuid = match auth_helpers::extract_user_from_token(&state, &headers).await {
        Ok(uid) => uid,
        Err(_) => {
            // If not authenticated, redirect to login page
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

    // 4. If authenticated, check consent (assume auto-consent for now)

    // 4.5. Validate PKCE parameters (required per OAuth 2.1)
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

/// Extract client credentials from the `Authorization: Basic` header.
///
/// Per RFC 6749 §2.3.1, the header value is `Basic base64(client_id:client_secret)`.
/// Returns `(client_id, client_secret)` on success.
fn extract_basic_auth(headers: &HeaderMap) -> Option<(String, String)> {
    use base64::{Engine, engine::general_purpose::STANDARD};

    let auth_header = headers.get("Authorization")?.to_str().ok()?;
    // Per RFC 7235 §2.1, the authentication scheme token is case-insensitive.
    let encoded = if auth_header.len() > 6 && auth_header[..6].eq_ignore_ascii_case("basic ") {
        &auth_header[6..]
    } else {
        return None;
    };
    let decoded = String::from_utf8(STANDARD.decode(encoded).ok()?).ok()?;
    let (id, secret) = decoded.split_once(':')?;
    Some((
        urlencoding::decode(id)
            .unwrap_or_else(|_| id.into())
            .into_owned(),
        urlencoding::decode(secret)
            .unwrap_or_else(|_| secret.into())
            .into_owned(),
    ))
}

/// POST /api/v1/oauth2/token - Token endpoint
///
/// OAuth2 token endpoint for exchanging authorization codes for tokens.
/// Supports authorization_code, refresh_token, and client_credentials grants.
///
/// Client credentials may be supplied via `client_secret_post` (in the
/// form-encoded body) or `client_secret_basic` (`Authorization: Basic`
/// header) per RFC 6749 §2.3.
///
/// Per RFC 6749 §4.1.3, the request body uses
/// `application/x-www-form-urlencoded` encoding.
pub async fn token_handler(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Form(request): Form<TokenRequest>,
) -> Result<Json<TokenResponse>, ErrorResponse> {
    use authenc_types::domain_types::{RealmId, TokenRequest as DomainTokenRequest};

    // Resolve client_id and client_secret.
    // Priority: body parameters > Authorization: Basic header.
    // Per RFC 6749 §2.3, a client MUST NOT use more than one method, but we
    // tolerate it by preferring the body values when both are present.
    let basic_auth = extract_basic_auth(&headers);
    let client_id = request
        .client_id
        .clone()
        .or_else(|| basic_auth.as_ref().map(|(id, _)| id.clone()))
        .ok_or_else(|| ErrorResponse {
            status_code: axum::http::StatusCode::BAD_REQUEST,
            error: "invalid_request".to_string(),
            message: "client_id is required (via request body or Authorization header)".to_string(),
        })?;
    let client_secret = request
        .client_secret
        .clone()
        .or_else(|| basic_auth.map(|(_, secret)| secret));

    // Default to master realm
    let realm_id = RealmId::from_uuid(authenc_types::domain::Realm::MASTER_ID);

    let grant_type = request.grant_type.clone();
    let domain_request = DomainTokenRequest {
        grant_type: request.grant_type,
        code: request.code,
        redirect_uri: request.redirect_uri,
        code_verifier: request.code_verifier,
        client_id: client_id.clone(),
        client_secret,
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
                    .map_err(|e| {
                        tracing::error!(
                            "ID token generation: failed to verify access token: {}",
                            e
                        );
                        ErrorResponse {
                            status_code: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            error: "server_error".to_string(),
                            message: "Failed to generate ID token".to_string(),
                        }
                    })?;

                let user_uuid = uuid::Uuid::parse_str(&claims.sub).map_err(|e| {
                    tracing::error!(
                        "ID token generation: invalid user ID in access token: {}",
                        e
                    );
                    ErrorResponse {
                        status_code: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        error: "server_error".to_string(),
                        message: "Failed to generate ID token".to_string(),
                    }
                })?;

                let user_id = authenc_types::UserId::from_uuid(user_uuid);
                let user = state.user_service.get_user(user_id).await.map_err(|e| {
                    tracing::error!("ID token generation: failed to fetch user: {}", e);
                    ErrorResponse {
                        status_code: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        error: "server_error".to_string(),
                        message: "Failed to generate ID token".to_string(),
                    }
                })?;

                id_token = Some(
                    super::oidc_jwt::generate_id_token(
                        &user,
                        &client_id,
                        resp.nonce.clone(),
                        state.jwt_service.issuer(),
                        state.jwt_service.signing_key(),
                    )
                    .map_err(|e| {
                        tracing::error!("ID token generation: signing failed: {}", e);
                        ErrorResponse {
                            status_code: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            error: "server_error".to_string(),
                            message: "Failed to generate ID token".to_string(),
                        }
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
                            // Return generic message for unknown OAuth2 errors
                            "An error occurred while processing the token request".to_string(),
                        )
                    }
                }
                _ => {
                    tracing::error!(error = %e, "OAuth2 token error (non-OAuth2)");
                    (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "server_error".to_string(),
                        "An internal error occurred while processing the token request".to_string(),
                    )
                }
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
    // First normalize possible trailing slashes, then strip the suffix.
    // This handles both "https://host/api/v1/auth" and
    // "https://host/api/v1/auth/" gracefully.
    //
    // If the issuer does not contain a scheme (e.g. "simpelv2-authenc"),
    // the derived endpoint URLs will be relative paths which are unlikely
    // to work.  Log a warning so operators can fix their JWT_ISSUER.
    let normalized = issuer.trim_end_matches('/');
    let base_url = normalized
        .strip_suffix("/api/v1/auth")
        .unwrap_or(normalized)
        .trim_end_matches('/');

    if !base_url.starts_with("http://") && !base_url.starts_with("https://") {
        tracing::warn!(
            "OIDC discovery: JWT_ISSUER '{}' does not look like a URL; \
             derived base_url '{}' may produce invalid endpoint URLs. \
             Set JWT_ISSUER to a full URL (e.g. 'https://authenc.example.com' \
             or 'https://host/api/v1/auth').",
            issuer,
            base_url,
        );
    }

    let discovery = OidcDiscoveryResponse {
        issuer: issuer.clone(),
        authorization_endpoint: format!("{}/api/v1/oauth2/authorize", base_url),
        token_endpoint: format!("{}/api/v1/oauth2/token", base_url),
        userinfo_endpoint: format!("{}/api/v1/oauth2/userinfo", base_url),
        jwks_uri: format!("{}/api/v1/oauth2/jwks", base_url),
        response_types_supported: vec!["code".to_string()],
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

    let claims = state.jwt_service.verify_token(&token).map_err(|e| {
        // Log the detailed error for debugging but return a generic
        // message to avoid leaking internal JWT details to callers.
        tracing::debug!("UserInfo: token verification failed: {}", e);
        ErrorResponse {
            status_code: axum::http::StatusCode::UNAUTHORIZED,
            error: "invalid_token".to_string(),
            message: "The access token is invalid or expired".to_string(),
        }
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
        Err(e) => {
            tracing::error!(error = %e, user_id = %user_uuid, "UserInfo: failed to fetch user");
            Err(ErrorResponse {
                status_code: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                error: "internal_error".to_string(),
                message: "Failed to retrieve user information".to_string(),
            })
        }
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

    #[test]
    fn test_extract_basic_auth_valid() {
        use base64::{Engine, engine::general_purpose::STANDARD};
        let mut headers = HeaderMap::new();
        let encoded = STANDARD.encode("my-client:my-secret");
        headers.insert(
            "Authorization",
            format!("Basic {}", encoded).parse().unwrap(),
        );
        let result = extract_basic_auth(&headers);
        assert_eq!(
            result,
            Some(("my-client".to_string(), "my-secret".to_string()))
        );
    }

    #[test]
    fn test_extract_basic_auth_url_encoded() {
        use base64::{Engine, engine::general_purpose::STANDARD};
        let mut headers = HeaderMap::new();
        // client_id contains special chars that are percent-encoded per RFC 6749 §2.3.1
        let encoded = STANDARD.encode("client%3Aid:secret%3Aval");
        headers.insert(
            "Authorization",
            format!("Basic {}", encoded).parse().unwrap(),
        );
        let result = extract_basic_auth(&headers);
        assert_eq!(
            result,
            Some(("client:id".to_string(), "secret:val".to_string()))
        );
    }

    #[test]
    fn test_extract_basic_auth_missing() {
        let headers = HeaderMap::new();
        assert_eq!(extract_basic_auth(&headers), None);
    }

    #[test]
    fn test_extract_basic_auth_bearer_ignored() {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "Bearer some-token".parse().unwrap());
        assert_eq!(extract_basic_auth(&headers), None);
    }

    #[test]
    fn test_extract_basic_auth_case_insensitive() {
        use base64::{Engine, engine::general_purpose::STANDARD};
        let encoded = STANDARD.encode("my-client:my-secret");

        // "BASIC " (uppercase)
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("BASIC {}", encoded).parse().unwrap(),
        );
        assert_eq!(
            extract_basic_auth(&headers),
            Some(("my-client".to_string(), "my-secret".to_string()))
        );

        // "basic " (lowercase)
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("basic {}", encoded).parse().unwrap(),
        );
        assert_eq!(
            extract_basic_auth(&headers),
            Some(("my-client".to_string(), "my-secret".to_string()))
        );
    }

    #[test]
    fn test_token_request_without_client_id() {
        // client_id omitted — should deserialize with client_id = None
        let json = r#"{
            "grant_type": "authorization_code",
            "code": "abc123"
        }"#;
        let request: TokenRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.grant_type, "authorization_code");
        assert!(request.client_id.is_none());
    }

    #[test]
    fn test_token_request_with_client_id() {
        let json = r#"{
            "grant_type": "authorization_code",
            "client_id": "portal-client",
            "code": "abc123"
        }"#;
        let request: TokenRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.client_id, Some("portal-client".to_string()));
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
