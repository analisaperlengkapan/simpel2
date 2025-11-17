//! SSH API handlers
//!
//! REST API endpoints for SSH secrets engine operations.

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
use secreton_core::services::secrets::ssh::{
    SshCertificate, SshCertificateRequest, SshKeyPair, SshKeyType, SshRole,
};

use super::AppState;

/// Create SSH routes
pub fn create_routes() -> Router<AppState> {
    Router::new()
        // Role management
        .route("/role", post(create_role))
        .route("/role", get(list_roles))
        .route("/role/:name", get(get_role))
        // CA management
        .route("/ca", post(create_ca))
        .route("/ca", get(list_cas))
        .route("/ca/:name/public_key", get(get_ca_public_key))
        // Key generation
        .route("/creds/:role", post(generate_keypair))
        // Certificate signing
        .route("/sign/:ca/:role", post(sign_certificate))
        // OTP operations
        .route("/otp/generate", post(generate_otp))
        .route("/otp/verify", post(verify_otp))
}

/// Request to create role
#[derive(Debug, Deserialize)]
pub struct CreateRoleRequest {
    pub name: String,
    pub key_type: SshKeyType,
    pub default_user: String,
    pub allowed_users: Option<Vec<String>>,
    pub default_ttl: Option<i64>,
    pub max_ttl: Option<i64>,
}

/// Request to create CA
#[derive(Debug, Deserialize)]
pub struct CreateCaRequest {
    pub name: String,
    pub key_type: SshKeyType,
}

/// Request to generate OTP
#[derive(Debug, Deserialize)]
pub struct GenerateOtpRequest {
    pub username: String,
    pub ip: String,
    pub ttl: Option<i64>,
}

/// Response for OTP generation
#[derive(Debug, Serialize)]
pub struct GenerateOtpResponse {
    pub otp: String,
}

/// Request to verify OTP
#[derive(Debug, Deserialize)]
pub struct VerifyOtpRequest {
    pub otp: String,
    pub username: String,
    pub ip: String,
}

/// Response for OTP verification
#[derive(Debug, Serialize)]
pub struct VerifyOtpResponse {
    pub valid: bool,
}

/// Create SSH role
///
/// # Endpoint
/// `POST /v1/ssh/role`
#[tracing::instrument(skip(state))]
async fn create_role(
    State(state): State<AppState>,
    Json(request): Json<CreateRoleRequest>,
) -> ApiResult<Json<ApiResponse<SshRole>>> {
    info!("Creating SSH role: {}", request.name);

    let mut role = SshRole::new(request.name.clone(), request.key_type, request.default_user);

    if let Some(allowed_users) = request.allowed_users {
        role.allowed_users = allowed_users;
    }
    if let Some(default_ttl) = request.default_ttl {
        role.default_ttl = default_ttl;
    }
    if let Some(max_ttl) = request.max_ttl {
        role.max_ttl = max_ttl;
    }

    match state.ssh_engine.create_role(role.clone()).await {
        Ok(_) => {
            info!("SSH role created successfully");
            Ok(Json(ApiResponse::success(role)))
        }
        Err(e) => {
            error!("Failed to create SSH role: {:?}", e);
            Err(ApiError::bad_request(format!(
                "Failed to create SSH role: {}",
                e
            )))
        }
    }
}

/// Get SSH role
///
/// # Endpoint
/// `GET /v1/ssh/role/:name`
#[tracing::instrument(skip(state))]
async fn get_role(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> ApiResult<Json<ApiResponse<SshRole>>> {
    info!("Getting SSH role: {}", name);

    match state.ssh_engine.get_role(&name).await {
        Some(role) => Ok(Json(ApiResponse::success(role))),
        None => {
            error!("SSH role not found: {}", name);
            Err(ApiError::not_found("SSH role not found".to_string()))
        }
    }
}

/// List SSH roles
///
/// # Endpoint
/// `GET /v1/ssh/role`
#[tracing::instrument(skip(state))]
async fn list_roles(State(state): State<AppState>) -> ApiResult<Json<ApiResponse<Vec<String>>>> {
    info!("Listing SSH roles");

    let roles = state.ssh_engine.list_roles().await;
    Ok(Json(ApiResponse::success(roles)))
}

/// Create SSH CA
///
/// # Endpoint
/// `POST /v1/ssh/ca`
#[tracing::instrument(skip(state))]
async fn create_ca(
    State(state): State<AppState>,
    Json(request): Json<CreateCaRequest>,
) -> ApiResult<Json<ApiResponse<()>>> {
    info!("Creating SSH CA: {}", request.name);

    match state
        .ssh_engine
        .create_ca(request.name.clone(), request.key_type)
        .await
    {
        Ok(_) => {
            info!("SSH CA created successfully");
            Ok(Json(ApiResponse::success(())))
        }
        Err(e) => {
            error!("Failed to create SSH CA: {:?}", e);
            Err(ApiError::bad_request(format!(
                "Failed to create SSH CA: {}",
                e
            )))
        }
    }
}

/// List SSH CAs
///
/// # Endpoint
/// `GET /v1/ssh/ca`
#[tracing::instrument(skip(state))]
async fn list_cas(State(state): State<AppState>) -> ApiResult<Json<ApiResponse<Vec<String>>>> {
    info!("Listing SSH CAs");

    let cas = state.ssh_engine.list_cas().await;
    Ok(Json(ApiResponse::success(cas)))
}

/// Get CA public key
///
/// # Endpoint
/// `GET /v1/ssh/ca/:name/public_key`
#[tracing::instrument(skip(state))]
async fn get_ca_public_key(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> ApiResult<Json<ApiResponse<String>>> {
    info!("Getting CA public key: {}", name);

    match state.ssh_engine.get_ca_public_key(&name).await {
        Ok(public_key) => Ok(Json(ApiResponse::success(public_key))),
        Err(e) => {
            error!("Failed to get CA public key: {:?}", e);
            Err(ApiError::not_found(format!(
                "Failed to get CA public key: {}",
                e
            )))
        }
    }
}

/// Generate SSH keypair
///
/// # Endpoint
/// `POST /v1/ssh/creds/:role`
#[tracing::instrument(skip(state))]
async fn generate_keypair(
    State(state): State<AppState>,
    Path(role): Path<String>,
) -> ApiResult<Json<ApiResponse<SshKeyPair>>> {
    info!("Generating SSH keypair for role: {}", role);

    match state.ssh_engine.generate_keypair(&role).await {
        Ok(keypair) => {
            info!("SSH keypair generated successfully");
            Ok(Json(ApiResponse::success(keypair)))
        }
        Err(e) => {
            error!("Failed to generate SSH keypair: {:?}", e);
            Err(ApiError::bad_request(format!(
                "Failed to generate SSH keypair: {}",
                e
            )))
        }
    }
}

/// Sign SSH certificate
///
/// # Endpoint
/// `POST /v1/ssh/sign/:ca/:role`
#[tracing::instrument(skip(state, request), fields(ca = %ca, role = %role))]
async fn sign_certificate(
    State(state): State<AppState>,
    Path((ca, role)): Path<(String, String)>,
    Json(request): Json<SshCertificateRequest>,
) -> ApiResult<Json<ApiResponse<SshCertificate>>> {
    info!("Signing SSH certificate with CA: {}, role: {}", ca, role);

    match state.ssh_engine.sign_certificate(&ca, &role, request).await {
        Ok(certificate) => {
            info!("SSH certificate signed successfully");
            Ok(Json(ApiResponse::success(certificate)))
        }
        Err(e) => {
            error!("Failed to sign SSH certificate: {:?}", e);
            Err(ApiError::bad_request(format!(
                "Failed to sign SSH certificate: {}",
                e
            )))
        }
    }
}

/// Generate SSH OTP
///
/// # Endpoint
/// `POST /v1/ssh/otp/generate`
#[tracing::instrument(skip(state))]
async fn generate_otp(
    State(state): State<AppState>,
    Json(request): Json<GenerateOtpRequest>,
) -> ApiResult<Json<ApiResponse<GenerateOtpResponse>>> {
    info!("Generating SSH OTP for user: {}", request.username);

    let ttl = request.ttl.unwrap_or(300); // Default 5 minutes

    match state
        .ssh_engine
        .generate_otp(&request.username, &request.ip, ttl)
        .await
    {
        Ok(otp) => {
            info!("SSH OTP generated successfully");
            Ok(Json(ApiResponse::success(GenerateOtpResponse { otp })))
        }
        Err(e) => {
            error!("Failed to generate SSH OTP: {:?}", e);
            Err(ApiError::bad_request(format!(
                "Failed to generate SSH OTP: {}",
                e
            )))
        }
    }
}

/// Verify SSH OTP
///
/// # Endpoint
/// `POST /v1/ssh/otp/verify`
#[tracing::instrument(skip(state, request))]
async fn verify_otp(
    State(state): State<AppState>,
    Json(request): Json<VerifyOtpRequest>,
) -> ApiResult<Json<ApiResponse<VerifyOtpResponse>>> {
    info!("Verifying SSH OTP for user: {}", request.username);

    match state
        .ssh_engine
        .verify_otp(&request.otp, &request.username, &request.ip)
        .await
    {
        Ok(valid) => {
            info!("SSH OTP verification result: {}", valid);
            Ok(Json(ApiResponse::success(VerifyOtpResponse { valid })))
        }
        Err(e) => {
            error!("Failed to verify SSH OTP: {:?}", e);
            Err(ApiError::bad_request(format!(
                "Failed to verify SSH OTP: {}",
                e
            )))
        }
    }
}
