//! Kafka Secrets Engine API handlers

use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::{ApiError, ApiResponse, ApiResult};
use secreton_core::services::secrets::kafka::{
    KafkaAcl, KafkaConfig, KafkaCredentialInfo, KafkaCredentials, KafkaError, KafkaRole,
    KafkaScramMechanism,
};

use super::AppState;

/// Create Kafka routes
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/kafka/config", post(configure_kafka))
        .route("/kafka/config", get(get_kafka_config))
        .route("/kafka/roles", post(create_role))
        .route("/kafka/roles", get(list_roles))
        .route("/kafka/roles/:role_name", get(get_role))
        .route("/kafka/roles/:role_name", delete(delete_role))
        .route("/kafka/creds/:role_name", post(generate_credentials))
        .route("/kafka/creds/:username/revoke", post(revoke_credentials))
        .route("/kafka/creds", get(list_credentials))
        .route("/kafka/creds/:username", get(get_credential_info))
}

/// Configure Kafka connection
#[tracing::instrument(skip(state, request))]
async fn configure_kafka(
    State(state): State<AppState>,
    Json(request): Json<KafkaConfig>,
) -> ApiResult<Json<ApiResponse<()>>> {
    info!("Configuring Kafka secrets engine");

    match state.kafka_engine.configure(request).await {
        Ok(_) => {
            info!("Kafka configured successfully");
            Ok(Json(ApiResponse::success(())))
        }
        Err(e) => {
            error!("Failed to configure Kafka: {:?}", e);
            Err(ApiError::bad_request(format!(
                "Failed to configure Kafka: {}",
                e
            )))
        }
    }
}

/// Get Kafka configuration
#[tracing::instrument(skip(state))]
async fn get_kafka_config(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<KafkaConfigResponse>>> {
    info!("Getting Kafka configuration");

    match state.kafka_engine.get_config().await {
        Ok(config) => {
            let response = KafkaConfigResponse {
                bootstrap_servers: config.bootstrap_servers,
                admin_username: config.admin_username,
                scram_mechanism: config.scram_mechanism,
                use_tls: config.use_tls,
                verify_connection: config.verify_connection,
                default_ttl: config.default_ttl,
                max_ttl: config.max_ttl,
            };
            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            error!("Failed to get Kafka config: {:?}", e);
            Err(ApiError::not_found(format!("Kafka not configured: {}", e)))
        }
    }
}

/// Create role
#[tracing::instrument(skip(state))]
async fn create_role(
    State(state): State<AppState>,
    Json(role): Json<KafkaRole>,
) -> ApiResult<Json<ApiResponse<()>>> {
    info!("Creating Kafka role: {}", role.name);

    match state.kafka_engine.create_role(role).await {
        Ok(_) => {
            info!("Kafka role created successfully");
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
) -> ApiResult<Json<ApiResponse<KafkaRole>>> {
    info!("Getting Kafka role: {}", role_name);

    match state.kafka_engine.get_role(&role_name).await {
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
    info!("Listing Kafka roles");

    let roles = state.kafka_engine.list_roles().await;
    Ok(Json(ApiResponse::success(roles)))
}

/// Delete role
#[tracing::instrument(skip(state))]
async fn delete_role(
    State(state): State<AppState>,
    Path(role_name): Path<String>,
) -> ApiResult<Json<ApiResponse<()>>> {
    info!("Deleting Kafka role: {}", role_name);

    match state.kafka_engine.delete_role(&role_name).await {
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
) -> ApiResult<Json<ApiResponse<KafkaCredentials>>> {
    info!("Generating Kafka credentials for role: {}", role_name);

    match state
        .kafka_engine
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
    info!("Revoking Kafka credentials: {}", username);

    match state.kafka_engine.revoke_credentials(&username).await {
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
    info!("Listing Kafka credentials");

    let creds = state.kafka_engine.list_credentials().await;
    Ok(Json(ApiResponse::success(creds)))
}

/// Get credential info
#[tracing::instrument(skip(state))]
async fn get_credential_info(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> ApiResult<Json<ApiResponse<KafkaCredentialInfo>>> {
    info!("Getting Kafka credential info: {}", username);

    match state.kafka_engine.get_credential_info(&username).await {
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
pub struct KafkaConfigResponse {
    pub bootstrap_servers: String,
    pub admin_username: String,
    pub scram_mechanism: KafkaScramMechanism,
    pub use_tls: bool,
    pub verify_connection: bool,
    pub default_ttl: i64,
    pub max_ttl: i64,
}
