//! Kafka Secrets Engine
//!
//! Dynamic Kafka ACL and SCRAM credentials generation.

use chrono::{DateTime, Duration, Utc};
use deadpool_postgres::Pool;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument};

/// Error types for Kafka secrets engine
#[derive(Debug, thiserror::Error)]
pub enum KafkaError {
    #[error("Kafka configuration not found")]
    ConfigNotFound,

    #[error("Kafka role not found: {0}")]
    RoleNotFound(String),

    #[error("Kafka role already exists: {0}")]
    RoleAlreadyExists(String),

    #[error("Invalid Kafka configuration: {0}")]
    InvalidConfig(String),

    #[error("Invalid TTL: {0}")]
    InvalidTtl(String),

    #[error("Kafka connection error: {0}")]
    ConnectionError(String),

    #[error("Kafka Admin API error: {0}")]
    ApiError(String),

    #[error("Credential generation failed: {0}")]
    CredentialGenerationFailed(String),

    #[error("Credential revocation failed: {0}")]
    RevocationFailed(String),

    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("ACL operation failed: {0}")]
    AclError(String),
}

/// Kafka ACL operation type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum KafkaAclOperation {
    Read,
    Write,
    Create,
    Delete,
    Alter,
    Describe,
    ClusterAction,
    DescribeConfigs,
    AlterConfigs,
    IdempotentWrite,
    All,
}

impl KafkaAclOperation {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Read => "READ",
            Self::Write => "WRITE",
            Self::Create => "CREATE",
            Self::Delete => "DELETE",
            Self::Alter => "ALTER",
            Self::Describe => "DESCRIBE",
            Self::ClusterAction => "CLUSTER_ACTION",
            Self::DescribeConfigs => "DESCRIBE_CONFIGS",
            Self::AlterConfigs => "ALTER_CONFIGS",
            Self::IdempotentWrite => "IDEMPOTENT_WRITE",
            Self::All => "ALL",
        }
    }
}

/// Kafka resource type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum KafkaResourceType {
    Topic,
    Group,
    Cluster,
    TransactionalId,
    DelegationToken,
}

impl KafkaResourceType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Topic => "TOPIC",
            Self::Group => "GROUP",
            Self::Cluster => "CLUSTER",
            Self::TransactionalId => "TRANSACTIONAL_ID",
            Self::DelegationToken => "DELEGATION_TOKEN",
        }
    }
}

/// Kafka ACL permission type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum KafkaPermissionType {
    Allow,
    Deny,
}

impl KafkaPermissionType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Allow => "ALLOW",
            Self::Deny => "DENY",
        }
    }
}

/// Kafka ACL entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KafkaAcl {
    pub resource_type: KafkaResourceType,
    pub resource_name: String,
    pub resource_pattern: String, // LITERAL, PREFIXED, MATCH
    pub principal: String,
    pub host: String,
    pub operation: KafkaAclOperation,
    pub permission_type: KafkaPermissionType,
}

impl Default for KafkaAcl {
    fn default() -> Self {
        Self {
            resource_type: KafkaResourceType::Topic,
            resource_name: "*".to_string(),
            resource_pattern: "LITERAL".to_string(),
            principal: "User:*".to_string(),
            host: "*".to_string(),
            operation: KafkaAclOperation::All,
            permission_type: KafkaPermissionType::Allow,
        }
    }
}

/// Kafka SCRAM mechanism
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum KafkaScramMechanism {
    ScramSha256,
    ScramSha512,
}

impl KafkaScramMechanism {
    pub fn as_str(&self) -> &str {
        match self {
            Self::ScramSha256 => "SCRAM-SHA-256",
            Self::ScramSha512 => "SCRAM-SHA-512",
        }
    }
}

/// Kafka configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KafkaConfig {
    pub bootstrap_servers: String,
    pub admin_username: String,
    #[serde(skip_serializing)]
    pub admin_password: String,
    pub scram_mechanism: KafkaScramMechanism,
    pub use_tls: bool,
    pub verify_connection: bool,
    pub default_ttl: i64,
    pub max_ttl: i64,
}

impl Default for KafkaConfig {
    fn default() -> Self {
        Self {
            bootstrap_servers: "localhost:9092".to_string(),
            admin_username: "admin".to_string(),
            admin_password: "admin-secret".to_string(),
            scram_mechanism: KafkaScramMechanism::ScramSha256,
            use_tls: false,
            verify_connection: true,
            default_ttl: 3600,
            max_ttl: 86400,
        }
    }
}

impl KafkaConfig {
    pub fn validate(&self) -> Result<(), KafkaError> {
        if self.bootstrap_servers.is_empty() {
            return Err(KafkaError::InvalidConfig(
                "Bootstrap servers are required".to_string(),
            ));
        }
        if self.admin_username.is_empty() {
            return Err(KafkaError::InvalidConfig(
                "Admin username is required".to_string(),
            ));
        }
        if self.admin_password.is_empty() {
            return Err(KafkaError::InvalidConfig(
                "Admin password is required".to_string(),
            ));
        }
        if self.default_ttl < 60 {
            return Err(KafkaError::InvalidTtl(
                "Default TTL must be at least 60 seconds".to_string(),
            ));
        }
        if self.max_ttl > 604800 {
            return Err(KafkaError::InvalidTtl(
                "Max TTL cannot exceed 7 days".to_string(),
            ));
        }
        if self.default_ttl > self.max_ttl {
            return Err(KafkaError::InvalidTtl(
                "Default TTL cannot exceed max TTL".to_string(),
            ));
        }
        Ok(())
    }
}

/// Kafka role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KafkaRole {
    pub name: String,
    pub acls: Vec<KafkaAcl>,
    pub scram_mechanism: KafkaScramMechanism,
    pub default_ttl: i64,
    pub max_ttl: i64,
    pub created_at: DateTime<Utc>,
}

impl KafkaRole {
    pub fn new(name: String) -> Self {
        Self {
            name,
            acls: vec![],
            scram_mechanism: KafkaScramMechanism::ScramSha256,
            default_ttl: 3600,
            max_ttl: 86400,
            created_at: Utc::now(),
        }
    }

    pub fn validate(&self) -> Result<(), KafkaError> {
        if self.name.is_empty() {
            return Err(KafkaError::InvalidConfig(
                "Role name is required".to_string(),
            ));
        }
        if self.acls.is_empty() {
            return Err(KafkaError::InvalidConfig(
                "At least one ACL is required".to_string(),
            ));
        }
        if self.default_ttl < 60 {
            return Err(KafkaError::InvalidTtl(
                "Default TTL must be at least 60 seconds".to_string(),
            ));
        }
        if self.max_ttl > 604800 {
            return Err(KafkaError::InvalidTtl(
                "Max TTL cannot exceed 7 days".to_string(),
            ));
        }
        if self.default_ttl > self.max_ttl {
            return Err(KafkaError::InvalidTtl(
                "Default TTL cannot exceed max TTL".to_string(),
            ));
        }
        Ok(())
    }
}

/// Kafka credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KafkaCredentials {
    pub username: String,
    pub password: String,
    pub bootstrap_servers: String,
    pub scram_mechanism: String,
    pub acls: Vec<KafkaAcl>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub role_name: String,
}

/// Kafka credential info (without password)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KafkaCredentialInfo {
    pub username: String,
    pub bootstrap_servers: String,
    pub scram_mechanism: String,
    pub acls: Vec<KafkaAcl>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub role_name: String,
}

impl From<&KafkaCredentials> for KafkaCredentialInfo {
    fn from(cred: &KafkaCredentials) -> Self {
        Self {
            username: cred.username.clone(),
            bootstrap_servers: cred.bootstrap_servers.clone(),
            scram_mechanism: cred.scram_mechanism.clone(),
            acls: cred.acls.clone(),
            created_at: cred.created_at,
            expires_at: cred.expires_at,
            role_name: cred.role_name.clone(),
        }
    }
}

/// Simulated Kafka Admin client for prototyping
#[derive(Debug, Clone)]
struct KafkaAdminClient {
    bootstrap_servers: String,
    admin_username: String,
    admin_password: String,
}

impl KafkaAdminClient {
    fn new(config: &KafkaConfig) -> Self {
        Self {
            bootstrap_servers: config.bootstrap_servers.clone(),
            admin_username: config.admin_username.clone(),
            admin_password: config.admin_password.clone(),
        }
    }

    /// Simulate creating SCRAM credentials
    async fn create_scram_user(
        &self,
        username: &str,
        password: &str,
        mechanism: &KafkaScramMechanism,
    ) -> Result<(), KafkaError> {
        info!(
            "Simulated: Creating Kafka SCRAM user '{}' with mechanism {}",
            username,
            mechanism.as_str()
        );
        // In production, this would use kafka-admin-client to create SCRAM credentials
        Ok(())
    }

    /// Simulate creating ACLs
    async fn create_acls(&self, username: &str, acls: &[KafkaAcl]) -> Result<(), KafkaError> {
        info!(
            "Simulated: Creating {} ACLs for user '{}'",
            acls.len(),
            username
        );
        for acl in acls {
            debug!(
                "  ACL: {} {} on {} {}",
                acl.permission_type.as_str(),
                acl.operation.as_str(),
                acl.resource_type.as_str(),
                acl.resource_name
            );
        }
        // In production, this would use kafka-admin-client to create ACLs
        Ok(())
    }

    /// Simulate deleting SCRAM user
    async fn delete_scram_user(&self, username: &str) -> Result<(), KafkaError> {
        info!("Simulated: Deleting Kafka SCRAM user '{}'", username);
        // In production, this would use kafka-admin-client to delete SCRAM credentials
        Ok(())
    }

    /// Simulate deleting ACLs
    async fn delete_acls(&self, username: &str) -> Result<(), KafkaError> {
        info!("Simulated: Deleting ACLs for user '{}'", username);
        // In production, this would use kafka-admin-client to delete ACLs
        Ok(())
    }

    /// Simulate checking if user exists
    async fn user_exists(&self, username: &str) -> Result<bool, KafkaError> {
        debug!("Simulated: Checking if Kafka user '{}' exists", username);
        // In production, this would query Kafka for SCRAM credentials
        Ok(false)
    }
}

/// Kafka Secrets Engine
pub struct KafkaEngine {
    config: Arc<RwLock<Option<KafkaConfig>>>,
    roles: Arc<RwLock<HashMap<String, KafkaRole>>>,
    credentials: Arc<RwLock<HashMap<String, KafkaCredentials>>>,
    admin_client: Arc<RwLock<Option<KafkaAdminClient>>>,
    pool: Option<Pool>,
}

impl KafkaEngine {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(None)),
            roles: Arc::new(RwLock::new(HashMap::new())),
            credentials: Arc::new(RwLock::new(HashMap::new())),
            admin_client: Arc::new(RwLock::new(None)),
            pool: None,
        }
    }

    pub fn with_storage(pool: Pool) -> Self {
        Self {
            config: Arc::new(RwLock::new(None)),
            roles: Arc::new(RwLock::new(HashMap::new())),
            credentials: Arc::new(RwLock::new(HashMap::new())),
            admin_client: Arc::new(RwLock::new(None)),
            pool: Some(pool),
        }
    }

    /// Configure Kafka connection
    #[instrument(skip(self, config))]
    pub async fn configure(&self, config: KafkaConfig) -> Result<(), KafkaError> {
        config.validate()?;

        let admin_client = KafkaAdminClient::new(&config);

        if config.verify_connection {
            info!("Verifying Kafka connection");
            // In production, verify connection to Kafka cluster
        }

        *self.config.write().await = Some(config);
        *self.admin_client.write().await = Some(admin_client);

        info!("Kafka secrets engine configured successfully");
        Ok(())
    }

    /// Get configuration
    pub async fn get_config(&self) -> Result<KafkaConfig, KafkaError> {
        self.config
            .read()
            .await
            .as_ref()
            .cloned()
            .ok_or(KafkaError::ConfigNotFound)
    }

    /// Create role
    #[instrument(skip(self))]
    pub async fn create_role(&self, role: KafkaRole) -> Result<(), KafkaError> {
        role.validate()?;

        let mut roles = self.roles.write().await;
        if roles.contains_key(&role.name) {
            return Err(KafkaError::RoleAlreadyExists(role.name));
        }

        info!("Creating Kafka role: {}", role.name);
        roles.insert(role.name.clone(), role);
        Ok(())
    }

    /// Get role
    pub async fn get_role(&self, role_name: &str) -> Result<KafkaRole, KafkaError> {
        let roles = self.roles.read().await;
        roles
            .get(role_name)
            .cloned()
            .ok_or_else(|| KafkaError::RoleNotFound(role_name.to_string()))
    }

    /// Delete role
    #[instrument(skip(self))]
    pub async fn delete_role(&self, role_name: &str) -> Result<(), KafkaError> {
        let mut roles = self.roles.write().await;
        roles
            .remove(role_name)
            .ok_or_else(|| KafkaError::RoleNotFound(role_name.to_string()))?;

        info!("Deleted Kafka role: {}", role_name);
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
    ) -> Result<KafkaCredentials, KafkaError> {
        let config = self.get_config().await?;
        let role = self.get_role(role_name).await?;

        let ttl = ttl.unwrap_or(role.default_ttl);
        if ttl > role.max_ttl {
            return Err(KafkaError::InvalidTtl(format!(
                "Requested TTL {} exceeds max TTL {}",
                ttl, role.max_ttl
            )));
        }

        let username = self.generate_username(role_name);
        let password = self.generate_password();

        let admin_client = self.admin_client.read().await;
        let admin_client = admin_client.as_ref().ok_or(KafkaError::ConfigNotFound)?;

        // Create SCRAM user in Kafka
        admin_client
            .create_scram_user(&username, &password, &role.scram_mechanism)
            .await?;

        // Create ACLs with the username as principal
        let mut acls_with_principal = role.acls.clone();
        for acl in &mut acls_with_principal {
            acl.principal = format!("User:{}", username);
        }

        admin_client
            .create_acls(&username, &acls_with_principal)
            .await?;

        let now = Utc::now();
        let credentials = KafkaCredentials {
            username: username.clone(),
            password: password.clone(),
            bootstrap_servers: config.bootstrap_servers.clone(),
            scram_mechanism: role.scram_mechanism.as_str().to_string(),
            acls: acls_with_principal,
            created_at: now,
            expires_at: now + Duration::seconds(ttl),
            role_name: role_name.to_string(),
        };

        self.credentials
            .write()
            .await
            .insert(username.clone(), credentials.clone());

        info!("Generated Kafka credentials for role: {}", role_name);
        Ok(credentials)
    }

    /// Revoke credentials
    #[instrument(skip(self))]
    pub async fn revoke_credentials(&self, username: &str) -> Result<(), KafkaError> {
        let admin_client = self.admin_client.read().await;
        let admin_client = admin_client.as_ref().ok_or(KafkaError::ConfigNotFound)?;

        // Delete ACLs first
        admin_client.delete_acls(username).await?;

        // Delete SCRAM user
        admin_client.delete_scram_user(username).await?;

        self.credentials.write().await.remove(username);

        info!("Revoked Kafka credentials for user: {}", username);
        Ok(())
    }

    /// List active credentials
    pub async fn list_credentials(&self) -> Vec<String> {
        let credentials = self.credentials.read().await;
        credentials.keys().cloned().collect()
    }

    /// Get credential info
    pub async fn get_credential_info(&self, username: &str) -> Option<KafkaCredentialInfo> {
        let credentials = self.credentials.read().await;
        credentials.get(username).map(KafkaCredentialInfo::from)
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
}

impl Default for KafkaEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_kafka_config_validation() {
        let mut config = KafkaConfig::default();
        assert!(config.validate().is_ok());

        config.default_ttl = 30;
        assert!(config.validate().is_err());

        config.default_ttl = 3600;
        config.max_ttl = 1000000;
        assert!(config.validate().is_err());
    }

    #[tokio::test]
    async fn test_kafka_role_creation() {
        let engine = KafkaEngine::new();

        let config = KafkaConfig::default();
        engine.configure(config).await.unwrap();

        let mut role = KafkaRole::new("test-role".to_string());
        role.acls.push(KafkaAcl::default());
        assert!(engine.create_role(role).await.is_ok());

        let retrieved = engine.get_role("test-role").await.unwrap();
        assert_eq!(retrieved.name, "test-role");
    }

    #[tokio::test]
    async fn test_kafka_credential_generation() {
        let engine = KafkaEngine::new();

        let config = KafkaConfig::default();
        engine.configure(config).await.unwrap();

        let mut role = KafkaRole::new("test-role".to_string());
        role.acls.push(KafkaAcl::default());
        engine.create_role(role).await.unwrap();

        let creds = engine
            .generate_credentials("test-role", None)
            .await
            .unwrap();
        assert!(creds.username.starts_with("v-test-role-"));
        assert_eq!(creds.password.len(), 32);
    }

    #[tokio::test]
    async fn test_kafka_password_generation() {
        let engine = KafkaEngine::new();
        let password = engine.generate_password();
        assert_eq!(password.len(), 32);
        assert!(password.chars().all(|c| c.is_alphanumeric()));
    }

    #[tokio::test]
    async fn test_kafka_acl_operations() {
        assert_eq!(KafkaAclOperation::Read.as_str(), "READ");
        assert_eq!(KafkaAclOperation::Write.as_str(), "WRITE");
        assert_eq!(KafkaResourceType::Topic.as_str(), "TOPIC");
        assert_eq!(KafkaResourceType::Group.as_str(), "GROUP");
    }
}
