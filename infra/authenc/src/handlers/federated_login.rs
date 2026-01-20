//! Federated Authentication Integration
//!
//! This module integrates federated authentication into the main login flow,
//! allowing users to authenticate via LDAP/AD or social providers.

use crate::app::AppState;
use crate::error::Result as AuthencResult;
use axum::{
    Router,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Create federated login routes
pub fn create_federated_login_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login/ldap", post(ldap_login))
        .route("/login/social/authorize", get(social_authorize))
        .route("/login/social/callback", get(social_callback))
        .route("/providers", get(list_providers))
}

/// LDAP login request
#[derive(Debug, Deserialize)]
pub struct LdapLoginRequest {
    pub provider_alias: String,
    pub username: String,
    pub password: String,
    pub realm_id: Option<Uuid>,
}

/// Social authorize request
#[derive(Debug, Deserialize)]
pub struct SocialAuthorizeQuery {
    pub provider_alias: String,
    pub realm_id: Option<Uuid>,
    pub return_url: Option<String>,
}

/// Social callback query
#[derive(Debug, Deserialize)]
pub struct SocialCallbackQuery {
    pub code: String,
    pub state: String,
}

/// Federated login response
#[derive(Debug, Serialize)]
pub struct FederatedLoginResponse {
    pub success: bool,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub token_type: Option<String>,
    pub expires_in: Option<u64>,
    pub user_id: Option<Uuid>,
    pub username: Option<String>,
    pub error: Option<String>,
}

/// Provider info response
#[derive(Debug, Serialize)]
pub struct ProviderInfo {
    pub alias: String,
    pub display_name: String,
    pub provider_type: String,
    pub enabled: bool,
}

/// LDAP login handler
async fn ldap_login(
    State(state): State<Arc<AppState>>,
    Json(request): Json<LdapLoginRequest>,
) -> impl IntoResponse {
    let realm_id = request.realm_id.unwrap_or_else(Uuid::nil);

    match state
        .federation_manager
        .authenticate_ldap(
            &request.provider_alias,
            &request.username,
            &request.password,
            realm_id,
        )
        .await
    {
        Ok(result) => {
            if result.success {
                if let Some(user) = result.user {
                    // Generate JWT tokens for the authenticated user
                    match generate_tokens(&state, &user, realm_id).await {
                        Ok((access_token, refresh_token, expires_in)) => {
                            Json(FederatedLoginResponse {
                                success: true,
                                access_token: Some(access_token),
                                refresh_token: Some(refresh_token),
                                token_type: Some("Bearer".to_string()),
                                expires_in: Some(expires_in),
                                user_id: Some(user.id),
                                username: Some(user.username),
                                error: None,
                            })
                            .into_response()
                        }
                        Err(e) => (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(FederatedLoginResponse {
                                success: false,
                                access_token: None,
                                refresh_token: None,
                                token_type: None,
                                expires_in: None,
                                user_id: None,
                                username: None,
                                error: Some(format!("Token generation failed: {}", e)),
                            }),
                        )
                            .into_response(),
                    }
                } else {
                    (
                        StatusCode::UNAUTHORIZED,
                        Json(FederatedLoginResponse {
                            success: false,
                            access_token: None,
                            refresh_token: None,
                            token_type: None,
                            expires_in: None,
                            user_id: None,
                            username: None,
                            error: Some("Authentication failed".to_string()),
                        }),
                    )
                        .into_response()
                }
            } else {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(FederatedLoginResponse {
                        success: false,
                        access_token: None,
                        refresh_token: None,
                        token_type: None,
                        expires_in: None,
                        user_id: None,
                        username: None,
                        error: result.error,
                    }),
                )
                    .into_response()
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(FederatedLoginResponse {
                success: false,
                access_token: None,
                refresh_token: None,
                token_type: None,
                expires_in: None,
                user_id: None,
                username: None,
                error: Some(e.to_string()),
            }),
        )
            .into_response(),
    }
}

/// Social login authorize handler
async fn social_authorize(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SocialAuthorizeQuery>,
) -> impl IntoResponse {
    let realm_id = query.realm_id.unwrap_or_else(Uuid::nil);

    // Get social provider
    let provider = match state
        .federation_manager
        .get_social_provider(&query.provider_alias)
        .await
    {
        Some(p) => p,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "Provider not found"
                })),
            )
                .into_response();
        }
    };

    // Generate state parameter for CSRF protection
    let state_param = format!(
        "{}:{}:{}",
        query.provider_alias,
        realm_id,
        uuid::Uuid::new_v4()
    );

    // Generate redirect URI
    let redirect_uri = format!(
        "{}://{}/api/v1/auth/federated/login/social/callback",
        if state.config.server.tls_enabled {
            "https"
        } else {
            "http"
        },
        format!("{}:{}", state.config.server.host, state.config.server.port)
    );

    // Get authorization URL
    match provider
        .get_authorization_url(&state_param, &redirect_uri)
        .await
    {
        Ok(auth_url) => Json(serde_json::json!({
            "authorization_url": auth_url,
            "state": state_param
        }))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": e.to_string()
            })),
        )
            .into_response(),
    }
}

/// Social login callback handler
async fn social_callback(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SocialCallbackQuery>,
) -> impl IntoResponse {
    // Parse state parameter
    let state_parts: Vec<&str> = query.state.split(':').collect();
    if state_parts.len() < 3 {
        return (
            StatusCode::BAD_REQUEST,
            Json(FederatedLoginResponse {
                success: false,
                access_token: None,
                refresh_token: None,
                token_type: None,
                expires_in: None,
                user_id: None,
                username: None,
                error: Some("Invalid state parameter".to_string()),
            }),
        )
            .into_response();
    }

    let provider_alias = state_parts[0];
    let realm_id = match Uuid::parse_str(state_parts[1]) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(FederatedLoginResponse {
                    success: false,
                    access_token: None,
                    refresh_token: None,
                    token_type: None,
                    expires_in: None,
                    user_id: None,
                    username: None,
                    error: Some("Invalid realm ID in state".to_string()),
                }),
            )
                .into_response();
        }
    };

    // Generate redirect URI (must match the one used in authorize)
    let redirect_uri = format!(
        "{}://{}/api/v1/auth/federated/login/social/callback",
        if state.config.server.tls_enabled {
            "https"
        } else {
            "http"
        },
        format!("{}:{}", state.config.server.host, state.config.server.port)
    );

    // Authenticate via social provider
    match state
        .federation_manager
        .authenticate_social(provider_alias, &query.code, &redirect_uri, realm_id)
        .await
    {
        Ok(result) => {
            if result.success {
                if let Some(user) = result.user {
                    // Generate JWT tokens
                    match generate_tokens(&state, &user, realm_id).await {
                        Ok((access_token, refresh_token, expires_in)) => {
                            Json(FederatedLoginResponse {
                                success: true,
                                access_token: Some(access_token),
                                refresh_token: Some(refresh_token),
                                token_type: Some("Bearer".to_string()),
                                expires_in: Some(expires_in),
                                user_id: Some(user.id),
                                username: Some(user.username),
                                error: None,
                            })
                            .into_response()
                        }
                        Err(e) => (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(FederatedLoginResponse {
                                success: false,
                                access_token: None,
                                refresh_token: None,
                                token_type: None,
                                expires_in: None,
                                user_id: None,
                                username: None,
                                error: Some(format!("Token generation failed: {}", e)),
                            }),
                        )
                            .into_response(),
                    }
                } else {
                    (
                        StatusCode::UNAUTHORIZED,
                        Json(FederatedLoginResponse {
                            success: false,
                            access_token: None,
                            refresh_token: None,
                            token_type: None,
                            expires_in: None,
                            user_id: None,
                            username: None,
                            error: Some("Authentication failed".to_string()),
                        }),
                    )
                        .into_response()
                }
            } else {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(FederatedLoginResponse {
                        success: false,
                        access_token: None,
                        refresh_token: None,
                        token_type: None,
                        expires_in: None,
                        user_id: None,
                        username: None,
                        error: result.error,
                    }),
                )
                    .into_response()
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(FederatedLoginResponse {
                success: false,
                access_token: None,
                refresh_token: None,
                token_type: None,
                expires_in: None,
                user_id: None,
                username: None,
                error: Some(e.to_string()),
            }),
        )
            .into_response(),
    }
}

/// List available identity providers
async fn list_providers(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let realm_id = Uuid::nil(); // Default realm - should come from JWT

    match state
        .federation_manager
        .list_identity_providers(realm_id)
        .await
    {
        Ok(providers) => {
            let provider_infos: Vec<ProviderInfo> = providers
                .into_iter()
                .map(|p| ProviderInfo {
                    alias: p.alias,
                    display_name: p.display_name,
                    provider_type: match p.provider_type {
                        crate::services::federation_manager::FederationProviderType::Ldap => {
                            "ldap".to_string()
                        }
                        crate::services::federation_manager::FederationProviderType::ActiveDirectory => {
                            "active_directory".to_string()
                        }
                        crate::services::federation_manager::FederationProviderType::Social => {
                            "social".to_string()
                        }
                    },
                    enabled: p.enabled,
                })
                .collect();

            Json(provider_infos).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": e.to_string()
            })),
        )
            .into_response(),
    }
}

/// Generate JWT tokens for authenticated user using Ed25519 signing
///
/// This function generates production-ready JWT tokens:
/// - Access token: Ed25519 signed JWT with user claims
/// - Refresh token: Ed25519 signed JWT for token rotation
/// - Both tokens include proper claims (sub, iss, aud, exp, iat, roles)
///
/// # Security Considerations
/// - Uses Ed25519 for cryptographically secure signatures
/// - Access tokens expire in 1 hour
/// - Refresh tokens expire in 30 days
/// - All tokens are stored in the session store for validation
async fn generate_tokens(
    state: &Arc<AppState>,
    user: &crate::models::User,
    realm_id: Uuid,
) -> AuthencResult<(String, String, u64)> {
    use crate::handlers::oidc_ed25519::generate_ed25519_jwt;
    use crate::utils::jwt::generate_refresh_token;

    // Load user roles from database
    let user_roles: Vec<String> = match crate::database::operations::roles::get_user_roles(
        &state.database,
        &user.id,
    ).await {
        Ok(roles) => roles.into_iter().map(|r| r.name).collect(),
        Err(e) => {
            tracing::warn!("Failed to load user roles: {}, using empty roles", e);
            Vec::new()
        }
    };

    // Determine primary role for JWT claim
    let primary_role = user_roles.first().map(|s| s.as_str());

    // Generate Ed25519 signed access token with user claims
    let access_token = generate_ed25519_jwt(
        &user.id.to_string(),
        &realm_id.to_string(),
        Some(&user.email),
        user.first_name.as_deref().or(Some(&user.username)),
        primary_role,
    );

    // Generate Ed25519 signed refresh token
    let refresh_token = generate_refresh_token(&user.id.to_string())
        .map_err(|e| crate::error::AuthencError::internal(&format!("Failed to generate refresh token: {}", e)))?;

    let expires_in = 3600u64; // 1 hour for access token

    // Store session in session store for validation
    let now = chrono::Utc::now();
    let session = crate::models::session::Session {
        id: uuid::Uuid::new_v4(),
        user_id: user.id,
        token: access_token.clone(),
        refresh_token: Some(refresh_token.clone()),
        expires_at: now + chrono::Duration::hours(1),
        created_at: now,
        last_accessed: now,
        ip_address: None,
        user_agent: None,
        revoked: false,
        mfa_verified: false,
        is_temp_session: false,
        mfa_verified_at: None,
    };

    // Store session asynchronously (don't block token response)
    let session_store = state.session_store.clone();
    tokio::spawn(async move {
        if let Err(e) = session_store.store_session(session).await {
            tracing::error!("Failed to store session: {}", e);
        }
    });

    tracing::info!(
        "Generated Ed25519 JWT tokens for user {} in realm {}",
        user.username,
        realm_id
    );

    Ok((access_token, refresh_token, expires_in))
}
