/// Admin API for managing Dynamic Client Registration
///
/// Provides endpoints for:
/// - Creating initial access tokens
/// - Listing initial access tokens
/// - Revoking initial access tokens
/// - Viewing registration policies
use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::app::AppState;
use crate::database::operations::client_registration as db_ops;
use crate::error::{AuthencError, Result};

/// Create DCR admin routes
pub fn create_dcr_admin_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Initial Access Token Management
        .route("/initial-access-tokens", post(create_initial_access_token))
        .route("/initial-access-tokens", get(list_initial_access_tokens))
        .route(
            "/initial-access-tokens/{id}",
            delete(revoke_initial_access_token),
        )
        // Policy Management
        .route("/policies/{realm_id}", get(get_registration_policy))
        .route("/policies/{realm_id}", post(create_registration_policy))
        .route(
            "/policies/{realm_id}/{policy_id}",
            put(update_registration_policy),
        )
        .route(
            "/policies/{realm_id}/{policy_id}",
            delete(delete_registration_policy),
        )
        // Software Statement Issuer Management
        .route(
            "/software-statement-issuers",
            post(create_software_statement_issuer),
        )
        .route(
            "/software-statement-issuers",
            get(list_software_statement_issuers),
        )
        .route(
            "/software-statement-issuers/{id}",
            get(get_software_statement_issuer),
        )
        .route(
            "/software-statement-issuers/{id}",
            put(update_software_statement_issuer),
        )
        .route(
            "/software-statement-issuers/{id}",
            delete(delete_software_statement_issuer),
        )
}

#[derive(Debug, Deserialize)]
struct CreateInitialAccessTokenRequest {
    /// Number of allowed registrations
    #[serde(default = "default_count")]
    count: i32,
    /// Expiration time in seconds
    expires_in: Option<i32>,
    /// Realm ID
    realm_id: Option<Uuid>,
}

fn default_count() -> i32 {
    1
}

#[derive(Debug, Serialize)]
struct InitialAccessTokenResponse {
    /// Token ID
    id: Uuid,
    /// The actual token (only returned on creation)
    token: String,
    /// Number of allowed registrations
    count: i32,
    /// Remaining registrations
    remaining_count: i32,
    /// Expiration timestamp
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Created timestamp
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
struct InitialAccessTokenListItem {
    /// Token ID
    id: Uuid,
    /// Token hash (not the actual token)
    token_hash: String,
    /// Number of allowed registrations
    count: i32,
    /// Remaining registrations
    remaining_count: i32,
    /// Expiration timestamp
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Whether revoked
    revoked: bool,
    /// Created timestamp
    created_at: chrono::DateTime<chrono::Utc>,
    /// Last used timestamp
    last_used_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Create a new initial access token
async fn create_initial_access_token(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateInitialAccessTokenRequest>,
) -> Result<(StatusCode, Json<InitialAccessTokenResponse>)> {
    // Generate secure token
    use rand::distributions::Alphanumeric;
    use rand::{Rng, thread_rng};

    let token: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();

    // Hash token for storage
    let token_hash =
        bcrypt::hash(&token, bcrypt::DEFAULT_COST).map_err(|e| AuthencError::InternalError {
            message: format!("Failed to hash token: {}", e),
        })?;

    // Store in database
    let iat = db_ops::create_initial_access_token(
        &state.database,
        &token_hash,
        request.realm_id,
        request.count,
        request.expires_in,
        None, // TODO: Get current user ID from session
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(InitialAccessTokenResponse {
            id: iat.id,
            token, // Return actual token only on creation
            count: iat.count,
            remaining_count: iat.remaining_count,
            expires_at: iat.expires_at,
            created_at: iat.created_at,
        }),
    ))
}

#[derive(Debug, Deserialize)]
struct ListQuery {
    realm_id: Option<Uuid>,
}

/// List all initial access tokens
async fn list_initial_access_tokens(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<InitialAccessTokenListItem>>> {
    let tokens = db_ops::list_initial_access_tokens(&state.database, query.realm_id).await?;

    let items: Vec<InitialAccessTokenListItem> = tokens
        .into_iter()
        .map(|iat| InitialAccessTokenListItem {
            id: iat.id,
            token_hash: iat.token_hash,
            count: iat.count,
            remaining_count: iat.remaining_count,
            expires_at: iat.expires_at,
            revoked: iat.revoked,
            created_at: iat.created_at,
            last_used_at: iat.last_used_at,
        })
        .collect();

    Ok(Json(items))
}

/// Revoke an initial access token (by ID or token_hash)
async fn revoke_initial_access_token(
    State(state): State<Arc<AppState>>,
    Path(id_or_hash): Path<String>,
) -> Result<StatusCode> {
    // Try to parse as UUID first (ID), otherwise treat as token_hash
    let revoked = if let Ok(uuid) = Uuid::parse_str(&id_or_hash) {
        // Revoke by ID - need to get the hash first
        if let Some(token) = db_ops::get_initial_access_token_by_id(&state.database, uuid).await? {
            db_ops::revoke_initial_access_token(&state.database, &token.token_hash).await?
        } else {
            false
        }
    } else {
        // Revoke by hash directly
        db_ops::revoke_initial_access_token(&state.database, &id_or_hash).await?
    };

    if revoked {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AuthencError::ResourceNotFound {
            resource: format!("initial access token {}", id_or_hash),
        })
    }
}

#[derive(Debug, Serialize)]
struct RegistrationPolicyResponse {
    id: Uuid,
    realm_id: Uuid,
    name: String,
    allow_dynamic_registration: bool,
    require_initial_access_token: bool,
    require_software_statement: bool,
    max_redirect_uris: Option<i32>,
    allowed_scopes: Option<Vec<String>>,
    default_scopes: Option<Vec<String>>,
    allowed_grant_types: Option<Vec<String>>,
    allowed_response_types: Option<Vec<String>>,
    require_https_redirect_uris: bool,
    allow_localhost_redirect: bool,
    client_secret_expires_in: Option<i32>,
    registration_token_expires_in: Option<i32>,
}

/// Get registration policy for a realm
async fn get_registration_policy(
    State(state): State<Arc<AppState>>,
    Path(realm_id): Path<Uuid>,
) -> Result<Json<RegistrationPolicyResponse>> {
    let policy = db_ops::get_or_create_default_policy(&state.database, realm_id).await?;

    Ok(Json(RegistrationPolicyResponse {
        id: policy.id,
        realm_id: policy.realm_id,
        name: policy.name,
        allow_dynamic_registration: policy.allow_dynamic_registration,
        require_initial_access_token: policy.require_initial_access_token,
        require_software_statement: policy.require_software_statement,
        max_redirect_uris: policy.max_redirect_uris,
        allowed_scopes: policy.allowed_scopes,
        default_scopes: policy.default_scopes,
        allowed_grant_types: policy.allowed_grant_types,
        allowed_response_types: policy.allowed_response_types,
        require_https_redirect_uris: policy.require_https_redirect_uris,
        allow_localhost_redirect: policy.allow_localhost_redirect,
        client_secret_expires_in: policy.client_secret_expires_in,
        registration_token_expires_in: policy.registration_token_expires_in,
    }))
}

#[derive(Debug, Deserialize)]
struct CreatePolicyRequest {
    name: String,
    allow_dynamic_registration: bool,
    require_initial_access_token: bool,
    require_software_statement: bool,
    allowed_redirect_uri_patterns: Option<Vec<String>>,
    blocked_redirect_uri_patterns: Option<Vec<String>>,
    max_redirect_uris: Option<i32>,
    allowed_scopes: Option<Vec<String>>,
    default_scopes: Option<Vec<String>>,
    allowed_grant_types: Option<Vec<String>>,
    allowed_response_types: Option<Vec<String>>,
    require_https_redirect_uris: bool,
    allow_localhost_redirect: bool,
    client_secret_expires_in: Option<i32>,
    registration_token_expires_in: Option<i32>,
}

/// Create a new registration policy
async fn create_registration_policy(
    State(state): State<Arc<AppState>>,
    Path(realm_id): Path<Uuid>,
    Json(request): Json<CreatePolicyRequest>,
) -> Result<(StatusCode, Json<RegistrationPolicyResponse>)> {
    let policy = crate::models::ClientRegistrationPolicy {
        id: Uuid::new_v4(),
        realm_id,
        name: request.name.clone(),
        allow_dynamic_registration: request.allow_dynamic_registration,
        require_initial_access_token: request.require_initial_access_token,
        require_software_statement: request.require_software_statement,
        allowed_redirect_uri_patterns: request.allowed_redirect_uri_patterns,
        blocked_redirect_uri_patterns: request.blocked_redirect_uri_patterns,
        max_redirect_uris: request.max_redirect_uris,
        allowed_scopes: request.allowed_scopes,
        default_scopes: request.default_scopes,
        allowed_grant_types: request.allowed_grant_types,
        allowed_response_types: request.allowed_response_types,
        require_https_redirect_uris: request.require_https_redirect_uris,
        allow_localhost_redirect: request.allow_localhost_redirect,
        client_secret_expires_in: request.client_secret_expires_in,
        registration_token_expires_in: request.registration_token_expires_in,
        enabled: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let created = db_ops::create_client_registration_policy(
        &state.database,
        realm_id,
        &request.name,
        &policy,
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(RegistrationPolicyResponse {
            id: created.id,
            realm_id: created.realm_id,
            name: created.name,
            allow_dynamic_registration: created.allow_dynamic_registration,
            require_initial_access_token: created.require_initial_access_token,
            require_software_statement: created.require_software_statement,
            max_redirect_uris: created.max_redirect_uris,
            allowed_scopes: created.allowed_scopes,
            default_scopes: created.default_scopes,
            allowed_grant_types: created.allowed_grant_types,
            allowed_response_types: created.allowed_response_types,
            require_https_redirect_uris: created.require_https_redirect_uris,
            allow_localhost_redirect: created.allow_localhost_redirect,
            client_secret_expires_in: created.client_secret_expires_in,
            registration_token_expires_in: created.registration_token_expires_in,
        }),
    ))
}

/// Update a registration policy
async fn update_registration_policy(
    State(state): State<Arc<AppState>>,
    Path((realm_id, policy_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<CreatePolicyRequest>,
) -> Result<Json<RegistrationPolicyResponse>> {
    let policy = crate::models::ClientRegistrationPolicy {
        id: policy_id,
        realm_id,
        name: request.name,
        allow_dynamic_registration: request.allow_dynamic_registration,
        require_initial_access_token: request.require_initial_access_token,
        require_software_statement: request.require_software_statement,
        allowed_redirect_uri_patterns: request.allowed_redirect_uri_patterns,
        blocked_redirect_uri_patterns: request.blocked_redirect_uri_patterns,
        max_redirect_uris: request.max_redirect_uris,
        allowed_scopes: request.allowed_scopes,
        default_scopes: request.default_scopes,
        allowed_grant_types: request.allowed_grant_types,
        allowed_response_types: request.allowed_response_types,
        require_https_redirect_uris: request.require_https_redirect_uris,
        allow_localhost_redirect: request.allow_localhost_redirect,
        client_secret_expires_in: request.client_secret_expires_in,
        registration_token_expires_in: request.registration_token_expires_in,
        enabled: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let updated =
        db_ops::update_client_registration_policy(&state.database, realm_id, &policy).await?;

    Ok(Json(RegistrationPolicyResponse {
        id: updated.id,
        realm_id: updated.realm_id,
        name: updated.name,
        allow_dynamic_registration: updated.allow_dynamic_registration,
        require_initial_access_token: updated.require_initial_access_token,
        require_software_statement: updated.require_software_statement,
        max_redirect_uris: updated.max_redirect_uris,
        allowed_scopes: updated.allowed_scopes,
        default_scopes: updated.default_scopes,
        allowed_grant_types: updated.allowed_grant_types,
        allowed_response_types: updated.allowed_response_types,
        require_https_redirect_uris: updated.require_https_redirect_uris,
        allow_localhost_redirect: updated.allow_localhost_redirect,
        client_secret_expires_in: updated.client_secret_expires_in,
        registration_token_expires_in: updated.registration_token_expires_in,
    }))
}

/// Delete a registration policy
async fn delete_registration_policy(
    State(state): State<Arc<AppState>>,
    Path((realm_id, policy_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode> {
    let deleted =
        db_ops::delete_client_registration_policy(&state.database, realm_id, policy_id).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AuthencError::ResourceNotFound {
            resource: format!("policy {}", policy_id),
        })
    }
}

// Software Statement Issuer Management

#[derive(Debug, Deserialize)]
struct CreateSoftwareStatementIssuerRequest {
    name: String,
    issuer: String,
    jwks_uri: Option<String>,
    jwks: Option<serde_json::Value>,
    realm_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
struct SoftwareStatementIssuerResponse {
    id: Uuid,
    name: String,
    issuer: String,
    jwks_uri: Option<String>,
    jwks: Option<serde_json::Value>,
    realm_id: Option<Uuid>,
    enabled: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

/// Create a software statement issuer
async fn create_software_statement_issuer(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateSoftwareStatementIssuerRequest>,
) -> Result<(StatusCode, Json<SoftwareStatementIssuerResponse>)> {
    let issuer = db_ops::create_software_statement_issuer(
        &state.database,
        &request.name,
        &request.issuer,
        request.jwks_uri.as_deref(),
        request.jwks,
        request.realm_id,
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(SoftwareStatementIssuerResponse {
            id: issuer.id,
            name: issuer.name,
            issuer: issuer.issuer,
            jwks_uri: issuer.jwks_uri,
            jwks: issuer.jwks,
            realm_id: issuer.realm_id,
            enabled: issuer.enabled,
            created_at: issuer.created_at,
            updated_at: issuer.updated_at,
        }),
    ))
}

/// List software statement issuers
async fn list_software_statement_issuers(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<SoftwareStatementIssuerResponse>>> {
    let issuers = db_ops::list_software_statement_issuers(&state.database, query.realm_id).await?;

    let response: Vec<SoftwareStatementIssuerResponse> = issuers
        .into_iter()
        .map(|issuer| SoftwareStatementIssuerResponse {
            id: issuer.id,
            name: issuer.name,
            issuer: issuer.issuer,
            jwks_uri: issuer.jwks_uri,
            jwks: issuer.jwks,
            realm_id: issuer.realm_id,
            enabled: issuer.enabled,
            created_at: issuer.created_at,
            updated_at: issuer.updated_at,
        })
        .collect();

    Ok(Json(response))
}

/// Get a software statement issuer by ID
async fn get_software_statement_issuer(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<SoftwareStatementIssuerResponse>> {
    // Get by ID requires adding a get_by_id function
    // For now, list all and filter
    let issuers = db_ops::list_software_statement_issuers(&state.database, None).await?;

    let issuer =
        issuers
            .into_iter()
            .find(|i| i.id == id)
            .ok_or_else(|| AuthencError::ResourceNotFound {
                resource: format!("software statement issuer {}", id),
            })?;

    Ok(Json(SoftwareStatementIssuerResponse {
        id: issuer.id,
        name: issuer.name,
        issuer: issuer.issuer,
        jwks_uri: issuer.jwks_uri,
        jwks: issuer.jwks,
        realm_id: issuer.realm_id,
        enabled: issuer.enabled,
        created_at: issuer.created_at,
        updated_at: issuer.updated_at,
    }))
}

#[derive(Debug, Deserialize)]
struct UpdateSoftwareStatementIssuerRequest {
    name: Option<String>,
    jwks_uri: Option<String>,
    jwks: Option<serde_json::Value>,
    enabled: Option<bool>,
}

/// Update a software statement issuer
async fn update_software_statement_issuer(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateSoftwareStatementIssuerRequest>,
) -> Result<Json<SoftwareStatementIssuerResponse>> {
    let issuer = db_ops::update_software_statement_issuer(
        &state.database,
        id,
        request.name.as_deref(),
        request.jwks_uri.as_deref(),
        request.jwks,
        request.enabled,
    )
    .await?;

    Ok(Json(SoftwareStatementIssuerResponse {
        id: issuer.id,
        name: issuer.name,
        issuer: issuer.issuer,
        jwks_uri: issuer.jwks_uri,
        jwks: issuer.jwks,
        realm_id: issuer.realm_id,
        enabled: issuer.enabled,
        created_at: issuer.created_at,
        updated_at: issuer.updated_at,
    }))
}

/// Delete a software statement issuer
async fn delete_software_statement_issuer(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let deleted = db_ops::delete_software_statement_issuer(&state.database, id).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AuthencError::ResourceNotFound {
            resource: format!("software statement issuer {}", id),
        })
    }
}
