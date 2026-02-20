/// Database operations for scopes
use crate::Database;
use authenc_types::{AuthencError, Result};
use authenc_core::models::scope::{CreateScopeRequest, Scope, UpdateScopeRequest};
use chrono::Utc;
use tracing::error;
use uuid::Uuid;

pub async fn create_scope(
    db: &Database,
    request: CreateScopeRequest,
    realm_id: Uuid,
    resource_server_id: Uuid,
) -> Result<Scope> {
    let scope_id = Uuid::new_v4();
    let now = Utc::now();

    let query = r#"
        INSERT INTO scopes (
            id, name, display_name, icon_uri, realm_id, resource_server_id,
            created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING
            id, name, display_name, icon_uri, realm_id, resource_server_id,
            created_at, updated_at
    "#;

    let row: tokio_postgres::Row = db
        .query_one(
            query,
            &[
                &scope_id,
                &request.name,
                &request.display_name,
                &request.icon_uri,
                &realm_id,
                &resource_server_id,
                &now,
                &now,
            ],
        )
        .await
        .map_err(|e| {
            error!("Failed to create scope: {}", e);
            AuthencError::database(format!("Failed to create scope: {}", e))
        })?;

    row.try_into()
}

pub async fn get_scope_by_id(db: &Database, id: Uuid) -> Result<Option<Scope>> {
    let query = r#"
        SELECT
            id, name, display_name, icon_uri, realm_id, resource_server_id,
            created_at, updated_at
        FROM scopes
        WHERE id = $1
    "#;

    match db.query_opt(query, &[&id]).await {
        Ok(Some(row)) => Ok(Some(row.try_into()?)),
        Ok(None) => Ok(None),
        Err(e) => {
            error!("Failed to get scope: {}", e);
            Err(AuthencError::database(format!(
                "Failed to get scope: {}",
                e
            )))
        }
    }
}

pub async fn get_scope_by_name(
    db: &Database,
    name: &str,
    resource_server_id: Uuid,
) -> Result<Option<Scope>> {
    let query = r#"
        SELECT
            id, name, display_name, icon_uri, realm_id, resource_server_id,
            created_at, updated_at
        FROM scopes
        WHERE name = $1 AND resource_server_id = $2
    "#;

    match db.query_opt(query, &[&name, &resource_server_id]).await {
        Ok(Some(row)) => Ok(Some(row.try_into()?)),
        Ok(None) => Ok(None),
        Err(e) => {
            error!("Failed to get scope by name: {}", e);
            Err(AuthencError::database(format!(
                "Failed to get scope by name: {}",
                e
            )))
        }
    }
}

pub async fn get_scopes_by_server(
    db: &Database,
    resource_server_id: Uuid,
    first: Option<i32>,
    max: Option<i32>,
) -> Result<Vec<Scope>> {
    let offset = first.unwrap_or(0);
    let limit = max.unwrap_or(100);

    let query = r#"
        SELECT
            id, name, display_name, icon_uri, realm_id, resource_server_id,
            created_at, updated_at
        FROM scopes
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
        .collect::<Result<Vec<Scope>>>()
}

pub async fn get_scopes_by_realm(
    db: &Database,
    realm_id: Uuid,
    first: Option<i32>,
    max: Option<i32>,
) -> Result<Vec<Scope>> {
    let offset = first.unwrap_or(0);
    let limit = max.unwrap_or(100);

    let query = r#"
        SELECT
            id, name, display_name, icon_uri, realm_id, resource_server_id,
            created_at, updated_at
        FROM scopes
        WHERE realm_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
    "#;

    let rows = db
        .query(query, &[&realm_id, &(limit as i64), &(offset as i64)])
        .await?;
    rows.into_iter()
        .map(|row: tokio_postgres::Row| row.try_into())
        .collect::<Result<Vec<Scope>>>()
}

pub async fn update_scope(db: &Database, id: Uuid, request: UpdateScopeRequest) -> Result<Scope> {
    let now = Utc::now();

    let query = r#"
        UPDATE scopes
        SET
            name = COALESCE($2, name),
            display_name = COALESCE($3, display_name),
            icon_uri = COALESCE($4, icon_uri),
            updated_at = $5
        WHERE id = $1
        RETURNING
            id, name, display_name, icon_uri, realm_id, resource_server_id,
            created_at, updated_at
    "#;

    let row = db
        .query_opt(
            query,
            &[
                &id,
                &request.name,
                &request.display_name,
                &request.icon_uri,
                &now,
            ],
        )
        .await
        .map_err(|e| {
            error!("Failed to update scope: {}", e);
            AuthencError::database(format!("Failed to update scope: {}", e))
        })?
        .ok_or_else(|| AuthencError::resource_not_found(format!("Scope {} not found", id)))?;

    row.try_into()
}

pub async fn delete_scope(db: &Database, id: Uuid) -> Result<()> {
    let query = "DELETE FROM scopes WHERE id = $1";

    let rows_affected = db.execute(query, &[&id]).await?;

    if rows_affected == 0 {
        return Err(AuthencError::resource_not_found(format!(
            "Scope {} not found",
            id
        )));
    }

    Ok(())
}

pub async fn search_scopes(
    db: &Database,
    name_pattern: &str,
    realm_id: Uuid,
    first: Option<i32>,
    max: Option<i32>,
) -> Result<Vec<Scope>> {
    let offset = first.unwrap_or(0);
    let limit = max.unwrap_or(100);
    let pattern = format!("%{}%", name_pattern);

    let query = r#"
        SELECT
            id, name, display_name, icon_uri, realm_id, resource_server_id,
            created_at, updated_at
        FROM scopes
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
        .collect::<Result<Vec<Scope>>>()
}

pub async fn count_scopes_by_server(db: &Database, resource_server_id: Uuid) -> Result<i64> {
    let query = "SELECT COUNT(*) FROM scopes WHERE resource_server_id = $1";

    let row: tokio_postgres::Row = db.query_one(query, &[&resource_server_id]).await?;
    Ok(row.get(0))
}
