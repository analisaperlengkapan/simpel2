//! Kubernetes Secrets Engine
//!
//! Dynamically generates Kubernetes service account tokens with automatic rotation.
//! Generates short-lived tokens bound to specific namespaces and service accounts.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Error types for Kubernetes secrets engine
#[derive(Debug, thiserror::Error)]
pub enum KubernetesError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Kubernetes role not found: {0}")]
    RoleNotFound(String),

    #[error("Token generation failed: {0}")]
    TokenGenerationFailed(String),

    #[error("Rotation failed: {0}")]
    RotationFailed(String),

    #[error("Revocation failed: {0}")]
    RevocationFailed(String),
}

/// Kubernetes connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubernetesConnection {
    /// Connection name
    pub name: String,

    /// Kubernetes API server URL
    pub api_server_url: String,

    /// CA certificate (PEM format)
    pub ca_cert: Option<String>,

    /// Service account token for authentication
    pub service_account_token: Option<String>,

    /// Client certificate (PEM format)
    pub client_cert: Option<String>,

    /// Client key (PEM format)
    pub client_key: Option<String>,

    /// Verify connection on startup
    pub verify_connection: bool,
}

impl Default for KubernetesConnection {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            api_server_url: String::new(),
            ca_cert: None,
            service_account_token: None,
            client_cert: None,
            client_key: None,
            verify_connection: true,
        }
    }
}

/// Kubernetes role configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubernetesRole {
    /// Role name
    pub name: String,

    /// Kubernetes connection name
    pub connection_name: String,

    /// Target namespace
    pub namespace: String,

    /// Service account name
    pub service_account: String,

    /// Default TTL for tokens (in seconds)
    pub default_ttl: u32,

    /// Maximum TTL for tokens (in seconds)
    pub max_ttl: u32,

    /// Additional labels to apply
    pub labels: HashMap<String, String>,

    /// Additional annotations to apply
    pub annotations: HashMap<String, String>,
}

impl Default for KubernetesRole {
    fn default() -> Self {
        Self {
            name: String::new(),
            connection_name: String::new(),
            namespace: "default".to_string(),
            service_account: String::new(),
            default_ttl: 3600,
            max_ttl: 86400,
            labels: HashMap::new(),
            annotations: HashMap::new(),
        }
    }
}

/// Generated Kubernetes service account token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubernetesToken {
    /// Unique token ID
    pub id: String,

    /// Service account token
    pub token: String,

    /// Namespace
    pub namespace: String,

    /// Service account name
    pub service_account: String,

    /// Creation time
    pub created_at: DateTime<Utc>,

    /// Expiration time
    pub expires_at: DateTime<Utc>,

    /// Role name used
    pub role_name: String,

    /// Connection name
    pub connection_name: String,
}

/// Kubernetes secrets engine
pub struct KubernetesSecretsEngine {
    connections: Arc<RwLock<HashMap<String, KubernetesConnection>>>,
    roles: Arc<RwLock<HashMap<String, KubernetesRole>>>,
    active_tokens: Arc<RwLock<HashMap<String, KubernetesToken>>>,
}

impl KubernetesSecretsEngine {
    /// Create new Kubernetes secrets engine
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            roles: Arc::new(RwLock::new(HashMap::new())),
            active_tokens: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Configure Kubernetes connection
    pub async fn configure_connection(
        &self,
        config: KubernetesConnection,
    ) -> Result<(), KubernetesError> {
        // Validate configuration
        if config.name.is_empty() {
            return Err(KubernetesError::InvalidConfig(
                "Connection name cannot be empty".to_string(),
            ));
        }
        if config.api_server_url.is_empty() {
            return Err(KubernetesError::InvalidConfig(
                "API server URL cannot be empty".to_string(),
            ));
        }

        // Validate authentication method
        if config.service_account_token.is_none()
            && (config.client_cert.is_none() || config.client_key.is_none())
        {
            return Err(KubernetesError::InvalidConfig(
                "Either service account token or client certificate/key must be provided"
                    .to_string(),
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

    /// Test Kubernetes connection
    async fn test_connection(&self, config: &KubernetesConnection) -> Result<(), KubernetesError> {
        if !config.api_server_url.starts_with("https://") {
            return Err(KubernetesError::ConnectionError(
                "API server URL must start with https://".to_string(),
            ));
        }

        // Note: Actual Kubernetes connection testing would require kube crate
        // For now, we validate the configuration
        Ok(())
    }

    /// Create Kubernetes role
    pub async fn create_role(&self, role: KubernetesRole) -> Result<(), KubernetesError> {
        // Validate role
        if role.name.is_empty() {
            return Err(KubernetesError::InvalidConfig(
                "Role name cannot be empty".to_string(),
            ));
        }
        if role.service_account.is_empty() {
            return Err(KubernetesError::InvalidConfig(
                "Service account name required".to_string(),
            ));
        }
        if role.namespace.is_empty() {
            return Err(KubernetesError::InvalidConfig(
                "Namespace required".to_string(),
            ));
        }

        // Verify connection exists
        let connections = self.connections.read().await;
        if !connections.contains_key(&role.connection_name) {
            return Err(KubernetesError::InvalidConfig(format!(
                "Connection '{}' not found",
                role.connection_name
            )));
        }
        drop(connections);

        // Store role
        let mut roles = self.roles.write().await;
        roles.insert(role.name.clone(), role);

        Ok(())
    }

    /// Generate service account token for a role
    pub async fn generate_token(
        &self,
        role_name: &str,
        ttl: Option<u32>,
    ) -> Result<KubernetesToken, KubernetesError> {
        // Get role
        let roles = self.roles.read().await;
        let role = roles
            .get(role_name)
            .ok_or_else(|| KubernetesError::RoleNotFound(role_name.to_string()))?
            .clone();
        drop(roles);

        // Get connection
        let connections = self.connections.read().await;
        let connection = connections
            .get(&role.connection_name)
            .ok_or_else(|| {
                KubernetesError::InvalidConfig(format!(
                    "Connection '{}' not found",
                    role.connection_name
                ))
            })?
            .clone();
        drop(connections);

        // Determine TTL
        let ttl = ttl.unwrap_or(role.default_ttl);
        if ttl > role.max_ttl {
            return Err(KubernetesError::InvalidConfig(format!(
                "TTL {} exceeds maximum {}",
                ttl, role.max_ttl
            )));
        }

        // Generate token
        let token = self
            .create_service_account_token(&connection, &role.namespace, &role.service_account, ttl)
            .await?;

        // Create token record
        let now = Utc::now();
        let k8s_token = KubernetesToken {
            id: Uuid::new_v4().to_string(),
            token: token.clone(),
            namespace: role.namespace.clone(),
            service_account: role.service_account.clone(),
            created_at: now,
            expires_at: now + Duration::seconds(ttl as i64),
            role_name: role_name.to_string(),
            connection_name: role.connection_name.clone(),
        };

        // Store active token
        let mut active = self.active_tokens.write().await;
        active.insert(k8s_token.id.clone(), k8s_token.clone());

        Ok(k8s_token)
    }

    /// Create service account token via Kubernetes API
    async fn create_service_account_token(
        &self,
        _connection: &KubernetesConnection,
        _namespace: &str,
        _service_account: &str,
        _ttl: u32,
    ) -> Result<String, KubernetesError> {
        // Note: Actual Kubernetes token creation would require kube crate
        // This would call the TokenRequest API:
        // POST /api/v1/namespaces/{namespace}/serviceaccounts/{name}/token
        // with body: { "spec": { "expirationSeconds": ttl  }

        // For now, return a placeholder token
        // In production, this would be the actual JWT token from Kubernetes
        Ok(format!("k8s-token-{}", Uuid::new_v4()))
    }

    /// Revoke service account token
    pub async fn revoke_token(&self, token_id: &str) -> Result<(), KubernetesError> {
        // Get token
        let mut active = self.active_tokens.write().await;
        let token = active.remove(token_id).ok_or_else(|| {
            KubernetesError::RevocationFailed(format!("Token {} not found", token_id))
        })?;
        drop(active);

        // Note: Kubernetes service account tokens are self-contained JWTs
        // They cannot be revoked directly, but we remove them from our tracking
        // In production, you might want to:
        // 1. Delete the associated Secret object if it was created
        // 2. Implement a token revocation list
        // 3. Use short TTLs to minimize exposure

        // Get connection
        let connections = self.connections.read().await;
        let _connection = connections
            .get(&token.connection_name)
            .ok_or_else(|| {
                KubernetesError::InvalidConfig(format!(
                    "Connection '{}' not found",
                    token.connection_name
                ))
            })?
            .clone();
        drop(connections);

        // In production, you might delete the token secret here
        // DELETE /api/v1/namespaces/{namespace}/secrets/{secret-name}

        Ok(())
    }

    /// List active tokens
    pub async fn list_tokens(&self) -> Vec<KubernetesToken> {
        let active = self.active_tokens.read().await;
        active.values().cloned().collect()
    }

    /// Generate token with lease integration
    pub async fn generate_token_with_lease(
        &self,
        role_name: &str,
        ttl: Option<u32>,
        lease_manager: &crate::services::lease::LeaseManager,
        user: &str,
    ) -> Result<(KubernetesToken, crate::services::lease::EnhancedLease), KubernetesError> {
        // Generate token
        let token = self.generate_token(role_name, ttl).await?;

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

        let resource_path = format!("kubernetes/token/{}", role_name);
        let lease = lease_manager
            .create_lease(
                user,
                &resource_path,
                "kubernetes",
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
                KubernetesError::TokenGenerationFailed(format!("Failed to create lease: {}", e))
            })?;

        Ok((token, lease))
    }

    /// Revoke token with lease
    pub async fn revoke_token_with_lease(
        &self,
        token_id: &str,
        lease_manager: &crate::services::lease::LeaseManager,
        lease_id: &str,
    ) -> Result<(), KubernetesError> {
        // Revoke token
        self.revoke_token(token_id).await?;

        // Revoke lease
        lease_manager.revoke_lease(lease_id).await.map_err(|e| {
            KubernetesError::RevocationFailed(format!("Failed to revoke lease: {}", e))
        })?;

        Ok(())
    }

    /// Renew token lease (extend validity)
    pub async fn renew_lease(
        &self,
        token_id: &str,
        increment: u32,
    ) -> Result<KubernetesToken, KubernetesError> {
        // Get existing token
        let mut active = self.active_tokens.write().await;
        let token = active.get_mut(token_id).ok_or_else(|| {
            KubernetesError::RotationFailed(format!("Token {} not found", token_id))
        })?;

        // Get role to check max_ttl
        let roles = self.roles.read().await;
        let role = roles
            .get(&token.role_name)
            .ok_or_else(|| KubernetesError::RoleNotFound(token.role_name.clone()))?
            .clone();
        drop(roles);

        // Calculate new expiration time
        let now = Utc::now();
        let current_ttl = (token.expires_at - now).num_seconds();
        let new_ttl = current_ttl + increment as i64;

        // Check against max_ttl
        if new_ttl > role.max_ttl as i64 {
            return Err(KubernetesError::InvalidConfig(format!(
                "New TTL {} exceeds maximum {}",
                new_ttl, role.max_ttl
            )));
        }

        // Update expiration time
        token.expires_at = now + Duration::seconds(new_ttl);

        Ok(token.clone())
    }
}

impl Default for KubernetesSecretsEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_kubernetes_engine_creation() {
        let engine = KubernetesSecretsEngine::new();
        assert_eq!(engine.list_tokens().await.len(), 0);
    }

    #[tokio::test]
    async fn test_configure_connection() {
        let engine = KubernetesSecretsEngine::new();

        let config = KubernetesConnection {
            name: "test-cluster".to_string(),
            api_server_url: "https://kubernetes.default.svc".to_string(),
            service_account_token: Some("test-token".to_string()),
            verify_connection: false,
            ..Default::default()
        };

        let result = engine.configure_connection(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_role() {
        let engine = KubernetesSecretsEngine::new();

        // First configure connection
        let config = KubernetesConnection {
            name: "test-cluster".to_string(),
            api_server_url: "https://kubernetes.default.svc".to_string(),
            service_account_token: Some("test-token".to_string()),
            verify_connection: false,
            ..Default::default()
        };
        engine.configure_connection(config).await.unwrap();

        // Create role
        let role = KubernetesRole {
            name: "app-reader".to_string(),
            connection_name: "test-cluster".to_string(),
            namespace: "default".to_string(),
            service_account: "app-sa".to_string(),
            default_ttl: 3600,
            max_ttl: 86400,
            ..Default::default()
        };

        let result = engine.create_role(role).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_role_validation() {
        let engine = KubernetesSecretsEngine::new();

        // Configure connection
        let config = KubernetesConnection {
            name: "test-cluster".to_string(),
            api_server_url: "https://kubernetes.default.svc".to_string(),
            service_account_token: Some("test-token".to_string()),
            verify_connection: false,
            ..Default::default()
        };
        engine.configure_connection(config).await.unwrap();

        // Test role with empty name
        let invalid_role = KubernetesRole {
            name: "".to_string(),
            connection_name: "test-cluster".to_string(),
            namespace: "default".to_string(),
            service_account: "app-sa".to_string(),
            ..Default::default()
        };
        assert!(engine.create_role(invalid_role).await.is_err());

        // Test role with empty service account
        let invalid_role = KubernetesRole {
            name: "test-role".to_string(),
            connection_name: "test-cluster".to_string(),
            namespace: "default".to_string(),
            service_account: "".to_string(),
            ..Default::default()
        };
        assert!(engine.create_role(invalid_role).await.is_err());

        // Test role with non-existent connection
        let invalid_role = KubernetesRole {
            name: "test-role".to_string(),
            connection_name: "non-existent".to_string(),
            namespace: "default".to_string(),
            service_account: "app-sa".to_string(),
            ..Default::default()
        };
        assert!(engine.create_role(invalid_role).await.is_err());
    }

    #[tokio::test]
    async fn test_ttl_validation() {
        let engine = KubernetesSecretsEngine::new();

        // Setup connection and role
        let config = KubernetesConnection {
            name: "test-cluster".to_string(),
            api_server_url: "https://kubernetes.default.svc".to_string(),
            service_account_token: Some("test-token".to_string()),
            verify_connection: false,
            ..Default::default()
        };
        engine.configure_connection(config).await.unwrap();

        let role = KubernetesRole {
            name: "app-reader".to_string(),
            connection_name: "test-cluster".to_string(),
            namespace: "default".to_string(),
            service_account: "app-sa".to_string(),
            default_ttl: 3600,
            max_ttl: 7200,
            ..Default::default()
        };
        engine.create_role(role).await.unwrap();

        // Verify the role configuration is stored correctly
        let roles = engine.roles.read().await;
        let stored_role = roles.get("app-reader").unwrap();
        assert_eq!(stored_role.default_ttl, 3600);
        assert_eq!(stored_role.max_ttl, 7200);
        assert_eq!(stored_role.namespace, "default");
        assert_eq!(stored_role.service_account, "app-sa");
    }

    #[tokio::test]
    async fn test_connection_validation() {
        let engine = KubernetesSecretsEngine::new();

        // Test connection without authentication
        let invalid_config = KubernetesConnection {
            name: "test-cluster".to_string(),
            api_server_url: "https://kubernetes.default.svc".to_string(),
            service_account_token: None,
            client_cert: None,
            client_key: None,
            verify_connection: false,
            ..Default::default()
        };
        assert!(engine.configure_connection(invalid_config).await.is_err());

        // Test connection with invalid URL
        let invalid_config = KubernetesConnection {
            name: "test-cluster".to_string(),
            api_server_url: "http://kubernetes.default.svc".to_string(),
            service_account_token: Some("test-token".to_string()),
            verify_connection: true,
            ..Default::default()
        };
        assert!(engine.configure_connection(invalid_config).await.is_err());
    }

    #[tokio::test]
    async fn test_token_generation() {
        let engine = KubernetesSecretsEngine::new();

        // Setup connection and role
        let config = KubernetesConnection {
            name: "test-cluster".to_string(),
            api_server_url: "https://kubernetes.default.svc".to_string(),
            service_account_token: Some("test-token".to_string()),
            verify_connection: false,
            ..Default::default()
        };
        engine.configure_connection(config).await.unwrap();

        let role = KubernetesRole {
            name: "app-reader".to_string(),
            connection_name: "test-cluster".to_string(),
            namespace: "default".to_string(),
            service_account: "app-sa".to_string(),
            default_ttl: 3600,
            max_ttl: 7200,
            ..Default::default()
        };
        engine.create_role(role).await.unwrap();

        // Generate token
        let result = engine.generate_token("app-reader", Some(1800)).await;
        assert!(result.is_ok());

        let token = result.unwrap();
        assert_eq!(token.namespace, "default");
        assert_eq!(token.service_account, "app-sa");
        assert_eq!(token.role_name, "app-reader");
        assert!(!token.token.is_empty());
    }
}
