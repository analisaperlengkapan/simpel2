use crate::app::AppState;
use crate::error::{AuthencError, Result};
use crate::handlers::api::auth_bearer::AuthBearer;
use crate::models::client_scope::*;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

// ============================================================================
// CLIENT SCOPE CRUD ENDPOINTS
// ============================================================================

#[axum::debug_handler]
pub async fn list_client_scopes(
    State(app_state): State<Arc<AppState>>,
    Path(realm_id): Path<Uuid>,
    Query(params): Query<ListScopesQuery>,
    _auth: AuthBearer,
) -> Result<Json<Vec<ClientScopeResponse>>> {
    let enabled_only = params.enabled_only.unwrap_or(true);

    let scopes = app_state
        .client_scope_service
        .list_scopes(realm_id, enabled_only)
        .await?;

    let responses: Vec<ClientScopeResponse> = scopes.into_iter().map(|s| s.into()).collect();
    Ok(Json(responses))
}

/// Mewakili struktur data `ListScopesQuery`.
#[derive(Debug, Deserialize)]
/// List all client scopes in a realm
/// GET /api/v1/realms/{realm_id}/client-scopes
pub struct ListScopesQuery {
    pub enabled_only: Option<bool>,
}

#[axum::debug_handler]
pub async fn get_client_scope(
    State(app_state): State<Arc<AppState>>,
    Path((_realm_id, scope_id)): Path<(Uuid, Uuid)>,
    _auth: AuthBearer,
) -> Result<Json<ClientScopeResponse>> {
    let scope = app_state
        .client_scope_service
        .get_scope(scope_id)
        .await?
        .ok_or_else(|| AuthencError::not_found("Client scope not found"))?;

    Ok(Json(scope.into()))
}

/// Create a new client scope
/// POST /api/v1/realms/{realm_id}/client-scopes
#[axum::debug_handler]
pub async fn create_client_scope(
    State(app_state): State<Arc<AppState>>,
    Path(realm_id): Path<Uuid>,
    _auth: AuthBearer,
    Json(request): Json<CreateClientScopeRequest>,
) -> Result<impl IntoResponse> {
    let scope = app_state
        .client_scope_service
        .create_scope(realm_id, request)
        .await?;

    Ok((StatusCode::CREATED, Json(ClientScopeResponse::from(scope))))
}

/// Update a client scope
/// PUT /api/v1/realms/{realm_id}/client-scopes/{scope_id}
#[axum::debug_handler]
pub async fn update_client_scope(
    State(app_state): State<Arc<AppState>>,
    Path((_realm_id, scope_id)): Path<(Uuid, Uuid)>,
    _auth: AuthBearer,
    Json(request): Json<UpdateClientScopeRequest>,
) -> Result<Json<ClientScopeResponse>> {
    let scope = app_state
        .client_scope_service
        .update_scope(scope_id, request)
        .await?;

    Ok(Json(scope.into()))
}

/// Delete a client scope
/// DELETE /api/v1/realms/{realm_id}/client-scopes/{scope_id}
#[axum::debug_handler]
pub async fn delete_client_scope(
    State(app_state): State<Arc<AppState>>,
    Path((_realm_id, scope_id)): Path<(Uuid, Uuid)>,
    _auth: AuthBearer,
) -> Result<impl IntoResponse> {
    let deleted = app_state
        .client_scope_service
        .delete_scope(scope_id)
        .await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AuthencError::not_found("Client scope not found"))
    }
}

// ============================================================================
// CLIENT SCOPE ASSIGNMENT ENDPOINTS
// ============================================================================

/// Get all scopes assigned to a client
/// GET /api/v1/clients/{client_id}/scopes
#[axum::debug_handler]
pub async fn get_client_scopes(
    State(app_state): State<Arc<AppState>>,
    Path(client_id): Path<Uuid>,
    _auth: AuthBearer,
) -> Result<Json<ClientScopesResponse>> {
    let assignments = app_state
        .client_scope_service
        .get_all_client_scopes(client_id)
        .await?;

    let default_scopes: Vec<ClientScopeResponse> = assignments
        .iter()
        .filter(|a| a.assignment_type == ScopeAssignmentType::Default)
        .map(|a| a.scope.clone())
        .collect();

    let optional_scopes: Vec<ClientScopeResponse> = assignments
        .iter()
        .filter(|a| a.assignment_type == ScopeAssignmentType::Optional)
        .map(|a| a.scope.clone())
        .collect();

    Ok(Json(ClientScopesResponse {
        default_scopes,
        optional_scopes,
    }))
}
/// Mewakili struktur data `ClientScopesResponse`.

#[derive(Debug, Serialize)]
/// Get a specific client scope
/// GET /api/v1/realms/{realm_id}/client-scopes/{scope_id}
pub struct ClientScopesResponse {
    pub default_scopes: Vec<ClientScopeResponse>,
    pub optional_scopes: Vec<ClientScopeResponse>,
}

#[axum::debug_handler]
pub async fn assign_client_scopes(
    State(app_state): State<Arc<AppState>>,
    Path(client_id): Path<Uuid>,
    _auth: AuthBearer,
    Json(request): Json<AssignClientScopesRequest>,
) -> Result<impl IntoResponse> {
    app_state
        .client_scope_service
        .assign_client_scopes(client_id, request)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Get default scopes for a client
/// GET /api/v1/clients/{client_id}/default-scopes
#[axum::debug_handler]
pub async fn get_client_default_scopes(
    State(app_state): State<Arc<AppState>>,
    Path(client_id): Path<Uuid>,
    _auth: AuthBearer,
) -> Result<Json<Vec<ClientScopeResponse>>> {
    let scopes = app_state
        .client_scope_service
        .get_default_scopes(client_id)
        .await?;

    let responses: Vec<ClientScopeResponse> = scopes.into_iter().map(|s| s.into()).collect();
    Ok(Json(responses))
}

/// Get optional scopes for a client
/// GET /api/v1/clients/{client_id}/optional-scopes
#[axum::debug_handler]
pub async fn get_client_optional_scopes(
    State(app_state): State<Arc<AppState>>,
    Path(client_id): Path<Uuid>,
    _auth: AuthBearer,
) -> Result<Json<Vec<ClientScopeResponse>>> {
    let scopes = app_state
        .client_scope_service
        .get_optional_scopes(client_id)
        .await?;

    let responses: Vec<ClientScopeResponse> = scopes.into_iter().map(|s| s.into()).collect();
    Ok(Json(responses))
}

// ============================================================================
// USER CONSENT ENDPOINTS
// ============================================================================

/// Check if user consent is required for scopes
/// POST /api/v1/users/{user_id}/clients/{client_id}/consent/check
#[axum::debug_handler]
pub async fn check_user_consent(
    State(app_state): State<Arc<AppState>>,
    Path((user_id, client_id)): Path<(Uuid, Uuid)>,
    _auth: AuthBearer,
    Json(request): Json<CheckConsentRequest>,
) -> Result<Json<ConsentCheckResult>> {
    let result = app_state
        .client_scope_service
        .check_consent_required(user_id, client_id, &request.scopes)
        .await?;

    Ok(Json(result))
/// Mewakili struktur data `CheckConsentRequest`.
}

/// Mewakili struktur data `CheckConsentRequest`.
#[derive(Debug, Deserialize)]
/// Assign scopes to a client
/// PUT /api/v1/clients/{client_id}/scopes
pub struct CheckConsentRequest {
    pub scopes: String,
}

#[axum::debug_handler]
pub async fn grant_user_consent(
    State(app_state): State<Arc<AppState>>,
    Path((user_id, client_id)): Path<(Uuid, Uuid)>,
    Query(params): Query<GrantConsentQuery>,
    _auth: AuthBearer,
    Json(request): Json<GrantConsentRequestBody>,
) -> Result<impl IntoResponse> {
    let realm_id = params
        .realm_id
        .ok_or_else(|| AuthencError::validation("realm_id query parameter required"))?;

    let scope_names: Vec<String> = request.scope_names.iter().map(|s| s.to_string()).collect();

    let _consents = app_state
        .client_scope_service
        .grant_consent(
            user_id,
            client_id,
            realm_id,
            &scope_names,
            request.expires_in,
        )
        .await?;

/// Mewakili struktur data `GrantConsentQuery`.
    Ok(StatusCode::CREATED)
}

#[derive(Debug, Deserialize)]
/// Mewakili struktur data `GrantConsentRequestBody`.
/// Grant user consent for scopes
/// POST /api/v1/users/{user_id}/clients/{client_id}/consent
pub struct GrantConsentQuery {
    pub realm_id: Option<Uuid>,
}
/// Mewakili struktur data `GrantConsentRequestBody`.

#[derive(Debug, Deserialize)]
pub struct GrantConsentRequestBody {
    pub scope_names: Vec<String>,
    pub expires_in: Option<i64>,
}

#[axum::debug_handler]
pub async fn get_user_consents(
    State(app_state): State<Arc<AppState>>,
    Path((user_id, client_id)): Path<(Uuid, Uuid)>,
    _auth: AuthBearer,
) -> Result<Json<Vec<ClientScopeResponse>>> {
    let scopes = app_state
        .client_scope_service
        .get_consented_scopes(user_id, client_id)
        .await?;

    let responses: Vec<ClientScopeResponse> = scopes.into_iter().map(|s| s.into()).collect();
    Ok(Json(responses))
}

/// Revoke user consent for scopes
/// DELETE /api/v1/users/{user_id}/clients/{client_id}/consents
#[axum::debug_handler]
pub async fn revoke_user_consent(
    State(app_state): State<Arc<AppState>>,
    Path((user_id, client_id)): Path<(Uuid, Uuid)>,
    _auth: AuthBearer,
    Json(request): Json<RevokeConsentRequest>,
) -> Result<impl IntoResponse> {
    let scope_names = request.scope_names.as_ref().map(|v| v.as_slice());

    app_state
        .client_scope_service
        .revoke_consent(user_id, client_id, scope_names)
/// Mewakili struktur data `RevokeConsentRequest`.
        .await?;

    Ok(StatusCode::NO_CONTENT)
/// Mewakili struktur data `RevokeConsentRequest`.
}

/// Mewakili struktur data `RevokeConsentRequest`.
#[derive(Debug, Deserialize)]
/// Get consented scopes for a user and client
/// GET /api/v1/users/{user_id}/clients/{client_id}/consents
pub struct RevokeConsentRequest {
    pub scope_names: Option<Vec<String>>,
}

// ============================================================================
// SCOPE VALIDATION ENDPOINTS (for OAuth2 flows)
// ============================================================================

#[axum::debug_handler]
pub async fn validate_scopes(
    State(app_state): State<Arc<AppState>>,
    Path(client_id): Path<Uuid>,
    Query(params): Query<ValidateScopesQuery>,
    _auth: AuthBearer,
    Json(request): Json<ValidateScopesRequest>,
) -> Result<Json<ScopeValidationResult>> {
    let realm_id = params
        .realm_id
        .ok_or_else(|| AuthencError::validation("realm_id query parameter required"))?;

    let result = app_state
        .client_scope_service
/// Mewakili struktur data `ValidateScopesQuery`.
        .validate_requested_scopes(client_id, realm_id, &request.scopes)
        .await?;

    Ok(Json(result))
/// Mewakili struktur data `ValidateScopesRequest`.
}
/// Mewakili struktur data `ValidateScopesQuery`.

#[derive(Debug, Deserialize)]
/// Validate requested scopes for a client
/// POST /api/v1/clients/{client_id}/validate-scopes
pub struct ValidateScopesQuery {
/// Mewakili struktur data `ValidateScopesRequest`.
    pub realm_id: Option<Uuid>,
}

/// Mewakili struktur data `ValidateScopesRequest`.
#[derive(Debug, Deserialize)]
/// Mewakili struktur data `ValidateScopesRequest`.
pub struct ValidateScopesRequest {
    pub scopes: String,
}

// ============================================================================
// UTILITY ENDPOINTS
// ============================================================================

#[axum::debug_handler]
pub async fn get_standard_scopes(
    State(app_state): State<Arc<AppState>>,
    Path(realm_id): Path<Uuid>,
    _auth: AuthBearer,
) -> Result<Json<Vec<ClientScopeResponse>>> {
    let scopes = app_state
        .client_scope_service
        .get_standard_oidc_scopes(realm_id)
        .await?;

    let responses: Vec<ClientScopeResponse> = scopes.into_iter().map(|s| s.into()).collect();
    Ok(Json(responses))
}

/// Initialize standard scopes for a realm
/// POST /api/v1/realms/{realm_id}/client-scopes/initialize
#[axum::debug_handler]
pub async fn initialize_standard_scopes(
    State(app_state): State<Arc<AppState>>,
    Path(realm_id): Path<Uuid>,
    _auth: AuthBearer,
) -> Result<impl IntoResponse> {
    app_state
        .client_scope_service
        .initialize_standard_scopes(realm_id)
        .await?;

    Ok(StatusCode::CREATED)
}

// ============================================================================
// ROUTE REGISTRATION
// ============================================================================

use axum::Router;
use axum::routing::{get, post};

/// Register client scope routes
/// Get standard OIDC scopes for a realm
/// GET /api/v1/realms/{realm_id}/client-scopes/standard
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        // Client scope CRUD
        .route(
            "/realms/:realm_id/client-scopes",
            get(list_client_scopes).post(create_client_scope),
        )
        .route(
            "/realms/:realm_id/client-scopes/:scope_id",
            get(get_client_scope)
                .put(update_client_scope)
                .delete(delete_client_scope),
        )
        // Client scope assignments
        .route(
            "/clients/:client_id/scopes",
            get(get_client_scopes).put(assign_client_scopes),
        )
        .route(
            "/clients/:client_id/default-scopes",
            get(get_client_default_scopes),
        )
        .route(
            "/clients/:client_id/optional-scopes",
            get(get_client_optional_scopes),
        )
        // User consent
        .route(
            "/users/:user_id/clients/:client_id/consent/check",
            post(check_user_consent),
        )
        .route(
            "/users/:user_id/clients/:client_id/consent",
            post(grant_user_consent),
        )
        .route(
            "/users/:user_id/clients/:client_id/consents",
            get(get_user_consents).delete(revoke_user_consent),
        )
        // Scope validation
        .route("/clients/:client_id/validate-scopes", post(validate_scopes))
        // Utility
        .route(
            "/realms/:realm_id/client-scopes/standard",
            get(get_standard_scopes),
        )
        .route(
            "/realms/:realm_id/client-scopes/initialize",
            post(initialize_standard_scopes),
        )
}
