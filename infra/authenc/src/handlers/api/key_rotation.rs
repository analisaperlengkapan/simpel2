//! Key rotation API handlers
//!
//! This module provides HTTP API endpoints for managing automatic key rotation.

use crate::error::{AuthencError, Result};
use crate::services::key_rotation::{KeyRotationEvent, KeyType};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Request to manually rotate a key
#[derive(Debug, Deserialize)]
pub struct RotateKeyRequest {
    /// Key identifier
    pub key_id: String,
    /// Key type
    pub key_type: KeyType,
}

/// Response from key rotation
#[derive(Debug, Serialize)]
pub struct RotateKeyResponse {
    /// Success status
    pub success: bool,
    /// Message
    pub message: String,
    /// New version number
    pub new_version: Option<u32>,
}

/// Request to register a key for automatic rotation
#[derive(Debug, Deserialize)]
pub struct RegisterKeyRequest {
    /// Key identifier
    pub key_id: String,
    /// Key type
    pub key_type: KeyType,
    /// Current version
    pub current_version: u32,
}

/// Response from key registration
#[derive(Debug, Serialize)]
pub struct RegisterKeyResponse {
    /// Success status
    pub success: bool,
    /// Message
    pub message: String,
}

/// Query parameters for rotation history
#[derive(Debug, Deserialize)]
pub struct RotationHistoryQuery {
    /// Maximum number of events to return
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    10
}

/// Response containing rotation history
#[derive(Debug, Serialize)]
pub struct RotationHistoryResponse {
    /// List of rotation events
    pub events: Vec<KeyRotationEvent>,
    /// Total count
    pub total: usize,
}

/// Manually rotate a key
///
/// POST /admin/keys/rotate
pub async fn rotate_key_handler(
    State(state): State<Arc<crate::app::AppState>>,
    Json(request): Json<RotateKeyRequest>,
) -> Result<impl IntoResponse> {
    // Check if key rotation service is available
    let key_rotation_service =
        state
            .key_rotation_service
            .as_ref()
            .ok_or_else(|| AuthencError::ResourceNotFound {
                resource: "Key rotation service not configured".to_string(),
            })?;

    // Perform rotation
    match key_rotation_service
        .rotate_key(&request.key_id, &request.key_type)
        .await
    {
        Ok(()) => {
            // Get the new version
            let new_version = key_rotation_service
                .get_key_version(&request.key_id)
                .await
                .ok();

            Ok((
                StatusCode::OK,
                Json(RotateKeyResponse {
                    success: true,
                    message: format!("Key {} rotated successfully", request.key_id),
                    new_version,
                }),
            ))
        }
        Err(e) => Ok((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(RotateKeyResponse {
                success: false,
                message: format!("Key rotation failed: {}", e),
                new_version: None,
            }),
        )),
    }
}

/// Register a key for automatic rotation
///
/// POST /admin/keys/register
pub async fn register_key_handler(
    State(state): State<Arc<crate::app::AppState>>,
    Json(request): Json<RegisterKeyRequest>,
) -> Result<impl IntoResponse> {
    // Check if key rotation service is available
    let key_rotation_service =
        state
            .key_rotation_service
            .as_ref()
            .ok_or_else(|| AuthencError::ResourceNotFound {
                resource: "Key rotation service not configured".to_string(),
            })?;

    // Register the key
    key_rotation_service
        .register_key(
            request.key_id.clone(),
            request.key_type,
            request.current_version,
        )
        .await?;

    Ok((
        StatusCode::OK,
        Json(RegisterKeyResponse {
            success: true,
            message: format!("Key {} registered for automatic rotation", request.key_id),
        }),
    ))
}

/// Unregister a key from automatic rotation
///
/// DELETE /admin/keys/{key_id}/register
pub async fn unregister_key_handler(
    State(state): State<Arc<crate::app::AppState>>,
    Path(key_id): Path<String>,
) -> Result<impl IntoResponse> {
    // Check if key rotation service is available
    let key_rotation_service =
        state
            .key_rotation_service
            .as_ref()
            .ok_or_else(|| AuthencError::ResourceNotFound {
                resource: "Key rotation service not configured".to_string(),
            })?;

    // Unregister the key
    key_rotation_service.unregister_key(&key_id).await?;

    Ok((
        StatusCode::OK,
        Json(RegisterKeyResponse {
            success: true,
            message: format!("Key {} unregistered from automatic rotation", key_id),
        }),
    ))
}

/// Get rotation history for a key
///
/// GET /admin/keys/{key_id}/history
pub async fn get_rotation_history_handler(
    State(state): State<Arc<crate::app::AppState>>,
    Path(key_id): Path<String>,
    Query(query): Query<RotationHistoryQuery>,
) -> Result<impl IntoResponse> {
    // Check if key rotation service is available
    let key_rotation_service =
        state
            .key_rotation_service
            .as_ref()
            .ok_or_else(|| AuthencError::ResourceNotFound {
                resource: "Key rotation service not configured".to_string(),
            })?;

    // Get rotation history
    let events = key_rotation_service
        .get_rotation_history(&key_id, query.limit)
        .await?;

    let total = events.len();

    Ok((
        StatusCode::OK,
        Json(RotationHistoryResponse { events, total }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_limit() {
        assert_eq!(default_limit(), 10);
    }

    #[test]
    fn test_rotate_key_request_deserialization() {
        let json = r#"{"key_id": "test_key", "key_type": "JwtSigning"}"#;
        let request: RotateKeyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.key_id, "test_key");
        assert_eq!(request.key_type, KeyType::JwtSigning);
    }
}
