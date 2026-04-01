//! Kubernetes Secrets Engine API handlers

use axum::{
    Router,
    extract::{Path, State},
    response::Json,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use crate::{ApiError, ApiResponse, ApiResult, extractors::AuthenticatedUser, handlers::AppState, helpers::create_audit_log};
use secreton_core::services::secrets::kubernetes::{
    KubernetesConnection, KubernetesRole,
};

/// Create Kubernetes routes
pub fn create_routes() -> Router<AppState> {
    Router::new()
        // Connection management
        .route("/config/:name", post(configure_connection))
        .route("/config/:name", get(get_connection))
        // Role management
        .route("/roles/:role", post(create_role))
        .route("/roles", get(list_roles))
        // Token generation
        .route("/creds/:role", get(generate_token))
}

/// Request to generate Kubernetes token
#[derive(Debug, Deserialize)]
pub struct GenerateTokenRequest {
    /// Optional TTL in seconds
    pub ttl: Option<u32>,
}

/// Response with generated token
#[derive(Debug, Serialize)]
pub struct GenerateTokenResponse {
    pub id: String,
    pub token: String,
    pub namespace: String,
    pub service_account: String,
    pub expires_at: String,
}

/// Configure Kubernetes connection
pub async fn configure_connection(
    State(state): State<AppState>,
    Path(name): Path<String>,
    user: AuthenticatedUser,
    Json(mut config): Json<KubernetesConnection>,
) -> ApiResult<Json<ApiResponse<()>>> {
    config.name = name.clone();

    state.kubernetes_engine.configure_connection(config).await
        .map_err(|e| ApiError::BadRequest { message: e.to_string() })?;

    // Log audit event
    let audit_entry = create_audit_log("k8s_connection_configured", &user.username, "k8s_connection", &name);
    let _ = state.audit.log(audit_entry).await;

    Ok(Json(ApiResponse::success(())))
}

/// Get Kubernetes connection
pub async fn get_connection(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> ApiResult<Json<ApiResponse<KubernetesConnection>>> {
    let connection = state.kubernetes_engine.get_connection(&name).await
        .ok_or_else(|| ApiError::NotFound { resource: format!("Kubernetes connection '{}'", name) })?;
    Ok(Json(ApiResponse::success(connection)))
}

/// Create Kubernetes role
pub async fn create_role(
    State(state): State<AppState>,
    Path(role_name): Path<String>,
    user: AuthenticatedUser,
    Json(mut role): Json<KubernetesRole>,
) -> ApiResult<Json<ApiResponse<()>>> {
    role.name = role_name.clone();

    state.kubernetes_engine.create_role(role).await
        .map_err(|e| ApiError::BadRequest { message: e.to_string() })?;

    // Log audit event
    let audit_entry = create_audit_log("k8s_role_created", &user.username, "k8s_role", &role_name);
    let _ = state.audit.log(audit_entry).await;

    Ok(Json(ApiResponse::success(())))
}

/// List Kubernetes roles
pub async fn list_roles(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<Vec<String>>>> {
    let roles = state.kubernetes_engine.list_roles().await;
    Ok(Json(ApiResponse::success(roles)))
}

/// Generate Kubernetes token for a role
pub async fn generate_token(
    State(state): State<AppState>,
    Path(role_name): Path<String>,
    user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<GenerateTokenResponse>>> {
    let token = state.kubernetes_engine.generate_token(&role_name, None).await
        .map_err(|e| ApiError::Internal { message: format!("Failed to generate token: {}", e) })?;

    // Log audit event
    let audit_entry = create_audit_log("k8s_token_generated", &user.username, "k8s_token", &role_name);
    let _ = state.audit.log(audit_entry).await;

    let response = GenerateTokenResponse {
        id: token.id,
        token: token.token,
        namespace: token.namespace,
        service_account: token.service_account,
        expires_at: token.expires_at.to_rfc3339(),
    };

    Ok(Json(ApiResponse::success(response)))
}
