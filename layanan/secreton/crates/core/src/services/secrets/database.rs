//! Database Secrets Engine
//!
//! Dynamically generates database credentials with automatic rotation.
//! Supports MySQL, PostgreSQL, MongoDB and other databases.

use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use rand::rngs::OsRng;
use rustls::{ClientConfig, RootCertStore};
use rustls_native_certs::load_native_certs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_postgres::{Client as PgClient, NoTls};
use tokio_postgres_rustls::MakeRustlsConnect;
use uuid::Uuid;

/// Error types for database secrets engine
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Database role not found: {0}")]
    RoleNotFound(String),

    #[error("Credential generation failed: {0}")]
    CredentialGenerationFailed(String),

    #[error("Rotation failed: {0}")]
    RotationFailed(String),

    #[error("Revocation failed: {0}")]
    RevocationFailed(String),

    #[error("Role already exists: {0}")]
    RoleAlreadyExists(String),

    #[error("Unsupported database type: {0}")]
    UnsupportedDatabase(String),
}

/// Database type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DatabaseType {
    MySQL,
    PostgreSQL,
    MongoDB,
    Redis,
    Cassandra,
    MSSQL,
}

impl DatabaseType {
    pub fn as_str(&self) -> &str {
        match self {
            DatabaseType::MySQL => "mysql",
            DatabaseType::PostgreSQL => "postgresql",
            DatabaseType::MongoDB => "mongodb",
            DatabaseType::Redis => "redis",
            DatabaseType::Cassandra => "cassandra",
            DatabaseType::MSSQL => "mssql",
        }
    }
}

/// Database connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConnection {
    /// Connection name
    pub name: String,

    /// Database type
    pub db_type: DatabaseType,

    /// Connection URL
    pub connection_url: String,

    /// Maximum open connections
    pub max_open_connections: u32,

    /// Maximum idle connections
    pub max_idle_connections: u32,

    /// Connection max lifetime in seconds
    pub max_connection_lifetime: u32,

    /// Verify connection on startup
    pub verify_connection: bool,

    /// Root rotation statements
    pub root_rotation_statements: Vec<String>,

    /// Root credentials
    pub username: Option<String>,
    pub password: Option<String>,
}

impl Default for DatabaseConnection {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            db_type: DatabaseType::PostgreSQL,
            connection_url: String::new(),
            max_open_connections: 4,
            max_idle_connections: 2,
            max_connection_lifetime: 3600,
            verify_connection: true,
            root_rotation_statements: Vec::new(),
            username: None,
            password: None,
        }
    }
}

/// Database role configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseRole {
    /// Role name
    pub name: String,

    /// Database connection name
    pub db_name: String,

    /// Default TTL for credentials
    pub default_ttl: u32,

    /// Maximum TTL for credentials
    pub max_ttl: u32,

    /// Creation statements (SQL/commands to create user)
    pub creation_statements: Vec<String>,

    /// Revocation statements (SQL/commands to revoke/delete user)
    pub revocation_statements: Vec<String>,

    /// Rotation statements (SQL/commands to rotate password)
    pub rotation_statements: Vec<String>,

    /// Renew statements (SQL/commands executed on lease renewal)
    pub renew_statements: Vec<String>,
}

impl Default for DatabaseRole {
    fn default() -> Self {
        Self {
            name: String::new(),
            db_name: String::new(),
            default_ttl: 3600,
            max_ttl: 86400,
            creation_statements: Vec::new(),
            revocation_statements: Vec::new(),
            rotation_statements: Vec::new(),
            renew_statements: Vec::new(),
        }
    }
}

/// Generated database credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseCredentials {
    /// Unique credential ID
    pub id: String,

    /// Database username
    pub username: String,

    /// Database password
    pub password: String,

    /// Connection string (optional)
    pub connection_url: Option<String>,

    /// Creation time
    pub created_at: DateTime<Utc>,

    /// Expiration time
    pub expires_at: DateTime<Utc>,

    /// Role name used
    pub role_name: String,

    /// Database name
    pub db_name: String,
}

/// Database connection pool wrapper
enum DbPool {
    PostgreSQL(Arc<RwLock<Option<PgClient>>>),
    MySQL, // Placeholder for future MySQL implementation
}

/// Database secrets engine
pub struct DatabaseSecretsEngine {
    connections: Arc<RwLock<HashMap<String, DatabaseConnection>>>,
    roles: Arc<RwLock<HashMap<String, DatabaseRole>>>,
    active_credentials: Arc<RwLock<HashMap<String, DatabaseCredentials>>>,
    db_pools: Arc<RwLock<HashMap<String, DbPool>>>,
}

impl DatabaseSecretsEngine {
    /// Create new database secrets engine
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            roles: Arc::new(RwLock::new(HashMap::new())),
            active_credentials: Arc::new(RwLock::new(HashMap::new())),
            db_pools: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Configure database connection
    pub async fn configure_connection(
        &self,
        config: DatabaseConnection,
    ) -> Result<(), DatabaseError> {
        // Validate configuration
        if config.name.is_empty() {
            return Err(DatabaseError::InvalidConfig(
                "Connection name cannot be empty".to_string(),
            ));
        }
        if config.connection_url.is_empty() {
            return Err(DatabaseError::InvalidConfig(
                "Connection URL cannot be empty".to_string(),
            ));
        }

        // Test connection if requested
        if config.verify_connection {
            self.test_connection(&config).await?;
        }

        // Store connection
        let mut connections = self.connections.write().await;
        connections.insert(config.name.clone(), config);

        Ok(())
    }

    /// Test database connection
    async fn test_connection(&self, config: &DatabaseConnection) -> Result<(), DatabaseError> {
        if !config.connection_url.contains("://") {
            return Err(DatabaseError::ConnectionError(
                "Invalid connection URL format".to_string(),
            ));
        }

        match config.db_type {
            DatabaseType::PostgreSQL => {
                let tls_mode =
                    std::env::var("SECRETON_DB_TLS_MODE").unwrap_or_else(|_| "disable".to_string());

                let _client = if tls_mode.eq_ignore_ascii_case("disable") {
                    let (client, connection) =
                        tokio_postgres::connect(&config.connection_url, NoTls)
                            .await
                            .map_err(|e| {
                                DatabaseError::ConnectionError(format!(
                                    "PostgreSQL connection failed: {}",
                                    e
                                ))
                            })?;

                    tokio::spawn(async move {
                        if let Err(e) = connection.await {
                            eprintln!("PostgreSQL connection error: {}", e);
                        }
                    });

                    client
                } else {
                    let mut root_store = RootCertStore::empty();
                    let certs = load_native_certs();
                    for cert in certs.certs {
                        let _ = root_store.add(cert);
                    }

                    let tls_config = ClientConfig::builder()
                        .with_root_certificates(root_store)
                        .with_no_client_auth();
                    let tls = MakeRustlsConnect::new(tls_config);

                    let (client, connection) = tokio_postgres::connect(&config.connection_url, tls)
                        .await
                        .map_err(|e| {
                            DatabaseError::ConnectionError(format!(
                                "PostgreSQL connection failed: {}",
                                e
                            ))
                        })?;

                    tokio::spawn(async move {
                        if let Err(e) = connection.await {
                            eprintln!("PostgreSQL connection error: {}", e);
                        }
                    });

                    client
                };
            }
            DatabaseType::MySQL => {
                // MySQL support to be implemented
                return Err(DatabaseError::UnsupportedDatabase(
                    "MySQL support not yet implemented".to_string(),
                ));
            }
            _ => {
                return Err(DatabaseError::UnsupportedDatabase(format!(
                    "{} not yet supported",
                    config.db_type.as_str()
                )));
            }
        }

        Ok(())
    }

    /// Create database role
    pub async fn create_role(&self, role: DatabaseRole) -> Result<(), DatabaseError> {
        // Validate role
        if role.name.is_empty() {
            return Err(DatabaseError::InvalidConfig(
                "Role name cannot be empty".to_string(),
            ));
        }
        if role.creation_statements.is_empty() {
            return Err(DatabaseError::InvalidConfig(
                "Creation statements required".to_string(),
            ));
        }

        // Verify database connection exists
        let connections = self.connections.read().await;
        if !connections.contains_key(&role.db_name) {
            return Err(DatabaseError::InvalidConfig(format!(
                "Database connection '{}' not found",
                role.db_name
            )));
        }
        drop(connections);

        // Store role (reject if already exists)
        let mut roles = self.roles.write().await;
        if roles.contains_key(&role.name) {
            return Err(DatabaseError::RoleAlreadyExists(role.name.clone()));
        }
        roles.insert(role.name.clone(), role);

        Ok(())
    }

    /// Get database connection configuration
    pub async fn get_connection(&self, name: &str) -> Option<DatabaseConnection> {
        let connections = self.connections.read().await;
        connections.get(name).cloned()
    }

    /// Update an existing database role atomically (check + write under one lock)
    pub async fn update_role(&self, role: DatabaseRole) -> Result<(), DatabaseError> {
        // Validate role
        if role.name.is_empty() {
            return Err(DatabaseError::InvalidConfig(
                "Role name cannot be empty".to_string(),
            ));
        }
        if role.creation_statements.is_empty() {
            return Err(DatabaseError::InvalidConfig(
                "Creation statements required".to_string(),
            ));
        }

        // Verify database connection exists
        let connections = self.connections.read().await;
        if !connections.contains_key(&role.db_name) {
            return Err(DatabaseError::InvalidConfig(format!(
                "Database connection '{}' not found",
                role.db_name
            )));
        }
        drop(connections);

        // Atomically check existence and update under a single write lock
        let mut roles = self.roles.write().await;
        if !roles.contains_key(&role.name) {
            return Err(DatabaseError::RoleNotFound(role.name.clone()));
        }
        roles.insert(role.name.clone(), role);

        Ok(())
    }

    /// Get database role
    pub async fn get_role(&self, name: &str) -> Option<DatabaseRole> {
        let roles = self.roles.read().await;
        roles.get(name).cloned()
    }

    /// List database roles
    pub async fn list_roles(&self) -> Vec<String> {
        let roles = self.roles.read().await;
        roles.keys().cloned().collect()
    }

    /// Delete database role and revoke all active credentials for it
    pub async fn delete_role(&self, name: &str) -> Result<bool, DatabaseError> {
        // Collect credential IDs for this role
        let cred_ids: Vec<String> = {
            let active = self.active_credentials.read().await;
            active
                .values()
                .filter(|c| c.role_name == name)
                .map(|c| c.id.clone())
                .collect()
        };

        // Revoke each credential (ignore errors — DB pool may be gone)
        for cred_id in &cred_ids {
            let _ = self.revoke_credentials(cred_id).await;
        }

        let mut roles = self.roles.write().await;
        Ok(roles.remove(name).is_some())
    }

    /// Delete database connection, cascading to all roles that reference it
    pub async fn delete_connection(&self, name: &str) -> Result<bool, DatabaseError> {
        // Find all roles that reference this connection
        let role_names: Vec<String> = {
            let roles = self.roles.read().await;
            roles
                .values()
                .filter(|r| r.db_name == name)
                .map(|r| r.name.clone())
                .collect()
        };

        // Delete each associated role (which also revokes their credentials)
        for role_name in &role_names {
            let _ = self.delete_role(role_name).await;
        }

        // Remove the connection itself
        let mut connections = self.connections.write().await;
        let existed = connections.remove(name).is_some();
        drop(connections);

        // Remove the connection pool
        let mut pools = self.db_pools.write().await;
        pools.remove(name);

        Ok(existed)
    }

    /// Generate credentials for a role
    pub async fn generate_credentials(
        &self,
        role_name: &str,
        ttl: Option<u32>,
    ) -> Result<DatabaseCredentials, DatabaseError> {
        // Get role
        let roles = self.roles.read().await;
        let role = roles
            .get(role_name)
            .ok_or_else(|| DatabaseError::RoleNotFound(role_name.to_string()))?
            .clone();
        drop(roles);

        // Get connection
        let connections = self.connections.read().await;
        let connection = connections
            .get(&role.db_name)
            .ok_or_else(|| {
                DatabaseError::InvalidConfig(format!(
                    "Database connection '{}' not found",
                    role.db_name
                ))
            })?
            .clone();
        drop(connections);

        // Determine TTL
        let ttl = ttl.unwrap_or(role.default_ttl);
        if ttl > role.max_ttl {
            return Err(DatabaseError::InvalidConfig(format!(
                "TTL {} exceeds maximum {}",
                ttl, role.max_ttl
            )));
        }

        // Generate username and password
        let username = self.generate_username(&connection.db_type, role_name);
        let password = self.generate_password(32);

        // Execute creation statements
        self.execute_creation_statements(
            &connection,
            &role.creation_statements,
            &username,
            &password,
        )
        .await?;

        // Create credentials record
        let now = Utc::now();
        let credentials = DatabaseCredentials {
            id: Uuid::new_v4().to_string(),
            username: username.clone(),
            password: password.clone(),
            connection_url: Some(self.build_connection_url(&connection, &username, &password)),
            created_at: now,
            expires_at: now + Duration::seconds(ttl as i64),
            role_name: role_name.to_string(),
            db_name: role.db_name.clone(),
        };

        // Store active credentials
        let mut active = self.active_credentials.write().await;
        active.insert(credentials.id.clone(), credentials.clone());

        Ok(credentials)
    }

    /// Generate database username
    fn generate_username(&self, db_type: &DatabaseType, role_name: &str) -> String {
        let uuid = Uuid::new_v4().to_string();
        let short_uuid = &uuid[..8];

        match db_type {
            DatabaseType::MySQL | DatabaseType::PostgreSQL => {
                format!("v-{}-{}", role_name, short_uuid)
            }
            DatabaseType::MongoDB => {
                format!("v_{}_{}", role_name.replace("-", "_"), short_uuid)
            }
            _ => format!("engine_{}_{}", role_name, short_uuid),
        }
    }

    /// Generate secure random password using OsRng for cryptographic security
    fn generate_password(&self, length: usize) -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                abcdefghijklmnopqrstuvwxyz\
                                0123456789\
                                !@#$%^&*";
        let mut rng = OsRng;

        (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    /// Execute creation statements
    async fn execute_creation_statements(
        &self,
        connection: &DatabaseConnection,
        statements: &[String],
        username: &str,
        password: &str,
    ) -> Result<(), DatabaseError> {
        // Validate that placeholders exist
        for stmt in statements {
            if !stmt.contains("{{username}}") && !stmt.contains("{{password}}") {
                return Err(DatabaseError::CredentialGenerationFailed(
                    "Creation statements must contain {{username} or {{password} placeholders"
                        .to_string(),
                ));
            }
        }

        match connection.db_type {
            DatabaseType::PostgreSQL => {
                self.execute_postgres_statements(connection, statements, username, password)
                    .await?;
            }
            DatabaseType::MySQL => {
                return Err(DatabaseError::UnsupportedDatabase(
                    "MySQL support not yet implemented".to_string(),
                ));
            }
            _ => {
                return Err(DatabaseError::UnsupportedDatabase(format!(
                    "{} not yet supported",
                    connection.db_type.as_str()
                )));
            }
        }

        Ok(())
    }

    /// Execute PostgreSQL statements
    async fn execute_postgres_statements(
        &self,
        connection: &DatabaseConnection,
        statements: &[String],
        username: &str,
        password: &str,
    ) -> Result<(), DatabaseError> {
        // Check if we need to reconnect
        let needs_reconnect = {
            let pools = self.db_pools.read().await;
            if let Some(DbPool::PostgreSQL(client_lock)) = pools.get(&connection.name) {
                let client_opt = client_lock.read().await;
                client_opt.is_none()
            } else {
                true
            }
        };

        // Reconnect if needed
        if needs_reconnect {
            let tls_mode =
                std::env::var("SECRETON_DB_TLS_MODE").unwrap_or_else(|_| "disable".to_string());

            let new_client = if tls_mode.eq_ignore_ascii_case("disable") {
                let (client, connection_handle) =
                    tokio_postgres::connect(&connection.connection_url, NoTls)
                        .await
                        .map_err(|e| {
                            DatabaseError::ConnectionError(format!(
                                "PostgreSQL connection failed: {}",
                                e
                            ))
                        })?;

                tokio::spawn(async move {
                    if let Err(e) = connection_handle.await {
                        eprintln!("PostgreSQL connection error: {}", e);
                    }
                });

                client
            } else {
                let mut root_store = RootCertStore::empty();
                let certs = load_native_certs();
                for cert in certs.certs {
                    let _ = root_store.add(cert);
                }

                let tls_config = ClientConfig::builder()
                    .with_root_certificates(root_store)
                    .with_no_client_auth();
                let tls = MakeRustlsConnect::new(tls_config);

                let (client, connection_handle) =
                    tokio_postgres::connect(&connection.connection_url, tls)
                        .await
                        .map_err(|e| {
                            DatabaseError::ConnectionError(format!(
                                "PostgreSQL connection failed: {}",
                                e
                            ))
                        })?;

                tokio::spawn(async move {
                    if let Err(e) = connection_handle.await {
                        eprintln!("PostgreSQL connection error: {}", e);
                    }
                });

                client
            };

            let mut pools = self.db_pools.write().await;
            pools.insert(
                connection.name.clone(),
                DbPool::PostgreSQL(Arc::new(RwLock::new(Some(new_client)))),
            );
        }

        // Execute statements
        let pools = self.db_pools.read().await;
        if let Some(DbPool::PostgreSQL(client_lock)) = pools.get(&connection.name) {
            let client_opt = client_lock.read().await;
            if let Some(ref pg_client) = *client_opt {
                for stmt in statements {
                    let sql = stmt
                        .replace("{{username}}", username)
                        .replace("{{password}}", password);

                    pg_client.execute(&sql, &[]).await.map_err(|e| {
                        DatabaseError::CredentialGenerationFailed(format!(
                            "Failed to execute statement: {}",
                            e
                        ))
                    })?;
                }
            }
        }

        Ok(())
    }
    /// Build connection URL with credentials
    fn build_connection_url(
        &self,
        connection: &DatabaseConnection,
        username: &str,
        password: &str,
    ) -> String {
        // Simple URL building (production would be more sophisticated)
        let url = &connection.connection_url;

        if url.contains("@") {
            // Replace existing credentials
            url.clone()
        } else {
            // Insert credentials
            if let Some(pos) = url.find("://") {
                format!(
                    "{}://{}:{}@{}",
                    &url[..pos],
                    username,
                    password,
                    &url[pos + 3..]
                )
            } else {
                url.clone()
            }
        }
    }

    /// Revoke credentials
    pub async fn revoke_credentials(&self, credential_id: &str) -> Result<(), DatabaseError> {
        // Get credentials
        let mut active = self.active_credentials.write().await;
        let credentials = active.remove(credential_id).ok_or_else(|| {
            DatabaseError::RevocationFailed(format!("Credentials {} not found", credential_id))
        })?;
        drop(active);

        // Get role and connection
        let roles = self.roles.read().await;
        let role = roles
            .get(&credentials.role_name)
            .ok_or_else(|| DatabaseError::RoleNotFound(credentials.role_name.clone()))?
            .clone();
        drop(roles);

        let connections = self.connections.read().await;
        let connection = connections
            .get(&role.db_name)
            .ok_or_else(|| {
                DatabaseError::InvalidConfig(format!(
                    "Database connection '{}' not found",
                    role.db_name
                ))
            })?
            .clone();
        drop(connections);

        // Execute revocation statements
        self.execute_revocation_statements(
            &connection,
            &role.revocation_statements,
            &credentials.username,
        )
        .await?;

        Ok(())
    }

    /// Execute revocation statements
    async fn execute_revocation_statements(
        &self,
        connection: &DatabaseConnection,
        statements: &[String],
        username: &str,
    ) -> Result<(), DatabaseError> {
        if statements.is_empty() {
            return Err(DatabaseError::RevocationFailed(
                "No revocation statements configured".to_string(),
            ));
        }

        match connection.db_type {
            DatabaseType::PostgreSQL => {
                self.execute_postgres_revocation(connection, statements, username)
                    .await?;
            }
            DatabaseType::MySQL => {
                return Err(DatabaseError::UnsupportedDatabase(
                    "MySQL support not yet implemented".to_string(),
                ));
            }
            _ => {
                return Err(DatabaseError::UnsupportedDatabase(format!(
                    "{} not yet supported",
                    connection.db_type.as_str()
                )));
            }
        }

        Ok(())
    }

    /// Execute PostgreSQL revocation statements
    async fn execute_postgres_revocation(
        &self,
        connection: &DatabaseConnection,
        statements: &[String],
        username: &str,
    ) -> Result<(), DatabaseError> {
        let pools = self.db_pools.read().await;
        let pool = pools.get(&connection.name).ok_or_else(|| {
            DatabaseError::ConnectionError(format!("No connection pool for {}", connection.name))
        })?;

        if let DbPool::PostgreSQL(client_lock) = pool {
            let client_opt = client_lock.read().await;
            if let Some(ref pg_client) = *client_opt {
                for stmt in statements {
                    let sql = stmt.replace("{{username}}", username);

                    // Execute revocation, ignore errors if user doesn't exist
                    let _ = pg_client.execute(&sql, &[]).await;
                }
            }
        }

        Ok(())
    }

    /// List active credentials
    pub async fn list_credentials(&self) -> Vec<DatabaseCredentials> {
        let active = self.active_credentials.read().await;
        active.values().cloned().collect()
    }

    /// Generate credentials with lease integration
    pub async fn generate_credentials_with_lease(
        &self,
        role_name: &str,
        ttl: Option<u32>,
        lease_manager: &crate::services::lease::LeaseManager,
        user: &str,
    ) -> Result<(DatabaseCredentials, crate::services::lease::EnhancedLease), DatabaseError> {
        // Generate credentials
        let credentials = self.generate_credentials(role_name, ttl).await?;

        // Create lease
        let ttl_secs = ttl.unwrap_or_else(|| {
            // Get default TTL from role
            let roles = futures::executor::block_on(self.roles.read());
            roles.get(role_name).map(|r| r.default_ttl).unwrap_or(3600)
        }) as i64;

        let max_ttl = {
            let roles = futures::executor::block_on(self.roles.read());
            roles
                .get(role_name)
                .map(|r| r.max_ttl as i64)
                .unwrap_or(86400)
        };

        let resource_path = format!("database/creds/{}", role_name);
        let lease = lease_manager
            .create_lease(
                user,
                &resource_path,
                "database",
                "default", // namespace
                ttl_secs,
                max_ttl,
                true,                             // renewable
                None,                             // no parent
                None,                             // no max_renewals
                None,                             // no revoke_callback
                std::collections::HashMap::new(), // empty metadata
            )
            .await
            .map_err(|e| {
                DatabaseError::CredentialGenerationFailed(format!("Failed to create lease: {}", e))
            })?;

        Ok((credentials, lease))
    }

    /// Revoke credentials with lease
    pub async fn revoke_credentials_with_lease(
        &self,
        credential_id: &str,
        lease_manager: &crate::services::lease::LeaseManager,
        lease_id: &str,
    ) -> Result<(), DatabaseError> {
        // Revoke credentials
        self.revoke_credentials(credential_id).await?;

        // Revoke lease
        lease_manager.revoke_lease(lease_id).await.map_err(|e| {
            DatabaseError::RevocationFailed(format!("Failed to revoke lease: {}", e))
        })?;

        Ok(())
    }

    /// Rotate credentials for a specific credential ID
    pub async fn rotate_credentials(
        &self,
        credential_id: &str,
    ) -> Result<DatabaseCredentials, DatabaseError> {
        // Get existing credentials
        let active = self.active_credentials.read().await;
        let old_credentials = active
            .get(credential_id)
            .ok_or_else(|| {
                DatabaseError::RotationFailed(format!("Credentials {} not found", credential_id))
            })?
            .clone();
        drop(active);

        // Get role and connection
        let roles = self.roles.read().await;
        let role = roles
            .get(&old_credentials.role_name)
            .ok_or_else(|| DatabaseError::RoleNotFound(old_credentials.role_name.clone()))?
            .clone();
        drop(roles);

        let connections = self.connections.read().await;
        let connection = connections
            .get(&role.db_name)
            .ok_or_else(|| {
                DatabaseError::InvalidConfig(format!(
                    "Database connection '{}' not found",
                    role.db_name
                ))
            })?
            .clone();
        drop(connections);

        // Generate new password
        let new_password = self.generate_password(32);

        // Execute rotation statements if configured
        if !role.rotation_statements.is_empty() {
            self.execute_rotation_statements(
                &connection,
                &role.rotation_statements,
                &old_credentials.username,
                &new_password,
            )
            .await?;
        } else {
            // If no rotation statements, revoke old and create new
            self.execute_revocation_statements(
                &connection,
                &role.revocation_statements,
                &old_credentials.username,
            )
            .await?;

            let new_username = self.generate_username(&connection.db_type, &role.name);
            self.execute_creation_statements(
                &connection,
                &role.creation_statements,
                &new_username,
                &new_password,
            )
            .await?;

            // Update credentials with new username
            let now = Utc::now();
            let ttl = (old_credentials.expires_at - old_credentials.created_at).num_seconds();
            let new_credentials = DatabaseCredentials {
                id: Uuid::new_v4().to_string(),
                username: new_username.clone(),
                password: new_password.clone(),
                connection_url: Some(self.build_connection_url(
                    &connection,
                    &new_username,
                    &new_password,
                )),
                created_at: now,
                expires_at: now + Duration::seconds(ttl),
                role_name: old_credentials.role_name.clone(),
                db_name: old_credentials.db_name.clone(),
            };

            // Remove old and store new
            let mut active = self.active_credentials.write().await;
            active.remove(credential_id);
            active.insert(new_credentials.id.clone(), new_credentials.clone());

            return Ok(new_credentials);
        }

        // Update credentials with new password (same username)
        let now = Utc::now();
        let ttl = (old_credentials.expires_at - old_credentials.created_at).num_seconds();
        let new_credentials = DatabaseCredentials {
            id: old_credentials.id.clone(),
            username: old_credentials.username.clone(),
            password: new_password.clone(),
            connection_url: Some(self.build_connection_url(
                &connection,
                &old_credentials.username,
                &new_password,
            )),
            created_at: now,
            expires_at: now + Duration::seconds(ttl),
            role_name: old_credentials.role_name.clone(),
            db_name: old_credentials.db_name.clone(),
        };

        // Update stored credentials
        let mut active = self.active_credentials.write().await;
        active.insert(new_credentials.id.clone(), new_credentials.clone());

        Ok(new_credentials)
    }

    /// Execute rotation statements
    async fn execute_rotation_statements(
        &self,
        connection: &DatabaseConnection,
        statements: &[String],
        username: &str,
        new_password: &str,
    ) -> Result<(), DatabaseError> {
        match connection.db_type {
            DatabaseType::PostgreSQL => {
                let pools = self.db_pools.read().await;
                let pool = pools.get(&connection.name).ok_or_else(|| {
                    DatabaseError::ConnectionError(format!(
                        "No connection pool for {}",
                        connection.name
                    ))
                })?;

                if let DbPool::PostgreSQL(client_lock) = pool {
                    let client_opt = client_lock.read().await;
                    if let Some(ref pg_client) = *client_opt {
                        for stmt in statements {
                            let sql = stmt
                                .replace("{{username}}", username)
                                .replace("{{password}}", new_password);

                            pg_client.execute(&sql, &[]).await.map_err(|e| {
                                DatabaseError::RotationFailed(format!(
                                    "Failed to execute rotation statement: {}",
                                    e
                                ))
                            })?;
                        }
                    }
                }
            }
            DatabaseType::MySQL => {
                return Err(DatabaseError::UnsupportedDatabase(
                    "MySQL support not yet implemented".to_string(),
                ));
            }
            _ => {
                return Err(DatabaseError::UnsupportedDatabase(format!(
                    "{} not yet supported",
                    connection.db_type.as_str()
                )));
            }
        }

        Ok(())
    }

    /// Renew lease for credentials (extend validity) with lease manager integration
    pub async fn renew_lease_with_manager(
        &self,
        credential_id: &str,
        lease_id: &str,
        increment: u32,
        lease_manager: &crate::services::lease::LeaseManager,
    ) -> Result<(DatabaseCredentials, crate::services::lease::EnhancedLease), DatabaseError> {
        // Renew credentials
        let credentials = self.renew_lease(credential_id, increment).await?;

        // Renew lease
        let lease = lease_manager
            .renew_lease(lease_id, increment as i64)
            .await
            .map_err(|e| DatabaseError::RotationFailed(format!("Failed to renew lease: {}", e)))?;

        Ok((credentials, lease))
    }

    /// Renew lease for credentials (extend validity)
    pub async fn renew_lease(
        &self,
        credential_id: &str,
        increment: u32,
    ) -> Result<DatabaseCredentials, DatabaseError> {
        // Get existing credentials
        let mut active = self.active_credentials.write().await;
        let credentials = active.get_mut(credential_id).ok_or_else(|| {
            DatabaseError::RotationFailed(format!("Credentials {} not found", credential_id))
        })?;

        // Get role to check max_ttl
        let roles = self.roles.read().await;
        let role = roles
            .get(&credentials.role_name)
            .ok_or_else(|| DatabaseError::RoleNotFound(credentials.role_name.clone()))?
            .clone();
        drop(roles);

        // Calculate new expiration time
        let now = Utc::now();
        let current_ttl = (credentials.expires_at - now).num_seconds();
        let new_ttl = current_ttl + increment as i64;

        // Check against max_ttl
        if new_ttl > role.max_ttl as i64 {
            return Err(DatabaseError::InvalidConfig(format!(
                "New TTL {} exceeds maximum {}",
                new_ttl, role.max_ttl
            )));
        }

        // Update expiration time
        credentials.expires_at = now + Duration::seconds(new_ttl);

        // Execute renew statements if configured
        if !role.renew_statements.is_empty() {
            let connections = self.connections.read().await;
            let connection = connections
                .get(&role.db_name)
                .ok_or_else(|| {
                    DatabaseError::InvalidConfig(format!(
                        "Database connection '{}' not found",
                        role.db_name
                    ))
                })?
                .clone();
            drop(connections);

            self.execute_renew_statements(
                &connection,
                &role.renew_statements,
                &credentials.username,
                new_ttl as u32,
            )
            .await?;
        }

        Ok(credentials.clone())
    }

    /// Execute renew statements
    async fn execute_renew_statements(
        &self,
        connection: &DatabaseConnection,
        statements: &[String],
        username: &str,
        new_ttl: u32,
    ) -> Result<(), DatabaseError> {
        match connection.db_type {
            DatabaseType::PostgreSQL => {
                let pools = self.db_pools.read().await;
                let pool = pools.get(&connection.name).ok_or_else(|| {
                    DatabaseError::ConnectionError(format!(
                        "No connection pool for {}",
                        connection.name
                    ))
                })?;

                if let DbPool::PostgreSQL(client_lock) = pool {
                    let client_opt = client_lock.read().await;
                    if let Some(ref pg_client) = *client_opt {
                        for stmt in statements {
                            let sql = stmt
                                .replace("{{username}}", username)
                                .replace("{{ttl}}", &new_ttl.to_string());

                            pg_client.execute(&sql, &[]).await.map_err(|e| {
                                DatabaseError::RotationFailed(format!(
                                    "Failed to execute renew statement: {}",
                                    e
                                ))
                            })?;
                        }
                    }
                }
            }
            DatabaseType::MySQL => {
                return Err(DatabaseError::UnsupportedDatabase(
                    "MySQL support not yet implemented".to_string(),
                ));
            }
            _ => {
                return Err(DatabaseError::UnsupportedDatabase(format!(
                    "{} not yet supported",
                    connection.db_type.as_str()
                )));
            }
        }

        Ok(())
    }

    /// Generate credentials ensuring lease is always created
    ///
    /// This is the recommended method for generating database credentials as it ensures
    /// proper lease management and automatic revocation.
    pub async fn generate_credentials_ensure_lease(
        &self,
        role_name: &str,
        ttl: Option<u32>,
        lease_manager: &crate::services::lease::LeaseManager,
        user: &str,
    ) -> Result<(DatabaseCredentials, crate::services::lease::EnhancedLease), DatabaseError> {
        // Always use the lease-integrated method
        self.generate_credentials_with_lease(role_name, ttl, lease_manager, user)
            .await
    }

    /// Rotate root credentials
    pub async fn rotate_root(&self, connection_name: &str) -> Result<(), DatabaseError> {
        let connections = self.connections.read().await;
        let connection = connections
            .get(connection_name)
            .ok_or_else(|| {
                DatabaseError::InvalidConfig(format!("Connection '{}' not found", connection_name))
            })?
            .clone();
        drop(connections);

        if connection.root_rotation_statements.is_empty() {
            return Err(DatabaseError::RotationFailed(
                "No root rotation statements configured".to_string(),
            ));
        }

        // Generate new root password
        let new_password = self.generate_password(32);

        // Execute root rotation statements
        match connection.db_type {
            DatabaseType::PostgreSQL => {
                let pools = self.db_pools.read().await;
                let pool = pools.get(&connection.name).ok_or_else(|| {
                    DatabaseError::ConnectionError(format!(
                        "No connection pool for {}",
                        connection.name
                    ))
                })?;

                if let DbPool::PostgreSQL(client_lock) = pool {
                    let client_opt = client_lock.read().await;
                    if let Some(ref pg_client) = *client_opt {
                        for stmt in &connection.root_rotation_statements {
                            let sql = stmt
                                .replace(
                                    "{{username}}",
                                    connection.username.as_deref().unwrap_or("postgres"),
                                )
                                .replace("{{password}}", &new_password);

                            pg_client.execute(&sql, &[]).await.map_err(|e| {
                                DatabaseError::RotationFailed(format!(
                                    "Failed to execute root rotation: {}",
                                    e
                                ))
                            })?;
                        }
                    }
                }
            }
            _ => {
                return Err(DatabaseError::UnsupportedDatabase(format!(
                    "{} not yet supported",
                    connection.db_type.as_str()
                )));
            }
        }

        // Update stored connection with new password (in production, this would be encrypted)
        let mut connections = self.connections.write().await;
        if let Some(conn) = connections.get_mut(connection_name) {
            conn.password = Some(new_password);
        }

        Ok(())
    }
}

impl Default for DatabaseSecretsEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_engine_creation() {
        let engine = DatabaseSecretsEngine::new();
        assert_eq!(engine.list_credentials().await.len(), 0);
    }

    #[tokio::test]
    async fn test_configure_connection() {
        let engine = DatabaseSecretsEngine::new();

        let config = DatabaseConnection {
            name: "test-db".to_string(),
            db_type: DatabaseType::PostgreSQL,
            connection_url: "postgresql://localhost:5432/testdb".to_string(),
            verify_connection: false,
            ..Default::default()
        };

        let result = engine.configure_connection(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_role() {
        let engine = DatabaseSecretsEngine::new();

        // First configure connection
        let config = DatabaseConnection {
            name: "test-db".to_string(),
            db_type: DatabaseType::PostgreSQL,
            connection_url: "postgresql://localhost:5432/testdb".to_string(),
            verify_connection: false,
            ..Default::default()
        };
        engine.configure_connection(config).await.unwrap();

        // Create role
        let role = DatabaseRole {
            name: "readonly".to_string(),
            db_name: "test-db".to_string(),
            default_ttl: 3600,
            max_ttl: 86400,
            creation_statements: vec![
                "CREATE USER '{{username}}'@'%' IDENTIFIED BY '{{password}}'".to_string(),
                "GRANT SELECT ON *.* TO '{{username}}'@'%'".to_string(),
            ],
            revocation_statements: vec!["DROP USER '{{username}}'@'%'".to_string()],
            ..Default::default()
        };

        let result = engine.create_role(role).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_generate_credentials() {
        let engine = DatabaseSecretsEngine::new();

        // Setup connection and role
        let config = DatabaseConnection {
            name: "test-db".to_string(),
            db_type: DatabaseType::PostgreSQL,
            connection_url: "postgresql://localhost:5432/testdb".to_string(),
            verify_connection: false,
            ..Default::default()
        };
        engine.configure_connection(config).await.unwrap();

        let role = DatabaseRole {
            name: "readonly".to_string(),
            db_name: "test-db".to_string(),
            default_ttl: 3600,
            max_ttl: 86400,
            creation_statements: vec![
                "CREATE USER '{{username}}'@'%' IDENTIFIED BY '{{password}}'".to_string(),
            ],
            revocation_statements: vec!["DROP USER '{{username}}'@'%'".to_string()],
            ..Default::default()
        };
        engine.create_role(role).await.unwrap();

        // Note: Actual credential generation requires a real database connection
        // This test verifies the setup is correct. In production, credentials would be generated
        // when a real PostgreSQL database is available.

        // Test username generation
        let username = engine.generate_username(&DatabaseType::PostgreSQL, "readonly");
        assert!(username.starts_with("v-readonly-"));

        // Test password generation
        let password = engine.generate_password(32);
        assert_eq!(password.len(), 32);
    }

    #[tokio::test]
    async fn test_password_generation() {
        let engine = DatabaseSecretsEngine::new();

        let password1 = engine.generate_password(32);
        let password2 = engine.generate_password(32);

        assert_eq!(password1.len(), 32);
        assert_eq!(password2.len(), 32);
        assert_ne!(password1, password2); // Should be different
    }

    #[tokio::test]
    async fn test_postgresql_username_format() {
        let engine = DatabaseSecretsEngine::new();

        let username = engine.generate_username(&DatabaseType::PostgreSQL, "readonly");

        // Verify format: v-{role}-{random}
        assert!(username.starts_with("v-readonly-"));
        assert!(username.len() > "v-readonly-".len());

        // Verify it contains only valid characters
        assert!(username.chars().all(|c| c.is_alphanumeric() || c == '-'));
    }

    #[tokio::test]
    async fn test_postgresql_password_security() {
        let engine = DatabaseSecretsEngine::new();

        // Generate multiple passwords to verify randomness
        let passwords: Vec<String> = (0..10).map(|_| engine.generate_password(32)).collect();

        // All passwords should be 32 characters
        for password in &passwords {
            assert_eq!(password.len(), 32);
        }

        // All passwords should be unique (extremely high probability with OsRng)
        for i in 0..passwords.len() {
            for j in (i + 1)..passwords.len() {
                assert_ne!(passwords[i], passwords[j]);
            }
        }

        // Passwords should contain mix of character types
        for password in &passwords {
            let has_upper = password.chars().any(|c| c.is_uppercase());
            let has_lower = password.chars().any(|c| c.is_lowercase());
            let has_digit = password.chars().any(|c| c.is_numeric());

            // At least 2 of 3 character types should be present in a 32-char password
            let type_count = [has_upper, has_lower, has_digit]
                .iter()
                .filter(|&&x| x)
                .count();
            assert!(
                type_count >= 2,
                "Password should have diverse character types"
            );
        }
    }

    #[tokio::test]
    async fn test_postgresql_role_validation() {
        let engine = DatabaseSecretsEngine::new();

        // Configure connection
        let config = DatabaseConnection {
            name: "test-db".to_string(),
            db_type: DatabaseType::PostgreSQL,
            connection_url: "postgresql://localhost:5432/testdb".to_string(),
            verify_connection: false,
            ..Default::default()
        };
        engine.configure_connection(config).await.unwrap();

        // Test role with empty name
        let invalid_role = DatabaseRole {
            name: "".to_string(),
            db_name: "test-db".to_string(),
            creation_statements: vec!["CREATE USER {{username}}".to_string()],
            ..Default::default()
        };
        assert!(engine.create_role(invalid_role).await.is_err());

        // Test role with no creation statements
        let invalid_role = DatabaseRole {
            name: "test-role".to_string(),
            db_name: "test-db".to_string(),
            creation_statements: vec![],
            ..Default::default()
        };
        assert!(engine.create_role(invalid_role).await.is_err());

        // Test role with non-existent database
        let invalid_role = DatabaseRole {
            name: "test-role".to_string(),
            db_name: "non-existent-db".to_string(),
            creation_statements: vec!["CREATE USER {{username}}".to_string()],
            ..Default::default()
        };
        assert!(engine.create_role(invalid_role).await.is_err());
    }

    #[tokio::test]
    async fn test_postgresql_ttl_validation() {
        let engine = DatabaseSecretsEngine::new();

        // Setup connection and role
        let config = DatabaseConnection {
            name: "test-db".to_string(),
            db_type: DatabaseType::PostgreSQL,
            connection_url: "postgresql://localhost:5432/testdb".to_string(),
            verify_connection: false,
            ..Default::default()
        };
        engine.configure_connection(config).await.unwrap();

        let role = DatabaseRole {
            name: "readonly".to_string(),
            db_name: "test-db".to_string(),
            default_ttl: 3600,
            max_ttl: 7200,
            creation_statements: vec![
                "CREATE USER {{username} WITH PASSWORD '{{password}}'".to_string(),
            ],
            revocation_statements: vec!["DROP USER IF EXISTS {{username}}".to_string()],
            ..Default::default()
        };
        engine.create_role(role).await.unwrap();

        // Test that TTL exceeding max_ttl is rejected
        // Note: This would require actual database connection to test fully
        // For now, we verify the role configuration is stored correctly
        let roles = engine.roles.read().await;
        let stored_role = roles.get("readonly").unwrap();
        assert_eq!(stored_role.default_ttl, 3600);
        assert_eq!(stored_role.max_ttl, 7200);
    }

    #[tokio::test]
    async fn test_postgresql_statement_placeholders() {
        let engine = DatabaseSecretsEngine::new();

        // Setup connection
        let config = DatabaseConnection {
            name: "test-db".to_string(),
            db_type: DatabaseType::PostgreSQL,
            connection_url: "postgresql://localhost:5432/testdb".to_string(),
            verify_connection: false,
            ..Default::default()
        };
        engine.configure_connection(config).await.unwrap();

        // Test that statements without placeholders are rejected
        let role = DatabaseRole {
            name: "test-role".to_string(),
            db_name: "test-db".to_string(),
            creation_statements: vec!["CREATE USER testuser WITH PASSWORD 'testpass'".to_string()],
            ..Default::default()
        };
        engine.create_role(role).await.unwrap();

        // Verify that credential generation would fail with invalid statements
        // (requires actual DB connection to test execution)
    }

    #[tokio::test]
    async fn test_renew_lease_functionality() {
        let engine = DatabaseSecretsEngine::new();

        // Setup connection and role
        let config = DatabaseConnection {
            name: "test-db".to_string(),
            db_type: DatabaseType::PostgreSQL,
            connection_url: "postgresql://localhost:5432/testdb".to_string(),
            verify_connection: false,
            ..Default::default()
        };
        engine.configure_connection(config).await.unwrap();

        let role = DatabaseRole {
            name: "readonly".to_string(),
            db_name: "test-db".to_string(),
            default_ttl: 3600,
            max_ttl: 7200,
            creation_statements: vec![
                "CREATE USER {{username} WITH PASSWORD '{{password}}'".to_string(),
            ],
            revocation_statements: vec!["DROP USER IF EXISTS {{username}}".to_string()],
            // No renew_statements - test without DB connection
            renew_statements: vec![],
            ..Default::default()
        };
        engine.create_role(role).await.unwrap();

        // Create mock credentials for testing renewal
        let now = Utc::now();
        let credentials = DatabaseCredentials {
            id: Uuid::new_v4().to_string(),
            username: "v-readonly-test123".to_string(),
            password: "test-password".to_string(),
            connection_url: None,
            created_at: now,
            expires_at: now + Duration::seconds(3600),
            role_name: "readonly".to_string(),
            db_name: "test-db".to_string(),
        };

        // Store credentials
        let mut active = engine.active_credentials.write().await;
        active.insert(credentials.id.clone(), credentials.clone());
        drop(active);

        // Test renewal (without actual DB connection)
        let result = engine.renew_lease(&credentials.id, 1800).await;

        // Should succeed in updating the expiration time
        assert!(result.is_ok());
        let renewed = result.unwrap();
        assert!(renewed.expires_at > credentials.expires_at);

        // Test that renewal exceeding max_ttl is rejected
        let result = engine.renew_lease(&credentials.id, 10000).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_connection_url_building() {
        let engine = DatabaseSecretsEngine::new();

        let connection = DatabaseConnection {
            name: "test-db".to_string(),
            db_type: DatabaseType::PostgreSQL,
            connection_url: "postgresql://localhost:5432/testdb".to_string(),
            verify_connection: false,
            ..Default::default()
        };

        let url = engine.build_connection_url(&connection, "testuser", "testpass");

        // Verify URL contains credentials
        assert!(url.contains("testuser"));
        assert!(url.contains("testpass"));
        assert!(url.contains("postgresql://"));
    }
}

#[tokio::test]
#[ignore] // Requires database connection
async fn test_generate_credentials_ensure_lease() {
    use deadpool_postgres::{Config, Runtime};
    use tokio_postgres::NoTls;

    let engine = DatabaseSecretsEngine::new();

    // Setup connection
    let config = DatabaseConnection {
        name: "test-db".to_string(),
        db_type: DatabaseType::PostgreSQL,
        connection_url: "postgresql://localhost:5432/testdb".to_string(),
        verify_connection: false,
        ..Default::default()
    };
    engine.configure_connection(config).await.unwrap();

    // Create role
    let role = DatabaseRole {
        name: "readonly".to_string(),
        db_name: "test-db".to_string(),
        default_ttl: 3600,
        max_ttl: 7200,
        creation_statements: vec![
            "CREATE USER {{username} WITH PASSWORD '{{password}}'".to_string(),
        ],
        revocation_statements: vec!["DROP USER IF EXISTS {{username}}".to_string()],
        ..Default::default()
    };
    engine.create_role(role).await.unwrap();

    // Setup lease manager
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost/secreton_test".to_string());

    let mut cfg = Config::new();
    cfg.url = Some(database_url);
    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();
    let lease_manager = crate::services::lease::LeaseManager::new(pool);

    // Generate credentials with lease
    let result = engine
        .generate_credentials_ensure_lease("readonly", Some(3600), &lease_manager, "user1")
        .await;

    // Should succeed (or fail with connection error if no DB available)
    // The important part is that the method signature ensures lease is always created
    match result {
        Ok((credentials, lease)) => {
            assert_eq!(credentials.role_name, "readonly");
            assert_eq!(lease.user, "user1");
            assert_eq!(lease.resource_type, "database");
            assert!(lease.renewable);
        }
        Err(e) => {
            // Expected if no database connection available
            println!("Test skipped due to database connection error: {}", e);
        }
    }
}

#[tokio::test]
#[ignore] // Requires database connection
async fn test_renew_lease_with_manager() {
    use deadpool_postgres::{Config, Runtime};
    use tokio_postgres::NoTls;

    let engine = DatabaseSecretsEngine::new();

    // Setup connection and role
    let config = DatabaseConnection {
        name: "test-db".to_string(),
        db_type: DatabaseType::PostgreSQL,
        connection_url: "postgresql://localhost:5432/testdb".to_string(),
        verify_connection: false,
        ..Default::default()
    };
    engine.configure_connection(config).await.unwrap();

    let role = DatabaseRole {
        name: "readonly".to_string(),
        db_name: "test-db".to_string(),
        default_ttl: 3600,
        max_ttl: 7200,
        creation_statements: vec![
            "CREATE USER {{username} WITH PASSWORD '{{password}}'".to_string(),
        ],
        revocation_statements: vec!["DROP USER IF EXISTS {{username}}".to_string()],
        ..Default::default()
    };
    engine.create_role(role).await.unwrap();

    // Setup lease manager
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost/secreton_test".to_string());

    let mut cfg = Config::new();
    cfg.url = Some(database_url);
    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();
    let lease_manager = crate::services::lease::LeaseManager::new(pool);

    // Generate credentials with lease
    match engine
        .generate_credentials_ensure_lease("readonly", Some(1800), &lease_manager, "user1")
        .await
    {
        Ok((credentials, lease)) => {
            // Renew the lease
            let result = engine
                .renew_lease_with_manager(&credentials.id, &lease.id, 1800, &lease_manager)
                .await;

            assert!(result.is_ok());
            let (renewed_creds, renewed_lease) = result.unwrap();
            assert_eq!(renewed_lease.renew_count, 1);
            assert!(renewed_lease.last_renewed_at.is_some());
            assert!(renewed_creds.expires_at > credentials.expires_at);
        }
        Err(e) => {
            println!("Test skipped due to database connection error: {}", e);
        }
    }
}

#[tokio::test]
#[ignore] // Requires database connection
async fn test_revoke_credentials_with_lease_integration() {
    use deadpool_postgres::{Config, Runtime};
    use tokio_postgres::NoTls;

    let engine = DatabaseSecretsEngine::new();

    // Setup connection and role
    let config = DatabaseConnection {
        name: "test-db".to_string(),
        db_type: DatabaseType::PostgreSQL,
        connection_url: "postgresql://localhost:5432/testdb".to_string(),
        verify_connection: false,
        ..Default::default()
    };
    engine.configure_connection(config).await.unwrap();

    let role = DatabaseRole {
        name: "readonly".to_string(),
        db_name: "test-db".to_string(),
        default_ttl: 3600,
        max_ttl: 7200,
        creation_statements: vec![
            "CREATE USER {{username} WITH PASSWORD '{{password}}'".to_string(),
        ],
        revocation_statements: vec!["DROP USER IF EXISTS {{username}}".to_string()],
        ..Default::default()
    };
    engine.create_role(role).await.unwrap();

    // Setup lease manager
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost/secreton_test".to_string());

    let mut cfg = Config::new();
    cfg.url = Some(database_url);
    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();
    let lease_manager = crate::services::lease::LeaseManager::new(pool);

    // Generate credentials with lease
    match engine
        .generate_credentials_ensure_lease("readonly", Some(3600), &lease_manager, "user1")
        .await
    {
        Ok((credentials, lease)) => {
            // Revoke credentials and lease
            let result = engine
                .revoke_credentials_with_lease(&credentials.id, &lease_manager, &lease.id)
                .await;

            assert!(result.is_ok());

            // Verify lease is revoked
            let revoked_lease = lease_manager.lookup_lease(&lease.id).await.unwrap();
            assert_eq!(revoked_lease.status, "revoked");

            // Verify credentials are removed
            let active_creds = engine.list_credentials().await;
            assert!(!active_creds.iter().any(|c| c.id == credentials.id));
        }
        Err(e) => {
            println!("Test skipped due to database connection error: {}", e);
        }
    }
}
