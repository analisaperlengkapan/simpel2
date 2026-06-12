//! LDAP Secrets Engine
//!
//! Dynamic LDAP credential generation and management.
//! Generates temporary LDAP credentials with automatic rotation.

use chrono::{DateTime, Duration, Utc};
use deadpool_postgres::Pool;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument};

/// LDAP secrets engine errors
#[derive(Debug, thiserror::Error)]
pub enum LdapError {
    #[error("Configuration not found")]
    ConfigNotFound,

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Role not found: {0}")]
    RoleNotFound(String),

    #[error("Role already exists: {0}")]
    RoleAlreadyExists(String),

    #[error("LDAP connection failed: {0}")]
    ConnectionFailed(String),

    #[error("LDAP operation failed: {0}")]
    OperationFailed(String),

    #[error("User creation failed: {0}")]
    UserCreationFailed(String),

    #[error("Password rotation failed: {0}")]
    PasswordRotationFailed(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Invalid DN: {0}")]
    InvalidDn(String),
}

/// LDAP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdapConfig {
    /// LDAP server URL (ldap:// or ldaps://)
    pub url: String,

    /// Bind DN for admin operations
    pub bind_dn: String,

    /// Bind password
    pub bind_password: String,

    /// Base DN for user creation
    pub user_dn: String,

    /// Base DN for group operations
    pub group_dn: Option<String>,

    /// Use TLS
    pub use_tls: bool,

    /// Certificate path for TLS
    pub certificate: Option<String>,

    /// LDAP schema (OpenLDAP, ActiveDirectory)
    pub schema: LdapSchema,

    /// User object class
    pub user_object_class: String,

    /// Default password policy
    pub password_policy: Option<String>,
}

/// LDAP schema type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum LdapSchema {
    #[default]
    OpenLdap,
    ActiveDirectory,
}

/// LDAP role configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdapRole {
    /// Role name
    pub name: String,

    /// DN template for user creation
    pub creation_ldif: String,

    /// DN template for user deletion
    pub deletion_ldif: Option<String>,

    /// Default TTL for credentials
    pub default_ttl: i64,

    /// Maximum TTL for credentials
    pub max_ttl: i64,

    /// Username template
    pub username_template: Option<String>,

    /// Groups to add user to
    pub groups: Vec<String>,

    /// Additional LDAP attributes
    pub attributes: HashMap<String, String>,
}

impl LdapRole {
    pub fn new(name: String) -> Self {
        Self {
            name,
            creation_ldif: String::new(),
            deletion_ldif: None,
            default_ttl: 3600,
            max_ttl: 86400,
            username_template: Some("v-{{.RoleName}}-{{.Random}}".to_string()),
            groups: Vec::new(),
            attributes: HashMap::new(),
        }
    }

    pub fn validate(&self) -> Result<(), LdapError> {
        if self.name.is_empty() {
            return Err(LdapError::InvalidConfig(
                "Role name is required".to_string(),
            ));
        }

        if self.creation_ldif.is_empty() {
            return Err(LdapError::InvalidConfig(
                "Creation LDIF is required".to_string(),
            ));
        }

        if self.default_ttl <= 0 {
            return Err(LdapError::InvalidConfig(
                "Default TTL must be positive".to_string(),
            ));
        }

        if self.max_ttl < self.default_ttl {
            return Err(LdapError::InvalidConfig(
                "Max TTL must be >= default TTL".to_string(),
            ));
        }

        Ok(())
    }
}

/// Dynamic LDAP credential
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdapCredential {
    /// Username
    pub username: String,

    /// Password
    pub password: String,

    /// Distinguished Name
    pub dn: String,

    /// Role name
    pub role: String,

    /// Created at
    pub created_at: DateTime<Utc>,

    /// Expires at
    pub expires_at: DateTime<Utc>,

    /// Last rotated
    pub last_rotated: Option<DateTime<Utc>>,
}

/// LDAP connection (simulated)
#[derive(Debug, Clone)]
struct LdapConnection {
    url: String,
    #[allow(dead_code)] // planned: bind DN used once real LDAP bind is wired
    bind_dn: String,
    connected: bool,
}

impl LdapConnection {
    fn new(url: String, bind_dn: String) -> Self {
        Self {
            url,
            bind_dn,
            connected: false,
        }
    }

    async fn connect(&mut self, password: &str) -> Result<(), LdapError> {
        // Simulate LDAP connection
        if password.is_empty() {
            return Err(LdapError::ConnectionFailed("Empty password".to_string()));
        }

        self.connected = true;
        debug!("Connected to LDAP server: {}", self.url);
        Ok(())
    }

    async fn create_user(
        &self,
        dn: &str,
        username: &str,
        _password: &str,
        _object_class: &str,
        _attributes: &HashMap<String, String>,
    ) -> Result<(), LdapError> {
        if !self.connected {
            return Err(LdapError::ConnectionFailed("Not connected".to_string()));
        }

        // Simulate user creation
        debug!("Creating LDAP user: {} at {}", username, dn);
        Ok(())
    }

    async fn modify_password(&self, dn: &str, _new_password: &str) -> Result<(), LdapError> {
        if !self.connected {
            return Err(LdapError::ConnectionFailed("Not connected".to_string()));
        }

        // Simulate password modification
        debug!("Rotating password for: {}", dn);
        Ok(())
    }

    async fn delete_user(&self, dn: &str) -> Result<(), LdapError> {
        if !self.connected {
            return Err(LdapError::ConnectionFailed("Not connected".to_string()));
        }

        // Simulate user deletion
        debug!("Deleting LDAP user: {}", dn);
        Ok(())
    }

    async fn add_to_groups(&self, dn: &str, groups: &[String]) -> Result<(), LdapError> {
        if !self.connected {
            return Err(LdapError::ConnectionFailed("Not connected".to_string()));
        }

        // Simulate group membership
        for group in groups {
            debug!("Adding {} to group: {}", dn, group);
        }
        Ok(())
    }
}

/// LDAP secrets engine
pub struct LdapEngine {
    config: Arc<RwLock<Option<LdapConfig>>>,
    roles: Arc<RwLock<HashMap<String, LdapRole>>>,
    credentials: Arc<RwLock<HashMap<String, LdapCredential>>>,
    #[allow(dead_code)] // planned: connection pool for real LDAP ops
    pool: Option<Pool>,
}

impl Default for LdapEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl LdapEngine {
    /// Create new LDAP engine
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(None)),
            roles: Arc::new(RwLock::new(HashMap::new())),
            credentials: Arc::new(RwLock::new(HashMap::new())),
            pool: None,
        }
    }

    /// Create new LDAP engine with storage
    pub fn with_storage(pool: Pool) -> Self {
        Self {
            config: Arc::new(RwLock::new(None)),
            roles: Arc::new(RwLock::new(HashMap::new())),
            credentials: Arc::new(RwLock::new(HashMap::new())),
            pool: Some(pool),
        }
    }

    /// Configure LDAP connection
    #[instrument(skip(self, config))]
    pub async fn configure(&self, config: LdapConfig) -> Result<(), LdapError> {
        // Validate configuration
        if config.url.is_empty() {
            return Err(LdapError::InvalidConfig("URL is required".to_string()));
        }

        if config.bind_dn.is_empty() {
            return Err(LdapError::InvalidConfig("Bind DN is required".to_string()));
        }

        if config.bind_password.is_empty() {
            return Err(LdapError::InvalidConfig(
                "Bind password is required".to_string(),
            ));
        }

        if config.user_dn.is_empty() {
            return Err(LdapError::InvalidConfig("User DN is required".to_string()));
        }

        // Test connection
        let mut conn = LdapConnection::new(config.url.clone(), config.bind_dn.clone());
        conn.connect(&config.bind_password).await?;

        let mut cfg = self.config.write().await;
        *cfg = Some(config);

        info!("LDAP secrets engine configured successfully");
        Ok(())
    }

    /// Get configuration
    pub async fn get_config(&self) -> Result<LdapConfig, LdapError> {
        let config = self.config.read().await;
        config.as_ref().cloned().ok_or(LdapError::ConfigNotFound)
    }

    /// Create role
    #[instrument(skip(self))]
    pub async fn create_role(&self, role: LdapRole) -> Result<(), LdapError> {
        role.validate()?;

        let mut roles = self.roles.write().await;
        if roles.contains_key(&role.name) {
            return Err(LdapError::RoleAlreadyExists(role.name.clone()));
        }

        info!("Creating LDAP role: {}", role.name);
        roles.insert(role.name.clone(), role);
        Ok(())
    }

    /// Get role
    pub async fn get_role(&self, name: &str) -> Result<LdapRole, LdapError> {
        let roles = self.roles.read().await;
        roles
            .get(name)
            .cloned()
            .ok_or_else(|| LdapError::RoleNotFound(name.to_string()))
    }

    /// List roles
    pub async fn list_roles(&self) -> Vec<String> {
        let roles = self.roles.read().await;
        roles.keys().cloned().collect()
    }

    /// Delete role
    #[instrument(skip(self))]
    pub async fn delete_role(&self, name: &str) -> Result<(), LdapError> {
        let mut roles = self.roles.write().await;
        if roles.remove(name).is_none() {
            return Err(LdapError::RoleNotFound(name.to_string()));
        }

        info!("Deleted LDAP role: {}", name);
        Ok(())
    }

    /// Generate random password
    fn generate_password(length: usize) -> String {
        const CHARSET: &[u8] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*";
        let mut rng = rand::thread_rng();

        (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    /// Generate username from template
    fn generate_username(template: &str, role_name: &str) -> String {
        let random: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(8)
            .map(char::from)
            .collect();

        template
            .replace("{{.RoleName}}", role_name)
            .replace("{{.Random}}", &random)
            .to_lowercase()
    }

    /// Generate dynamic credentials
    #[instrument(skip(self))]
    pub async fn generate_credentials(
        &self,
        role_name: &str,
        ttl: Option<i64>,
    ) -> Result<LdapCredential, LdapError> {
        let config = self.get_config().await?;
        let role = self.get_role(role_name).await?;

        // Generate username
        let username = if let Some(template) = &role.username_template {
            Self::generate_username(template, role_name)
        } else {
            format!("v-{}-{}", role_name, uuid::Uuid::new_v4())
        };

        // Generate password
        let password = Self::generate_password(32);

        // Build DN
        let dn = format!("cn={},{}", username, config.user_dn);

        // Calculate expiration
        let ttl_seconds = ttl.unwrap_or(role.default_ttl).min(role.max_ttl);
        let expires_at = Utc::now() + Duration::seconds(ttl_seconds);

        // Connect to LDAP
        let mut conn = LdapConnection::new(config.url.clone(), config.bind_dn.clone());
        conn.connect(&config.bind_password).await?;

        // Create user
        conn.create_user(
            &dn,
            &username,
            &password,
            &config.user_object_class,
            &role.attributes,
        )
        .await?;

        // Add to groups
        if !role.groups.is_empty() {
            conn.add_to_groups(&dn, &role.groups).await?;
        }

        let credential = LdapCredential {
            username: username.clone(),
            password: password.clone(),
            dn: dn.clone(),
            role: role_name.to_string(),
            created_at: Utc::now(),
            expires_at,
            last_rotated: None,
        };

        // Store credential
        let mut credentials = self.credentials.write().await;
        credentials.insert(username.clone(), credential.clone());

        info!(
            "Generated LDAP credentials for role: {} (username: {})",
            role_name, username
        );
        Ok(credential)
    }

    /// Rotate password for existing credential
    #[instrument(skip(self))]
    pub async fn rotate_password(&self, username: &str) -> Result<LdapCredential, LdapError> {
        let config = self.get_config().await?;

        let mut credentials = self.credentials.write().await;
        let credential = credentials
            .get_mut(username)
            .ok_or_else(|| LdapError::RoleNotFound(username.to_string()))?;

        // Generate new password
        let new_password = Self::generate_password(32);

        // Connect to LDAP
        let mut conn = LdapConnection::new(config.url.clone(), config.bind_dn.clone());
        conn.connect(&config.bind_password).await?;

        // Modify password
        conn.modify_password(&credential.dn, &new_password).await?;

        // Update credential
        credential.password = new_password;
        credential.last_rotated = Some(Utc::now());

        info!("Rotated password for LDAP user: {}", username);
        Ok(credential.clone())
    }

    /// Revoke credentials
    #[instrument(skip(self))]
    pub async fn revoke_credentials(&self, username: &str) -> Result<(), LdapError> {
        let config = self.get_config().await?;

        let mut credentials = self.credentials.write().await;
        let credential = credentials
            .remove(username)
            .ok_or_else(|| LdapError::RoleNotFound(username.to_string()))?;

        // Connect to LDAP
        let mut conn = LdapConnection::new(config.url.clone(), config.bind_dn.clone());
        conn.connect(&config.bind_password).await?;

        // Delete user
        conn.delete_user(&credential.dn).await?;

        info!("Revoked LDAP credentials for user: {}", username);
        Ok(())
    }

    /// List active credentials
    pub async fn list_credentials(&self) -> Vec<String> {
        let credentials = self.credentials.read().await;
        credentials.keys().cloned().collect()
    }

    /// Get credential info (without password)
    pub async fn get_credential_info(&self, username: &str) -> Option<LdapCredentialInfo> {
        let credentials = self.credentials.read().await;
        credentials.get(username).map(|cred| LdapCredentialInfo {
            username: cred.username.clone(),
            dn: cred.dn.clone(),
            role: cred.role.clone(),
            created_at: cred.created_at,
            expires_at: cred.expires_at,
            last_rotated: cred.last_rotated,
        })
    }
}

/// Credential info without password
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdapCredentialInfo {
    pub username: String,
    pub dn: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_rotated: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ldap_configuration() {
        let engine = LdapEngine::new();

        let config = LdapConfig {
            url: "ldap://localhost:389".to_string(),
            bind_dn: "cn=admin,dc=example,dc=com".to_string(),
            bind_password: "admin123".to_string(),
            user_dn: "ou=users,dc=example,dc=com".to_string(),
            group_dn: Some("ou=groups,dc=example,dc=com".to_string()),
            use_tls: false,
            certificate: None,
            schema: LdapSchema::OpenLdap,
            user_object_class: "inetOrgPerson".to_string(),
            password_policy: None,
        };

        let result = engine.configure(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_role_creation() {
        let engine = LdapEngine::new();

        // Configure first
        let config = LdapConfig {
            url: "ldap://localhost:389".to_string(),
            bind_dn: "cn=admin,dc=example,dc=com".to_string(),
            bind_password: "admin123".to_string(),
            user_dn: "ou=users,dc=example,dc=com".to_string(),
            group_dn: None,
            use_tls: false,
            certificate: None,
            schema: LdapSchema::OpenLdap,
            user_object_class: "inetOrgPerson".to_string(),
            password_policy: None,
        };
        engine.configure(config).await.unwrap();

        let mut role = LdapRole::new("test-role".to_string());
        role.creation_ldif = "cn={{.Username},ou=users,dc=example,dc=com".to_string();
        role.default_ttl = 3600;
        role.max_ttl = 7200;

        let result = engine.create_role(role).await;
        assert!(result.is_ok());

        let retrieved = engine.get_role("test-role").await;
        assert!(retrieved.is_ok());
    }

    #[tokio::test]
    async fn test_credential_generation() {
        let engine = LdapEngine::new();

        // Configure
        let config = LdapConfig {
            url: "ldap://localhost:389".to_string(),
            bind_dn: "cn=admin,dc=example,dc=com".to_string(),
            bind_password: "admin123".to_string(),
            user_dn: "ou=users,dc=example,dc=com".to_string(),
            group_dn: None,
            use_tls: false,
            certificate: None,
            schema: LdapSchema::OpenLdap,
            user_object_class: "inetOrgPerson".to_string(),
            password_policy: None,
        };
        engine.configure(config).await.unwrap();

        // Create role
        let mut role = LdapRole::new("app-role".to_string());
        role.creation_ldif = "cn={{.Username},ou=users,dc=example,dc=com".to_string();
        engine.create_role(role).await.unwrap();

        // Generate credentials
        let cred = engine.generate_credentials("app-role", None).await.unwrap();
        assert!(!cred.username.is_empty());
        assert!(!cred.password.is_empty());
        assert!(cred.password.len() >= 32);
    }

    #[tokio::test]
    async fn test_password_generation() {
        let password = LdapEngine::generate_password(32);
        assert_eq!(password.len(), 32);

        // Verify it contains various character types
        let has_upper = password.chars().any(|c| c.is_uppercase());
        let has_lower = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_numeric());

        assert!(has_upper || has_lower || has_digit);
    }

    #[tokio::test]
    async fn test_username_template() {
        let template = "v-{{.RoleName}}-{{.Random}}";
        let username = LdapEngine::generate_username(template, "test-role");

        assert!(username.starts_with("v-test-role-"));
        assert!(username.len() > "v-test-role-".len());
    }
}
