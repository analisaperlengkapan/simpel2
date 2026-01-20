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

use crate::{
    ApiError, ApiResponse, ApiResult, extractors::AuthenticatedUser, handlers::AppState,
    helpers::create_audit_log, services::vault::SecretMetadata,
};
use secreton_core::audit::AuditLog;

/// Create secret management operation routes
pub fn create_routes() -> Router<AppState> {
    Router::new()
        // Secret operations
        .route("/data/{path}", get(get_secret))
        .route("/data/{path}", post(create_secret))
        .route("/data/{path}", put(update_secret))
        .route("/data/{path}", delete(delete_secret))
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
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub status: Option<String>,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub namespace: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

pub async fn get_audit_logs(
    State(state): State<AppState>,
    Query(query): Query<AuditQuery>,
) -> ApiResult<Json<ApiResponse<Vec<AuditLog>>>> {
    let status = match query.status.as_deref() {
        Some(s) => match s.to_lowercase().as_str() {
            "success" => Some(secreton_core::audit::AuditStatus::Success),
            "failure" => Some(secreton_core::audit::AuditStatus::Failure),
            "denied" => Some(secreton_core::audit::AuditStatus::Denied),
            _ => return Err(ApiError::BadRequest {
                message: format!("Invalid status: {}. Must be success, failure, or denied.", s),
            }),
        },
        None => None,
    };

    let core_query = secreton_core::audit::AuditQuery {
        action: query.action,
        actor: query.user_id,
        resource_type: query.resource_type,
        resource_id: query.resource_id,
        status,
        start_time: query.start_time,
        end_time: query.end_time,
        namespace: query.namespace,
        limit: query.limit,
        offset: query.offset,
    };

    let logs = state
        .audit
        .query(core_query)
        .await
        .map_err(|e| ApiError::Internal {
            message: format!("Audit query failed: {}", e),
        })?;

    Ok(Json(ApiResponse::success(logs)))
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

#[cfg(all(test, feature = "enable-inline-tests"))]
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
        // First put a secret so we can retrieve it
        let payload = serde_json::json!({
            "data": {"key1": "value1"},
            "metadata": {
                "description": "Config",
                "tags": ["config"],
                "owner": "dev",
                "classification": "confidential"
            },
            "ttl": 3600
        });

        let put_response = server.post("/data/app%2Fconfig")
            .add_header("Authorization", "Bearer token")
            .json(&payload)
            .await;
        put_response.assert_status_ok();

        // Now retrieve it
        let response = server.get("/data/app%2Fconfig")
            .add_header("Authorization", "Bearer token")
            .await;
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

        let response = server.post("/data/app%2Fadmin")
            .add_header("Authorization", "Bearer token")
            .json(&payload)
            .await;
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

        let response = server.post("/keys")
            .add_header("Authorization", "Bearer token")
            .json(&request)
            .await;
        response.assert_status_ok();

        let body: ApiResponse<KeyResponse> = response.json();
        assert!(body.success);
        let key = body.data.expect("key response");
        assert_eq!(key.name, "signing-key");
        assert_eq!(key.algorithm, "Ed25519");
        assert!(key.public_key.is_some());
    }

    #[tokio::test]
    async fn test_get_audit_logs_returns_filtered_results() {
        let server = server_with_routes().await;

        // 1. Generate some audit activity by creating a secret
        let payload = serde_json::json!({
            "data": {"foo": "bar"},
            "metadata": {"owner": "test-user"},
            "ttl": 300
        });

        server.post("/data/app%2Faudit-test")
            .add_header("Authorization", "Bearer token")
            .json(&payload)
            .await
            .assert_status_ok();

        // 2. Query logs filtering by action "secret_created"
        let response = server.get("/audit")
            .add_query_param("action", "secret_created")
            .add_query_param("status", "success")
            .add_header("Authorization", "Bearer token")
            .await;

        response.assert_status_ok();

        let body: ApiResponse<Vec<AuditLog>> = response.json();
        assert!(body.success);
        let logs = body.data.expect("logs");

        // Note: The memory backend in test environment might be shared or fresh per test depending on implementation.
        // Assuming fresh or at least containing our new log.
        assert!(!logs.is_empty(), "Should have at least one log");
        let log = logs.first().unwrap();
        assert_eq!(log.action, "secret_created");
        assert_eq!(log.status, secreton_core::audit::AuditStatus::Success);

        // 3. Test invalid status returns bad request
        let response_invalid = server.get("/audit")
            .add_query_param("status", "invalid_status")
            .add_header("Authorization", "Bearer token")
            .await;

        response_invalid.assert_status_bad_request();
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
pub struct SecretResponse {
    pub path: String,
    pub data: HashMap<String, String>,
    pub metadata: SecretMetadata,
    pub version: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct KeyMetadata {
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub owner: Option<String>,
    pub purpose: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyResponse {
    pub valid: bool,
    pub key_version: u32,
}

#[derive(Debug, Deserialize)]
pub struct HashRequest {
    pub data: String,
    pub algorithm: String,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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
        metadata: secret_data.metadata,
        version: secret_data.version,
        created_at: secret_data.created_at,
        updated_at: secret_data.updated_at,
        expires_at: secret_data.expires_at,
    };

    Ok(Json(ApiResponse::success(secret)))
}

pub async fn create_secret(
    State(state): State<AppState>,
    Path(path): Path<String>,
    user: AuthenticatedUser,
    Json(request): Json<CreateSecretRequest>,
) -> ApiResult<Json<ApiResponse<SecretResponse>>> {
    let expires_at = request
        .ttl
        .map(|ttl| chrono::Utc::now() + chrono::Duration::seconds(ttl as i64));
    
    // Prepare metadata with default owner if not provided
    let mut metadata = request.metadata.clone().unwrap_or_default();
    if metadata.owner.is_none() {
        metadata.owner = Some(user.username.clone());
    }

    // Create secret using vault service
    let secret_data = state
        .vault
        .put_secret(
            &path,
            request.data.clone(),
            metadata.clone(),
            &user.id.to_string(),
            expires_at,
        )
        .await
        .map_err(|e| ApiError::Internal {
            message: format!("Failed to create secret: {}", e),
        })?;

    let secret = SecretResponse {
        path: secret_data.path,
        data: secret_data.data,
        metadata,
        version: secret_data.version,
        created_at: secret_data.created_at,
        updated_at: secret_data.updated_at,
        expires_at: secret_data.expires_at,
    };

    // Audit log
    let audit_entry = create_audit_log("secret_created", &user.username, "secret", &path);
    let _ = state.audit.log(audit_entry).await;

    Ok(Json(ApiResponse::success(secret)))
}

pub async fn update_secret(
    State(state): State<AppState>,
    Path(path): Path<String>,
    user: AuthenticatedUser,
    Json(request): Json<CreateSecretRequest>,
) -> ApiResult<Json<ApiResponse<SecretResponse>>> {
    let expires_at = request
        .ttl
        .map(|ttl| chrono::Utc::now() + chrono::Duration::seconds(ttl as i64));

    // Update secret using vault service
    let secret_data = state
        .vault
        .put_secret(
            &path,
            request.data.clone(),
            request.metadata.clone().unwrap_or_default(),
            &user.id.to_string(),
            expires_at,
        )
        .await
        .map_err(|e| ApiError::Internal {
            message: format!("Failed to update secret: {}", e),
        })?;

    let secret = SecretResponse {
        path: secret_data.path,
        data: secret_data.data,
        metadata: request.metadata.unwrap_or_default(),
        version: secret_data.version,
        created_at: secret_data.created_at,
        updated_at: secret_data.updated_at,
        expires_at: secret_data.expires_at,
    };

    // Audit log
    let audit_entry = create_audit_log("secret_updated", &user.username, "secret", &path);
    let _ = state.audit.log(audit_entry).await;

    Ok(Json(ApiResponse::success(secret)))
}

pub async fn delete_secret(
    State(state): State<AppState>,
    Path(path): Path<String>,
    user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<serde_json::Value>>> {
    // Delete secret using vault service
    state
        .vault
        .delete_secret(&path, &user.id.to_string())
        .await
        .map_err(|e| ApiError::Internal {
            message: format!("Failed to delete secret: {}", e),
        })?;

    let data = serde_json::json!({
        "message": "Secret deleted successfully",
        "path": path
    });

    // Audit log
    let audit_entry = create_audit_log("secret_deleted", &user.username, "secret", &path);
    let _ = state.audit.log(audit_entry).await;

    Ok(Json(ApiResponse::success(data)))
}

pub async fn list_secrets(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
    user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<Vec<SecretListItem>>>> {
    // List secrets using vault service
    let paths = state
        .vault
        .list_secrets(query.filter.as_deref(), &user.id.to_string())
        .await
        .map_err(|e| ApiError::Internal {
            message: format!("Failed to list secrets: {}", e),
        })?;

    // Convert paths to SecretListItem
    let secrets: Vec<SecretListItem> = paths
        .into_iter()
        .map(|path| SecretListItem {
            path,
            metadata: SecretMetadata::default(),
            version: 1,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
        .collect();

    Ok(Json(ApiResponse::success(secrets)))
}

/// Key operations
pub async fn create_key(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(request): Json<CreateKeyRequest>,
) -> ApiResult<Json<ApiResponse<KeyResponse>>> {
    // Create key using vault service
    let key_info = state
        .vault
        .create_key(&request.name, &request.key_type, &user.id.to_string())
        .await
        .map_err(|e| ApiError::Internal {
            message: format!("Failed to create key: {}", e),
        })?;

    let key = KeyResponse {
        id: key_info.id.clone(),
        name: key_info.name,
        key_type: key_info.key_type,
        algorithm: request.algorithm,
        size: request.size.unwrap_or(256),
        usage: request.usage,
        metadata: request.metadata.unwrap_or_default(),
        version: key_info.version,
        created_at: key_info.created_at,
        status: "active".to_string(),
        public_key: Some("-----BEGIN PUBLIC KEY-----\n...\n-----END PUBLIC KEY-----".to_string()),
    };

    // Audit log
    let audit_entry = create_audit_log("key_generated", &user.username, "key", &key.id);
    let _ = state.audit.log(audit_entry).await;

    Ok(Json(ApiResponse::success(key)))
}

pub async fn get_key(
    State(state): State<AppState>,
    Path(key_id): Path<String>,
    user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<KeyResponse>>> {
    // Get key using vault service
    let key_info = state
        .vault
        .get_key(&key_id, &user.id.to_string())
        .await
        .map_err(|e| ApiError::NotFound {
            resource: format!("Key '{}'", key_id),
        })?;

    let key = KeyResponse {
        id: key_info.id,
        name: key_info.name,
        key_type: key_info.key_type.clone(),
        algorithm: key_info.key_type,
        size: 256,
        usage: vec!["sign".to_string(), "verify".to_string()],
        metadata: KeyMetadata::default(),
        version: key_info.version,
        created_at: key_info.created_at,
        status: "active".to_string(),
        public_key: Some("-----BEGIN PUBLIC KEY-----\n...\n-----END PUBLIC KEY-----".to_string()),
    };

    Ok(Json(ApiResponse::success(key)))
}

pub async fn list_keys(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
    user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<Vec<KeyResponse>>>> {
    // List keys using vault service
    let key_infos = state
        .vault
        .list_keys(&user.id.to_string())
        .await
        .map_err(|e| ApiError::Internal {
            message: format!("Failed to list keys: {}", e),
        })?;

    let keys: Vec<KeyResponse> = key_infos
        .into_iter()
        .map(|key_info| KeyResponse {
            id: key_info.id,
            name: key_info.name,
            key_type: key_info.key_type.clone(),
            algorithm: key_info.key_type,
            size: 256,
            usage: vec!["sign".to_string()],
            metadata: KeyMetadata::default(),
            version: key_info.version,
            created_at: key_info.created_at,
            status: "active".to_string(),
            public_key: None,
        })
        .collect();

    Ok(Json(ApiResponse::success(keys)))
}

pub async fn rotate_key(
    State(state): State<AppState>,
    Path(key_id): Path<String>,
    user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<KeyResponse>>> {
    // Rotate key using vault service
    let key_info = state
        .vault
        .rotate_key(&key_id, &user.id.to_string())
        .await
        .map_err(|e| ApiError::Internal {
            message: format!("Failed to rotate key: {}", e),
        })?;

    let key = KeyResponse {
        id: key_info.id.clone(),
        name: key_info.name,
        key_type: key_info.key_type.clone(),
        algorithm: key_info.key_type,
        size: 256,
        usage: vec!["sign".to_string(), "verify".to_string()],
        metadata: KeyMetadata::default(),
        version: key_info.version,
        created_at: key_info.created_at,
        status: "active".to_string(),
        public_key: Some("-----BEGIN PUBLIC KEY-----\n...\n-----END PUBLIC KEY-----".to_string()),
    };

    // Audit log
    let audit_entry = create_audit_log("key_rotated", &user.username, "key", &key.id);
    let _ = state.audit.log(audit_entry).await;

    Ok(Json(ApiResponse::success(key)))
}

/// Cryptographic operations
pub async fn encrypt_data(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(request): Json<EncryptRequest>,
) -> ApiResult<Json<ApiResponse<EncryptResponse>>> {
    // Encrypt using vault service
    let result = state
        .vault
        .encrypt(&request.key_id, &request.plaintext, &user.id.to_string())
        .await
        .map_err(|e| ApiError::Internal {
            message: format!("Encryption failed: {}", e),
        })?;

    let response = EncryptResponse {
        ciphertext: result.ciphertext,
        key_version: result.key_version,
        algorithm: request.algorithm.unwrap_or("AES-256-GCM".to_string()),
    };

    // Audit log
    let audit_entry = create_audit_log("data_encrypted", &user.username, "crypto", &request.key_id);
    let _ = state.audit.log(audit_entry).await;

    Ok(Json(ApiResponse::success(response)))
}

pub async fn decrypt_data(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(request): Json<DecryptRequest>,
) -> ApiResult<Json<ApiResponse<DecryptResponse>>> {
    // Decrypt using vault service
    let result = state
        .vault
        .decrypt(&request.key_id, &request.ciphertext, &user.id.to_string())
        .await
        .map_err(|e| ApiError::Internal {
            message: format!("Decryption failed: {}", e),
        })?;

    let response = DecryptResponse {
        plaintext: result.plaintext,
        key_version: 1,
    };

    // Audit log
    let audit_entry = create_audit_log("data_decrypted", &user.username, "crypto", &request.key_id);
    let _ = state.audit.log(audit_entry).await;

    Ok(Json(ApiResponse::success(response)))
}

pub async fn sign_data(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(request): Json<SignRequest>,
) -> ApiResult<Json<ApiResponse<SignResponse>>> {
    // Sign using vault service
    let result = state
        .vault
        .sign(
            &request.key_id,
            &request.data,
            request.algorithm.as_deref(),
            &user.id.to_string(),
        )
        .await
        .map_err(|e| ApiError::Internal {
            message: format!("Signing failed: {}", e),
        })?;

    let response = SignResponse {
        signature: result.signature,
        key_version: result.key_version,
        algorithm: result.algorithm,
    };

    Ok(Json(ApiResponse::success(response)))
}

pub async fn verify_signature(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(request): Json<VerifyRequest>,
) -> ApiResult<Json<ApiResponse<VerifyResponse>>> {
    // Verify using vault service
    let result = state
        .vault
        .verify(
            &request.key_id,
            &request.data,
            &request.signature,
            &user.id.to_string(),
        )
        .await
        .map_err(|e| ApiError::Internal {
            message: format!("Verification failed: {}", e),
        })?;

    let response = VerifyResponse {
        valid: result.valid,
        key_version: result.key_version,
    };

    Ok(Json(ApiResponse::success(response)))
}

pub async fn hash_data(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(request): Json<HashRequest>,
) -> ApiResult<Json<ApiResponse<HashResponse>>> {
    // Hash using vault service
    let result = state
        .vault
        .hash(&request.data, &request.algorithm, &user.id.to_string())
        .await
        .map_err(|e| ApiError::Internal {
            message: format!("Hashing failed: {}", e),
        })?;

    let response = HashResponse {
        hash: result.hash,
        algorithm: result.algorithm,
    };

    Ok(Json(ApiResponse::success(response)))
}


// Key management handlers
pub async fn update_key(
    State(_state): State<AppState>,
    Path(key_id): Path<String>,
    Json(_payload): Json<serde_json::Value>,
) -> ApiResult<Json<ApiResponse<KeyResponse>>> {
    // Key updates are typically metadata changes (description, tags, etc.)
    // The actual key material should not change - use rotation instead
    tracing::warn!(key_id = %key_id, "Key update requested but not fully implemented");

    // For now, return the current key info
    // TODO: Implement metadata updates in key store
    Err(ApiError::BadRequest {
        message: "Key updates not supported. Use key rotation to change key material.".to_string(),
    })
}

pub async fn delete_key(
    State(state): State<AppState>,
    Path(key_id): Path<String>,
    headers: axum::http::HeaderMap,
) -> ApiResult<Json<ApiResponse<()>>> {
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

    // Key deletion should be done carefully with audit trail
    tracing::info!(key_id = %key_id, user_id = %user.id, "Key deletion requested");

    // Delete key using vault service (includes safeguards)
    state
        .vault
        .delete_key(&key_id, &user.id.to_string())
        .await
        .map_err(|e| ApiError::Internal {
            message: format!("Failed to delete key: {}", e),
        })?;

    // Audit: KeyDeletion
    let audit_entry = secreton_core::audit::AuditLog {
        id: uuid::Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        action: "key_deleted".to_string(),
        actor: Some(user.username.clone()),
        resource_type: "key".to_string(),
        resource_id: key_id.clone(),
        status: secreton_core::audit::AuditStatus::Success,
        ip: None,
        user_agent: None,
        namespace: None,
        metadata: std::collections::HashMap::new(),
    };
    let _ = state.audit.log(audit_entry).await;

    Ok(Json(ApiResponse::success(())))
}

pub async fn list_key_versions(
    State(_state): State<AppState>,
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
    Err(ApiError::NotFound {
        resource: format!("Policy '{}'", name),
    })
}

pub async fn create_policy(
    State(_state): State<AppState>,
    Path(name): Path<String>,
    Json(_policy_data): Json<serde_json::Value>,
) -> ApiResult<Json<ApiResponse<()>>> {
    // TODO: Implement policy storage system with validation
    tracing::warn!(policy_name = %name, "Policy create requested but policy storage not yet implemented");
    Err(ApiError::NotImplemented(
        "Policy storage not yet implemented".to_string(),
    ))
}

pub async fn update_policy(
    State(_state): State<AppState>,
    Path(name): Path<String>,
    Json(_policy_data): Json<serde_json::Value>,
) -> ApiResult<Json<ApiResponse<()>>> {
    // TODO: Implement policy storage system
    tracing::warn!(policy_name = %name, "Policy update requested but policy storage not yet implemented");
    Err(ApiError::NotFound {
        resource: format!("Policy '{}'", name),
    })
}

pub async fn delete_policy(
    State(_state): State<AppState>,
    Path(name): Path<String>,
) -> ApiResult<Json<ApiResponse<()>>> {
    // TODO: Implement policy storage system
    tracing::warn!(policy_name = %name, "Policy delete requested but policy storage not yet implemented");
    Err(ApiError::NotFound {
        resource: format!("Policy '{}'", name),
    })
}

// Backup management handlers - delegate to admin service
pub async fn create_backup(State(state): State<AppState>) -> ApiResult<Json<ApiResponse<String>>> {
    match state.admin.create_backup().await {
        Ok(backup_info) => {
            tracing::info!(backup_id = %backup_info.id, "Backup created");
            Ok(Json(ApiResponse::success(backup_info.id)))
        }
        Err(e) => Err(ApiError::Internal {
            message: format!("Backup creation failed: {}", e),
        }),
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
        Err(e) => Err(ApiError::Internal {
            message: format!("Failed to list backups: {}", e),
        }),
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
                let backup_json =
                    serde_json::to_string_pretty(&backup).map_err(|e| ApiError::Internal {
                        message: format!("Failed to serialize backup: {}", e),
                    })?;
                Ok(Json(ApiResponse::success(backup_json)))
            } else {
                Err(ApiError::NotFound {
                    resource: format!("Backup '{}'", backup_id),
                })
            }
        }
        Err(e) => Err(ApiError::Internal {
            message: format!("Failed to get backup: {}", e),
        }),
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
        Err(e) => Err(ApiError::Internal {
            message: format!("Backup restoration failed: {}", e),
        }),
    }
}

pub async fn delete_backup(
    State(state): State<AppState>,
    Path(backup_id): Path<String>,
) -> ApiResult<Json<ApiResponse<()>>> {
    // Backup deletion requires persistent storage implementation
    tracing::warn!(backup_id = %backup_id, "Backup deletion requested but not implemented");
    Err(ApiError::BadRequest {
        message: "Backup deletion not yet implemented - persistent storage required".to_string(),
    })
}
