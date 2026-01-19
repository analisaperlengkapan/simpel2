//! GCP Secrets Engine API handlers

use axum::{
    Router,
    extract::{Path, State},
    response::Json,
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::{ApiError, ApiResponse, ApiResult};
use secreton_core::services::secrets::gcp::{
    GcpConfig, GcpCredentials, GcpCredentialsRequest, GcpRoleCreateRequest, GcpRoleResponse,
};

use super::AppState;

/// Create GCP routes
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/config/root", post(configure_gcp))
        .route("/config/root", get(get_gcp_config))
        .route("/roles", post(create_gcp_role))
        .route("/roles", get(list_gcp_roles))
        .route("/roles/:role_name", get(get_gcp_role))
        .route("/roles/:role_name", delete(delete_gcp_role))
        .route("/creds/:role_name", get(generate_gcp_credentials))
}

/// Configure GCP root credentials
#[tracing::instrument(skip(state, request))]
async fn configure_gcp(
    State(state): State<AppState>,
    Json(request): Json<GcpConfig>,
) -> ApiResult<Json<ApiResponse<()>>> {
    info!("Configuring GCP secrets engine");

    match state.gcp_engine.configure(request).await {
        Ok(_) => {
            info!("GCP secrets engine configured successfully");
            Ok(Json(ApiResponse::success(())))
        }
        Err(e) => {
            error!("Failed to configure GCP: {:?}", e);
            Err(ApiError::bad_request(format!(
                "Failed to configure GCP: {}",
                e
            )))
        }
    }
}

/// Get GCP configuration
#[tracing::instrument(skip(state))]
async fn get_gcp_config(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<GcpConfigResponse>>> {
    info!("Getting GCP configuration");

    match state.gcp_engine.get_config().await {
        Ok(config) => {
            let response = GcpConfigResponse {
                project_id: config.project_id,
                max_ttl: config.max_ttl,
                default_ttl: config.default_ttl,
            };
            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            error!("Failed to get GCP config: {:?}", e);
            Err(ApiError::not_found(format!("GCP not configured: {}", e)))
        }
    }
}

/// Create GCP role
#[tracing::instrument(skip(state))]
async fn create_gcp_role(
    State(state): State<AppState>,
    Json(request): Json<GcpRoleCreateRequest>,
) -> ApiResult<Json<ApiResponse<GcpRoleResponse>>> {
    info!("Creating GCP role: {}", request.name);

    match state.gcp_engine.create_role(request).await {
        Ok(response) => {
            info!("GCP role created successfully");
            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            error!("Failed to create GCP role: {:?}", e);
            Err(ApiError::bad_request(format!(
                "Failed to create role: {}",
                e
            )))
        }
    }
}

/// Get GCP role
#[tracing::instrument(skip(state))]
async fn get_gcp_role(
    State(state): State<AppState>,
    Path(role_name): Path<String>,
) -> ApiResult<Json<ApiResponse<GcpRoleResponse>>> {
    info!("Getting GCP role: {}", role_name);

    match state.gcp_engine.get_role(&role_name).await {
        Ok(response) => Ok(Json(ApiResponse::success(response))),
        Err(e) => {
            error!("Failed to get GCP role: {:?}", e);
            Err(ApiError::not_found(format!("Role not found: {}", e)))
        }
    }
}

/// Delete GCP role
#[tracing::instrument(skip(state))]
async fn delete_gcp_role(
    State(state): State<AppState>,
    Path(role_name): Path<String>,
) -> ApiResult<Json<ApiResponse<()>>> {
    info!("Deleting GCP role: {}", role_name);

    match state.gcp_engine.delete_role(&role_name).await {
        Ok(_) => {
            info!("GCP role deleted successfully");
            Ok(Json(ApiResponse::success(())))
        }
        Err(e) => {
            error!("Failed to delete GCP role: {:?}", e);
            Err(ApiError::not_found(format!("Failed to delete role: {}", e)))
        }
    }
}

/// List GCP roles
#[tracing::instrument(skip(state))]
async fn list_gcp_roles(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<GcpRoleListResponse>>> {
    info!("Listing GCP roles");

    let roles = state.gcp_engine.list_roles().await;
    Ok(Json(ApiResponse::success(GcpRoleListResponse { roles })))
}

/// Generate GCP credentials
#[tracing::instrument(skip(state))]
async fn generate_gcp_credentials(
    State(state): State<AppState>,
    Path(role_name): Path<String>,
) -> ApiResult<Json<ApiResponse<GcpCredentials>>> {
    info!("Generating GCP credentials for role: {}", role_name);

    let request = GcpCredentialsRequest {
        role_name,
        ttl: None,
    };

    match state.gcp_engine.generate_credentials(request).await {
        Ok(credentials) => {
            info!("GCP credentials generated successfully");
            Ok(Json(ApiResponse::success(credentials)))
        }
        Err(e) => {
            error!("Failed to generate GCP credentials: {:?}", e);
            Err(ApiError::bad_request(format!(
                "Failed to generate credentials: {}",
                e
            )))
        }
    }
}

/// GCP configuration response (without sensitive data)
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `GcpConfigResponse`.
pub struct GcpConfigResponse {
    pub project_id: String,
    pub max_ttl: u32,
    pub default_ttl: u32,
}

/// GCP role list response
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `GcpRoleListResponse`.
pub struct GcpRoleListResponse {
    pub roles: Vec<String>,
}
