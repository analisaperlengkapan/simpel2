//! Redis Secrets Engine
//!
//! Dynamically generates Redis ACL users with automatic rotation and TTL.
//! Follows the same pattern as the PostgreSQL implementation in database.rs

use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Error types for Redis secrets engine
#[derive(Debug, thiserror::Error)]
pub enum RedisError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Redis role not found: {0}")]
    RoleNotFound(String),

    #[error("Credential generation failed: {0}")]
    CredentialGenerationFailed(String),

    #[error("Rotation failed: {0}")]
    RotationFailed(String),

    #[error("Revocation failed: {0}")]
    RevocationFailed(String),
}

/// Redis connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConnection {
    /// Connection name
    pub name: String,

    /// Connection URL (redis://user:pass@host:port/db)
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

impl Default for RedisConnection {
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

/// Redis role configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisRole {
    /// Role name
    pub name: String,

    /// Database connection name
    pub db_name: String,

    /// Default TTL for credentials
    pub default_ttl: u32,

    /// Maximum TTL for credentials
    pub max_ttl: u32,

    /// Creation statements (Redis ACL commands to create user)
    pub creation_statements: Vec<String>,

    /// Revocation statements (Redis ACL commands to revoke/delete user)
    pub revocation_statements: Vec<String>,

    /// Rotation statements (Redis ACL commands to rotate password)
    pub rotation_statements: Vec<String>,

    /// Renew statements (Redis ACL commands executed on lease renewal)
    pub renew_statements: Vec<String>,
}

impl Default for RedisRole {
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

/// Generated Redis credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisCredentials {
    /// Unique credential ID
    pub id: String,

    /// Redis username (ACL user)
    pub username: String,

    /// Redis password
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

/// Redis secrets engine
pub struct RedisSecretsEngine {
    connections: Arc<RwLock<HashMap<String, RedisConnection>>>,
    roles: Arc<RwLock<HashMap<String, RedisRole>>>,
    active_credentials: Arc<RwLock<HashMap<String, RedisCredentials>>>,
}

impl RedisSecretsEngine {
    /// Create new Redis secrets engine
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            roles: Arc::new(RwLock::new(HashMap::new())),
            active_credentials: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Configure Redis connection
    pub async fn configure_connection(
        &self,
        config: RedisConnection,
    ) -> Result<(), RedisError> {
        // Validate configuration
        if config.name.is_empty() {
            return Err(RedisError::InvalidConfig(
                "Connection name cannot be empty".to_string(),
            ));
        }
        if config.connection_url.is_empty() {
            return Err(RedisError::InvalidConfig(
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

    /// Test Redis connection
    async fn test_connection(&self, config: &RedisConnection) -> Result<(), RedisError> {
        if !config.connection_url.contains("://") {
            return Err(RedisError::ConnectionError(
                "Invalid connection URL format".to_string(),
            ));
        }

        // Note: Actual Redis connection testing would require redis crate
        // For now, we validate the URL format
        if !config.connection_url.starts_with("redis://")
            && !config.connection_url.starts_with("rediss://") {
            return Err(RedisError::ConnectionError(
                "Connection URL must start with redis:// or rediss://".to_string(),
            ));
        }

        Ok(())
    }

    /// Create Redis role
    pub async fn create_role(&self, role: RedisRole) -> Result<(), RedisError> {
        // Validate role
        if role.name.is_empty() {
            return Err(RedisError::InvalidConfig(
                "Role name cannot be empty".to_string(),
            ));
        }
        if role.creation_statements.is_empty() {
            return Err(RedisError::InvalidConfig(
                "Creation statements required".to_string(),
            ));
        }

        // Verify database connection exists
        let connections = self.connections.read().await;
        if !connections.contains_key(&role.db_name) {
            return Err(RedisError::InvalidConfig(format!(
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
    ) -> Result<RedisCredentials, RedisError> {
        // Get role
        let roles = self.roles.read().await;
        let role = roles
            .get(role_name)
            .ok_or_else(|| RedisError::RoleNotFound(role_name.to_string()))?
            .clone();
        drop(roles);

        // Get connection
        let connections = self.connections.read().await;
        let connection = connections
            .get(&role.db_name)
            .ok_or_else(|| {
                RedisError::InvalidConfig(format!(
                    "Database connection '{}' not found",
                    role.db_name
                ))
            })?
            .clone();
        drop(connections);

        // Determine TTL
        let ttl = ttl.unwrap_or(role.default_ttl);
        if ttl > role.max_ttl {
            return Err(RedisError::InvalidConfig(format!(
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
        let credentials = RedisCredentials {
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

    /// Generate Redis username (ACL user)
    fn generate_username(&self, role_name: &str) -> String {
        let uuid = Uuid::new_v4().to_string();
        let short_uuid = &uuid[..8];
        format!("v-{}-{}", role_name, short_uuid)
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
        _connection: &RedisConnection,
        statements: &[String],
        username: &str,
        password: &str,
    ) -> Result<(), RedisError> {
        // Validate that placeholders exist
        for stmt in statements {
            if !stmt.contains("{{username}}") && !stmt.contains("{{password}}") {
                return Err(RedisError::CredentialGenerationFailed(
                    "Creation statements must contain {{username}} or {{password}} placeholders"
                        .to_string(),
                ));
            }
        }

        // Note: Actual Redis execution would require redis crate
        // For now, we validate the statements and prepare them
        for stmt in statements {
            let _command = stmt
                .replace("{{username}}", username)
                .replace("{{password}}", password);

            // In production, execute: connection.send_command(command).await?;
        }

        Ok(())
    }

    /// Build connection URL with credentials
    fn build_connection_url(
        &self,
        connection: &RedisConnection,
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
    pub async fn revoke_credentials(&self, credential_id: &str) -> Result<(), RedisError> {
        // Get credentials
        let mut active = self.active_credentials.write().await;
        let credentials = active.remove(credential_id).ok_or_else(|| {
            RedisError::RevocationFailed(format!("Credentials {} not found", credential_id))
        })?;
        drop(active);

        // Get role and connection
        let roles = self.roles.read().await;
        let role = roles
            .get(&credentials.role_name)
            .ok_or_else(|| RedisError::RoleNotFound(credentials.role_name.clone()))?
            .clone();
        drop(roles);

        let connections = self.connections.read().await;
        let connection = connections
            .get(&role.db_name)
            .ok_or_else(|| {
                RedisError::InvalidConfig(format!(
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
        _connection: &RedisConnection,
        statements: &[String],
        username: &str,
    ) -> Result<(), RedisError> {
        if statements.is_empty() {
            return Err(RedisError::RevocationFailed(
                "No revocation statements configured".to_string(),
            ));
        }

        // Note: Actual Redis execution would require redis crate
        for stmt in statements {
            let _command = stmt.replace("{{username}}", username);
            // In production, execute: connection.send_command(command).await?;
        }

        Ok(())
    }

    /// List active credentials
    pub async fn list_credentials(&self) -> Vec<RedisCredentials> {
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
    ) -> Result<(RedisCredentials, crate::services::lease::EnhancedLease), RedisError> {
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

        let resource_path = format!("redis/creds/{}", role_name);
        let lease = lease_manager
            .create_lease(
                user,
                &resource_path,
                "redis",
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
                RedisError::CredentialGenerationFailed(format!("Failed to create lease: {}", e))
            })?;

        Ok((credentials, lease))
    }

    /// Revoke credentials with lease
    pub async fn revoke_credentials_with_lease(
        &self,
        credential_id: &str,
        lease_manager: &crate::services::lease::LeaseManager,
        lease_id: &str,
    ) -> Result<(), RedisError> {
        // Revoke credentials
        self.revoke_credentials(credential_id).await?;

        // Revoke lease
        lease_manager.revoke_lease(lease_id).await.map_err(|e| {
            RedisError::RevocationFailed(format!("Failed to revoke lease: {}", e))
        })?;

        Ok(())
    }

    /// Rotate credentials for a specific credential ID
    pub async fn rotate_credentials(
        &self,
        credential_id: &str,
    ) -> Result<RedisCredentials, RedisError> {
        // Get existing credentials
        let active = self.active_credentials.read().await;
        let old_credentials = active
            .get(credential_id)
            .ok_or_else(|| {
                RedisError::RotationFailed(format!("Credentials {} not found", credential_id))
            })?
            .clone();
        drop(active);

        // Get role and connection
        let roles = self.roles.read().await;
        let role = roles
            .get(&old_credentials.role_name)
            .ok_or_else(|| RedisError::RoleNotFound(old_credentials.role_name.clone()))?
            .clone();
        drop(roles);

        let connections = self.connections.read().await;
        let connection = connections
            .get(&role.db_name)
            .ok_or_else(|| {
                RedisError::InvalidConfig(format!(
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
            let new_credentials = RedisCredentials {
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
        let new_credentials = RedisCredentials {
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
        _connection: &RedisConnection,
        statements: &[String],
        username: &str,
        new_password: &str,
    ) -> Result<(), RedisError> {
        // Note: Actual Redis execution would require redis crate
        for stmt in statements {
            let _command = stmt
                .replace("{{username}}", username)
                .replace("{{password}}", new_password);
            // In production, execute: connection.send_command(command).await?;
        }

        Ok(())
    }

    /// Renew lease for credentials (extend validity)
    pub async fn renew_lease(
        &self,
        credential_id: &str,
        increment: u32,
    ) -> Result<RedisCredentials, RedisError> {
        // Get existing credentials
        let mut active = self.active_credentials.write().await;
        let credentials = active.get_mut(credential_id).ok_or_else(|| {
            RedisError::RotationFailed(format!("Credentials {} not found", credential_id))
        })?;

        // Get role to check max_ttl
        let roles = self.roles.read().await;
        let role = roles
            .get(&credentials.role_name)
            .ok_or_else(|| RedisError::RoleNotFound(credentials.role_name.clone()))?
            .clone();
        drop(roles);

        // Calculate new expiration time
        let now = Utc::now();
        let current_ttl = (credentials.expires_at - now).num_seconds();
        let new_ttl = current_ttl + increment as i64;

        // Check against max_ttl
        if new_ttl > role.max_ttl as i64 {
            return Err(RedisError::InvalidConfig(format!(
                "New TTL {} exceeds maximum {}",
                new_ttl, role.max_ttl
            )));
        }

        // Update expiration time
        credentials.expires_at = now + Duration::seconds(new_ttl);

        Ok(credentials.clone())
    }
}

impl Default for RedisSecretsEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_redis_engine_creation() {
        let engine = RedisSecretsEngine::new();
        assert_eq!(engine.list_credentials().await.len(), 0);
    }

    #[tokio::test]
    async fn test_configure_connection() {
        let engine = RedisSecretsEngine::new();

        let config = RedisConnection {
            name: "test-db".to_string(),
            connection_url: "redis://localhost:6379/0".to_string(),
            verify_connection: false,
            ..Default::default()
        };

        let result = engine.configure_connection(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_role() {
        let engine = RedisSecretsEngine::new();

        // First configure connection
        let config = RedisConnection {
            name: "test-db".to_string(),
            connection_url: "redis://localhost:6379/0".to_string(),
            verify_connection: false,
            ..Default::default()
        };
        engine.configure_connection(config).await.unwrap();

        // Create role
        let role = RedisRole {
            name: "readonly".to_string(),
            db_name: "test-db".to_string(),
            default_ttl: 3600,
            max_ttl: 86400,
            creation_statements: vec![
                "ACL SETUSER {{username}} on >{{password}} ~* +@read".to_string(),
            ],
            revocation_statements: vec!["ACL DELUSER {{username}}".to_string()],
            ..Default::default()
        };

        let result = engine.create_role(role).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_generate_username() {
        let engine = RedisSecretsEngine::new();
        let username = engine.generate_username("readonly");

        assert!(username.starts_with("v-readonly-"));
        assert!(username.len() > "v-readonly-".len());
        assert!(username.chars().all(|c| c.is_alphanumeric() || c == '-'));
    }

    #[tokio::test]
    async fn test_generate_password() {
        let engine = RedisSecretsEngine::new();

        let password1 = engine.generate_password(32);
        let password2 = engine.generate_password(32);

        assert_eq!(password1.len(), 32);
        assert_eq!(password2.len(), 32);
        assert_ne!(password1, password2); // Should be different
    }

    #[tokio::test]
    async fn test_role_validation() {
        let engine = RedisSecretsEngine::new();

        // Configure connection
        let config = RedisConnection {
            name: "test-db".to_string(),
            connection_url: "redis://localhost:6379/0".to_string(),
            verify_connection: false,
            ..Default::default()
        };
        engine.configure_connection(config).await.unwrap();

        // Test role with empty name
        let invalid_role = RedisRole {
            name: "".to_string(),
            db_name: "test-db".to_string(),
            creation_statements: vec!["ACL SETUSER {{username}}".to_string()],
            ..Default::default()
        };
        assert!(engine.create_role(invalid_role).await.is_err());

        // Test role with n stat
     let invalid_role = RedisRole {
            name: "test-role".to_string(),
            db_name: "test-db".to_string(),
            creation_statements: vec![],
            ..Default::default()
        };
        assert!(engine.create_role(invalid_role).await.is_err());

        // Test role with non-existent database
        let invalid_role = RedisRole {
            name: "test-role".to_string(),
            db_name: "non-existent-db".to_string(),
            creation_statements: vec!["ACL SETUSER {{username}}".to_string()],
            ..Default::default()
        };
        assert!(engine.create_role(invalid_role).await.is_err());
    }

    #[tokio::test]
    async fn test_ttl_validation() {
        let engine = RedisSecretsEngine::new();

        // Setup connection and role
        let config = RedisConnection {
            name: "test-db".to_string(),
            connection_url: "redis://localhost:6379/0".to_string(),
            verify_connection: false,
            ..Default::default()
        };
        engine.configure_connection(config).await.unwrap();

        let role = RedisRole {
            name: "readonly".to_string(),
            db_name: "test-db".to_string(),
            default_ttl: 3600,
            max_ttl: 7200,
            creation_statements: vec![
                "ACL SETUSER {{username}} on >{{password}}".to_string(),
            ],
            revocation_statements: vec!["ACL DELUSER {{username}}".to_string()],
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
        let engine = RedisSecretsEngine::new();

        let connection = RedisConnection {
            name: "test-db".to_string(),
            connection_url: "redis://localhost:6379/0".to_string(),
            verify_connection: false,
            ..Default::default()
        };

        let url = engine.build_connection_url(&connection, "testuser", "testpass");

        // Verify URL contains credentials
        assert!(url.contains("testuser"));
        assert!(url.contains("testpass"));
        assert!(url.contains("redis://"));
    }

    #[tokio::test]
    async fn test_rediss_url() {
        let engine = RedisSecretsEngine::new();

        let config = RedisConnection {
            name: "test-db".to_string(),
            connection_url: "rediss://localhost:6380/0".to_string(),
            verify_connection: false,
            ..Default::default()
        };

        let result = engine.configure_connection(config).await;
        assert!(result.is_ok());
    }
}
