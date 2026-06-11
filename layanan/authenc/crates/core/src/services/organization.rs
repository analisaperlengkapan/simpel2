//! Organization service for multi-tenancy

use authenc_storage::Database;
use authenc_storage::operations::organizations;
use authenc_types::domain::{Organization, OrganizationMember};
use authenc_types::{AuthencError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

/// Organization roles
///
/// # Deprecation Notice
/// This enum is deprecated in favor of dynamic role types from the database.
/// Use `dynamic_role::RoleType` with `DynamicRoleStore` for new code.
///
/// Migration: `dynamic_role::migration::map_organization_role(role.as_str())`
#[deprecated(
    since = "2.0.0",
    note = "Use dynamic role types from DynamicRoleStore instead"
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrganizationRole {
    /// Owner of the organization with full administrative privileges
    Owner,
    /// Administrator with elevated privileges
    Admin,
    /// Regular member with standard access
    Member,
    /// Guest with limited access
    Guest,
}

#[allow(deprecated)]
impl OrganizationRole {
    /// Convert the role to its string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            OrganizationRole::Owner => "OWNER",
            OrganizationRole::Admin => "ADMIN",
            OrganizationRole::Member => "MEMBER",
            OrganizationRole::Guest => "GUEST",
        }
    }

    /// Convert to dynamic role code
    pub fn to_dynamic_role_code(&self) -> &'static str {
        match self {
            OrganizationRole::Owner => "org-owner",
            OrganizationRole::Admin => "org-admin",
            OrganizationRole::Member => "org-member",
            OrganizationRole::Guest => "org-guest",
        }
    }
}

#[allow(deprecated)]
impl FromStr for OrganizationRole {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "OWNER" => Ok(OrganizationRole::Owner),
            "ADMIN" => Ok(OrganizationRole::Admin),
            "MEMBER" => Ok(OrganizationRole::Member),
            "GUEST" => Ok(OrganizationRole::Guest),
            _ => Err(()),
        }
    }
}

/// Organization invitation
// `role: OrganizationRole` kept pending migration to DynamicRoleStore.
#[allow(deprecated)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationInvitation {
    /// Unique identifier for the invitation
    pub id: Uuid,
    /// ID of the organization being invited to
    pub organization_id: Uuid,
    /// Email address of the invited user
    pub email: String,
    /// Role to be assigned to the user upon acceptance
    pub role: OrganizationRole,
    /// ID of the user who sent the invitation
    pub invited_by: Uuid,
    /// Timestamp when the invitation was sent
    pub invited_at: chrono::DateTime<chrono::Utc>,
    /// Timestamp when the invitation expires
    pub expires_at: chrono::DateTime<chrono::Utc>,
    /// Timestamp when the invitation was accepted
    pub accepted_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Unique token for the invitation
    pub token: String,
}

/// Organization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationSettings {
    /// ID of the organization these settings apply to
    pub organization_id: Uuid,
    /// Whether public signup is allowed for this organization
    pub allow_public_signup: bool,
    /// Whether email verification is required for new users
    pub require_email_verification: bool,
    /// Whether two-factor authentication is enabled
    pub enable_two_factor: bool,
    /// Password policy rules for the organization
    pub password_policy: String,
    /// Session timeout in seconds
    pub session_timeout: u64,
    /// Maximum number of users allowed in the organization
    pub max_users: Option<u32>,
    /// List of enabled features for the organization
    pub features: Vec<String>,
}

/// Organization service for multi-tenancy
pub struct OrganizationService {
    #[allow(dead_code)]
    db: Arc<Database>,
}

impl OrganizationService {
    /// Create new organization service
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Create a new organization
    // Uses OrganizationRole::Owner; kept pending migration to DynamicRoleStore.
    #[allow(deprecated)]
    pub async fn create_organization(
        &self,
        name: &str,
        display_name: &str,
        description: Option<&str>,
        created_by: Uuid,
    ) -> Result<Organization> {
        let organization = Organization {
            id: Uuid::new_v4(),
            name: name.to_string(),
            display_name: Some(display_name.to_string()),
            description: description.map(|s| s.to_string()),
            domain: None,
            logo_url: None,
            website_url: None,
            owner_id: created_by,
            realm_id: None,
            enabled: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deleted_at: None,
        };

        // Store organization in database
        let stored_org = organizations::create_organization(&self.db, &organization).await?;

        // Add creator as owner
        organizations::add_member(
            &self.db,
            stored_org.id,
            created_by,
            OrganizationRole::Owner.as_str(),
            Some(created_by),
        )
        .await?;

        Ok(stored_org)
    }

    /// Get organization by ID
    pub async fn get_organization(&self, organization_id: &Uuid) -> Result<Option<Organization>> {
        organizations::get_organization_by_id(&self.db, *organization_id).await
    }

    /// Get organization by domain
    pub async fn get_organization_by_domain(&self, domain: &str) -> Result<Option<Organization>> {
        organizations::get_organization_by_domain(&self.db, domain).await
    }

    /// Update organization
    pub async fn update_organization(
        &self,
        organization_id: &Uuid,
        updates: &OrganizationUpdate,
    ) -> Result<()> {
        let mut org = self
            .get_organization(organization_id)
            .await?
            .ok_or_else(|| AuthencError::resource_not_found("Organization not found"))?;

        if let Some(name) = &updates.display_name {
            org.display_name = Some(name.clone());
        }
        if let Some(desc) = &updates.description {
            org.description = Some(desc.clone());
        }
        if let Some(domain) = &updates.domain {
            org.domain = Some(domain.clone());
        }
        if let Some(logo) = &updates.logo_url {
            org.logo_url = Some(logo.clone());
        }
        if let Some(web) = &updates.website {
            org.website_url = Some(web.clone());
        }
        if let Some(enabled) = updates.enabled {
            org.enabled = enabled;
        }

        org.updated_at = chrono::Utc::now();

        organizations::update_organization(&self.db, &org).await
    }

    /// Delete organization
    pub async fn delete_organization(&self, organization_id: &Uuid) -> Result<()> {
        organizations::delete_organization(&self.db, organization_id).await
    }

    /// Add member to organization
    #[allow(deprecated)]
    pub async fn add_member(
        &self,
        organization_id: &Uuid,
        user_id: &Uuid,
        role: OrganizationRole,
        invited_by: Option<Uuid>,
    ) -> Result<()> {
        organizations::add_member(
            &self.db,
            *organization_id,
            *user_id,
            role.as_str(),
            invited_by,
        )
        .await
    }

    /// Remove member from organization
    pub async fn remove_member(&self, organization_id: &Uuid, user_id: &Uuid) -> Result<()> {
        organizations::remove_organization_member(&self.db, organization_id, user_id).await
    }

    /// Update member role
    #[allow(deprecated)]
    pub async fn update_member_role(
        &self,
        organization_id: &Uuid,
        user_id: &Uuid,
        new_role: OrganizationRole,
    ) -> Result<()> {
        organizations::update_member_role(&self.db, organization_id, user_id, new_role.as_str())
            .await
    }

    /// Get organization members
    pub async fn get_members(&self, organization_id: &Uuid) -> Result<Vec<OrganizationMember>> {
        organizations::get_organization_members(&self.db, organization_id).await
    }

    /// Check if user is member of organization
    pub async fn is_member(&self, organization_id: &Uuid, user_id: &Uuid) -> Result<bool> {
        let members = organizations::get_organization_members(&self.db, organization_id).await?;
        Ok(members.iter().any(|m| m.user_id == *user_id))
    }

    /// Check if user has role in organization
    #[allow(deprecated)]
    pub async fn has_role(
        &self,
        organization_id: &Uuid,
        user_id: &Uuid,
        role: &OrganizationRole,
    ) -> Result<bool> {
        let members = organizations::get_organization_members(&self.db, organization_id).await?;
        let required_role_str = role.as_str();
        Ok(members
            .iter()
            .any(|m| m.user_id == *user_id && m.role == required_role_str))
    }

    /// Create invitation
    #[allow(deprecated)]
    pub async fn create_invitation(
        &self,
        organization_id: &Uuid,
        email: &str,
        role: OrganizationRole,
        invited_by: Uuid,
        expires_in_days: u32,
    ) -> Result<OrganizationInvitation> {
        let token = self.generate_invitation_token();
        let now = chrono::Utc::now();
        let expires_at = now + chrono::Duration::days(expires_in_days as i64);
        let id = Uuid::new_v4();

        let db_invitation = authenc_types::domain::organization::OrganizationInvitation {
            id,
            organization_id: *organization_id,
            email: email.to_string(),
            role: role.as_str().to_string(),
            invited_by,
            token_hash: token.clone(), // In real implementation, we would hash this
            expires_at,
            accepted_at: None,
            accepted_by: None,
            created_at: now,
        };

        organizations::create_invitation(&self.db, &db_invitation).await?;

        // Return the service model
        let service_invitation = OrganizationInvitation {
            id,
            organization_id: *organization_id,
            email: email.to_string(),
            role,
            invited_by,
            invited_at: now,
            expires_at,
            accepted_at: None,
            token, // Return raw token to user
        };

        Ok(service_invitation)
    }

    /// Accept invitation
    pub async fn accept_invitation(&self, token: &str, user_id: Uuid) -> Result<()> {
        organizations::accept_invitation(&self.db, token, user_id).await
    }

    /// Get organization settings
    pub async fn get_settings(&self, organization_id: &Uuid) -> Result<OrganizationSettings> {
        // In production, retrieve from database
        Ok(OrganizationSettings {
            organization_id: *organization_id,
            allow_public_signup: false,
            require_email_verification: true,
            enable_two_factor: true,
            password_policy: "default".to_string(),
            session_timeout: 3600,
            max_users: Some(1000),
            features: vec!["oidc".to_string(), "saml".to_string()],
        })
    }

    /// Update organization settings
    pub async fn update_settings(&self, _settings: &OrganizationSettings) -> Result<()> {
        // In production, update in database
        Ok(())
    }

    /// Get user's organizations
    pub async fn get_user_organizations(&self, user_id: &Uuid) -> Result<Vec<Organization>> {
        organizations::get_user_organizations(&self.db, user_id).await
    }

    /// Transfer organization ownership
    #[allow(deprecated)]
    pub async fn transfer_ownership(
        &self,
        organization_id: &Uuid,
        current_owner: &Uuid,
        new_owner: &Uuid,
    ) -> Result<()> {
        // Verify current user is owner
        if !self
            .has_role(organization_id, current_owner, &OrganizationRole::Owner)
            .await?
        {
            return Err(AuthencError::forbidden("Only owner can transfer ownership"));
        }

        // Update member roles
        self.update_member_role(organization_id, current_owner, OrganizationRole::Admin)
            .await?;
        self.update_member_role(organization_id, new_owner, OrganizationRole::Owner)
            .await?;

        Ok(())
    }

    /// Add domain to organization for verification
    pub async fn add_domain(
        &self,
        organization_id: &Uuid,
        domain: &str,
        verification_method: &str,
    ) -> Result<()> {
        organizations::add_domain(&self.db, *organization_id, domain, verification_method).await?;
        Ok(())
    }

    /// Verify organization domain
    pub async fn verify_domain(&self, domain_id: &Uuid) -> Result<()> {
        organizations::verify_domain(&self.db, *domain_id).await
    }

    /// Get organization domains
    pub async fn get_domains(&self, organization_id: &Uuid) -> Result<Vec<String>> {
        let domains = organizations::get_domains(&self.db, *organization_id).await?;
        Ok(domains.into_iter().map(|d| d.domain).collect())
    }

    /// Link identity provider to organization
    pub async fn link_identity_provider(
        &self,
        organization_id: &Uuid,
        identity_provider_id: &Uuid,
        priority: i32,
    ) -> Result<()> {
        organizations::link_identity_provider(
            &self.db,
            *organization_id,
            *identity_provider_id,
            priority,
        )
        .await
    }

    /// Unlink identity provider from organization
    pub async fn unlink_identity_provider(
        &self,
        organization_id: &Uuid,
        identity_provider_id: &Uuid,
    ) -> Result<()> {
        organizations::unlink_identity_provider(&self.db, *organization_id, *identity_provider_id)
            .await
    }

    /// Generate secure invitation token
    fn generate_invitation_token(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let token: String = (0..32)
            .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
            .collect();
        token
    }
}

/// Organization update request
#[derive(Debug, Serialize, Deserialize)]
pub struct OrganizationUpdate {
    /// New display name for the organization
    pub display_name: Option<String>,
    /// New description for the organization
    pub description: Option<String>,
    /// New domain for the organization
    pub domain: Option<String>,
    /// New logo URL for the organization
    pub logo_url: Option<String>,
    /// New website URL for the organization
    pub website: Option<String>,
    /// Whether the organization should be enabled
    pub enabled: Option<bool>,
    /// New attributes for the organization
    pub attributes: Option<HashMap<String, String>>,
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_create_organization() {
        // This would need a test database setup
        // let db = Arc::new(Database::new_test().await);
        // let service = OrganizationService::new(db);
        // let org = service.create_organization("test", "Test Org", Some("Test"), Uuid::new_v4()).await;
        // assert!(org.is_ok());
    }
}
