//! Client Policy Enforcement Service
//!
//! Integrates policy evaluation with OAuth2/OIDC flows.
//! Evaluates client policies during authorization and token requests.

use crate::error::Result;
use crate::models::client_policy::ClientPolicyModel;
use crate::models::oauth2::OAuth2Client;
use crate::services::client_policy::{
    ClientPolicy, ClientPolicyCondition, ClientPolicyContext, ClientPolicyExecutor,
    ClientPolicyManager, ClientProfile, store::ClientPolicyStore,
};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Client Policy Enforcement Service
/// Orchestrates policy evaluation for OAuth2/OIDC clients during authentication flows.
pub struct ClientPolicyEnforcer {
    policy_store: Arc<ClientPolicyStore>,
    policy_manager: ClientPolicyManager,
}

impl ClientPolicyEnforcer {
    /// Create a new client policy enforcer
    pub fn new(policy_store: Arc<ClientPolicyStore>) -> Self {
        Self {
            policy_store,
            policy_manager: ClientPolicyManager::default(),
        }
    }

    /// Evaluate policies for an authorization request
    ///
    /// This is called during the OAuth2 authorization code flow or OIDC authentication.
    pub async fn evaluate_authorization_request(
        &self,
        client: &OAuth2Client,
        grant_type: Option<String>,
        response_type: Option<String>,
        redirect_uri: Option<String>,
        parameters: HashMap<String, String>,
        user_id: Option<String>,
    ) -> Result<()> {
        debug!(
            "Evaluating authorization request for client: {}",
            client.client_id
        );

        // Build context
        let context = ClientPolicyContext {
            client: client.clone(),
            grant_type,
            response_type,
            redirect_uri,
            client_auth_method: Some(client.token_endpoint_auth_method.clone()),
            parameters,
            user_id,
            device_fingerprint: None,
        };

        // Load policies for the client
        let policies = self
            .policy_store
            .get_client_policies(client.id)
            .await
            .map_err(|e| {
                error!("Failed to load policies for client {}: {}", client.id, e);
                e
            })?;

        if policies.is_empty() {
            debug!("No policies assigned to client {}", client.client_id);
            return Ok(());
        }

        info!(
            "Evaluating {} policies for client {}",
            policies.len(),
            client.client_id
        );

        // Convert database models to in-memory policy structures
        let policy_profiles = self.build_policy_profiles(policies)?;

        // Temporarily replace the policy manager's profiles
        let mut temp_manager = ClientPolicyManager::new();
        self.register_default_conditions_and_executors(&mut temp_manager);

        for profile in policy_profiles {
            temp_manager.add_profile(profile);
        }

        // Evaluate policies
        match temp_manager.evaluate_policies(context).await {
            Ok(_) => {
                info!("All policies passed for client {}", client.client_id);
                Ok(())
            }
            Err(e) => {
                warn!("Policy violation for client {}: {}", client.client_id, e);
                Err(e)
            }
        }
    }

    /// Evaluate policies for a token request
    ///
    /// This is called during token endpoint requests (authorization code, refresh, etc.)
    pub async fn evaluate_token_request(
        &self,
        client: &OAuth2Client,
        grant_type: String,
        parameters: HashMap<String, String>,
    ) -> Result<()> {
        debug!(
            "Evaluating token request for client: {} (grant_type: {})",
            client.client_id, grant_type
        );

        let context = ClientPolicyContext {
            client: client.clone(),
            grant_type: Some(grant_type),
            response_type: None,
            redirect_uri: parameters.get("redirect_uri").cloned(),
            client_auth_method: Some(client.token_endpoint_auth_method.clone()),
            parameters,
            user_id: None,
            device_fingerprint: None,
        };

        // Load and evaluate policies
        let policies = self.policy_store.get_client_policies(client.id).await?;

        if policies.is_empty() {
            return Ok(());
        }

        let policy_profiles = self.build_policy_profiles(policies)?;
        let mut temp_manager = ClientPolicyManager::new();
        self.register_default_conditions_and_executors(&mut temp_manager);

        for profile in policy_profiles {
            temp_manager.add_profile(profile);
        }

        match temp_manager.evaluate_policies(context).await {
            Ok(_) => Ok(()),
            Err(e) => {
                warn!(
                    "Policy violation for client {} during token request: {}",
                    client.client_id, e
                );
                Err(e)
            }
        }
    }

    /// Build in-memory policy profiles from database models
    fn build_policy_profiles(
        &self,
        policies: Vec<ClientPolicyModel>,
    ) -> Result<Vec<ClientProfile>> {
        // Group policies into a single profile for evaluation
        let client_policies: Vec<ClientPolicy> = policies
            .into_iter()
            .map(|p| ClientPolicy {
                name: p.name,
                description: p.description,
                enabled: p.enabled,
                conditions: p.conditions,
                executors: p.executors,
                priority: p.priority,
            })
            .collect();

        // Create a runtime profile
        let profile = ClientProfile {
            name: "Runtime Policy Profile".to_string(),
            description: "Combined policies for this client".to_string(),
            enabled: true,
            policies: client_policies,
        };

        Ok(vec![profile])
    }

    /// Register default conditions and executors
    fn register_default_conditions_and_executors(&self, manager: &mut ClientPolicyManager) {
        use crate::services::client_policy::*;

        // Register conditions
        manager.register_condition(Box::new(AnyClientCondition));

        manager.register_condition(Box::new(GrantTypeCondition {
            allowed_grant_types: vec![
                "authorization_code".to_string(),
                "client_credentials".to_string(),
                "refresh_token".to_string(),
            ],
        }));

        manager.register_condition(Box::new(ClientAccessTypeCondition {
            allowed_access_types: vec!["confidential".to_string(), "public".to_string()],
        }));

        // Register executors
        manager.register_executor(Box::new(PkceEnforcerExecutor { enforce_pkce: true }));

        manager.register_executor(Box::new(DPoPBindEnforcerExecutor {
            enforce_dpop: false,
        }));

        manager.register_executor(Box::new(SecureRedirectUrisEnforcerExecutor {
            enforce_https: true,
        }));

        manager.register_executor(Box::new(RejectImplicitGrantExecutor));

        manager.register_executor(Box::new(ConfidentialClientAcceptExecutor {
            accept_confidential_only: false,
        }));

        manager.register_executor(Box::new(ConsentRequiredExecutor {
            require_consent: false,
        }));

        manager.register_executor(Box::new(SecureSigningAlgorithmExecutor {
            enforce_secure_algorithm: false,
            allowed_algorithms: vec![
                "EdDSA".to_string(),
                "ES256".to_string(),
                "RS256".to_string(),
            ],
        }));

        manager.register_executor(Box::new(
            RejectResourceOwnerPasswordCredentialsGrantExecutor { reject_ropc: false },
        ));
    }

    /// Check if PKCE is required for a client
    pub async fn is_pkce_required(&self, client_id: Uuid) -> Result<bool> {
        let policies = self.policy_store.get_client_policies(client_id).await?;

        // Check if any policy has PKCE enforcer
        for policy in policies {
            if policy
                .executors
                .contains(&"pkce-enforcer-executor".to_string())
            {
                if let Some(config) = policy.executor_config.get("pkce-enforcer-executor") {
                    if let Some(enforce) = config.get("enforce_pkce") {
                        if enforce.as_bool() == Some(true) {
                            return Ok(true);
                        }
                    }
                }
            }
        }

        Ok(false)
    }

    /// Check if DPoP is required for a client
    pub async fn is_dpop_required(&self, client_id: Uuid) -> Result<bool> {
        let policies = self.policy_store.get_client_policies(client_id).await?;

        for policy in policies {
            if policy
                .executors
                .contains(&"dpop-bind-enforcer-executor".to_string())
            {
                if let Some(config) = policy.executor_config.get("dpop-bind-enforcer-executor") {
                    if let Some(enforce) = config.get("enforce_dpop") {
                        if enforce.as_bool() == Some(true) {
                            return Ok(true);
                        }
                    }
                }
            }
        }

        Ok(false)
    }

    /// Check if consent is required for a client
    pub async fn is_consent_required(&self, client_id: Uuid) -> Result<bool> {
        let policies = self.policy_store.get_client_policies(client_id).await?;

        for policy in policies {
            if policy
                .executors
                .contains(&"consent-required-executor".to_string())
            {
                if let Some(config) = policy.executor_config.get("consent-required-executor") {
                    if let Some(require) = config.get("require_consent") {
                        if require.as_bool() == Some(true) {
                            return Ok(true);
                        }
                    }
                }
            }
        }

        Ok(false)
    }

    /// Get allowed grant types for a client
    pub async fn get_allowed_grant_types(&self, client_id: Uuid) -> Result<Vec<String>> {
        let policies = self.policy_store.get_client_policies(client_id).await?;

        // Check for grant type restrictions
        for policy in policies {
            if policy
                .conditions
                .contains(&"grant-type-condition".to_string())
            {
                if let Some(config) = policy.condition_config.get("grant-type-condition") {
                    if let Some(allowed) = config.get("allowed_grant_types") {
                        if let Some(grant_types) = allowed.as_array() {
                            return Ok(grant_types
                                .iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect());
                        }
                    }
                }
            }
        }

        // Default: all standard grant types
        Ok(vec![
            "authorization_code".to_string(),
            "client_credentials".to_string(),
            "refresh_token".to_string(),
        ])
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    #[ignore] // Requires database
    async fn test_policy_evaluation() {
        // Test implementation would go here
    }
}
