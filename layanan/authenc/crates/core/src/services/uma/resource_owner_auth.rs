//! UMA 2.0 Resource Owner Authorization
//!
//! Handles resource owner authorization and delegation flows.
//! Allows resource owners to grant/deny access to their resources.

use crate::services::permission_ticket_store::PermissionTicketStoreTrait;
use crate::services::resource_store::ResourceStoreTrait;
use authenc_types::{AuthencError, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Authorization decision by resource owner
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AuthorizationDecision {
    /// Grant access
    Grant,
    /// Deny access
    Deny,
    /// Defer decision (no action)
    Defer,
}

/// Resource owner authorization request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceOwnerAuthorizationRequest {
    /// Permission ticket ID
    pub ticket_id: String,
    /// Resource ID
    pub resource_id: String,
    /// Requested scopes
    pub scopes: Vec<String>,
    /// Requesting party ID
    pub requesting_party: String,
    /// Optional justification from requesting party
    pub justification: Option<String>,
}

/// Resource owner authorization response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceOwnerAuthorizationResponse {
    /// Ticket ID
    pub ticket_id: String,
    /// Authorization decision
    pub decision: AuthorizationDecision,
    /// Optional reason for decision
    pub reason: Option<String>,
    /// Timestamp of decision
    pub decided_at: i64,
}

/// Delegation policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationPolicy {
    /// Policy ID
    pub id: Uuid,
    /// Resource owner ID
    pub owner_id: String,
    /// Resource ID (None means all resources owned by this owner)
    pub resource_id: Option<String>,
    /// Scopes covered by this policy
    pub scopes: Vec<String>,
    /// Delegates (users who can access)
    pub delegates: Vec<String>,
    /// Conditions for delegation
    pub conditions: DelegationConditions,
    /// Whether policy is active
    pub enabled: bool,
    /// Valid from timestamp
    pub valid_from: Option<i64>,
    /// Valid until timestamp
    pub valid_until: Option<i64>,
    /// Realm ID
    pub realm_id: String,
}

/// Conditions for delegation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationConditions {
    /// Require MFA
    pub require_mfa: bool,
    /// Allowed IP addresses
    pub allowed_ips: Option<Vec<String>>,
    /// Allowed times (cron expression)
    pub allowed_times: Option<String>,
    /// Maximum uses (None = unlimited)
    pub max_uses: Option<u32>,
    /// Minimum trust score
    pub min_trust_score: Option<f64>,
}

/// Resource owner authorization service
pub struct ResourceOwnerAuthService {
    resource_store: Arc<dyn ResourceStoreTrait>,
    ticket_store: Arc<dyn PermissionTicketStoreTrait>,
}

impl ResourceOwnerAuthService {
    /// Create new resource owner authorization service
    pub fn new(
        resource_store: Arc<dyn ResourceStoreTrait>,
        ticket_store: Arc<dyn PermissionTicketStoreTrait>,
    ) -> Self {
        Self {
            resource_store,
            ticket_store,
        }
    }

    /// Get pending authorization requests for resource owner
    pub async fn get_pending_requests(
        &self,
        owner_id: &str,
        realm_id: &str,
    ) -> Result<Vec<ResourceOwnerAuthorizationRequest>> {
        // Get resources owned by this user
        let resources = self
            .resource_store
            .get_resources_by_owner(owner_id, None, None)
            .await?;

        let mut requests = Vec::new();

        // Parse realm_id to UUID for comparison with resource.realm_id
        let realm_uuid = Uuid::parse_str(realm_id)
            .map_err(|_| AuthencError::validation("Invalid realm ID format"))?;

        // Get ungranted permission tickets for these resources
        for resource in resources {
            if resource.realm_id != realm_uuid {
                continue;
            }

            // Get tickets for this resource
            let tickets = self
                .ticket_store
                .get_tickets_for_resource(resource.id, None)
                .await?;

            for ticket in tickets {
                if !ticket.is_granted() {
                    requests.push(ResourceOwnerAuthorizationRequest {
                        ticket_id: ticket.id.to_string(),
                        resource_id: resource.id.to_string(),
                        scopes: vec![ticket.scope_id.to_string()], // In production, resolve scope names
                        requesting_party: ticket.requester.clone(),
                        justification: None, // Would be stored with ticket
                    });
                }
            }
        }

        Ok(requests)
    }

    /// Authorize or deny access request
    pub async fn authorize_request(
        &self,
        owner_id: &str,
        ticket_id: &str,
        decision: AuthorizationDecision,
        reason: Option<String>,
    ) -> Result<ResourceOwnerAuthorizationResponse> {
        let ticket_uuid = Uuid::parse_str(ticket_id)
            .map_err(|_| AuthencError::validation("Invalid ticket ID format"))?;

        // Get ticket
        let ticket = self
            .ticket_store
            .get_ticket(ticket_uuid)
            .await?
            .ok_or_else(|| {
                AuthencError::resource_not_found("Permission ticket not found".to_string())
            })?;

        // Verify owner
        if ticket.owner != owner_id {
            return Err(AuthencError::forbidden(
                "You are not the owner of this resource",
            ));
        }

        // Apply decision
        match decision {
            AuthorizationDecision::Grant => {
                self.ticket_store.grant_ticket(ticket_uuid).await?;
            }
            AuthorizationDecision::Deny => {
                self.ticket_store.revoke_ticket(ticket_uuid).await?;
            }
            AuthorizationDecision::Defer => {
                // No action - keep ticket in pending state
            }
        }

        Ok(ResourceOwnerAuthorizationResponse {
            ticket_id: ticket_id.to_string(),
            decision,
            reason,
            decided_at: Utc::now().timestamp(),
        })
    }

    /// Create delegation policy
    pub async fn create_delegation_policy(
        &self,
        owner_id: &str,
        resource_id: Option<String>,
        scopes: Vec<String>,
        delegates: Vec<String>,
        conditions: DelegationConditions,
        realm_id: &str,
    ) -> Result<DelegationPolicy> {
        // Verify resource ownership if resource_id specified
        if let Some(ref res_id) = resource_id {
            let resource_uuid = Uuid::parse_str(res_id)
                .map_err(|_| AuthencError::validation("Invalid resource ID format"))?;
            let resource = self
                .resource_store
                .get_resource(resource_uuid)
                .await?
                .ok_or_else(|| {
                    AuthencError::resource_not_found("Resource not found".to_string())
                })?;

            if resource.owner != owner_id {
                return Err(AuthencError::forbidden("You do not own this resource"));
            }
        }

        let policy = DelegationPolicy {
            id: Uuid::new_v4(),
            owner_id: owner_id.to_string(),
            resource_id,
            scopes,
            delegates,
            conditions,
            enabled: true,
            valid_from: None,
            valid_until: None,
            realm_id: realm_id.to_string(),
        };

        // In production, store policy in database
        // self.db.create_delegation_policy(&policy).await?;

        Ok(policy)
    }

    /// Check if delegation policy allows access
    pub fn check_delegation_policy(
        &self,
        policy: &DelegationPolicy,
        requesting_party: &str,
        resource_id: &str,
        scopes: &[String],
        mfa_completed: bool,
        ip_address: Option<&str>,
        trust_score: Option<f64>,
    ) -> Result<bool> {
        // Check if policy is enabled
        if !policy.enabled {
            return Ok(false);
        }

        // Check resource match
        if let Some(ref policy_resource_id) = policy.resource_id {
            if policy_resource_id != resource_id {
                return Ok(false);
            }
        }

        // Check if requesting party is a delegate
        if !policy.delegates.contains(&requesting_party.to_string()) {
            return Ok(false);
        }

        // Check scopes
        let all_scopes_allowed = scopes.iter().all(|scope| policy.scopes.contains(scope));

        if !all_scopes_allowed {
            return Ok(false);
        }

        // Check validity period
        let now = Utc::now().timestamp();

        if let Some(valid_from) = policy.valid_from {
            if now < valid_from {
                return Ok(false);
            }
        }

        if let Some(valid_until) = policy.valid_until {
            if now > valid_until {
                return Ok(false);
            }
        }

        // Check conditions
        if policy.conditions.require_mfa && !mfa_completed {
            return Ok(false);
        }

        if let Some(ref allowed_ips) = policy.conditions.allowed_ips {
            if let Some(ip) = ip_address {
                if !allowed_ips.contains(&ip.to_string()) {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }
        }

        if let Some(min_trust) = policy.conditions.min_trust_score {
            if let Some(score) = trust_score {
                if score < min_trust {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Revoke delegation policy
    pub async fn revoke_delegation_policy(&self, owner_id: &str, policy_id: &Uuid) -> Result<()> {
        // In production:
        // 1. Load policy from database
        // 2. Verify owner_id matches
        // 3. Mark as disabled or delete

        // For now, return Ok
        Ok(())
    }

    /// Get delegation policies for owner
    pub async fn get_delegation_policies(
        &self,
        owner_id: &str,
        realm_id: &str,
    ) -> Result<Vec<DelegationPolicy>> {
        // In production, load from database
        // For now, return empty vec
        Ok(Vec::new())
    }
}
