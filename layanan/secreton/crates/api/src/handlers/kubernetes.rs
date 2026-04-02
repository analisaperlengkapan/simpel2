//! Kubernetes Secrets API Handlers
//!
//! Provides REST endpoints for Kubernetes dynamic token generation,
//! role management, and connection configuration.

use axum::{
    Router,
    extract::{Path, Query, State},
    response::Json,
    routing::{delete, get, post, put},
};

use serde::{Deserialize, Serialize};

use crate::{
    ApiError, ApiResponse, ApiResult, extractors::AuthenticatedUser, handlers::AppState,
    helpers::create_audit_log,
};

use secreton_core::services::secrets::kubernetes::{KubernetesConnection, KubernetesRole};

/// Create Kubernetes secrets routes
pub fn create_routes() -> Router<AppState> {
    Router::new()
        // Token generation
        .route("/tokens/:role", get(generate_kubernetes_token))
        // Role management
        .route("/roles", get(list_kubernetes_roles))
        .route("/roles/:role", post(create_kubernetes_role))
        .route("/roles/:role", get(get_kubernetes_role))
        .route("/roles/:role", put(update_kubernetes_role))
        .route("/roles/:role", delete(delete_kubernetes_role))
        // Connection management
        .route("/config/:name", post(configure_kubernetes_connection))
        .route("/config/:name", get(get_kubernetes_connection))
        .route("/config/:name", delete(delete_kubernetes_connection))
}

/// Request to generate Kubernetes token
#[derive(Debug, Deserialize)]
pub struct GenerateTokenRequest {
    /// Optional TTL in seconds
    pub ttl: Option<u32>,
}

/// Response with generated token and lease
#[derive(Debug, Serialize)]
pub struct GenerateTokenResponse {
    /// Lease ID
    pub lease_id: String,

    /// Lease duration in seconds
    pub lease_duration: i64,

    /// Whether lease is renewable
    pub renewable: bool,

    /// Token data
    pub data: TokenData,
}

#[derive(Debug, Serialize)]
pub struct TokenData {
    /// Service account token
    pub token: String,

    /// Namespace
    pub namespace: String,

    /// Service account name
    pub service_account: String,

    /// Expiration time
    pub expires_at: chrono::DateTime<chrono::Utc>,

    /// Role name
    pub role: String,
}

/// Generate Kubernetes token for a role
pub async fn generate_kubernetes_token(
    State(state): State<AppState>,
    Path(role_name): Path<String>,
    user: AuthenticatedUser,
    Query(params): Query<GenerateTokenRequest>,
) -> ApiResult<Json<ApiResponse<GenerateTokenResponse>>> {
    let k8s_engine = &state.kubernetes_engine;
    let lease_manager = &state.lease_manager;

    let (token, lease) = k8s_engine
        .generate_token_with_lease(&role_name, params.ttl, lease_manager, &user.username)
        .await
        .map_err(|e| ApiError::Internal {
            message: format!("Failed to generate Kubernetes token: {}", e),
        })?;

    // Log audit event
    let audit_entry = create_audit_log(
        "k8s_token_generated",
        &user.username,
        "kubernetes_role",
        &role_name,
    );
    let _ = state.audit.log(audit_entry).await;

    let response = GenerateTokenResponse {
        lease_id: lease.id.clone(),
        lease_duration: (lease.expired_at - lease.issued_at).num_seconds(),
        renewable: lease.renewable,
        data: TokenData {
            token: token.token,
            namespace: token.namespace,
            service_account: token.service_account,
            expires_at: token.expires_at,
            role: token.role_name,
        },
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Request to create/update a Kubernetes role
#[derive(Debug, Deserialize)]
pub struct CreateK8sRoleRequest {
    /// Kubernetes connection name
    pub connection_name: String,

    /// Target namespace
    pub namespace: String,

    /// Service account name
    pub service_account: String,

    /// Default TTL for tokens (in seconds)
    #[serde(default = "default_ttl")]
    pub default_ttl: u32,

    /// Maximum TTL for tokens (in seconds)
    #[serde(default = "default_max_ttl")]
    pub max_ttl: u32,

    /// Additional labels
    #[serde(default)]
    pub labels: std::collections::HashMap<String, String>,

    /// Additional annotations
    #[serde(default)]
    pub annotations: std::collections::HashMap<String, String>,
}

fn default_ttl() -> u32 {
    3600
}
fn default_max_ttl() -> u32 {
    86400
}

/// Response for Kubernetes role operations
#[derive(Debug, Serialize)]
pub struct K8sRoleResponse {
    pub name: String,
    pub connection_name: String,
    pub namespace: String,
    pub service_account: String,
}

/// Create/Update Kubernetes role
async fn upsert_kubernetes_role(
    state: &AppState,
    role_name: String,
    request: CreateK8sRoleRequest,
) -> ApiResult<K8sRoleResponse> {
    let role = KubernetesRole {
        name: role_name.clone(),
        connection_name: request.connection_name.clone(),
        namespace: request.namespace.clone(),
        service_account: request.service_account.clone(),
        default_ttl: request.default_ttl,
        max_ttl: request.max_ttl,
        labels: request.labels,
        annotations: request.annotations,
    };

    state
        .kubernetes_engine
        .create_role(role)
        .await
        .map_err(|e| ApiError::Internal {
            message: format!("Failed to configure Kubernetes role: {}", e),
        })?;

    Ok(K8sRoleResponse {
        name: role_name,
        connection_name: request.connection_name,
        namespace: request.namespace,
        service_account: request.service_account,
    })
}

pub async fn create_kubernetes_role(
    State(state): State<AppState>,
    Path(role_name): Path<String>,
    user: AuthenticatedUser,
    Json(request): Json<CreateK8sRoleRequest>,
) -> ApiResult<Json<ApiResponse<K8sRoleResponse>>> {
    let response = upsert_kubernetes_role(&state, role_name.clone(), request).await?;

    let audit_entry =
        create_audit_log("k8s_role_created", &user.username, "kubernetes_role", &role_name);
    let _ = state.audit.log(audit_entry).await;

    Ok(Json(ApiResponse::success(response)))
}

pub async fn update_kubernetes_role(
    State(state): State<AppState>,
    Path(role_name): Path<String>,
    user: AuthenticatedUser,
    Json(request): Json<CreateK8sRoleRequest>,
) -> ApiResult<Json<ApiResponse<K8sRoleResponse>>> {
    let response = upsert_kubernetes_role(&state, role_name.clone(), request).await?;

    let audit_entry =
        create_audit_log("k8s_role_updated", &user.username, "kubernetes_role", &role_name);
    let _ = state.audit.log(audit_entry).await;

    Ok(Json(ApiResponse::success(response)))
}

pub async fn get_kubernetes_role(
    State(state): State<AppState>,
    Path(role_name): Path<String>,
) -> ApiResult<Json<ApiResponse<K8sRoleResponse>>> {
    let role = state
        .kubernetes_engine
        .get_role(&role_name)
        .await
        .ok_or_else(|| ApiError::NotFound {
            resource: format!("Kubernetes role {} not found", role_name),
        })?;

    Ok(Json(ApiResponse::success(K8sRoleResponse {
        name: role.name,
        connection_name: role.connection_name,
        namespace: role.namespace,
        service_account: role.service_account,
    })))
}

pub async fn delete_kubernetes_role(
    State(state): State<AppState>,
    Path(role_name): Path<String>,
    user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<serde_json::Value>>> {
    let deleted = state.kubernetes_engine.delete_role(&role_name).await;

    if !deleted {
        return Err(ApiError::NotFound {
            resource: format!("Kubernetes role {} not found", role_name),
        });
    }

    // Log audit event
    let audit_entry =
        create_audit_log("k8s_role_deleted", &user.username, "kubernetes_role", &role_name);
    let _ = state.audit.log(audit_entry).await;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "deleted": true,
        "role": role_name
    }))))
}

pub async fn list_kubernetes_roles(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<Vec<String>>>> {
    let roles = state.kubernetes_engine.list_roles().await;
    Ok(Json(ApiResponse::success(roles)))
}

/// Request to configure Kubernetes connection
#[derive(Debug, Deserialize)]
pub struct ConfigureK8sConnectionRequest {
    pub api_server_url: String,
    pub ca_cert: Option<String>,
    pub service_account_token: Option<String>,
    pub client_cert: Option<String>,
    pub client_key: Option<String>,
    #[serde(default = "default_true")]
    pub verify_connection: bool,
}

fn default_true() -> bool {
    true
}

pub async fn configure_kubernetes_connection(
    State(state): State<AppState>,
    Path(name): Path<String>,
    user: AuthenticatedUser,
    Json(request): Json<ConfigureK8sConnectionRequest>,
) -> ApiResult<Json<ApiResponse<serde_json::Value>>> {
    let config = KubernetesConnection {
        name: name.clone(),
        api_server_url: request.api_server_url,
        ca_cert: request.ca_cert,
        service_account_token: request.service_account_token,
        client_cert: request.client_cert,
        client_key: request.client_key,
        verify_connection: request.verify_connection,
    };

    state
        .kubernetes_engine
        .configure_connection(config)
        .await
        .map_err(|e| ApiError::Internal {
            message: format!("Failed to configure Kubernetes connection: {}", e),
        })?;

    let audit_entry = create_audit_log(
        "k8s_connection_configured",
        &user.username,
        "kubernetes_connection",
        &name,
    );
    let _ = state.audit.log(audit_entry).await;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "configured": true,
        "connection": name
    }))))
}

pub async fn get_kubernetes_connection(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> ApiResult<Json<ApiResponse<serde_json::Value>>> {
    let conn = state
        .kubernetes_engine
        .get_connection(&name)
        .await
        .ok_or_else(|| ApiError::NotFound {
            resource: format!("Kubernetes connection {} not found", name),
        })?;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "connection": conn.name,
        "api_server_url": conn.api_server_url,
        "verify_connection": conn.verify_connection
    }))))
}

pub async fn delete_kubernetes_connection(
    State(state): State<AppState>,
    Path(name): Path<String>,
    user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<serde_json::Value>>> {
    let deleted = state.kubernetes_engine.delete_connection(&name).await;

    if !deleted {
        return Err(ApiError::NotFound {
            resource: format!("Kubernetes connection {} not found", name),
        });
    }

    let audit_entry = create_audit_log(
        "k8s_connection_deleted",
        &user.username,
        "kubernetes_connection",
        &name,
    );
    let _ = state.audit.log(audit_entry).await;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "deleted": true,
        "connection": name
    }))))
}
