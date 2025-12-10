//! RabbitMQ Secrets Engine API handlers

use axum::{
    Router,
    extract::{Path, State},
    response::Json,
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::{ApiError, ApiResponse, ApiResult};
use secreton_core::services::secrets::rabbitmq::{
    RabbitMqConfig, RabbitMqCredentialInfo, RabbitMqCredentials, RabbitMqRole,
};

use super::AppState;

/// Create RabbitMQ routes
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/rabbitmq/config", post(configure_rabbitmq))
        .route("/rabbitmq/config", get(get_rabbitmq_config))
        .route("/rabbitmq/roles", post(create_role))
        .route("/rabbitmq/roles", get(list_roles))
        .route("/rabbitmq/roles/:role_name", get(get_role))
        .route("/rabbitmq/roles/:role_name", delete(delete_role))
        .route("/rabbitmq/creds/:role_name", post(generate_credentials))
        .route("/rabbitmq/creds/:username/revoke", post(revoke_credentials))
        .route("/rabbitmq/creds", get(list_credentials))
        .route("/rabbitmq/creds/:username", get(get_credential_info))
}

/// Configure RabbitMQ connection
#[tracing::instrument(skip(state, request))]
async fn configure_rabbitmq(
    State(state): State<AppState>,
    Json(request): Json<RabbitMqConfig>,
) -> ApiResult<Json<ApiResponse<()>>> {
    info!("Configuring RabbitMQ secrets engine");

    match state.rabbitmq_engine.configure(request).await {
        Ok(_) => {
            info!("RabbitMQ configured successfully");
            Ok(Json(ApiResponse::success(())))
        }
        Err(e) => {
            error!("Failed to configure RabbitMQ: {:?}", e);
            Err(ApiError::bad_request(format!(
                "Failed to configure RabbitMQ: {}",
                e
            )))
        }
    }
}

/// Get RabbitMQ configuration
#[tracing::instrument(skip(state))]
async fn get_rabbitmq_config(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<RabbitMqConfigResponse>>> {
    info!("Getting RabbitMQ configuration");

    match state.rabbitmq_engine.get_config().await {
        Ok(config) => {
            let response = RabbitMqConfigResponse {
                connection_uri: config.connection_uri,
                management_uri: config.management_uri,
                username: config.username,
                verify_connection: config.verify_connection,
                default_ttl: config.default_ttl,
                max_ttl: config.max_ttl,
            };
            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            error!("Failed to get RabbitMQ config: {:?}", e);
            Err(ApiError::not_found(format!(
                "RabbitMQ not configured: {}",
                e
            )))
        }
    }
}

/// Create role
#[tracing::instrument(skip(state))]
async fn create_role(
    State(state): State<AppState>,
    Json(role): Json<RabbitMqRole>,
) -> ApiResult<Json<ApiResponse<()>>> {
    info!("Creating RabbitMQ role: {}", role.name);

    match state.rabbitmq_engine.create_role(role).await {
        Ok(_) => {
            info!("RabbitMQ role created successfully");
            Ok(Json(ApiResponse::success(())))
        }
        Err(e) => {
            error!("Failed to create role: {:?}", e);
            Err(ApiError::bad_request(format!(
                "Failed to create role: {}",
                e
            )))
        }
    }
}

/// Get role
#[tracing::instrument(skip(state))]
async fn get_role(
    State(state): State<AppState>,
    Path(role_name): Path<String>,
) -> ApiResult<Json<ApiResponse<RabbitMqRole>>> {
    info!("Getting RabbitMQ role: {}", role_name);

    match state.rabbitmq_engine.get_role(&role_name).await {
        Ok(role) => Ok(Json(ApiResponse::success(role))),
        Err(e) => {
            error!("Failed to get role: {:?}", e);
            Err(ApiError::not_found(format!("Role not found: {}", e)))
        }
    }
}

/// List roles
#[tracing::instrument(skip(state))]
async fn list_roles(State(state): State<AppState>) -> ApiResult<Json<ApiResponse<Vec<String>>>> {
    info!("Listing RabbitMQ roles");

    let roles = state.rabbitmq_engine.list_roles().await;
    Ok(Json(ApiResponse::success(roles)))
}

/// Delete role
#[tracing::instrument(skip(state))]
async fn delete_role(
    State(state): State<AppState>,
    Path(role_name): Path<String>,
) -> ApiResult<Json<ApiResponse<()>>> {
    info!("Deleting RabbitMQ role: {}", role_name);

    match state.rabbitmq_engine.delete_role(&role_name).await {
        Ok(_) => {
            info!("Role deleted successfully");
            Ok(Json(ApiResponse::success(())))
        }
        Err(e) => {
            error!("Failed to delete role: {:?}", e);
            Err(ApiError::bad_request(format!(
                "Failed to delete role: {}",
                e
            )))
        }
    }
}

/// Generate dynamic credentials
#[tracing::instrument(skip(state))]
async fn generate_credentials(
    State(state): State<AppState>,
    Path(role_name): Path<String>,
    Json(request): Json<GenerateCredentialsRequest>,
) -> ApiResult<Json<ApiResponse<RabbitMqCredentials>>> {
    info!("Generating RabbitMQ credentials for role: {}", role_name);

    match state
        .rabbitmq_engine
        .generate_credentials(&role_name, request.ttl)
        .await
    {
        Ok(cred) => {
            info!("Credentials generated successfully: {}", cred.username);
            Ok(Json(ApiResponse::success(cred)))
        }
        Err(e) => {
            error!("Failed to generate credentials: {:?}", e);
            Err(ApiError::bad_request(format!(
                "Failed to generate credentials: {}",
                e
            )))
        }
    }
}

/// Revoke credentials
#[tracing::instrument(skip(state))]
async fn revoke_credentials(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> ApiResult<Json<ApiResponse<()>>> {
    info!("Revoking RabbitMQ credentials: {}", username);

    match state.rabbitmq_engine.revoke_credentials(&username).await {
        Ok(_) => {
            info!("Credentials revoked successfully");
            Ok(Json(ApiResponse::success(())))
        }
        Err(e) => {
            error!("Failed to revoke credentials: {:?}", e);
            Err(ApiError::bad_request(format!(
                "Failed to revoke credentials: {}",
                e
            )))
        }
    }
}

/// List active credentials
#[tracing::instrument(skip(state))]
async fn list_credentials(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<Vec<String>>>> {
    info!("Listing RabbitMQ credentials");

    let creds = state.rabbitmq_engine.list_credentials().await;
    Ok(Json(ApiResponse::success(creds)))
}

/// Get credential info
#[tracing::instrument(skip(state))]
async fn get_credential_info(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> ApiResult<Json<ApiResponse<RabbitMqCredentialInfo>>> {
    info!("Getting RabbitMQ credential info: {}", username);

    match state.rabbitmq_engine.get_credential_info(&username).await {
        Some(info) => Ok(Json(ApiResponse::success(info))),
        None => {
            error!("Credential not found: {}", username);
            Err(ApiError::not_found("Credential not found".to_string()))
        }
    }
}

// Request/Response types

#[derive(Debug, Deserialize)]
pub struct GenerateCredentialsRequest {
    pub ttl: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct RabbitMqConfigResponse {
    pub connection_uri: String,
    pub management_uri: String,
    pub username: String,
    pub verify_connection: bool,
    pub default_ttl: i64,
    pub max_ttl: i64,
}
