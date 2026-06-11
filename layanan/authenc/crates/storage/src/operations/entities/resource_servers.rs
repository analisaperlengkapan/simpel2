/// Resource server operations
use crate::{
    database::Database,
    error::{AuthencError, Result},
    models::resource_server::{
        CreateResourceServerRequest, ResourceServer, UpdateResourceServerRequest,
    },
};
use uuid::Uuid;

pub async fn create_resource_server(
    db: &Database,
    request: CreateResourceServerRequest,
    realm_id: Uuid,
) -> Result<ResourceServer> {
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();

    let query = r#"
        INSERT INTO resource_servers (
            id, client_id, name, description, enabled, realm_id,
            policy_enforcement_mode, decision_strategy, allow_remote_resource_management,
            created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING
            id, client_id, name, description, enabled, realm_id,
            policy_enforcement_mode, decision_strategy, allow_remote_resource_management,
            created_at, updated_at
    "#;

    let policy_mode = "enforcing"; // Default
    let decision_strat = "unanimous"; // Default

    let row: tokio_postgres::Row = db
        .query_one(
            query,
            &[
                &id,
                &request.client_id,
                &request.name,
                &request.description,
                &true, // enabled by default
                &realm_id,
                &policy_mode,
                &decision_strat,
                &false, // allow_remote_resource_management default
                &now,
                &now,
            ],
        )
        .await?;

    row.try_into()
}

pub async fn get_resource_server_by_id(db: &Database, id: Uuid) -> Result<Option<ResourceServer>> {
    let query = r#"
        SELECT
            id, client_id, name, description, enabled, realm_id,
            policy_enforcement_mode, decision_strategy, allow_remote_resource_management,
            created_at, updated_at
        FROM resource_servers
        WHERE id = $1
    "#;

    match db.query_opt(query, &[&id]).await? {
        Some(row) => Ok(Some(row.try_into()?)),
        None => Ok(None),
    }
}

pub async fn get_resource_server_by_client(
    db: &Database,
    client_id: &str,
    realm_id: Uuid,
) -> Result<Option<ResourceServer>> {
    let query = r#"
        SELECT
            id, client_id, name, description, enabled, realm_id,
            policy_enforcement_mode, decision_strategy, allow_remote_resource_management,
            created_at, updated_at
        FROM resource_servers
        WHERE client_id = $1 AND realm_id = $2
    "#;

    match db.query_opt(query, &[&client_id, &realm_id]).await? {
        Some(row) => Ok(Some(row.try_into()?)),
        None => Ok(None),
    }
}

pub async fn get_resource_servers_by_realm(
    db: &Database,
    realm_id: Uuid,
    first: Option<i32>,
    max: Option<i32>,
) -> Result<Vec<ResourceServer>> {
    let offset = first.unwrap_or(0);
    let limit = max.unwrap_or(100);

    let query = r#"
        SELECT
            id, client_id, name, description, enabled, realm_id,
            policy_enforcement_mode, decision_strategy, allow_remote_resource_management,
            created_at, updated_at
        FROM resource_servers
        WHERE realm_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
    "#;

    let rows = db
        .query(query, &[&realm_id, &(limit as i64), &(offset as i64)])
        .await?;
    rows.into_iter()
        .map(|row: tokio_postgres::Row| row.try_into())
        .collect::<Result<Vec<ResourceServer>>>()
}

pub async fn update_resource_server(
    db: &Database,
    id: Uuid,
    request: UpdateResourceServerRequest,
) -> Result<ResourceServer> {
    let now = chrono::Utc::now();

    let query = r#"
        UPDATE resource_servers
        SET
            name = COALESCE($2, name),
            description = COALESCE($3, description),
            policy_enforcement_mode = COALESCE($4, policy_enforcement_mode),
            decision_strategy = COALESCE($5, decision_strategy),
            allow_remote_resource_management = COALESCE($6, allow_remote_resource_management),
            updated_at = $7
        WHERE id = $1
        RETURNING
            id, client_id, name, description, enabled, realm_id,
            policy_enforcement_mode, decision_strategy, allow_remote_resource_management,
            created_at, updated_at
    "#;

    let policy_mode = request.policy_enforcement_mode.as_ref().map(|m| m.as_str());
    let decision_strat = request.decision_strategy.as_ref().map(|s| s.as_str());

    match db
        .query_opt(
            query,
            &[
                &id,
                &request.name,
                &request.description,
                &policy_mode,
                &decision_strat,
                &request.allow_remote_resource_management,
                &now,
            ],
        )
        .await?
    {
        Some(row) => row.try_into(),
        None => Err(AuthencError::not_found(format!(
            "Resource server with id {} not found",
            id
        ))),
    }
}

pub async fn delete_resource_server(db: &Database, id: Uuid) -> Result<()> {
    let query = "DELETE FROM resource_servers WHERE id = $1";

    let rows_affected = db.execute(query, &[&id]).await?;

    if rows_affected == 0 {
        return Err(AuthencError::not_found(format!(
            "Resource server with id {} not found",
            id
        )));
    }

    Ok(())
}

pub async fn search_resource_servers(
    db: &Database,
    name_pattern: &str,
    realm_id: Uuid,
    first: Option<i32>,
    max: Option<i32>,
) -> Result<Vec<ResourceServer>> {
    let offset = first.unwrap_or(0);
    let limit = max.unwrap_or(100);
    let pattern = format!("%{}%", name_pattern);

    let query = r#"
        SELECT
            id, client_id, name, description, enabled, realm_id,
            policy_enforcement_mode, decision_strategy, allow_remote_resource_management,
            created_at, updated_at
        FROM resource_servers
        WHERE realm_id = $1 AND (name ILIKE $2 OR client_id ILIKE $2)
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
        .collect::<Result<Vec<ResourceServer>>>()
}

pub async fn count_resource_servers_by_realm(db: &Database, realm_id: Uuid) -> Result<i64> {
    let query = "SELECT COUNT(*) FROM resource_servers WHERE realm_id = $1";

    let row: tokio_postgres::Row = db.query_one(query, &[&realm_id]).await?;
    Ok(row.get(0))
}
