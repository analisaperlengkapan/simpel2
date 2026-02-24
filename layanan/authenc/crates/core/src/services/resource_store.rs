use async_trait::async_trait;
use authenc_storage::Database;
use authenc_types::domain::resource::{CreateResourceRequest, Resource, UpdateResourceRequest};
use authenc_types::error::AuthencError;
use std::sync::Arc;
use uuid::Uuid;

/// Resource store for managing resources in the database
#[derive(Debug, Clone)]
pub struct ResourceStore {
    /// Database instance
    database: Arc<Database>,
}

impl ResourceStore {
    /// Create a new resource store
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }
}

/// Trait for resource store operations
#[async_trait]
pub trait ResourceStoreTrait: Send + Sync {
    /// Create a new resource
    async fn create_resource(
        &self,
        request: CreateResourceRequest,
        realm_id: Uuid,
        resource_server_id: Uuid,
        owner: String,
    ) -> Result<Resource, AuthencError>;

    /// Get resource by ID
    async fn get_resource(&self, id: Uuid) -> Result<Option<Resource>, AuthencError>;

    /// Get resource by name and resource server
    async fn get_resource_by_name(
        &self,
        name: &str,
        resource_server_id: Uuid,
    ) -> Result<Option<Resource>, AuthencError>;

    /// Get resources by owner
    async fn get_resources_by_owner(
        &self,
        owner: &str,
        first: Option<i32>,
        max: Option<i32>,
    ) -> Result<Vec<Resource>, AuthencError>;

    /// Get resources by resource server
    async fn get_resources_by_server(
        &self,
        resource_server_id: Uuid,
        first: Option<i32>,
        max: Option<i32>,
    ) -> Result<Vec<Resource>, AuthencError>;

    /// Get resources by realm
    async fn get_resources_by_realm(
        &self,
        realm_id: Uuid,
        first: Option<i32>,
        max: Option<i32>,
    ) -> Result<Vec<Resource>, AuthencError>;

    /// Update resource
    async fn update_resource(
        &self,
        id: Uuid,
        request: UpdateResourceRequest,
    ) -> Result<Resource, AuthencError>;

    /// Delete resource
    async fn delete_resource(&self, id: Uuid) -> Result<(), AuthencError>;

    /// Search resources by name
    async fn search_resources(
        &self,
        name: &str,
        realm_id: Uuid,
        first: Option<i32>,
        max: Option<i32>,
    ) -> Result<Vec<Resource>, AuthencError>;

    /// Get resource count for owner
    async fn count_resources_by_owner(&self, owner: &str) -> Result<i64, AuthencError>;
}

#[async_trait]
impl ResourceStoreTrait for ResourceStore {
    async fn create_resource(
        &self,
        _request: CreateResourceRequest,
        _realm_id: Uuid,
        _resource_server_id: Uuid,
        _owner: String,
    ) -> Result<Resource, AuthencError> {
        // TODO: Implement when operations::resources is available in authenc-storage
        Err(AuthencError::database(
            "Resource operations not yet available in storage crate",
        ))
    }

    async fn get_resource(&self, _id: Uuid) -> Result<Option<Resource>, AuthencError> {
        // TODO: Implement when operations::resources is available in authenc-storage
        Err(AuthencError::database(
            "Resource operations not yet available in storage crate",
        ))
    }

    async fn get_resource_by_name(
        &self,
        _name: &str,
        _resource_server_id: Uuid,
    ) -> Result<Option<Resource>, AuthencError> {
        // TODO: Implement when operations::resources is available in authenc-storage
        Err(AuthencError::database(
            "Resource operations not yet available in storage crate",
        ))
    }

    async fn get_resources_by_owner(
        &self,
        _owner: &str,
        _first: Option<i32>,
        _max: Option<i32>,
    ) -> Result<Vec<Resource>, AuthencError> {
        // TODO: Implement when operations::resources is available in authenc-storage
        Err(AuthencError::database(
            "Resource operations not yet available in storage crate",
        ))
    }

    async fn get_resources_by_server(
        &self,
        _resource_server_id: Uuid,
        _first: Option<i32>,
        _max: Option<i32>,
    ) -> Result<Vec<Resource>, AuthencError> {
        // TODO: Implement when operations::resources is available in authenc-storage
        Err(AuthencError::database(
            "Resource operations not yet available in storage crate",
        ))
    }

    async fn get_resources_by_realm(
        &self,
        _realm_id: Uuid,
        _first: Option<i32>,
        _max: Option<i32>,
    ) -> Result<Vec<Resource>, AuthencError> {
        // TODO: Implement when operations::resources is available in authenc-storage
        Err(AuthencError::database(
            "Resource operations not yet available in storage crate",
        ))
    }

    async fn update_resource(
        &self,
        _id: Uuid,
        _request: UpdateResourceRequest,
    ) -> Result<Resource, AuthencError> {
        // TODO: Implement when operations::resources is available in authenc-storage
        Err(AuthencError::database(
            "Resource operations not yet available in storage crate",
        ))
    }

    async fn delete_resource(&self, _id: Uuid) -> Result<(), AuthencError> {
        // TODO: Implement when operations::resources is available in authenc-storage
        Err(AuthencError::database(
            "Resource operations not yet available in storage crate",
        ))
    }

    async fn search_resources(
        &self,
        _name: &str,
        _realm_id: Uuid,
        _first: Option<i32>,
        _max: Option<i32>,
    ) -> Result<Vec<Resource>, AuthencError> {
        // TODO: Implement when operations::resources is available in authenc-storage
        Err(AuthencError::database(
            "Resource operations not yet available in storage crate",
        ))
    }

    async fn count_resources_by_owner(&self, _owner: &str) -> Result<i64, AuthencError> {
        // TODO: Implement when operations::resources is available in authenc-storage
        Err(AuthencError::database(
            "Resource operations not yet available in storage crate",
        ))
    }
}
