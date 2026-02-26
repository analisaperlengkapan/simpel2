//! Device management handlers
//!
//! Handles device registration, management, and trust for authentication flows.

use axum::{
    Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::state::ApiState;

// ===== Types =====

/// Request to register a new device
#[derive(Debug, Deserialize)]
pub struct RegisterDeviceRequest {
    pub device_name: Option<String>,
    pub device_type: Option<String>,
    pub fingerprint: Option<String>,
}

/// Response for device registration
#[derive(Debug, Serialize)]
pub struct DeviceResponse {
    pub device_id: String,
    pub device_name: Option<String>,
    pub device_type: Option<String>,
    pub trusted: bool,
    pub created_at: String,
    pub last_used_at: Option<String>,
}

/// Request to update device trust status
#[derive(Debug, Deserialize)]
pub struct UpdateDeviceTrustRequest {
    pub trusted: bool,
}

/// Response for device listing
#[derive(Debug, Serialize)]
pub struct DeviceListResponse {
    pub devices: Vec<DeviceResponse>,
    pub total: usize,
}

// ===== Routes =====

/// Create device management routes
pub fn create_device_routes() -> Router<Arc<ApiState>> {
    Router::new()
        .route("/devices", get(list_devices).post(register_device))
        .route(
            "/devices/{id}",
            get(get_device).put(update_device).delete(remove_device),
        )
        .route("/devices/{id}/trust", post(update_device_trust))
}

// ===== Handlers =====

/// List all registered devices for the current user
async fn list_devices(State(_state): State<Arc<ApiState>>) -> impl IntoResponse {
    // TODO: Query device store for user's devices
    Json(DeviceListResponse {
        devices: vec![],
        total: 0,
    })
}

/// Register a new device
async fn register_device(
    State(_state): State<Arc<ApiState>>,
    Json(_req): Json<RegisterDeviceRequest>,
) -> impl IntoResponse {
    // TODO: Create device record in storage
    Json(serde_json::json!({
        "status": "ok",
        "message": "Device registration (stub)"
    }))
}

/// Get a specific device by ID
async fn get_device(
    State(_state): State<Arc<ApiState>>,
    Path(_id): Path<String>,
) -> impl IntoResponse {
    // TODO: Look up device by ID
    Json(serde_json::json!({
        "status": "not_found",
        "message": "Device not found (stub)"
    }))
}

/// Update a device
async fn update_device(
    State(_state): State<Arc<ApiState>>,
    Path(_id): Path<String>,
    Json(_body): Json<serde_json::Value>,
) -> impl IntoResponse {
    // TODO: Update device record
    Json(serde_json::json!({
        "status": "ok",
        "message": "Device updated (stub)"
    }))
}

/// Remove a device
async fn remove_device(
    State(_state): State<Arc<ApiState>>,
    Path(_id): Path<String>,
) -> impl IntoResponse {
    // TODO: Delete device from storage
    Json(serde_json::json!({
        "status": "ok",
        "message": "Device removed (stub)"
    }))
}

/// Update the trust status of a device
async fn update_device_trust(
    State(_state): State<Arc<ApiState>>,
    Path(_id): Path<String>,
    Json(_req): Json<UpdateDeviceTrustRequest>,
) -> impl IntoResponse {
    // TODO: Update device trust level
    Json(serde_json::json!({
        "status": "ok",
        "message": "Device trust updated (stub)"
    }))
}
