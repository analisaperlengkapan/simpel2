//! CI/CD Environment Injection Handler
//!
//! This module provides endpoints for injecting secrets as environment variables
//! for CI/CD pipelines with automatic cleanup after job completion.

use axum::{
    Json,
    extract::{Extension, Path, State},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

use secreton_storage::{QueryParams, SecretEntry, SecurityLevel};

use crate::error::ApiError;
use crate::middleware::RequestContext;
use crate::response::ApiResponse;

/// Request to inject secrets as environment variables
#[derive(Debug, Deserialize, Serialize)]
pub struct InjectEnvRequest {
    /// Paths to secrets to inject
    pub secrets: Vec<SecretPath>,

    /// Job identifier for tracking
    pub job_id: String,

    /// TTL for the injection session in seconds (default: 3600)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u64>,

    /// Prefix for environment variable names (default: "SECRET_")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,

    /// Format for environment variables (flat, nested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<EnvFormat>,
}

/// Secret path configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecretPath {
    /// Path to the secret in Secreton
    pub path: String,

    /// Optional key within the secret (if not specified, all keys are injected)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// Optional custom environment variable name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_name: Option<String>,
}

/// Environment variable format
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum EnvFormat {
    /// Flat format: SECRET_KEY=value
    Flat,
    /// Nested format: SECRET_PATH_KEY=value
    Nested,
}

impl Default for EnvFormat {
    fn default() -> Self {
        Self::Flat
    }
}

/// Response containing environment variables
#[derive(Debug, Serialize)]
pub struct InjectEnvResponse {
    /// Session ID for cleanup
    pub session_id: String,

    /// Environment variables to inject
    pub env_vars: HashMap<String, String>,

    /// Expiration time (RFC3339)
    pub expires_at: String,

    /// Cleanup endpoint
    pub cleanup_url: String,
}

/// Request to cleanup an injection session
#[derive(Debug, Deserialize)]
pub struct CleanupRequest {
    /// Session ID to cleanup
    pub session_id: String,
}

/// Injection session tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectionSession {
    pub id: String,
    pub job_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub secret_paths: Vec<String>,
    pub active: bool,
    pub created_by: String,
}

/// Inject secrets as environment variables
///
/// POST /v1/inject/env
pub async fn inject_env(
    State(state): State<Arc<crate::services::ServiceContainer>>,
    Extension(ctx): Extension<RequestContext>,
    Json(request): Json<InjectEnvRequest>,
) -> Result<Json<ApiResponse<InjectEnvResponse>>, ApiError> {
    info!(
        "Injecting secrets for job {} with {} secret paths",
        request.job_id,
        request.secrets.len()
    );

    // Generate session ID
    let session_id = Uuid::new_v4().to_string();

    // Calculate expiration
    let ttl = request.ttl.unwrap_or(3600);
    if ttl < 1 || ttl > 86400 {
        return Err(ApiError::BadRequest {
            message: "TTL must be between 1 and 86400 seconds".to_string(),
        });
    }
    let expires_at = chrono::Utc::now() + chrono::Duration::seconds(ttl as i64);

    // Get prefix
    let prefix = request.prefix.unwrap_or_else(|| "SECRET_".to_string());

    // Get format
    let format = request.format.unwrap_or_default();

    // Fetch secrets and build environment variables
    let mut env_vars = HashMap::new();
    let mut secret_paths = Vec::new();

    // Build policy context for authorization
    let user_id = ctx.user_id.as_deref().unwrap_or("anonymous");
    let policy_context = serde_json::json!({
        "user_id": ctx.user_id,
        "user_email": ctx.user_email,
        "user_roles": ctx.user_roles,
        "client_ip": ctx.client_ip.clone().unwrap_or_else(|| "unknown".to_string()),
        "request_id": ctx.request_id,
        "mfa_passed": ctx
            .jwt_claims
            .as_ref()
            .and_then(|c| c.metadata.get("mfa_passed"))
            .map(|v| v == "true")
            .unwrap_or(false),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    for secret_config in &request.secrets {
        secret_paths.push(secret_config.path.clone());

        // Authorization check
        {
            let policy_set = state.policy.read().map_err(|_| ApiError::Internal {
                message: "Failed to acquire policy lock".to_string(),
            })?;

            if !policy_set.evaluate(user_id, &secret_config.path, "read", Some(&policy_context)) {
                return Err(ApiError::Forbidden);
            }
        }

        // Fetch secret from Secreton
        let secret_data = fetch_secret(&state, &secret_config.path, user_id).await?;

        // Process based on configuration
        if let Some(key) = &secret_config.key {
            // Single key requested
            if let Some(value) = secret_data.get(key) {
                let env_name = secret_config
                    .env_name
                    .clone()
                    .unwrap_or_else(|| format!("{}{}", prefix, key.to_uppercase()));
                env_vars.insert(env_name, value.clone());
            } else {
                return Err(ApiError::BadRequest {
                    message: format!("Key '{}' not found in secret '{}'", key, secret_config.path),
                });
            }
        } else {
            // All keys requested
            for (key, value) in secret_data {
                let env_name = match &format {
                    EnvFormat::Flat => {
                        format!("{}{}", prefix, key.to_uppercase())
                    }
                    EnvFormat::Nested => {
                        let path_part = secret_config
                            .path
                            .trim_start_matches('/')
                            .replace('/', "_")
                            .to_uppercase();
                        format!("{}{}_{}", prefix, path_part, key.to_uppercase())
                    }
                };
                env_vars.insert(env_name, value.clone());
            }
        }
    }

    // Create session
    let session = InjectionSession {
        id: session_id.clone(),
        job_id: request.job_id.clone(),
        created_at: chrono::Utc::now(),
        expires_at,
        secret_paths,
        active: true,
        created_by: user_id.to_string(),
    };

    // Store session for tracking
    store_session(&state, &session).await?;

    // Schedule automatic cleanup
    schedule_cleanup(state, &session_id, ttl).await;

    let response = InjectEnvResponse {
        session_id: session_id.clone(),
        env_vars,
        expires_at: expires_at.to_rfc3339(),
        cleanup_url: format!("/v1/inject/cleanup/{}", session_id),
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Cleanup an injection session
///
/// DELETE /v1/inject/cleanup/{session_id}
pub async fn cleanup_session(
    State(state): State<Arc<crate::services::ServiceContainer>>,
    Extension(ctx): Extension<RequestContext>,
    Path(session_id): Path<String>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    info!("Cleaning up injection session {}", session_id);

    // Retrieve session first
    let session = get_session(&state, &session_id).await?;

    // Authorization check: Allow if owner OR if has delete permission
    let user_id = ctx.user_id.as_deref().unwrap_or("anonymous");
    // Prevent "anonymous" users from claiming ownership
    let is_owner = session.created_by == user_id && user_id != "anonymous";

    if !is_owner {
        let policy_set = state.policy.read().map_err(|_| ApiError::Internal {
            message: "Failed to acquire policy lock".to_string(),
        })?;
        if !policy_set.evaluate(user_id, "sys/inject/sessions", "delete", None) {
            return Err(ApiError::Forbidden);
        }
    }

    if !session.active {
        return Err(ApiError::NotFound {
            resource: format!("Active session {}", session_id),
        });
    }

    // Mark session as inactive
    deactivate_session(&state, &session_id).await?;

    // Audit log the cleanup
    audit_cleanup(&session).await;

    info!("Successfully cleaned up session {}", session_id);

    Ok(Json(ApiResponse::success(())))
}

/// List active injection sessions
///
/// GET /v1/inject/sessions
pub async fn list_sessions(
    State(state): State<Arc<crate::services::ServiceContainer>>,
    Extension(ctx): Extension<RequestContext>,
) -> Result<Json<ApiResponse<Vec<InjectionSession>>>, ApiError> {
    // Authorization check
    let user_id = ctx.user_id.as_deref().unwrap_or("anonymous");
    {
        let policy_set = state.policy.read().map_err(|_| ApiError::Internal {
            message: "Failed to acquire policy lock".to_string(),
        })?;
        if !policy_set.evaluate(user_id, "sys/inject/sessions", "list", None) {
            return Err(ApiError::Forbidden);
        }
    }

    let sessions = list_active_sessions(&state).await?;

    Ok(Json(ApiResponse::success(sessions)))
}

/// Get injection session details
///
/// GET /v1/inject/sessions/{session_id}
pub async fn get_session_details(
    State(state): State<Arc<crate::services::ServiceContainer>>,
    Extension(ctx): Extension<RequestContext>,
    Path(session_id): Path<String>,
) -> Result<Json<ApiResponse<InjectionSession>>, ApiError> {
    // Authorization check
    let user_id = ctx.user_id.as_deref().unwrap_or("anonymous");
    {
        let policy_set = state.policy.read().map_err(|_| ApiError::Internal {
            message: "Failed to acquire policy lock".to_string(),
        })?;
        if !policy_set.evaluate(user_id, "sys/inject/sessions", "read", None) {
            return Err(ApiError::Forbidden);
        }
    }

    let session = get_session(&state, &session_id).await?;

    Ok(Json(ApiResponse::success(session)))
}

// Helper functions (to be implemented with actual storage)

async fn fetch_secret(
    state: &Arc<crate::services::ServiceContainer>,
    path: &str,
    user_id: &str,
) -> Result<HashMap<String, String>, ApiError> {
    info!("Fetching secret from path: {}", path);

    let secret_data = state
        .engine
        .get_secret(path, user_id)
        .await
        .map_err(|e| match e {
            crate::services::secret_engine::SecretServiceError::SecretNotFound { .. } => {
                ApiError::NotFound {
                    resource: path.to_string(),
                }
            }
            crate::services::secret_engine::SecretServiceError::PermissionDenied(_msg) => {
                ApiError::Forbidden
            }
            _ => ApiError::Internal {
                message: e.to_string(),
            },
        })?;

    Ok(secret_data.data)
}

const SESSION_PREFIX: &str = "sys/inject/sessions/";

async fn store_session(
    state: &Arc<crate::services::ServiceContainer>,
    session: &InjectionSession,
) -> Result<(), ApiError> {
    info!("Storing injection session: {}", session.id);
    let path = format!("{}{}", SESSION_PREFIX, session.id);
    let serialized = serde_json::to_vec(session).map_err(|e| ApiError::Internal {
        message: e.to_string(),
    })?;

    let entry = SecretEntry::new(
        path,
        serialized,
        serde_json::json!({}),
        SecurityLevel::Confidential,
        "system".to_string(),
    );

    state
        .storage
        .store(&entry)
        .await
        .map_err(ApiError::Storage)?;
    Ok(())
}

async fn get_session_internal(
    state: &Arc<crate::services::ServiceContainer>,
    session_id: &str,
) -> Result<InjectionSession, ApiError> {
    info!("Retrieving session: {}", session_id);
    let path = format!("{}{}", SESSION_PREFIX, session_id);

    let entry = state
        .storage
        .get_by_path(&path)
        .await
        .map_err(ApiError::Storage)?
        .ok_or_else(|| ApiError::NotFound {
            resource: format!("Session {}", session_id),
        })?;

    let session: InjectionSession =
        serde_json::from_slice(&entry.encrypted_data).map_err(|e| ApiError::Internal {
            message: format!("Failed to deserialize session: {}", e),
        })?;

    Ok(session)
}

async fn get_session(
    state: &Arc<crate::services::ServiceContainer>,
    session_id: &str,
) -> Result<InjectionSession, ApiError> {
    let mut session = get_session_internal(state, session_id).await?;

    // Check expiration (lazy cleanup)
    if session.active && chrono::Utc::now() > session.expires_at {
        info!("Session {} expired, lazily deactivating", session_id);
        if let Err(e) = deactivate_session(state, session_id).await {
            error!("Failed to lazily deactivate session {}: {}", session_id, e);
        }
        session.active = false;
    }

    Ok(session)
}

async fn deactivate_session(
    state: &Arc<crate::services::ServiceContainer>,
    session_id: &str,
) -> Result<(), ApiError> {
    info!("Deactivating session: {}", session_id);
    let mut session = get_session_internal(state, session_id).await?;
    session.active = false;
    store_session(state, &session).await
}

async fn list_active_sessions(
    state: &Arc<crate::services::ServiceContainer>,
) -> Result<Vec<InjectionSession>, ApiError> {
    let params = QueryParams {
        path_prefix: Some(SESSION_PREFIX.to_string()),
        ..Default::default()
    };

    let entries = state
        .storage
        .list(&params)
        .await
        .map_err(ApiError::Storage)?;
    let mut sessions = Vec::new();
    let now = chrono::Utc::now();

    for entry in entries {
        if let Ok(session) = serde_json::from_slice::<InjectionSession>(&entry.encrypted_data) {
            // Check if active and not expired
            if session.active {
                if now > session.expires_at {
                    // Lazy cleanup for listed items
                    let _ = deactivate_session(state, &session.id).await;
                    continue;
                }
                sessions.push(session);
            }
        }
    }

    Ok(sessions)
}

async fn schedule_cleanup(
    state: Arc<crate::services::ServiceContainer>,
    session_id: &str,
    ttl: u64,
) {
    let session_id = session_id.to_string();

    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(ttl)).await;

        info!("Auto-cleanup triggered for session {}", session_id);

        if let Err(e) = deactivate_session(&state, &session_id).await {
            error!("Failed to auto-cleanup session {}: {}", session_id, e);
        }
    });
}

async fn audit_cleanup(session: &InjectionSession) {
    info!(
        "Audit: Session {} for job {} cleaned up",
        session.id, session.job_id
    );
    // TODO: Write to audit log
}

/// Create routes for CI/CD injection endpoints
pub fn create_routes() -> axum::Router<std::sync::Arc<crate::services::ServiceContainer>> {
    use axum::routing::{delete, get, post};

    axum::Router::new()
        .route("/env", post(inject_env))
        .route("/cleanup/{session_id}", delete(cleanup_session))
        .route("/sessions", get(list_sessions))
        .route("/sessions/{session_id}", get(get_session_details))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_format_default() {
        let format = EnvFormat::default();
        assert!(matches!(format, EnvFormat::Flat));
    }

    #[test]
    fn test_secret_path_serialization() {
        let path = SecretPath {
            path: "/secret/data/myapp".to_string(),
            key: Some("password".to_string()),
            env_name: Some("DB_PASSWORD".to_string()),
        };

        let json = serde_json::to_string(&path).unwrap();
        assert!(json.contains("password"));
        assert!(json.contains("DB_PASSWORD"));
    }

    #[test]
    fn test_inject_env_request_deserialization() {
        let json = r#"{
            "secrets": [
                {
                    "path": "/secret/data/db",
                    "key": "password",
                    "env_name": "DB_PASSWORD"
                }
            ],
            "job_id": "ci-job-123",
            "ttl": 3600
        }"#;

        let request: InjectEnvRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.job_id, "ci-job-123");
        assert_eq!(request.secrets.len(), 1);
        assert_eq!(request.ttl, Some(3600));
    }

    #[tokio::test]
    async fn test_session_lifecycle() {
        use deadpool_postgres::{Config, Runtime};
        use secreton_storage::MemoryBackend;
        use tokio_postgres::NoTls;

        // Setup mock services
        let storage = Arc::new(MemoryBackend::new());
        let mut cfg = Config::new();
        cfg.dbname = Some("test".to_string());
        // We need a pool even if unused by session storage
        let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();

        let state = Arc::new(crate::services::ServiceContainer::new_mock(storage, pool));

        let session = InjectionSession {
            id: "test-session".to_string(),
            job_id: "test-job".to_string(),
            created_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
            secret_paths: vec!["/secret/data/test".to_string()],
            active: true,
            created_by: "user1".to_string(),
        };

        // Test store
        assert!(store_session(&state, &session).await.is_ok());

        // Test retrieve
        let retrieved = get_session(&state, &session.id).await.unwrap();
        assert_eq!(retrieved.id, session.id);

        // Test deactivate
        assert!(deactivate_session(&state, &session.id).await.is_ok());

        // Test verify inactive
        let inactive = get_session(&state, &session.id).await.unwrap();
        assert!(!inactive.active);
    }
}
