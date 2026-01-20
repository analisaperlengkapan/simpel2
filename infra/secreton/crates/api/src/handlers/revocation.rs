//! Revocation API handlers
//!
//! REST API endpoints for secret revocation operations.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use secreton_core::services::revocation::{
    RevocationManager, RevocationRequest, RevocationService,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

/// Revocation request payload
#[derive(Debug, Deserialize, Serialize)]
pub struct RevokeSecretRequest {
    /// Reason for revocation
    pub reason: String,

    /// Whether to cascade to dependent secrets
    #[serde(default)]
    pub cascade: bool,

    /// Emergency revocation flag
    #[serde(default)]
    pub emergency: bool,
}

/// Emergency revocation request
#[derive(Debug, Deserialize)]
pub struct EmergencyRevokeRequest {
    /// Pattern to match secrets
    pub pattern: String,

    /// Reason for emergency revocation
    pub reason: String,
}

/// Query parameters for orphan detection
#[derive(Debug, Deserialize)]
pub struct OrphanQueryParams {
    /// Threshold in days (default: 30)
    #[serde(default = "default_threshold")]
    pub threshold_days: u32,
}

fn default_threshold() -> u32 {
    30
}

/// Revoke a secret
///
/// POST /v1/revoke/{path}
pub async fn revoke_secret(
    State(revocation_manager): State<Arc<RevocationManager>>,
    Path(path): Path<String>,
    Json(payload): Json<RevokeSecretRequest>,
) -> impl IntoResponse {
    info!("Revoking secret: {}", path);

    // TODO: Extract actor from authentication context
    let actor = "system".to_string(); // Placeholder

    // TODO: Extract namespace from authentication context
    let namespace = "default".to_string(); // Placeholder

    let request = RevocationRequest {
        path: path.clone(),
        reason: payload.reason,
        cascade: payload.cascade,
        emergency: payload.emergency,
        actor,
        namespace,
    };

    match revocation_manager.revoke(request).await {
        Ok(record) => {
            info!("Successfully revoked secret: {}", path);
            (StatusCode::OK, Json(record)).into_response()
        }
        Err(e) => {
            error!("Failed to revoke secret {}: {}", path, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to revoke secret: {}", e)
                })),
            )
                .into_response()
        }
    }
}

/// Emergency revocation by pattern
///
/// POST /v1/revoke/emergency
pub async fn emergency_revoke(
    State(revocation_manager): State<Arc<RevocationManager>>,
    Json(payload): Json<EmergencyRevokeRequest>,
) -> impl IntoResponse {
    info!("Emergency revocation for pattern: {}", payload.pattern);

    // TODO: Extract actor from authentication context
    let actor = "system".to_string(); // Placeholder

    // TODO: Extract namespace from authentication context
    let namespace = "default".to_string(); // Placeholder

    match revocation_manager
        .emergency_revoke(&payload.pattern, &actor, &namespace)
        .await
    {
        Ok(records) => {
            info!(
                "Successfully revoked {} secrets matching pattern: {}",
                records.len(),
                payload.pattern
            );
            (StatusCode::OK, Json(records)).into_response()
        }
        Err(e) => {
            error!(
                "Failed to emergency revoke pattern {}: {}",
                payload.pattern, e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to emergency revoke: {}", e)
                })),
            )
                .into_response()
        }
    }
}

/// Get revocation history for a secret
///
/// GET /v1/revoke/history/{path}
pub async fn get_revocation_history(
    State(revocation_manager): State<Arc<RevocationManager>>,
    Path(path): Path<String>,
) -> impl IntoResponse {
    info!("Getting revocation history for: {}", path);

    match revocation_manager.get_history(&path).await {
        Ok(history) => {
            info!("Retrieved {} revocation records for {}", history.len(), path);
            (StatusCode::OK, Json(history)).into_response()
        }
        Err(e) => {
            error!("Failed to get revocation history for {}: {}", path, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to get revocation history: {}", e)
                })),
            )
                .into_response()
        }
    }
}

/// Detect orphaned secrets
///
/// GET /v1/secrets/orphans
pub async fn detect_orphans(
    State(revocation_manager): State<Arc<RevocationManager>>,
    Query(params): Query<OrphanQueryParams>,
) -> impl IntoResponse {
    info!("Detecting orphaned secrets with threshold: {} days", params.threshold_days);

    match revocation_manager.detect_orphans(params.threshold_days).await {
        Ok(orphans) => {
            info!("Detected {} orphaned secrets", orphans.len());
            (StatusCode::OK, Json(orphans)).into_response()
        }
        Err(e) => {
            error!("Failed to detect orphaned secrets: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to detect orphaned secrets: {}", e)
                })),
            )
                .into_response()
        }
    }
}

/// Get revocation statistics
///
/// GET /v1/revoke/stats
pub async fn get_revocation_stats(
    State(revocation_manager): State<Arc<RevocationManager>>,
) -> impl IntoResponse {
    info!("Getting revocation statistics");

    match revocation_manager.get_stats().await {
        Ok(stats) => {
            info!("Retrieved revocation statistics");
            (StatusCode::OK, Json(stats)).into_response()
        }
        Err(e) => {
            error!("Failed to get revocation statistics: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to get revocation statistics: {}", e)
                })),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_revoke_request_deserialization() {
        let json = r#"{"reason": "Test revocation", "cascade": true, "emergency": false}"#;
        let request: RevokeSecretRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.reason, "Test revocation");
        assert!(request.cascade);
        assert!(!request.emergency);
    }

    #[test]
    fn test_revoke_request_defaults() {
        let json = r#"{"reason": "Test revocation"}"#;
        let request: RevokeSecretRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.reason, "Test revocation");
        assert!(!request.cascade);
        assert!(!request.emergency);
    }

    #[test]
    fn test_emergency_revoke_request_deserialization() {
        let json = r#"{"pattern": "/secret/test/*", "reason": "Security incident"}"#;
        let request: EmergencyRevokeRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.pattern, "/secret/test/*");
        assert_eq!(request.reason, "Security incident");
    }

    #[test]
    fn test_orphan_query_params_default() {
        let params = OrphanQueryParams {
            threshold_days: default_threshold(),
        };
        assert_eq!(params.threshold_days, 30);
    }
}

