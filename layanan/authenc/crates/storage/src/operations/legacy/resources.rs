/// Database operations for resources
use crate::Database;
use authenc_types::domain::resource::{CreateResourceRequest, Resource, UpdateResourceRequest};
use authenc_types::{AuthencError, Result};
use chrono::Utc;
use tracing::error;
use uuid::Uuid;

pub async fn create_resource(
    db: &Database,
    request: CreateResourceRequest,
    owner: String,
    realm_id: Uuid,
    resource_server_id: Uuid,
) -> Result<Resource> {
    let resource_id = Uuid::new_v4();
    let now = Utc::now();

    let uris = request.uris.unwrap_or_default();
    let scopes = request.scopes.unwrap_or_default();
    let attributes_json = serde_json::to_value(request.attributes.unwrap_or_default())
        .map_err(|e| AuthencError::validation(format!("Invalid attributes: {}", e)))?;

    let query = r#"
        INSERT INTO resources (
            id, name, display_name, uris, icon_uri, resource_type, owner,
            enabled, realm_id, resource_server_id, scopes, attributes,
            created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        RETURNING
            id, name, display_name, uris, icon_uri, resource_type, owner,
            enabled, realm_id, resource_server_id, scopes, attributes,
            created_at, updated_at
    "#;

    let row: tokio_postgres::Row = db
        .query_one(
            query,
            &[
                &resource_id,
                &request.name,
                &request.display_name,
                &uris,
                &request.icon_uri,
                &request.resource_type,
                &owner,
                &true, // enabled
                &realm_id,
                &resource_server_id,
                &scopes,
                &attributes_json,
                &now,
                &now,
            ],
        )
        .await
        .map_err(|e| {
            error!("Failed to create resource: {}", e);
            AuthencError::database(format!("Failed to create resource: {}", e))
        })?;

    row.try_into()
}

pub async fn get_resource_by_id(db: &Database, resource_id: Uuid) -> Result<Option<Resource>> {
    let query = r#"
        SELECT
            id, name, display_name, uris, icon_uri, resource_type, owner,
            enabled, realm_id, resource_server_id, scopes, attributes,
            created_at, updated_at
        FROM resources
        WHERE id = $1
    "#;

    match db.query_opt(query, &[&resource_id]).await {
        Ok(Some(row)) => Ok(Some(row.try_into()?)),
        Ok(None) => Ok(None),
        Err(e) => {
            error!("Failed to get resource: {}", e);
            Err(AuthencError::database(format!(
                "Failed to get resource: {}",
                e
            )))
        }
    }
}

pub async fn get_resource_by_name(
    db: &Database,
    name: &str,
    resource_server_id: Uuid,
) -> Result<Option<Resource>> {
    let query = r#"
        SELECT
            id, name, display_name, uris, icon_uri, resource_type, owner,
            enabled, realm_id, resource_server_id, scopes, attributes,
            created_at, updated_at
        FROM resources
        WHERE name = $1 AND resource_server_id = $2
    "#;

    match db.query_opt(query, &[&name, &resource_server_id]).await {
        Ok(Some(row)) => Ok(Some(row.try_into()?)),
        Ok(None) => Ok(None),
        Err(e) => {
            error!("Failed to get resource by name: {}", e);
            Err(AuthencError::database(format!(
                "Failed to get resource by name: {}",
                e
            )))
        }
    }
}

pub async fn get_resources_by_owner(
    db: &Database,
    owner: &str,
    first: Option<i32>,
    max: Option<i32>,
) -> Result<Vec<Resource>> {
    let offset = first.unwrap_or(0);
    let limit = max.unwrap_or(100);

    let query = r#"
        SELECT
            id, name, display_name, uris, icon_uri, resource_type, owner,
            enabled, realm_id, resource_server_id, scopes, attributes,
            created_at, updated_at
        FROM resources
        WHERE owner = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
    "#;

    let rows = db
        .query(query, &[&owner, &(limit as i64), &(offset as i64)])
        .await?;
    rows.into_iter()
        .map(|row: tokio_postgres::Row| row.try_into())
        .collect::<Result<Vec<Resource>>>()
}

pub async fn get_resources_by_server(
    db: &Database,
    resource_server_id: Uuid,
    first: Option<i32>,
    max: Option<i32>,
) -> Result<Vec<Resource>> {
    let offset = first.unwrap_or(0);
    let limit = max.unwrap_or(100);

    let query = r#"
        SELECT
            id, name, display_name, uris, icon_uri, resource_type, owner,
            enabled, realm_id, resource_server_id, scopes, attributes,
            created_at, updated_at
        FROM resources
        WHERE resource_server_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
    "#;

    let rows = db
        .query(
            query,
            &[&resource_server_id, &(limit as i64), &(offset as i64)],
        )
        .await?;
    rows.into_iter()
        .map(|row: tokio_postgres::Row| row.try_into())
        .collect::<Result<Vec<Resource>>>()
}

pub async fn get_resources_by_realm(
    db: &Database,
    realm_id: Uuid,
    first: Option<i32>,
    max: Option<i32>,
) -> Result<Vec<Resource>> {
    let offset = first.unwrap_or(0);
    let limit = max.unwrap_or(100);

    let query = r#"
        SELECT
            id, name, display_name, uris, icon_uri, resource_type, owner,
            enabled, realm_id, resource_server_id, scopes, attributes,
            created_at, updated_at
        FROM resources
        WHERE realm_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
    "#;

    let rows = db
        .query(query, &[&realm_id, &(limit as i64), &(offset as i64)])
        .await?;
    rows.into_iter()
        .map(|row: tokio_postgres::Row| row.try_into())
        .collect::<Result<Vec<Resource>>>()
}

pub async fn update_resource(
    db: &Database,
    resource_id: Uuid,
    request: UpdateResourceRequest,
) -> Result<Resource> {
    let now = Utc::now();

    let attributes_json = request
        .attributes
        .map(|a| serde_json::to_value(&a))
        .transpose()
        .map_err(|e| AuthencError::validation(format!("Invalid attributes: {}", e)))?;

    let query = r#"
        UPDATE resources
        SET
            display_name = COALESCE($2, display_name),
            uris = COALESCE($3, uris),
            icon_uri = COALESCE($4, icon_uri),
            resource_type = COALESCE($5, resource_type),
            owner = COALESCE($6, owner),
            scopes = COALESCE($7, scopes),
            attributes = COALESCE($8, attributes),
            updated_at = $9
        WHERE id = $1
        RETURNING
            id, name, display_name, uris, icon_uri, resource_type, owner,
            enabled, realm_id, resource_server_id, scopes, attributes,
            created_at, updated_at
    "#;

    let row: tokio_postgres::Row = db
        .query_one(
            query,
            &[
                &resource_id,
                &request.display_name,
                &request.uris,
                &request.icon_uri,
                &request.resource_type,
                &request.owner,
                &request.scopes,
                &attributes_json,
                &now,
            ],
        )
        .await
        .map_err(|e| {
            error!("Failed to update resource: {}", e);
            AuthencError::database(format!("Failed to update resource: {}", e))
        })?;

    row.try_into()
}

pub async fn delete_resource(db: &Database, resource_id: Uuid) -> Result<()> {
    let query = "DELETE FROM resources WHERE id = $1";

    let rows_affected = db.execute(query, &[&resource_id]).await?;

    if rows_affected == 0 {
        return Err(AuthencError::resource_not_found(format!(
            "Resource {} not found",
            resource_id
        )));
    }

    Ok(())
}

pub async fn search_resources(
    db: &Database,
    name_pattern: &str,
    realm_id: Uuid,
    first: Option<i32>,
    max: Option<i32>,
) -> Result<Vec<Resource>> {
    let offset = first.unwrap_or(0);
    let limit = max.unwrap_or(100);
    let pattern = format!("%{}%", name_pattern);

    let query = r#"
        SELECT
            id, name, display_name, uris, icon_uri, resource_type, owner,
            enabled, realm_id, resource_server_id, scopes, attributes,
            created_at, updated_at
        FROM resources
        WHERE realm_id = $1 AND (name ILIKE $2 OR display_name ILIKE $2)
        ORDER BY created_at DESC
        LIMIT $3 OFFSET $4
    "#;

    let rows = db
        .query(
            query,
            &[&realm_id, &pattern, &(limit as i64), &(offset as i64)],
        )
        .await?;
    rows.into_iter()
        .map(|row: tokio_postgres::Row| row.try_into())
        .collect::<Result<Vec<Resource>>>()
}

pub async fn count_resources(db: &Database, resource_server_id: Uuid) -> Result<i64> {
    let query = "SELECT COUNT(*) FROM resources WHERE resource_server_id = $1";

    let row: tokio_postgres::Row = db.query_one(query, &[&resource_server_id]).await?;
    Ok(row.get(0))
}

pub async fn count_resources_by_owner(db: &Database, owner: &str) -> Result<i64> {
    let query = "SELECT COUNT(*) FROM resources WHERE owner = $1";

    let row: tokio_postgres::Row = db.query_one(query, &[&owner]).await?;
    Ok(row.get(0))
}
