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
        request: CreateResourceRequest,
        realm_id: Uuid,
        resource_server_id: Uuid,
        owner: String,
    ) -> Result<Resource, AuthencError> {
        let id = Uuid::new_v4();
        let uris = request.uris.unwrap_or_default();
        let scopes = request.scopes.unwrap_or_default();
        let attributes_json = serde_json::to_value(request.attributes.unwrap_or_default())
            .unwrap_or(serde_json::json!({}));

        let row = self
            .database
            .query_one(
                r#"INSERT INTO resources
                    (id, name, display_name, uris, icon_uri, resource_type, owner,
                     enabled, realm_id, resource_server_id, scopes, attributes)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, true, $8, $9, $10, $11)
                   RETURNING *"#,
                &[
                    &id,
                    &request.name,
                    &request.display_name,
                    &uris,
                    &request.icon_uri,
                    &request.resource_type,
                    &owner,
                    &realm_id,
                    &resource_server_id,
                    &scopes,
                    &attributes_json,
                ],
            )
            .await
            .map_err(|e| AuthencError::database(format!("Failed to create resource: {}", e)))?;

        Resource::try_from(row)
    }

    async fn get_resource(&self, id: Uuid) -> Result<Option<Resource>, AuthencError> {
        let row = self
            .database
            .query_opt("SELECT * FROM resources WHERE id = $1", &[&id])
            .await
            .map_err(|e| AuthencError::database(format!("Failed to get resource: {}", e)))?;

        match row {
            Some(r) => Ok(Some(Resource::try_from(r)?)),
            None => Ok(None),
        }
    }

    async fn get_resource_by_name(
        &self,
        name: &str,
        resource_server_id: Uuid,
    ) -> Result<Option<Resource>, AuthencError> {
        let row = self
            .database
            .query_opt(
                "SELECT * FROM resources WHERE name = $1 AND resource_server_id = $2",
                &[&name, &resource_server_id],
            )
            .await
            .map_err(|e| {
                AuthencError::database(format!("Failed to get resource by name: {}", e))
            })?;

        match row {
            Some(r) => Ok(Some(Resource::try_from(r)?)),
            None => Ok(None),
        }
    }

    async fn get_resources_by_owner(
        &self,
        owner: &str,
        first: Option<i32>,
        max: Option<i32>,
    ) -> Result<Vec<Resource>, AuthencError> {
        let offset = first.unwrap_or(0) as i64;
        let limit = max.unwrap_or(100) as i64;

        let rows = self
            .database
            .query(
                "SELECT * FROM resources WHERE owner = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
                &[&owner, &limit, &offset],
            )
            .await
            .map_err(|e| AuthencError::database(format!("Failed to get resources by owner: {}", e)))?;

        rows.into_iter()
            .map(Resource::try_from)
            .collect::<Result<Vec<_>, _>>()
    }

    async fn get_resources_by_server(
        &self,
        resource_server_id: Uuid,
        first: Option<i32>,
        max: Option<i32>,
    ) -> Result<Vec<Resource>, AuthencError> {
        let offset = first.unwrap_or(0) as i64;
        let limit = max.unwrap_or(100) as i64;

        let rows = self
            .database
            .query(
                "SELECT * FROM resources WHERE resource_server_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
                &[&resource_server_id, &limit, &offset],
            )
            .await
            .map_err(|e| AuthencError::database(format!("Failed to get resources by server: {}", e)))?;

        rows.into_iter()
            .map(Resource::try_from)
            .collect::<Result<Vec<_>, _>>()
    }

    async fn get_resources_by_realm(
        &self,
        realm_id: Uuid,
        first: Option<i32>,
        max: Option<i32>,
    ) -> Result<Vec<Resource>, AuthencError> {
        let offset = first.unwrap_or(0) as i64;
        let limit = max.unwrap_or(100) as i64;

        let rows = self
            .database
            .query(
                "SELECT * FROM resources WHERE realm_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
                &[&realm_id, &limit, &offset],
            )
            .await
            .map_err(|e| AuthencError::database(format!("Failed to get resources by realm: {}", e)))?;

        rows.into_iter()
            .map(Resource::try_from)
            .collect::<Result<Vec<_>, _>>()
    }

    async fn update_resource(
        &self,
        id: Uuid,
        request: UpdateResourceRequest,
    ) -> Result<Resource, AuthencError> {
        let existing = self
            .get_resource(id)
            .await?
            .ok_or_else(|| AuthencError::not_found("Resource not found"))?;

        let display_name = request.display_name.or(existing.display_name);
        let uris = request.uris.unwrap_or(existing.uris);
        let icon_uri = request.icon_uri.or(existing.icon_uri);
        let resource_type = request.resource_type.or(existing.resource_type);
        let owner = request.owner.unwrap_or(existing.owner);
        let scopes = request.scopes.unwrap_or(existing.scopes);
        let attributes_json = match request.attributes {
            Some(attrs) => serde_json::to_value(attrs).unwrap_or(serde_json::json!({})),
            None => serde_json::to_value(existing.attributes).unwrap_or(serde_json::json!({})),
        };

        let row = self
            .database
            .query_one(
                r#"UPDATE resources
                   SET display_name = $2, uris = $3, icon_uri = $4, resource_type = $5,
                       owner = $6, scopes = $7, attributes = $8, updated_at = NOW()
                   WHERE id = $1
                   RETURNING *"#,
                &[
                    &id,
                    &display_name,
                    &uris,
                    &icon_uri,
                    &resource_type,
                    &owner,
                    &scopes,
                    &attributes_json,
                ],
            )
            .await
            .map_err(|e| AuthencError::database(format!("Failed to update resource: {}", e)))?;

        Resource::try_from(row)
    }

    async fn delete_resource(&self, id: Uuid) -> Result<(), AuthencError> {
        self.database
            .execute("DELETE FROM resources WHERE id = $1", &[&id])
            .await
            .map_err(|e| AuthencError::database(format!("Failed to delete resource: {}", e)))?;
        Ok(())
    }

    async fn search_resources(
        &self,
        name: &str,
        realm_id: Uuid,
        first: Option<i32>,
        max: Option<i32>,
    ) -> Result<Vec<Resource>, AuthencError> {
        let offset = first.unwrap_or(0) as i64;
        let limit = max.unwrap_or(100) as i64;
        let pattern = format!("%{}%", name);

        let rows = self
            .database
            .query(
                r#"SELECT * FROM resources
                   WHERE realm_id = $1 AND (name ILIKE $2 OR display_name ILIKE $2)
                   ORDER BY created_at DESC LIMIT $3 OFFSET $4"#,
                &[&realm_id, &pattern, &limit, &offset],
            )
            .await
            .map_err(|e| AuthencError::database(format!("Failed to search resources: {}", e)))?;

        rows.into_iter()
            .map(Resource::try_from)
            .collect::<Result<Vec<_>, _>>()
    }

    async fn count_resources_by_owner(&self, owner: &str) -> Result<i64, AuthencError> {
        let row = self
            .database
            .query_one(
                "SELECT COUNT(*) as count FROM resources WHERE owner = $1",
                &[&owner],
            )
            .await
            .map_err(|e| AuthencError::database(format!("Failed to count resources: {}", e)))?;

        Ok(row.get::<_, i64>("count"))
    }
}
