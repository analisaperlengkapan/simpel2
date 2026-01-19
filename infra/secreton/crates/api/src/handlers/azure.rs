//! Azure Secrets Engine API handlers

use axum::{
    Router,
    extract::{Path, State},
    response::Json,
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::{ApiError, ApiResponse, ApiResult};
use secreton_core::services::secrets::azure::{
    AzureConfig, AzureCredentials, AzureCredentialsRequest, AzureRoleCreateRequest,
    AzureRoleResponse,
};

use super::AppState;

/// Create Azure routes
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/config/root", post(configure_azure))
        .route("/config/root", get(get_azure_config))
        .route("/roles", post(create_azure_role))
        .route("/roles", get(list_azure_roles))
        .route("/roles/:role_name", get(get_azure_role))
        .route("/roles/:role_name", delete(delete_azure_role))
        .route("/creds/:role_name", get(generate_azure_credentials))
}

/// Configure Azure root credentials
#[tracing::instrument(skip(state, request))]
async fn configure_azure(
    State(state): State<AppState>,
    Json(request): Json<AzureConfig>,
) -> ApiResult<Json<ApiResponse<()>>> {
    info!("Configuring Azure secrets engine");

    match state.azure_engine.configure(request).await {
        Ok(_) => {
            info!("Azure secrets engine configured successfully");
            Ok(Json(ApiResponse::success(())))
        }
        Err(e) => {
            error!("Failed to configure Azure: {:?}", e);
            Err(ApiError::bad_request(format!(
                "Failed to configure Azure: {}",
                e
            )))
        }
    }
}

/// Get Azure configuration
#[tracing::instrument(skip(state))]
async fn get_azure_config(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<AzureConfigResponse>>> {
    info!("Getting Azure configuration");

    match state.azure_engine.get_config().await {
        Ok(config) => {
            let response = AzureConfigResponse {
                subscription_id: config.subscription_id,
                tenant_id: config.tenant_id,
                environment: config.environment,
                max_ttl: config.max_ttl,
                default_ttl: config.default_ttl,
            };
            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            error!("Failed to get Azure config: {:?}", e);
            Err(ApiError::not_found(format!("Azure not configured: {}", e)))
        }
    }
}

/// Create Azure role
#[tracing::instrument(skip(state))]
async fn create_azure_role(
    State(state): State<AppState>,
    Json(request): Json<AzureRoleCreateRequest>,
) -> ApiResult<Json<ApiResponse<AzureRoleResponse>>> {
    info!("Creating Azure role: {}", request.name);

    match state.azure_engine.create_role(request).await {
        Ok(response) => {
            info!("Azure role created successfully");
            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            error!("Failed to create Azure role: {:?}", e);
            Err(ApiError::bad_request(format!(
                "Failed to create role: {}",
                e
            )))
        }
    }
}

/// Get Azure role
#[tracing::instrument(skip(state))]
async fn get_azure_role(
    State(state): State<AppState>,
    Path(role_name): Path<String>,
) -> ApiResult<Json<ApiResponse<AzureRoleResponse>>> {
    info!("Getting Azure role: {}", role_name);

    match state.azure_engine.get_role(&role_name).await {
        Ok(response) => Ok(Json(ApiResponse::success(response))),
        Err(e) => {
            error!("Failed to get Azure role: {:?}", e);
            Err(ApiError::not_found(format!("Role not found: {}", e)))
        }
    }
}

/// Delete Azure role
#[tracing::instrument(skip(state))]
async fn delete_azure_role(
    State(state): State<AppState>,
    Path(role_name): Path<String>,
) -> ApiResult<Json<ApiResponse<()>>> {
    info!("Deleting Azure role: {}", role_name);

    match state.azure_engine.delete_role(&role_name).await {
        Ok(_) => {
            info!("Azure role deleted successfully");
            Ok(Json(ApiResponse::success(())))
        }
        Err(e) => {
            error!("Failed to delete Azure role: {:?}", e);
            Err(ApiError::not_found(format!("Failed to delete role: {}", e)))
        }
    }
}

/// List Azure roles
#[tracing::instrument(skip(state))]
async fn list_azure_roles(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<AzureRoleListResponse>>> {
    info!("Listing Azure roles");

    let roles = state.azure_engine.list_roles().await;
    Ok(Json(ApiResponse::success(AzureRoleListResponse { roles })))
}

/// Generate Azure credentials
#[tracing::instrument(skip(state))]
async fn generate_azure_credentials(
    State(state): State<AppState>,
    Path(role_name): Path<String>,
) -> ApiResult<Json<ApiResponse<AzureCredentials>>> {
    info!("Generating Azure credentials for role: {}", role_name);

    let request = AzureCredentialsRequest {
        role_name,
        ttl: None,
    };

    match state.azure_engine.generate_credentials(request).await {
        Ok(credentials) => {
            info!("Azure credentials generated successfully");
            Ok(Json(ApiResponse::success(credentials)))
        }
        Err(e) => {
            error!("Failed to generate Azure credentials: {:?}", e);
            Err(ApiError::bad_request(format!(
                "Failed to generate credentials: {}",
                e
            )))
        }
    }
}

/// Azure configuration response (without sensitive data)
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `AzureConfigResponse`.
pub struct AzureConfigResponse {
    pub subscription_id: String,
    pub tenant_id: String,
    pub environment: String,
    pub max_ttl: u32,
    pub default_ttl: u32,
}

/// Azure role list response
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `AzureRoleListResponse`.
pub struct AzureRoleListResponse {
    pub roles: Vec<String>,
}
