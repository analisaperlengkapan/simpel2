//! Secret management operations handlers.
//!
//! Provides endpoints for secret storage, retrieval, key operations,
//! cryptographic functions, and audit trail management.
//!
//! This module handles secure secret management operations without
//! dependency on external vault systems like HashiCorp Vault.

use axum::{
    Router,
    extract::{Path, Query, State},
    response::Json,
    routing::{delete, get, post, put},
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{ApiError, ApiResponse, ApiResult, handlers::AppState};
use secreton_core::audit::AuditLog;

/// Create secret management operation routes
pub fn create_routes() -> Router<AppState> {
    Router::new()
        // Secret operations
        .route("/secrets/{path}", get(get_secret))
        .route("/secrets/{path}", post(create_secret))
        .route("/secrets/{path}", put(update_secret))
        .route("/secrets/{path}", delete(delete_secret))
        .route("/secrets", get(list_secrets))
        // Key operations
        .route("/keys", get(list_keys))
        .route("/keys", post(create_key))
        .route("/keys/{key_id}", get(get_key))
        .route("/keys/{key_id}", put(update_key))
        .route("/keys/{key_id}", delete(delete_key))
        .route("/keys/{key_id}/rotate", post(rotate_key))
        .route("/keys/{key_id}/versions", get(list_key_versions))
        // Encryption operations
        .route("/encrypt", post(encrypt_data))
        .route("/decrypt", post(decrypt_data))
        .route("/sign", post(sign_data))
        .route("/verify", post(verify_signature))
        .route("/hash", post(hash_data))
        // Policy operations
        .route("/policies", get(list_policies))
        .route("/policies/{name}", get(get_policy))
        .route("/policies/{name}", post(create_policy))
        .route("/policies/{name}", put(update_policy))
        .route("/policies/{name}", delete(delete_policy))
        // Audit operations
        .route("/audit", get(get_audit_logs))
        .route("/audit/export", get(export_audit_logs))
        // Backup operations
        .route("/backup", post(create_backup))
        .route("/backup", get(list_backups))
        .route("/backup/{backup_id}", get(get_backup))
        .route("/backup/{backup_id}/restore", post(restore_backup))
        .route("/backup/{backup_id}", delete(delete_backup))
}

#[derive(Debug, Deserialize)]
pub struct AuditQuery {
    pub user_id: Option<String>,
    pub action: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

pub async fn get_audit_logs(
    State(_state): State<AppState>,
    Query(_query): Query<AuditQuery>,
) -> ApiResult<Json<ApiResponse<Vec<AuditLog>>>> {
    // TODO: Implement audit log retrieval with filtering
    // Currently the AuditLogger only supports writing logs, not querying them
    // Need to implement audit backend with query capabilities
    let entries: Vec<AuditLog> = vec![];
    Ok(Json(ApiResponse::success(entries)))
}

#[derive(Debug, Deserialize)]
pub struct AuditExportQuery {
    pub format: Option<String>,
    pub user_id: Option<String>,
    pub action: Option<String>,
}

pub async fn export_audit_logs(
    State(_state): State<AppState>,
    Query(_query): Query<AuditExportQuery>,
) -> ApiResult<Json<ApiResponse<String>>> {
    // TODO: Implement audit log export with filtering and format conversion
    // Currently the AuditLogger only supports writing logs, not exporting them
    // Need to implement audit backend with export capabilities
    let data = String::from("[]");
    Ok(Json(ApiResponse::success(data)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ApiConfig;
    use crate::services::ServiceContainer;
    use axum_test::TestServer;
    use std::sync::Arc;

    async fn server_with_routes() -> TestServer {
        let config = ApiConfig::default();
        let services = Arc::new(
            ServiceContainer::new(&config)
                .await
                .expect("Failed to create services"),
        );

        let app = create_routes().with_state(services);
        TestServer::new(app).expect("failed to start test server")
    }

    #[tokio::test]
    async fn test_get_secret_returns_placeholder_data() {
        let server = server_with_routes().await;
        let response = server.get("/secrets/app/config").await;
        response.assert_status_ok();

        let body: ApiResponse<SecretResponse> = response.json();
        assert!(body.success);
        let data = body.data.expect("secret payload");
        assert_eq!(data.path, "app/config");
        assert!(data.data.contains_key("key1"));
    }

    #[tokio::test]
    async fn test_create_secret_accepts_payload() {
        let server = server_with_routes().await;
        let payload = serde_json::json!({
            "data": {"username": "admin"},
            "metadata": {
                "description": "Admin credentials",
                "tags": ["auth"],
                "owner": "security",
                "classification": "secret"
            },
            "ttl": 90
        });

        let response = server.post("/secrets/app/admin").json(&payload).await;
        response.assert_status_ok();

        let body: ApiResponse<SecretResponse> = response.json();
        assert!(body.success);
        let secret = body.data.expect("secret response");
        assert_eq!(secret.path, "app/admin");
        assert!(secret.expires_at.is_some());
    }

    #[tokio::test]
    async fn test_create_key_returns_public_key() {
        let server = server_with_routes().await;
        let request = serde_json::json!({
            "name": "signing-key",
            "key_type": "Ed25519",
            "algorithm": "Ed25519",
            "usage": ["sign", "verify"],
            "exportable": true
        });

        let response = server.post("/keys").json(&request).await;
        response.assert_status_ok();

        let body: ApiResponse<KeyResponse> = response.json();
        assert!(body.success);
        let key = body.data.expect("key response");
        assert_eq!(key.name, "signing-key");
        assert_eq!(key.algorithm, "Ed25519");
        assert!(key.public_key.is_some());
    }
}

/// Query parameters for listing operations
#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub sort: Option<String>,
    pub filter: Option<String>,
}

/// Secret request/response models
#[derive(Debug, Deserialize)]
pub struct CreateSecretRequest {
    pub data: HashMap<String, String>,
    pub metadata: Option<SecretMetadata>,
    pub ttl: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecretMetadata {
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub owner: Option<String>,
    pub classification: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SecretResponse {
    pub path: String,
    pub data: HashMap<String, String>,
    pub metadata: SecretMetadata,
    pub version: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize)]
pub struct SecretListItem {
    pub path: String,
    pub metadata: SecretMetadata,
    pub version: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Key request/response models
#[derive(Debug, Deserialize)]
pub struct CreateKeyRequest {
    pub name: String,
    pub key_type: String,
    pub algorithm: String,
    pub size: Option<u32>,
    pub usage: Vec<String>,
    pub metadata: Option<KeyMetadata>,
    pub exportable: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[derive(Default)]
pub struct KeyMetadata {
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub owner: Option<String>,
    pub purpose: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct KeyResponse {
    pub id: String,
    pub name: String,
    pub key_type: String,
    pub algorithm: String,
    pub size: u32,
    pub usage: Vec<String>,
    pub metadata: KeyMetadata,
    pub version: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub status: String,
    pub public_key: Option<String>,
}

/// Cryptographic operation models
#[derive(Debug, Deserialize)]
pub struct EncryptRequest {
    pub key_id: String,
    pub plaintext: String,
    pub context: Option<HashMap<String, String>>,
    pub algorithm: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EncryptResponse {
    pub ciphertext: String,
    pub key_version: u32,
    pub algorithm: String,
}

#[derive(Debug, Deserialize)]
pub struct DecryptRequest {
    pub key_id: String,
    pub ciphertext: String,
    pub context: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize)]
pub struct DecryptResponse {
    pub plaintext: String,
    pub key_version: u32,
}

#[derive(Debug, Deserialize)]
pub struct SignRequest {
    pub key_id: String,
    pub data: String,
    pub algorithm: Option<String>,
    pub format: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SignResponse {
    pub signature: String,
    pub key_version: u32,
    pub algorithm: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
    pub key_id: String,
    pub data: String,
    pub signature: String,
    pub algorithm: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct VerifyResponse {
    pub valid: bool,
    pub key_version: u32,
}

#[derive(Debug, Deserialize)]
pub struct HashRequest {
    pub data: String,
    pub algorithm: String,
}

#[derive(Debug, Serialize)]
pub struct HashResponse {
    pub hash: String,
    pub algorithm: String,
}

/// Policy models
#[derive(Debug, Deserialize)]
pub struct CreatePolicyRequest {
    pub name: String,
    pub rules: Vec<PolicyRule>,
    pub metadata: Option<PolicyMetadata>,
}

// Use canonical PolicyRule from core
pub use secreton_core::models::PolicyRule;

// API-specific extension if capabilities needed
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiPolicyRule {
    pub path: String,
    pub capabilities: Vec<String>,
    pub conditions: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PolicyMetadata {
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub owner: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PolicyResponse {
    pub name: String,
    pub rules: Vec<PolicyRule>,
    pub metadata: PolicyMetadata,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Secret operations
pub async fn get_secret(
    State(state): State<AppState>,
    Path(path): Path<String>,
    headers: axum::http::HeaderMap,
) -> ApiResult<Json<ApiResponse<SecretResponse>>> {
    // Extract user from token
    let token = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .unwrap_or("");

    // Validate token and get user
    let user = state
        .auth
        .validate_token(token)
        .await
        .map_err(|e| ApiError::Authentication {
            message: format!("Authentication required: {}", e),
        })?;

    // Retrieve secret from vault service
    let secret_data = state
        .vault
        .get_secret(&path, &user.id.to_string())
        .await
        .map_err(|e| ApiError::NotFound {
            resource: format!("Secret at path '{}'", path),
        })?;

    let secret = SecretResponse {
        path: secret_data.path,
        data: secret_data.data,
        metadata: SecretMetadata {
            description: None, // TODO: Add metadata field to SecretData
            tags: vec![],
            owner: Some(user.username.clone()),
            classification: Some("confidential".to_string()),
        },
        version: secret_data.version,
        created_at: secret_data.created_at,
        updated_at: secret_data.updated_at,
        expires_at: None, // TODO: Add expires_at to SecretData
    };

    Ok(Json(ApiResponse::success(secret)))
}

pub async fn create_secret(
    State(state): State<AppState>,
    Path(path): Path<String>,
    Json(request): Json<CreateSecretRequest>,
) -> ApiResult<Json<ApiResponse<SecretResponse>>> {
    // RBAC check (placeholder user)
    // TODO: Implement secret creation
    let secret = SecretResponse {
        path: path.clone(),
        data: request.data,
        metadata: request.metadata.unwrap_or_default(),
        version: 1,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        expires_at: request
            .ttl
            .map(|ttl| chrono::Utc::now() + chrono::Duration::seconds(ttl as i64)),
    };

    // Audit: SecretCreation
    let audit_entry = secreton_core::audit::AuditLog {
        id: uuid::Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        action: "secret_created".to_string(),
        actor: Some("unknown".to_string()),
        resource_type: "secret".to_string(),
        resource_id: path.clone(),
        status: secreton_core::audit::AuditStatus::Success,
        ip: None,
        user_agent: None,
        namespace: None,
        metadata: std::collections::HashMap::new(),
    };
    let _ = state.audit.log(audit_entry).await;

    Ok(Json(ApiResponse::success(secret)))
}

pub async fn update_secret(
    State(state): State<AppState>,
    Path(path): Path<String>,
    Json(request): Json<CreateSecretRequest>,
) -> ApiResult<Json<ApiResponse<SecretResponse>>> {
    // RBAC check (placeholder user)
    // TODO: Implement secret update
    let secret = SecretResponse {
        path: path.clone(),
        data: request.data,
        metadata: request.metadata.unwrap_or_default(),
        version: 2,
        created_at: chrono::Utc::now() - chrono::Duration::hours(1),
        updated_at: chrono::Utc::now(),
        expires_at: request
            .ttl
            .map(|ttl| chrono::Utc::now() + chrono::Duration::seconds(ttl as i64)),
    };

    // Audit: SecretVersionChange
    let audit_entry = secreton_core::audit::AuditLog {
        id: uuid::Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        action: "secret_updated".to_string(),
        actor: Some("unknown".to_string()),
        resource_type: "secret".to_string(),
        resource_id: path.clone(),
        status: secreton_core::audit::AuditStatus::Success,
        ip: None,
        user_agent: None,
        namespace: None,
        metadata: std::collections::HashMap::new(),
    };
    let _ = state.audit.log(audit_entry).await;

    Ok(Json(ApiResponse::success(secret)))
}

pub async fn delete_secret(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> ApiResult<Json<ApiResponse<serde_json::Value>>> {
    // RBAC check (placeholder user)
    // TODO: Implement secret deletion
    let data = serde_json::json!({
        "message": "Secret deleted successfully",
        "path": path
    });

    // Audit: SecretDeletion
    let audit_entry = secreton_core::audit::AuditLog {
        id: uuid::Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        action: "secret_deleted".to_string(),
        actor: Some("unknown".to_string()),
        resource_type: "secret".to_string(),
        resource_id: path.clone(),
        status: secreton_core::audit::AuditStatus::Success,
        ip: None,
        user_agent: None,
        namespace: None,
        metadata: std::collections::HashMap::new(),
    };
    let _ = state.audit.log(audit_entry).await;

    Ok(Json(ApiResponse::success(data)))
}

pub async fn list_secrets(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> ApiResult<Json<ApiResponse<Vec<SecretListItem>>>> {
    // RBAC check could be resource-specific; allow listing with generic check
    // TODO: Implement secret listing
    let secrets = vec![SecretListItem {
        path: "app/database".to_string(),
        metadata: SecretMetadata {
            description: Some("Database credentials".to_string()),
            tags: vec!["database".to_string()],
            owner: Some("admin".to_string()),
            classification: Some("sensitive".to_string()),
        },
        version: 3,
        created_at: chrono::Utc::now() - chrono::Duration::days(7),
        updated_at: chrono::Utc::now() - chrono::Duration::hours(2),
    }];

    Ok(Json(ApiResponse::success(secrets)))
}

/// Key operations
pub async fn create_key(
    State(state): State<AppState>,
    Json(request): Json<CreateKeyRequest>,
) -> ApiResult<Json<ApiResponse<KeyResponse>>> {
    // RBAC check
    // TODO: Implement key creation
    let key = KeyResponse {
        id: uuid::Uuid::new_v4().to_string(),
        name: request.name,
        key_type: request.key_type,
        algorithm: request.algorithm,
        size: request.size.unwrap_or(256),
        usage: request.usage,
        metadata: request.metadata.unwrap_or_default(),
        version: 1,
        created_at: chrono::Utc::now(),
        status: "active".to_string(),
        public_key: Some("-----BEGIN PUBLIC KEY-----\n...\n-----END PUBLIC KEY-----".to_string()),
    };

    // Audit: KeyGeneration
    let audit_entry = secreton_core::audit::AuditLog {
        id: uuid::Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        action: "key_generated".to_string(),
        actor: Some("unknown".to_string()),
        resource_type: "key".to_string(),
        resource_id: key.id.clone(),
        status: secreton_core::audit::AuditStatus::Success,
        ip: None,
        user_agent: None,
        namespace: None,
        metadata: std::collections::HashMap::new(),
    };
    let _ = state.audit.log(audit_entry).await;

    Ok(Json(ApiResponse::success(key)))
}

pub async fn get_key(
    State(state): State<AppState>,
    Path(key_id): Path<String>,
) -> ApiResult<Json<ApiResponse<KeyResponse>>> {
    // RBAC check
    // TODO: Implement key retrieval
    let key = KeyResponse {
        id: key_id,
        name: "example-key".to_string(),
        key_type: "Ed25519".to_string(), // Changed from RSA to Ed25519 for security
        algorithm: "Ed25519".to_string(), // Changed from RS256 to Ed25519
        size: 256,                       // Ed25519 key size
        usage: vec!["sign".to_string(), "verify".to_string()],
        metadata: KeyMetadata::default(),
        version: 1,
        created_at: chrono::Utc::now(),
        status: "active".to_string(),
        public_key: Some("-----BEGIN PUBLIC KEY-----\n...\n-----END PUBLIC KEY-----".to_string()),
    };

    Ok(Json(ApiResponse::success(key)))
}

pub async fn list_keys(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> ApiResult<Json<ApiResponse<Vec<KeyResponse>>>> {
    // TODO: Implement key listing
    let keys = vec![KeyResponse {
        id: uuid::Uuid::new_v4().to_string(),
        name: "signing-key".to_string(),
        key_type: "Ed25519".to_string(), // Changed from RSA to Ed25519
        algorithm: "Ed25519".to_string(), // Changed from RS256 to Ed25519
        size: 256,                       // Ed25519 key size
        usage: vec!["sign".to_string()],
        metadata: KeyMetadata::default(),
        version: 1,
        created_at: chrono::Utc::now(),
        status: "active".to_string(),
        public_key: None,
    }];

    Ok(Json(ApiResponse::success(keys)))
}

pub async fn rotate_key(
    State(state): State<AppState>,
    Path(key_id): Path<String>,
) -> ApiResult<Json<ApiResponse<KeyResponse>>> {
    // RBAC check
    // TODO: Implement key rotation
    let key = KeyResponse {
        id: key_id,
        name: "example-key".to_string(),
        key_type: "Ed25519".to_string(), // Changed from RSA to Ed25519
        algorithm: "Ed25519".to_string(), // Changed from RS256 to Ed25519
        size: 256,                       // Ed25519 key size
        usage: vec!["sign".to_string(), "verify".to_string()],
        metadata: KeyMetadata::default(),
        version: 2, // Incremented version
        created_at: chrono::Utc::now(),
        status: "active".to_string(),
        public_key: Some("-----BEGIN PUBLIC KEY-----\n...\n-----END PUBLIC KEY-----".to_string()),
    };

    // Audit: KeyRotation
    let audit_entry = secreton_core::audit::AuditLog {
        id: uuid::Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        action: "key_rotated".to_string(),
        actor: Some("unknown".to_string()),
        resource_type: "key".to_string(),
        resource_id: key.id.clone(),
        status: secreton_core::audit::AuditStatus::Success,
        ip: None,
        user_agent: None,
        namespace: None,
        metadata: std::collections::HashMap::new(),
    };
    let _ = state.audit.log(audit_entry).await;

    Ok(Json(ApiResponse::success(key)))
}

/// Cryptographic operations
pub async fn encrypt_data(
    State(state): State<AppState>,
    Json(request): Json<EncryptRequest>,
) -> ApiResult<Json<ApiResponse<EncryptResponse>>> {
    // RBAC check
    // TODO: Implement encryption
    let response = EncryptResponse {
        ciphertext: "encrypted_data_base64".to_string(),
        key_version: 1,
        algorithm: request.algorithm.unwrap_or("AES-GCM".to_string()),
    };

    // Audit: EncryptionOperation
    let audit_entry = secreton_core::audit::AuditLog {
        id: uuid::Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        action: "data_encrypted".to_string(),
        actor: Some("unknown".to_string()),
        resource_type: "crypto".to_string(),
        resource_id: request.key_id.clone(),
        status: secreton_core::audit::AuditStatus::Success,
        ip: None,
        user_agent: None,
        namespace: None,
        metadata: std::collections::HashMap::new(),
    };
    let _ = state.audit.log(audit_entry).await;

    Ok(Json(ApiResponse::success(response)))
}

pub async fn decrypt_data(
    State(state): State<AppState>,
    Json(request): Json<DecryptRequest>,
) -> ApiResult<Json<ApiResponse<DecryptResponse>>> {
    // RBAC check
    // TODO: Implement decryption
    let response = DecryptResponse {
        plaintext: "decrypted_data".to_string(),
        key_version: 1,
    };

    // Audit: DecryptionOperation
    let audit_entry = secreton_core::audit::AuditLog {
        id: uuid::Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        action: "data_decrypted".to_string(),
        actor: Some("unknown".to_string()),
        resource_type: "crypto".to_string(),
        resource_id: request.key_id.clone(),
        status: secreton_core::audit::AuditStatus::Success,
        ip: None,
        user_agent: None,
        namespace: None,
        metadata: std::collections::HashMap::new(),
    };
    let _ = state.audit.log(audit_entry).await;

    Ok(Json(ApiResponse::success(response)))
}

pub async fn sign_data(
    State(_state): State<AppState>,
    Json(request): Json<SignRequest>,
) -> ApiResult<Json<ApiResponse<SignResponse>>> {
    // TODO: Implement signing
    let response = SignResponse {
        signature: "signature_base64".to_string(),
        key_version: 1,
        algorithm: request.algorithm.unwrap_or("RS256".to_string()),
    };

    Ok(Json(ApiResponse::success(response)))
}

pub async fn verify_signature(
    State(_state): State<AppState>,
    Json(request): Json<VerifyRequest>,
) -> ApiResult<Json<ApiResponse<VerifyResponse>>> {
    // TODO: Implement signature verification
    let response = VerifyResponse {
        valid: true,
        key_version: 1,
    };

    Ok(Json(ApiResponse::success(response)))
}

pub async fn hash_data(
    State(_state): State<AppState>,
    Json(request): Json<HashRequest>,
) -> ApiResult<Json<ApiResponse<HashResponse>>> {
    // TODO: Implement hashing
    let response = HashResponse {
        hash: "hash_hex".to_string(),
        algorithm: request.algorithm,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Default implementations for metadata
impl Default for SecretMetadata {
    fn default() -> Self {
        Self {
            description: None,
            tags: vec![],
            owner: None,
            classification: None,
        }
    }
}


// Key management handlers
pub async fn update_key(
    State(state): State<AppState>,
    Path(key_id): Path<String>,
    Json(_payload): Json<serde_json::Value>,
) -> ApiResult<Json<ApiResponse<KeyResponse>>> {
    // Key updates are typically metadata changes (description, tags, etc.)
    // The actual key material should not change - use rotation instead
    tracing::warn!(key_id = %key_id, "Key update requested but not fully implemented");

    // For now, return the current key info
    // TODO: Implement metadata updates in key store
    Err(ApiError::BadRequest {
        message: "Key updates not supported. Use key rotation to change key material.".to_string()
    })
}

pub async fn delete_key(
    State(state): State<AppState>,
    Path(key_id): Path<String>,
) -> ApiResult<Json<ApiResponse<()>>> {
    // Key deletion should be done carefully with audit trail
    tracing::info!(key_id = %key_id, "Key deletion requested");

    // TODO: Implement key deletion with proper safeguards:
    // 1. Check if key is in use
    // 2. Archive before deletion
    // 3. Audit trail
    // 4. Cascade to associated data

    Err(ApiError::BadRequest {
        message: "Key deletion not yet implemented. Consider key deactivation instead for safety.".to_string()
    })
}

pub async fn list_key_versions(
    State(state): State<AppState>,
    Path(key_id): Path<String>,
) -> ApiResult<Json<ApiResponse<Vec<KeyResponse>>>> {
    tracing::debug!(key_id = %key_id, "Listing key versions");

    // TODO: Implement key versioning in key store
    // Key versioning is important for key rotation and historical access

    // For now, return empty list
    Ok(Json(ApiResponse::success(vec![])))
}

// Policy management handlers
// Note: Policy management requires a dedicated policy store with CRUD operations
// The current PolicySet is designed for policy evaluation, not storage management
pub async fn list_policies(
    State(_state): State<AppState>,
) -> ApiResult<Json<ApiResponse<Vec<String>>>> {
    // TODO: Implement policy storage/retrieval system
    // Current PolicySet is for evaluation only
    tracing::warn!("Policy list requested but policy storage not yet implemented");
    Ok(Json(ApiResponse::success(vec![])))
}

pub async fn get_policy(
    State(_state): State<AppState>,
    Path(name): Path<String>,
) -> ApiResult<Json<ApiResponse<String>>> {
    // TODO: Implement policy storage/retrieval system
    tracing::warn!(policy_name = %name, "Policy get requested but policy storage not yet implemented");
    Err(ApiError::NotFound { resource: format!("Policy '{}'", name) })
}

pub async fn create_policy(
    State(_state): State<AppState>,
    Path(name): Path<String>,
    Json(_policy_data): Json<serde_json::Value>,
) -> ApiResult<Json<ApiResponse<()>>> {
    // TODO: Implement policy storage system with validation
    tracing::warn!(policy_name = %name, "Policy create requested but policy storage not yet implemented");
    Err(ApiError::NotImplemented("Policy storage not yet implemented".to_string()))
}

pub async fn update_policy(
    State(_state): State<AppState>,
    Path(name): Path<String>,
    Json(_policy_data): Json<serde_json::Value>,
) -> ApiResult<Json<ApiResponse<()>>> {
    // TODO: Implement policy storage system
    tracing::warn!(policy_name = %name, "Policy update requested but policy storage not yet implemented");
    Err(ApiError::NotFound { resource: format!("Policy '{}'", name) })
}

pub async fn delete_policy(
    State(_state): State<AppState>,
    Path(name): Path<String>,
) -> ApiResult<Json<ApiResponse<()>>> {
    // TODO: Implement policy storage system
    tracing::warn!(policy_name = %name, "Policy delete requested but policy storage not yet implemented");
    Err(ApiError::NotFound { resource: format!("Policy '{}'", name) })
}

// Backup management handlers - delegate to admin service
pub async fn create_backup(
    State(state): State<AppState>
) -> ApiResult<Json<ApiResponse<String>>> {
    match state.admin.create_backup().await {
        Ok(backup_info) => {
            tracing::info!(backup_id = %backup_info.id, "Backup created");
            Ok(Json(ApiResponse::success(backup_info.id)))
        }
        Err(e) => Err(ApiError::Internal { message: format!("Backup creation failed: {}", e) })
    }
}

pub async fn list_backups(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<Vec<String>>>> {
    match state.admin.list_backups().await {
        Ok(backups) => {
            let backup_ids: Vec<String> = backups.into_iter().map(|b| b.id).collect();
            Ok(Json(ApiResponse::success(backup_ids)))
        }
        Err(e) => Err(ApiError::Internal { message: format!("Failed to list backups: {}", e) })
    }
}

pub async fn get_backup(
    State(state): State<AppState>,
    Path(backup_id): Path<String>,
) -> ApiResult<Json<ApiResponse<String>>> {
    // Get backup metadata
    match state.admin.list_backups().await {
        Ok(backups) => {
            if let Some(backup) = backups.into_iter().find(|b| b.id == backup_id) {
                let backup_json = serde_json::to_string_pretty(&backup)
                    .map_err(|e| ApiError::Internal { message: format!("Failed to serialize backup: {}", e) })?;
                Ok(Json(ApiResponse::success(backup_json)))
            } else {
                Err(ApiError::NotFound { resource: format!("Backup '{}'", backup_id) })
            }
        }
        Err(e) => Err(ApiError::Internal { message: format!("Failed to get backup: {}", e) })
    }
}

pub async fn restore_backup(
    State(state): State<AppState>,
    Path(backup_id): Path<String>,
) -> ApiResult<Json<ApiResponse<()>>> {
    match state.admin.restore_backup(&backup_id).await {
        Ok(_result) => {
            tracing::info!(backup_id = %backup_id, "Backup restored");
            Ok(Json(ApiResponse::success(())))
        }
        Err(e) => Err(ApiError::Internal { message: format!("Backup restoration failed: {}", e) })
    }
}

pub async fn delete_backup(
    State(state): State<AppState>,
    Path(backup_id): Path<String>,
) -> ApiResult<Json<ApiResponse<()>>> {
    // Backup deletion requires persistent storage implementation
    tracing::warn!(backup_id = %backup_id, "Backup deletion requested but not implemented");
    Err(ApiError::BadRequest {
        message: "Backup deletion not yet implemented - persistent storage required".to_string()
    })
}
