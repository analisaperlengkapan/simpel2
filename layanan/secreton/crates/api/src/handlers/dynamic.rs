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

use crate::{
    ApiError, ApiResponse, ApiResult, extractors::AuthenticatedUser, handlers::AppState,
    helpers::create_audit_log,
};

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
        .route(
            "/database/config/{name}",
            delete(delete_database_connection),
        )
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
    user: AuthenticatedUser,
    Query(params): Query<GenerateCredsRequest>,
) -> ApiResult<Json<ApiResponse<GenerateCredsResponse>>> {
    // Get database engine from service container
    let db_engine = &state.database_engine;
    let lease_manager = &state.lease_manager;

    // Generate credentials with lease
    let start_time = std::time::Instant::now();
    let result = db_engine
        .generate_credentials_with_lease(&role_name, params.ttl, lease_manager, &user.username)
        .await;

    // Record metrics
    let duration = start_time.elapsed().as_secs_f64();
    match &result {
        Ok(_) => {
            metrics::counter!(
                "secreton_dynamic_credentials_generated_total",
                "role" => role_name.clone(),
                "status" => "success"
            )
            .increment(1);
            metrics::histogram!(
                "secreton_dynamic_credentials_generation_duration_seconds",
                "role" => role_name.clone()
            )
            .record(duration);
        }
        Err(_) => {
            // Use "unknown" role on failure to prevent cardinality explosion from invalid role names
            metrics::counter!(
                "secreton_dynamic_credentials_generated_total",
                "role" => "unknown",
                "status" => "failure"
            )
            .increment(1);
        }
    }

    let (credentials, lease) = result.map_err(|e| ApiError::Internal {
        message: format!("Failed to generate credentials: {}", e),
    })?;

    // Log audit event
    let audit_entry = create_audit_log(
        "creds_generated",
        &user.username,
        "dynamic_role",
        &role_name,
    );
    let _ = state.audit.log(audit_entry).await;

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

/// Validate a role request's fields (shared by create and update handlers)
fn validate_role_request(role_name: &str, request: &CreateRoleRequest) -> ApiResult<()> {
    // Validate role name
    if role_name.is_empty() {
        return Err(ApiError::BadRequest {
            message: "Role name cannot be empty".to_string(),
        });
    }

    // Validate creation statements are not empty
    if request.creation_statements.is_empty() {
        return Err(ApiError::BadRequest {
            message: "Creation statements are required".to_string(),
        });
    }

    // Validate revocation statements are not empty
    if request.revocation_statements.is_empty() {
        return Err(ApiError::BadRequest {
            message: "Revocation statements are required".to_string(),
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

    // Validate rotation statements for SQL injection (if provided)
    for stmt in &request.rotation_statements {
        if contains_dangerous_sql(stmt) {
            return Err(ApiError::BadRequest {
                message: "Rotation statements contain potentially dangerous SQL".to_string(),
            });
        }
    }

    // Validate renew statements for SQL injection (if provided)
    for stmt in &request.renew_statements {
        if contains_dangerous_sql(stmt) {
            return Err(ApiError::BadRequest {
                message: "Renew statements contain potentially dangerous SQL".to_string(),
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

    Ok(())
}

/// Internal helper to create database role
async fn create_role_internal(
    state: &AppState,
    role_name: String,
    request: CreateRoleRequest,
) -> ApiResult<RoleResponse> {
    validate_role_request(&role_name, &request)?;

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
        .map_err(|e| match e {
            secreton_core::services::secrets::database::DatabaseError::RoleAlreadyExists(name) => {
                ApiError::Conflict {
                    resource: format!("Role {}", name),
                }
            }
            secreton_core::services::secrets::database::DatabaseError::InvalidConfig(msg) => {
                ApiError::BadRequest { message: msg }
            }
            other => ApiError::Internal {
                message: format!("Failed to create role: {}", other),
            },
        })?;

    Ok(RoleResponse {
        name: role_name,
        db_name: request.db_name,
        default_ttl: request.default_ttl,
        max_ttl: request.max_ttl,
    })
}

/// Create a database role
pub async fn create_database_role(
    State(state): State<AppState>,
    Path(role_name): Path<String>,
    user: AuthenticatedUser,
    Json(request): Json<CreateRoleRequest>,
) -> ApiResult<Json<ApiResponse<RoleResponse>>> {
    let response = create_role_internal(&state, role_name.clone(), request).await?;

    // Log audit event
    let audit_entry = create_audit_log("role_created", &user.username, "dynamic_role", &role_name);
    let _ = state.audit.log(audit_entry).await;

    Ok(Json(ApiResponse::success(response)))
}

/// List all database roles
pub async fn list_database_roles(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<Vec<String>>>> {
    let roles = state.database_engine.list_roles().await;

    // Log audit event
    let audit_entry = create_audit_log("roles_listed", &user.username, "dynamic_role", "");
    let _ = state.audit.log(audit_entry).await;

    Ok(Json(ApiResponse::success(roles)))
}

/// Get database role details
pub async fn get_database_role(
    State(state): State<AppState>,
    Path(role_name): Path<String>,
    user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<RoleResponse>>> {
    let role = state
        .database_engine
        .get_role(&role_name)
        .await
        .ok_or_else(|| ApiError::NotFound {
            resource: format!("Role {}", role_name),
        })?;

    // Log audit event
    let audit_entry = create_audit_log("role_read", &user.username, "dynamic_role", &role_name);
    let _ = state.audit.log(audit_entry).await;

    let response = RoleResponse {
        name: role.name,
        db_name: role.db_name,
        default_ttl: role.default_ttl,
        max_ttl: role.max_ttl,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Update database role
pub async fn update_database_role(
    State(state): State<AppState>,
    Path(role_name): Path<String>,
    user: AuthenticatedUser,
    Json(request): Json<CreateRoleRequest>,
) -> ApiResult<Json<ApiResponse<RoleResponse>>> {
    validate_role_request(&role_name, &request)?;

    // Capture response values before moving fields into the role struct
    let response_db_name = request.db_name.clone();
    let response_default_ttl = request.default_ttl;
    let response_max_ttl = request.max_ttl;

    // Build role and use atomic update_role (checks existence + writes under one lock)
    let role = DatabaseRole {
        name: role_name.clone(),
        db_name: request.db_name,
        default_ttl: request.default_ttl,
        max_ttl: request.max_ttl,
        creation_statements: request.creation_statements,
        revocation_statements: request.revocation_statements,
        rotation_statements: request.rotation_statements,
        renew_statements: request.renew_statements,
    };

    state
        .database_engine
        .update_role(role)
        .await
        .map_err(|e| match e {
            secreton_core::services::secrets::database::DatabaseError::RoleNotFound(_) => {
                ApiError::NotFound {
                    resource: format!("Role {}", role_name),
                }
            }
            secreton_core::services::secrets::database::DatabaseError::InvalidConfig(msg) => {
                ApiError::BadRequest { message: msg }
            }
            other => ApiError::Internal {
                message: format!("Failed to update role: {}", other),
            },
        })?;

    // Log audit event
    let audit_entry = create_audit_log("role_updated", &user.username, "dynamic_role", &role_name);
    let _ = state.audit.log(audit_entry).await;

    Ok(Json(ApiResponse::success(RoleResponse {
        name: role_name,
        db_name: response_db_name,
        default_ttl: response_default_ttl,
        max_ttl: response_max_ttl,
    })))
}

/// Delete database role
pub async fn delete_database_role(
    State(state): State<AppState>,
    Path(role_name): Path<String>,
    user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<serde_json::Value>>> {
    let deleted = state
        .database_engine
        .delete_role(&role_name)
        .await
        .map_err(|e| ApiError::Internal {
            message: format!("Failed to delete role: {}", e),
        })?;

    if !deleted {
        return Err(ApiError::NotFound {
            resource: format!("Role {}", role_name),
        });
    }

    // Log audit event
    let audit_entry = create_audit_log("role_deleted", &user.username, "dynamic_role", &role_name);
    let _ = state.audit.log(audit_entry).await;

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
    user: AuthenticatedUser,
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
    let audit_entry = create_audit_log(
        "connection_configured",
        &user.username,
        "dynamic_connection",
        &name,
    );
    let _ = state.audit.log(audit_entry).await;

    let response = ConnectionResponse {
        name,
        db_type: db_type.as_str().to_string(),
        verified: request.verify_connection,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Get database connection details
pub async fn get_database_connection(
    State(state): State<AppState>,
    Path(name): Path<String>,
    user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<ConnectionResponse>>> {
    let conn = state
        .database_engine
        .get_connection(&name)
        .await
        .ok_or_else(|| ApiError::NotFound {
            resource: format!("Connection {}", name),
        })?;

    // Log audit event
    let audit_entry = create_audit_log(
        "connection_read",
        &user.username,
        "dynamic_connection",
        &name,
    );
    let _ = state.audit.log(audit_entry).await;

    let response = ConnectionResponse {
        name: conn.name,
        db_type: conn.db_type.as_str().to_string(),
        verified: conn.verify_connection,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Delete database connection
pub async fn delete_database_connection(
    State(state): State<AppState>,
    Path(name): Path<String>,
    user: AuthenticatedUser,
) -> ApiResult<Json<ApiResponse<serde_json::Value>>> {
    let result = state
        .database_engine
        .delete_connection(&name)
        .await
        .map_err(|e| ApiError::Internal {
            message: format!("Failed to delete connection: {}", e),
        })?
        .ok_or_else(|| ApiError::NotFound {
            resource: format!("Connection {}", name),
        })?;

    // Log cascade audit entries for the roles the engine actually deleted
    for role_name in &result.deleted_roles {
        let audit_entry = create_audit_log(
            "role_deleted_cascade",
            &user.username,
            "dynamic_role",
            role_name,
        );
        let _ = state.audit.log(audit_entry).await;
    }

    // Log audit event
    let audit_entry = create_audit_log(
        "connection_deleted",
        &user.username,
        "dynamic_connection",
        &name,
    );
    let _ = state.audit.log(audit_entry).await;

    let mut response = serde_json::json!({
        "deleted": true,
        "connection": name,
    });

    // Surface revocation warnings so callers can detect partial failures
    // and operators can manually clean up orphaned database users.
    if !result.revocation_warnings.is_empty() {
        response["warnings"] = serde_json::json!(result.revocation_warnings);
    }

    Ok(Json(ApiResponse::success(response)))
}

/// Basic SQL injection prevention
fn contains_dangerous_sql(sql: &str) -> bool {
    let sql_lower = sql.to_lowercase();

    // Reject multiple statements: a semicolon followed by any non-whitespace
    // content indicates statement chaining which is never legitimate in a
    // single creation/revocation/rotation/renew statement template.
    if let Some(semi_pos) = sql_lower.find(';') {
        let after_semi = sql_lower[semi_pos + 1..].trim();
        if !after_semi.is_empty() {
            return true;
        }
    }

    // Patterns that are dangerous as substrings anywhere
    let substring_patterns = [
        ";--",
        "/*",
        "*/",
        "xp_",
        "sp_",
        "drop database",
        "drop table",
        "truncate",
        "delete from",
    ];

    if substring_patterns
        .iter()
        .any(|pattern| sql_lower.contains(pattern))
    {
        return true;
    }

    // Patterns that must appear as standalone words (not inside other words
    // like "execute" inside "GRANT EXECUTE ON …")
    let word_boundary_patterns = ["exec", "execute"];
    for pattern in &word_boundary_patterns {
        for (idx, _) in sql_lower.match_indices(pattern) {
            let before_ok = idx == 0
                || (!sql_lower.as_bytes()[idx - 1].is_ascii_alphanumeric()
                    && sql_lower.as_bytes()[idx - 1] != b'_');
            let end = idx + pattern.len();
            let after_ok = end >= sql_lower.len()
                || (!sql_lower.as_bytes()[end].is_ascii_alphanumeric()
                    && sql_lower.as_bytes()[end] != b'_');

            // Only flag if the word stands alone AND is followed by something
            // that looks like a procedure call or dynamic SQL (not a privilege grant)
            if before_ok && after_ok {
                // Allow "GRANT EXECUTE" / "REVOKE EXECUTE" which are legitimate DDL
                let raw = idx.saturating_sub(20);
                // Avoid slicing in the middle of a multi-byte UTF-8 character
                let mut prefix_start = raw;
                while prefix_start < idx && !sql_lower.is_char_boundary(prefix_start) {
                    prefix_start += 1;
                }
                let prefix_text = &sql_lower[prefix_start..idx].trim_end();
                let is_grant_or_revoke = ["grant", "revoke"].iter().any(|keyword| {
                    if let Some(stripped) = prefix_text.strip_suffix(keyword) {
                        // keyword must also be at a word boundary
                        stripped.is_empty()
                            || stripped
                                .as_bytes()
                                .last()
                                .map(|&b| !b.is_ascii_alphanumeric() && b != b'_')
                                .unwrap_or(true)
                    } else {
                        false
                    }
                });
                if is_grant_or_revoke {
                    continue;
                }
                return true;
            }
        }
    }

    false
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
        assert!(contains_dangerous_sql("EXEC sp_executesql"));
        assert!(!contains_dangerous_sql(
            "CREATE USER {{username}} WITH PASSWORD '{{password}}'"
        ));
        assert!(!contains_dangerous_sql(
            "GRANT SELECT ON database.* TO {{username}}"
        ));
        // GRANT EXECUTE is legitimate DDL, not injection
        assert!(!contains_dangerous_sql(
            "GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA public TO {{username}}"
        ));
        assert!(!contains_dangerous_sql(
            "REVOKE EXECUTE ON ALL FUNCTIONS IN SCHEMA public FROM {{username}}"
        ));
        // Partial-word "grant"/"revoke" must NOT bypass detection
        assert!(contains_dangerous_sql("FOREGRANT EXECUTE dangerous_call"));
        assert!(contains_dangerous_sql("SOMEREVOKE EXECUTE dangerous_call"));
        // Non-ASCII input must not panic (multi-byte UTF-8 chars near exec)
        assert!(contains_dangerous_sql("ñañañañañañañañaña exec something"));
        assert!(!contains_dangerous_sql(
            "GRANT EXECUTE ON schéma.función TO {{username}}"
        ));
        // Statement chaining via semicolons must be detected
        assert!(contains_dangerous_sql(
            "CREATE USER {{username}} WITH PASSWORD '{{password}}'; SELECT * FROM pg_shadow"
        ));
        assert!(contains_dangerous_sql(
            "CREATE USER {{username}}; DROP TABLE users"
        ));
        // Trailing semicolons (no content after) are acceptable
        assert!(!contains_dangerous_sql(
            "CREATE USER {{username}} WITH PASSWORD '{{password}}';"
        ));
        // Trailing semicolons with only whitespace are acceptable
        assert!(!contains_dangerous_sql(
            "CREATE USER {{username}} WITH PASSWORD '{{password}}';   "
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

    #[tokio::test]
    async fn test_list_database_roles() {
        use crate::services::ServiceContainer;
        use deadpool_postgres::{Config, Runtime};
        use secreton_storage::MemoryBackend;
        use std::sync::Arc;
        use tokio_postgres::NoTls;

        // Setup dependencies
        let storage = Arc::new(MemoryBackend::new());

        let mut cfg = Config::new();
        cfg.url = Some("postgresql://dummy:dummy@localhost:5432/dummy".to_string());
        let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();

        let container = ServiceContainer::new_mock(storage, pool);
        let state = Arc::new(container);

        // Configure a connection (must be done before creating a role)
        // verify_connection = false to avoid actual connection attempt
        let connection = DatabaseConnection {
            name: "test-db".to_string(),
            db_type: DatabaseType::PostgreSQL,
            connection_url: "postgresql://localhost/test".to_string(),
            verify_connection: false,
            ..Default::default()
        };

        state
            .database_engine
            .configure_connection(connection)
            .await
            .unwrap();

        // Create a role directly in the engine
        let role = DatabaseRole {
            name: "test-role".to_string(),
            db_name: "test-db".to_string(),
            default_ttl: 3600,
            max_ttl: 86400,
            creation_statements: vec!["CREATE USER {{username}}".to_string()],
            revocation_statements: vec!["DROP USER {{username}}".to_string()],
            rotation_statements: vec![],
            renew_statements: vec![],
        };
        state.database_engine.create_role(role).await.unwrap();

        // Create mock user
        let user = AuthenticatedUser {
            id: uuid::Uuid::new_v4(),
            username: "test_admin".to_string(),
            email: None,
            roles: vec![],
        };

        // Call the handler
        let result = list_database_roles(State(state.clone()), user).await;

        assert!(result.is_ok());
        let Json(response) = result.unwrap();
        assert!(response.success);
        let data = response.data.unwrap();

        // Assert that "test-role" is present
        assert!(
            data.contains(&"test-role".to_string()),
            "Role list should contain test-role, found: {:?}",
            data
        );
    }
}
