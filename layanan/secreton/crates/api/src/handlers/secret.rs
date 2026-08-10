//! Secret management operations handlers.
//!
//! Provides endpoints for secret storage, retrieval, key operations,
//! cryptographic functions, and audit trail management.
//!
//! This module handles secure secret management operations without
//! dependency on external engine systems like HashiCorp Engine.

use axum::{
    Router,
    extract::{Path, Query, State},
    response::Json,
    routing::{delete, get, post, put},
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    ApiError, ApiResponse, ApiResult,
    extractors::AuthenticatedUser,
    handlers::AppState,
    helpers::create_audit_log,
    services::engine::{KeyMetadata, SecretMetadata},
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
        // Policy operations live at /v1/sys/policies (handlers/policy.rs) and
        // ONLY there. A second, unguarded copy of the same CRUD used to be
        // mounted here at /v1/secret/policies: it called the same
        // `state.policy_service` with no admin check, so the `require_admin`
        // guard added to the /v1/sys routes could be sidestepped by changing
        // the URL. Do not re-add a policy route to this module.
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
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub status: Option<String>,
}

pub async fn get_audit_logs(
    State(state): State<AppState>,
    Query(query): Query<AuditQuery>,
) -> ApiResult<Json<ApiResponse<Vec<AuditLog>>>> {
    let mut core_query = secreton_core::audit::AuditQuery::new();

    if let Some(user_id) = query.user_id {
        core_query = core_query.actor(user_id);
    }
    if let Some(action) = query.action {
        core_query = core_query.action(action);
    }
    if let Some(start_time) = query.start_time
        && let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&start_time)
    {
        core_query.start_time = Some(dt.with_timezone(&chrono::Utc));
    }
    if let Some(end_time) = query.end_time
        && let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&end_time)
    {
        core_query.end_time = Some(dt.with_timezone(&chrono::Utc));
    }
    if let Some(status) = query.status {
        let status = match status.to_lowercase().as_str() {
            "success" => Some(secreton_core::audit::AuditStatus::Success),
            "failure" => Some(secreton_core::audit::AuditStatus::Failure),
            "denied" => Some(secreton_core::audit::AuditStatus::Denied),
            _ => None,
        };
        if let Some(s) = status {
            core_query = core_query.status(s);
        }
    }
    // Enforce default limit to prevent DoS (default 100, max 1000)
    core_query.limit = Some(query.limit.unwrap_or(100).min(1000));
    core_query.offset = query.offset;

    let logs = state
        .audit
        .query(&core_query)
        .await
        .map_err(|e| ApiError::Internal {
            message: e.to_string(),
        })?;

    Ok(Json(ApiResponse::success(logs)))
}

#[derive(Debug, Deserialize)]
pub struct AuditExportQuery {
    pub format: Option<String>,
    pub user_id: Option<String>,
    pub action: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub status: Option<String>,
    pub limit: Option<usize>,
}

pub async fn export_audit_logs(
    State(state): State<AppState>,
    Query(query): Query<AuditExportQuery>,
) -> ApiResult<Json<ApiResponse<String>>> {
    let mut core_query = secreton_core::audit::AuditQuery::new();

    if let Some(user_id) = query.user_id {
        core_query = core_query.actor(user_id);
    }
    if let Some(action) = query.action {
        core_query = core_query.action(action);
    }
    if let Some(start_time) = query.start_time
        && let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&start_time)
    {
        core_query.start_time = Some(dt.with_timezone(&chrono::Utc));
    }
    if let Some(end_time) = query.end_time
        && let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&end_time)
    {
        core_query.end_time = Some(dt.with_timezone(&chrono::Utc));
    }
    if let Some(status) = query.status {
        let status = match status.to_lowercase().as_str() {
            "success" => Some(secreton_core::audit::AuditStatus::Success),
            "failure" => Some(secreton_core::audit::AuditStatus::Failure),
            "denied" => Some(secreton_core::audit::AuditStatus::Denied),
            _ => None,
        };
        if let Some(s) = status {
            core_query = core_query.status(s);
        }
    }
    // Enforce default limit to prevent DoS (default 1000, max 10000 for export)
    core_query.limit = Some(query.limit.unwrap_or(1000).min(10000));

    let logs = state
        .audit
        .query(&core_query)
        .await
        .map_err(|e| ApiError::Internal {
            message: e.to_string(),
        })?;

    let format = query.format.unwrap_or_else(|| "json".to_string());
    let data = match format.to_lowercase().as_str() {
        "csv" => {
            let mut csv = String::from(
                "id,timestamp,action,actor,resource_type,resource_id,status,ip,user_agent\n",
            );
            for log in logs {
                let status_str = match log.status {
                    secreton_core::audit::AuditStatus::Success => "success",
                    secreton_core::audit::AuditStatus::Failure => "failure",
                    secreton_core::audit::AuditStatus::Denied => "denied",
                };
                csv.push_str(&format!(
                    "{},{},{},{},{},{},{},{},{}\n",
                    log.id,
                    log.timestamp.to_rfc3339(),
                    escape_csv(&log.action),
                    escape_csv(log.actor.as_deref().unwrap_or("")),
                    escape_csv(&log.resource_type),
                    escape_csv(&log.resource_id),
                    status_str,
                    escape_csv(log.ip.as_deref().unwrap_or("")),
                    escape_csv(log.user_agent.as_deref().unwrap_or(""))
                ));
            }
            csv
        }
        _ => serde_json::to_string_pretty(&logs).map_err(|e| ApiError::Internal {
            message: format!("Failed to serialize logs: {}", e),
        })?,
    };

    Ok(Json(ApiResponse::success(data)))
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[cfg(all(test, feature = "enable-inline-tests"))]
mod tests {
    use super::*;
    use crate::config::ApiConfig;
    use crate::services::ServiceContainer;
    use axum_test::TestServer;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_export_audit_logs_direct() {
        use secreton_storage::MemoryBackend;

        // Create dummy pool (mock logic inside ServiceContainer or irrelevant)
        let config = tokio_postgres::Config::new();
        let mgr_config = deadpool_postgres::ManagerConfig {
            recycling_method: deadpool_postgres::RecyclingMethod::Verified,
        };
        let mgr =
            deadpool_postgres::Manager::from_config(config, tokio_postgres::NoTls, mgr_config);
        let pool = deadpool_postgres::Pool::builder(mgr)
            .max_size(1)
            .build()
            .unwrap();

        // Create services
        let storage = Arc::new(MemoryBackend::new());
        let services = Arc::new(ServiceContainer::new_mock(storage, pool));

        // Log some events
        let log = secreton_core::audit::AuditLog {
            id: uuid::Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            action: "test_action".to_string(),
            actor: Some("test_user".to_string()),
            resource_type: "test".to_string(),
            resource_id: "123".to_string(),
            status: secreton_core::audit::AuditStatus::Success,
            ip: None,
            user_agent: None,
            namespace: None,
            metadata: HashMap::new(),
        };
        services.audit.log(log).await.unwrap();

        // Call handler
        let query = AuditExportQuery {
            format: Some("csv".to_string()),
            user_id: None,
            action: None,
            start_time: None,
            end_time: None,
            status: None,
            limit: None,
        };

        let result = export_audit_logs(State(services), Query(query)).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        let body = response.0;
        assert!(body.success);
        let csv = body.data.unwrap();
        assert!(csv.contains("test_action"));
        assert!(csv.contains("test_user"));
        assert!(csv.contains("success"));
    }

    async fn server_with_routes() -> TestServer {
        let config = ApiConfig::default();
        let services = Arc::new(
            ServiceContainer::new(&config)
                .await
                .expect("Failed to create services"),
        );

        let app = create_routes().with_state(services);
        TestServer::new(app)
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

        let put_response = server
            .post("/data/app%2Fconfig")
            .add_header("Authorization", "Bearer token")
            .json(&payload)
            .await;
        put_response.assert_status_ok();

        // Now retrieve it
        let response = server
            .get("/data/app%2Fconfig")
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

        let response = server
            .post("/data/app%2Fadmin")
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

        let response = server
            .post("/keys")
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
    async fn test_key_versioning() {
        let server = server_with_routes().await;
        // 1. Create a key
        let create_request = serde_json::json!({
            "name": "versioned-key",
            "key_type": "AES",
            "algorithm": "AES-256-GCM",
            "usage": ["encrypt", "decrypt"],
            "exportable": false
        });

        let response = server
            .post("/keys")
            .add_header("Authorization", "Bearer token")
            .json(&create_request)
            .await;
        response.assert_status_ok();
        let key: ApiResponse<KeyResponse> = response.json();
        let key_id = key.data.unwrap().name; // Use name as ID for now

        // 2. List versions (should be 1)
        let response = server
            .get(&format!("/keys/{}/versions", key_id))
            .add_header("Authorization", "Bearer token")
            .await;
        response.assert_status_ok();
        let body: ApiResponse<Vec<KeyResponse>> = response.json();
        let versions = body.data.expect("versions list");
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].version, 1);

        // 3. Rotate key
        let response = server
            .post(&format!("/keys/{}/rotate", key_id))
            .add_header("Authorization", "Bearer token")
            .await;
        response.assert_status_ok();
        let body: ApiResponse<KeyResponse> = response.json();
        assert_eq!(body.data.unwrap().version, 2);

        // 4. List versions (should be 2)
        let response = server
            .get(&format!("/keys/{}/versions", key_id))
            .add_header("Authorization", "Bearer token")
            .await;
        response.assert_status_ok();
        let body: ApiResponse<Vec<KeyResponse>> = response.json();
        let versions = body.data.expect("versions list");
        assert_eq!(versions.len(), 2);

        // Check versions are 2 and 1 (sorted descending)
        assert_eq!(versions[0].version, 2);
        assert_eq!(versions[1].version, 1);
    }

    #[tokio::test]
    async fn test_update_key_metadata() {
        let server = server_with_routes().await;
        // 1. Create a key
        let create_payload = serde_json::json!({
            "name": "metadata-test-key",
            "key_type": "Ed25519",
            "algorithm": "Ed25519",
            "usage": ["sign"],
            "metadata": {
                "description": "Initial description",
                "tags": ["initial"]
            }
        });

        let create_response = server
            .post("/keys")
            .add_header("Authorization", "Bearer token")
            .json(&create_payload)
            .await;
        create_response.assert_status_ok();
        let created_key: ApiResponse<KeyResponse> = create_response.json();
        let key_id = created_key.data.unwrap().id;

        // 2. Update metadata
        let update_payload = serde_json::json!({
            "description": "Updated description",
            "tags": ["updated", "test"],
            "owner": "new-owner",
            "purpose": "testing updates"
        });

        let update_response = server
            .put(&format!("/keys/{}", key_id))
            .add_header("Authorization", "Bearer token")
            .json(&update_payload)
            .await;
        update_response.assert_status_ok();

        // 3. Verify update in response
        let updated_key: ApiResponse<KeyResponse> = update_response.json();
        let metadata = updated_key.data.unwrap().metadata;
        assert_eq!(
            metadata.description,
            Some("Updated description".to_string())
        );
        assert!(metadata.tags.contains(&"updated".to_string()));
        assert_eq!(metadata.owner, Some("new-owner".to_string()));

        // 4. Verify persistence with get_key
        let get_response = server
            .get(&format!("/keys/{}", key_id))
            .add_header("Authorization", "Bearer token")
            .await;
        get_response.assert_status_ok();
        let fetched_key: ApiResponse<KeyResponse> = get_response.json();
        let fetched_metadata = fetched_key.data.unwrap().metadata;
        assert_eq!(
            fetched_metadata.description,
            Some("Updated description".to_string())
        );
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

// The policy DTOs that lived here (CreatePolicyRequest, ApiPolicyRule,
// PolicyMetadata, PolicyResponse) went with the duplicate policy routes. The
// canonical ones are in handlers/policy.rs. `ApiPolicyRule` never had a single
// reader even before that.

/// Secret operations
#[axum::debug_handler]
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

    // Retrieve secret from engine service
    let secret_data = state
        .engine
        .get_secret(&path, &user.id.to_string())
        .await
        .map_err(|_e| ApiError::NotFound {
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

    // Create secret using engine service
    let secret_data = state
        .engine
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

    // Update secret using engine service
    let secret_data = state
        .engine
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
    // Delete secret using engine service
    state
        .engine
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
    // List secrets using engine service
    let paths = state
        .engine
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
    // Create key using engine service
    let key_info = state
        .engine
        .create_key(
            &request.name,
            &request.key_type,
            request.metadata.clone().unwrap_or_default(),
            &user.id.to_string(),
        )
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
        metadata: key_info.metadata,
        version: key_info.version,
        created_at: key_info.created_at,
        status: "active".to_string(),
        public_key: key_info.public_key,
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
    // Get key using engine service
    let key_info = state
        .engine
        .get_key(&key_id, &user.id.to_string())
        .await
        .map_err(|_e| ApiError::NotFound {
            resource: format!("Key '{}'", key_id),
        })?;

    let key = KeyResponse {
        id: key_info.id,
        name: key_info.name,
        key_type: key_info.key_type.clone(),
        algorithm: key_info.key_type,
        size: 256,
        usage: vec!["sign".to_string(), "verify".to_string()],
        metadata: key_info.metadata,
        version: key_info.version,
        created_at: key_info.created_at,
        status: "active".to_string(),
        public_key: key_info.public_key,
    };

    Ok(Json(ApiResponse::success(key)))
}

pub async fn list_keys(
    State(state): State<AppState>,
    Query(_query): Query<ListQuery>,
    user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<Vec<KeyResponse>>>> {
    // List keys using engine service
    let key_infos = state
        .engine
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
            metadata: key_info.metadata,
            version: key_info.version,
            created_at: key_info.created_at,
            status: "active".to_string(),
            public_key: key_info.public_key,
        })
        .collect();

    Ok(Json(ApiResponse::success(keys)))
}

pub async fn rotate_key(
    State(state): State<AppState>,
    Path(key_id): Path<String>,
    user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<KeyResponse>>> {
    // Rotate key using engine service
    let key_info = state
        .engine
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
        metadata: key_info.metadata,
        version: key_info.version,
        created_at: key_info.created_at,
        status: "active".to_string(),
        public_key: key_info.public_key,
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
    // Encrypt using engine service
    let result = state
        .engine
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
    // Decrypt using engine service
    let result = state
        .engine
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
    // Sign using engine service
    let result = state
        .engine
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
    // Verify using engine service
    let result = state
        .engine
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
    // Hash using engine service
    let result = state
        .engine
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
    State(state): State<AppState>,
    Path(key_id): Path<String>,
    headers: axum::http::HeaderMap,
    Json(metadata): Json<KeyMetadata>,
) -> ApiResult<Json<ApiResponse<KeyResponse>>> {
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

    // Update key metadata using engine service
    let key_info = state
        .engine
        .update_key(&key_id, metadata, &user.id.to_string())
        .await
        .map_err(|e| ApiError::Internal {
            message: format!("Failed to update key: {}", e),
        })?;

    let key = KeyResponse {
        id: key_info.id.clone(),
        name: key_info.name,
        key_type: key_info.key_type.clone(),
        algorithm: key_info.key_type,
        size: 256,
        usage: vec!["sign".to_string(), "verify".to_string()],
        metadata: key_info.metadata,
        version: key_info.version,
        created_at: key_info.created_at,
        status: "active".to_string(),
        public_key: Some("-----BEGIN PUBLIC KEY-----\n...\n-----END PUBLIC KEY-----".to_string()),
    };

    // Audit log
    let audit_entry = create_audit_log("key_updated", &user.username, "key", &key.id);
    let _ = state.audit.log(audit_entry).await;

    Ok(Json(ApiResponse::success(key)))
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

    // Delete key using engine service (includes safeguards)
    state
        .engine
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
    State(state): State<AppState>,
    Path(key_id): Path<String>,
    headers: axum::http::HeaderMap,
) -> ApiResult<Json<ApiResponse<Vec<KeyResponse>>>> {
    tracing::debug!(key_id = %key_id, "Listing key versions");

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

    // List key versions using engine service
    let key_infos = state
        .engine
        .list_key_versions(&key_id, &user.id.to_string())
        .await
        .map_err(|e| ApiError::Internal {
            message: format!("Failed to list key versions: {}", e),
        })?;

    let keys: Vec<KeyResponse> = key_infos
        .into_iter()
        .map(|key_info| KeyResponse {
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
            public_key: key_info.public_key,
        })
        .collect();

    Ok(Json(ApiResponse::success(keys)))
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
        Err(_e) => Err(ApiError::Internal {
            message: format!("Failed to get backup: {}", _e),
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
    State(_state): State<AppState>,
    Path(backup_id): Path<String>,
) -> ApiResult<Json<ApiResponse<()>>> {
    // Backup deletion requires persistent storage implementation
    tracing::warn!(backup_id = %backup_id, "Backup deletion requested but not implemented");
    Err(ApiError::BadRequest {
        message: "Backup deletion not yet implemented - persistent storage required".to_string(),
    })
}
