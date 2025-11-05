//! Dynamic Secrets API Handlers
//!
//! Provides REST endpoints for dynamic database credential generation,
//! role management, and lease integration.

use axum::{
    Router,
    extract::{Path, Query, State},
    response::Json,
    routing::{delete, get, post, put},
};

use serde::{Deserialize, Serialize};

use crate::{ApiError, ApiResponse, ApiResult, handlers::AppState};

use secreton_core::services::secrets::database::{DatabaseConnection, DatabaseRole, DatabaseType};

/// Create dynamic secrets routes
pub fn create_routes() -> Router<AppState> {
    Router::new()
        // Credential generation
        .route("/database/creds/{role}", get(generate_database_credentials))
        // Role management
        .route("/database/roles", get(list_database_roles))
        .route("/database/roles/{role}", post(create_database_role))
        .route("/database/roles/{role}", get(get_database_role))
        .route("/database/roles/{role}", put(update_database_role))
        .route("/database/roles/{role}", delete(delete_database_role))
        // Connection management
        .route(
            "/database/config/{name}",
            post(configure_database_connection),
        )
        .route("/database/config/{name}", get(get_database_connection))
        .route("/database/config/{name}", delete(delete_database_connection))
}

/// Request to generate database credentials
#[derive(Debug, Deserialize)]
pub struct GenerateCredsRequest {
    /// Optional TTL in seconds
    pub ttl: Option<u32>,
}

/// Response with generated credentials and lease
#[derive(Debug, Serialize)]
pub struct GenerateCredsResponse {
    /// Lease ID
    pub lease_id: String,

    /// Lease duration in seconds
    pub lease_duration: i64,

    /// Whether lease is renewable
    pub renewable: bool,

    /// Credential data
    pub data: CredentialData,
}

#[derive(Debug, Serialize)]
pub struct CredentialData {
    /// Database username
    pub username: String,

    /// Database password
    pub password: String,

    /// Connection URL (optional)
    pub connection_url: Option<String>,

    /// Database name
    pub database: String,

    /// Role name
    pub role: String,
}

/// Generate database credentials for a role
pub async fn generate_database_credentials(
    State(state): State<AppState>,
    Path(role_name): Path<String>,
    Query(params): Query<GenerateCredsRequest>,
) -> ApiResult<Json<ApiResponse<GenerateCredsResponse>>> {
    // Extract user from auth context (TODO: implement proper auth extraction)
    let user = "system"; // Placeholder

    // Get database engine from service container
    let db_engine = &state.database_engine;
    let lease_manager = &state.lease_manager;

    // Generate credentials with lease
    let (credentials, lease) = db_engine
        .generate_credentials_with_lease(&role_name, params.ttl, lease_manager, user)
        .await
        .map_err(|e| ApiError::Internal {
            message: format!("Failed to generate credentials: {}", e),
        })?;

    // Log audit event
    // TODO: Fix audit logging

    // Record metrics
    // TODO: Implement metrics recording

    let response = GenerateCredsResponse {
        lease_id: lease.id.clone(),
        lease_duration: (lease.expired_at - lease.issued_at).num_seconds(),
        renewable: lease.renewable,
        data: CredentialData {
            username: credentials.username,
            password: credentials.password,
            connection_url: credentials.connection_url,
            database: credentials.db_name,
            role: credentials.role_name,
        },
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Request to create a database role
#[derive(Debug, Deserialize)]
pub struct CreateRoleRequest {
    /// Database connection name
    pub db_name: String,

    /// Default TTL in seconds
    #[serde(default = "default_ttl")]
    pub default_ttl: u32,

    /// Maximum TTL in seconds
    #[serde(default = "default_max_ttl")]
    pub max_ttl: u32,

    /// SQL statements to create user
    pub creation_statements: Vec<String>,

    /// SQL statements to revoke/delete user
    pub revocation_statements: Vec<String>,

    /// SQL statements to rotate password (optional)
    #[serde(default)]
    pub rotation_statements: Vec<String>,

    /// SQL statements executed on lease renewal (optional)
    #[serde(default)]
    pub renew_statements: Vec<String>,
}

fn default_ttl() -> u32 {
    3600
}
fn default_max_ttl() -> u32 {
    86400
}

/// Response for role operations
#[derive(Debug, Serialize)]
pub struct RoleResponse {
    pub name: String,
    pub db_name: String,
    pub default_ttl: u32,
    pub max_ttl: u32,
}

/// Create a database role
pub async fn create_database_role(
    State(state): State<AppState>,
    Path(role_name): Path<String>,
    Json(request): Json<CreateRoleRequest>,
) -> ApiResult<Json<ApiResponse<RoleResponse>>> {
    // Validate role name
    if role_name.is_empty() {
        return Err(ApiError::BadRequest {
            message: "Role name cannot be empty".to_string(),
        });
    }

    // Validate statements contain placeholders
    for stmt in &request.creation_statements {
        if !stmt.contains("{{username}}") && !stmt.contains("{{password}}") {
            return Err(ApiError::BadRequest {
                message:
                    "Creation statements must contain {{username}} or {{password}} placeholders"
                        .to_string(),
            });
        }

        // Basic SQL injection prevention
        if contains_dangerous_sql(stmt) {
            return Err(ApiError::BadRequest {
                message: "Creation statements contain potentially dangerous SQL".to_string(),
            });
        }
    }

    for stmt in &request.revocation_statements {
        if !stmt.contains("{{username}}") {
            return Err(ApiError::BadRequest {
                message: "Revocation statements must contain {{username}} placeholder".to_string(),
            });
        }

        if contains_dangerous_sql(stmt) {
            return Err(ApiError::BadRequest {
                message: "Revocation statements contain potentially dangerous SQL".to_string(),
            });
        }
    }

    // Validate TTL values
    if request.default_ttl == 0 || request.default_ttl > request.max_ttl {
        return Err(ApiError::BadRequest {
            message: format!(
                "Invalid TTL: default_ttl ({}) must be between 1 and max_ttl ({})",
                request.default_ttl, request.max_ttl
            ),
        });
    }

    // Create role
    let role = DatabaseRole {
        name: role_name.clone(),
        db_name: request.db_name.clone(),
        default_ttl: request.default_ttl,
        max_ttl: request.max_ttl,
        creation_statements: request.creation_statements,
        revocation_statements: request.revocation_statements,
        rotation_statements: request.rotation_statements,
        renew_statements: request.renew_statements,
    };

    state
        .database_engine
        .create_role(role)
        .await
        .map_err(|e| ApiError::Internal {
            message: format!("Failed to create role: {}", e),
        })?;

    // Log audit event
    // TODO: Fix audit logging

    let response = RoleResponse {
        name: role_name,
        db_name: request.db_name,
        default_ttl: request.default_ttl,
        max_ttl: request.max_ttl,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// List all database roles
pub async fn list_database_roles(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<Vec<String>>>> {
    // TODO: Implement role listing in database engine
    // For now, return empty list
    Ok(Json(ApiResponse::success(vec![])))
}

/// Get database role details
pub async fn get_database_role(
    State(state): State<AppState>,
    Path(role_name): Path<String>,
) -> ApiResult<Json<ApiResponse<RoleResponse>>> {
    // TODO: Implement role retrieval in database engine
    Err(ApiError::NotFound {
        resource: format!("Role {} not found", role_name),
    })
}

/// Update database role
pub async fn update_database_role(
    State(state): State<AppState>,
    Path(role_name): Path<String>,
    Json(request): Json<CreateRoleRequest>,
) -> ApiResult<Json<ApiResponse<RoleResponse>>> {
    // Validate and create role (same as create)
    create_database_role(State(state), Path(role_name), Json(request)).await
}

/// Delete database role
pub async fn delete_database_role(
    State(state): State<AppState>,
    Path(role_name): Path<String>,
) -> ApiResult<Json<ApiResponse<serde_json::Value>>> {
    // TODO: Implement role deletion in database engine
    // Should also revoke all active credentials for this role

    // Log audit event
    // TODO: Fix audit logging

    Ok(Json(ApiResponse::success(serde_json::json!({
        "deleted": true,
        "role": role_name,
    }))))
}

/// Request to configure database connection
#[derive(Debug, Deserialize)]
pub struct ConfigureConnectionRequest {
    /// Database type
    pub db_type: String,

    /// Connection URL
    pub connection_url: String,

    /// Maximum open connections
    #[serde(default = "default_max_connections")]
    pub max_open_connections: u32,

    /// Maximum idle connections
    #[serde(default = "default_max_idle")]
    pub max_idle_connections: u32,

    /// Connection max lifetime in seconds
    #[serde(default = "default_max_lifetime")]
    pub max_connection_lifetime: u32,

    /// Verify connection on startup
    #[serde(default = "default_verify")]
    pub verify_connection: bool,

    /// Root rotation statements (optional)
    #[serde(default)]
    pub root_rotation_statements: Vec<String>,
}

fn default_max_connections() -> u32 {
    4
}
fn default_max_idle() -> u32 {
    2
}
fn default_max_lifetime() -> u32 {
    3600
}
fn default_verify() -> bool {
    true
}

/// Response for connection operations
#[derive(Debug, Serialize)]
pub struct ConnectionResponse {
    pub name: String,
    pub db_type: String,
    pub verified: bool,
}

/// Configure database connection
pub async fn configure_database_connection(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(request): Json<ConfigureConnectionRequest>,
) -> ApiResult<Json<ApiResponse<ConnectionResponse>>> {
    // Validate connection name
    if name.is_empty() {
        return Err(ApiError::BadRequest {
            message: "Connection name cannot be empty".to_string(),
        });
    }

    // Validate connection URL
    if request.connection_url.is_empty() {
        return Err(ApiError::BadRequest {
            message: "Connection URL cannot be empty".to_string(),
        });
    }

    // Parse database type
    let db_type = match request.db_type.to_lowercase().as_str() {
        "postgresql" | "postgres" => DatabaseType::PostgreSQL,
        "mysql" => DatabaseType::MySQL,
        "mongodb" | "mongo" => DatabaseType::MongoDB,
        "redis" => DatabaseType::Redis,
        "cassandra" => DatabaseType::Cassandra,
        "mssql" | "sqlserver" => DatabaseType::MSSQL,
        _ => {
            return Err(ApiError::BadRequest {
                message: format!("Unsupported database type: {}", request.db_type),
            });
        }
    };

    // Create connection config
    let config = DatabaseConnection {
        name: name.clone(),
        db_type: db_type.clone(),
        connection_url: request.connection_url,
        max_open_connections: request.max_open_connections,
        max_idle_connections: request.max_idle_connections,
        max_connection_lifetime: request.max_connection_lifetime,
        verify_connection: request.verify_connection,
        root_rotation_statements: request.root_rotation_statements,
        username: None,
        password: None,
    };

    // Configure connection
    state
        .database_engine
        .configure_connection(config)
        .await
        .map_err(|e| ApiError::Internal {
            message: format!("Failed to configure connection: {}", e),
        })?;

    // Log audit event
    // TODO: Fix audit logging

    let response = ConnectionResponse {
        name,
        db_type: request.db_type,
        verified: request.verify_connection,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Get database connection details
pub async fn get_database_connection(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> ApiResult<Json<ApiResponse<ConnectionResponse>>> {
    // TODO: Implement connection retrieval in database engine
    Err(ApiError::NotFound {
        resource: format!("Connection {} not found", name),
    })
}

/// Delete database connection
pub async fn delete_database_connection(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> ApiResult<Json<ApiResponse<serde_json::Value>>> {
    // TODO: Implement connection deletion in database engine
    // Should also delete all roles using this connection

    // Log audit event
    // TODO: Fix audit logging

    Ok(Json(ApiResponse::success(serde_json::json!({
        "deleted": true,
        "connection": name,
    }))))
}

/// Basic SQL injection prevention
fn contains_dangerous_sql(sql: &str) -> bool {
    let dangerous_patterns = [
        ";--",
        "/*",
        "*/",
        "xp_",
        "sp_",
        "exec",
        "execute",
        "drop database",
        "drop table",
        "truncate",
        "delete from",
    ];

    let sql_lower = sql.to_lowercase();
    dangerous_patterns
        .iter()
        .any(|pattern| sql_lower.contains(pattern))
}

#[cfg(all(test, feature = "enable-inline-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_sql_injection_detection() {
        assert!(contains_dangerous_sql("DROP TABLE users;--"));
        assert!(contains_dangerous_sql(
            "SELECT * FROM users; DROP TABLE users;"
        ));
        assert!(contains_dangerous_sql("/* comment */ DROP DATABASE"));
        assert!(!contains_dangerous_sql(
            "CREATE USER {{username}} WITH PASSWORD '{{password}}'"
        ));
        assert!(!contains_dangerous_sql(
            "GRANT SELECT ON database.* TO {{username}}"
        ));
    }

    #[test]
    fn test_default_values() {
        assert_eq!(default_ttl(), 3600);
        assert_eq!(default_max_ttl(), 86400);
        assert_eq!(default_max_connections(), 4);
        assert_eq!(default_max_idle(), 2);
        assert_eq!(default_max_lifetime(), 3600);
        assert_eq!(default_verify(), true);
    }
}
