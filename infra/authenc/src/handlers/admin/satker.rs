use crate::database::{Database, operations};
use crate::models::satker::{Satker, SatkerType};
use crate::services::satker_authorization::{SatkerAuthorizationService, SatkerHierarchyInfo};
use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Application state with satker authorization service
pub struct SatkerAppState {
    pub db: Arc<Database>,
    pub satker_auth: Arc<SatkerAuthorizationService>,
}

/// Create satker request
#[derive(Debug, Deserialize)]
pub struct CreateSatkerRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub parent_code: Option<String>,
    pub level: i32,
    pub satker_type: SatkerType,
    pub attributes: Option<serde_json::Value>,
}

/// Update satker request
#[derive(Debug, Deserialize)]
pub struct UpdateSatkerRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub parent_code: Option<String>,
    pub level: Option<i32>,
    pub satker_type: Option<SatkerType>,
    pub attributes: Option<serde_json::Value>,
}

/// Satker response
#[derive(Debug, Serialize)]
pub struct SatkerResponse {
    pub id: uuid::Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub parent_code: Option<String>,
    pub level: i32,
    pub satker_type: SatkerType,
    pub active: bool,
    pub attributes: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<Satker> for SatkerResponse {
    fn from(satker: Satker) -> Self {
        Self {
            id: satker.id,
            code: satker.code,
            name: satker.name,
            description: satker.description,
            parent_code: satker.parent_code,
            level: satker.level,
            satker_type: satker.satker_type,
            active: satker.active,
            attributes: satker.attributes,
            created_at: satker.created_at,
            updated_at: satker.updated_at,
        }
    }
}

/// List satkers query parameters
#[derive(Debug, Deserialize)]
pub struct ListSatkersQuery {
    pub parent_code: Option<String>,
    pub level: Option<i32>,
    pub search: Option<String>,
}

/// Check access request
#[derive(Debug, Deserialize)]
pub struct CheckAccessRequest {
    pub user_id: uuid::Uuid,
    pub target_satker_code: String,
}

/// Check access response
#[derive(Debug, Serialize)]
pub struct CheckAccessResponse {
    pub allowed: bool,
    pub reason: Option<String>,
}

/// Cross-satker operation request
#[derive(Debug, Deserialize)]
pub struct CrossSatkerOperationRequest {
    pub user_id: uuid::Uuid,
    pub source_satker: String,
    pub target_satker: String,
    pub operation: String,
    pub resource_type: String,
}

/// Get a satker by code
pub async fn get_satker(
    State(state): State<Arc<SatkerAppState>>,
    Path(code): Path<String>,
) -> Result<Json<SatkerResponse>, (StatusCode, String)> {
    let satker = state
        .db
        .get_satker_by_code(&code)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("Satker '{}' not found", code),
            )
        })?;

    Ok(Json(satker.into()))
}

/// List satkers with optional filtering
pub async fn list_satkers(
    State(state): State<Arc<SatkerAppState>>,
    Query(query): Query<ListSatkersQuery>,
) -> Result<Json<Vec<SatkerResponse>>, (StatusCode, String)> {
    let satkers = if let Some(search) = query.search {
        state
            .db
            .search_satkers(&search)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    } else if let Some(parent_code) = query.parent_code {
        state
            .db
            .get_satkers_by_parent(&parent_code)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    } else if let Some(level) = query.level {
        state
            .db
            .get_satkers_by_level(level)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    } else {
        state
            .db
            .get_all_satkers()
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    };

    Ok(Json(satkers.into_iter().map(Into::into).collect()))
}

/// Create a new satker
pub async fn create_satker(
    State(state): State<Arc<SatkerAppState>>,
    Json(request): Json<CreateSatkerRequest>,
) -> Result<Json<SatkerResponse>, (StatusCode, String)> {
    let satker = state
        .db
        .create_satker(
            request.code,
            request.name,
            request.description,
            request.parent_code,
            request.level,
            request.satker_type,
            request.attributes,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Update hierarchy in authorization service
    let all_satkers = state
        .db
        .get_all_satkers()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    state.satker_auth.update_hierarchy(all_satkers).await;

    Ok(Json(satker.into()))
}

/// Update a satker
pub async fn update_satker(
    State(state): State<Arc<SatkerAppState>>,
    Path(code): Path<String>,
    Json(request): Json<UpdateSatkerRequest>,
) -> Result<Json<SatkerResponse>, (StatusCode, String)> {
    let satker = state
        .db
        .update_satker(
            &code,
            request.name,
            request.description,
            request.parent_code,
            request.level,
            request.satker_type,
            request.attributes,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Update hierarchy in authorization service
    let all_satkers = state
        .db
        .get_all_satkers()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    state.satker_auth.update_hierarchy(all_satkers).await;

    Ok(Json(satker.into()))
}

/// Get satker hierarchy information
pub async fn get_satker_hierarchy(
    State(state): State<Arc<SatkerAppState>>,
    Path(code): Path<String>,
) -> Result<Json<SatkerHierarchyInfo>, (StatusCode, String)> {
    let hierarchy = state
        .satker_auth
        .get_satker_hierarchy(&code)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(hierarchy))
}

/// Get root satkers
pub async fn get_root_satkers(
    State(state): State<Arc<SatkerAppState>>,
) -> Result<Json<Vec<SatkerResponse>>, (StatusCode, String)> {
    let satkers = state
        .db
        .get_root_satkers()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(satkers.into_iter().map(Into::into).collect()))
}

/// Check if a user can access a target satker
pub async fn check_satker_access(
    State(state): State<Arc<SatkerAppState>>,
    Json(request): Json<CheckAccessRequest>,
) -> Result<Json<CheckAccessResponse>, (StatusCode, String)> {
    // Get user from database
    let user = operations::users::get_user_by_id(&state.db, request.user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("User '{}' not found", request.user_id),
            )
        })?;

    let allowed = state
        .satker_auth
        .can_access_satker(&user, &request.target_satker_code)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(CheckAccessResponse {
        allowed,
        reason: if allowed {
            Some("Access granted based on user roles and hierarchy".to_string())
        } else {
            Some("Access denied: insufficient permissions".to_string())
        },
    }))
}

/// Validate a cross-satker operation
pub async fn validate_cross_satker_operation(
    State(state): State<Arc<SatkerAppState>>,
    Json(request): Json<CrossSatkerOperationRequest>,
) -> Result<Json<crate::models::satker::CrossSatkerValidation>, (StatusCode, String)> {
    // Get user from database
    let user = operations::users::get_user_by_id(&state.db, request.user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("User '{}' not found", request.user_id),
            )
        })?;

    let validation = state
        .satker_auth
        .validate_cross_satker_operation(
            &user,
            &request.source_satker,
            &request.target_satker,
            &request.operation,
            &request.resource_type,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(validation))
}

/// Get accessible satkers for a user
pub async fn get_accessible_satkers(
    State(state): State<Arc<SatkerAppState>>,
    Path(user_id): Path<uuid::Uuid>,
) -> Result<Json<Vec<String>>, (StatusCode, String)> {
    // Get user from database
    let user = operations::users::get_user_by_id(&state.db, user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("User '{}' not found", user_id),
            )
        })?;

    let accessible = state
        .satker_auth
        .get_accessible_satkers(&user)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(accessible))
}

/// Check if a user can manage a target satker
pub async fn check_satker_management(
    State(state): State<Arc<SatkerAppState>>,
    Path((user_id, satker_code)): Path<(uuid::Uuid, String)>,
) -> Result<Json<CheckAccessResponse>, (StatusCode, String)> {
    // Get user from database
    let user = operations::users::get_user_by_id(&state.db, user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("User '{}' not found", user_id),
            )
        })?;

    let can_manage = state
        .satker_auth
        .can_manage_satker(&user, &satker_code)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(CheckAccessResponse {
        allowed: can_manage,
        reason: if can_manage {
            Some("User has management permissions for this satker".to_string())
        } else {
            Some("User does not have management permissions for this satker".to_string())
        },
    }))
}

/// Create satker routes
pub fn create_satker_routes() -> Router<Arc<SatkerAppState>> {
    Router::new()
        // Satker CRUD
        .route("/satkers", get(list_satkers))
        .route("/satkers", post(create_satker))
        .route("/satkers/:code", get(get_satker))
        .route("/satkers/:code", put(update_satker))
        .route("/satkers/roots", get(get_root_satkers))
        // Hierarchy queries
        .route("/satkers/:code/hierarchy", get(get_satker_hierarchy))
        // Authorization checks
        .route("/satkers/check-access", post(check_satker_access))
        .route(
            "/satkers/validate-cross-operation",
            post(validate_cross_satker_operation),
        )
        .route(
            "/satkers/users/:user_id/accessible",
            get(get_accessible_satkers),
        )
        .route(
            "/satkers/users/:user_id/can-manage/:satker_code",
            get(check_satker_management),
        )
}
