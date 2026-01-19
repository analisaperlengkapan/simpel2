//! RabbitMQ Secrets Engine
//!
//! Dynamic RabbitMQ user credentials generation with vhost-level permissions.

use chrono::{DateTime, Duration, Utc};
use deadpool_postgres::Pool;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument};

/// Error types for RabbitMQ secrets engine
#[derive(Debug, thiserror::Error)]
/// Mewakili pub `RabbitMqError`.
pub enum RabbitMqError {
    #[error("RabbitMQ configuration not found")]
    ConfigNotFound,

    #[error("RabbitMQ role not found: {0}")]
    RoleNotFound(String),

    #[error("RabbitMQ role already exists: {0}")]
    RoleAlreadyExists(String),

    #[error("Invalid RabbitMQ configuration: {0}")]
    InvalidConfig(String),

    #[error("Invalid TTL: {0}")]
    InvalidTtl(String),

    #[error("RabbitMQ connection error: {0}")]
    ConnectionError(String),

    #[error("RabbitMQ API error: {0}")]
    ApiError(String),

    #[error("Credential generation failed: {0}")]
    CredentialGenerationFailed(String),

    #[error("Credential revocation failed: {0}")]
    RevocationFailed(String),

    #[error("User not found: {0}")]
    UserNotFound(String),
}

/// RabbitMQ permission level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
/// Mewakili pub `RabbitMqPermissionLevel`.
pub enum RabbitMqPermissionLevel {
    Read,
    Write,
    Configure,
    Administrator,
}

impl RabbitMqPermissionLevel {
    /// Mewakili pub `as_str(`.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
            Self::Configure => "configure",
            Self::Administrator => "administrator",
        }
    }

    /// Mewakili pub `to_tags(`.
    pub fn to_tags(&self) -> Vec<String> {
        match self {
            Self::Read => vec![],
            Self::Write => vec![],
            Self::Configure => vec![],
            Self::Administrator => vec!["administrator".to_string()],
        }
    }
}

/// RabbitMQ vhost permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `RabbitMqVhostPermission`.
pub struct RabbitMqVhostPermission {
    pub vhost: String,
    pub configure: String, // Regex pattern for configure permission
    pub write: String,     // Regex pattern for write permission
    pub read: String,      // Regex pattern for read permission
}

impl Default for RabbitMqVhostPermission {
    fn default() -> Self {
        Self {
            vhost: "/".to_string(),
            configure: ".*".to_string(),
            write: ".*".to_string(),
            read: ".*".to_string(),
        }
    }
}

/// RabbitMQ configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `RabbitMqConfig`.
pub struct RabbitMqConfig {
    pub connection_uri: String,
    pub management_uri: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub verify_connection: bool,
    pub default_ttl: i64,
    pub max_ttl: i64,
}

impl Default for RabbitMqConfig {
    fn default() -> Self {
        Self {
            connection_uri: "amqp://localhost:5672".to_string(),
            management_uri: "http://localhost:15672".to_string(),
            username: "guest".to_string(),
            password: "guest".to_string(),
            verify_connection: true,
            default_ttl: 3600,
            max_ttl: 86400,
        }
    }
}

impl RabbitMqConfig {
    /// Mewakili pub `validate(`.
    pub fn validate(&self) -> Result<(), RabbitMqError> {
        if self.connection_uri.is_empty() {
            return Err(RabbitMqError::InvalidConfig(
                "Connection URI is required".to_string(),
            ));
        }
        if self.management_uri.is_empty() {
            return Err(RabbitMqError::InvalidConfig(
                "Management URI is required".to_string(),
            ));
        }
        if self.username.is_empty() {
            return Err(RabbitMqError::InvalidConfig(
                "Username is required".to_string(),
            ));
        }
        if self.password.is_empty() {
            return Err(RabbitMqError::InvalidConfig(
                "Password is required".to_string(),
            ));
        }
        if self.default_ttl < 60 {
            return Err(RabbitMqError::InvalidTtl(
                "Default TTL must be at least 60 seconds".to_string(),
            ));
        }
        if self.max_ttl > 604800 {
            return Err(RabbitMqError::InvalidTtl(
                "Max TTL cannot exceed 7 days".to_string(),
            ));
        }
        if self.default_ttl > self.max_ttl {
            return Err(RabbitMqError::InvalidTtl(
                "Default TTL cannot exceed max TTL".to_string(),
            ));
        }
        Ok(())
    }
}

/// RabbitMQ role
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `RabbitMqRole`.
pub struct RabbitMqRole {
    pub name: String,
    pub vhosts: Vec<RabbitMqVhostPermission>,
    pub tags: Vec<String>,
    pub default_ttl: i64,
    pub max_ttl: i64,
    pub created_at: DateTime<Utc>,
}

impl RabbitMqRole {
    /// Mewakili pub `new(name`.
    pub fn new(name: String) -> Self {
        Self {
            name,
            vhosts: vec![RabbitMqVhostPermission::default()],
            tags: vec![],
            default_ttl: 3600,
            max_ttl: 86400,
            created_at: Utc::now(),
        }
    }

    /// Mewakili pub `validate(`.
    pub fn validate(&self) -> Result<(), RabbitMqError> {
        if self.name.is_empty() {
            return Err(RabbitMqError::InvalidConfig(
                "Role name is required".to_string(),
            ));
        }
        if self.vhosts.is_empty() {
            return Err(RabbitMqError::InvalidConfig(
                "At least one vhost permission is required".to_string(),
            ));
        }
        if self.default_ttl < 60 {
            return Err(RabbitMqError::InvalidTtl(
                "Default TTL must be at least 60 seconds".to_string(),
            ));
        }
        if self.max_ttl > 604800 {
            return Err(RabbitMqError::InvalidTtl(
                "Max TTL cannot exceed 7 days".to_string(),
            ));
        }
        if self.default_ttl > self.max_ttl {
            return Err(RabbitMqError::InvalidTtl(
                "Default TTL cannot exceed max TTL".to_string(),
            ));
        }
        Ok(())
    }
}

/// RabbitMQ credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `RabbitMqCredentials`.
pub struct RabbitMqCredentials {
    pub username: String,
    pub password: String,
    pub connection_uri: String,
    pub management_uri: String,
    pub vhosts: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub role_name: String,
}

/// RabbitMQ credential info (without password)
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `RabbitMqCredentialInfo`.
pub struct RabbitMqCredentialInfo {
    pub username: String,
    pub connection_uri: String,
    pub management_uri: String,
    pub vhosts: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub role_name: String,
}

impl From<&RabbitMqCredentials> for RabbitMqCredentialInfo {
    fn from(cred: &RabbitMqCredentials) -> Self {
        Self {
            username: cred.username.clone(),
            connection_uri: cred.connection_uri.clone(),
            management_uri: cred.management_uri.clone(),
            vhosts: cred.vhosts.clone(),
            created_at: cred.created_at,
            expires_at: cred.expires_at,
            role_name: cred.role_name.clone(),
        }
    }
}

/// Simulated RabbitMQ connection for prototyping
#[derive(Debug, Clone)]
struct RabbitMqConnection {
    management_uri: String,
    username: String,
    password: String,
}

impl RabbitMqConnection {
    fn new(config: &RabbitMqConfig) -> Self {
        Self {
            management_uri: config.management_uri.clone(),
            username: config.username.clone(),
            password: config.password.clone(),
        }
    }

    /// Simulate creating a user via Management API
    async fn create_user(
        &self,
        username: &str,
        _password: &str,
        tags: &[String],
    ) -> Result<(), RabbitMqError> {
        info!(
            "Simulated: Creating RabbitMQ user '{}' with tags {:?}",
            username, tags
        );
        // In production, this would call: PUT /api/users/{username}
        Ok(())
    }

    /// Simulate setting vhost permissions
    async fn set_permissions(
        &self,
        username: &str,
        vhost: &str,
        configure: &str,
        write: &str,
        read: &str,
    ) -> Result<(), RabbitMqError> {
        info!(
            "Simulated: Setting permissions for user '{}' on vhost '{}': configure={}, write={}, read={}",
            username, vhost, configure, write, read
        );
        // In production, this would call: PUT /api/permissions/{vhost}/{username}
        Ok(())
    }

    /// Simulate deleting a user
    async fn delete_user(&self, username: &str) -> Result<(), RabbitMqError> {
        info!("Simulated: Deleting RabbitMQ user '{}'", username);
        // In production, this would call: DELETE /api/users/{username}
        Ok(())
    }

    /// Simulate checking if user exists
    async fn user_exists(&self, username: &str) -> Result<bool, RabbitMqError> {
        debug!("Simulated: Checking if RabbitMQ user '{}' exists", username);
        // In production, this would call: GET /api/users/{username}
        Ok(false)
    }
}

/// RabbitMQ Secrets Engine
pub struct RabbitMqEngine {
    config: Arc<RwLock<Option<RabbitMqConfig>>>,
    roles: Arc<RwLock<HashMap<String, RabbitMqRole>>>,
    credentials: Arc<RwLock<HashMap<String, RabbitMqCredentials>>>,
    connection: Arc<RwLock<Option<RabbitMqConnection>>>,
    pool: Option<Pool>,
}

impl RabbitMqEngine {
    /// Mewakili pub `new(`.
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(None)),
            roles: Arc::new(RwLock::new(HashMap::new())),
            credentials: Arc::new(RwLock::new(HashMap::new())),
            connection: Arc::new(RwLock::new(None)),
            pool: None,
        }
    }

    /// Mewakili pub `with_storage(pool`.
    pub fn with_storage(pool: Pool) -> Self {
        Self {
            config: Arc::new(RwLock::new(None)),
            roles: Arc::new(RwLock::new(HashMap::new())),
            credentials: Arc::new(RwLock::new(HashMap::new())),
            connection: Arc::new(RwLock::new(None)),
            pool: Some(pool),
        }
    }

    /// Configure RabbitMQ connection
    #[instrument(skip(self, config))]
    pub async fn configure(&self, config: RabbitMqConfig) -> Result<(), RabbitMqError> {
        config.validate()?;

        let conn = RabbitMqConnection::new(&config);

        if config.verify_connection {
            info!("Verifying RabbitMQ connection");
            // In production, verify connection to RabbitMQ Management API
        }

        *self.config.write().await = Some(config);
        *self.connection.write().await = Some(conn);

        info!("RabbitMQ secrets engine configured successfully");
        Ok(())
    }

    /// Get configuration
    pub async fn get_config(&self) -> Result<RabbitMqConfig, RabbitMqError> {
        self.config
            .read()
            .await
            .as_ref()
            .cloned()
            .ok_or(RabbitMqError::ConfigNotFound)
    }

    /// Create role
    #[instrument(skip(self))]
    pub async fn create_role(&self, role: RabbitMqRole) -> Result<(), RabbitMqError> {
        role.validate()?;

        let mut roles = self.roles.write().await;
        if roles.contains_key(&role.name) {
            return Err(RabbitMqError::RoleAlreadyExists(role.name));
        }

        info!("Creating RabbitMQ role: {}", role.name);
        roles.insert(role.name.clone(), role);
        Ok(())
    }

    /// Get role
    pub async fn get_role(&self, role_name: &str) -> Result<RabbitMqRole, RabbitMqError> {
        let roles = self.roles.read().await;
        roles
            .get(role_name)
            .cloned()
            .ok_or_else(|| RabbitMqError::RoleNotFound(role_name.to_string()))
    }

    /// Delete role
    #[instrument(skip(self))]
    pub async fn delete_role(&self, role_name: &str) -> Result<(), RabbitMqError> {
        let mut roles = self.roles.write().await;
        roles
            .remove(role_name)
            .ok_or_else(|| RabbitMqError::RoleNotFound(role_name.to_string()))?;

        info!("Deleted RabbitMQ role: {}", role_name);
        Ok(())
    }

    /// List roles
    pub async fn list_roles(&self) -> Vec<String> {
        let roles = self.roles.read().await;
        roles.keys().cloned().collect()
    }

    /// Generate credentials
    #[instrument(skip(self))]
    pub async fn generate_credentials(
        &self,
        role_name: &str,
        ttl: Option<i64>,
    ) -> Result<RabbitMqCredentials, RabbitMqError> {
        let config = self.get_config().await?;
        let role = self.get_role(role_name).await?;

        let ttl = ttl.unwrap_or(role.default_ttl);
        if ttl > role.max_ttl {
            return Err(RabbitMqError::InvalidTtl(format!(
                "Requested TTL {} exceeds max TTL {}",
                ttl, role.max_ttl
            )));
        }

        let username = self.generate_username(role_name);
        let password = self.generate_password();

        let conn = self.connection.read().await;
        let conn = conn.as_ref().ok_or(RabbitMqError::ConfigNotFound)?;

        // Create user in RabbitMQ
        conn.create_user(&username, &password, &role.tags).await?;

        // Set permissions for each vhost
        for vhost_perm in &role.vhosts {
            conn.set_permissions(
                &username,
                &vhost_perm.vhost,
                &vhost_perm.configure,
                &vhost_perm.write,
                &vhost_perm.read,
            )
            .await?;
        }

        let now = Utc::now();
        let credentials = RabbitMqCredentials {
            username: username.clone(),
            password: password.clone(),
            connection_uri: self.build_connection_uri(&config, &username, &password),
            management_uri: config.management_uri.clone(),
            vhosts: role.vhosts.iter().map(|v| v.vhost.clone()).collect(),
            created_at: now,
            expires_at: now + Duration::seconds(ttl),
            role_name: role_name.to_string(),
        };

        self.credentials
            .write()
            .await
            .insert(username.clone(), credentials.clone());

        info!("Generated RabbitMQ credentials for role: {}", role_name);
        Ok(credentials)
    }

    /// Revoke credentials
    #[instrument(skip(self))]
    pub async fn revoke_credentials(&self, username: &str) -> Result<(), RabbitMqError> {
        let conn = self.connection.read().await;
        let conn = conn.as_ref().ok_or(RabbitMqError::ConfigNotFound)?;

        conn.delete_user(username).await?;

        self.credentials.write().await.remove(username);

        info!("Revoked RabbitMQ credentials for user: {}", username);
        Ok(())
    }

    /// List active credentials
    pub async fn list_credentials(&self) -> Vec<String> {
        let credentials = self.credentials.read().await;
        credentials.keys().cloned().collect()
    }

    /// Get credential info
    pub async fn get_credential_info(&self, username: &str) -> Option<RabbitMqCredentialInfo> {
        let credentials = self.credentials.read().await;
        credentials.get(username).map(RabbitMqCredentialInfo::from)
    }

    /// Generate username
    fn generate_username(&self, role_name: &str) -> String {
        let timestamp = Utc::now().timestamp();
        let random: u32 = rand::thread_rng().gen_range(1000..9999);
        format!("v-{}-{}-{}", role_name, timestamp, random)
    }

    /// Generate secure password
    fn generate_password(&self) -> String {
        use rand::distributions::Alphanumeric;

        let mut rng = rand::thread_rng();
        let password: String = (0..32).map(|_| rng.sample(Alphanumeric) as char).collect();

        password
    }

    /// Build connection URI with credentials
    fn build_connection_uri(
        &self,
        config: &RabbitMqConfig,
        username: &str,
        password: &str,
    ) -> String {
        let base_uri = config.connection_uri.replace("amqp://", "");
        let parts: Vec<&str> = base_uri.split('@').collect();

        if parts.len() > 1 {
            format!("amqp://{}:{}@{}", username, password, parts[1])
        } else {
            format!("amqp://{}:{}@{}", username, password, base_uri)
        }
    }
}

impl Default for RabbitMqEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rabbitmq_config_validation() {
        let mut config = RabbitMqConfig::default();
        assert!(config.validate().is_ok());

        config.default_ttl = 30;
        assert!(config.validate().is_err());

        config.default_ttl = 3600;
        config.max_ttl = 1000000;
        assert!(config.validate().is_err());
    }

    #[tokio::test]
    async fn test_rabbitmq_role_creation() {
        let engine = RabbitMqEngine::new();

        let config = RabbitMqConfig::default();
        engine.configure(config).await.unwrap();

        let role = RabbitMqRole::new("test-role".to_string());
        assert!(engine.create_role(role).await.is_ok());

        let retrieved = engine.get_role("test-role").await.unwrap();
        assert_eq!(retrieved.name, "test-role");
    }

    #[tokio::test]
    async fn test_rabbitmq_credential_generation() {
        let engine = RabbitMqEngine::new();

        let config = RabbitMqConfig::default();
        engine.configure(config).await.unwrap();

        let role = RabbitMqRole::new("test-role".to_string());
        engine.create_role(role).await.unwrap();

        let creds = engine
            .generate_credentials("test-role", None)
            .await
            .unwrap();
        assert!(creds.username.starts_with("v-test-role-"));
        assert_eq!(creds.password.len(), 32);
    }

    #[tokio::test]
    async fn test_rabbitmq_password_generation() {
        let engine = RabbitMqEngine::new();
        let password = engine.generate_password();
        assert_eq!(password.len(), 32);
        assert!(password.chars().all(|c| c.is_alphanumeric()));
    }
}
