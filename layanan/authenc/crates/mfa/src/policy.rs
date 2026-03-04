//! MFA policy enforcement
//!
//! Provides MFA requirement checking and policy enforcement logic.

use async_trait::async_trait;
use authenc_types::{AuthencError, Result, UserId};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info};

/// MFA requirement level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum MfaRequirement {
    /// MFA is disabled
    Disabled,
    /// MFA is optional (user can choose)
    #[default]
    Optional,
    /// MFA is required for all users
    Required,
    /// MFA is required for admin users only
    RequiredForAdmins,
}


/// MFA policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaPolicy {
    /// MFA requirement level
    pub requirement: MfaRequirement,
    /// Grace period in days before MFA is enforced (for Required mode)
    pub grace_period_days: Option<u32>,
    /// Allow backup codes as MFA method
    pub allow_backup_codes: bool,
    /// Allow TOTP as MFA method
    pub allow_totp: bool,
    /// Allow WebAuthn as MFA method
    pub allow_webauthn: bool,
}

impl Default for MfaPolicy {
    fn default() -> Self {
        Self {
            requirement: MfaRequirement::Optional,
            grace_period_days: Some(30),
            allow_backup_codes: true,
            allow_totp: true,
            allow_webauthn: true,
        }
    }
}

/// User MFA status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaStatus {
    /// Whether MFA is enabled for the user
    pub enabled: bool,
    /// TOTP is configured
    pub totp_configured: bool,
    /// Backup codes are available
    pub backup_codes_available: bool,
    /// WebAuthn is configured
    pub webauthn_configured: bool,
    /// Number of remaining backup codes
    pub remaining_backup_codes: usize,
}

/// Trait for MFA policy storage
#[async_trait]
pub trait MfaPolicyStore: Send + Sync {
    /// Get MFA policy for a realm
    async fn get_mfa_policy(&self, realm_id: &str) -> Result<MfaPolicy>;

    /// Update MFA policy for a realm
    async fn update_mfa_policy(&self, realm_id: &str, policy: &MfaPolicy) -> Result<()>;

    /// Get user MFA status
    async fn get_user_mfa_status(&self, user_id: UserId) -> Result<MfaStatus>;

    /// Check if user is admin
    async fn is_user_admin(&self, user_id: UserId) -> Result<bool>;
}

/// MFA policy enforcement service
pub struct MfaPolicyService<S: MfaPolicyStore> {
    store: Arc<S>,
}

impl<S: MfaPolicyStore> MfaPolicyService<S> {
    /// Create a new MFA policy service
    pub fn new(store: Arc<S>) -> Self {
        Self { store }
    }

    /// Check if MFA is required for a user
    pub async fn is_mfa_required(&self, user_id: UserId, realm_id: &str) -> Result<bool> {
        debug!(
            "Checking MFA requirement for user: {} in realm: {}",
            user_id, realm_id
        );

        // Get policy
        let policy = self.store.get_mfa_policy(realm_id).await?;

        match policy.requirement {
            MfaRequirement::Disabled => Ok(false),
            MfaRequirement::Optional => Ok(false),
            MfaRequirement::Required => Ok(true),
            MfaRequirement::RequiredForAdmins => {
                let is_admin = self.store.is_user_admin(user_id).await?;
                Ok(is_admin)
            }
        }
    }

    /// Check if user has MFA configured
    pub async fn has_mfa_configured(&self, user_id: UserId) -> Result<bool> {
        let status = self.store.get_user_mfa_status(user_id).await?;
        Ok(status.enabled && (status.totp_configured || status.webauthn_configured))
    }

    /// Validate MFA setup for a user
    pub async fn validate_mfa_setup(&self, user_id: UserId, realm_id: &str) -> Result<()> {
        info!(
            "Validating MFA setup for user: {} in realm: {}",
            user_id, realm_id
        );

        // Check if MFA is required
        let is_required = self.is_mfa_required(user_id, realm_id).await?;

        if !is_required {
            return Ok(());
        }

        // Check if user has MFA configured
        let has_mfa = self.has_mfa_configured(user_id).await?;

        if !has_mfa {
            return Err(AuthencError::MfaRequired);
        }

        Ok(())
    }

    /// Get available MFA methods for a user
    pub async fn get_available_mfa_methods(
        &self,
        user_id: UserId,
        realm_id: &str,
    ) -> Result<Vec<String>> {
        let policy = self.store.get_mfa_policy(realm_id).await?;
        let status = self.store.get_user_mfa_status(user_id).await?;

        let mut methods = Vec::new();

        if policy.allow_totp && status.totp_configured {
            methods.push("totp".to_string());
        }

        if policy.allow_webauthn && status.webauthn_configured {
            methods.push("webauthn".to_string());
        }

        if policy.allow_backup_codes && status.backup_codes_available {
            methods.push("backup_code".to_string());
        }

        Ok(methods)
    }

    /// Check if a specific MFA method is allowed
    pub async fn is_mfa_method_allowed(&self, realm_id: &str, method: &str) -> Result<bool> {
        let policy = self.store.get_mfa_policy(realm_id).await?;

        let allowed = match method {
            "totp" => policy.allow_totp,
            "webauthn" => policy.allow_webauthn,
            "backup_code" => policy.allow_backup_codes,
            _ => false,
        };

        Ok(allowed)
    }

    /// Update MFA policy for a realm
    pub async fn update_policy(&self, realm_id: &str, policy: MfaPolicy) -> Result<()> {
        info!("Updating MFA policy for realm: {}", realm_id);
        self.store.update_mfa_policy(realm_id, &policy).await
    }

    /// Get MFA policy for a realm
    pub async fn get_policy(&self, realm_id: &str) -> Result<MfaPolicy> {
        self.store.get_mfa_policy(realm_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tokio::sync::RwLock;

    /// Mock MFA policy store for testing
    struct MockMfaPolicyStore {
        policies: Arc<RwLock<HashMap<String, MfaPolicy>>>,
        user_statuses: Arc<RwLock<HashMap<UserId, MfaStatus>>>,
        admin_users: Arc<RwLock<Vec<UserId>>>,
    }

    impl MockMfaPolicyStore {
        fn new() -> Self {
            Self {
                policies: Arc::new(RwLock::new(HashMap::new())),
                user_statuses: Arc::new(RwLock::new(HashMap::new())),
                admin_users: Arc::new(RwLock::new(Vec::new())),
            }
        }

        async fn with_policy_async(self, realm_id: &str, policy: MfaPolicy) -> Self {
            self.policies
                .write()
                .await
                .insert(realm_id.to_string(), policy);
            self
        }

        async fn with_user_status_async(self, user_id: UserId, status: MfaStatus) -> Self {
            self.user_statuses.write().await.insert(user_id, status);
            self
        }

        async fn with_admin_user_async(self, user_id: UserId) -> Self {
            self.admin_users.write().await.push(user_id);
            self
        }
    }

    #[async_trait]
    impl MfaPolicyStore for MockMfaPolicyStore {
        async fn get_mfa_policy(&self, realm_id: &str) -> Result<MfaPolicy> {
            self.policies
                .read()
                .await
                .get(realm_id)
                .cloned()
                .ok_or_else(|| AuthencError::not_found("Policy not found"))
        }

        async fn update_mfa_policy(&self, realm_id: &str, policy: &MfaPolicy) -> Result<()> {
            self.policies
                .write()
                .await
                .insert(realm_id.to_string(), policy.clone());
            Ok(())
        }

        async fn get_user_mfa_status(&self, user_id: UserId) -> Result<MfaStatus> {
            self.user_statuses
                .read()
                .await
                .get(&user_id)
                .cloned()
                .ok_or_else(|| AuthencError::not_found("User status not found"))
        }

        async fn is_user_admin(&self, user_id: UserId) -> Result<bool> {
            Ok(self.admin_users.read().await.contains(&user_id))
        }
    }

    #[tokio::test]
    async fn test_mfa_required_for_all() {
        let user_id = UserId::new();
        let realm_id = "test-realm";

        let store = MockMfaPolicyStore::new()
            .with_policy_async(
                realm_id,
                MfaPolicy {
                    requirement: MfaRequirement::Required,
                    ..Default::default()
                },
            )
            .await;

        let service = MfaPolicyService::new(Arc::new(store));

        let is_required = service.is_mfa_required(user_id, realm_id).await.unwrap();
        assert!(is_required);
    }

    #[tokio::test]
    async fn test_mfa_required_for_admins_only() {
        let admin_user_id = UserId::new();
        let regular_user_id = UserId::new();
        let realm_id = "test-realm";

        let store = MockMfaPolicyStore::new()
            .with_policy_async(
                realm_id,
                MfaPolicy {
                    requirement: MfaRequirement::RequiredForAdmins,
                    ..Default::default()
                },
            )
            .await
            .with_admin_user_async(admin_user_id)
            .await;

        let service = MfaPolicyService::new(Arc::new(store));

        // Admin user should require MFA
        let is_required = service
            .is_mfa_required(admin_user_id, realm_id)
            .await
            .unwrap();
        assert!(is_required);

        // Regular user should not require MFA
        let is_required = service
            .is_mfa_required(regular_user_id, realm_id)
            .await
            .unwrap();
        assert!(!is_required);
    }

    #[tokio::test]
    async fn test_validate_mfa_setup() {
        let user_id = UserId::new();
        let realm_id = "test-realm";

        let store = MockMfaPolicyStore::new()
            .with_policy_async(
                realm_id,
                MfaPolicy {
                    requirement: MfaRequirement::Required,
                    ..Default::default()
                },
            )
            .await
            .with_user_status_async(
                user_id,
                MfaStatus {
                    enabled: true,
                    totp_configured: true,
                    backup_codes_available: true,
                    webauthn_configured: false,
                    remaining_backup_codes: 10,
                },
            )
            .await;

        let service = MfaPolicyService::new(Arc::new(store));

        // Should succeed because user has MFA configured
        let result = service.validate_mfa_setup(user_id, realm_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_mfa_setup_fails_when_not_configured() {
        let user_id = UserId::new();
        let realm_id = "test-realm";

        let store = MockMfaPolicyStore::new()
            .with_policy_async(
                realm_id,
                MfaPolicy {
                    requirement: MfaRequirement::Required,
                    ..Default::default()
                },
            )
            .await
            .with_user_status_async(
                user_id,
                MfaStatus {
                    enabled: false,
                    totp_configured: false,
                    backup_codes_available: false,
                    webauthn_configured: false,
                    remaining_backup_codes: 0,
                },
            )
            .await;

        let service = MfaPolicyService::new(Arc::new(store));

        // Should fail because user doesn't have MFA configured
        let result = service.validate_mfa_setup(user_id, realm_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_available_mfa_methods() {
        let user_id = UserId::new();
        let realm_id = "test-realm";

        let store = MockMfaPolicyStore::new()
            .with_policy_async(
                realm_id,
                MfaPolicy {
                    requirement: MfaRequirement::Optional,
                    allow_totp: true,
                    allow_webauthn: true,
                    allow_backup_codes: true,
                    ..Default::default()
                },
            )
            .await
            .with_user_status_async(
                user_id,
                MfaStatus {
                    enabled: true,
                    totp_configured: true,
                    backup_codes_available: true,
                    webauthn_configured: false,
                    remaining_backup_codes: 10,
                },
            )
            .await;

        let service = MfaPolicyService::new(Arc::new(store));

        let methods = service
            .get_available_mfa_methods(user_id, realm_id)
            .await
            .unwrap();

        assert!(methods.contains(&"totp".to_string()));
        assert!(methods.contains(&"backup_code".to_string()));
        assert!(!methods.contains(&"webauthn".to_string()));
    }

    #[tokio::test]
    async fn test_is_mfa_method_allowed() {
        let realm_id = "test-realm";

        let store = MockMfaPolicyStore::new()
            .with_policy_async(
                realm_id,
                MfaPolicy {
                    requirement: MfaRequirement::Optional,
                    allow_totp: true,
                    allow_webauthn: false,
                    allow_backup_codes: true,
                    ..Default::default()
                },
            )
            .await;

        let service = MfaPolicyService::new(Arc::new(store));

        assert!(
            service
                .is_mfa_method_allowed(realm_id, "totp")
                .await
                .unwrap()
        );
        assert!(
            !service
                .is_mfa_method_allowed(realm_id, "webauthn")
                .await
                .unwrap()
        );
        assert!(
            service
                .is_mfa_method_allowed(realm_id, "backup_code")
                .await
                .unwrap()
        );
        assert!(
            !service
                .is_mfa_method_allowed(realm_id, "unknown")
                .await
                .unwrap()
        );
    }
}
