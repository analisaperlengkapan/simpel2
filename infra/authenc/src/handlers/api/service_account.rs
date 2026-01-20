use crate::app::AppState;
use crate::handlers::api::auth_bearer::AuthBearer;
use crate::models::events::{OperationType, ResourceType};
use crate::models::{
    CreateServiceAccountRequest, RegenerateSecretResponse, ServiceAccountListItem,
    ServiceAccountResponse, UpdateServiceAccountRequest,
};
use crate::services::events::AdminEventBuilder;
use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
};
use serde::Deserialize;
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

/// Create service account management routes for a realm
pub fn create_service_account_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/realms/{realm}/service-accounts",
            get(list_service_accounts),
        )
        .route(
            "/realms/{realm}/service-accounts",
            post(create_service_account),
        )
        .route(
            "/realms/{realm}/service-accounts/{sa_id}",
            get(get_service_account),
        )
        .route(
            "/realms/{realm}/service-accounts/{sa_id}",
            put(update_service_account),
        )
        .route(
            "/realms/{realm}/service-accounts/{sa_id}",
            delete(delete_service_account),
        )
        .route(
            "/realms/{realm}/service-accounts/{sa_id}/regenerate-secret",
            post(regenerate_secret),
        )
        .route(
            "/realms/{realm}/service-accounts/{sa_id}/roles",
            get(get_service_account_roles),
        )
        .route(
            "/realms/{realm}/service-accounts/{sa_id}/roles/{role_id}",
            post(assign_role),
        )
        .route(
            "/realms/{realm}/service-accounts/{sa_id}/roles/{role_id}",
            delete(revoke_role),
        )
        .route(
            "/realms/{realm}/service-accounts/{sa_id}/audit-log",
            get(get_audit_log),
        )
}

#[derive(Deserialize)]
struct ListQueryParams {
    first: Option<i64>,
    max: Option<i64>,
}

#[derive(Deserialize)]
struct AuditLogQueryParams {
    limit: Option<i64>,
    offset: Option<i64>,
}

/// List all service accounts in the specified realm
/// # Authorization
/// Requires admin permissions for the realm
/// # Query Parameters
/// - `first`: Pagination offset (default: 0)
/// - `max`: Maximum number of results (default: 100, max: 1000)
/// # Response
/// - `200 OK`: List of service accounts (without secrets)
/// - `401 Unauthorized`: Missing or invalid authentication
/// - `403 Forbidden`: Insufficient permissions
/// - `404 Not Found`: Realm not found
/// - `500 Internal Server Error`: Database error
pub async fn list_service_accounts(
    State(state): State<Arc<AppState>>,
    AuthBearer(_auth): AuthBearer,
    Path(realm): Path<String>,
    Query(params): Query<ListQueryParams>,
) -> Result<Json<Vec<ServiceAccountListItem>>, StatusCode> {
    // Get realm by name to get the UUID
    let realm_obj = match state.realm_store.get_by_name(&realm) {
        Some(r) => r,
        None => return Err(StatusCode::NOT_FOUND),
    };

    match state
        .service_account_store
        .list_by_realm(realm_obj.id, params.first, params.max)
        .await
    {
        Ok(service_accounts) => Ok(Json(service_accounts)),
        Err(e) => {
            error!("Failed to list service accounts: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get a specific service account by ID
/// # Authorization
/// Requires admin permissions for the realm
/// # Response
/// - `200 OK`: Service account details (without secret)
/// - `401 Unauthorized`: Missing or invalid authentication
/// - `403 Forbidden`: Insufficient permissions
/// - `404 Not Found`: Service account or realm not found
/// - `500 Internal Server Error`: Database error
pub async fn get_service_account(
    State(state): State<Arc<AppState>>,
    AuthBearer(_auth): AuthBearer,
    Path((realm, sa_id)): Path<(String, Uuid)>,
) -> Result<Json<ServiceAccountResponse>, StatusCode> {
    // Verify realm exists
    if state.realm_store.get_by_name(&realm).is_none() {
        return Err(StatusCode::NOT_FOUND);
    }

    match state.service_account_store.get(sa_id).await {
        Ok(Some(service_account)) => Ok(Json(ServiceAccountResponse::from(service_account))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get service account: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Create a new service account
/// # Authorization
/// Requires admin permissions for the realm
/// # Request Body
/// ```json
/// {
///   "name": "layanan-dasbor",
///   "description": "Dashboard microservice",
///   "client_secret": "optional-secret-or-auto-generated",
///   "enabled": true,
///   "roles": ["role-uuid-1", "role-uuid-2"]
/// }
/// ```
/// # Response
/// - `201 Created`: Service account created successfully
/// - `400 Bad Request`: Invalid input (empty name, invalid role IDs, etc.)
/// - `401 Unauthorized`: Missing or invalid authentication
/// - `403 Forbidden`: Insufficient permissions
/// - `404 Not Found`: Realm not found
/// - `409 Conflict`: Service account with same name already exists
/// - `500 Internal Server Error`: Database error
/// # Security
/// - Client secret is auto-generated if not provided
/// - Client secret is returned only in this response
/// - Client secret cannot be retrieved later (only regenerated)
pub async fn create_service_account(
    State(state): State<Arc<AppState>>,
    AuthBearer(auth): AuthBearer,
    Path(realm): Path<String>,
    Json(req): Json<CreateServiceAccountRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Get realm by name to get the UUID
    let realm_obj = match state.realm_store.get_by_name(&realm) {
        Some(r) => r,
        None => return Err(StatusCode::NOT_FOUND),
    };

    // Create service account
    let service_account = match state
        .service_account_store
        .create(
            realm_obj.id,
            &req.name,
            req.description.as_deref(),
            req.client_secret.clone(),
            req.enabled.unwrap_or(true),
            req.roles.clone(),
        )
        .await
    {
        Ok(sa) => sa,
        Err(e) => {
            error!("Failed to create service account: {}", e);
            return Err(if e.to_string().contains("already exists") {
                StatusCode::CONFLICT
            } else if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else if e.to_string().contains("validation")
                || e.to_string().contains("cannot be empty")
            {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            });
        }
    };

    // Fire admin event
    let auth_details = crate::models::events::AuthDetails {
        user_id: auth.sub.clone(),
        username: None,
        ip_address: None,
        user_agent: None,
    };

    let resource_path = format!("/realms/{}/service-accounts/{}", realm, service_account.id);
    let representation = serde_json::to_string(&service_account).unwrap_or_default();

    let admin_event = AdminEventBuilder::new(
        realm_obj.id.to_string(),
        auth_details,
        ResourceType::Client, // Service accounts are a type of client
        OperationType::Create,
        resource_path,
    )
    .representation(representation)
    .build();

    if let Err(e) = state
        .event_manager
        .write()
        .await
        .fire_admin_event(admin_event, true)
        .await
    {
        error!(
            "Failed to fire admin event for service account creation: {}",
            e
        );
    }

    // Return response with client secret (only shown once)
    let response = serde_json::json!({
        "id": service_account.id,
        "name": service_account.name,
        "description": service_account.description,
        "client_id": service_account.client_id,
        "client_secret": req.client_secret.unwrap_or_else(|| "AUTO_GENERATED".to_string()),
        "realm_id": service_account.realm_id,
        "enabled": service_account.enabled,
        "roles": service_account.roles,
        "created_at": service_account.created_at,
        "message": "Client secret is shown only once. Store it securely."
    });

    Ok(Json(response))
}

/// Update a service account
/// # Authorization
/// Requires admin permissions for the realm
/// # Request Body
/// All fields are optional (partial update supported):
/// ```json
/// {
///   "name": "new-name",
///   "description": "updated description",
///   "enabled": false,
///   "roles": ["new-role-uuid-1"]
/// }
/// ```
/// # Response
/// - `200 OK`: Service account updated successfully
/// - `400 Bad Request`: Invalid input
/// - `401 Unauthorized`: Missing or invalid authentication
/// - `403 Forbidden`: Insufficient permissions
/// - `404 Not Found`: Service account or realm not found
/// - `500 Internal Server Error`: Database error
pub async fn update_service_account(
    State(state): State<Arc<AppState>>,
    AuthBearer(auth): AuthBearer,
    Path((realm, sa_id)): Path<(String, Uuid)>,
    Json(req): Json<UpdateServiceAccountRequest>,
) -> Result<Json<ServiceAccountResponse>, StatusCode> {
    // Verify realm exists
    let realm_obj = match state.realm_store.get_by_name(&realm) {
        Some(r) => r,
        None => return Err(StatusCode::NOT_FOUND),
    };

    // Update service account
    let service_account = match state
        .service_account_store
        .update(
            sa_id,
            req.name.as_deref(),
            req.description.as_ref().map(|s| Some(s.as_str())),
            req.enabled,
            req.roles.clone(),
        )
        .await
    {
        Ok(sa) => sa,
        Err(e) => {
            error!("Failed to update service account: {}", e);
            return Err(if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else if e.to_string().contains("validation") {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            });
        }
    };

    // Fire admin event
    let auth_details = crate::models::events::AuthDetails {
        user_id: auth.sub.clone(),
        username: None,
        ip_address: None,
        user_agent: None,
    };

    let resource_path = format!("/realms/{}/service-accounts/{}", realm, sa_id);
    let representation = serde_json::to_string(&service_account).unwrap_or_default();

    let admin_event = AdminEventBuilder::new(
        realm_obj.id.to_string(),
        auth_details,
        ResourceType::Client,
        OperationType::Update,
        resource_path,
    )
    .representation(representation)
    .build();

    if let Err(e) = state
        .event_manager
        .write()
        .await
        .fire_admin_event(admin_event, true)
        .await
    {
        error!(
            "Failed to fire admin event for service account update: {}",
            e
        );
    }

    Ok(Json(ServiceAccountResponse::from(service_account)))
}

/// Delete a service account
/// # Authorization
/// Requires admin permissions for the realm
/// # Response
/// - `204 No Content`: Service account deleted successfully
/// - `401 Unauthorized`: Missing or invalid authentication
/// - `403 Forbidden`: Insufficient permissions
/// - `404 Not Found`: Service account or realm not found
/// - `500 Internal Server Error`: Database error
/// # Security
/// - Deletion is audit logged automatically (database trigger)
/// - All role assignments are removed (CASCADE)
/// - Authentication attempts with deleted service accounts will fail
pub async fn delete_service_account(
    State(state): State<Arc<AppState>>,
    AuthBearer(auth): AuthBearer,
    Path((realm, sa_id)): Path<(String, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    // Verify realm exists
    let realm_obj = match state.realm_store.get_by_name(&realm) {
        Some(r) => r,
        None => return Err(StatusCode::NOT_FOUND),
    };

    // Delete service account
    match state.service_account_store.delete(sa_id).await {
        Ok(_) => {
            // Fire admin event
            let auth_details = crate::models::events::AuthDetails {
                user_id: auth.sub.clone(),
                username: None,
                ip_address: None,
                user_agent: None,
            };

            let resource_path = format!("/realms/{}/service-accounts/{}", realm, sa_id);

            let admin_event = AdminEventBuilder::new(
                realm_obj.id.to_string(),
                auth_details,
                ResourceType::Client,
                OperationType::Delete,
                resource_path,
            )
            .build();

            if let Err(e) = state
                .event_manager
                .write()
                .await
                .fire_admin_event(admin_event, true)
                .await
            {
                error!(
                    "Failed to fire admin event for service account deletion: {}",
                    e
                );
            }

            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) => {
            error!("Failed to delete service account: {}", e);
            Err(if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            })
        }
    }
}

/// Regenerate client secret for a service account
/// # Authorization
/// Requires admin permissions for the realm
/// # Response
/// - `200 OK`: New client secret returned (only shown once)
/// - `401 Unauthorized`: Missing or invalid authentication
/// - `403 Forbidden`: Insufficient permissions
/// - `404 Not Found`: Service account or realm not found
/// - `500 Internal Server Error`: Database error
/// # Security
/// - Old secret is immediately invalidated
/// - New secret is auto-generated (32 characters, secure random)
/// - New secret is bcrypt-hashed before storage
/// - Regeneration is audit logged
/// - Secret is shown only in this response
pub async fn regenerate_secret(
    State(state): State<Arc<AppState>>,
    AuthBearer(auth): AuthBearer,
    Path((realm, sa_id)): Path<(String, Uuid)>,
) -> Result<Json<RegenerateSecretResponse>, StatusCode> {
    // Verify realm exists
    let realm_obj = match state.realm_store.get_by_name(&realm) {
        Some(r) => r,
        None => return Err(StatusCode::NOT_FOUND),
    };

    // Regenerate secret
    let (client_id, new_secret) = match state.service_account_store.regenerate_secret(sa_id).await {
        Ok(result) => result,
        Err(e) => {
            error!("Failed to regenerate service account secret: {}", e);
            return Err(if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            });
        }
    };

    // Fire admin event
    let auth_details = crate::models::events::AuthDetails {
        user_id: auth.sub.clone(),
        username: None,
        ip_address: None,
        user_agent: None,
    };

    let resource_path = format!(
        "/realms/{}/service-accounts/{}/regenerate-secret",
        realm, sa_id
    );

    let admin_event = AdminEventBuilder::new(
        realm_obj.id.to_string(),
        auth_details,
        ResourceType::Client,
        OperationType::Action,
        resource_path,
    )
    .build();

    if let Err(e) = state
        .event_manager
        .write()
        .await
        .fire_admin_event(admin_event, true)
        .await
    {
        error!("Failed to fire admin event for secret regeneration: {}", e);
    }

    Ok(Json(RegenerateSecretResponse {
        client_id,
        client_secret: new_secret,
        message: "Client secret regenerated successfully. Store this securely - it cannot be retrieved again.".to_string(),
    }))
}

/// Get roles assigned to a service account
/// # Authorization
/// Requires admin permissions for the realm
/// # Response
/// - `200 OK`: List of role IDs
/// - `401 Unauthorized`: Missing or invalid authentication
/// - `403 Forbidden`: Insufficient permissions
/// - `404 Not Found`: Service account or realm not found
/// - `500 Internal Server Error`: Database error
pub async fn get_service_account_roles(
    State(state): State<Arc<AppState>>,
    AuthBearer(_auth): AuthBearer,
    Path((realm, sa_id)): Path<(String, Uuid)>,
) -> Result<Json<Vec<Uuid>>, StatusCode> {
    // Verify realm exists
    if state.realm_store.get_by_name(&realm).is_none() {
        return Err(StatusCode::NOT_FOUND);
    }

    // Get service account
    let service_account = match state.service_account_store.get(sa_id).await {
        Ok(Some(sa)) => sa,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get service account: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok(Json(service_account.roles))
}

/// Assign a role to a service account
/// # Authorization
/// Requires admin permissions for the realm
/// # Response
/// - `204 No Content`: Role assigned successfully
/// - `401 Unauthorized`: Missing or invalid authentication
/// - `403 Forbidden`: Insufficient permissions
/// - `404 Not Found`: Service account, role, or realm not found
/// - `500 Internal Server Error`: Database error
pub async fn assign_role(
    State(state): State<Arc<AppState>>,
    AuthBearer(auth): AuthBearer,
    Path((realm, sa_id, role_id)): Path<(String, Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    // Verify realm exists
    let realm_obj = match state.realm_store.get_by_name(&realm) {
        Some(r) => r,
        None => return Err(StatusCode::NOT_FOUND),
    };

    // Get auth user ID
    let granted_by = Uuid::parse_str(&auth.sub).ok();

    // Assign role
    match state
        .service_account_store
        .assign_role(sa_id, role_id, granted_by)
        .await
    {
        Ok(_) => {
            // Fire admin event
            let auth_details = crate::models::events::AuthDetails {
                user_id: auth.sub.clone(),
                username: None,
                ip_address: None,
                user_agent: None,
            };

            let resource_path = format!(
                "/realms/{}/service-accounts/{}/roles/{}",
                realm, sa_id, role_id
            );

            let admin_event = AdminEventBuilder::new(
                realm_obj.id.to_string(),
                auth_details,
                ResourceType::Client,
                OperationType::Action,
                resource_path,
            )
            .build();

            if let Err(e) = state
                .event_manager
                .write()
                .await
                .fire_admin_event(admin_event, true)
                .await
            {
                error!("Failed to fire admin event for role assignment: {}", e);
            }

            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) => {
            error!("Failed to assign role to service account: {}", e);
            Err(if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            })
        }
    }
}

/// Revoke a role from a service account
/// # Authorization
/// Requires admin permissions for the realm
/// # Response
/// - `204 No Content`: Role revoked successfully
/// - `401 Unauthorized`: Missing or invalid authentication
/// - `403 Forbidden`: Insufficient permissions
/// - `404 Not Found`: Service account or realm not found
/// - `500 Internal Server Error`: Database error
pub async fn revoke_role(
    State(state): State<Arc<AppState>>,
    AuthBearer(auth): AuthBearer,
    Path((realm, sa_id, role_id)): Path<(String, Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    // Verify realm exists
    let realm_obj = match state.realm_store.get_by_name(&realm) {
        Some(r) => r,
        None => return Err(StatusCode::NOT_FOUND),
    };

    // Revoke role
    match state
        .service_account_store
        .revoke_role(sa_id, role_id)
        .await
    {
        Ok(_) => {
            // Fire admin event
            let auth_details = crate::models::events::AuthDetails {
                user_id: auth.sub.clone(),
                username: None,
                ip_address: None,
                user_agent: None,
            };

            let resource_path = format!(
                "/realms/{}/service-accounts/{}/roles/{}",
                realm, sa_id, role_id
            );

            let admin_event = AdminEventBuilder::new(
                realm_obj.id.to_string(),
                auth_details,
                ResourceType::Client,
                OperationType::Action,
                resource_path,
            )
            .build();

            if let Err(e) = state
                .event_manager
                .write()
                .await
                .fire_admin_event(admin_event, true)
                .await
            {
                error!("Failed to fire admin event for role revocation: {}", e);
            }

            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) => {
            error!("Failed to revoke role from service account: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get audit log for a service account
/// # Authorization
/// Requires admin permissions for the realm
/// # Query Parameters
/// - `limit`: Maximum number of entries (default: 100, max: 1000)
/// - `offset`: Pagination offset (default: 0)
/// # Response
/// - `200 OK`: Audit log entries
/// - `401 Unauthorized`: Missing or invalid authentication
/// - `403 Forbidden`: Insufficient permissions
/// - `404 Not Found`: Service account or realm not found
/// - `500 Internal Server Error`: Database error
pub async fn get_audit_log(
    State(state): State<Arc<AppState>>,
    AuthBearer(_auth): AuthBearer,
    Path((realm, sa_id)): Path<(String, Uuid)>,
    Query(params): Query<AuditLogQueryParams>,
) -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
    // Verify realm exists
    if state.realm_store.get_by_name(&realm).is_none() {
        return Err(StatusCode::NOT_FOUND);
    }

    // Get audit log
    match state
        .service_account_store
        .get_audit_log(sa_id, params.limit, params.offset)
        .await
    {
        Ok(logs) => Ok(Json(logs)),
        Err(e) => {
            error!("Failed to get audit log: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
