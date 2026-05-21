//! Satker (Government Hierarchy) management HTTP handlers

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};
use crate::state::IamApiState;
use authenc_types::AuthencError;

/// Satker type enum
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SatkerType {
    Kejaksaan,
    Kejari,
    Kejati,
    Cabang,
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
    pub id: Uuid,
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

/// Satker hierarchy information
#[derive(Debug, Serialize)]
pub struct SatkerHierarchyInfo {
    pub satker_code: String,
    pub ancestors: Vec<String>,
    pub descendants: Vec<String>,
    pub level: i32,
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
    pub user_id: Uuid,
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
    pub user_id: Uuid,
    pub source_satker: String,
    pub target_satker: String,
    pub operation: String,
    pub resource_type: String,
}

/// Cross-satker validation response
#[derive(Debug, Serialize)]
pub struct CrossSatkerValidation {
    pub allowed: bool,
    pub requires_approval: bool,
    pub approval_level: Option<String>,
    pub reason: String,
}

// ============================================================
// Helper Mapping Functions
// ============================================================

fn map_api_type_to_domain(api_type: SatkerType) -> authenc_types::domain::satker::SatkerType {
    match api_type {
        SatkerType::Kejaksaan => authenc_types::domain::satker::SatkerType::Pusat,
        SatkerType::Kejati => authenc_types::domain::satker::SatkerType::KejaksaanTinggi,
        SatkerType::Kejari => authenc_types::domain::satker::SatkerType::KejaksaanNegeri,
        SatkerType::Cabang => authenc_types::domain::satker::SatkerType::Cabang,
    }
}

fn map_domain_type_to_api(domain_type: authenc_types::domain::satker::SatkerType) -> SatkerType {
    match domain_type {
        authenc_types::domain::satker::SatkerType::Pusat => SatkerType::Kejaksaan,
        authenc_types::domain::satker::SatkerType::KejaksaanTinggi => SatkerType::Kejati,
        authenc_types::domain::satker::SatkerType::KejaksaanNegeri => SatkerType::Kejari,
        authenc_types::domain::satker::SatkerType::Cabang => SatkerType::Cabang,
        authenc_types::domain::satker::SatkerType::UnitKhusus => SatkerType::Kejaksaan,
    }
}

fn map_domain_to_response(satker: authenc_types::domain::satker::Satker) -> SatkerResponse {
    SatkerResponse {
        id: satker.id,
        code: satker.code,
        name: satker.name,
        description: satker.description,
        parent_code: satker.parent_code,
        level: satker.level,
        satker_type: map_domain_type_to_api(satker.satker_type),
        active: satker.active,
        attributes: satker.attributes,
        created_at: satker.created_at,
        updated_at: satker.updated_at,
    }
}

fn map_hierarchy_info_to_api(
    info: authenc_core::services::SatkerHierarchyInfo,
) -> SatkerHierarchyInfo {
    SatkerHierarchyInfo {
        satker_code: info.satker.code,
        ancestors: info.ancestors.into_iter().map(|s| s.code).collect(),
        descendants: info.descendants.into_iter().map(|s| s.code).collect(),
        level: info.level,
    }
}

// ============================================================
// HTTP Handlers
// ============================================================

/// GET /api/v1/iam/satker/{id} - Get a satker by code
pub async fn get_satker(
    State(state): State<Arc<IamApiState>>,
    Path(code): Path<String>,
) -> ApiResult<Json<SatkerResponse>> {
    let satker = state
        .satker_service
        .get_satker(&code)
        .await
        .map_err(ApiError)?
        .ok_or_else(|| {
            ApiError(AuthencError::not_found(format!(
                "Satker '{}' tidak ditemukan",
                code
            )))
        })?;

    Ok(Json(map_domain_to_response(satker)))
}

/// GET /api/v1/iam/satker - List satkers with optional filtering
pub async fn list_satkers(
    State(state): State<Arc<IamApiState>>,
    Query(query): Query<ListSatkersQuery>,
) -> ApiResult<Json<Vec<SatkerResponse>>> {
    let satkers = state
        .satker_service
        .list_satkers(query.parent_code, query.level, query.search)
        .await
        .map_err(ApiError)?;

    let response = satkers.into_iter().map(map_domain_to_response).collect();
    Ok(Json(response))
}

/// POST /api/v1/iam/satker - Create a new satker
pub async fn create_satker(
    State(state): State<Arc<IamApiState>>,
    Json(request): Json<CreateSatkerRequest>,
) -> ApiResult<(StatusCode, Json<SatkerResponse>)> {
    let domain_type = map_api_type_to_domain(request.satker_type);
    let satker = state
        .satker_service
        .create_satker(
            request.code,
            request.name,
            request.description,
            request.parent_code,
            request.level,
            domain_type,
            request.attributes,
        )
        .await
        .map_err(ApiError)?;

    Ok((StatusCode::CREATED, Json(map_domain_to_response(satker))))
}

/// PUT /api/v1/iam/satker/{id} - Update a satker
pub async fn update_satker(
    State(state): State<Arc<IamApiState>>,
    Path(code): Path<String>,
    Json(request): Json<UpdateSatkerRequest>,
) -> ApiResult<Json<SatkerResponse>> {
    let domain_type = request.satker_type.map(map_api_type_to_domain);
    let satker = state
        .satker_service
        .update_satker(
            &code,
            request.name,
            request.description,
            request.parent_code,
            request.level,
            domain_type,
            request.attributes,
        )
        .await
        .map_err(ApiError)?;

    Ok(Json(map_domain_to_response(satker)))
}

/// GET /api/v1/iam/satker/{id}/hierarchy - Get satker hierarchy information
pub async fn get_satker_hierarchy(
    State(state): State<Arc<IamApiState>>,
    Path(code): Path<String>,
) -> ApiResult<Json<SatkerHierarchyInfo>> {
    let info = state
        .satker_service
        .get_satker_hierarchy_info(&code)
        .await
        .map_err(ApiError)?;

    Ok(Json(map_hierarchy_info_to_api(info)))
}

/// GET /api/v1/iam/satker/roots - Get root satkers
pub async fn get_root_satkers(
    State(state): State<Arc<IamApiState>>,
) -> ApiResult<Json<Vec<SatkerResponse>>> {
    let satkers = state
        .satker_service
        .get_root_satkers()
        .await
        .map_err(ApiError)?;

    let response = satkers.into_iter().map(map_domain_to_response).collect();
    Ok(Json(response))
}

/// POST /api/v1/iam/satker/{id}/authorization - Check if a user can access a target satker
pub async fn check_satker_access(
    State(state): State<Arc<IamApiState>>,
    Path(code): Path<String>,
    Json(request): Json<CheckAccessRequest>,
) -> ApiResult<Json<CheckAccessResponse>> {
    let user = state
        .user_service
        .get_user(authenc_types::domain_types::UserId(request.user_id))
        .await
        .map_err(ApiError)?;

    let allowed = state
        .satker_auth_service
        .can_access_satker(&user, &code)
        .await
        .map_err(ApiError)?;

    Ok(Json(CheckAccessResponse {
        allowed,
        reason: if allowed {
            Some("Akses diizinkan berdasarkan hierarki Satker".to_string())
        } else {
            Some("Akses ditolak: User tidak berada dalam hierarki Satker target".to_string())
        },
    }))
}

/// POST /api/v1/iam/satker/validate-cross-operation - Validate a cross-satker operation
pub async fn validate_cross_satker_operation(
    State(state): State<Arc<IamApiState>>,
    Json(request): Json<CrossSatkerOperationRequest>,
) -> ApiResult<Json<CrossSatkerValidation>> {
    let user = state
        .user_service
        .get_user(authenc_types::domain_types::UserId(request.user_id))
        .await
        .map_err(ApiError)?;

    let validation = state
        .satker_auth_service
        .validate_cross_satker_operation(
            &user,
            &request.source_satker,
            &request.target_satker,
            &request.operation,
            &request.resource_type,
        )
        .await
        .map_err(ApiError)?;

    Ok(Json(CrossSatkerValidation {
        allowed: validation.allowed,
        requires_approval: !validation.allowed && !validation.involved_satkers.is_empty(),
        approval_level: if !validation.allowed {
            Some("wilayah".to_string())
        } else {
            None
        },
        reason: validation.reason,
    }))
}

/// GET /api/v1/iam/satker/users/{user_id}/accessible - Get accessible satkers for a user
pub async fn get_accessible_satkers(
    State(state): State<Arc<IamApiState>>,
    Path(user_id): Path<Uuid>,
) -> ApiResult<Json<Vec<String>>> {
    let user = state
        .user_service
        .get_user(authenc_types::domain_types::UserId(user_id))
        .await
        .map_err(ApiError)?;

    let accessible = state
        .satker_auth_service
        .get_accessible_satkers(&user)
        .await
        .map_err(ApiError)?;

    Ok(Json(accessible))
}

/// GET /api/v1/iam/satker/users/{user_id}/can-manage/{satker_code} - Check if a user can manage a target satker
pub async fn check_satker_management(
    State(state): State<Arc<IamApiState>>,
    Path((user_id, satker_code)): Path<(Uuid, String)>,
) -> ApiResult<Json<CheckAccessResponse>> {
    let user = state
        .user_service
        .get_user(authenc_types::domain_types::UserId(user_id))
        .await
        .map_err(ApiError)?;

    let allowed = state
        .satker_auth_service
        .can_manage_satker(&user, &satker_code)
        .await
        .map_err(ApiError)?;

    Ok(Json(CheckAccessResponse {
        allowed,
        reason: if allowed {
            Some("User memiliki hak akses manajemen untuk Satker ini".to_string())
        } else {
            Some("User tidak memiliki hak akses manajemen untuk Satker ini".to_string())
        },
    }))
}
