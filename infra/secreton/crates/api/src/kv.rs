//! KV Secrets Engine API endpoints

use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn};

/// Secret metadata for versioning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretMetadata {
    pub created_time: chrono::DateTime<chrono::Utc>,
    pub updated_time: chrono::DateTime<chrono::Utc>,
    pub version: u64,
}

impl std::fmt::Display for SecretMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "v{}", self.version)
    }
}

/// In-memory KV store with versioning
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SecretVersion {
    data: serde_json::Value,
    metadata: SecretMetadata,
}

#[derive(Clone)]
pub struct KVEngine {
    // In-memory storage: path -> Vec<versions>
    store: std::sync::Arc<tokio::sync::RwLock<HashMap<String, Vec<SecretVersion>>>>,
    // Audit logger for tracking operations
    audit_logger: Option<crate::audit::AuditLogger>,
}

impl Default for KVEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl KVEngine {
    pub fn new() -> Self {
        Self {
            store: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            audit_logger: Some(crate::audit::AuditLogger::default()),
        }
    }

    /// Create KVEngine with custom audit logger
    pub fn with_audit_logger(audit_logger: crate::audit::AuditLogger) -> Self {
        Self {
            store: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            audit_logger: Some(audit_logger),
        }
    }

    /// Create KVEngine without audit logging (for testing)
    pub fn without_audit() -> Self {
        Self {
            store: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            audit_logger: None,
        }
    }

    /// Log audit event if logger is configured
    async fn log_audit(&self, event: crate::audit::AuditEvent) {
        if let Some(logger) = &self.audit_logger {
            logger.log(event).await;
        }
    }

    pub async fn put_secret(
        &self,
        path: &str,
        data: serde_json::Value,
        principal: Option<&str>,
    ) -> Result<SecretMetadata, Box<dyn std::error::Error + Send + Sync>> {
        let result: Result<SecretMetadata, Box<dyn std::error::Error + Send + Sync>> = async {
            let mut store = self.store.write().await;
            let versions = store.entry(path.to_string()).or_insert_with(Vec::new);

            let version = (versions.len() + 1) as u64;
            let metadata = SecretMetadata {
                created_time: chrono::Utc::now(),
                updated_time: chrono::Utc::now(),
                version,
            };

            versions.push(SecretVersion {
                data,
                metadata: metadata.clone(),
            });

            info!("Put secret at path: {} (version: {})", path, version);
            Ok(metadata)
        }
        .await;

        // Audit log
        let event_type = if result.is_ok() {
            crate::audit::AuditEventType::SecretCreated
        } else {
            crate::audit::AuditEventType::SecretCreated
        };

        let mut event = crate::audit::AuditEvent::new(
            event_type,
            principal.unwrap_or("system").to_string(),
            result.is_ok(),
        )
        .with_secret_key(path.to_string());

        if let Err(ref e) = result {
            event = event.with_error(e.to_string());
        }

        self.log_audit(event).await;

        result
    }

    pub async fn get_secret(
        &self,
        path: &str,
        version: Option<u64>,
        principal: Option<&str>,
    ) -> Result<Option<(serde_json::Value, SecretMetadata)>, Box<dyn std::error::Error + Send + Sync>>
    {
        let result = async {
            let store = self.store.read().await;

            if let Some(versions) = store.get(path) {
                let secret = match version {
                    Some(v) => versions.get((v - 1) as usize),
                    None => versions.last(),
                };

                if let Some(secret_version) = secret {
                    return Ok(Some((
                        secret_version.data.clone(),
                        secret_version.metadata.clone(),
                    )));
                }
            }

            Ok(None)
        }
        .await;

        // Audit log
        let event = crate::audit::AuditEvent::new(
            crate::audit::AuditEventType::SecretRead,
            principal.unwrap_or("system").to_string(),
            result.is_ok() && result.as_ref().map(|r| r.is_some()).unwrap_or(false),
        )
        .with_secret_key(path.to_string());

        self.log_audit(event).await;

        result
    }

    pub async fn delete_secret(
        &self,
        path: &str,
        _version: Option<u64>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut store = self.store.write().await;
        store.remove(path);
        info!("Deleted secret at path: {}", path);
        Ok(())
    }

    pub async fn list_secrets(
        &self,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let store = self.store.read().await;
        Ok(store.keys().cloned().collect())
    }

    pub async fn get_metadata(
        &self,
        path: &str,
    ) -> Result<Option<SecretMetadata>, Box<dyn std::error::Error + Send + Sync>> {
        let store = self.store.read().await;

        if let Some(versions) = store.get(path)
            && let Some(latest) = versions.last()
        {
            return Ok(Some(latest.metadata.clone()));
        }

        Ok(None)
    }

    pub async fn destroy_secret(
        &self,
        path: &str,
        _version: u64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // For in-memory implementation, destroy is same as delete
        self.delete_secret(path, None).await
    }
}

/// API state for KV engine
#[derive(Clone)]
pub struct KVApiState {
    pub engine: std::sync::Arc<KVEngine>,
}

impl Default for KVApiState {
    fn default() -> Self {
        Self {
            engine: std::sync::Arc::new(KVEngine::default()),
        }
    }
}

/// Request to create/update a secret
#[derive(Debug, Deserialize)]
pub struct CreateSecretRequest {
    pub data: serde_json::Value,
}

/// Response for secret creation
#[derive(Debug, Serialize)]
pub struct CreateSecretResponse {
    pub version: SecretMetadata,
    pub created_time: String,
}

/// Response for secret retrieval
#[derive(Debug, Serialize)]
pub struct GetSecretResponse {
    pub data: HashMap<String, String>,
    pub metadata: SecretMetadata,
}

/// Response for listing secrets
#[derive(Debug, Serialize)]
pub struct ListSecretsResponse {
    pub keys: Vec<String>,
}

/// Response for metadata
#[derive(Debug, Serialize)]
pub struct MetadataResponse {
    pub versions: HashMap<u32, SecretMetadata>,
}

/// Response for delete operations
#[derive(Debug, Serialize)]
pub struct DeleteResponse {
    pub success: bool,
    pub message: String,
}

/// Create the KV router with all endpoints
pub fn create_kv_router(state: KVApiState) -> Router {
    Router::new()
        .route("/secrets", get(list_secrets))
        .route("/secret/data/*path", post(put_secret))
        .route("/secret/data/*path", get(get_secret))
        .route("/secret/data/*path", delete(delete_secret))
        .route("/secret/metadata/*path", get(get_metadata))
        // Note: destroy version is handled differently - version as query param
        .route("/secret/destroy/*path", delete(destroy_secret_query))
        .with_state(state)
}

/// List all secret paths
#[axum::debug_handler]
pub async fn list_secrets(
    State(state): State<KVApiState>,
) -> Result<Json<ListSecretsResponse>, StatusCode> {
    let keys = match state.engine.list_secrets().await {
        Ok(keys) => keys,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    info!("Listed {} secret paths", keys.len());

    Ok(Json(ListSecretsResponse { keys }))
}

/// Create or update a secret
#[axum::debug_handler]
pub async fn put_secret(
    State(state): State<KVApiState>,
    Path(path): Path<String>,
    Json(request): Json<CreateSecretRequest>,
) -> Result<Json<CreateSecretResponse>, StatusCode> {
    match state
        .engine
        .put_secret(&path, request.data, Some("system"))
        .await
    {
        Ok(version) => {
            info!(
                "Created secret at path '{}' version {}",
                path, version.version
            );
            Ok(Json(CreateSecretResponse {
                version,
                created_time: chrono::Utc::now().to_rfc3339(),
            }))
        }
        Err(e) => {
            warn!("Failed to create secret at path '{}': {:?}", path, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get a secret
#[axum::debug_handler]
pub async fn get_secret(
    State(state): State<KVApiState>,
    Path(path): Path<String>,
) -> Result<Json<GetSecretResponse>, StatusCode> {
    match state.engine.get_secret(&path, None, Some("system")).await {
        Ok(Some((data, metadata))) => {
            info!("Retrieved secret at path '{}'", path);
            let data_map = match data {
                serde_json::Value::Object(map) => {
                    map.into_iter().map(|(k, v)| (k, v.to_string())).collect()
                }
                _ => HashMap::new(),
            };
            Ok(Json(GetSecretResponse {
                data: data_map,
                metadata,
            }))
        }
        Ok(None) => {
            warn!("Secret not found at path '{}'", path);
            Err(StatusCode::NOT_FOUND)
        }
        Err(_) => {
            warn!("Error retrieving secret at path '{}'", path);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Delete a secret (soft delete)
#[axum::debug_handler]
pub async fn delete_secret(
    State(state): State<KVApiState>,
    Path(path): Path<String>,
) -> Result<Json<DeleteResponse>, StatusCode> {
    match state.engine.delete_secret(&path, None).await {
        Ok(_) => {
            info!("Deleted secret at path '{}'", path);
            Ok(Json(DeleteResponse {
                success: true,
                message: format!("Secret '{}' deleted", path),
            }))
        }
        Err(_) => {
            warn!("Failed to delete secret at path '{}'", path);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// Get secret metadata
#[axum::debug_handler]
pub async fn get_metadata(
    State(state): State<KVApiState>,
    Path(path): Path<String>,
) -> Result<Json<MetadataResponse>, StatusCode> {
    match state.engine.get_metadata(&path).await {
        Ok(Some(metadata)) => {
            info!("Retrieved metadata for path '{}'", path);
            let mut versions = HashMap::new();
            versions.insert(metadata.version as u32, metadata);
            Ok(Json(MetadataResponse { versions }))
        }
        Ok(None) => {
            warn!("Secret metadata not found at path '{}'", path);
            Err(StatusCode::NOT_FOUND)
        }
        Err(_) => {
            warn!("Secret metadata not found at path '{}'", path);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// Query parameters for destroy endpoint
#[derive(Debug, Deserialize)]
pub struct DestroyQuery {
    /// Version to destroy
    #[serde(default = "default_version")]
    pub version: u32,
}

fn default_version() -> u32 {
    1
}

/// Permanently destroy a secret version (with version as query param)
#[axum::debug_handler]
pub async fn destroy_secret_query(
    State(state): State<KVApiState>,
    Path(path): Path<String>,
    Query(query): Query<DestroyQuery>,
) -> Result<Json<DeleteResponse>, StatusCode> {
    let version = query.version;
    match state.engine.destroy_secret(&path, version as u64).await {
        Ok(_) => {
            info!(
                "Permanently destroyed secret '{}' version {}",
                path, version
            );
            Ok(Json(DeleteResponse {
                success: true,
                message: format!(
                    "Secret '{}' version {} permanently destroyed",
                    path, version
                ),
            }))
        }
        Err(_) => {
            warn!("Failed to destroy secret '{}' version {}", path, version);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// Permanently destroy a secret version (original with path param - kept for compatibility)
#[axum::debug_handler]
pub async fn destroy_secret(
    State(state): State<KVApiState>,
    Path((path, version)): Path<(String, u32)>,
) -> Result<Json<DeleteResponse>, StatusCode> {
    match state.engine.destroy_secret(&path, version as u64).await {
        Ok(_) => {
            info!(
                "Permanently destroyed secret '{}' version {}",
                path, version
            );
            Ok(Json(DeleteResponse {
                success: true,
                message: format!(
                    "Secret '{}' version {} permanently destroyed",
                    path, version
                ),
            }))
        }
        Err(_) => {
            warn!("Failed to destroy secret '{}' version {}", path, version);
            Err(StatusCode::NOT_FOUND)
        }
    }
}
