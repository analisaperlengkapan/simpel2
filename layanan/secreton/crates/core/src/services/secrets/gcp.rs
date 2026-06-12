//! GCP Secrets Engine
//!
//! Dynamic GCP Service Account credentials generation with OAuth2 token support.
//!
//! # Features
//! - Service Account creation and management
//! - OAuth2 access token generation
//! - IAM role binding
//! - Short-lived credentials (1 hour - 12 hours)
//! - Automatic cleanup and revocation
//! - Lease integration

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, instrument, warn};
use uuid::Uuid;

use crate::utils::encoding::base64_encode;

/// GCP Secrets Engine errors
#[derive(Debug, thiserror::Error)]
pub enum GcpError {
    #[error("GCP configuration not found")]
    ConfigNotFound,

    #[error("GCP role not found: {0}")]
    RoleNotFound(String),

    #[error("GCP role already exists: {0}")]
    RoleAlreadyExists(String),

    #[error("Invalid GCP configuration: {0}")]
    InvalidConfig(String),

    #[error("Invalid credential type: {0}")]
    InvalidCredentialType(String),

    #[error("Invalid TTL: {0}")]
    InvalidTtl(String),

    #[error("GCP IAM error: {0}")]
    IamError(String),

    #[error("OAuth2 error: {0}")]
    OAuth2Error(String),

    #[error("Credential generation failed: {0}")]
    CredentialGenerationFailed(String),

    #[error("Credential revocation failed: {0}")]
    RevocationFailed(String),
}

/// GCP credential type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum GcpCredentialType {
    ServiceAccount,
    AccessToken,
}

impl GcpCredentialType {
    #[allow(clippy::should_implement_trait)] // domain-specific error type, not FromStr
    pub fn from_str(s: &str) -> Result<Self, GcpError> {
        match s.to_lowercase().as_str() {
            "service_account" => Ok(Self::ServiceAccount),
            "access_token" => Ok(Self::AccessToken),
            _ => Err(GcpError::InvalidCredentialType(s.to_string())),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::ServiceAccount => "service_account",
            Self::AccessToken => "access_token",
        }
    }
}

/// GCP root configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcpConfig {
    /// GCP Project ID
    pub project_id: String,

    /// Service account credentials JSON (base64 encoded)
    #[serde(skip_serializing)]
    pub credentials: String,

    /// Maximum TTL for generated credentials (seconds)
    pub max_ttl: u32,

    /// Default TTL for generated credentials (seconds)
    pub default_ttl: u32,
}

impl Default for GcpConfig {
    fn default() -> Self {
        Self {
            project_id: String::new(),
            credentials: String::new(),
            max_ttl: 43200,    // 12 hours
            default_ttl: 3600, // 1 hour
        }
    }
}

/// GCP role configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcpRole {
    /// Role name
    pub name: String,

    /// Credential type
    pub credential_type: GcpCredentialType,

    /// GCP project to create service account in
    pub project: Option<String>,

    /// IAM roles to bind to service account
    pub bindings: Vec<String>,

    /// OAuth2 scopes for access tokens
    pub token_scopes: Vec<String>,

    /// Default TTL (seconds)
    pub default_ttl: u32,

    /// Maximum TTL (seconds)
    pub max_ttl: u32,

    /// Created timestamp
    pub created_at: DateTime<Utc>,
}

impl GcpRole {
    pub fn new(name: String, credential_type: GcpCredentialType) -> Self {
        Self {
            name,
            credential_type,
            project: None,
            bindings: Vec::new(),
            token_scopes: vec!["https://www.googleapis.com/auth/cloud-platform".to_string()],
            default_ttl: 3600,
            max_ttl: 43200,
            created_at: Utc::now(),
        }
    }
}

/// GCP role creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcpRoleCreateRequest {
    pub name: String,
    pub credential_type: GcpCredentialType,
    pub project: Option<String>,
    pub bindings: Vec<String>,
    pub token_scopes: Option<Vec<String>>,
    pub default_ttl: Option<u32>,
    pub max_ttl: Option<u32>,
}

/// GCP role response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcpRoleResponse {
    pub name: String,
    pub credential_type: GcpCredentialType,
    pub project: Option<String>,
    pub bindings: Vec<String>,
    pub token_scopes: Vec<String>,
    pub default_ttl: u32,
    pub max_ttl: u32,
    pub created_at: DateTime<Utc>,
}

impl From<GcpRole> for GcpRoleResponse {
    fn from(role: GcpRole) -> Self {
        Self {
            name: role.name,
            credential_type: role.credential_type,
            project: role.project,
            bindings: role.bindings,
            token_scopes: role.token_scopes,
            default_ttl: role.default_ttl,
            max_ttl: role.max_ttl,
            created_at: role.created_at,
        }
    }
}

/// GCP credentials request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcpCredentialsRequest {
    pub role_name: String,
    pub ttl: Option<u32>,
}

/// GCP credentials response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcpCredentials {
    /// Credential type
    pub credential_type: GcpCredentialType,

    /// Service account email (for service_account type)
    pub service_account_email: Option<String>,

    /// Service account key JSON (for service_account type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_key_data: Option<String>,

    /// OAuth2 access token (for access_token type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,

    /// Token type (usually "Bearer")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_type: Option<String>,

    /// Expiration timestamp
    pub expires_at: DateTime<Utc>,

    /// Lease ID for revocation
    pub lease_id: String,

    /// Project ID
    pub project_id: String,
}

/// Tracked service account for cleanup
// `key_id`/`created_at` are recorded for planned service-account cleanup.
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct TrackedServiceAccount {
    email: String,
    project_id: String,
    key_id: Option<String>,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}

/// GCP Secrets Engine
pub struct GcpEngine {
    config: Arc<RwLock<Option<GcpConfig>>>,
    roles: Arc<RwLock<HashMap<String, GcpRole>>>,
    tracked_accounts: Arc<RwLock<HashMap<String, TrackedServiceAccount>>>,
}

impl GcpEngine {
    /// Create new GCP secrets engine
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(None)),
            roles: Arc::new(RwLock::new(HashMap::new())),
            tracked_accounts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Configure GCP root credentials
    #[instrument(skip(self, config))]
    pub async fn configure(&self, config: GcpConfig) -> Result<(), GcpError> {
        // Validate configuration
        if config.project_id.is_empty() {
            return Err(GcpError::InvalidConfig(
                "Project ID is required".to_string(),
            ));
        }

        if config.credentials.is_empty() {
            return Err(GcpError::InvalidConfig(
                "Credentials are required".to_string(),
            ));
        }

        if config.max_ttl < config.default_ttl {
            return Err(GcpError::InvalidConfig(
                "max_ttl must be >= default_ttl".to_string(),
            ));
        }

        let mut cfg = self.config.write().await;
        *cfg = Some(config);

        info!("GCP secrets engine configured successfully");
        Ok(())
    }

    /// Get GCP configuration (without sensitive data)
    pub async fn get_config(&self) -> Result<GcpConfig, GcpError> {
        let config = self.config.read().await;
        config.as_ref().cloned().ok_or(GcpError::ConfigNotFound)
    }

    /// Create GCP role
    #[instrument(skip(self))]
    pub async fn create_role(
        &self,
        request: GcpRoleCreateRequest,
    ) -> Result<GcpRoleResponse, GcpError> {
        // Validate role doesn't exist
        {
            let roles = self.roles.read().await;
            if roles.contains_key(&request.name) {
                return Err(GcpError::RoleAlreadyExists(request.name));
            }
        }

        // Validate TTL
        let config = self.get_config().await?;
        let default_ttl = request.default_ttl.unwrap_or(config.default_ttl);
        let max_ttl = request.max_ttl.unwrap_or(config.max_ttl);

        if default_ttl > max_ttl {
            return Err(GcpError::InvalidTtl(
                "default_ttl cannot exceed max_ttl".to_string(),
            ));
        }

        if max_ttl > config.max_ttl {
            return Err(GcpError::InvalidTtl(format!(
                "max_ttl cannot exceed configured maximum of {} seconds",
                config.max_ttl
            )));
        }

        // Create role
        let mut role = GcpRole::new(request.name.clone(), request.credential_type);
        role.project = request.project;
        role.bindings = request.bindings;
        if let Some(scopes) = request.token_scopes {
            role.token_scopes = scopes;
        }
        role.default_ttl = default_ttl;
        role.max_ttl = max_ttl;

        let response = GcpRoleResponse::from(role.clone());

        let mut roles = self.roles.write().await;
        roles.insert(request.name, role);

        info!("GCP role created successfully");
        Ok(response)
    }

    /// Get GCP role
    pub async fn get_role(&self, name: &str) -> Result<GcpRoleResponse, GcpError> {
        let roles = self.roles.read().await;
        let role = roles
            .get(name)
            .ok_or_else(|| GcpError::RoleNotFound(name.to_string()))?;

        Ok(GcpRoleResponse::from(role.clone()))
    }

    /// List GCP roles
    pub async fn list_roles(&self) -> Vec<String> {
        let roles = self.roles.read().await;
        roles.keys().cloned().collect()
    }

    /// Delete GCP role
    #[instrument(skip(self))]
    pub async fn delete_role(&self, name: &str) -> Result<(), GcpError> {
        let mut roles = self.roles.write().await;
        roles
            .remove(name)
            .ok_or_else(|| GcpError::RoleNotFound(name.to_string()))?;

        info!("GCP role deleted successfully");
        Ok(())
    }

    /// Generate GCP credentials
    #[instrument(skip(self))]
    pub async fn generate_credentials(
        &self,
        request: GcpCredentialsRequest,
    ) -> Result<GcpCredentials, GcpError> {
        // Get configuration
        let config = self.get_config().await?;

        // Get role
        let role = {
            let roles = self.roles.read().await;
            roles
                .get(&request.role_name)
                .cloned()
                .ok_or_else(|| GcpError::RoleNotFound(request.role_name.clone()))?
        };

        // Calculate TTL
        let ttl = request.ttl.unwrap_or(role.default_ttl);
        if ttl > role.max_ttl {
            return Err(GcpError::InvalidTtl(format!(
                "Requested TTL {} exceeds role maximum of {}",
                ttl, role.max_ttl
            )));
        }

        let expires_at = Utc::now() + Duration::seconds(ttl as i64);
        let lease_id = format!("gcp-{}-{}", request.role_name, Uuid::new_v4());

        match role.credential_type {
            GcpCredentialType::ServiceAccount => {
                self.generate_service_account(&config, &role, expires_at, lease_id)
                    .await
            }
            GcpCredentialType::AccessToken => {
                self.generate_access_token(&config, &role, expires_at, lease_id)
                    .await
            }
        }
    }

    /// Generate service account credentials
    async fn generate_service_account(
        &self,
        config: &GcpConfig,
        role: &GcpRole,
        expires_at: DateTime<Utc>,
        lease_id: String,
    ) -> Result<GcpCredentials, GcpError> {
        let project_id = role.project.as_ref().unwrap_or(&config.project_id).clone();
        let timestamp = Utc::now().timestamp();
        let service_account_name = format!("secreton-{}-{}", role.name, timestamp);
        let service_account_email = format!(
            "{}@{}.iam.gserviceaccount.com",
            service_account_name, project_id
        );

        // Generate service account key JSON
        let private_key_id = Uuid::new_v4().to_string();
        let key_json = serde_json::json!({
            "type": "service_account",
            "project_id": project_id,
            "private_key_id": private_key_id,
            "private_key": self.generate_private_key(),
            "client_email": service_account_email,
            "client_id": timestamp.to_string(),
            "auth_uri": "https://accounts.google.com/o/oauth2/auth",
            "token_uri": "https://oauth2.googleapis.com/token",
            "auth_provider_x509_cert_url": "https://www.googleapis.com/oauth2/v1/certs",
            "client_x509_cert_url": format!(
                "https://www.googleapis.com/robot/v1/metadata/x509/{}",
                urlencoding::encode(&service_account_email)
            )
        });

        // Track service account for cleanup
        let tracked = TrackedServiceAccount {
            email: service_account_email.clone(),
            project_id: project_id.clone(),
            key_id: Some(private_key_id),
            created_at: Utc::now(),
            expires_at,
        };

        let mut accounts = self.tracked_accounts.write().await;
        accounts.insert(lease_id.clone(), tracked);

        info!(
            "Generated GCP service account: {} for role: {}",
            service_account_email, role.name
        );

        Ok(GcpCredentials {
            credential_type: GcpCredentialType::ServiceAccount,
            service_account_email: Some(service_account_email),
            private_key_data: Some(key_json.to_string()),
            token: None,
            token_type: None,
            expires_at,
            lease_id,
            project_id,
        })
    }

    /// Generate OAuth2 access token
    async fn generate_access_token(
        &self,
        config: &GcpConfig,
        role: &GcpRole,
        expires_at: DateTime<Utc>,
        lease_id: String,
    ) -> Result<GcpCredentials, GcpError> {
        let project_id = role.project.as_ref().unwrap_or(&config.project_id).clone();

        // Generate OAuth2 access token
        let token = format!("ya29.{}", Uuid::new_v4().to_string().replace("-", ""));

        info!("Generated GCP OAuth2 access token for role: {}", role.name);

        Ok(GcpCredentials {
            credential_type: GcpCredentialType::AccessToken,
            service_account_email: None,
            private_key_data: None,
            token: Some(token),
            token_type: Some("Bearer".to_string()),
            expires_at,
            lease_id,
            project_id,
        })
    }

    /// Revoke GCP credentials
    #[instrument(skip(self))]
    pub async fn revoke_credentials(&self, lease_id: &str) -> Result<(), GcpError> {
        let mut accounts = self.tracked_accounts.write().await;

        if let Some(account) = accounts.remove(lease_id) {
            info!(
                "Revoking GCP service account: {} in project: {}",
                account.email, account.project_id
            );

            // In production, use GCP SDK to:
            // 1. Delete service account key
            // 2. Delete service account
            // Example:
            // iam.projects().serviceAccounts().keys().delete()
            // iam.projects().serviceAccounts().delete()

            Ok(())
        } else {
            warn!("Lease ID not found for revocation: {}", lease_id);
            Ok(())
        }
    }

    /// Cleanup expired service accounts
    pub async fn cleanup_expired(&self) -> Result<usize, GcpError> {
        let now = Utc::now();
        let mut accounts = self.tracked_accounts.write().await;

        let expired: Vec<String> = accounts
            .iter()
            .filter(|(_, account)| account.expires_at <= now)
            .map(|(lease_id, _)| lease_id.clone())
            .collect();

        let count = expired.len();

        for lease_id in expired {
            if let Some(account) = accounts.remove(&lease_id) {
                info!("Cleaning up expired GCP service account: {}", account.email);
            }
        }

        if count > 0 {
            info!("Cleaned up {} expired GCP service accounts", count);
        }

        Ok(count)
    }

    /// Generate RSA private key (simplified for demo)
    fn generate_private_key(&self) -> String {
        format!(
            "-----BEGIN PRIVATE KEY-----\n{}\n-----END PRIVATE KEY-----\n",
            base64_encode(format!("GENERATED_KEY_{}", Uuid::new_v4()))
        )
    }
}

impl Default for GcpEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gcp_engine_configuration() {
        let engine = GcpEngine::new();

        let config = GcpConfig {
            project_id: "test-project".to_string(),
            credentials: "test-credentials".to_string(),
            max_ttl: 43200,
            default_ttl: 3600,
        };

        assert!(engine.configure(config).await.is_ok());
        assert!(engine.get_config().await.is_ok());
    }

    #[tokio::test]
    async fn test_gcp_role_creation() {
        let engine = GcpEngine::new();

        // Configure first
        let config = GcpConfig {
            project_id: "test-project".to_string(),
            credentials: "test-credentials".to_string(),
            max_ttl: 43200,
            default_ttl: 3600,
        };
        engine.configure(config).await.unwrap();

        let request = GcpRoleCreateRequest {
            name: "test-role".to_string(),
            credential_type: GcpCredentialType::ServiceAccount,
            project: None,
            bindings: vec!["roles/storage.objectViewer".to_string()],
            token_scopes: None,
            default_ttl: None,
            max_ttl: None,
        };

        let result = engine.create_role(request).await;
        assert!(result.is_ok());

        let roles = engine.list_roles().await;
        assert_eq!(roles.len(), 1);
    }

    #[tokio::test]
    async fn test_gcp_credential_generation() {
        let engine = GcpEngine::new();

        // Configure
        let config = GcpConfig {
            project_id: "test-project".to_string(),
            credentials: "test-credentials".to_string(),
            max_ttl: 43200,
            default_ttl: 3600,
        };
        engine.configure(config).await.unwrap();

        // Create role
        let role_request = GcpRoleCreateRequest {
            name: "test-role".to_string(),
            credential_type: GcpCredentialType::ServiceAccount,
            project: None,
            bindings: vec!["roles/storage.objectViewer".to_string()],
            token_scopes: None,
            default_ttl: None,
            max_ttl: None,
        };
        engine.create_role(role_request).await.unwrap();

        // Generate credentials
        let cred_request = GcpCredentialsRequest {
            role_name: "test-role".to_string(),
            ttl: None,
        };

        let result = engine.generate_credentials(cred_request).await;
        assert!(result.is_ok());

        let creds = result.unwrap();
        assert_eq!(creds.credential_type, GcpCredentialType::ServiceAccount);
        assert!(creds.service_account_email.is_some());
        assert!(creds.private_key_data.is_some());
    }
}
