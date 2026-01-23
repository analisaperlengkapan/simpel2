//! Zero-Knowledge API handlers
//!
//! REST API endpoints for zero-knowledge end-to-end encryption operations.

use axum::{
    Router,
    extract::{Path, State},
    response::Json,
    routing::{get, post},
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::{ApiError, ApiResponse, ApiResult, extractors::AuthenticatedUser};
use secreton_core::services::zero_knowledge::{
    KeyDerivationParams, ZeroKnowledgeMetadata, ZeroKnowledgeService,
};

use super::AppState;

/// Create Zero-Knowledge routes
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/store", post(store_secret))
        .route("/retrieve/{path}", get(retrieve_secret))
        .route("/derive-params", post(derive_params))
        .route("/list", get(list_secrets))
        .route("/delete/{path}", post(delete_secret))
}

/// Store secret request
#[derive(Debug, Deserialize, Serialize)]
pub struct StoreRequest {
    /// Path where secret should be stored
    pub path: String,
    /// Pre-encrypted data (base64 encoded)
    pub encrypted_data: String,
    /// Encryption algorithm used by client
    pub encryption_algorithm: String,
    /// Key derivation parameters used
    pub key_derivation_params: KeyDerivationParams,
}

/// Store secret response
#[derive(Debug, Serialize, Deserialize)]
pub struct StoreResponse {
    /// Path where secret was stored
    pub path: String,
    /// Success message
    pub message: String,
}

/// Retrieve secret response
#[derive(Debug, Serialize, Deserialize)]
pub struct RetrieveResponse {
    /// Path of the secret
    pub path: String,
    /// Encrypted data (base64 encoded)
    pub encrypted_data: String,
    /// Encryption algorithm
    pub encryption_algorithm: String,
    /// Key derivation parameters
    pub key_derivation_params: KeyDerivationParams,
    /// Creation timestamp
    pub created_at: String,
}

/// Derive params request
#[derive(Debug, Deserialize, Serialize)]
pub struct DeriveParamsRequest {
    /// Client entropy (base64 encoded)
    pub client_entropy: String,
}

/// Derive params response
#[derive(Debug, Serialize, Deserialize)]
pub struct DeriveParamsResponse {
    /// Key derivation parameters
    pub params: KeyDerivationParams,
}

/// List secrets response
#[derive(Debug, Serialize, Deserialize)]
pub struct ListResponse {
    /// List of secret paths
    pub paths: Vec<String>,
}

/// Delete secret response
#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteResponse {
    /// Path of deleted secret
    pub path: String,
    /// Success message
    pub message: String,
}

/// Store a pre-encrypted secret
///
/// POST /v1/zk/store
async fn store_secret(
    State(_state): State<AppState>,
    user: AuthenticatedUser,
    Json(request): Json<StoreRequest>,
) -> ApiResult<Json<ApiResponse<StoreResponse>>> {
    use secreton_core::audit::{AuditLog, AuditStatus};
    use std::collections::HashMap;

    info!("Storing zero-knowledge secret at path: {}", request.path);

    // Decode base64 encrypted data
    let encrypted_data =
        BASE64
            .decode(&request.encrypted_data)
            .map_err(|e| ApiError::BadRequest {
                message: format!("Invalid base64 encoding: {}", e),
            })?;

    // Create metadata
    let metadata = ZeroKnowledgeMetadata::new(
        request.path.clone(),
        request.encryption_algorithm.clone(),
        request.key_derivation_params.clone(),
    );

    // Store secret using the zero-knowledge service
    // Note: In production, this would use the actual service from AppState
    // For now, we'll create a temporary service instance
    let service = secreton_core::services::zero_knowledge::ZeroKnowledgeServiceImpl::new();

    let result = service.store(&request.path, encrypted_data, metadata).await;

    // Audit log - Note: Does NOT contain secret content or encryption keys
    let mut audit_metadata = HashMap::new();
    audit_metadata.insert("operation".to_string(), "zk.store".to_string());
    audit_metadata.insert("path".to_string(), request.path.clone());
    audit_metadata.insert(
        "algorithm".to_string(),
        request.encryption_algorithm.clone(),
    );
    // Deliberately NOT logging encrypted_data or key material

    let audit_log = AuditLog {
        id: uuid::Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        action: "zero_knowledge.store".to_string(),
        actor: Some(user.username.clone()),
        resource_type: "zero_knowledge_secret".to_string(),
        resource_id: request.path.clone(),
        status: if result.is_ok() {
            AuditStatus::Success
        } else {
            AuditStatus::Failure
        },
        ip: None, // TODO: Extract from request
        user_agent: None,
        namespace: None,
        metadata: audit_metadata,
    };

    // Log audit entry (in production, this would use the audit logger from AppState)
    info!("Audit: {:?}", audit_log);

    result.map_err(|e| ApiError::Internal {
        message: format!("Failed to store secret: {}", e),
    })?;

    info!(
        "Successfully stored zero-knowledge secret at: {}",
        request.path
    );

    Ok(Json(ApiResponse::success(StoreResponse {
        path: request.path,
        message: "Secret stored successfully".to_string(),
    })))
}

/// Retrieve an encrypted secret
///
/// GET /v1/zk/retrieve/{path}
async fn retrieve_secret(
    State(_state): State<AppState>,
    Path(path): Path<String>,
    user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<RetrieveResponse>>> {
    use secreton_core::audit::{AuditLog, AuditStatus};
    use std::collections::HashMap;

    info!("Retrieving zero-knowledge secret from path: {}", path);

    // Retrieve secret using the zero-knowledge service
    let service = secreton_core::services::zero_knowledge::ZeroKnowledgeServiceImpl::new();

    let result = service.retrieve(&path).await;

    // Audit log - Note: Does NOT contain secret content
    let mut audit_metadata = HashMap::new();
    audit_metadata.insert("operation".to_string(), "zk.retrieve".to_string());
    audit_metadata.insert("path".to_string(), path.clone());

    let audit_log = AuditLog {
        id: uuid::Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        action: "zero_knowledge.retrieve".to_string(),
        actor: Some(user.username.clone()),
        resource_type: "zero_knowledge_secret".to_string(),
        resource_id: path.clone(),
        status: if result.is_ok() {
            AuditStatus::Success
        } else {
            AuditStatus::Failure
        },
        ip: None,
        user_agent: None,
        namespace: None,
        metadata: audit_metadata,
    };

    info!("Audit: {:?}", audit_log);

    let (encrypted_data, metadata) = result.map_err(|e| ApiError::NotFound {
        resource: format!("Secret not found: {}", e),
    })?;

    // Encode encrypted data as base64
    let encrypted_data_b64 = BASE64.encode(&encrypted_data);

    info!(
        "Successfully retrieved zero-knowledge secret from: {}",
        path
    );

    Ok(Json(ApiResponse::success(RetrieveResponse {
        path: metadata.path,
        encrypted_data: encrypted_data_b64,
        encryption_algorithm: metadata.encryption_algorithm,
        key_derivation_params: metadata.key_derivation_params,
        created_at: metadata.created_at.to_rfc3339(),
    })))
}

/// Derive key parameters for client-side encryption
///
/// POST /v1/zk/derive-params
async fn derive_params(
    State(_state): State<AppState>,
    Json(request): Json<DeriveParamsRequest>,
) -> ApiResult<Json<ApiResponse<DeriveParamsResponse>>> {
    info!("Deriving key parameters for client");

    // Decode base64 client entropy
    let client_entropy =
        BASE64
            .decode(&request.client_entropy)
            .map_err(|e| ApiError::BadRequest {
                message: format!("Invalid base64 encoding: {}", e),
            })?;

    // Derive parameters using the zero-knowledge service
    let service = secreton_core::services::zero_knowledge::ZeroKnowledgeServiceImpl::new();

    let params = service
        .derive_params(&client_entropy)
        .await
        .map_err(|e| ApiError::Internal {
            message: format!("Failed to derive params: {}", e),
        })?;

    info!("Successfully derived key parameters");

    Ok(Json(ApiResponse::success(DeriveParamsResponse { params })))
}

/// List all zero-knowledge secret paths
///
/// GET /v1/zk/list
async fn list_secrets(State(_state): State<AppState>) -> ApiResult<Json<ApiResponse<ListResponse>>> {
    info!("Listing zero-knowledge secrets");

    // List secrets using the zero-knowledge service
    let service = secreton_core::services::zero_knowledge::ZeroKnowledgeServiceImpl::new();

    let paths = service.list_paths().await.map_err(|e| ApiError::Internal {
        message: format!("Failed to list secrets: {}", e),
    })?;

    info!("Successfully listed {} zero-knowledge secrets", paths.len());

    Ok(Json(ApiResponse::success(ListResponse { paths })))
}

/// Delete a zero-knowledge secret
///
/// POST /v1/zk/delete/{path}
async fn delete_secret(
    State(_state): State<AppState>,
    Path(path): Path<String>,
    user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<DeleteResponse>>> {
    use secreton_core::audit::{AuditLog, AuditStatus};
    use std::collections::HashMap;

    info!("Deleting zero-knowledge secret at path: {}", path);

    // Delete secret using the zero-knowledge service
    let service = secreton_core::services::zero_knowledge::ZeroKnowledgeServiceImpl::new();

    let result = service.delete(&path).await;

    let audit_log = AuditLog {
        id: uuid::Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        action: "zero_knowledge.delete".to_string(),
        actor: Some(user.username.clone()),
        resource_type: "zero_knowledge_secret".to_string(),
        resource_id: path.clone(),
        status: if result.is_ok() {
            AuditStatus::Success
        } else {
            AuditStatus::Failure
        },
        ip: None,
        user_agent: None,
        namespace: None,
        metadata: HashMap::new(),
    };

    // Log audit entry (in production, this would use the audit logger from AppState)
    info!("Audit: {:?}", audit_log);

    result.map_err(|e| ApiError::NotFound {
        resource: format!("Failed to delete secret: {}", e),
    })?;

    info!("Successfully deleted zero-knowledge secret at: {}", path);

    Ok(Json(ApiResponse::success(DeleteResponse {
        path,
        message: "Secret deleted successfully".to_string(),
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_request_serialization() {
        let request = StoreRequest {
            path: "secret/test".to_string(),
            encrypted_data: "dGVzdA==".to_string(),
            encryption_algorithm: "aes-256-gcm".to_string(),
            key_derivation_params: KeyDerivationParams::default_hkdf(),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: StoreRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.path, deserialized.path);
        assert_eq!(request.encrypted_data, deserialized.encrypted_data);
        assert_eq!(
            request.encryption_algorithm,
            deserialized.encryption_algorithm
        );
    }

    #[test]
    fn test_derive_params_request_serialization() {
        let request = DeriveParamsRequest {
            client_entropy: "dXNlci1wYXNzd29yZA==".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: DeriveParamsRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.client_entropy, deserialized.client_entropy);
    }
}
