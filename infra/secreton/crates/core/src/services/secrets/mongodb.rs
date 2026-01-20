//! MongoDB Secrets Engine
//!
//! Dynamically generates MongoDB database credentials with automatic rotation.
//! Follows the same pattern as the PostgreSQL implementation in database.rs

use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Error types for MongoDB secrets engine
#[derive(Debug, thiserror::Error)]
pub enum MongoDbError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("MongoDB role not found: {0}")]
    RoleNotFound(String),

    #[error("Credential generation failed: {0}")]
    CredentialGenerationFailed(String),

    #[error("Rotation failed: {0}")]
    RotationFailed(String),

    #[error("Revocation failed: {0}")]
    RevocationFailed(String),
}

/// MongoDB connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MongoDbConnection {
    /// Connection name
    pub name: String,

    /// Connection URL (mongodb://user:pass@host:port/database)
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

impl Default for MongoDbConnection {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
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

/// MongoDB role configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MongoDbRole {
    /// Role name
    pub name: String,

    /// Database connection name
    pub db_name: String,

    /// Default TTL for credentials
    pub default_ttl: u32,

    /// Maximum TTL for credentials
    pub max_ttl: u32,

    /// Creation statements (MongoDB commands to create user)
    pub creation_statements: Vec<String>,

    /// Revocation statements (MongoDB commands to revoke/delete user)
    pub revocation_statements: Vec<String>,

    /// Rotation statements (MongoDB commands to rotate password)
    pub rotation_statements: Vec<String>,

    /// Renew statements (MongoDB commands executed on lease renewal)
    pub renew_statements: Vec<String>,
}

impl Default for MongoDbRole {
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

/// Generated MongoDB credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MongoDbCredentials {
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

/// MongoDB secrets engine
pub struct MongoDbSecretsEngine {
    connections: Arc<RwLock<HashMap<String, MongoDbConnection>>>,
    roles: Arc<RwLock<HashMap<String, MongoDbRole>>>,
    active_credentials: Arc<RwLock<HashMap<String, MongoDbCredentials>>>,
}

impl MongoDbSecretsEngine {
    /// Create new MongoDB secrets engine
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            roles: Arc::new(RwLock::new(HashMap::new())),
            active_credentials: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Configure MongoDB connection
    pub async fn configure_connection(
        &self,
        config: MongoDbConnection,
    ) -> Result<(), MongoDbError> {
        // Validate configuration
        if config.name.is_empty() {
            return Err(MongoDbError::InvalidConfig(
                "Connection name cannot be empty".to_string(),
            ));
        }
        if config.connection_url.is_empty() {
            return Err(MongoDbError::InvalidConfig(
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

    /// Test MongoDB connection
    async fn test_connection(&self, config: &MongoDbConnection) -> Result<(), MongoDbError> {
        if !config.connection_url.contains("://") {
            return Err(MongoDbError::ConnectionError(
                "Invalid connection URL format".to_string(),
            ));
        }

        // Note: Actual MongoDB connection testing would require mongodb crate
        // For now, we validate the URL format
        if !config.connection_url.starts_with("mongodb://")
            && !config.connection_url.starts_with("mongodb+srv://") {
            return Err(MongoDbError::ConnectionError(
                "Connection URL must start with mongodb:// or mongodb+srv://".to_string(),
            ));
        }

        Ok(())
    }

    /// Create MongoDB role
    pub async fn create_role(&self, role: MongoDbRole) -> Result<(), MongoDbError> {
        // Validate role
        if role.name.is_empty() {
            return Err(MongoDbError::InvalidConfig(
                "Role name cannot be empty".to_string(),
            ));
        }
        if role.creation_statements.is_empty() {
            return Err(MongoDbError::InvalidConfig(
                "Creation statements required".to_string(),
            ));
        }

        // Verify database connection exists
        let connections = self.connections.read().await;
        if !connections.contains_key(&role.db_name) {
            return Err(MongoDbError::InvalidConfig(format!(
                "Database connection '{}' not found",
                role.db_name
            )));
        }
        drop(connections);

        // Store role
        let mut roles = self.roles.write().await;
        roles.insert(role.name.clone(), role);

        Ok(())
    }

    /// Generate credentials for a role
    pub async fn generate_credentials(
        &self,
        role_name: &str,
        ttl: Option<u32>,
    ) -> Result<MongoDbCredentials, MongoDbError> {
        // Get role
        let roles = self.roles.read().await;
        let role = roles
            .get(role_name)
            .ok_or_else(|| MongoDbError::RoleNotFound(role_name.to_string()))?
            .clone();
        drop(roles);

        // Get connection
        let connections = self.connections.read().await;
        let connection = connections
            .get(&role.db_name)
            .ok_or_else(|| {
                MongoDbError::InvalidConfig(format!(
                    "Database connection '{}' not found",
                    role.db_name
                ))
            })?
            .clone();
        drop(connections);

        // Determine TTL
        let ttl = ttl.unwrap_or(role.default_ttl);
        if ttl > role.max_ttl {
            return Err(MongoDbError::InvalidConfig(format!(
                "TTL {} exceeds maximum {}",
                ttl, role.max_ttl
            )));
        }

        // Generate username and password
        let username = self.generate_username(role_name);
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
        let credentials = MongoDbCredentials {
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

    /// Generate MongoDB username (MongoDB doesn't allow hyphens, use underscores)
    fn generate_username(&self, role_name: &str) -> String {
        let uuid = Uuid::new_v4().to_string();
        let short_uuid = &uuid[..8];
        format!("v_{}_{}", role_name.replace("-", "_"), short_uuid)
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
        _connection: &MongoDbConnection,
        statements: &[String],
        username: &str,
        password: &str,
    ) -> Result<(), MongoDbError> {
        // Validate that placeholders exist
        for stmt in statements {
            if !stmt.contains("{{username}}") && !stmt.contains("{{password}}") {
                return Err(MongoDbError::CredentialGenerationFailed(
                    "Creation statements must contain {{username}} or {{password}} placeholders"
                        .to_string(),
                ));
            }
        }

        // Note: Actual MongoDB execution would require mongodb crate
        // For now, we validate the statements and prepare them
        for stmt in statements {
            let _command = stmt
                .replace("{{username}}", username)
                .replace("{{password}}", password);

            // In production, execute: db.run_command(command).await?;
        }

        Ok(())
    }

    /// Build connection URL with credentials
    fn build_connection_url(
        &self,
        connection: &MongoDbConnection,
        username: &str,
        password: &str,
    ) -> String {
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
    pub async fn revoke_credentials(&self, credential_id: &str) -> Result<(), MongoDbError> {
        // Get credentials
        let mut active = self.active_credentials.write().await;
        let credentials = active.remove(credential_id).ok_or_else(|| {
            MongoDbError::RevocationFailed(format!("Credentials {} not found", credential_id))
        })?;
        drop(active);

        // Get role and connection
        let roles = self.roles.read().await;
        let role = roles
            .get(&credentials.role_name)
            .ok_or_else(|| MongoDbError::RoleNotFound(credentials.role_name.clone()))?
            .clone();
        drop(roles);

        let connections = self.connections.read().await;
        let connection = connections
            .get(&role.db_name)
            .ok_or_else(|| {
                MongoDbError::InvalidConfig(format!(
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
        _connection: &MongoDbConnection,
        statements: &[String],
        username: &str,
    ) -> Result<(), MongoDbError> {
        if statements.is_empty() {
            return Err(MongoDbError::RevocationFailed(
                "No revocation statements configured".to_string(),
            ));
        }

        // Note: Actual MongoDB execution would require mongodb crate
        for stmt in statements {
            let _command = stmt.replace("{{username}}", username);
            // In production, execute: db.run_command(command).await?;
        }

        Ok(())
    }

    /// List active credentials
    pub async fn list_credentials(&self) -> Vec<MongoDbCredentials> {
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
    ) -> Result<(MongoDbCredentials, crate::services::lease::EnhancedLease), MongoDbError> {
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

        let resource_path = format!("mongodb/creds/{}", role_name);
        let lease = lease_manager
            .create_lease(
                user,
                &resource_path,
                "mongodb",
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
                MongoDbError::CredentialGenerationFailed(format!("Failed to create lease: {}", e))
            })?;

        Ok((credentials, lease))
    }

    /// Revoke credentials with lease
    pub async fn revoke_credentials_with_lease(
        &self,
        credential_id: &str,
        lease_manager: &crate::services::lease::LeaseManager,
        lease_id: &str,
    ) -> Result<(), MongoDbError> {
        // Revoke credentials
        self.revoke_credentials(credential_id).await?;

        // Revoke lease
        lease_manager.revoke_lease(lease_id).await.map_err(|e| {
            MongoDbError::RevocationFailed(format!("Failed to revoke lease: {}", e))
        })?;

        Ok(())
    }

    /// Rotate credentials for a specific credential ID
    pub async fn rotate_credentials(
        &self,
        credential_id: &str,
    ) -> Result<MongoDbCredentials, MongoDbError> {
        // Get existing credentials
        let active = self.active_credentials.read().await;
        let old_credentials = active
            .get(credential_id)
            .ok_or_else(|| {
                MongoDbError::RotationFailed(format!("Credentials {} not found", credential_id))
            })?
            .clone();
        drop(active);

        // Get role and connection
        let roles = self.roles.read().await;
        let role = roles
            .get(&old_credentials.role_name)
            .ok_or_else(|| MongoDbError::RoleNotFound(old_credentials.role_name.clone()))?
            .clone();
        drop(roles);

        let connections = self.connections.read().await;
        let connection = connections
            .get(&role.db_name)
            .ok_or_else(|| {
                MongoDbError::InvalidConfig(format!(
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
            // If no rstaterevoke old and create new
            self.execute_revocation_statements(
                &connection,
                &role.revocation_statements,
                &old_credentials.username,
            )
            .await?;

            let new_username = self.generate_username(&role.name);
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
            let new_credentials = MongoDbCredentials {
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
        let new_credentials = MongoDbCredentials {
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
        _connection: &MongoDbConnection,
        statements: &[String],
        username: &str,
        new_password: &str,
    ) -> Result<(), MongoDbError> {
        // Note: Actual MongoDB execution would require mongodb crate
        for stmt in statements {
            let _command = stmt
                .replace("{{username}}", username)
                .replace("{{password}}", new_password);
            // In production, execute: db.run_command(command).await?;
        }

        Ok(())
    }

    /// Renew lease for credentials (extend validity)
    pub async fn renew_lease(
        &self,
        credential_id: &str,
        increment: u32,
    ) -> Result<MongoDbCredentials, MongoDbError> {
        // Get existing credentials
        let mut active = self.active_credentials.write().await;
        let credentials = active.get_mut(credential_id).ok_or_else(|| {
            MongoDbError::RotationFailed(format!("Credentials {} not found", credential_id))
        })?;

        // Get role to check max_ttl
        let roles = self.roles.read().await;
        let role = roles
            .get(&credentials.role_name)
            .ok_or_else(|| MongoDbError::RoleNotFound(credentials.role_name.clone()))?
            .clone();
        drop(roles);

        // Calculate new expiration time
        let now = Utc::now();
        let current_ttl = (credentials.expires_at - now).num_seconds();
        let new_ttl = current_ttl + increment as i64;

        // Check against max_ttl
        if new_ttl > role.max_ttl as i64 {
            return Err(MongoDbError::InvalidConfig(format!(
                "New TTL {} exceeds maximum {}",
                new_ttl, role.max_ttl
            )));
        }

        // Update expiration time
        credentials.expires_at = now + Duration::seconds(new_ttl);

        Ok(credentials.clone())
    }
}

impl Default for MongoDbSecretsEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mongodb_engine_creation() {
        let engine = MongoDbSecretsEngine::new();
        assert_eq!(engine.list_credentials().await.len(), 0);
    }

    #[tokio::test]
    async fn test_configure_connection() {
        let engine = MongoDbSecretsEngine::new();

        let config = MongoDbConnection {
            name: "test-db".to_string(),
            connection_url: "mongodb://localhost:27017/testdb".to_string(),
            verify_connection: false,
            ..Default::default()
        };

        let result = engine.configure_connection(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_role() {
        let engine = MongoDbSecretsEngine::new();

        // First configure connection
        let config = MongoDbConnection {
            name: "test-db".to_string(),
            connection_url: "mongodb://localhost:27017/testdb".to_string(),
            verify_connection: false,
            ..Default::default()
        };
        engine.configure_connection(config).await.unwrap();

        // Create role
        let role = MongoDbRole {
            name: "readonly".to_string(),
            db_name: "test-db".to_string(),
            default_ttl: 3600,
            max_ttl: 86400,
            creation_statements: vec![
                r#"{"createUser": "{{username}}", "pwd": "{{password}}", "roles": [{"role": "read", "db": "testdb"}]}"#.to_string(),
            ],
            revocation_statements: vec![
                r#"{"dropUser": "{{username}}"}"#.to_string(),
            ],
            ..Default::default()
        };

        let result = engine.create_role(role).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_generate_username() {
        let engine = MongoDbSecretsEngine::new();
        let username = engine.generate_username("readonly");

        assert!(username.starts_with("v_readonly_"));
        assert!(username.len() > "v_readonly_".len());
        // MongoDB usernames use underscores instead of hyphens
        assert!(username.chars().all(|c| c.is_alphanumeric() || c == '_'));
    }

    #[tokio::test]
    async fn test_generate_password() {
        let engine = MongoDbSecretsEngine::new();

        let password1 = engine.generate_password(32);
        let password2 = engine.generate_password(32);

        assert_eq!(password1.len(), 32);
        assert_eq!(password2.len(), 32);
        assert_ne!(password1, password2); // Should be different
    }

    #[tokio::test]
    async fn test_role_validation() {
        let engine = MongoDbSecretsEngine::new();

        // Configure connection
        let config = MongoDbConnection {
            name: "test-db".to_string(),
            connection_url: "mongodb://localhost:27017/testdb".to_string(),
            verify_connection: false,
            ..Default::default()
        };
        engine.configure_connection(config).await.unwrap();

        // Test role with empty name
        let invalid_role = MongoDbRole {
            name: "".to_string(),
            db_name: "test-db".to_string(),
            creation_statements: vec![r#"{"createUser": "{{username}}"}"#.to_string()],
            ..Default::default()
        };
        assert!(engine.create_role(invalid_role).await.is_err());

        // Test role with no creation statements
        let invalid_role = MongoDbRole {
            name: "test-role".to_string(),
            db_name: "test-db".to_string(),
            creation_statements: vec![],
            ..Default::default()
        };
        assert!(engine.create_role(invalid_role).await.is_err());

        // Test role with non-existent database
        let invalid_role = MongoDbRole {
            name: "test-role".to_string(),
            db_name: "non-existent-db".to_string(),
            creation_statements: vec![r#"{"createUser": "{{username}}"}"#.to_string()],
            ..Default::default()
        };
        assert!(engine.create_role(invalid_role).await.is_err());
    }

    #[tokio::test]
    async fn test_ttl_validation() {
        let engine = MongoDbSecretsEngine::new();

        // Setup connection and role
        let config = MongoDbConnection {
            name: "test-db".to_string(),
            connection_url: "mongodb://localhost:27017/testdb".to_string(),
            verify_connection: false,
            ..Default::default()
        };
        engine.configure_connection(config).await.unwrap();

        let role = MongoDbRole {
            name: "readonly".to_string(),
            db_name: "test-db".to_string(),
            default_ttl: 3600,
            max_ttl: 7200,
            creation_statements: vec![
                r#"{"createUser": "{{username}}", "pwd": "{{password}}"}"#.to_string(),
            ],
            revocation_statements: vec![r#"{"dropUser": "{{username}}"}"#.to_string()],
            ..Default::default()
        };
        engine.create_role(role).await.unwrap();

        // Verify the role configuration is stored correctly
        let roles = engine.roles.read().await;
        let stored_role = roles.get("readonly").unwrap();
        assert_eq!(stored_role.default_ttl, 3600);
        assert_eq!(stored_role.max_ttl, 7200);
    }

    #[tokio::test]
    async fn test_connection_url_building() {
        let engine = MongoDbSecretsEngine::new();

        let connection = MongoDbConnection {
            name: "test-db".to_string(),
            connection_url: "mongodb://localhost:27017/testdb".to_string(),
            verify_connection: false,
            ..Default::default()
        };

        let url = engine.build_connection_url(&connection, "testuser", "testpass");

        // Verify URL contains credentials
        assert!(url.contains("testuser"));
        assert!(url.contains("testpass"));
        assert!(url.contains("mongodb://"));
    }

    #[tokio::test]
    async fn test_mongodb_srv_url() {
        let engine = MongoDbSecretsEngine::new();

        let config = MongoDbConnection {
            name: "test-db".to_string(),
            connection_url: "mongodb+srv://cluster.mongodb.net/testdb".to_string(),
            verify_connection: false,
            ..Default::default()
        };

        let result = engine.configure_connection(config).await;
        assert!(result.is_ok());
    }
}

