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
        request: CreateResourceServerRequest,
        realm_id: Uuid,
    ) -> Result<ResourceServer, AuthencError> {
        let id = Uuid::new_v4();
        let policy_mode = request
            .policy_enforcement_mode
            .unwrap_or_default()
            .as_str()
            .to_string();
        let decision_strat = request
            .decision_strategy
            .unwrap_or_default()
            .as_str()
            .to_string();
        let allow_remote = request.allow_remote_resource_management.unwrap_or(false);

        let row = self
            .database
            .query_one(
                r#"INSERT INTO resource_servers
                    (id, client_id, name, description, enabled, realm_id,
                     policy_enforcement_mode, decision_strategy, allow_remote_resource_management)
                   VALUES ($1, $2, $3, $4, true, $5, $6, $7, $8)
                   RETURNING *"#,
                &[
                    &id,
                    &request.client_id,
                    &request.name,
                    &request.description,
                    &realm_id,
                    &policy_mode,
                    &decision_strat,
                    &allow_remote,
                ],
            )
            .await
            .map_err(|e| {
                AuthencError::database(format!("Failed to create resource server: {}", e))
            })?;

        ResourceServer::try_from(row)
    }

    async fn get_resource_server(&self, id: Uuid) -> Result<Option<ResourceServer>, AuthencError> {
        let row = self
            .database
            .query_opt("SELECT * FROM resource_servers WHERE id = $1", &[&id])
            .await
            .map_err(|e| AuthencError::database(format!("Failed to get resource server: {}", e)))?;

        match row {
            Some(r) => Ok(Some(ResourceServer::try_from(r)?)),
            None => Ok(None),
        }
    }

    async fn get_resource_server_by_client(
        &self,
        client_id: &str,
        realm_id: Uuid,
    ) -> Result<Option<ResourceServer>, AuthencError> {
        let row = self
            .database
            .query_opt(
                "SELECT * FROM resource_servers WHERE client_id = $1 AND realm_id = $2",
                &[&client_id, &realm_id],
            )
            .await
            .map_err(|e| {
                AuthencError::database(format!("Failed to get resource server by client: {}", e))
            })?;

        match row {
            Some(r) => Ok(Some(ResourceServer::try_from(r)?)),
            None => Ok(None),
        }
    }

    async fn get_resource_servers_by_realm(
        &self,
        realm_id: Uuid,
        first: Option<i32>,
        max: Option<i32>,
    ) -> Result<Vec<ResourceServer>, AuthencError> {
        let offset = first.unwrap_or(0) as i64;
        let limit = max.unwrap_or(100) as i64;

        let rows = self
            .database
            .query(
                "SELECT * FROM resource_servers WHERE realm_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
                &[&realm_id, &limit, &offset],
            )
            .await
            .map_err(|e| {
                AuthencError::database(format!("Failed to list resource servers: {}", e))
            })?;

        rows.into_iter()
            .map(ResourceServer::try_from)
            .collect::<Result<Vec<_>, _>>()
    }

    async fn update_resource_server(
        &self,
        id: Uuid,
        request: UpdateResourceServerRequest,
    ) -> Result<ResourceServer, AuthencError> {
        // First get existing server
        let existing = self
            .get_resource_server(id)
            .await?
            .ok_or_else(|| AuthencError::not_found("Resource server not found"))?;

        let name = request.name.or(existing.name);
        let description = request.description.or(existing.description);
        let policy_mode = request
            .policy_enforcement_mode
            .unwrap_or(existing.policy_enforcement_mode)
            .as_str()
            .to_string();
        let decision_strat = request
            .decision_strategy
            .unwrap_or(existing.decision_strategy)
            .as_str()
            .to_string();
        let allow_remote = request
            .allow_remote_resource_management
            .unwrap_or(existing.allow_remote_resource_management);

        let row = self
            .database
            .query_one(
                r#"UPDATE resource_servers
                   SET name = $2, description = $3, policy_enforcement_mode = $4,
                       decision_strategy = $5, allow_remote_resource_management = $6,
                       updated_at = NOW()
                   WHERE id = $1
                   RETURNING *"#,
                &[
                    &id,
                    &name,
                    &description,
                    &policy_mode,
                    &decision_strat,
                    &allow_remote,
                ],
            )
            .await
            .map_err(|e| {
                AuthencError::database(format!("Failed to update resource server: {}", e))
            })?;

        ResourceServer::try_from(row)
    }

    async fn delete_resource_server(&self, id: Uuid) -> Result<(), AuthencError> {
        self.database
            .execute("DELETE FROM resource_servers WHERE id = $1", &[&id])
            .await
            .map_err(|e| {
                AuthencError::database(format!("Failed to delete resource server: {}", e))
            })?;
        Ok(())
    }

    async fn search_resource_servers(
        &self,
        name: &str,
        realm_id: Uuid,
        first: Option<i32>,
        max: Option<i32>,
    ) -> Result<Vec<ResourceServer>, AuthencError> {
        let offset = first.unwrap_or(0) as i64;
        let limit = max.unwrap_or(100) as i64;
        let pattern = format!("%{}%", name);

        let rows = self
            .database
            .query(
                r#"SELECT * FROM resource_servers
                   WHERE realm_id = $1 AND (name ILIKE $2 OR client_id ILIKE $2)
                   ORDER BY created_at DESC LIMIT $3 OFFSET $4"#,
                &[&realm_id, &pattern, &limit, &offset],
            )
            .await
            .map_err(|e| {
                AuthencError::database(format!("Failed to search resource servers: {}", e))
            })?;

        rows.into_iter()
            .map(ResourceServer::try_from)
            .collect::<Result<Vec<_>, _>>()
    }

    async fn count_resource_servers_by_realm(&self, realm_id: Uuid) -> Result<i64, AuthencError> {
        let row = self
            .database
            .query_one(
                "SELECT COUNT(*) as count FROM resource_servers WHERE realm_id = $1",
                &[&realm_id],
            )
            .await
            .map_err(|e| {
                AuthencError::database(format!("Failed to count resource servers: {}", e))
            })?;

        Ok(row.get::<_, i64>("count"))
    }
}
