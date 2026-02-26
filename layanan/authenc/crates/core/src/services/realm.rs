//! Realm service for multi-tenant realm management

use async_trait::async_trait;
use authenc_storage::Database;
use authenc_types::domain::{CreateRealmRequest, RealmResponse, UpdateRealmRequest};
use std::sync::Arc;
use uuid::Uuid;

/// Realm service trait for multi-tenant realm management
#[async_trait]
pub trait RealmService: Send + Sync {
    /// Create a new realm
    async fn create_realm(&self, request: CreateRealmRequest) -> Result<RealmResponse, String>;

    /// Get realm by ID
    async fn get_realm_by_id(&self, realm_id: &Uuid) -> Result<Option<RealmResponse>, String>;

    /// Get realm by name
    async fn get_realm_by_name(&self, name: &str) -> Result<Option<RealmResponse>, String>;

    /// Update realm
    async fn update_realm(
        &self,
        realm_id: &Uuid,
        request: UpdateRealmRequest,
    ) -> Result<RealmResponse, String>;

    /// Delete realm
    async fn delete_realm(&self, realm_id: &Uuid) -> Result<(), String>;

    /// List all realms
    async fn list_realms(&self) -> Result<Vec<RealmResponse>, String>;

    /// Enable/disable realm
    async fn set_realm_enabled(&self, realm_id: &Uuid, enabled: bool) -> Result<(), String>;
}

/// PostgreSQL implementation of RealmService
pub struct PostgresRealmService {
    /// Database connection
    #[allow(dead_code)]
    db: Arc<Database>,
}

impl PostgresRealmService {
    /// Create new PostgreSQL realm service
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl RealmService for PostgresRealmService {
    async fn create_realm(&self, _request: CreateRealmRequest) -> Result<RealmResponse, String> {
        // TODO: Implement when operations::realms is available in authenc-storage
        Err("Realm operations not yet available in storage crate".to_string())
    }

    async fn get_realm_by_id(&self, _realm_id: &Uuid) -> Result<Option<RealmResponse>, String> {
        // TODO: Implement when operations::realms is available in authenc-storage
        Err("Realm operations not yet available in storage crate".to_string())
    }

    async fn get_realm_by_name(&self, _name: &str) -> Result<Option<RealmResponse>, String> {
        // TODO: Implement when operations::realms is available in authenc-storage
        Err("Realm operations not yet available in storage crate".to_string())
    }

    async fn update_realm(
        &self,
        _realm_id: &Uuid,
        _request: UpdateRealmRequest,
    ) -> Result<RealmResponse, String> {
        // TODO: Implement when operations::realms is available in authenc-storage
        Err("Realm operations not yet available in storage crate".to_string())
    }

    async fn delete_realm(&self, _realm_id: &Uuid) -> Result<(), String> {
        // TODO: Implement when operations::realms is available in authenc-storage
        Err("Realm operations not yet available in storage crate".to_string())
    }

    async fn list_realms(&self) -> Result<Vec<RealmResponse>, String> {
        // TODO: Implement when operations::realms is available in authenc-storage
        Err("Realm operations not yet available in storage crate".to_string())
    }

    async fn set_realm_enabled(&self, _realm_id: &Uuid, _enabled: bool) -> Result<(), String> {
        // TODO: Implement when operations::realms is available in authenc-storage
        Err("Realm operations not yet available in storage crate".to_string())
    }
}

/// Realm management service for multi-tenant operations
pub struct RealmManager {
    /// Realm service implementation
    service: Arc<dyn RealmService>,
}

impl RealmManager {
    /// Create new realm manager
    pub fn new(service: Arc<dyn RealmService>) -> Self {
        Self { service }
    }

    /// Create a new realm with default settings
    pub async fn create_realm(
        &self,
        name: &str,
        display_name: Option<String>,
        description: Option<String>,
    ) -> Result<RealmResponse, String> {
        let request = CreateRealmRequest {
            name: name.to_string(),
            display_name,
            description,
            enabled: Some(true),
            attributes: None,
        };

        self.service.create_realm(request).await
    }

    /// Get realm by name or ID
    pub async fn get_realm(&self, identifier: &str) -> Result<Option<RealmResponse>, String> {
        // Try to parse as UUID first
        if let Ok(uuid) = Uuid::parse_str(identifier) {
            self.service.get_realm_by_id(&uuid).await
        } else {
            self.service.get_realm_by_name(identifier).await
        }
    }

    /// List all active realms
    pub async fn list_active_realms(&self) -> Result<Vec<RealmResponse>, String> {
        let realms = self.service.list_realms().await?;
        Ok(realms.into_iter().filter(|r| r.enabled).collect())
    }

    /// Check if realm exists and is enabled
    pub async fn realm_exists_and_enabled(&self, identifier: &str) -> Result<bool, String> {
        match self.get_realm(identifier).await? {
            Some(realm) => Ok(realm.enabled),
            None => Ok(false),
        }
    }

    /// Enable or disable a realm
    pub async fn set_realm_status(&self, identifier: &str, enabled: bool) -> Result<(), String> {
        let realm = self
            .get_realm(identifier)
            .await?
            .ok_or_else(|| format!("Realm '{}' not found", identifier))?;

        self.service.set_realm_enabled(&realm.id, enabled).await
    }
}
