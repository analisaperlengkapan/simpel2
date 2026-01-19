//! KMIP Secrets Engine API handlers

use axum::{
    Router,
    extract::{Path, State},
    response::Json,
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::{ApiError, ApiResponse, ApiResult};
use secreton_core::services::secrets::kmip::{KmipKeyObject, KmipRole, KmipServerConfig};

use super::AppState;

/// Create KMIP routes
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/config", post(configure_kmip_server))
        .route("/config", get(get_kmip_config))
        .route("/keys", post(create_key))
        .route("/keys", get(list_keys))
        .route("/keys/:key_id", get(get_key))
        .route("/keys/:key_id/activate", post(activate_key))
        .route("/keys/:key_id/revoke", post(revoke_key))
        .route("/keys/:key_id/destroy", delete(destroy_key))
        .route("/register", post(register_key))
        .route("/roles", post(create_role))
        .route("/roles/:role_name", get(get_role))
}

/// Configure KMIP server connection
#[tracing::instrument(skip(state, request))]
async fn configure_kmip_server(
    State(state): State<AppState>,
    Json(request): Json<KmipServerConfig>,
) -> ApiResult<Json<ApiResponse<()>>> {
    info!("Configuring KMIP server");

    match state.kmip_engine.configure_server(request).await {
        Ok(_) => {
            info!("KMIP server configured successfully");
            Ok(Json(ApiResponse::success(())))
        }
        Err(e) => {
            error!("Failed to configure KMIP server: {:?}", e);
            Err(ApiError::bad_request(format!(
                "Failed to configure KMIP: {}",
                e
            )))
        }
    }
}

/// Get KMIP server configuration
#[tracing::instrument(skip(state))]
async fn get_kmip_config(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<KmipConfigResponse>>> {
    info!("Getting KMIP configuration");

    match state.kmip_engine.get_config().await {
        Ok(config) => {
            let response = KmipConfigResponse {
                host: config.host,
                port: config.port,
                tls_enabled: config.tls_enabled,
                has_ca_cert: config.ca_cert.is_some(),
                has_client_cert: config.client_cert.is_some(),
            };
            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            error!("Failed to get KMIP config: {:?}", e);
            Err(ApiError::not_found(format!("KMIP not configured: {}", e)))
        }
    }
}

/// Create new cryptographic key
#[tracing::instrument(skip(state))]
async fn create_key(
    State(state): State<AppState>,
    Json(request): Json<CreateKeyRequest>,
) -> ApiResult<Json<ApiResponse<KmipKeyResponse>>> {
    info!(
        "Creating KMIP key: algorithm={}, length={}",
        request.algorithm, request.key_length
    );

    match state
        .kmip_engine
        .create_key(
            request.algorithm,
            request.key_length,
            request.attributes.unwrap_or_default(),
        )
        .await
    {
        Ok(key) => {
            info!("KMIP key created successfully: {}", key.key_id);
            Ok(Json(ApiResponse::success(KmipKeyResponse::from(key))))
        }
        Err(e) => {
            error!("Failed to create KMIP key: {:?}", e);
            Err(ApiError::bad_request(format!(
                "Failed to create key: {}",
                e
            )))
        }
    }
}

/// Get key by ID
#[tracing::instrument(skip(state))]
async fn get_key(
    State(state): State<AppState>,
    Path(key_id): Path<String>,
) -> ApiResult<Json<ApiResponse<KmipKeyResponse>>> {
    info!("Getting KMIP key: {}", key_id);

    match state.kmip_engine.get_key(&key_id).await {
        Ok(key) => Ok(Json(ApiResponse::success(KmipKeyResponse::from(key)))),
        Err(e) => {
            error!("Failed to get KMIP key: {:?}", e);
            Err(ApiError::not_found(format!("Key not found: {}", e)))
        }
    }
}

/// List all keys
#[tracing::instrument(skip(state))]
async fn list_keys(State(state): State<AppState>) -> ApiResult<Json<ApiResponse<Vec<String>>>> {
    info!("Listing KMIP keys");

    let keys = state.kmip_engine.list_keys().await;
    Ok(Json(ApiResponse::success(keys)))
}

/// Register external key
#[tracing::instrument(skip(state))]
async fn register_key(
    State(state): State<AppState>,
    Json(request): Json<RegisterKeyRequest>,
) -> ApiResult<Json<ApiResponse<KmipKeyResponse>>> {
    info!(
        "Registering external KMIP key: algorithm={}",
        request.algorithm
    );

    match state
        .kmip_engine
        .register_key(
            request.algorithm,
            request.key_material,
            request.attributes.unwrap_or_default(),
        )
        .await
    {
        Ok(key) => {
            info!("External key registered successfully: {}", key.key_id);
            Ok(Json(ApiResponse::success(KmipKeyResponse::from(key))))
        }
        Err(e) => {
            error!("Failed to register key: {:?}", e);
            Err(ApiError::bad_request(format!(
                "Failed to register key: {}",
                e
            )))
        }
    }
}

/// Activate key
#[tracing::instrument(skip(state))]
async fn activate_key(
    State(state): State<AppState>,
    Path(key_id): Path<String>,
) -> ApiResult<Json<ApiResponse<KmipKeyResponse>>> {
    info!("Activating KMIP key: {}", key_id);

    match state.kmip_engine.activate_key(&key_id).await {
        Ok(key) => {
            info!("Key activated successfully: {}", key_id);
            Ok(Json(ApiResponse::success(KmipKeyResponse::from(key))))
        }
        Err(e) => {
            error!("Failed to activate key: {:?}", e);
            Err(ApiError::bad_request(format!(
                "Failed to activate key: {}",
                e
            )))
        }
    }
}

/// Revoke key
#[tracing::instrument(skip(state))]
async fn revoke_key(
    State(state): State<AppState>,
    Path(key_id): Path<String>,
) -> ApiResult<Json<ApiResponse<KmipKeyResponse>>> {
    info!("Revoking KMIP key: {}", key_id);

    match state.kmip_engine.revoke_key(&key_id).await {
        Ok(key) => {
            info!("Key revoked successfully: {}", key_id);
            Ok(Json(ApiResponse::success(KmipKeyResponse::from(key))))
        }
        Err(e) => {
            error!("Failed to revoke key: {:?}", e);
            Err(ApiError::bad_request(format!(
                "Failed to revoke key: {}",
                e
            )))
        }
    }
}

/// Destroy key
#[tracing::instrument(skip(state))]
async fn destroy_key(
    State(state): State<AppState>,
    Path(key_id): Path<String>,
) -> ApiResult<Json<ApiResponse<()>>> {
    info!("Destroying KMIP key: {}", key_id);

    match state.kmip_engine.destroy_key(&key_id).await {
        Ok(_) => {
            info!("Key destroyed successfully: {}", key_id);
            Ok(Json(ApiResponse::success(())))
        }
        Err(e) => {
            error!("Failed to destroy key: {:?}", e);
            Err(ApiError::bad_request(format!(
                "Failed to destroy key: {}",
                e
            )))
        }
    }
}

/// Create role
#[tracing::instrument(skip(state))]
async fn create_role(
    State(state): State<AppState>,
    Json(role): Json<KmipRole>,
) -> ApiResult<Json<ApiResponse<()>>> {
    info!("Creating KMIP role: {}", role.name);

    match state.kmip_engine.create_role(role).await {
        Ok(_) => {
            info!("KMIP role created successfully");
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
) -> ApiResult<Json<ApiResponse<KmipRole>>> {
    info!("Getting KMIP role: {}", role_name);

    match state.kmip_engine.get_role(&role_name).await {
        Some(role) => Ok(Json(ApiResponse::success(role))),
        None => {
            error!("Role not found: {}", role_name);
            Err(ApiError::not_found("Role not found".to_string()))
        }
    }
}

// Request/Response types

#[derive(Debug, Deserialize)]
/// Mewakili pub `CreateKeyRequest`.
pub struct CreateKeyRequest {
    pub algorithm: String,
    pub key_length: usize,
    pub attributes: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
/// Mewakili pub `RegisterKeyRequest`.
pub struct RegisterKeyRequest {
    pub algorithm: String,
    pub key_material: Vec<u8>,
    pub attributes: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Serialize)]
/// Mewakili pub `KmipConfigResponse`.
pub struct KmipConfigResponse {
    pub host: String,
    pub port: u16,
    pub tls_enabled: bool,
    pub has_ca_cert: bool,
    pub has_client_cert: bool,
}

#[derive(Debug, Serialize)]
/// Mewakili pub `KmipKeyResponse`.
pub struct KmipKeyResponse {
    pub key_id: String,
    pub key_format: String,
    pub key_state: String,
    pub algorithm: String,
    pub key_length: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub modified_at: chrono::DateTime<chrono::Utc>,
    pub namespace: Option<String>,
}

impl From<KmipKeyObject> for KmipKeyResponse {
    fn from(key: KmipKeyObject) -> Self {
        Self {
            key_id: key.key_id,
            key_format: format!("{:?}", key.key_format),
            key_state: format!("{:?}", key.key_state),
            algorithm: key.algorithm,
            key_length: key.key_length,
            created_at: key.created_at,
            modified_at: key.modified_at,
            namespace: key.namespace,
        }
    }
}
