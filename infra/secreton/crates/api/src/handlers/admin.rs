//! Administrative handlers for system management.
//!
//! Provides endpoints for user management, system configuration,
//! monitoring, and maintenance operations.

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
    handlers::{AppState, ListQuery},
};

/// Create administrative routes
pub fn create_routes() -> Router<AppState> {
    Router::new()
        // User management
        .route("/users", get(list_users))
        .route("/users", post(create_user))
        .route("/users/:user_id", get(get_user))
        .route("/users/:user_id", put(update_user))
        .route("/users/:user_id", delete(delete_user))
        .route("/users/:user_id/roles", get(get_user_roles))
        .route("/users/:user_id/roles", post(assign_user_roles))
        .route("/users/:user_id/permissions", get(get_user_permissions))
        // Role management
        .route("/roles", get(list_roles))
        .route("/roles", post(create_role))
        .route("/roles/:role_name", get(get_role))
        .route("/roles/:role_name", put(update_role))
        .route("/roles/:role_name", delete(delete_role))
        // System configuration
        .route("/config", get(get_config))
        .route("/config", put(update_config))
        .route("/config/reload", post(reload_config))
        // System monitoring
        .route("/metrics", get(get_system_metrics))
        .route("/status", get(get_system_status))
        .route("/logs", get(get_system_logs))
        // Maintenance operations
        .route("/maintenance/gc", post(run_garbage_collection))
        .route("/maintenance/compact", post(compact_database))
        .route("/maintenance/vacuum", post(vacuum_database))
        // Security operations
        .route("/security/scan", post(run_security_scan))
        .route("/security/reports", get(get_security_reports))
        .route("/security/incidents", get(get_security_incidents))
        .route(
            "/security/incidents/:incident_id",
            get(get_security_incident),
        )
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
        TestServer::new(app).expect("Failed to start test server")
    }

    #[tokio::test]
    async fn test_list_users_returns_placeholder_user() {
        let server = server_with_routes().await;
        let response = server.get("/users").await;
        response.assert_status_ok();

        let body: ApiResponse<Vec<UserResponse>> = response.json();
        assert!(body.success);
        let users = body.data.expect("users payload");
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].username, "admin");
    }

    #[tokio::test]
    async fn test_create_role_endpoint() {
        let server = server_with_routes().await;
        let request = CreateRoleRequest {
            name: "auditor".to_string(),
            description: Some("Audit role".to_string()),
            permissions: vec!["vault:read".to_string()],
            metadata: None,
        };

        let response = server.post("/roles").json(&request).await;
        response.assert_status_ok();

        let body: ApiResponse<RoleResponse> = response.json();
        assert!(body.success);
        let role = body.data.expect("role payload");
        assert_eq!(role.name, "auditor");
        assert!(role.permissions.contains(&"vault:read".to_string()));
    }

    #[tokio::test]
    async fn test_get_config_returns_security_info() {
        let server = server_with_routes().await;
        let response = server.get("/config").await;
        response.assert_status_ok();

        let body: ApiResponse<SystemConfig> = response.json();
        assert!(body.success);
        let config = body.data.expect("config payload");
        assert!(config.security.mfa_enabled);
        assert_eq!(config.api.version, "1.0.0");
    }

    #[tokio::test]
    async fn test_get_system_metrics_returns_real_uptime() {
        let server = server_with_routes().await;

        // Sleep briefly to ensure uptime > 0 (1.1s to be safe vs 1s granularity)
        tokio::time::sleep(tokio::time::Duration::from_millis(1100)).await;

        let response = server.get("/metrics").await;
        response.assert_status_ok();

        let body: ApiResponse<SystemMetrics> = response.json();
        assert!(body.success);
        let metrics = body.data.expect("metrics payload");

        // Uptime should be > 0 since we slept
        assert!(metrics.uptime > 0, "Uptime should be greater than 0");
        // And definitely not the hardcoded 86400 (1 day)
        assert!(metrics.uptime < 86400, "Uptime should not be hardcoded to 1 day");
    }

    #[tokio::test]
    async fn test_get_system_status_returns_real_uptime() {
        let server = server_with_routes().await;

        // Sleep briefly (1.1s)
        tokio::time::sleep(tokio::time::Duration::from_millis(1100)).await;

        let response = server.get("/status").await;
        response.assert_status_ok();

        let body: ApiResponse<SystemStatus> = response.json();
        assert!(body.success);
        let status = body.data.expect("status payload");

        assert!(status.uptime > 0, "Uptime should be greater than 0");
        assert!(status.uptime < 86400, "Uptime should not be hardcoded to 1 day");
    }

    #[tokio::test]
    async fn test_get_user_retrieves_real_data() {
        // Initialize services
        let config = ApiConfig::default();
        let services = Arc::new(
            ServiceContainer::new(&config)
                .await
                .expect("Failed to create services"),
        );

        // Create a test user via auth service
        let user = services
            .auth
            .create_user(
                "realuser",
                "real@example.com",
                "password123",
                Some("Real User"),
                vec!["user".to_string()],
                None,
                true,
            )
            .await
            .expect("Failed to create user");

        // Start server with these services
        let app = create_routes().with_state(services);
        let server = TestServer::new(app).expect("Failed to start test server");

        // Fetch the user via API
        let response = server.get(&format!("/users/{}", user.id)).await;
        response.assert_status_ok();

        let body: ApiResponse<UserResponse> = response.json();
        assert!(body.success);
        let fetched_user = body.data.expect("user payload");

        assert_eq!(fetched_user.id, user.id.to_string());
        assert_eq!(fetched_user.username, "realuser");
        assert_eq!(fetched_user.email, "real@example.com");
        assert_eq!(fetched_user.full_name, Some("Real User".to_string()));
        assert!(fetched_user.roles.contains(&"user".to_string()));
    }

    #[tokio::test]
    async fn test_get_user_with_multiple_roles_and_permissions() {
        let config = ApiConfig::default();
        let services = Arc::new(
            ServiceContainer::new(&config)
                .await
                .expect("Failed to create services"),
        );

        // Create user with multiple roles (admin has "*", user has basic)
        let user = services
            .auth
            .create_user(
                "poweruser",
                "power@example.com",
                "password123",
                None,
                vec!["admin".to_string(), "user".to_string()],
                None,
                true,
            )
            .await
            .expect("Failed to create user");

        let app = create_routes().with_state(services);
        let server = TestServer::new(app).expect("Failed to start test server");

        let response = server.get(&format!("/users/{}", user.id)).await;
        response.assert_status_ok();

        let body: ApiResponse<UserResponse> = response.json();
        let fetched_user = body.data.expect("user payload");

        // Admin role should grant "*" permission
        assert!(fetched_user.permissions.contains(&"*".to_string()));
        // Roles should be sorted
        assert_eq!(fetched_user.roles, vec!["admin".to_string(), "user".to_string()]);
    }

    #[tokio::test]
    async fn test_get_user_not_found() {
        let config = ApiConfig::default();
        let services = Arc::new(
            ServiceContainer::new(&config)
                .await
                .expect("Failed to create services"),
        );

        let app = create_routes().with_state(services);
        let server = TestServer::new(app).expect("Failed to start test server");

        // Random UUID
        let random_id = uuid::Uuid::new_v4();
        let response = server.get(&format!("/users/{}", random_id)).await;

        // Should return 404
        response.assert_status_not_found();
    }

    #[tokio::test]
    async fn test_create_user_persists() {
        let server = server_with_routes().await;
        let request = CreateUserRequest {
            username: "newuser".to_string(),
            email: "new@example.com".to_string(),
            password: "password123".to_string(),
            full_name: Some("New User".to_string()),
            roles: vec!["user".to_string()],
            enabled: Some(true),
            metadata: None,
        };

        let response = server.post("/users").json(&request).await;
        response.assert_status_ok();

        let body: ApiResponse<UserResponse> = response.json();
        assert!(body.success);
        let user = body.data.expect("user payload");
        assert_eq!(user.username, "newuser");

        // Verify we can fetch it
        let response = server.get(&format!("/users/{}", user.id)).await;
        response.assert_status_ok();
        let body: ApiResponse<UserResponse> = response.json();
        assert_eq!(body.data.unwrap().username, "newuser");
    }
}

/// User management models
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub full_name: Option<String>,
    pub roles: Vec<String>,
    pub enabled: Option<bool>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub full_name: Option<String>,
    pub enabled: Option<bool>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
    pub full_name: Option<String>,
    pub enabled: bool,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub last_login: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct AssignRolesRequest {
    pub roles: Vec<String>,
}

/// Role management models
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRoleRequest {
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoleResponse {
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    pub users: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, String>,
}

/// System configuration models
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemConfig {
    pub api: ApiConfigInfo,
    pub security: SecurityConfigInfo,
    pub storage: StorageConfigInfo,
    pub monitoring: MonitoringConfigInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiConfigInfo {
    pub version: String,
    pub bind_address: String,
    pub max_connections: u32,
    pub timeout: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityConfigInfo {
    pub mfa_enabled: bool,
    pub password_policy: PasswordPolicyInfo,
    pub session_timeout: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PasswordPolicyInfo {
    pub min_length: u8,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_special: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageConfigInfo {
    pub backend: String,
    pub encryption_enabled: bool,
    pub backup_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MonitoringConfigInfo {
    pub metrics_enabled: bool,
    pub tracing_enabled: bool,
    pub log_level: String,
}

/// System monitoring models
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub uptime: u64,
    pub memory_usage: MemoryMetrics,
    pub cpu_usage: CpuMetrics,
    pub disk_usage: DiskMetrics,
    pub network: NetworkMetrics,
    pub vault: VaultMetrics,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub cached: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CpuMetrics {
    pub cores: u32,
    pub usage_percent: f64,
    pub load_average: [f64; 3],
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskMetrics {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub usage_percent: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultMetrics {
    pub total_secrets: u64,
    pub total_keys: u64,
    pub total_policies: u64,
    pub active_sessions: u64,
    pub operations_per_second: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStatus {
    pub status: String,
    pub version: String,
    pub uptime: u64,
    pub components: ComponentStatus,
    pub last_check: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentStatus {
    pub database: String,
    pub cache: String,
    pub crypto: String,
    pub storage: String,
    pub auth: String,
}

/// Security models
#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityScanResult {
    pub scan_id: String,
    pub status: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub findings: Vec<SecurityFinding>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityFinding {
    pub severity: String,
    pub category: String,
    pub title: String,
    pub description: String,
    pub recommendation: String,
    pub affected_resources: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityIncident {
    pub id: String,
    pub severity: String,
    pub status: String,
    pub title: String,
    pub description: String,
    pub source: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub resolved_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// User management endpoints
/// TODO: Implement using StorageBackend trait instead of direct database access
pub async fn list_users(
    State(_state): State<AppState>,
    Query(_query): Query<ListQuery>,
) -> ApiResult<Json<ApiResponse<Vec<UserResponse>>>> {
    // Placeholder implementation - needs to be refactored to use StorageBackend
    Ok(Json(ApiResponse::success(vec![])))
}

pub async fn create_user(
    State(state): State<AppState>,
    Json(request): Json<CreateUserRequest>,
) -> ApiResult<Json<ApiResponse<UserResponse>>> {
    // Validate input
    if request.username.is_empty() {
        return Err(ApiError::bad_request("Username is required"));
    }
    if request.email.is_empty() {
        return Err(ApiError::bad_request("Email is required"));
    }
    if !request.email.contains('@') {
        return Err(ApiError::bad_request("Invalid email format"));
    }

    // Use AuthService to create user
    let user = state
        .auth
        .create_user(
            &request.username,
            &request.email,
            &request.password,
            request.full_name.as_deref(),
            request.roles,
            request.metadata,
            request.enabled.unwrap_or(true),
        )
        .await?;

    // Collect and sort roles
    let mut roles: Vec<String> = user.roles.into_iter().collect();
    roles.sort();

    // Calculate permissions
    let permissions = calculate_permissions_from_roles(&roles);

    let response = UserResponse {
        id: user.id.to_string(),
        username: user.username,
        email: user.email,
        full_name: user.full_name,
        enabled: user.is_active,
        roles,
        permissions,
        last_login: user.last_login,
        created_at: user.created_at,
        updated_at: user.updated_at,
        metadata: user.metadata,
    };

    Ok(Json(ApiResponse::success(response)))
}

pub async fn get_user(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> ApiResult<Json<ApiResponse<UserResponse>>> {
    let user = state.auth.get_user(&user_id).await?;

    // Collect and sort roles
    let mut roles: Vec<String> = user.roles.into_iter().collect();
    roles.sort();

    // Calculate permissions
    let permissions = calculate_permissions_from_roles(&roles);

    // Build metadata
    let mut metadata = HashMap::new();
    metadata.insert("namespace".to_string(), user.namespace);
    metadata.insert("is_superuser".to_string(), user.is_superuser.to_string());
    metadata.insert("mfa_enabled".to_string(), user.mfa_enabled.to_string());

    let response = UserResponse {
        id: user.id.to_string(),
        username: user.username,
        email: user.email,
        full_name: user.full_name,
        enabled: user.is_active,
        roles,
        permissions,
        last_login: user.last_login,
        created_at: user.created_at,
        updated_at: user.updated_at,
        metadata,
    };

    Ok(Json(ApiResponse::success(response)))
}

pub async fn update_user(
    State(_state): State<AppState>,
    Path(user_id): Path<String>,
    Json(request): Json<UpdateUserRequest>,
) -> ApiResult<Json<ApiResponse<UserResponse>>> {
    // TODO: Implement user update
    let user = UserResponse {
        id: user_id,
        username: "testuser".to_string(),
        email: request.email.unwrap_or("test@example.com".to_string()),
        full_name: request.full_name,
        enabled: request.enabled.unwrap_or(true),
        roles: vec!["user".to_string()],
        permissions: vec!["read".to_string()],
        last_login: Some(chrono::Utc::now()),
        created_at: chrono::Utc::now() - chrono::Duration::days(7),
        updated_at: chrono::Utc::now(),
        metadata: request.metadata.unwrap_or_default(),
    };

    Ok(Json(ApiResponse::success(user)))
}

/// System configuration endpoints
pub async fn get_config(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<SystemConfig>>> {
    let config = &state.config;

    // Get storage backend type dynamically
    let backend_type = match state.storage.get_stats().await {
        Ok(stats) => stats.backend_type,
        Err(_) => "unknown".to_string(),
    };

    let system_config = SystemConfig {
        api: ApiConfigInfo {
            version: env!("CARGO_PKG_VERSION").to_string(),
            bind_address: config.http.bind_address.to_string(),
            max_connections: config.database.max_connections,
            timeout: config.http.timeout.as_secs(),
        },
        security: SecurityConfigInfo {
            mfa_enabled: config.auth.mfa.enabled,
            password_policy: PasswordPolicyInfo {
                min_length: config.auth.password_policy.min_length,
                require_uppercase: config.auth.password_policy.require_uppercase,
                require_lowercase: config.auth.password_policy.require_lowercase,
                require_numbers: config.auth.password_policy.require_numbers,
                require_special: config.auth.password_policy.require_special,
            },
            session_timeout: config.auth.session.timeout.as_secs(),
        },
        storage: StorageConfigInfo {
            backend: backend_type,
            encryption_enabled: true,
            backup_enabled: true,
        },
        monitoring: MonitoringConfigInfo {
            metrics_enabled: config.monitoring.metrics,
            tracing_enabled: config.monitoring.tracing,
            log_level: config.logging.level.clone(),
        },
    };

    Ok(Json(ApiResponse::success(system_config)))
}

/// System monitoring endpoints
pub async fn get_system_metrics(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<SystemMetrics>>> {
    // Get stats from admin service
    let stats = state
        .admin
        .get_system_stats()
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    // Use actual collected metrics
    let metrics = SystemMetrics {
        uptime: stats.uptime_seconds,
        memory_usage: MemoryMetrics {
            total: stats.memory.total,
            used: stats.memory.used,
            free: stats.memory.free,
            cached: stats.memory.cached,
        },
        cpu_usage: CpuMetrics {
            cores: stats.cpu.cores,
            usage_percent: stats.cpu.usage_percent,
            load_average: stats.cpu.load_average,
        },
        disk_usage: DiskMetrics {
            total: stats.disk.total,
            used: stats.disk.used,
            free: stats.disk.free,
            usage_percent: stats.disk.usage_percent,
        },
        network: NetworkMetrics {
            bytes_sent: stats.network.bytes_sent,
            bytes_received: stats.network.bytes_received,
            packets_sent: stats.network.packets_sent,
            packets_received: stats.network.packets_received,
        },
        vault: VaultMetrics {
            total_secrets: stats.total_secrets,
            total_keys: stats.total_keys,
            total_policies: 25, // Placeholder - policy count not yet in stats
            active_sessions: stats.active_sessions,
            operations_per_second: stats.requests_per_minute / 60.0,
        },
    };

    Ok(Json(ApiResponse::success(metrics)))
}

pub async fn get_system_status(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<SystemStatus>>> {
    // Get stats from admin service to get actual uptime
    let stats = state
        .admin
        .get_system_stats()
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    let status = SystemStatus {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime: stats.uptime_seconds,
        components: ComponentStatus {
            database: "healthy".to_string(),
            cache: "healthy".to_string(),
            crypto: "healthy".to_string(),
            storage: "healthy".to_string(),
            auth: "healthy".to_string(),
        },
        last_check: chrono::Utc::now(),
    };

    Ok(Json(ApiResponse::success(status)))
}

/// Security endpoints
pub async fn run_security_scan(
    State(_state): State<AppState>,
) -> ApiResult<Json<ApiResponse<SecurityScanResult>>> {
    // TODO: Implement security scan
    let scan_result = SecurityScanResult {
        scan_id: uuid::Uuid::new_v4().to_string(),
        status: "completed".to_string(),
        started_at: chrono::Utc::now() - chrono::Duration::minutes(5),
        completed_at: Some(chrono::Utc::now()),
        findings: vec![SecurityFinding {
            severity: "low".to_string(),
            category: "configuration".to_string(),
            title: "Default admin password".to_string(),
            description: "The default admin password should be changed".to_string(),
            recommendation: "Change the default admin password to a strong, unique password"
                .to_string(),
            affected_resources: vec!["admin".to_string()],
        }],
    };

    Ok(Json(ApiResponse::success(scan_result)))
}

pub async fn get_security_incidents(
    State(_state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> ApiResult<Json<ApiResponse<Vec<SecurityIncident>>>> {
    // TODO: Implement incident retrieval
    let incidents = vec![SecurityIncident {
        id: uuid::Uuid::new_v4().to_string(),
        severity: "medium".to_string(),
        status: "resolved".to_string(),
        title: "Multiple failed login attempts".to_string(),
        description: "User account experienced 5 failed login attempts from IP 192.168.1.100"
            .to_string(),
        source: "authentication".to_string(),
        created_at: chrono::Utc::now() - chrono::Duration::hours(2),
        updated_at: chrono::Utc::now() - chrono::Duration::minutes(30),
        resolved_at: Some(chrono::Utc::now() - chrono::Duration::minutes(30)),
    }];

    Ok(Json(ApiResponse::success(incidents)))
}

/// Maintenance operations
pub async fn run_garbage_collection(
    State(_state): State<AppState>,
) -> ApiResult<Json<ApiResponse<serde_json::Value>>> {
    // TODO: Implement garbage collection
    let data = serde_json::json!({
        "message": "Garbage collection completed",
        "cleaned_objects": 150,
        "freed_space": "2.5MB"
    });

    Ok(Json(ApiResponse::success(data)))
}

pub async fn compact_database(
    State(_state): State<AppState>,
) -> ApiResult<Json<ApiResponse<serde_json::Value>>> {
    // TODO: Implement database compaction
    let data = serde_json::json!({
        "message": "Database compaction completed",
        "original_size": "1.2GB",
        "compacted_size": "950MB",
        "space_saved": "250MB"
    });

    Ok(Json(ApiResponse::success(data)))
}

/// Calculate permissions from roles
fn calculate_permissions_from_roles(roles: &[String]) -> Vec<String> {
    let mut permissions = Vec::new();

    for role in roles {
        match role.as_str() {
            "admin" | "root" => {
                permissions.push("*".to_string());
                return permissions; // Admin has all permissions
            }
            "operator" => {
                permissions.extend_from_slice(&[
                    "secrets:read".to_string(),
                    "secrets:write".to_string(),
                    "leases:read".to_string(),
                    "leases:renew".to_string(),
                ]);
            }
            "auditor" => {
                permissions
                    .extend_from_slice(&["audit:read".to_string(), "metrics:read".to_string()]);
            }
            "developer" => {
                permissions.extend_from_slice(&[
                    "secrets:read".to_string(),
                    "transit:encrypt".to_string(),
                    "transit:decrypt".to_string(),
                ]);
            }
            _ => {}
        }
    }

    // Remove duplicates
    permissions.sort();
    permissions.dedup();
    permissions
}

// Stub handlers for missing functions
pub async fn delete_user(
    State(_state): State<AppState>,
    Path(_user_id): Path<String>,
) -> ApiResult<Json<ApiResponse<()>>> {
    Err(ApiError::NotImplemented(
        "delete_user not yet implemented".to_string(),
    ))
}

pub async fn get_user_roles(
    State(_state): State<AppState>,
    Path(_user_id): Path<String>,
) -> ApiResult<Json<ApiResponse<Vec<String>>>> {
    Err(ApiError::NotImplemented(
        "get_user_roles not yet implemented".to_string(),
    ))
}

pub async fn assign_user_roles(
    State(_state): State<AppState>,
    Path(_user_id): Path<String>,
) -> ApiResult<Json<ApiResponse<()>>> {
    Err(ApiError::NotImplemented(
        "assign_user_roles not yet implemented".to_string(),
    ))
}

pub async fn get_user_permissions(
    State(_state): State<AppState>,
    Path(_user_id): Path<String>,
) -> ApiResult<Json<ApiResponse<Vec<String>>>> {
    Err(ApiError::NotImplemented(
        "get_user_permissions not yet implemented".to_string(),
    ))
}

pub async fn list_roles(
    State(_state): State<AppState>,
) -> ApiResult<Json<ApiResponse<Vec<String>>>> {
    Err(ApiError::NotImplemented(
        "list_roles not yet implemented".to_string(),
    ))
}

pub async fn create_role(State(_state): State<AppState>) -> ApiResult<Json<ApiResponse<()>>> {
    Err(ApiError::NotImplemented(
        "create_role not yet implemented".to_string(),
    ))
}

pub async fn get_role(
    State(_state): State<AppState>,
    Path(_role_name): Path<String>,
) -> ApiResult<Json<ApiResponse<String>>> {
    Err(ApiError::NotImplemented(
        "get_role not yet implemented".to_string(),
    ))
}

pub async fn update_role(
    State(_state): State<AppState>,
    Path(_role_name): Path<String>,
) -> ApiResult<Json<ApiResponse<()>>> {
    Err(ApiError::NotImplemented(
        "update_role not yet implemented".to_string(),
    ))
}

pub async fn delete_role(
    State(_state): State<AppState>,
    Path(_role_name): Path<String>,
) -> ApiResult<Json<ApiResponse<()>>> {
    Err(ApiError::NotImplemented(
        "delete_role not yet implemented".to_string(),
    ))
}

pub async fn update_config(State(_state): State<AppState>) -> ApiResult<Json<ApiResponse<()>>> {
    Err(ApiError::NotImplemented(
        "update_config not yet implemented".to_string(),
    ))
}

pub async fn reload_config(State(_state): State<AppState>) -> ApiResult<Json<ApiResponse<()>>> {
    Err(ApiError::NotImplemented(
        "reload_config not yet implemented".to_string(),
    ))
}

pub async fn get_system_logs(
    State(_state): State<AppState>,
) -> ApiResult<Json<ApiResponse<Vec<String>>>> {
    Err(ApiError::NotImplemented(
        "get_system_logs not yet implemented".to_string(),
    ))
}

pub async fn vacuum_database(State(_state): State<AppState>) -> ApiResult<Json<ApiResponse<()>>> {
    Err(ApiError::NotImplemented(
        "vacuum_database not yet implemented".to_string(),
    ))
}

pub async fn get_security_reports(
    State(_state): State<AppState>,
) -> ApiResult<Json<ApiResponse<Vec<String>>>> {
    Err(ApiError::NotImplemented(
        "get_security_reports not yet implemented".to_string(),
    ))
}

pub async fn get_security_incident(
    State(_state): State<AppState>,
    Path(_incident_id): Path<String>,
) -> ApiResult<Json<ApiResponse<String>>> {
    Err(ApiError::NotImplemented(
        "get_security_incident not yet implemented".to_string(),
    ))
}
