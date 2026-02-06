//! Configuration Management API Routes
//!
//! Provides REST API endpoints for managing application configuration:
//! - GET /admin/config - View all configuration
//! - GET /admin/config/{key} - Get specific configuration
//! - GET /admin/config/category/{category} - Get category configuration
//! - PUT /admin/config/{key} - Update configuration
//! - DELETE /admin/config/{key} - Delete configuration
//! - POST /admin/config/reload - Hot reload configuration
//! - GET /admin/config/{key}/history - View change history
//!
//! ## Authorization
//!
//! All endpoints require capability-based authorization:
//! - `config:read` - Required for viewing configuration
//! - `config:write` - Required for updating configuration
//! - `config:delete` - Required for deleting configuration
//! - `config:reload` - Required for hot reload

use crate::error::{AuthencError, Result};
use crate::extractors::AuthenticatedUser;
use crate::services::authorization::{CapabilityChecker, capabilities};
use crate::services::config_manager::ConfigManager;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Clone)]
pub struct ConfigState {
    pub config_manager: Arc<ConfigManager>,
    pub capability_checker: Arc<CapabilityChecker>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigRequest {
    pub value: Value,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigResponse {
    pub key: String,
    pub value: Value,
    pub category: Option<String>,
    pub is_secret: bool,
    pub updated_at: Option<String>,
}

/// Create configuration routes
///
/// # Arguments
/// * `config_manager` - Configuration manager instance
/// * `capability_checker` - Capability checker for authorization
pub fn create_routes(
    config_manager: Arc<ConfigManager>,
    capability_checker: Arc<CapabilityChecker>,
) -> Router {
    let state = ConfigState {
        config_manager,
        capability_checker,
    };

    Router::new()
        // Get all configuration
        .route("/", get(get_all_config))
        // Get configuration by category
        .route("/category/{category}", get(get_category_config))
        // Get specific configuration
        .route("/{key}", get(get_config))
        // Get configuration history
        .route("/{key}/history", get(get_config_history))
        // Update configuration
        .route("/{key}", put(update_config))
        // Delete configuration
        .route("/{key}", delete(delete_config))
        // Hot reload configuration
        .route("/reload", post(reload_config))
        .with_state(state)
}

/// Get all configuration
async fn get_all_config(
    State(state): State<ConfigState>,
    AuthenticatedUser { user, .. }: AuthenticatedUser,
) -> Result<impl IntoResponse> {
    // Check capability (replaces hardcoded role check)
    let user_id = Uuid::parse_str(&user.id)
        .map_err(|_| AuthencError::bad_request("Invalid user ID"))?;
    state
        .capability_checker
        .require_capability(&user_id, capabilities::CONFIG_READ)
        .await?;

    let config = state.config_manager.get_all().await?;

    Ok(Json(json!({
        "status": "success",
        "data": config
    })))
}

/// Get configuration by category
async fn get_category_config(
    State(state): State<ConfigState>,
    Path(category): Path<String>,
    AuthenticatedUser { user, .. }: AuthenticatedUser,
) -> Result<impl IntoResponse> {
    // Check capability
    let user_id = Uuid::parse_str(&user.id)
        .map_err(|_| AuthencError::bad_request("Invalid user ID"))?;
    state
        .capability_checker
        .require_capability(&user_id, capabilities::CONFIG_READ)
        .await?;

    let config = state.config_manager.get_category(&category).await?;

    Ok(Json(json!({
        "status": "success",
        "category": category,
        "data": config
    })))
}

/// Get specific configuration
async fn get_config(
    State(state): State<ConfigState>,
    Path(key): Path<String>,
    AuthenticatedUser { user, .. }: AuthenticatedUser,
) -> Result<impl IntoResponse> {
    // Check capability
    let user_id = Uuid::parse_str(&user.id)
        .map_err(|_| AuthencError::bad_request("Invalid user ID"))?;
    state
        .capability_checker
        .require_capability(&user_id, capabilities::CONFIG_READ)
        .await?;

    let value = state.config_manager.get(&key).await?;

    Ok(Json(json!({
        "status": "success",
        "key": key,
        "value": value
    })))
}

/// Update configuration
async fn update_config(
    State(state): State<ConfigState>,
    Path(key): Path<String>,
    AuthenticatedUser { user, .. }: AuthenticatedUser,
    Json(req): Json<ConfigRequest>,
) -> Result<impl IntoResponse> {
    // Check capability
    let user_id = Uuid::parse_str(&user.id)
        .map_err(|_| AuthencError::bad_request("Invalid user ID"))?;
    state
        .capability_checker
        .require_capability(&user_id, capabilities::CONFIG_WRITE)
        .await?;

    // Prevent modification of system configuration without explicit approval
    if key.starts_with("system.") {
        warn!("Attempted to modify system configuration: {}", key);
        return Err(AuthencError::forbidden(
            "System configuration cannot be modified",
        ));
    }

    state
        .config_manager
        .set(
            &key,
            req.value.clone(),
            Some(&user.id),
            req.reason.as_deref(),
        )
        .await?;

    info!(
        "Configuration updated by user {}: {} = {}",
        user.id, key, req.value
    );

    Ok((
        StatusCode::OK,
        Json(json!({
            "status": "success",
            "message": format!("Configuration '{}' updated", key),
            "key": key,
            "value": req.value
        })),
    ))
}

/// Delete configuration
async fn delete_config(
    State(state): State<ConfigState>,
    Path(key): Path<String>,
    AuthenticatedUser { user, .. }: AuthenticatedUser,
) -> Result<impl IntoResponse> {
    // Check capability
    let user_id = Uuid::parse_str(&user.id)
        .map_err(|_| AuthencError::bad_request("Invalid user ID"))?;
    state
        .capability_checker
        .require_capability(&user_id, capabilities::CONFIG_DELETE)
        .await?;

    // Prevent deletion of system configuration
    if key.starts_with("system.") {
        warn!("Attempted to delete system configuration: {}", key);
        return Err(AuthencError::forbidden(
            "System configuration cannot be deleted",
        ));
    }

    state
        .config_manager
        .delete(&key, Some(&user.id), Some("Deleted via API"))
        .await?;

    info!("Configuration deleted by user {}: {}", user.id, key);

    Ok((
        StatusCode::OK,
        Json(json!({
            "status": "success",
            "message": format!("Configuration '{}' deleted", key)
        })),
    ))
}

/// Hot reload configuration
async fn reload_config(
    State(state): State<ConfigState>,
    AuthenticatedUser { user, .. }: AuthenticatedUser,
) -> Result<impl IntoResponse> {
    // Check capability
    let user_id = Uuid::parse_str(&user.id)
        .map_err(|_| AuthencError::bad_request("Invalid user ID"))?;
    state
        .capability_checker
        .require_capability(&user_id, capabilities::CONFIG_RELOAD)
        .await?;

    state.config_manager.reload_all().await?;

    info!("Configuration reloaded by user {}", user.id);

    Ok((
        StatusCode::OK,
        Json(json!({
            "status": "success",
            "message": "Configuration reloaded successfully"
        })),
    ))
}

/// Get configuration change history
async fn get_config_history(
    State(state): State<ConfigState>,
    Path(key): Path<String>,
    AuthenticatedUser { user, .. }: AuthenticatedUser,
) -> Result<impl IntoResponse> {
    // Check capability (audit read for history)
    let user_id = Uuid::parse_str(&user.id)
        .map_err(|_| AuthencError::bad_request("Invalid user ID"))?;
    state
        .capability_checker
        .require_any_capability(&user_id, &[capabilities::CONFIG_READ, capabilities::AUDIT_READ])
        .await?;

    let history = state.config_manager.get_history(&key, 50).await?;

    Ok(Json(json!({
        "status": "success",
        "key": key,
        "history": history
    })))
}
