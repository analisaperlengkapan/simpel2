//! AWS Secrets Engine
//!
//! Dynamic AWS IAM credentials generation with STS AssumeRole support.

use aws_config::{BehaviorVersion, Region, SdkConfig};
use aws_credential_types::Credentials;
use aws_sdk_iam::{Client as IamClient, types::Tag};
use aws_sdk_sts::{Client as StsClient, types::PolicyDescriptorType};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Error types for AWS secrets engine
#[derive(Debug, thiserror::Error)]
pub enum AwsError {
    #[error("AWS configuration not found")]
    ConfigNotFound,

    #[error("AWS role not found: {0}")]
    RoleNotFound(String),

    #[error("AWS role already exists: {0}")]
    RoleAlreadyExists(String),

    #[error("Invalid AWS configuration: {0}")]
    InvalidConfig(String),

    #[error("Invalid credential type: {0}")]
    InvalidCredentialType(String),

    #[error("Invalid TTL: {0}")]
    InvalidTtl(String),

    #[error("AWS IAM error: {0}")]
    IamError(String),

    #[error("AWS STS error: {0}")]
    StsError(String),

    #[error("Credential generation failed: {0}")]
    CredentialGenerationFailed(String),

    #[error("Credential revocation failed: {0}")]
    RevocationFailed(String),
}

/// AWS credential type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AwsCredentialType {
    IamUser,
    AssumeRole,
    FederationToken,
}

impl AwsCredentialType {
    pub fn from_str(s: &str) -> Result<Self, AwsError> {
        match s.to_lowercase().as_str() {
            "iam_user" => Ok(Self::IamUser),
            "assume_role" => Ok(Self::AssumeRole),
            "federation_token" => Ok(Self::FederationToken),
            _ => Err(AwsError::InvalidCredentialType(s.to_string())),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::IamUser => "iam_user",
            Self::AssumeRole => "assume_role",
            Self::FederationToken => "federation_token",
        }
    }
}

/// AWS root configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsConfig {
    pub access_key: String,
    #[serde(skip_serializing)]
    pub secret_key: String,
    pub region: String,
    pub sts_endpoint: Option<String>,
    pub iam_endpoint: Option<String>,
    pub max_ttl: u32,
    pub default_ttl: u32,
}

impl Default for AwsConfig {
    fn default() -> Self {
        Self {
            access_key: String::new(),
            secret_key: String::new(),
            region: "us-east-1".to_string(),
            sts_endpoint: None,
            iam_endpoint: None,
            max_ttl: 43200,
            default_ttl: 3600,
        }
    }
}

/// AWS role configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsRole {
    pub name: String,
    pub credential_type: AwsCredentialType,
    pub policy_arns: Vec<String>,
    pub policy_document: Option<String>,
    pub role_arns: Vec<String>,
    pub default_ttl: u32,
    pub max_ttl: u32,
    pub user_path: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl AwsRole {
    pub fn new(name: String, credential_type: AwsCredentialType) -> Self {
        Self {
            name,
            credential_type,
            policy_arns: Vec::new(),
            policy_document: None,
            role_arns: Vec::new(),
            default_ttl: 3600,
            max_ttl: 43200,
            user_path: Some("/secreton/".to_string()),
            created_at: Utc::now(),
        }
    }

    pub fn validate(&self) -> Result<(), AwsError> {
        if self.default_ttl < 900 {
            return Err(AwsError::InvalidTtl(
                "Default TTL must be at least 15 minutes".to_string(),
            ));
        }
        if self.max_ttl > 43200 {
            return Err(AwsError::InvalidTtl(
                "Max TTL cannot exceed 12 hours".to_string(),
            ));
        }
        if self.default_ttl > self.max_ttl {
            return Err(AwsError::InvalidTtl(
                "Default TTL cannot exceed max TTL".to_string(),
            ));
        }

        match self.credential_type {
            AwsCredentialType::AssumeRole => {
                if self.role_arns.is_empty() {
                    return Err(AwsError::InvalidConfig(
                        "AssumeRole requires at least one role ARN".to_string(),
                    ));
                }
            }
            AwsCredentialType::IamUser | AwsCredentialType::FederationToken => {
                if self.policy_arns.is_empty() && self.policy_document.is_none() {
                    return Err(AwsError::InvalidConfig(
                        "IAM user/federation token requires policies".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }
}

/// AWS credentials response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsCredentials {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: Option<String>,
    pub credential_type: String,
    pub expiration: DateTime<Utc>,
    pub lease_id: String,
    pub user_name: Option<String>,
}

/// AWS role creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsRoleCreateRequest {
    pub name: String,
    pub credential_type: AwsCredentialType,
    pub policy_arns: Option<Vec<String>>,
    pub policy_document: Option<String>,
    pub role_arns: Option<Vec<String>>,
    pub default_ttl: Option<u32>,
    pub max_ttl: Option<u32>,
    pub user_path: Option<String>,
}

/// AWS role response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsRoleResponse {
    pub name: String,
    pub credential_type: String,
    pub policy_arns: Vec<String>,
    pub policy_document: Option<String>,
    pub role_arns: Vec<String>,
    pub default_ttl: u32,
    pub max_ttl: u32,
    pub user_path: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<&AwsRole> for AwsRoleResponse {
    fn from(role: &AwsRole) -> Self {
        Self {
            name: role.name.clone(),
            credential_type: role.credential_type.as_str().to_string(),
            policy_arns: role.policy_arns.clone(),
            policy_document: role.policy_document.clone(),
            role_arns: role.role_arns.clone(),
            default_ttl: role.default_ttl,
            max_ttl: role.max_ttl,
            user_path: role.user_path.clone(),
            created_at: role.created_at,
        }
    }
}

/// AWS credentials generation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsCredentialsRequest {
    pub role_name: String,
    pub ttl: Option<u32>,
    pub role_session_name: Option<String>,
}

/// Tracked IAM user for cleanup
#[derive(Debug, Clone)]
struct TrackedIamUser {
    user_name: String,
    access_key_id: String,
    created_at: DateTime<Utc>,
    lease_id: String,
}

/// AWS Secrets Engine
pub struct AwsEngine {
    config: Arc<RwLock<Option<AwsConfig>>>,
    roles: Arc<RwLock<HashMap<String, AwsRole>>>,
    tracked_users: Arc<RwLock<HashMap<String, TrackedIamUser>>>,
    iam_client: Arc<RwLock<Option<IamClient>>>,
    sts_client: Arc<RwLock<Option<StsClient>>>,
}

impl AwsEngine {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(None)),
            roles: Arc::new(RwLock::new(HashMap::new())),
            tracked_users: Arc::new(RwLock::new(HashMap::new())),
            iam_client: Arc::new(RwLock::new(None)),
            sts_client: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn configure(&self, config: AwsConfig) -> Result<(), AwsError> {
        if config.access_key.is_empty() || config.secret_key.is_empty() {
            return Err(AwsError::InvalidConfig(
                "Access key and secret key are required".to_string(),
            ));
        }

        let sdk_config = self.create_sdk_config(&config).await?;
        let iam_client = IamClient::new(&sdk_config);
        let sts_client = StsClient::new(&sdk_config);

        match sts_client.get_caller_identity().send().await {
            Ok(response) => {
                info!(
                    "AWS configuration successful. Account: {:?}",
                    response.account()
                );
            }
            Err(e) => {
                return Err(AwsError::StsError(format!(
                    "Failed to verify credentials: {}",
                    e
                )));
            }
        }

        *self.config.write().await = Some(config);
        *self.iam_client.write().await = Some(iam_client);
        *self.sts_client.write().await = Some(sts_client);

        info!("AWS secrets engine configured successfully");
        Ok(())
    }

    pub async fn get_config(&self) -> Result<AwsConfig, AwsError> {
        self.config
            .read()
            .await
            .as_ref()
            .cloned()
            .ok_or(AwsError::ConfigNotFound)
    }

    async fn create_sdk_config(&self, config: &AwsConfig) -> Result<SdkConfig, AwsError> {
        let credentials = Credentials::new(
            &config.access_key,
            &config.secret_key,
            None,
            None,
            "secreton-aws-engine",
        );

        let config_builder = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new(config.region.clone()))
            .credentials_provider(credentials);

        Ok(config_builder.load().await)
    }

    pub async fn create_role(
        &self,
        request: AwsRoleCreateRequest,
    ) -> Result<AwsRoleResponse, AwsError> {
        let mut roles = self.roles.write().await;

        if roles.contains_key(&request.name) {
            return Err(AwsError::RoleAlreadyExists(request.name));
        }

        let mut role = AwsRole::new(request.name.clone(), request.credential_type);

        if let Some(policy_arns) = request.policy_arns {
            role.policy_arns = policy_arns;
        }
        if let Some(policy_document) = request.policy_document {
            role.policy_document = Some(policy_document);
        }
        if let Some(role_arns) = request.role_arns {
            role.role_arns = role_arns;
        }
        if let Some(default_ttl) = request.default_ttl {
            role.default_ttl = default_ttl;
        }
        if let Some(max_ttl) = request.max_ttl {
            role.max_ttl = max_ttl;
        }
        if let Some(user_path) = request.user_path {
            role.user_path = Some(user_path);
        }

        role.validate()?;

        let response = AwsRoleResponse::from(&role);
        roles.insert(request.name, role);

        info!("Created AWS role: {}", response.name);
        Ok(response)
    }

    pub async fn get_role(&self, role_name: &str) -> Result<AwsRoleResponse, AwsError> {
        let roles = self.roles.read().await;
        let role = roles
            .get(role_name)
            .ok_or_else(|| AwsError::RoleNotFound(role_name.to_string()))?;
        Ok(AwsRoleResponse::from(role))
    }

    pub async fn delete_role(&self, role_name: &str) -> Result<(), AwsError> {
        let mut roles = self.roles.write().await;
        roles
            .remove(role_name)
            .ok_or_else(|| AwsError::RoleNotFound(role_name.to_string()))?;

        info!("Deleted AWS role: {}", role_name);
        Ok(())
    }

    pub async fn list_roles(&self) -> Vec<String> {
        let roles = self.roles.read().await;
        roles.keys().cloned().collect()
    }

    pub async fn generate_credentials(
        &self,
        request: AwsCredentialsRequest,
    ) -> Result<AwsCredentials, AwsError> {
        let role = {
            let roles = self.roles.read().await;
            roles
                .get(&request.role_name)
                .ok_or_else(|| AwsError::RoleNotFound(request.role_name.clone()))?
                .clone()
        };

        let ttl = request.ttl.unwrap_or(role.default_ttl);
        if ttl > role.max_ttl {
            return Err(AwsError::InvalidTtl(format!(
                "Requested TTL {} exceeds max TTL {}",
                ttl, role.max_ttl
            )));
        }

        match role.credential_type {
            AwsCredentialType::IamUser => self.generate_iam_user_credentials(&role, ttl).await,
            AwsCredentialType::AssumeRole => {
                let session_name = request
                    .role_session_name
                    .unwrap_or_else(|| format!("secreton-{}", Utc::now().timestamp()));
                self.generate_assume_role_credentials(&role, ttl, &session_name)
                    .await
            }
            AwsCredentialType::FederationToken => {
                self.generate_federation_token_credentials(&role, ttl).await
            }
        }
    }

    async fn generate_iam_user_credentials(
        &self,
        role: &AwsRole,
        ttl: u32,
    ) -> Result<AwsCredentials, AwsError> {
        let iam_client = self
            .iam_client
            .read()
            .await
            .as_ref()
            .ok_or(AwsError::ConfigNotFound)?
            .clone();

        let user_name = format!("secreton-{}-{}", role.name, Utc::now().timestamp());
        let user_path = role.user_path.as_deref().unwrap_or("/secreton/");

        let tag1 = Tag::builder()
            .key("ManagedBy")
            .value("Secreton")
            .build()
            .map_err(|e| AwsError::IamError(format!("Failed to build tag: {}", e)))?;
        let tag2 = Tag::builder()
            .key("Role")
            .value(&role.name)
            .build()
            .map_err(|e| AwsError::IamError(format!("Failed to build tag: {}", e)))?;
        let tag3 = Tag::builder()
            .key("TTL")
            .value(ttl.to_string())
            .build()
            .map_err(|e| AwsError::IamError(format!("Failed to build tag: {}", e)))?;

        let _create_user_result = iam_client
            .create_user()
            .user_name(&user_name)
            .path(user_path)
            .tags(tag1)
            .tags(tag2)
            .tags(tag3)
            .send()
            .await
            .map_err(|e| AwsError::IamError(format!("Failed to create user: {}", e)))?;

        debug!("Created IAM user: {}", user_name);

        for policy_arn in &role.policy_arns {
            iam_client
                .attach_user_policy()
                .user_name(&user_name)
                .policy_arn(policy_arn)
                .send()
                .await
                .map_err(|e| AwsError::IamError(format!("Failed to attach policy: {}", e)))?;
        }

        if let Some(ref policy_doc) = role.policy_document {
            iam_client
                .put_user_policy()
                .user_name(&user_name)
                .policy_name("secreton-inline-policy")
                .policy_document(policy_doc)
                .send()
                .await
                .map_err(|e| AwsError::IamError(format!("Failed to put inline policy: {}", e)))?;
        }

        let create_key_result = iam_client
            .create_access_key()
            .user_name(&user_name)
            .send()
            .await
            .map_err(|e| AwsError::IamError(format!("Failed to create access key: {}", e)))?;

        let access_key = create_key_result.access_key().ok_or_else(|| {
            AwsError::CredentialGenerationFailed("No access key returned".to_string())
        })?;

        let lease_id = uuid::Uuid::new_v4().to_string();
        let expiration = Utc::now() + Duration::seconds(ttl as i64);

        let tracked_user = TrackedIamUser {
            user_name: user_name.clone(),
            access_key_id: access_key.access_key_id().to_string(),
            created_at: Utc::now(),
            lease_id: lease_id.clone(),
        };
        self.tracked_users
            .write()
            .await
            .insert(lease_id.clone(), tracked_user);

        info!("Generated IAM user credentials for role: {}", role.name);

        Ok(AwsCredentials {
            access_key_id: access_key.access_key_id().to_string(),
            secret_access_key: access_key.secret_access_key().to_string(),
            session_token: None,
            credential_type: "iam_user".to_string(),
            expiration,
            lease_id,
            user_name: Some(user_name),
        })
    }

    async fn generate_assume_role_credentials(
        &self,
        role: &AwsRole,
        ttl: u32,
        session_name: &str,
    ) -> Result<AwsCredentials, AwsError> {
        let sts_client = self
            .sts_client
            .read()
            .await
            .as_ref()
            .ok_or(AwsError::ConfigNotFound)?
            .clone();

        let role_arn = role
            .role_arns
            .first()
            .ok_or_else(|| AwsError::InvalidConfig("No role ARN specified".to_string()))?;

        let mut assume_role_request = sts_client
            .assume_role()
            .role_arn(role_arn)
            .role_session_name(session_name)
            .duration_seconds(ttl as i32);

        for policy_arn in &role.policy_arns {
            assume_role_request = assume_role_request
                .policy_arns(PolicyDescriptorType::builder().arn(policy_arn).build());
        }

        if let Some(ref policy_doc) = role.policy_document {
            assume_role_request = assume_role_request.policy(policy_doc);
        }

        let assume_role_result = assume_role_request
            .send()
            .await
            .map_err(|e| AwsError::StsError(format!("Failed to assume role: {}", e)))?;

        let credentials = assume_role_result.credentials().ok_or_else(|| {
            AwsError::CredentialGenerationFailed("No credentials returned".to_string())
        })?;

        let lease_id = uuid::Uuid::new_v4().to_string();
        let expiration = Utc::now() + Duration::seconds(ttl as i64);

        info!("Generated AssumeRole credentials for role: {}", role.name);

        Ok(AwsCredentials {
            access_key_id: credentials.access_key_id().to_string(),
            secret_access_key: credentials.secret_access_key().to_string(),
            session_token: Some(credentials.session_token().to_string()),
            credential_type: "assume_role".to_string(),
            expiration,
            lease_id,
            user_name: None,
        })
    }

    async fn generate_federation_token_credentials(
        &self,
        role: &AwsRole,
        ttl: u32,
    ) -> Result<AwsCredentials, AwsError> {
        let sts_client = self
            .sts_client
            .read()
            .await
            .as_ref()
            .ok_or(AwsError::ConfigNotFound)?
            .clone();

        let token_name = format!("secreton-{}-{}", role.name, Utc::now().timestamp());

        let mut federation_request = sts_client
            .get_federation_token()
            .name(&token_name)
            .duration_seconds(ttl as i32);

        if let Some(ref policy_doc) = role.policy_document {
            federation_request = federation_request.policy(policy_doc);
        }

        let federation_result = federation_request
            .send()
            .await
            .map_err(|e| AwsError::StsError(format!("Failed to get federation token: {}", e)))?;

        let credentials = federation_result.credentials().ok_or_else(|| {
            AwsError::CredentialGenerationFailed("No credentials returned".to_string())
        })?;

        let lease_id = uuid::Uuid::new_v4().to_string();
        let expiration = Utc::now() + Duration::seconds(ttl as i64);

        info!(
            "Generated federation token credentials for role: {}",
            role.name
        );

        Ok(AwsCredentials {
            access_key_id: credentials.access_key_id().to_string(),
            secret_access_key: credentials.secret_access_key().to_string(),
            session_token: Some(credentials.session_token().to_string()),
            credential_type: "federation_token".to_string(),
            expiration,
            lease_id,
            user_name: None,
        })
    }

    pub async fn revoke_credentials(&self, lease_id: &str) -> Result<(), AwsError> {
        let mut tracked_users = self.tracked_users.write().await;

        if let Some(tracked_user) = tracked_users.remove(lease_id) {
            let iam_client = self
                .iam_client
                .read()
                .await
                .as_ref()
                .ok_or(AwsError::ConfigNotFound)?
                .clone();

            let _ = iam_client
                .delete_access_key()
                .user_name(&tracked_user.user_name)
                .access_key_id(&tracked_user.access_key_id)
                .send()
                .await;

            if let Ok(attached_policies) = iam_client
                .list_attached_user_policies()
                .user_name(&tracked_user.user_name)
                .send()
                .await
            {
                for policy in attached_policies.attached_policies() {
                    if let Some(policy_arn) = policy.policy_arn() {
                        let _ = iam_client
                            .detach_user_policy()
                            .user_name(&tracked_user.user_name)
                            .policy_arn(policy_arn)
                            .send()
                            .await;
                    }
                }
            }

            if let Ok(inline_policies) = iam_client
                .list_user_policies()
                .user_name(&tracked_user.user_name)
                .send()
                .await
            {
                for policy_name in inline_policies.policy_names() {
                    let _ = iam_client
                        .delete_user_policy()
                        .user_name(&tracked_user.user_name)
                        .policy_name(policy_name)
                        .send()
                        .await;
                }
            }

            iam_client
                .delete_user()
                .user_name(&tracked_user.user_name)
                .send()
                .await
                .map_err(|e| AwsError::RevocationFailed(format!("Failed to delete user: {}", e)))?;

            info!("Revoked IAM user credentials: {}", tracked_user.user_name);
        } else {
            debug!(
                "Lease {} not found or already expired (STS credentials)",
                lease_id
            );
        }

        Ok(())
    }

    pub async fn cleanup_expired_users(&self) -> usize {
        let tracked_users = self.tracked_users.write().await;
        let now = Utc::now();
        let mut expired = Vec::new();

        for (lease_id, user) in tracked_users.iter() {
            if now.signed_duration_since(user.created_at).num_hours() > 13 {
                expired.push(lease_id.clone());
            }
        }

        let count = expired.len();

        for lease_id in expired {
            if let Err(e) = self.revoke_credentials(&lease_id).await {
                warn!(
                    "Failed to cleanup expired user for lease {}: {}",
                    lease_id, e
                );
            }
        }

        if count > 0 {
            info!("Cleaned up {} expired IAM users", count);
        }

        count
    }
}

impl Default for AwsEngine {
    fn default() -> Self {
        Self::new()
    }
}

// Enhanced methods for lease integration
impl AwsEngine {
    /// Generate credentials with lease integration
    pub async fn generate_credentials_with_lease(
        &self,
        request: AwsCredentialsRequest,
        lease_manager: &crate::services::lease::LeaseManager,
        user: &str,
    ) -> Result<(AwsCredentials, crate::services::lease::EnhancedLease), AwsError> {
        // Generate credentials
        let credentials = self.generate_credentials(request.clone()).await?;

        // Get role for TTL info
        let role = {
            let roles = self.roles.read().await;
            roles
                .get(&request.role_name)
                .ok_or_else(|| AwsError::RoleNotFound(request.role_name.clone()))?
                .clone()
        };

        let ttl_secs = request.ttl.unwrap_or(role.default_ttl) as i64;
        let max_ttl = role.max_ttl as i64;

        let resource_path = format!("aws/creds/{}", request.role_name);
        let lease = lease_manager
            .create_lease(
                user,
                &resource_path,
                "aws",
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
                AwsError::CredentialGenerationFailed(format!("Failed to create lease: {}", e))
            })?;

        Ok((credentials, lease))
    }

    /// Revoke credentials with lease
    pub async fn revoke_credentials_with_lease(
        &self,
        lease_id: &str,
        lease_manager: &crate::services::lease::LeaseManager,
    ) -> Result<(), AwsError> {
        // Revoke credentials
        self.revoke_credentials(lease_id).await?;

        // Revoke lease
        lease_manager
            .revoke_lease(lease_id)
            .await
            .map_err(|e| AwsError::RevocationFailed(format!("Failed to revoke lease: {}", e)))?;

        Ok(())
    }

    /// List all active credentials
    pub async fn list_active_credentials(&self) -> Vec<String> {
        let tracked_users = self.tracked_users.read().await;
        tracked_users.keys().cloned().collect()
    }

    /// Get credential details by lease ID
    pub async fn get_credential_details(&self, lease_id: &str) -> Option<String> {
        let tracked_users = self.tracked_users.read().await;
        tracked_users
            .get(lease_id)
            .map(|user| user.user_name.clone())
    }
}
