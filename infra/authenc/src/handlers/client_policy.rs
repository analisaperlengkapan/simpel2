//! Client Policy Management API Handlers
//!
//! RESTful API endpoints for managing client policies, profiles, and assignments.
//! Supports full CRUD operations and policy enforcement configuration.

use crate::error::AuthencError;
use crate::models::client_policy::*;
use crate::services::client_policy::ClientPolicyStore;
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

/// Shared application state for policy handlers
#[derive(Clone)]
pub struct PolicyHandlerState {
    pub policy_store: Arc<ClientPolicyStore>,
}

/// Query parameters for listing policies
#[derive(Debug, Deserialize)]
pub struct ListPoliciesQuery {
    pub realm_id: Uuid,
    pub enabled: Option<bool>,
    pub policy_type: Option<String>,
}

/// Query parameters for listing profiles
#[derive(Debug, Deserialize)]
pub struct ListProfilesQuery {
    pub realm_id: Uuid,
    pub enabled: Option<bool>,
}

// ========== Client Policy Handlers ==========

/// Create a new client policy
///
/// POST /api/v1/admin/client-policies
pub async fn create_client_policy(
    State(state): State<Arc<PolicyHandlerState>>,
    Json(request): Json<CreateClientPolicyRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    info!(
        "Creating client policy: {} in realm {}",
        request.name, request.realm_id
    );

    let realm_id = request.realm_id;

    let policy = state
        .policy_store
        .create_policy(realm_id, request, None)
        .await
        .map_err(|e| {
            error!("Failed to create policy: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok((
        StatusCode::CREATED,
        Json(ClientPolicyResponse::from(policy)),
    ))
}

/// Get a client policy by ID
///
/// GET /api/v1/admin/client-policies/{id}
pub async fn get_client_policy(
    State(state): State<Arc<PolicyHandlerState>>,
    Path(policy_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let policy = state
        .policy_store
        .get_policy(policy_id)
        .await
        .map_err(|e| {
            error!("Failed to get policy: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    match policy {
        Some(p) => Ok(Json(ClientPolicyResponse::from(p))),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// List client policies for a realm
///
/// GET /api/v1/admin/client-policies?realm_id=<uuid>
pub async fn list_client_policies(
    State(state): State<Arc<PolicyHandlerState>>,
    Query(query): Query<ListPoliciesQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let policies = state
        .policy_store
        .list_policies(query.realm_id)
        .await
        .map_err(|e| {
            error!("Failed to list policies: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Filter by enabled status if provided
    let filtered: Vec<ClientPolicyResponse> = policies
        .into_iter()
        .filter(|p| {
            if let Some(enabled) = query.enabled {
                p.enabled == enabled
            } else {
                true
            }
        })
        .filter(|p| {
            if let Some(ref policy_type) = query.policy_type {
                &p.policy_type == policy_type
            } else {
                true
            }
        })
        .map(ClientPolicyResponse::from)
        .collect();

    Ok(Json(filtered))
}

/// Update a client policy
///
/// PUT /api/v1/admin/client-policies/{id}
pub async fn update_client_policy(
    State(state): State<Arc<PolicyHandlerState>>,
    Path(policy_id): Path<Uuid>,
    Json(request): Json<UpdateClientPolicyRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    info!("Updating client policy: {}", policy_id);

    let policy = state
        .policy_store
        .update_policy(policy_id, request)
        .await
        .map_err(|e| {
            error!("Failed to update policy: {}", e);
            match e {
                AuthencError::ResourceNotFound { .. } => StatusCode::NOT_FOUND,
                AuthencError::ValidationError { .. } => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(ClientPolicyResponse::from(policy)))
}

/// Delete a client policy
///
/// DELETE /api/v1/admin/client-policies/{id}
pub async fn delete_client_policy(
    State(state): State<Arc<PolicyHandlerState>>,
    Path(policy_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    info!("Deleting client policy: {}", policy_id);

    state
        .policy_store
        .delete_policy(policy_id)
        .await
        .map_err(|e| {
            error!("Failed to delete policy: {}", e);
            match e {
                AuthencError::ResourceNotFound { .. } => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// ========== Client Profile Handlers ==========

/// Create a new client profile
///
/// POST /api/v1/admin/client-profiles
pub async fn create_client_profile(
    State(state): State<Arc<PolicyHandlerState>>,
    Json(request): Json<CreateClientProfileRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    info!(
        "Creating client profile: {} in realm {}",
        request.name, request.realm_id
    );

    let realm_id = request.realm_id;

    let profile = state
        .policy_store
        .create_profile(realm_id, request, None)
        .await
        .map_err(|e| {
            error!("Failed to create profile: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok((StatusCode::CREATED, Json(profile)))
}

/// Get a client profile by ID (with policies)
///
/// GET /api/v1/admin/client-profiles/{id}
pub async fn get_client_profile(
    State(state): State<Arc<PolicyHandlerState>>,
    Path(profile_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let profile = state
        .policy_store
        .get_profile_with_policies(profile_id)
        .await
        .map_err(|e| {
            error!("Failed to get profile: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    match profile {
        Some(p) => Ok(Json(p)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// List client profiles for a realm
///
/// GET /api/v1/admin/client-profiles?realm_id=<uuid>
pub async fn list_client_profiles(
    State(state): State<Arc<PolicyHandlerState>>,
    Query(query): Query<ListProfilesQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let profiles = state
        .policy_store
        .list_profiles(query.realm_id)
        .await
        .map_err(|e| {
            error!("Failed to list profiles: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Filter by enabled status if provided
    let filtered: Vec<ClientProfileModel> = profiles
        .into_iter()
        .filter(|p| {
            if let Some(enabled) = query.enabled {
                p.enabled == enabled
            } else {
                true
            }
        })
        .collect();

    Ok(Json(filtered))
}

/// Update a client profile
///
/// PUT /api/v1/admin/client-profiles/{id}
pub async fn update_client_profile(
    State(state): State<Arc<PolicyHandlerState>>,
    Path(profile_id): Path<Uuid>,
    Json(request): Json<UpdateClientProfileRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    info!("Updating client profile: {}", profile_id);

    let profile = state
        .policy_store
        .update_profile(profile_id, request)
        .await
        .map_err(|e| {
            error!("Failed to update profile: {}", e);
            match e {
                AuthencError::ResourceNotFound { .. } => StatusCode::NOT_FOUND,
                AuthencError::ValidationError { .. } => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(profile))
}

/// Delete a client profile
///
/// DELETE /api/v1/admin/client-profiles/{id}
pub async fn delete_client_profile(
    State(state): State<Arc<PolicyHandlerState>>,
    Path(profile_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    info!("Deleting client profile: {}", profile_id);

    state
        .policy_store
        .delete_profile(profile_id)
        .await
        .map_err(|e| {
            error!("Failed to delete profile: {}", e);
            match e {
                AuthencError::ResourceNotFound { .. } => StatusCode::NOT_FOUND,
                AuthencError::ValidationError { .. } => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// ========== Policy Assignment Handlers ==========

/// Assign a policy to a client
///
/// POST /api/v1/admin/clients/{client_id}/policies
pub async fn assign_policy_to_client(
    State(state): State<Arc<PolicyHandlerState>>,
    Path(client_id): Path<Uuid>,
    Json(request): Json<AssignClientPolicyRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    info!("Assigning policy to client: {}", client_id);

    let assignment = if let Some(policy_id) = request.policy_id {
        state
            .policy_store
            .assign_policy_to_client(
                client_id,
                policy_id,
                request.enabled,
                request.priority_override,
                None,
            )
            .await
    } else if let Some(profile_id) = request.profile_id {
        state
            .policy_store
            .assign_profile_to_client(client_id, profile_id, request.enabled, None)
            .await
    } else {
        return Err(StatusCode::BAD_REQUEST);
    };

    assignment
        .map(|a| (StatusCode::CREATED, Json(a)))
        .map_err(|e| {
            error!("Failed to assign policy: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

/// Get all policies assigned to a client
///
/// GET /api/v1/admin/clients/{client_id}/policies
pub async fn get_client_policies(
    State(state): State<Arc<PolicyHandlerState>>,
    Path(client_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let policies = state
        .policy_store
        .get_client_policies(client_id)
        .await
        .map_err(|e| {
            error!("Failed to get client policies: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let responses: Vec<ClientPolicyResponse> = policies
        .into_iter()
        .map(ClientPolicyResponse::from)
        .collect();

    Ok(Json(responses))
}

/// Get all policy assignments for a client
///
/// GET /api/v1/admin/clients/{client_id}/policy-assignments
pub async fn get_client_policy_assignments(
    State(state): State<Arc<PolicyHandlerState>>,
    Path(client_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let assignments = state
        .policy_store
        .list_client_assignments(client_id)
        .await
        .map_err(|e| {
            error!("Failed to get client assignments: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(assignments))
}

/// Remove a policy assignment from a client
///
/// DELETE /api/v1/admin/clients/{client_id}/policies/{policy_id}
pub async fn remove_policy_assignment(
    State(state): State<Arc<PolicyHandlerState>>,
    Path((client_id, policy_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, StatusCode> {
    info!(
        "Removing policy assignment from client: {} (policy: {})",
        client_id, policy_id
    );

    state
        .policy_store
        .remove_policy_assignment(client_id, Some(policy_id), None)
        .await
        .map_err(|e| {
            error!("Failed to remove policy assignment: {}", e);
            match e {
                AuthencError::ResourceNotFound { .. } => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// Remove a profile assignment from a client
///
/// DELETE /api/v1/admin/clients/{client_id}/profiles/{profile_id}
pub async fn remove_profile_assignment(
    State(state): State<Arc<PolicyHandlerState>>,
    Path((client_id, profile_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, StatusCode> {
    info!(
        "Removing profile assignment from client: {} (profile: {})",
        client_id, profile_id
    );

    state
        .policy_store
        .remove_policy_assignment(client_id, None, Some(profile_id))
        .await
        .map_err(|e| {
            error!("Failed to remove profile assignment: {}", e);
            match e {
                AuthencError::ResourceNotFound { .. } => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// ========== Router Setup ==========

use axum::Router;
use axum::routing::{delete, get, post, put};

/// Create the router for client policy management endpoints
pub fn create_policy_router(state: Arc<PolicyHandlerState>) -> Router {
    Router::new()
        // Policy CRUD
        .route("/client-policies", post(create_client_policy))
        .route("/client-policies", get(list_client_policies))
        .route("/client-policies/{id}", get(get_client_policy))
        .route("/client-policies/{id}", put(update_client_policy))
        .route("/client-policies/{id}", delete(delete_client_policy))
        // Profile CRUD
        .route("/client-profiles", post(create_client_profile))
        .route("/client-profiles", get(list_client_profiles))
        .route("/client-profiles/{id}", get(get_client_profile))
        .route("/client-profiles/{id}", put(update_client_profile))
        .route("/client-profiles/{id}", delete(delete_client_profile))
        // Client policy assignments
        .route(
            "/clients/{client_id}/policies",
            post(assign_policy_to_client),
        )
        .route("/clients/{client_id}/policies", get(get_client_policies))
        .route(
            "/clients/{client_id}/policy-assignments",
            get(get_client_policy_assignments),
        )
        .route(
            "/clients/{client_id}/policies/{policy_id}",
            delete(remove_policy_assignment),
        )
        .route(
            "/clients/{client_id}/profiles/{profile_id}",
            delete(remove_profile_assignment),
        )
        .with_state(state)
}
