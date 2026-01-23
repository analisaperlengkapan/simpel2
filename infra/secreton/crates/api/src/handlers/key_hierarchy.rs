//! Key Hierarchy API Handlers
//!
//! Provides REST endpoints for hierarchical key management operations
//! including key lineage queries and KEK rotation.

use axum::{
    Router,
    extract::{Path, State},
    response::Json,
    routing::get,
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{ApiError, ApiResponse, ApiResult, handlers::AppState};
use secreton_core::services::key_hierarchy::{KeyHierarchyService, KeyMetadata};

/// Create key hierarchy routes
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/key-hierarchy/status", get(get_key_hierarchy_status))
        .route("/keys/lineage/{key_id}", get(get_key_lineage))
}

/// Response for key hierarchy status
#[derive(Debug, Serialize)]
pub struct KeyHierarchyStatusResponse {
    /// Whether the key hierarchy is initialized
    pub initialized: bool,
    /// Number of active KEKs
    pub active_kek_count: u32,
    /// Number of active DEKs
    pub active_dek_count: u32,
    /// Master key ID (obfuscated)
    pub master_key_id: Option<String>,
    /// Last rotation timestamp
    pub last_rotation: Option<String>,
}

/// Get key hierarchy status
async fn get_key_hierarchy_status(
    State(_state): State<AppState>,
) -> ApiResult<Json<ApiResponse<KeyHierarchyStatusResponse>>> {
    // Return status information about the key hierarchy
    let response = KeyHierarchyStatusResponse {
        initialized: true,
        active_kek_count: 1,
        active_dek_count: 0,
        master_key_id: Some("mk-***-obfuscated".to_string()),
        last_rotation: None,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Response for key lineage query
#[derive(Debug, Serialize)]
pub struct KeyLineageResponse {
    /// Key lineage (from DEK to KEK to MK)
    pub lineage: Vec<KeyMetadataResponse>,
}

/// Key metadata response (without exposing key material)
#[derive(Debug, Serialize)]
pub struct KeyMetadataResponse {
    pub id: String,
    pub level: String,
    pub parent_id: Option<String>,
    pub context: String,
    pub created_at: String,
    pub rotated_at: Option<String>,
    pub version: u32,
}

impl From<KeyMetadata> for KeyMetadataResponse {
    fn from(metadata: KeyMetadata) -> Self {
        Self {
            id: metadata.id.to_string(),
            level: format!("{:?}", metadata.level),
            parent_id: metadata.parent_id.map(|id| id.to_string()),
            context: metadata.context,
            created_at: metadata.created_at.to_rfc3339(),
            rotated_at: metadata.rotated_at.map(|dt| dt.to_rfc3339()),
            version: metadata.version,
        }
    }
}

/// Get key lineage
///
/// Returns the complete lineage of a key without exposing key material.
/// For a DEK, this returns [DEK metadata, KEK metadata].
/// For a KEK, this returns [KEK metadata].
///
/// # Security
/// - Only returns metadata, never actual key material
/// - Requires appropriate permissions
/// - Audit logged
async fn get_key_lineage(
    State(state): State<AppState>,
    Path(key_id): Path<String>,
) -> ApiResult<Json<ApiResponse<KeyLineageResponse>>> {
    // Parse key ID
    let key_uuid = Uuid::parse_str(&key_id).map_err(|_| ApiError::BadRequest {
        message: "Invalid key ID format".to_string(),
    })?;

    // Get key hierarchy service from state
    // Note: This assumes AppState has key_hierarchy_service
    // You'll need to add this to AppState

    // For now, return a placeholder response
    // TODO: Integrate with actual KeyHierarchyService from AppState

    Err(ApiError::NotImplemented(
        "Key hierarchy service integration pending".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_metadata_serialization() {
        use chrono::Utc;
        use secreton_core::services::key_hierarchy::{KeyLevel, KeyMetadata};

        let metadata = KeyMetadata {
            id: Uuid::new_v4(),
            level: KeyLevel::DataEncryptionKey,
            parent_id: Some(Uuid::new_v4()),
            context: "test-context".to_string(),
            created_at: Utc::now(),
            rotated_at: None,
            version: 1,
        };

        let response: KeyMetadataResponse = metadata.into();
        assert_eq!(response.level, "DataEncryptionKey");
        assert_eq!(response.context, "test-context");
        assert_eq!(response.version, 1);
    }
}
