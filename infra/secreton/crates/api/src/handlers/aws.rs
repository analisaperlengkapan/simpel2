//! AWS Secrets Engine API handlers

use axum::{
    Router,
    extract::{Path, State},
    response::Json,
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::{ApiError, ApiResponse, ApiResult};
use secreton_core::services::secrets::aws::{
    AwsConfig, AwsCredentials, AwsCredentialsRequest, AwsRoleCreateRequest, AwsRoleResponse,
};

use super::AppState;

/// Create AWS routes
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/config/root", post(configure_aws))
        .route("/config/root", get(get_aws_config))
        .route("/roles", post(create_aws_role))
        .route("/roles", get(list_aws_roles))
        .route("/roles/:role_name", get(get_aws_role))
        .route("/roles/:role_name", delete(delete_aws_role))
        .route("/creds/:role_name", get(generate_aws_credentials))
        .route("/sts/:role_name", post(generate_sts_credentials))
}

/// Configure AWS root credentials
#[tracing::instrument(skip(state, request))]
async fn configure_aws(
    State(state): State<AppState>,
    Json(request): Json<AwsConfig>,
) -> ApiResult<Json<ApiResponse<()>>> {
    info!("Configuring AWS secrets engine");

    match state.aws_engine.configure(request).await {
        Ok(_) => {
            info!("AWS secrets engine configured successfully");
            Ok(Json(ApiResponse::success(())))
        }
        Err(e) => {
            error!("Failed to configure AWS: {:?}", e);
            Err(ApiError::bad_request(format!(
                "Failed to configure AWS: {}",
                e
            )))
        }
    }
}

/// Get AWS configuration
#[tracing::instrument(skip(state))]
async fn get_aws_config(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<AwsConfigResponse>>> {
    info!("Getting AWS configuration");

    match state.aws_engine.get_config().await {
        Ok(config) => {
            let response = AwsConfigResponse {
                region: config.region,
                sts_endpoint: config.sts_endpoint,
                iam_endpoint: config.iam_endpoint,
                max_ttl: config.max_ttl,
                default_ttl: config.default_ttl,
            };
            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            error!("Failed to get AWS config: {:?}", e);
            Err(ApiError::not_found(format!("AWS not configured: {}", e)))
        }
    }
}

/// Create AWS role
#[tracing::instrument(skip(state))]
async fn create_aws_role(
    State(state): State<AppState>,
    Json(request): Json<AwsRoleCreateRequest>,
) -> ApiResult<Json<ApiResponse<AwsRoleResponse>>> {
    info!("Creating AWS role: {}", request.name);

    match state.aws_engine.create_role(request).await {
        Ok(response) => {
            info!("AWS role created successfully");
            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            error!("Failed to create AWS role: {:?}", e);
            Err(ApiError::bad_request(format!(
                "Failed to create role: {}",
                e
            )))
        }
    }
}

/// Get AWS role
#[tracing::instrument(skip(state))]
async fn get_aws_role(
    State(state): State<AppState>,
    Path(role_name): Path<String>,
) -> ApiResult<Json<ApiResponse<AwsRoleResponse>>> {
    info!("Getting AWS role: {}", role_name);

    match state.aws_engine.get_role(&role_name).await {
        Ok(response) => Ok(Json(ApiResponse::success(response))),
        Err(e) => {
            error!("Failed to get AWS role: {:?}", e);
            Err(ApiError::not_found(format!("Role not found: {}", e)))
        }
    }
}

/// Delete AWS role
#[tracing::instrument(skip(state))]
async fn delete_aws_role(
    State(state): State<AppState>,
    Path(role_name): Path<String>,
) -> ApiResult<Json<ApiResponse<()>>> {
    info!("Deleting AWS role: {}", role_name);

    match state.aws_engine.delete_role(&role_name).await {
        Ok(_) => {
            info!("AWS role deleted successfully");
            Ok(Json(ApiResponse::success(())))
        }
        Err(e) => {
            error!("Failed to delete AWS role: {:?}", e);
            Err(ApiError::not_found(format!("Failed to delete role: {}", e)))
        }
    }
}

/// List AWS roles
#[tracing::instrument(skip(state))]
async fn list_aws_roles(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<AwsRoleListResponse>>> {
    info!("Listing AWS roles");

    let roles = state.aws_engine.list_roles().await;
    Ok(Json(ApiResponse::success(AwsRoleListResponse { roles })))
}

/// Generate AWS credentials
#[tracing::instrument(skip(state))]
async fn generate_aws_credentials(
    State(state): State<AppState>,
    Path(role_name): Path<String>,
) -> ApiResult<Json<ApiResponse<AwsCredentials>>> {
    info!("Generating AWS credentials for role: {}", role_name);

    let request = AwsCredentialsRequest {
        role_name,
        ttl: None,
        role_session_name: None,
    };

    match state.aws_engine.generate_credentials(request).await {
        Ok(credentials) => {
            info!("AWS credentials generated successfully");
            Ok(Json(ApiResponse::success(credentials)))
        }
        Err(e) => {
            error!("Failed to generate AWS credentials: {:?}", e);
            Err(ApiError::bad_request(format!(
                "Failed to generate credentials: {}",
                e
            )))
        }
    }
}

/// Generate STS credentials with custom parameters
#[tracing::instrument(skip(state))]
async fn generate_sts_credentials(
    State(state): State<AppState>,
    Path(role_name): Path<String>,
    Json(mut request): Json<AwsCredentialsRequest>,
) -> ApiResult<Json<ApiResponse<AwsCredentials>>> {
    info!("Generating STS credentials for role: {}", role_name);

    request.role_name = role_name;

    match state.aws_engine.generate_credentials(request).await {
        Ok(credentials) => {
            info!("STS credentials generated successfully");
            Ok(Json(ApiResponse::success(credentials)))
        }
        Err(e) => {
            error!("Failed to generate STS credentials: {:?}", e);
            Err(ApiError::bad_request(format!(
                "Failed to generate STS credentials: {}",
                e
            )))
        }
    }
}

/// AWS configuration response (without sensitive data)
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `AwsConfigResponse`.
pub struct AwsConfigResponse {
    pub region: String,
    pub sts_endpoint: Option<String>,
    pub iam_endpoint: Option<String>,
    pub max_ttl: u32,
    pub default_ttl: u32,
}

/// AWS role list response
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `AwsRoleListResponse`.
pub struct AwsRoleListResponse {
    pub roles: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ApiConfig;
    use crate::services::ServiceContainer;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_list_aws_roles() {
        let config = ApiConfig::default();
        let services = ServiceContainer::new(&config).await.unwrap();
        let app = create_routes().with_state(services.into());

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/roles")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
