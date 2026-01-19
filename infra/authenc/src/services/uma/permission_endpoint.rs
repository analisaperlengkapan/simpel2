//! UMA 2.0 Permission Endpoint
//!
//! Handles permission ticket requests and RPT issuance.
//! Implements the UMA 2.0 authorization flow.

use super::policy_engine::{
    EnvironmentContext, PolicyDecision, PolicyEngine, PolicyEvaluationContext, ResourceContext,
    SubjectContext,
};
use super::rpt::{Permission as RptPermission, Rpt, RptService};
use super::{UmaAuthorizationRequest, UmaAuthorizationResponse, UmaError, UmaPermissionRequest};
use crate::database::Database;
use crate::error::{AuthencError, Result};
use crate::models::permission_ticket::{CreatePermissionTicketRequest, PermissionTicket};
use crate::models::resource::Resource;
use crate::services::permission_ticket_store::PermissionTicketStoreTrait;
use crate::services::resource_store::ResourceStoreTrait;
use crate::services::uma_policy_store::UmaPolicyStore;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Permission endpoint service
pub struct PermissionEndpoint {
    /// Database connection
    db: Arc<Database>,
    /// Resource store
    resource_store: Arc<dyn ResourceStoreTrait>,
    /// Permission ticket store
    ticket_store: Arc<dyn PermissionTicketStoreTrait>,
    /// Policy engine
    policy_engine: Arc<PolicyEngine>,
    /// RPT service
    rpt_service: Arc<RptService>,
    /// UMA policy store
    uma_policy_store: Arc<UmaPolicyStore>,
    /// Permission ticket lifetime in seconds
    ticket_lifetime: i64,
}

impl PermissionEndpoint {
    /// Create new permission endpoint
    pub fn new(
        db: Arc<Database>,
        resource_store: Arc<dyn ResourceStoreTrait>,
        ticket_store: Arc<dyn PermissionTicketStoreTrait>,
        policy_engine: Arc<PolicyEngine>,
        rpt_service: Arc<RptService>,
        uma_policy_store: Arc<UmaPolicyStore>,
        ticket_lifetime: i64,
    ) -> Self {
        Self {
            db,
            resource_store,
            ticket_store,
            policy_engine,
            rpt_service,
            uma_policy_store,
            ticket_lifetime,
        }
    }

    /// Request permission ticket
    ///
    /// Called by resource server when client attempts access without valid RPT
    pub async fn request_permission_ticket(
        &self,
        realm_id: &str,
        resource_server_id: &str,
        requests: Vec<UmaPermissionRequest>,
    ) -> Result<String> {
        // Parse realm and resource server IDs to UUIDs for comparison and ticket creation
        let realm_uuid = Uuid::parse_str(realm_id)
            .map_err(|_| AuthencError::validation("Invalid realm ID format"))?;

        let resource_server_uuid = Uuid::parse_str(resource_server_id)
            .map_err(|_| AuthencError::validation("Invalid resource server ID format"))?;

        // Validate that all requested resources exist and belong to this resource server
        for req in &requests {
            let resource_uuid = Uuid::parse_str(&req.resource_id)
                .map_err(|_| AuthencError::validation("Invalid resource ID format"))?;
            let resource = self
                .resource_store
                .get_resource(resource_uuid)
                .await?
                .ok_or_else(|| {
                    AuthencError::resource_not_found(format!(
                        "Resource {} not found",
                        req.resource_id
                    ))
                })?;

            // Verify resource belongs to this resource server
            if resource.resource_server_id != resource_server_uuid {
                return Err(AuthencError::unauthorized(
                    "Resource does not belong to this resource server",
                ));
            }

            // Verify resource is in the same realm
            if resource.realm_id != realm_uuid {
                return Err(AuthencError::unauthorized(
                    "Resource is not in the same realm",
                ));
            }

            // Verify requested scopes are valid for this resource
            for scope in &req.resource_scopes {
                if !resource.scopes.contains(scope) {
                    return Err(AuthencError::validation(format!(
                        "Invalid scope '{}' for resource '{}'",
                        scope, req.resource_id
                    )));
                }
            }
        }

        // Create permission ticket
        let ticket_id = Uuid::new_v4();
        let ticket_string = ticket_id.to_string();
        let expires_at = Utc::now().timestamp() + self.ticket_lifetime;

        // Store ticket with requested permissions
        // Create permission tickets for each resource+scope combination
        for req in &requests {
            for scope in &req.resource_scopes {
                // Store ticket in database
                self.ticket_store
                    .create_ticket(
                        CreatePermissionTicketRequest {
                            resource_id: Uuid::parse_str(&req.resource_id).map_err(|_| {
                                AuthencError::validation("Invalid resource ID format")
                            })?,
                            scope_id: Uuid::new_v4(), // In production: lookup actual scope ID
                            requester: "requesting_party".to_string(), // In production: actual requester ID
                        },
                        "resource_owner".to_string(), // In production: actual resource owner
                        realm_uuid,
                        resource_server_uuid,
                    )
                    .await
                    .map_err(|e| {
                        tracing::error!("Failed to store permission ticket: {}", e);
                        AuthencError::internal("Failed to create permission ticket")
                    })?;
            }
        }

        Ok(ticket_string)
    }

    /// Authorize access and issue RPT
    ///
    /// Called by client to exchange permission ticket for RPT
    pub async fn authorize_access(
        &self,
        realm_id: &str,
        request: UmaAuthorizationRequest,
        subject_id: &str,
        client_id: &str,
        subject_attributes: HashMap<String, serde_json::Value>,
        environment: EnvironmentContext,
    ) -> Result<UmaAuthorizationResponse> {
        // Validate permission ticket
        let ticket = self
            .validate_permission_ticket(&request.ticket, realm_id)
            .await?;

        // Get resources from ticket
        let resources = self.get_resources_from_ticket(&ticket).await?;

        // Build policy evaluation context
        let context = self.build_evaluation_context(
            subject_id,
            client_id,
            &resources,
            subject_attributes,
            environment,
            request.claim_token,
        )?;

        // Evaluate policies for each resource
        let mut granted_permissions = Vec::new();
        let mut all_required_claims = Vec::new();

        for resource in &resources {
            // Load policies for this resource
            let policies = self.load_policies_for_resource(&resource.id).await?;

            // Evaluate policies
            let result = self
                .policy_engine
                .evaluate_policies(&context, &policies)
                .await?;

            match result.decision {
                PolicyDecision::Permit => {
                    // Add granted permissions
                    for scope in &resource.scopes {
                        granted_permissions.push(RptPermission {
                            resource_id: resource.id.to_string(),
                            resource_name: Some(resource.name.clone()),
                            scopes: vec![scope.clone()],
                            claims: None,
                        });
                    }
                }
                PolicyDecision::Deny => {
                    // Policy explicitly denied - return error
                    return Err(AuthencError::forbidden(
                        result
                            .reason
                            .unwrap_or_else(|| "Access denied by policy".to_string()),
                    ));
                }
                PolicyDecision::NeedInfo => {
                    // Need more claims - collect them
                    if let Some(claims) = result.required_claims {
                        all_required_claims.extend(claims);
                    }
                }
                PolicyDecision::NotApplicable => {
                    // No applicable policy - default deny
                    return Err(AuthencError::forbidden(
                        "No applicable authorization policy",
                    ));
                }
            }
        }

        // If we need more claims, return error with claims requirement
        if !all_required_claims.is_empty() {
            return Err(AuthencError::Uma(UmaError {
                error: "need_info".to_string(),
                error_description: Some("Additional claims required".to_string()),
                status: Some(403),
                ticket: Some(request.ticket.clone()),
                required_claims: Some(all_required_claims),
                redirect_uri: None,
            }));
        }

        // Check if this is an upgrade request
        let is_upgrade = request.rpt.is_some();
        let rpt = if let Some(ref existing_rpt) = request.rpt {
            // Verify existing RPT and reconstruct full Rpt
            let claims = self.rpt_service.verify_rpt(existing_rpt)?;
            let existing = Rpt::from_token(existing_rpt.clone(), claims);

            // Upgrade RPT with new permissions
            self.rpt_service
                .upgrade_rpt(&existing, granted_permissions)?
        } else {
            // Create new RPT
            self.rpt_service.create_rpt(
                subject_id,
                vec![client_id.to_string()],
                client_id,
                realm_id,
                granted_permissions,
            )?
        };

        // Mark permission ticket as used
        // In production: self.ticket_store.revoke_ticket(&request.ticket).await?;

        Ok(UmaAuthorizationResponse {
            access_token: rpt.token,
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            refresh_token: None,
            upgraded: Some(is_upgrade),
            pct: None,
        })
    }

    /// Validate permission ticket
    async fn validate_permission_ticket(
        &self,
        ticket: &str,
        realm_id: &str,
    ) -> Result<Vec<PermissionTicket>> {
        // 1. Parse ticket ID - validate UUID format
        let ticket_uuid = Uuid::parse_str(ticket).map_err(|_| {
            tracing::warn!("Invalid permission ticket format: {}", ticket);
            AuthencError::validation("Invalid permission ticket format")
        })?;

        // 2. Get ticket from database - verify existence
        let db_ticket = self
            .ticket_store
            .get_ticket(ticket_uuid)
            .await?
            .ok_or_else(|| {
                tracing::warn!("Permission ticket not found: {}", ticket_uuid);
                AuthencError::not_found("Permission ticket not found")
            })?;

        // 3. Parse realm ID for comparison
        let realm_uuid = Uuid::parse_str(realm_id).map_err(|_| {
            tracing::warn!("Invalid realm ID format: {}", realm_id);
            AuthencError::validation("Invalid realm ID format")
        })?;

        // 4. Verify ticket belongs to this realm - prevent cross-realm attacks
        if db_ticket.realm_id != realm_uuid {
            tracing::warn!(
                "Permission ticket {} does not belong to realm {}",
                ticket_uuid,
                realm_id
            );
            return Err(AuthencError::forbidden(
                "Permission ticket does not belong to this realm",
            ));
        }

        // 5. Check if ticket is expired - enforce time-bound tickets
        let ticket_age = Utc::now().timestamp() - db_ticket.created_at.timestamp();
        if ticket_age > self.ticket_lifetime {
            tracing::warn!(
                "Permission ticket {} has expired (age: {}s, lifetime: {}s)",
                ticket_uuid,
                ticket_age,
                self.ticket_lifetime
            );
            return Err(AuthencError::forbidden("Permission ticket has expired"));
        }

        // 6. Check if ticket has already been granted/used - prevent reuse
        if db_ticket.granted {
            tracing::warn!("Permission ticket {} has already been used", ticket_uuid);
            return Err(AuthencError::forbidden(
                "Permission ticket has already been used",
            ));
        }

        // All validation passed
        tracing::info!(
            "Permission ticket {} validated successfully for realm {}",
            ticket_uuid,
            realm_id
        );

        Ok(vec![db_ticket])
    }

    /// Get resources from permission ticket
    async fn get_resources_from_ticket(
        &self,
        tickets: &[PermissionTicket],
    ) -> Result<Vec<Resource>> {
        let mut resources = Vec::new();

        for ticket in tickets {
            if let Some(resource) = self.resource_store.get_resource(ticket.resource_id).await? {
                resources.push(resource);
            }
        }

        Ok(resources)
    }

    /// Build policy evaluation context
    fn build_evaluation_context(
        &self,
        subject_id: &str,
        client_id: &str,
        resources: &[Resource],
        subject_attributes: HashMap<String, serde_json::Value>,
        environment: EnvironmentContext,
        _claim_token: Option<String>,
    ) -> Result<PolicyEvaluationContext> {
        // In production, would load user roles, groups, etc. from database
        let subject = SubjectContext {
            user_id: subject_id.to_string(),
            roles: vec![], // TODO: Load from database
            attributes: subject_attributes,
            groups: vec![], // TODO: Load from database
        };

        // For simplicity, use first resource for context
        // In production, evaluate each resource separately
        let resource_context = if let Some(resource) = resources.first() {
            ResourceContext {
                id: resource.id.to_string(),
                resource_type: resource.resource_type.clone(),
                owner: resource.owner.clone(),
                attributes: resource.attributes.clone(),
                scopes: resource.scopes.clone(),
            }
        } else {
            return Err(AuthencError::validation("No resources in ticket"));
        };

        Ok(PolicyEvaluationContext {
            subject,
            resource: resource_context,
            action: "access".to_string(),
            environment,
            claims: HashMap::new(), // TODO: Parse claim_token if provided
        })
    }

    /// Load policies for resource
    async fn load_policies_for_resource(
        &self,
        resource_id: &Uuid,
    ) -> Result<Vec<super::policy_engine::UmaPolicy>> {
        // Load policies from database using UmaPolicyStore
        self.uma_policy_store
            .get_policies_for_resource(resource_id)
            .await
    }
}

/// Helper to create authorization context from HTTP request
pub struct AuthorizationContextBuilder {
    subject_id: Option<String>,
    client_id: Option<String>,
    ip_address: Option<String>,
    user_agent: Option<String>,
    device_id: Option<String>,
    location: Option<String>,
    mfa_completed: bool,
    trust_score: Option<f64>,
    attributes: HashMap<String, serde_json::Value>,
}

impl AuthorizationContextBuilder {
/// Fungsi `new(`.
    pub fn new() -> Self {
        Self {
            subject_id: None,
            client_id: None,
            ip_address: None,
            user_agent: None,
            device_id: None,
            location: None,
            mfa_completed: false,
            trust_score: None,
            attributes: HashMap::new(),
        }
    }
/// Fungsi `subject_id(mut`.

    pub fn subject_id(mut self, subject_id: String) -> Self {
        self.subject_id = Some(subject_id);
        self
/// Fungsi `client_id(mut`.
    }

/// Fungsi `client_id(mut`.
    pub fn client_id(mut self, client_id: String) -> Self {
        self.client_id = Some(client_id);
/// Fungsi `ip_address(mut`.
        self
    }
/// Fungsi `ip_address(mut`.

    pub fn ip_address(mut self, ip: String) -> Self {
/// Fungsi `user_agent(mut`.
        self.ip_address = Some(ip);
        self
    }

/// Fungsi `mfa_completed(mut`.
    pub fn user_agent(mut self, ua: String) -> Self {
        self.user_agent = Some(ua);
        self
    }
/// Fungsi `trust_score(mut`.

    pub fn mfa_completed(mut self, completed: bool) -> Self {
        self.mfa_completed = completed;
        self
/// Fungsi `attribute(mut`.
    }

    pub fn trust_score(mut self, score: f64) -> Self {
        self.trust_score = Some(score);
/// Fungsi `build(`.
        self
/// Fungsi `attribute(mut`.
    }

    pub fn attribute(mut self, key: String, value: serde_json::Value) -> Self {
        self.attributes.insert(key, value);
/// Fungsi `build(`.
        self
    }

    pub fn build(
        self,
    ) -> Result<(
        String,
        String,
        HashMap<String, serde_json::Value>,
        EnvironmentContext,
    )> {
        let subject_id = self
            .subject_id
            .ok_or_else(|| AuthencError::missing_field("subject_id"))?;

        let client_id = self
            .client_id
            .ok_or_else(|| AuthencError::missing_field("client_id"))?;

        let environment = EnvironmentContext {
            ip_address: self.ip_address,
            time: Utc::now().timestamp(),
            user_agent: self.user_agent,
            device_id: self.device_id,
            location: self.location,
            mfa_completed: self.mfa_completed,
            trust_score: self.trust_score,
        };

        Ok((subject_id, client_id, self.attributes, environment))
    }
}

impl Default for AuthorizationContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_builder() {
        let result = AuthorizationContextBuilder::new()
            .subject_id("user123".to_string())
            .client_id("client456".to_string())
            .ip_address("192.168.1.1".to_string())
            .mfa_completed(true)
            .trust_score(0.95)
            .attribute("department".to_string(), serde_json::json!("engineering"))
            .build();

        assert!(result.is_ok());

        let (subject_id, client_id, attributes, environment) = result.unwrap();
        assert_eq!(subject_id, "user123");
        assert_eq!(client_id, "client456");
        assert_eq!(environment.ip_address, Some("192.168.1.1".to_string()));
        assert!(environment.mfa_completed);
        assert_eq!(environment.trust_score, Some(0.95));
        assert_eq!(
            attributes.get("department"),
            Some(&serde_json::json!("engineering"))
        );
    }

    #[test]
    fn test_context_builder_missing_required() {
        let result = AuthorizationContextBuilder::new()
            .subject_id("user123".to_string())
            // Missing client_id
            .build();

        assert!(result.is_err());
    }
}
