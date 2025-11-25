//! LDAP Secrets Engine API handlers

use axum::{
    Router,
    extract::{Path, State},
    response::Json,
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::{ApiError, ApiResponse, ApiResult};
use secreton_core::services::secrets::ldap::{
    LdapConfig, LdapCredential, LdapCredentialInfo, LdapRole, LdapSchema,
};

use super::AppState;

/// Create LDAP routes
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/config", post(configure_ldap))
        .route("/config", get(get_ldap_config))
        .route("/roles", post(create_role))
        .route("/roles", get(list_roles))
        .route("/roles/:role_name", get(get_role))
        .route("/roles/:role_name", delete(delete_role))
        .route("/creds/:role_name", post(generate_credentials))
        .route("/creds/:username/rotate", post(rotate_password))
        .route("/creds/:username/revoke", post(revoke_credentials))
        .route("/creds", get(list_credentials))
        .route("/creds/:username", get(get_credential_info))
}

/// Configure LDAP connection
#[tracing::instrument(skip(state, request))]
async fn configure_ldap(
    State(state): State<AppState>,
    Json(request): Json<LdapConfig>,
) -> ApiResult<Json<ApiResponse<()>>> {
    info!("Configuring LDAP secrets engine");

    match state.ldap_engine.configure(request).await {
        Ok(_) => {
            info!("LDAP configured successfully");
            Ok(Json(ApiResponse::success(())))
        }
        Err(e) => {
            error!("Failed to configure LDAP: {:?}", e);
            Err(ApiError::bad_request(format!(
                "Failed to configure LDAP: {}",
                e
            )))
        }
    }
}

/// Get LDAP configuration
#[tracing::instrument(skip(state))]
async fn get_ldap_config(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<LdapConfigResponse>>> {
    info!("Getting LDAP configuration");

    match state.ldap_engine.get_config().await {
        Ok(config) => {
            let response = LdapConfigResponse {
                url: config.url,
                bind_dn: config.bind_dn,
                user_dn: config.user_dn,
                group_dn: config.group_dn,
                use_tls: config.use_tls,
                schema: config.schema,
                user_object_class: config.user_object_class,
            };
            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            error!("Failed to get LDAP config: {:?}", e);
            Err(ApiError::not_found(format!("LDAP not configured: {}", e)))
        }
    }
}

/// Create role
#[tracing::instrument(skip(state))]
async fn create_role(
    State(state): State<AppState>,
    Json(role): Json<LdapRole>,
) -> ApiResult<Json<ApiResponse<()>>> {
    info!("Creating LDAP role: {}", role.name);

    match state.ldap_engine.create_role(role).await {
        Ok(_) => {
            info!("LDAP role created successfully");
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
) -> ApiResult<Json<ApiResponse<LdapRole>>> {
    info!("Getting LDAP role: {}", role_name);

    match state.ldap_engine.get_role(&role_name).await {
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
    info!("Listing LDAP roles");

    let roles = state.ldap_engine.list_roles().await;
    Ok(Json(ApiResponse::success(roles)))
}

/// Delete role
#[tracing::instrument(skip(state))]
async fn delete_role(
    State(state): State<AppState>,
    Path(role_name): Path<String>,
) -> ApiResult<Json<ApiResponse<()>>> {
    info!("Deleting LDAP role: {}", role_name);

    match state.ldap_engine.delete_role(&role_name).await {
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
) -> ApiResult<Json<ApiResponse<LdapCredential>>> {
    info!("Generating LDAP credentials for role: {}", role_name);

    match state
        .ldap_engine
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

/// Rotate password
#[tracing::instrument(skip(state))]
async fn rotate_password(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> ApiResult<Json<ApiResponse<LdapCredential>>> {
    info!("Rotating password for LDAP user: {}", username);

    match state.ldap_engine.rotate_password(&username).await {
        Ok(cred) => {
            info!("Password rotated successfully");
            Ok(Json(ApiResponse::success(cred)))
        }
        Err(e) => {
            error!("Failed to rotate password: {:?}", e);
            Err(ApiError::bad_request(format!(
                "Failed to rotate password: {}",
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
    info!("Revoking LDAP credentials: {}", username);

    match state.ldap_engine.revoke_credentials(&username).await {
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
    info!("Listing LDAP credentials");

    let creds = state.ldap_engine.list_credentials().await;
    Ok(Json(ApiResponse::success(creds)))
}

/// Get credential info
#[tracing::instrument(skip(state))]
async fn get_credential_info(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> ApiResult<Json<ApiResponse<LdapCredentialInfo>>> {
    info!("Getting LDAP credential info: {}", username);

    match state.ldap_engine.get_credential_info(&username).await {
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
pub struct LdapConfigResponse {
    pub url: String,
    pub bind_dn: String,
    pub user_dn: String,
    pub group_dn: Option<String>,
    pub use_tls: bool,
    pub schema: LdapSchema,
    pub user_object_class: String,
}
