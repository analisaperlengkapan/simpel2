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

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use tokio::sync::Mutex;

    // Mock implementation for testing

    struct MockRealmStore {
        realms: Arc<Mutex<HashMap<RealmId, Realm>>>,
        realms_by_name: Arc<Mutex<HashMap<String, RealmId>>>,
    }

    impl MockRealmStore {
        fn new() -> Self {
            Self {
                realms: Arc::new(Mutex::new(HashMap::new())),
                realms_by_name: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    #[async_trait]
    impl RealmStore for MockRealmStore {
        async fn get_realm(&self, id: RealmId) -> Result<Realm> {
            let realms = self.realms.lock().await;
            realms
                .get(&id)
                .cloned()
                .ok_or_else(|| AuthencError::RealmNotFound(format!("Realm {} not found", id)))
        }

        async fn get_realm_by_name(&self, name: &str) -> Result<Realm> {
            let realms_by_name = self.realms_by_name.lock().await;
            let realm_id = realms_by_name
                .get(name)
                .ok_or_else(|| AuthencError::RealmNotFound(format!("Realm {} not found", name)))?;

            let realms = self.realms.lock().await;
            realms
                .get(realm_id)
                .cloned()
                .ok_or_else(|| AuthencError::RealmNotFound(format!("Realm {} not found", name)))
        }

        async fn create_realm(&self, name: String, display_name: String) -> Result<Realm> {
            let realm = Realm {
                id: RealmId::new(),
                name: name.clone(),
                display_name,
                enabled: true,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };

            let mut realms = self.realms.lock().await;
            realms.insert(realm.id, realm.clone());

            let mut realms_by_name = self.realms_by_name.lock().await;
            realms_by_name.insert(name, realm.id);

            Ok(realm)
        }

        async fn update_realm(
            &self,
            id: RealmId,
            display_name: Option<String>,
            enabled: Option<bool>,
        ) -> Result<Realm> {
            let mut realms = self.realms.lock().await;
            let realm = realms
                .get_mut(&id)
                .ok_or_else(|| AuthencError::RealmNotFound(format!("Realm {} not found", id)))?;

            if let Some(display_name) = display_name {
                realm.display_name = display_name;
            }
            if let Some(enabled) = enabled {
                realm.enabled = enabled;
            }

            realm.updated_at = chrono::Utc::now();

            Ok(realm.clone())
        }

        async fn delete_realm(&self, id: RealmId) -> Result<()> {
            let mut realms = self.realms.lock().await;
            let realm = realms
                .remove(&id)
                .ok_or_else(|| AuthencError::RealmNotFound(format!("Realm {} not found", id)))?;

            let mut realms_by_name = self.realms_by_name.lock().await;
            realms_by_name.remove(&realm.name);

            Ok(())
        }

        async fn list_realms(&self) -> Result<Vec<Realm>> {
            let realms = self.realms.lock().await;
            let mut realm_list: Vec<Realm> = realms.values().cloned().collect();
            realm_list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            Ok(realm_list)
        }

        async fn realm_name_exists(&self, name: &str) -> Result<bool> {
            let realms_by_name = self.realms_by_name.lock().await;
            Ok(realms_by_name.contains_key(name))
        }
    }

    #[tokio::test]
    async fn test_create_realm_success() {
        let realm_store = Arc::new(MockRealmStore::new());
        let service = RealmManagementServiceImpl::new(realm_store);

        let realm = service
            .create_realm("test-realm".to_string(), "Test Realm".to_string())
            .await
            .unwrap();

        assert_eq!(realm.name, "test-realm");
        assert_eq!(realm.display_name, "Test Realm");
        assert!(realm.enabled);
    }

    #[tokio::test]
    async fn test_create_realm_invalid_name_too_short() {
        let realm_store = Arc::new(MockRealmStore::new());
        let service = RealmManagementServiceImpl::new(realm_store);

        let result = service
            .create_realm("ab".to_string(), "Test Realm".to_string())
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AuthencError::ValidationError(_)
        ));
    }

    #[tokio::test]
    async fn test_create_realm_invalid_name_uppercase() {
        let realm_store = Arc::new(MockRealmStore::new());
        let service = RealmManagementServiceImpl::new(realm_store);

        let result = service
            .create_realm("TestRealm".to_string(), "Test Realm".to_string())
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AuthencError::ValidationError(_)
        ));
    }

    #[tokio::test]
    async fn test_create_realm_invalid_name_starts_with_digit() {
        let realm_store = Arc::new(MockRealmStore::new());
        let service = RealmManagementServiceImpl::new(realm_store);

        let result = service
            .create_realm("1test".to_string(), "Test Realm".to_string())
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AuthencError::ValidationError(_)
        ));
    }

    #[tokio::test]
    async fn test_create_realm_duplicate_name() {
        let realm_store = Arc::new(MockRealmStore::new());
        let service = RealmManagementServiceImpl::new(realm_store);

        // Create first realm
        service
            .create_realm("test-realm".to_string(), "Test Realm".to_string())
            .await
            .unwrap();

        // Try to create second realm with same name
        let result = service
            .create_realm("test-realm".to_string(), "Another Realm".to_string())
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthencError::Conflict(_)));
    }

    #[tokio::test]
    async fn test_get_realm() {
        let realm_store = Arc::new(MockRealmStore::new());
        let service = RealmManagementServiceImpl::new(realm_store);

        let created_realm = service
            .create_realm("test-realm".to_string(), "Test Realm".to_string())
            .await
            .unwrap();

        let retrieved_realm = service.get_realm(created_realm.id).await.unwrap();

        assert_eq!(retrieved_realm.id, created_realm.id);
        assert_eq!(retrieved_realm.name, "test-realm");
    }

    #[tokio::test]
    async fn test_get_realm_by_name() {
        let realm_store = Arc::new(MockRealmStore::new());
        let service = RealmManagementServiceImpl::new(realm_store);

        service
            .create_realm("test-realm".to_string(), "Test Realm".to_string())
            .await
            .unwrap();

        let retrieved_realm = service.get_realm_by_name("test-realm").await.unwrap();

        assert_eq!(retrieved_realm.name, "test-realm");
    }

    #[tokio::test]
    async fn test_update_realm_display_name() {
        let realm_store = Arc::new(MockRealmStore::new());
        let service = RealmManagementServiceImpl::new(realm_store);

        let realm = service
            .create_realm("test-realm".to_string(), "Test Realm".to_string())
            .await
            .unwrap();

        let updated_realm = service
            .update_realm(realm.id, Some("Updated Realm".to_string()), None)
            .await
            .unwrap();

        assert_eq!(updated_realm.display_name, "Updated Realm");
        assert_eq!(updated_realm.name, "test-realm"); // Name should not change
    }

    #[tokio::test]
    async fn test_enable_disable_realm() {
        let realm_store = Arc::new(MockRealmStore::new());
        let service = RealmManagementServiceImpl::new(realm_store);

        let realm = service
            .create_realm("test-realm".to_string(), "Test Realm".to_string())
            .await
            .unwrap();

        // Realm should be enabled by default
        assert!(realm.enabled);

        // Disable realm
        let disabled_realm = service.disable_realm(realm.id).await.unwrap();
        assert!(!disabled_realm.enabled);

        // Enable realm
        let enabled_realm = service.enable_realm(realm.id).await.unwrap();
        assert!(enabled_realm.enabled);
    }

    #[tokio::test]
    async fn test_delete_realm() {
        let realm_store = Arc::new(MockRealmStore::new());
        let service = RealmManagementServiceImpl::new(realm_store);

        let realm = service
            .create_realm("test-realm".to_string(), "Test Realm".to_string())
            .await
            .unwrap();

        // Delete realm
        service.delete_realm(realm.id).await.unwrap();

        // Verify realm is deleted
        let result = service.get_realm(realm.id).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AuthencError::RealmNotFound(_)
        ));
    }

    #[tokio::test]
    async fn test_list_realms() {
        let realm_store = Arc::new(MockRealmStore::new());
        let service = RealmManagementServiceImpl::new(realm_store);

        // Create multiple realms
        service
            .create_realm("realm1".to_string(), "Realm 1".to_string())
            .await
            .unwrap();
        service
            .create_realm("realm2".to_string(), "Realm 2".to_string())
            .await
            .unwrap();
        service
            .create_realm("realm3".to_string(), "Realm 3".to_string())
            .await
            .unwrap();

        // List realms
        let realms = service.list_realms().await.unwrap();
        assert_eq!(realms.len(), 3);
    }

    #[tokio::test]
    async fn test_is_realm_enabled() {
        let realm_store = Arc::new(MockRealmStore::new());
        let service = RealmManagementServiceImpl::new(realm_store);

        let realm = service
            .create_realm("test-realm".to_string(), "Test Realm".to_string())
            .await
            .unwrap();

        // Check enabled status
        assert!(service.is_realm_enabled(realm.id).await.unwrap());

        // Disable and check again
        service.disable_realm(realm.id).await.unwrap();
        assert!(!service.is_realm_enabled(realm.id).await.unwrap());
    }
}
