//! Realm management service implementation
//!
//! This module implements realm management operations, including:
//! - Realm CRUD operations
//! - Realm configuration management
//! - Realm-specific policies
//! - Multi-tenant isolation

use std::sync::Arc;
use tracing::{debug, info, warn};

use authenc_types::{
    domain::*, domain_types::RealmId, error::AuthencError, result::Result, traits::*,
};

/// Implementation of the realm management service
///
/// This service provides CRUD operations for realms with:
/// - Input validation (realm name format, uniqueness)
/// - Multi-tenant isolation
/// - Realm configuration management
/// - Realm-specific policies
pub struct RealmManagementServiceImpl {
    /// Realm storage for CRUD operations
    realm_store: Arc<dyn RealmStore>,
}

impl RealmManagementServiceImpl {
    /// Create a new realm management service
    ///
    /// # Arguments
    ///
    /// * `realm_store` - Realm storage implementation
    pub fn new(realm_store: Arc<dyn RealmStore>) -> Self {
        Self { realm_store }
    }

    /// Validate realm name format
    ///
    /// Requirements:
    /// - Minimum 3 characters
    /// - Maximum 64 characters
    /// - Only lowercase alphanumeric, underscore, and hyphen
    /// - Must start with a letter
    ///
    /// # Arguments
    ///
    /// * `name` - Realm name to validate
    ///
    /// # Returns
    ///
    /// `Ok(())` if valid, `Err` if invalid
    fn validate_realm_name(&self, name: &str) -> Result<()> {
        if name.len() < 3 {
            return Err(AuthencError::ValidationError(
                "Realm name must be at least 3 characters".to_string(),
            ));
        }

        if name.len() > 64 {
            return Err(AuthencError::ValidationError(
                "Realm name must be at most 64 characters".to_string(),
            ));
        }

        // Must start with a letter
        if !name
            .chars()
            .next()
            .map_or(false, |c| c.is_ascii_lowercase())
        {
            return Err(AuthencError::ValidationError(
                "Realm name must start with a lowercase letter".to_string(),
            ));
        }

        // Only lowercase alphanumeric, underscore, and hyphen
        if !name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
        {
            return Err(AuthencError::ValidationError(
                "Realm name can only contain lowercase letters, digits, underscore, and hyphen"
                    .to_string(),
            ));
        }

        Ok(())
    }

    /// Validate display name format
    ///
    /// Requirements:
    /// - Minimum 1 character
    /// - Maximum 128 characters
    ///
    /// # Arguments
    ///
    /// * `display_name` - Display name to validate
    ///
    /// # Returns
    ///
    /// `Ok(())` if valid, `Err` if invalid
    fn validate_display_name(&self, display_name: &str) -> Result<()> {
        if display_name.is_empty() {
            return Err(AuthencError::ValidationError(
                "Display name cannot be empty".to_string(),
            ));
        }

        if display_name.len() > 128 {
            return Err(AuthencError::ValidationError(
                "Display name must be at most 128 characters".to_string(),
            ));
        }

        Ok(())
    }

    /// Create a new realm
    ///
    /// This method:
    /// 1. Validates realm name format
    /// 2. Validates display name format
    /// 3. Checks realm name uniqueness
    /// 4. Creates realm in database
    ///
    /// # Arguments
    ///
    /// * `name` - Unique realm name (lowercase, alphanumeric, underscore, hyphen)
    /// * `display_name` - Human-readable display name
    ///
    /// # Returns
    ///
    /// The created realm
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails or realm name already exists
    ///
    /// # Requirements
    ///
    /// - REQ-REALM-001: Multi-realm architecture support
    /// - REQ-REALM-002: Realm CRUD operations
    pub async fn create_realm(&self, name: String, display_name: String) -> Result<Realm> {
        debug!(
            name = %name,
            display_name = %display_name,
            "Creating realm"
        );

        // Step 1: Validate realm name
        self.validate_realm_name(&name)?;

        // Step 2: Validate display name
        self.validate_display_name(&display_name)?;

        // Step 3: Check realm name uniqueness
        if self.realm_store.realm_name_exists(&name).await? {
            warn!(
                name = %name,
                "Realm name already exists"
            );
            return Err(AuthencError::Conflict(format!(
                "Realm name '{}' already exists",
                name
            )));
        }

        // Step 4: Create realm in database
        let realm = self
            .realm_store
            .create_realm(name.clone(), display_name)
            .await?;

        info!(
            realm_id = %realm.id,
            name = %realm.name,
            "Realm created successfully"
        );

        Ok(realm)
    }

    /// Get a realm by ID
    ///
    /// # Arguments
    ///
    /// * `realm_id` - Realm ID
    ///
    /// # Returns
    ///
    /// The realm
    ///
    /// # Requirements
    ///
    /// - REQ-REALM-001: Multi-realm architecture support
    pub async fn get_realm(&self, realm_id: RealmId) -> Result<Realm> {
        debug!(realm_id = %realm_id, "Getting realm");
        self.realm_store.get_realm(realm_id).await
    }

    /// Get a realm by name
    ///
    /// # Arguments
    ///
    /// * `name` - Realm name
    ///
    /// # Returns
    ///
    /// The realm
    ///
    /// # Requirements
    ///
    /// - REQ-REALM-001: Multi-realm architecture support
    pub async fn get_realm_by_name(&self, name: &str) -> Result<Realm> {
        debug!(name = %name, "Getting realm by name");
        self.realm_store.get_realm_by_name(name).await
    }

    /// Update a realm
    ///
    /// This method:
    /// 1. Validates new display name (if provided)
    /// 2. Updates realm in database
    ///
    /// # Arguments
    ///
    /// * `realm_id` - Realm ID
    /// * `display_name` - New display name (optional)
    /// * `enabled` - New enabled status (optional)
    ///
    /// # Returns
    ///
    /// The updated realm
    ///
    /// # Requirements
    ///
    /// - REQ-REALM-002: Realm CRUD operations
    /// - REQ-REALM-003: Realm configuration management
    pub async fn update_realm(
        &self,
        realm_id: RealmId,
        display_name: Option<String>,
        enabled: Option<bool>,
    ) -> Result<Realm> {
        debug!(
            realm_id = %realm_id,
            display_name = ?display_name,
            enabled = ?enabled,
            "Updating realm"
        );

        // Prevent disabling the master realm
        if *realm_id.as_uuid() == Realm::MASTER_ID {
            if let Some(false) = enabled {
                warn!("Attempt to disable the master realm was rejected");
                return Err(AuthencError::AuthorizationFailed(
                    "Cannot disable the master realm".to_string(),
                ));
            }
        }

        // Step 1: Validate new display name if provided
        if let Some(ref display_name) = display_name {
            self.validate_display_name(display_name)?;
        }

        // Step 2: Update realm in database
        let realm = self
            .realm_store
            .update_realm(realm_id, display_name, enabled)
            .await?;

        info!(
            realm_id = %realm.id,
            name = %realm.name,
            "Realm updated successfully"
        );

        Ok(realm)
    }

    /// Delete a realm
    ///
    /// This method performs a hard delete of the realm.
    ///
    /// **Warning**: This will delete the realm and all associated data (users, clients, sessions).
    /// In production, you should:
    /// 1. Check if realm has users/clients before deleting
    /// 2. Implement soft delete (set enabled=false) instead
    /// 3. Cascade delete related entities
    ///
    /// # Arguments
    ///
    /// * `realm_id` - Realm ID
    ///
    /// # Requirements
    ///
    /// - REQ-REALM-002: Realm CRUD operations
    pub async fn delete_realm(&self, realm_id: RealmId) -> Result<()> {
        debug!(realm_id = %realm_id, "Deleting realm");

        // Prevent deletion of the master realm
        if *realm_id.as_uuid() == Realm::MASTER_ID {
            warn!("Attempt to delete the master realm was rejected");
            return Err(AuthencError::AuthorizationFailed(
                "Cannot delete the master realm".to_string(),
            ));
        }

        // TODO: Add checks for existing users/clients before deletion
        // TODO: Consider implementing soft delete instead

        self.realm_store.delete_realm(realm_id).await?;

        info!(realm_id = %realm_id, "Realm deleted successfully");

        Ok(())
    }

    /// List all realms
    ///
    /// # Returns
    ///
    /// List of all realms
    ///
    /// # Requirements
    ///
    /// - REQ-REALM-001: Multi-realm architecture support
    pub async fn list_realms(&self) -> Result<Vec<Realm>> {
        debug!("Listing all realms");
        self.realm_store.list_realms().await
    }

    /// Enable a realm
    ///
    /// This is a convenience method that sets enabled=true.
    ///
    /// # Arguments
    ///
    /// * `realm_id` - Realm ID
    ///
    /// # Returns
    ///
    /// The updated realm
    ///
    /// # Requirements
    ///
    /// - REQ-REALM-003: Realm configuration management
    pub async fn enable_realm(&self, realm_id: RealmId) -> Result<Realm> {
        debug!(realm_id = %realm_id, "Enabling realm");
        self.update_realm(realm_id, None, Some(true)).await
    }

    /// Disable a realm
    ///
    /// This is a convenience method that sets enabled=false.
    /// Disabling a realm prevents users from authenticating in that realm.
    ///
    /// # Arguments
    ///
    /// * `realm_id` - Realm ID
    ///
    /// # Returns
    ///
    /// The updated realm
    ///
    /// # Requirements
    ///
    /// - REQ-REALM-003: Realm configuration management
    pub async fn disable_realm(&self, realm_id: RealmId) -> Result<Realm> {
        debug!(realm_id = %realm_id, "Disabling realm");
        self.update_realm(realm_id, None, Some(false)).await
    }

    /// Check if a realm is enabled
    ///
    /// # Arguments
    ///
    /// * `realm_id` - Realm ID
    ///
    /// # Returns
    ///
    /// `true` if the realm is enabled, `false` otherwise
    ///
    /// # Requirements
    ///
    /// - REQ-REALM-003: Realm configuration management
    pub async fn is_realm_enabled(&self, realm_id: RealmId) -> Result<bool> {
        let realm = self.get_realm(realm_id).await?;
        Ok(realm.enabled)
    }
}
