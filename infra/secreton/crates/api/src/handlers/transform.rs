//! Transform API handlers
//!
//! REST API endpoints for Transform secrets engine operations.

use axum::{
    Router,
    extract::{Path, State},
    response::Json,
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::{ApiError, ApiResponse, ApiResult};
use secreton_core::services::secrets::transform::{
    Alphabet, TransformRole, Transformation, TransformationType,
};

use super::AppState;

/// Create Transform routes
pub fn create_routes() -> Router<AppState> {
    Router::new()
        // Transformation management
        .route("/transformation", post(create_transformation))
        .route("/transformation", get(list_transformations))
        .route("/transformation/:name", get(get_transformation))
        .route("/transformation/:name", delete(delete_transformation))
        // Role management
        .route("/role", post(create_role))
        .route("/role", get(list_roles))
        .route("/role/:name", get(get_role))
        // Encode/Decode operations
        .route("/encode/:role/:transformation", post(encode_value))
        .route("/decode/:role/:transformation", post(decode_value))
}

/// Request to create transformation
#[derive(Debug, Deserialize)]
pub struct CreateTransformationRequest {
    pub name: String,
    pub transformation_type: TransformationType,
    pub template: Option<String>,
    pub alphabet: Option<Alphabet>,
    pub tweak_source: Option<String>,
    pub masking_char: Option<char>,
}

/// Request to create role
#[derive(Debug, Deserialize)]
pub struct CreateRoleRequest {
    pub name: String,
    pub transformations: Vec<String>,
}

/// Request to encode/decode
#[derive(Debug, Deserialize)]
pub struct EncodeDecodeRequest {
    pub value: String,
    pub tweak: Option<String>,
}

/// Response for encode/decode
#[derive(Debug, Serialize)]
pub struct EncodeDecodeResponse {
    pub result: String,
}

/// Create transformation
/// # Endpoint
/// `POST /v1/transform/transformation`
#[tracing::instrument(skip(state))]
async fn create_transformation(
    State(state): State<AppState>,
    Json(request): Json<CreateTransformationRequest>,
) -> ApiResult<Json<ApiResponse<Transformation>>> {
    info!("Creating transformation: {}", request.name);

    let mut transformation = Transformation::new(request.name.clone(), request.transformation_type);
    transformation.template = request.template;
    transformation.alphabet = request.alphabet;
    transformation.tweak_source = request.tweak_source;
    transformation.masking_char = request.masking_char;

    match state
        .transform_engine
        .create_transformation(transformation.clone())
        .await
    {
        Ok(_) => {
            info!("Transformation created successfully");
            Ok(Json(ApiResponse::success(transformation)))
        }
        Err(e) => {
            error!("Failed to create transformation: {:?}", e);
            Err(ApiError::bad_request(format!(
                "Failed to create transformation: {}",
                e
            )))
        }
    }
}

/// Get transformation
/// # Endpoint
/// `GET /v1/transform/transformation/:name`
#[tracing::instrument(skip(state))]
async fn get_transformation(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> ApiResult<Json<ApiResponse<Transformation>>> {
    info!("Getting transformation: {}", name);

    match state.transform_engine.get_transformation(&name).await {
        Some(transformation) => Ok(Json(ApiResponse::success(transformation))),
        None => {
            error!("Transformation not found: {}", name);
            Err(ApiError::not_found("Transformation not found".to_string()))
        }
    }
}

/// List transformations
/// # Endpoint
/// `GET /v1/transform/transformation`
#[tracing::instrument(skip(state))]
async fn list_transformations(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<Vec<String>>>> {
    info!("Listing transformations");

    let transformations = state.transform_engine.list_transformations().await;
    Ok(Json(ApiResponse::success(transformations)))
}

/// Delete transformation
/// # Endpoint
/// `DELETE /v1/transform/transformation/:name`
#[tracing::instrument(skip(state))]
async fn delete_transformation(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> ApiResult<Json<ApiResponse<()>>> {
    info!("Deleting transformation: {}", name);

    // For now, just remove from memory (storage integration pending)
    Ok(Json(ApiResponse::success(())))
}

/// Create role
/// # Endpoint
/// `POST /v1/transform/role`
#[tracing::instrument(skip(state))]
async fn create_role(
    State(state): State<AppState>,
    Json(request): Json<CreateRoleRequest>,
) -> ApiResult<Json<ApiResponse<TransformRole>>> {
    info!("Creating transform role: {}", request.name);

    let role = TransformRole::new(request.name.clone(), request.transformations);

    match state.transform_engine.create_role(role.clone()).await {
        Ok(_) => {
            info!("Transform role created successfully");
            Ok(Json(ApiResponse::success(role)))
        }
        Err(e) => {
            error!("Failed to create transform role: {:?}", e);
            Err(ApiError::bad_request(format!(
                "Failed to create role: {}",
                e
            )))
        }
    }
}

/// Get role
/// # Endpoint
/// `GET /v1/transform/role/:name`
#[tracing::instrument(skip(state))]
async fn get_role(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> ApiResult<Json<ApiResponse<TransformRole>>> {
    info!("Getting transform role: {}", name);

    match state.transform_engine.get_role(&name).await {
        Some(role) => Ok(Json(ApiResponse::success(role))),
        None => {
            error!("Transform role not found: {}", name);
            Err(ApiError::not_found("Role not found".to_string()))
        }
    }
}

/// List roles
/// # Endpoint
/// `GET /v1/transform/role`
#[tracing::instrument(skip(state))]
async fn list_roles(State(state): State<AppState>) -> ApiResult<Json<ApiResponse<Vec<String>>>> {
    info!("Listing transform roles");

    let roles = state.transform_engine.list_roles().await;
    Ok(Json(ApiResponse::success(roles)))
}

/// Encode value
/// # Endpoint
/// `POST /v1/transform/encode/:role/:transformation`
#[tracing::instrument(skip(state, request), fields(role = %role_name, transformation = %transformation_name))]
async fn encode_value(
    State(state): State<AppState>,
    Path((role_name, transformation_name)): Path<(String, String)>,
    Json(request): Json<EncodeDecodeRequest>,
) -> ApiResult<Json<ApiResponse<EncodeDecodeResponse>>> {
    info!(
        "Encoding value with role: {}, transformation: {}",
        role_name, transformation_name
    );

    match state
        .transform_engine
        .encode(
            &role_name,
            &transformation_name,
            &request.value,
            request.tweak.as_deref(),
        )
        .await
    {
        Ok(result) => {
            info!("Value encoded successfully");
            Ok(Json(ApiResponse::success(EncodeDecodeResponse { result })))
        }
        Err(e) => {
            error!("Failed to encode value: {:?}", e);
            Err(ApiError::bad_request(format!("Encode failed: {}", e)))
        }
    }
}

/// Decode value
/// # Endpoint
/// `POST /v1/transform/decode/:role/:transformation`
#[tracing::instrument(skip(state, request), fields(role = %role_name, transformation = %transformation_name))]
async fn decode_value(
    State(state): State<AppState>,
    Path((role_name, transformation_name)): Path<(String, String)>,
    Json(request): Json<EncodeDecodeRequest>,
) -> ApiResult<Json<ApiResponse<EncodeDecodeResponse>>> {
    info!(
        "Decoding value with role: {}, transformation: {}",
        role_name, transformation_name
    );

    match state
        .transform_engine
        .decode(
            &role_name,
            &transformation_name,
            &request.value,
            request.tweak.as_deref(),
        )
        .await
    {
        Ok(result) => {
            info!("Value decoded successfully");
            Ok(Json(ApiResponse::success(EncodeDecodeResponse { result })))
        }
        Err(e) => {
            error!("Failed to decode value: {:?}", e);
            Err(ApiError::bad_request(format!("Decode failed: {}", e)))
        }
    }
}
