//! CI/CD Environment Injection Handler
//!
//! This module provides endpoints for injecting secrets as environment variables
//! for CI/CD pipelines with automatic cleanup after job completion.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

use crate::error::ApiError;
use crate::extractors::AuthenticatedUser;
use crate::response::ApiResponse;
use crate::services::engine::SecretServiceError;

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
#[derive(Debug, Serialize, Deserialize)]
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
}

/// Inject secrets as environment variables
///
/// POST /v1/inject/env
pub async fn inject_env(
    State(state): State<Arc<crate::services::ServiceContainer>>,
    user: AuthenticatedUser,
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
    let expires_at = chrono::Utc::now() + chrono::Duration::seconds(ttl as i64);

    // Get prefix
    let prefix = request.prefix.unwrap_or_else(|| "SECRET_".to_string());

    // Get format
    let format = request.format.unwrap_or_default();

    // Fetch secrets and build environment variables
    let mut env_vars = HashMap::new();
    let mut secret_paths = Vec::new();

    for secret_config in &request.secrets {
        secret_paths.push(secret_config.path.clone());

        // Fetch secret from Secreton
        let secret_data = fetch_secret(
            &state.engine,
            &secret_config.path,
            &user.id.to_string(),
        )
        .await?;

        // Process based on configuration
        if let Some(key) = &secret_config.key {
            // Single key requested
            if let Some(value) = secret_data.get(key) {
                let env_name = secret_config
                    .env_name
                    .clone()
                    .unwrap_or_else(|| format!("{}{}", prefix, key.to_uppercase()));
                env_vars.insert(env_name, value.clone());
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
                env_vars.insert(env_name, value);
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
    };

    // Store session for tracking
    // TODO: Store in actual session storage
    store_session(&session).await?;

    // Schedule automatic cleanup
    schedule_cleanup(&session_id, ttl).await;

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
    State(_state): State<Arc<crate::services::ServiceContainer>>,
    Path(session_id): Path<String>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    info!("Cleaning up injection session {}", session_id);

    // Retrieve session
    let session = get_session(&session_id).await?;

    if !session.active {
        return Err(ApiError::NotFound {
            resource: format!("Active session {}", session_id),
        });
    }

    // Mark session as inactive
    deactivate_session(&session_id).await?;

    // Audit log the cleanup
    audit_cleanup(&session).await;

    info!("Successfully cleaned up session {}", session_id);

    Ok(Json(ApiResponse::success(())))
}

/// List active injection sessions
///
/// GET /v1/inject/sessions
pub async fn list_sessions(
    State(_state): State<Arc<crate::services::ServiceContainer>>,
) -> Result<Json<ApiResponse<Vec<InjectionSession>>>, ApiError> {
    // TODO: Implement actual session listing
    let sessions = list_active_sessions().await?;

    Ok(Json(ApiResponse::success(sessions)))
}

/// Get injection session details
///
/// GET /v1/inject/sessions/{session_id}
pub async fn get_session_details(
    State(_state): State<Arc<crate::services::ServiceContainer>>,
    Path(session_id): Path<String>,
) -> Result<Json<ApiResponse<InjectionSession>>, ApiError> {
    let session = get_session(&session_id).await?;

    Ok(Json(ApiResponse::success(session)))
}

// Helper functions (to be implemented with actual storage)

async fn fetch_secret(
    engine: &crate::services::engine::SecretService,
    path: &str,
    user_id: &str,
) -> Result<HashMap<String, String>, ApiError> {
    info!("Fetching secret from path: {}", path);

    let secret_data = engine.get_secret(path, user_id).await.map_err(|e| match e {
        SecretServiceError::SecretNotFound { path } => ApiError::NotFound {
            resource: format!("Secret {}", path),
        },
        SecretServiceError::PermissionDenied(_) => ApiError::Forbidden,
        e => ApiError::Internal {
            message: anyhow::anyhow!(e).to_string(),
        },
    })?;

    Ok(secret_data.data)
}

async fn store_session(session: &InjectionSession) -> Result<(), ApiError> {
    // TODO: Store in Redis or PostgreSQL
    info!("Storing injection session: {}", session.id);
    Ok(())
}

async fn get_session(session_id: &str) -> Result<InjectionSession, ApiError> {
    // TODO: Retrieve from storage
    info!("Retrieving session: {}", session_id);

    // Mock session for now
    Ok(InjectionSession {
        id: session_id.to_string(),
        job_id: "mock-job".to_string(),
        created_at: chrono::Utc::now(),
        expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
        secret_paths: vec![],
        active: true,
    })
}

async fn deactivate_session(session_id: &str) -> Result<(), ApiError> {
    // TODO: Update in storage
    info!("Deactivating session: {}", session_id);
    Ok(())
}

async fn list_active_sessions() -> Result<Vec<InjectionSession>, ApiError> {
    // TODO: Query from storage
    Ok(vec![])
}

async fn schedule_cleanup(session_id: &str, ttl: u64) {
    let session_id = session_id.to_string();

    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(ttl)).await;

        info!("Auto-cleanup triggered for session {}", session_id);

        if let Err(e) = deactivate_session(&session_id).await {
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
        let session = InjectionSession {
            id: "test-session".to_string(),
            job_id: "test-job".to_string(),
            created_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
            secret_paths: vec!["/secret/data/test".to_string()],
            active: true,
        };

        // Test store
        assert!(store_session(&session).await.is_ok());

        // Test retrieve
        let retrieved = get_session(&session.id).await.unwrap();
        assert_eq!(retrieved.id, session.id);

        // Test deactivate
        assert!(deactivate_session(&session.id).await.is_ok());
    }

    #[tokio::test]
    async fn test_inject_env_integration() {
        use axum::http::{Request, StatusCode};
        use axum::routing::post;
        use axum::Router;
        use secreton_storage::MemoryBackend;
        use std::sync::Arc;
        use tower::ServiceExt;

        use crate::middleware::RequestContext;
        use crate::services::ServiceContainer;

        // 1. Setup mock service container
        let storage = Arc::new(MemoryBackend::new());
        let mut cfg = deadpool_postgres::Config::new();
        cfg.dbname = Some("test".to_string());
        cfg.host = Some("localhost".to_string());
        cfg.user = Some("test".to_string());
        cfg.password = Some("test".to_string());
        let pool = cfg.create_pool(None, tokio_postgres::NoTls).unwrap();
        let services = ServiceContainer::new_mock(storage, pool);

        // 2. Populate a secret
        let mut secret_data = std::collections::HashMap::new();
        secret_data.insert("password".to_string(), "s3cr3t".to_string());

        let user_id = "test-user";

        services
            .engine
            .put_secret(
                "app/test",
                secret_data,
                Default::default(),
                user_id,
                None,
            )
            .await
            .expect("Failed to put secret");

        let state = Arc::new(services);

        // 3. Create Router
        let app = Router::new()
            .route("/env", post(inject_env))
            .with_state(state);

        // 4. Create Request
        let payload = serde_json::json!({
            "job_id": "test-job",
            "secrets": [
                {
                    "path": "app/test",
                    "key": "password",
                    "env_name": "MY_PASSWORD"
                }
            ]
        });

        // Mock RequestContext
        let context = RequestContext {
            request_id: "req-1".to_string(),
            user_id: Some(user_id.to_string()),
            user_email: Some("test@example.com".to_string()),
            user_roles: vec![],
            user_permissions: vec![],
            start_time: std::time::Instant::now(),
            jwt_claims: None,
            auth_token: None,
            client_ip: None,
            user_agent: None,
            policy_names: vec![],
        };

        let request = Request::builder()
            .uri("/env")
            .method("POST")
            .header("content-type", "application/json")
            .extension(context)
            .body(axum::body::Body::from(serde_json::to_string(&payload).unwrap()))
            .unwrap();

        // 5. Send Request
        let response = app.oneshot(request).await.unwrap();

        // 6. Verify Response
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let body: ApiResponse<InjectEnvResponse> = serde_json::from_slice(&body_bytes).unwrap();

        assert!(body.success);
        let data = body.data.unwrap();
        assert_eq!(
            data.env_vars.get("MY_PASSWORD").map(|s| s.as_str()),
            Some("s3cr3t")
        );
    }
}
