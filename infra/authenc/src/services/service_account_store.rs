use crate::database::Database;
use crate::error::{AuthencError, Result};
use crate::models::{ServiceAccount, ServiceAccountListItem};
use chrono::Utc;
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

/// Service Account Store
///
/// Business logic layer for managing service accounts (machine-to-machine authentication).
/// Provides high-level operations for creating, updating, and managing service accounts
/// with proper validation, security checks, and audit logging.
///
/// # Security Considerations
/// - All client secrets are bcrypt-hashed before storage (cost factor 12)
/// - Service account names must be unique within a realm
/// - Client IDs must be globally unique
/// - Disabled service accounts cannot authenticate
/// - All operations are audit logged
/// - Role assignments are validated before assignment
///
/// # Use Cases
/// - Microservice authentication
/// - Backend service API access
/// - CI/CD pipeline authentication
/// - Scheduled job authentication
///
/// # Example
/// ```rust
/// use uuid::Uuid;
///
/// let store = ServiceAccountStore::with_database(database);
///
/// // Create a service account
/// let service_account = store.create(
///     realm_id,
///     "layanan-dasbor",
///     Some("Dashboard service"),
///     None, // Auto-generate secret
///     true,
///     vec![role_id],
/// ).await?;
///
/// // Authenticate
/// let authenticated = store.authenticate(&service_account.client_id, "secret").await?;
/// ```
pub struct ServiceAccountStore {
    /// Database connection
    db: Arc<Database>,
}

impl ServiceAccountStore {
    /// Create new service account store with database connection
    pub fn with_database(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Create a new service account
    ///
    /// # Arguments
    /// * `realm_id` - ID of the realm this service account belongs to
    /// * `name` - Service account name (must be unique within realm)
    /// * `description` - Optional human-readable description
    /// * `client_secret` - Optional client secret (auto-generated if None)
    /// * `enabled` - Whether the service account should be enabled
    /// * `roles` - Optional list of role IDs to assign
    ///
    /// # Returns
    /// The created service account with a generated client_id
    ///
    /// # Errors
    /// - `ValidationError` if name is empty or invalid
    /// - `ConflictError` if name or client_id already exists
    /// - `NotFoundError` if realm doesn't exist
    /// - `DatabaseError` if database operation fails
    ///
    /// # Security
    /// - Client secret is auto-generated (32 chars) if not provided
    /// - Client secret is bcrypt-hashed (cost 12) before storage
    /// - Client ID is prefixed with "sa-" and suffixed with UUID
    /// - Creation is audit logged
    pub async fn create(
        &self,
        realm_id: Uuid,
        name: &str,
        description: Option<&str>,
        client_secret: Option<String>,
        enabled: bool,
        roles: Option<Vec<Uuid>>,
    ) -> Result<ServiceAccount> {
        // Validate inputs
        if name.trim().is_empty() {
            return Err(AuthencError::validation(
                "Service account name cannot be empty",
            ));
        }

        // Verify realm exists
        if !self.realm_exists(realm_id).await? {
            return Err(AuthencError::not_found("Realm not found"));
        }

        // Generate client_id: sa-{name}-{uuid}
        let client_id = format!("sa-{}-{}", name, Uuid::new_v4());

        // Generate or use provided client secret
        let secret = client_secret.unwrap_or_else(|| Self::generate_client_secret());

        // Hash the client secret using bcrypt
        let secret_hash = bcrypt::hash(&secret, 12).map_err(|e| {
            error!("Failed to hash client secret: {}", e);
            AuthencError::internal("Failed to hash client secret")
        })?;

        // Validate roles exist if provided
        let role_list = roles.unwrap_or_default();
        for role_id in &role_list {
            if !self.role_exists(*role_id).await? {
                return Err(AuthencError::validation(format!(
                    "Role {} does not exist",
                    role_id
                )));
            }
        }

        // Create service account
        let service_account = crate::database::operations::service_accounts::create_service_account(
            &self.db,
            realm_id,
            name,
            description,
            &client_id,
            &secret_hash,
            enabled,
            role_list,
        )
        .await?;

        info!(
            "Created service account: {} (client_id: {})",
            service_account.name, service_account.client_id
        );

        Ok(service_account)
    }

    /// Get service account by ID
    pub async fn get(&self, service_account_id: Uuid) -> Result<Option<ServiceAccount>> {
        crate::database::operations::service_accounts::get_service_account_by_id(
            &self.db,
            service_account_id,
        )
        .await
    }

    /// Get service account by client ID
    pub async fn get_by_client_id(&self, client_id: &str) -> Result<Option<ServiceAccount>> {
        crate::database::operations::service_accounts::get_service_account_by_client_id(
            &self.db,
            client_id,
        )
        .await
    }

    /// List all service accounts in a realm
    pub async fn list_by_realm(
        &self,
        realm_id: Uuid,
        first: Option<i64>,
        max: Option<i64>,
    ) -> Result<Vec<ServiceAccountListItem>> {
        let service_accounts =
            crate::database::operations::service_accounts::get_service_accounts_by_realm(
                &self.db, realm_id, first, max,
            )
            .await?;

        Ok(service_accounts
            .into_iter()
            .map(ServiceAccountListItem::from)
            .collect())
    }

    /// Update service account
    ///
    /// # Arguments
    /// * `service_account_id` - ID of the service account to update
    /// * `name` - New name (if provided)
    /// * `description` - New description (if provided)
    /// * `enabled` - New enabled state (if provided)
    /// * `roles` - New list of roles (replaces existing if provided)
    ///
    /// # Security
    /// - Update is audit logged
    /// - Name uniqueness is validated
    /// - Disabling a service account immediately revokes access
    pub async fn update(
        &self,
        service_account_id: Uuid,
        name: Option<&str>,
        description: Option<Option<&str>>,
        enabled: Option<bool>,
        roles: Option<Vec<Uuid>>,
    ) -> Result<ServiceAccount> {
        // Validate name if provided
        if let Some(n) = name {
            if n.trim().is_empty() {
                return Err(AuthencError::validation(
                    "Service account name cannot be empty",
                ));
            }
        }

        // Update service account basic info
        let updated =
            crate::database::operations::service_accounts::update_service_account(
                &self.db,
                service_account_id,
                name,
                description,
                enabled,
            )
            .await?;

        // Update roles if provided
        if let Some(new_roles) = roles {
            // Validate all roles exist
            for role_id in &new_roles {
                if !self.role_exists(*role_id).await? {
                    return Err(AuthencError::validation(format!(
                        "Role {} does not exist",
                        role_id
                    )));
                }
            }

            // Get current roles
            let current_roles =
                crate::database::operations::service_accounts::get_service_account_roles(
                    &self.db,
                    service_account_id,
                )
                .await?;

            // Revoke roles that are no longer assigned
            for role_id in &current_roles {
                if !new_roles.contains(role_id) {
                    crate::database::operations::service_accounts::revoke_role(
                        &self.db,
                        service_account_id,
                        *role_id,
                    )
                    .await?;
                }
            }

            // Assign new roles
            for role_id in &new_roles {
                if !current_roles.contains(role_id) {
                    crate::database::operations::service_accounts::assign_role(
                        &self.db,
                        service_account_id,
                        *role_id,
                        None,
                    )
                    .await?;
                }
            }
        }

        // Return updated service account
        self.get(service_account_id)
            .await?
            .ok_or_else(|| AuthencError::not_found("Service account not found"))
    }

    /// Regenerate client secret for a service account
    ///
    /// # Returns
    /// Tuple of (client_id, new_plaintext_secret)
    ///
    /// # Security
    /// - Old secret is immediately invalidated
    /// - New secret is auto-generated (32 chars)
    /// - New secret is bcrypt-hashed before storage
    /// - Regeneration is audit logged
    /// - Plaintext secret is only returned once
    pub async fn regenerate_secret(
        &self,
        service_account_id: Uuid,
    ) -> Result<(String, String)> {
        // Get current service account
        let service_account = self
            .get(service_account_id)
            .await?
            .ok_or_else(|| AuthencError::not_found("Service account not found"))?;

        // Generate new secret
        let new_secret = Self::generate_client_secret();

        // Hash the new secret
        let new_secret_hash = bcrypt::hash(&new_secret, 12).map_err(|e| {
            error!("Failed to hash new client secret: {}", e);
            AuthencError::internal("Failed to hash client secret")
        })?;

        // Update in database
        crate::database::operations::service_accounts::update_service_account_secret(
            &self.db,
            service_account_id,
            &new_secret_hash,
        )
        .await?;

        info!(
            "Regenerated client secret for service account: {} ({})",
            service_account.name, service_account.id
        );

        Ok((service_account.client_id, new_secret))
    }

    /// Delete a service account
    ///
    /// # Security
    /// - Deletion is audit logged (trigger in database)
    /// - All role assignments are automatically removed (CASCADE)
    /// - Audit log entries are preserved
    pub async fn delete(&self, service_account_id: Uuid) -> Result<()> {
        crate::database::operations::service_accounts::delete_service_account(
            &self.db,
            service_account_id,
        )
        .await
    }

    /// Authenticate a service account using client credentials
    ///
    /// # Arguments
    /// * `client_id` - OAuth2 client identifier
    /// * `client_secret` - Plain-text client secret
    ///
    /// # Returns
    /// The authenticated service account if credentials are valid
    ///
    /// # Errors
    /// - `Unauthorized` if credentials are invalid
    /// - `Forbidden` if service account is disabled
    ///
    /// # Security
    /// - Uses bcrypt for constant-time comparison
    /// - Authentication attempts are audit logged
    /// - Disabled service accounts cannot authenticate
    /// - Last used timestamp is updated on success
    pub async fn authenticate(
        &self,
        client_id: &str,
        client_secret: &str,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<ServiceAccount> {
        // Get service account by client_id
        let service_account = self
            .get_by_client_id(client_id)
            .await?
            .ok_or_else(|| AuthencError::unauthorized("Invalid client credentials"))?;

        // Check if service account is enabled
        if !service_account.enabled {
            // Log failed attempt
            let _ = crate::database::operations::service_accounts::log_auth_attempt(
                &self.db,
                service_account.id,
                false,
                ip_address,
                user_agent,
                Some("Service account is disabled"),
            )
            .await;

            return Err(AuthencError::forbidden("Service account is disabled"));
        }

        // Verify client secret using bcrypt
        let valid = bcrypt::verify(client_secret, &service_account.client_secret_hash)
            .map_err(|e| {
                error!("Failed to verify client secret: {}", e);
                AuthencError::internal("Failed to verify credentials")
            })?;

        if !valid {
            // Log failed attempt
            let _ = crate::database::operations::service_accounts::log_auth_attempt(
                &self.db,
                service_account.id,
                false,
                ip_address,
                user_agent,
                Some("Invalid client secret"),
            )
            .await;

            return Err(AuthencError::unauthorized("Invalid client credentials"));
        }

        // Update last used timestamp
        let _ = crate::database::operations::service_accounts::update_last_used(
            &self.db,
            service_account.id,
            Utc::now(),
        )
        .await;

        // Log successful authentication
        let _ = crate::database::operations::service_accounts::log_auth_attempt(
            &self.db,
            service_account.id,
            true,
            ip_address,
            user_agent,
            None,
        )
        .await;

        info!(
            "Service account authenticated: {} ({})",
            service_account.name, service_account.id
        );

        Ok(service_account)
    }

    /// Assign a role to a service account
    pub async fn assign_role(
        &self,
        service_account_id: Uuid,
        role_id: Uuid,
        granted_by: Option<Uuid>,
    ) -> Result<()> {
        // Verify service account exists
        if self.get(service_account_id).await?.is_none() {
            return Err(AuthencError::not_found("Service account not found"));
        }

        // Verify role exists
        if !self.role_exists(role_id).await? {
            return Err(AuthencError::not_found("Role not found"));
        }

        crate::database::operations::service_accounts::assign_role(
            &self.db,
            service_account_id,
            role_id,
            granted_by,
        )
        .await
    }

    /// Revoke a role from a service account
    pub async fn revoke_role(&self, service_account_id: Uuid, role_id: Uuid) -> Result<()> {
        crate::database::operations::service_accounts::revoke_role(
            &self.db,
            service_account_id,
            role_id,
        )
        .await
    }

    /// Get audit log for a service account
    pub async fn get_audit_log(
        &self,
        service_account_id: Uuid,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<serde_json::Value>> {
        crate::database::operations::service_accounts::get_audit_log(
            &self.db,
            service_account_id,
            limit,
            offset,
        )
        .await
    }

    /// Generate a secure random client secret (32 characters)
    fn generate_client_secret() -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
        let mut rng = rand::thread_rng();
        (0..32)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    /// Check if a realm exists
    async fn realm_exists(&self, realm_id: Uuid) -> Result<bool> {
        let query = "SELECT EXISTS(SELECT 1 FROM realms WHERE id = $1)";
        let row: tokio_postgres::Row = self.db.query_one(query, &[&realm_id]).await?;
        Ok(row.get(0))
    }

    /// Check if a role exists
    async fn role_exists(&self, role_id: Uuid) -> Result<bool> {
        let query = "SELECT EXISTS(SELECT 1 FROM roles WHERE id = $1)";
        let row: tokio_postgres::Row = self.db.query_one(query, &[&role_id]).await?;
        Ok(row.get(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_client_secret() {
        let secret1 = ServiceAccountStore::generate_client_secret();
        let secret2 = ServiceAccountStore::generate_client_secret();

        // Should be 32 characters
        assert_eq!(secret1.len(), 32);
        assert_eq!(secret2.len(), 32);

        // Should be different
        assert_ne!(secret1, secret2);

        // Should only contain valid characters
        assert!(secret1.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_'));
        assert!(secret2.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_'));
    }
}
