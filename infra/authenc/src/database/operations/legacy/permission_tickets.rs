/// Database operations for permission tickets
use crate::database::Database;
use crate::error::{AuthencError, Result};
use crate::models::permission_ticket::{
    CreatePermissionTicketRequest, PermissionTicket, PermissionTicketFilter,
};
use chrono::Utc;
use log::error;
use uuid::Uuid;

pub async fn create_permission_ticket(
    db: &Database,
    request: CreatePermissionTicketRequest,
    owner: String,
    realm_id: Uuid,
    resource_server_id: Uuid,
) -> Result<PermissionTicket> {
    let ticket_id = Uuid::new_v4();
    let now = Utc::now();

    let query = r#"
        INSERT INTO permission_tickets (
            id, resource_id, scope_id, owner, requester, granted,
            granted_timestamp, realm_id, resource_server_id,
            created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING
            id, resource_id, scope_id, owner, requester, granted,
            granted_timestamp, realm_id, resource_server_id,
            created_at, updated_at
    "#;

    let row: tokio_postgres::Row = db
        .query_one(
            query,
            &[
                &ticket_id,
                &request.resource_id,
                &request.scope_id,
                &owner,
                &request.requester,
                &false,                         // granted
                &None::<chrono::DateTime<Utc>>, // granted_timestamp
                &realm_id,
                &resource_server_id,
                &now,
                &now,
            ],
        )
        .await
        .map_err(|e| {
            error!("Failed to create permission ticket: {}", e);
            AuthencError::database(format!("Failed to create permission ticket: {}", e))
        })?;

    row.try_into()
}

pub async fn get_permission_ticket(db: &Database, id: Uuid) -> Result<Option<PermissionTicket>> {
    let query = r#"
        SELECT
            id, resource_id, scope_id, owner, requester, granted,
            granted_timestamp, realm_id, resource_server_id,
            created_at, updated_at
        FROM permission_tickets
        WHERE id = $1
    "#;

    match db.query_opt(query, &[&id]).await {
        Ok(Some(row)) => Ok(Some(row.try_into()?)),
        Ok(None) => Ok(None),
        Err(e) => {
            error!("Failed to get permission ticket: {}", e);
            Err(AuthencError::database(format!(
                "Failed to get permission ticket: {}",
                e
            )))
        }
    }
}

pub async fn get_permission_tickets(
    db: &Database,
    filters: Vec<PermissionTicketFilter>,
    first: Option<i32>,
    max: Option<i32>,
) -> Result<Vec<PermissionTicket>> {
    let offset = first.unwrap_or(0);
    let limit = max.unwrap_or(100);

    let mut conditions = Vec::new();
    let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();
    let mut param_index = 1;

    for filter in &filters {
        match filter {
            PermissionTicketFilter::Owner(owner) => {
                conditions.push(format!("owner = ${}", param_index));
                params.push(owner);
                param_index += 1;
            }
            PermissionTicketFilter::Requester(requester) => {
                conditions.push(format!("requester = ${}", param_index));
                params.push(requester);
                param_index += 1;
            }
            PermissionTicketFilter::ResourceId(resource_id) => {
                conditions.push(format!("resource_id = ${}", param_index));
                params.push(resource_id);
                param_index += 1;
            }
            PermissionTicketFilter::Granted(granted) => {
                conditions.push(format!("granted = ${}", param_index));
                params.push(granted);
                param_index += 1;
            }
            PermissionTicketFilter::ResourceServerId(resource_server_id) => {
                conditions.push(format!("resource_server_id = ${}", param_index));
                params.push(resource_server_id);
                param_index += 1;
            }
        }
    }

    let where_clause = if conditions.is_empty() {
        String::from("TRUE")
    } else {
        conditions.join(" AND ")
    };

    let limit_i64 = limit as i64;
    let offset_i64 = offset as i64;

    let query = format!(
        r#"
        SELECT
            id, resource_id, scope_id, owner, requester, granted,
            granted_timestamp, realm_id, resource_server_id,
            created_at, updated_at
        FROM permission_tickets
        WHERE {}
        ORDER BY created_at DESC
        LIMIT ${} OFFSET ${}
        "#,
        where_clause,
        param_index,
        param_index + 1
    );

    params.push(&limit_i64);
    params.push(&offset_i64);

    let rows = db.query(&query, &params).await?;
    rows.into_iter()
        .map(|row: tokio_postgres::Row| row.try_into())
        .collect::<Result<Vec<PermissionTicket>>>()
}

pub async fn get_granted_resources(
    db: &Database,
    user_id: &str,
    name_filter: Option<&str>,
    first: Option<i32>,
    max: Option<i32>,
) -> Result<Vec<Uuid>> {
    let offset = first.unwrap_or(0);
    let limit = max.unwrap_or(100);

    let query = if let Some(name_pattern) = name_filter {
        r#"
            SELECT DISTINCT pt.resource_id
            FROM permission_tickets pt
            JOIN resources r ON pt.resource_id = r.id
            WHERE pt.requester = $1 AND pt.granted = true
              AND r.name ILIKE $2
            ORDER BY pt.resource_id
            LIMIT $3 OFFSET $4
        "#
    } else {
        r#"
            SELECT DISTINCT resource_id
            FROM permission_tickets
            WHERE requester = $1 AND granted = true
            ORDER BY resource_id
            LIMIT $2 OFFSET $3
        "#
    };

    let rows = if name_filter.is_some() {
        let pattern = format!("%{}%", name_filter.unwrap());
        db.query(
            query,
            &[&user_id, &pattern, &(limit as i64), &(offset as i64)],
        )
        .await?
    } else {
        db.query(query, &[&user_id, &(limit as i64), &(offset as i64)])
            .await?
    };

    Ok(rows
        .into_iter()
        .map(|row: tokio_postgres::Row| row.get(0))
        .collect())
}

pub async fn get_granted_owner_resources(
    db: &Database,
    owner: &str,
    first: Option<i32>,
    max: Option<i32>,
) -> Result<Vec<Uuid>> {
    let offset = first.unwrap_or(0);
    let limit = max.unwrap_or(100);

    let query = r#"
        SELECT DISTINCT resource_id
        FROM permission_tickets
        WHERE owner = $1 AND granted = true
        ORDER BY resource_id
        LIMIT $2 OFFSET $3
    "#;

    let rows = db
        .query(query, &[&owner, &(limit as i64), &(offset as i64)])
        .await?;
    Ok(rows
        .into_iter()
        .map(|row: tokio_postgres::Row| row.get::<_, Uuid>(0))
        .collect())
}

pub async fn get_tickets_for_resource(
    db: &Database,
    resource_id: Uuid,
    granted: Option<bool>,
) -> Result<Vec<PermissionTicket>> {
    let query = if let Some(granted_filter) = granted {
        r#"
            SELECT
                id, resource_id, scope_id, owner, requester, granted,
                granted_timestamp, realm_id, resource_server_id,
                created_at, updated_at
            FROM permission_tickets
            WHERE resource_id = $1 AND granted = $2
            ORDER BY created_at DESC
        "#
    } else {
        r#"
            SELECT
                id, resource_id, scope_id, owner, requester, granted,
                granted_timestamp, realm_id, resource_server_id,
                created_at, updated_at
            FROM permission_tickets
            WHERE resource_id = $1
            ORDER BY created_at DESC
        "#
    };

    let rows = if let Some(granted_filter) = granted {
        db.query(query, &[&resource_id, &granted_filter]).await?
    } else {
        db.query(query, &[&resource_id]).await?
    };

    rows.into_iter()
        .map(|row: tokio_postgres::Row| row.try_into())
        .collect::<Result<Vec<PermissionTicket>>>()
}

pub async fn get_tickets_for_requester(
    db: &Database,
    requester: &str,
    granted: Option<bool>,
) -> Result<Vec<PermissionTicket>> {
    let query = if let Some(granted_filter) = granted {
        r#"
            SELECT
                id, resource_id, scope_id, owner, requester, granted,
                granted_timestamp, realm_id, resource_server_id,
                created_at, updated_at
            FROM permission_tickets
            WHERE requester = $1 AND granted = $2
            ORDER BY created_at DESC
        "#
    } else {
        r#"
            SELECT
                id, resource_id, scope_id, owner, requester, granted,
                granted_timestamp, realm_id, resource_server_id,
                created_at, updated_at
            FROM permission_tickets
            WHERE requester = $1
            ORDER BY created_at DESC
        "#
    };

    let rows = if let Some(granted_filter) = granted {
        db.query(query, &[&requester, &granted_filter]).await?
    } else {
        db.query(query, &[&requester]).await?
    };

    rows.into_iter()
        .map(|row: tokio_postgres::Row| row.try_into())
        .collect::<Result<Vec<PermissionTicket>>>()
}

pub async fn grant_permission_ticket(db: &Database, id: Uuid) -> Result<PermissionTicket> {
    let now = Utc::now();

    let query = r#"
        UPDATE permission_tickets
        SET granted = true, granted_timestamp = $2, updated_at = $3
        WHERE id = $1
        RETURNING
            id, resource_id, scope_id, owner, requester, granted,
            granted_timestamp, realm_id, resource_server_id,
            created_at, updated_at
    "#;

    let row = db
        .query_opt(query, &[&id, &now, &now])
        .await
        .map_err(|e| {
            error!("Failed to grant permission ticket: {}", e);
            AuthencError::database(format!("Failed to grant permission ticket: {}", e))
        })?
        .ok_or_else(|| {
            AuthencError::resource_not_found(format!("Permission ticket {} not found", id))
        })?;

    row.try_into()
}

pub async fn revoke_permission_ticket(db: &Database, id: Uuid) -> Result<PermissionTicket> {
    let now = Utc::now();

    let query = r#"
        UPDATE permission_tickets
        SET granted = false, granted_timestamp = NULL, updated_at = $2
        WHERE id = $1
        RETURNING
            id, resource_id, scope_id, owner, requester, granted,
            granted_timestamp, realm_id, resource_server_id,
            created_at, updated_at
    "#;

    let row = db
        .query_opt(query, &[&id, &now])
        .await
        .map_err(|e| {
            error!("Failed to revoke permission ticket: {}", e);
            AuthencError::database(format!("Failed to revoke permission ticket: {}", e))
        })?
        .ok_or_else(|| {
            AuthencError::resource_not_found(format!("Permission ticket {} not found", id))
        })?;

    row.try_into()
}

pub async fn delete_permission_ticket(db: &Database, id: Uuid) -> Result<()> {
    let query = "DELETE FROM permission_tickets WHERE id = $1";

    let rows_affected = db.execute(query, &[&id]).await?;

    if rows_affected == 0 {
        return Err(AuthencError::resource_not_found(format!(
            "Permission ticket {} not found",
            id
        )));
    }

    Ok(())
}

pub async fn count_permission_tickets(
    db: &Database,
    filters: Vec<PermissionTicketFilter>,
) -> Result<i64> {
    let mut conditions = Vec::new();
    let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();
    let mut param_index = 1;

    for filter in &filters {
        match filter {
            PermissionTicketFilter::Owner(owner) => {
                conditions.push(format!("owner = ${}", param_index));
                params.push(owner);
                param_index += 1;
            }
            PermissionTicketFilter::Requester(requester) => {
                conditions.push(format!("requester = ${}", param_index));
                params.push(requester);
                param_index += 1;
            }
            PermissionTicketFilter::ResourceId(resource_id) => {
                conditions.push(format!("resource_id = ${}", param_index));
                params.push(resource_id);
                param_index += 1;
            }
            PermissionTicketFilter::Granted(granted) => {
                conditions.push(format!("granted = ${}", param_index));
                params.push(granted);
                param_index += 1;
            }
            PermissionTicketFilter::ResourceServerId(resource_server_id) => {
                conditions.push(format!("resource_server_id = ${}", param_index));
                params.push(resource_server_id);
                param_index += 1;
            }
        }
    }

    let where_clause = if conditions.is_empty() {
        String::from("TRUE")
    } else {
        conditions.join(" AND ")
    };

    let query = format!(
        "SELECT COUNT(*) FROM permission_tickets WHERE {}",
        where_clause
    );

    let row: tokio_postgres::Row = db.query_one(&query, &params).await?;
    Ok(row.get(0))
}
