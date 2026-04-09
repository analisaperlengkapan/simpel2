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
        // The DB partial unique index on (name) WHERE deleted_at IS NULL is the
        // ultimate guard against duplicates. If a concurrent request slips past
        // the realm_name_exists check above, the INSERT will fail with a DB
        // error. We translate that into a user-friendly Conflict error.
        let realm = self
            .realm_store
            .create_realm(name.clone(), display_name)
            .await
            .map_err(|e| {
                if let AuthencError::DatabaseError(ref msg) = e {
                    if msg.contains("duplicate key") || msg.contains("unique") {
                        return AuthencError::Conflict(format!(
                            "Realm name '{}' already exists",
                            name
                        ));
                    }
                }
                e
            })?;

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

    /// Delete a realm (soft delete)
    ///
    /// This method performs a soft delete by setting `deleted_at` on the realm record.
    /// The realm will no longer appear in queries but the data is preserved for
    /// auditing and potential recovery.
    ///
    /// # Arguments
    ///
    /// * `realm_id` - Realm ID
    ///
    /// # Errors
    ///
    /// - `AuthorizationFailed` if attempting to delete the master realm
    /// - `RealmNotFound` if the realm does not exist or is already deleted
    ///
    /// # Cascade behaviour
    ///
    /// Before soft-deleting the realm, all live users in the realm are disabled
    /// (`enabled = false`, `deleted_at = NOW()`). This prevents users from
    /// authenticating against a soft-deleted realm.
    ///
    /// **Note:** OAuth2 clients and active sessions in the realm are not yet
    /// cascaded. Sessions will expire naturally, but clients may need manual
    /// cleanup until full cascade logic is implemented.
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

        // Step 1: Disable all users in the realm so they cannot authenticate
        // against a soft-deleted realm. This must happen BEFORE the realm is
        // marked as deleted to maintain consistency.
        let users_disabled = self.realm_store.disable_users_in_realm(realm_id).await?;
        info!(
            realm_id = %realm_id,
            users_disabled = users_disabled,
            "Disabled users in realm before soft-delete"
        );

        // Step 2: Soft-delete the realm itself
        self.realm_store.delete_realm(realm_id).await?;

        info!(realm_id = %realm_id, "Realm soft-deleted successfully");

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
    use authenc_types::domain::realm::Realm;
    use authenc_types::domain_types::RealmId;
    use authenc_types::error::AuthencError;
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// In-memory mock implementation of RealmStore for unit testing.
    struct MockRealmStore {
        realms: Mutex<HashMap<uuid::Uuid, Realm>>,
    }

    impl MockRealmStore {
        fn new() -> Self {
            Self {
                realms: Mutex::new(HashMap::new()),
            }
        }

        /// Seed the store with a realm (used to pre-populate for update/delete tests).
        fn seed(&self, realm: Realm) {
            self.realms.lock().unwrap().insert(realm.id, realm);
        }
    }

    #[async_trait]
    impl RealmStore for MockRealmStore {
        async fn get_realm(&self, id: RealmId) -> Result<Realm> {
            self.realms
                .lock()
                .unwrap()
                .get(id.as_uuid())
                .cloned()
                .ok_or_else(|| AuthencError::NotFound("Realm not found".to_string()))
        }

        async fn get_realm_by_name(&self, name: &str) -> Result<Realm> {
            self.realms
                .lock()
                .unwrap()
                .values()
                .find(|r| r.name == name)
                .cloned()
                .ok_or_else(|| AuthencError::NotFound("Realm not found".to_string()))
        }

        async fn create_realm(&self, name: String, display_name: String) -> Result<Realm> {
            let mut realm = Realm::new(name);
            realm.display_name = Some(display_name);
            self.realms.lock().unwrap().insert(realm.id, realm.clone());
            Ok(realm)
        }

        async fn update_realm(
            &self,
            id: RealmId,
            display_name: Option<String>,
            enabled: Option<bool>,
        ) -> Result<Realm> {
            let mut realms = self.realms.lock().unwrap();
            let realm = realms
                .get_mut(id.as_uuid())
                .ok_or_else(|| AuthencError::NotFound("Realm not found".to_string()))?;
            if let Some(dn) = display_name {
                realm.display_name = Some(dn);
            }
            if let Some(e) = enabled {
                realm.enabled = e;
            }
            Ok(realm.clone())
        }

        async fn delete_realm(&self, id: RealmId) -> Result<()> {
            self.realms
                .lock()
                .unwrap()
                .remove(id.as_uuid())
                .ok_or_else(|| AuthencError::NotFound("Realm not found".to_string()))?;
            Ok(())
        }

        async fn list_realms(&self) -> Result<Vec<Realm>> {
            Ok(self.realms.lock().unwrap().values().cloned().collect())
        }

        async fn realm_name_exists(&self, name: &str) -> Result<bool> {
            Ok(self.realms.lock().unwrap().values().any(|r| r.name == name))
        }

        async fn disable_users_in_realm(&self, _realm_id: RealmId) -> Result<u64> {
            // Mock: no actual users to disable; return 0.
            Ok(0)
        }
    }

    fn make_service(store: Arc<MockRealmStore>) -> RealmManagementServiceImpl {
        RealmManagementServiceImpl::new(store)
    }

    fn master_realm() -> Realm {
        let mut realm = Realm::default();
        realm.id = Realm::MASTER_ID;
        realm.name = "master".to_string();
        realm.display_name = Some("Master".to_string());
        realm.enabled = true;
        realm
    }

    // ── Master realm protection tests ────────────────────────────────────

    #[tokio::test]
    async fn test_disable_master_realm_is_rejected() {
        let store = Arc::new(MockRealmStore::new());
        store.seed(master_realm());
        let svc = make_service(store);

        let result = svc
            .update_realm(RealmId::from_uuid(Realm::MASTER_ID), None, Some(false))
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, AuthencError::AuthorizationFailed(_)),
            "Expected AuthorizationFailed, got: {:?}",
            err
        );
    }

    #[tokio::test]
    async fn test_enable_master_realm_is_allowed() {
        let store = Arc::new(MockRealmStore::new());
        store.seed(master_realm());
        let svc = make_service(store);

        let result = svc
            .update_realm(RealmId::from_uuid(Realm::MASTER_ID), None, Some(true))
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_master_realm_display_name_is_allowed() {
        let store = Arc::new(MockRealmStore::new());
        store.seed(master_realm());
        let svc = make_service(store);

        let result = svc
            .update_realm(
                RealmId::from_uuid(Realm::MASTER_ID),
                Some("New Master Name".to_string()),
                None,
            )
            .await;

        assert!(result.is_ok());
        let realm = result.unwrap();
        assert_eq!(realm.display_name, Some("New Master Name".to_string()));
    }

    #[tokio::test]
    async fn test_delete_master_realm_is_rejected() {
        let store = Arc::new(MockRealmStore::new());
        store.seed(master_realm());
        let svc = make_service(store);

        let result = svc.delete_realm(RealmId::from_uuid(Realm::MASTER_ID)).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, AuthencError::AuthorizationFailed(_)),
            "Expected AuthorizationFailed, got: {:?}",
            err
        );
    }

    #[tokio::test]
    async fn test_disable_master_realm_via_convenience_method_is_rejected() {
        let store = Arc::new(MockRealmStore::new());
        store.seed(master_realm());
        let svc = make_service(store);

        let result = svc
            .disable_realm(RealmId::from_uuid(Realm::MASTER_ID))
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AuthencError::AuthorizationFailed(_)
        ));
    }

    // ── Normal realm CRUD tests ──────────────────────────────────────────

    #[tokio::test]
    async fn test_create_realm_success() {
        let store = Arc::new(MockRealmStore::new());
        let svc = make_service(store);

        let result = svc
            .create_realm("test-realm".to_string(), "Test Realm".to_string())
            .await;

        assert!(result.is_ok());
        let realm = result.unwrap();
        assert_eq!(realm.name, "test-realm");
        assert_eq!(realm.display_name, Some("Test Realm".to_string()));
    }

    #[tokio::test]
    async fn test_create_realm_duplicate_name_rejected() {
        let store = Arc::new(MockRealmStore::new());
        let svc = make_service(store);

        svc.create_realm("test-realm".to_string(), "Test Realm".to_string())
            .await
            .unwrap();

        let result = svc
            .create_realm("test-realm".to_string(), "Another Realm".to_string())
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthencError::Conflict(_)));
    }

    #[tokio::test]
    async fn test_delete_normal_realm_succeeds() {
        let store = Arc::new(MockRealmStore::new());
        let svc = make_service(store.clone());

        let realm = svc
            .create_realm("deletable".to_string(), "Deletable Realm".to_string())
            .await
            .unwrap();

        let result = svc.delete_realm(RealmId::from_uuid(realm.id)).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_disable_normal_realm_succeeds() {
        let store = Arc::new(MockRealmStore::new());
        let svc = make_service(store.clone());

        let realm = svc
            .create_realm("disableable".to_string(), "Disableable Realm".to_string())
            .await
            .unwrap();

        let result = svc
            .update_realm(RealmId::from_uuid(realm.id), None, Some(false))
            .await;

        assert!(result.is_ok());
        assert!(!result.unwrap().enabled);
    }

    // ── Validation tests ─────────────────────────────────────────────────

    #[tokio::test]
    async fn test_create_realm_name_too_short() {
        let store = Arc::new(MockRealmStore::new());
        let svc = make_service(store);

        let result = svc
            .create_realm("ab".to_string(), "Display".to_string())
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AuthencError::ValidationError(_)
        ));
    }

    #[tokio::test]
    async fn test_create_realm_name_must_start_with_letter() {
        let store = Arc::new(MockRealmStore::new());
        let svc = make_service(store);

        let result = svc
            .create_realm("123realm".to_string(), "Display".to_string())
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AuthencError::ValidationError(_)
        ));
    }

    #[tokio::test]
    async fn test_create_realm_empty_display_name_rejected() {
        let store = Arc::new(MockRealmStore::new());
        let svc = make_service(store);

        let result = svc
            .create_realm("valid-name".to_string(), "".to_string())
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AuthencError::ValidationError(_)
        ));
    }

    #[tokio::test]
    async fn test_update_realm_empty_display_name_rejected() {
        let store = Arc::new(MockRealmStore::new());
        let svc = make_service(store.clone());

        let realm = svc
            .create_realm("updatable".to_string(), "Original".to_string())
            .await
            .unwrap();

        let result = svc
            .update_realm(RealmId::from_uuid(realm.id), Some("".to_string()), None)
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AuthencError::ValidationError(_)
        ));
    }

    #[tokio::test]
    async fn test_list_realms() {
        let store = Arc::new(MockRealmStore::new());
        let svc = make_service(store);

        svc.create_realm("realm-one".to_string(), "Realm One".to_string())
            .await
            .unwrap();
        svc.create_realm("realm-two".to_string(), "Realm Two".to_string())
            .await
            .unwrap();

        let realms = svc.list_realms().await.unwrap();
        assert_eq!(realms.len(), 2);
    }
}
