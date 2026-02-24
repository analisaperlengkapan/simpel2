use async_trait::async_trait;
use authenc_storage::Database;
use authenc_types::domain::resource_server::{
    CreateResourceServerRequest, ResourceServer, UpdateResourceServerRequest,
};
use authenc_types::error::AuthencError;
use std::sync::Arc;
use uuid::Uuid;

/// Resource server store for managing resource servers in the database
#[derive(Debug, Clone)]
pub struct ResourceServerStore {
    /// Database instance
    database: Arc<Database>,
}

impl ResourceServerStore {
    /// Create a new resource server store
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }
}

/// Trait for resource server store operations
#[async_trait]
pub trait ResourceServerStoreTrait: Send + Sync {
    /// Create a new resource server
    async fn create_resource_server(
        &self,
        request: CreateResourceServerRequest,
        realm_id: Uuid,
    ) -> Result<ResourceServer, AuthencError>;

    /// Get resource server by ID
    async fn get_resource_server(&self, id: Uuid) -> Result<Option<ResourceServer>, AuthencError>;

    /// Get resource server by client ID
    async fn get_resource_server_by_client(
        &self,
        client_id: &str,
        realm_id: Uuid,
    ) -> Result<Option<ResourceServer>, AuthencError>;

    /// Get resource servers by realm
    async fn get_resource_servers_by_realm(
        &self,
        realm_id: Uuid,
        first: Option<i32>,
        max: Option<i32>,
    ) -> Result<Vec<ResourceServer>, AuthencError>;

    /// Update resource server
    async fn update_resource_server(
        &self,
        id: Uuid,
        request: UpdateResourceServerRequest,
    ) -> Result<ResourceServer, AuthencError>;

    /// Delete resource server
    async fn delete_resource_server(&self, id: Uuid) -> Result<(), AuthencError>;

    /// Search resource servers by name
    async fn search_resource_servers(
        &self,
        name: &str,
        realm_id: Uuid,
        first: Option<i32>,
        max: Option<i32>,
    ) -> Result<Vec<ResourceServer>, AuthencError>;

    /// Get resource server count for realm
    async fn count_resource_servers_by_realm(&self, realm_id: Uuid) -> Result<i64, AuthencError>;
}

#[async_trait]
impl ResourceServerStoreTrait for ResourceServerStore {
    async fn create_resource_server(
        &self,
        _request: CreateResourceServerRequest,
        _realm_id: Uuid,
    ) -> Result<ResourceServer, AuthencError> {
        // TODO: Implement when operations::resource_servers is available in authenc-storage
        Err(AuthencError::database(
            "Resource server operations not yet available in storage crate",
        ))
    }

    async fn get_resource_server(&self, _id: Uuid) -> Result<Option<ResourceServer>, AuthencError> {
        // TODO: Implement when operations::resource_servers is available in authenc-storage
        Err(AuthencError::database(
            "Resource server operations not yet available in storage crate",
        ))
    }

    async fn get_resource_server_by_client(
        &self,
        _client_id: &str,
        _realm_id: Uuid,
    ) -> Result<Option<ResourceServer>, AuthencError> {
        // TODO: Implement when operations::resource_servers is available in authenc-storage
        Err(AuthencError::database(
            "Resource server operations not yet available in storage crate",
        ))
    }

    async fn get_resource_servers_by_realm(
        &self,
        _realm_id: Uuid,
        _first: Option<i32>,
        _max: Option<i32>,
    ) -> Result<Vec<ResourceServer>, AuthencError> {
        // TODO: Implement when operations::resource_servers is available in authenc-storage
        Err(AuthencError::database(
            "Resource server operations not yet available in storage crate",
        ))
    }

    async fn update_resource_server(
        &self,
        _id: Uuid,
        _request: UpdateResourceServerRequest,
    ) -> Result<ResourceServer, AuthencError> {
        // TODO: Implement when operations::resource_servers is available in authenc-storage
        Err(AuthencError::database(
            "Resource server operations not yet available in storage crate",
        ))
    }

    async fn delete_resource_server(&self, _id: Uuid) -> Result<(), AuthencError> {
        // TODO: Implement when operations::resource_servers is available in authenc-storage
        Err(AuthencError::database(
            "Resource server operations not yet available in storage crate",
        ))
    }

    async fn search_resource_servers(
        &self,
        _name: &str,
        _realm_id: Uuid,
        _first: Option<i32>,
        _max: Option<i32>,
    ) -> Result<Vec<ResourceServer>, AuthencError> {
        // TODO: Implement when operations::resource_servers is available in authenc-storage
        Err(AuthencError::database(
            "Resource server operations not yet available in storage crate",
        ))
    }

    async fn count_resource_servers_by_realm(&self, _realm_id: Uuid) -> Result<i64, AuthencError> {
        // TODO: Implement when operations::resource_servers is available in authenc-storage
        Err(AuthencError::database(
            "Resource server operations not yet available in storage crate",
        ))
    }
}
