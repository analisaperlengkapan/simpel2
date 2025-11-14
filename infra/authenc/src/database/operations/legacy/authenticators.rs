/// Database operations for custom authenticators
use crate::{database::Database, error::Result};
use chrono::Utc;
use serde_json::Value as JsonValue;
use uuid::Uuid;

pub async fn register_authenticator(
    db: &Database,
    realm_id: Uuid,
    name: String,
    alias: String,
    authenticator_type: String,
    config: JsonValue,
    priority: i32,
) -> Result<Uuid> {
    let query = r#"
        INSERT INTO authenticator_configs (id, realm_id, name, alias, authenticator_type, config, priority, enabled, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, TRUE, $8, $9)
        RETURNING id
    "#;

    let authenticator_id = Uuid::new_v4();
    let now = Utc::now();

    let rows = db
        .query_raw(
            query,
            &[
                &authenticator_id,
                &realm_id,
                &name,
                &alias,
                &authenticator_type,
                &config,
                &priority,
                &now,
                &now,
            ],
        )
        .await?;

    Ok(rows[0].get::<_, Uuid>(0))
}

pub async fn get_realm_authenticators(
    db: &Database,
    realm_id: Uuid,
    enabled_only: bool,
) -> Result<Vec<JsonValue>> {
    let mut query = String::from(
        r#"
        SELECT id, realm_id, name, alias, authenticator_type, config, priority, enabled, created_at, updated_at
        FROM authenticator_configs
        WHERE realm_id = $1
    "#,
    );

    if enabled_only {
        query.push_str(" AND enabled = TRUE");
    }

    query.push_str(" ORDER BY priority ASC");

    let rows = db.query_raw(&query, &[&realm_id]).await?;

    Ok(rows
        .into_iter()
        .map(|row: tokio_postgres::Row| {
            serde_json::json!({
                "id": row.get::<_, Uuid>(0),
                "realm_id": row.get::<_, Uuid>(1),
                "name": row.get::<_, String>(2),
                "alias": row.get::<_, String>(3),
                "authenticator_type": row.get::<_, String>(4),
                "config": row.get::<_, JsonValue>(5),
                "priority": row.get::<_, i32>(6),
                "enabled": row.get::<_, bool>(7),
                "created_at": row.get::<_, chrono::DateTime<Utc>>(8).to_rfc3339(),
                "updated_at": row.get::<_, chrono::DateTime<Utc>>(9).to_rfc3339(),
            })
        })
        .collect())
}

pub async fn create_execution(
    db: &Database,
    realm_id: Uuid,
    flow_id: Uuid,
    authenticator_id: Option<Uuid>,
    requirement: String,
    priority: i32,
    parent_flow_id: Option<Uuid>,
) -> Result<Uuid> {
    let query = r#"
        INSERT INTO authenticator_executions (id, realm_id, flow_id, authenticator_id, requirement, priority, parent_flow_id, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id
    "#;

    let execution_id = Uuid::new_v4();
    let now = Utc::now();

    let rows = db
        .query_raw(
            query,
            &[
                &execution_id,
                &realm_id,
                &flow_id,
                &authenticator_id,
                &requirement,
                &priority,
                &parent_flow_id,
                &now,
                &now,
            ],
        )
        .await?;

    Ok(rows[0].get::<_, Uuid>(0))
}

pub async fn get_flow_executions(db: &Database, flow_id: Uuid) -> Result<Vec<JsonValue>> {
    let query = r#"
        SELECT
            e.id, e.realm_id, e.flow_id, e.authenticator_id, e.requirement,
            e.priority, e.parent_flow_id, e.created_at, e.updated_at,
            a.name as authenticator_name, a.authenticator_type
        FROM authenticator_executions e
        LEFT JOIN authenticator_configs a ON e.authenticator_id = a.id
        WHERE e.flow_id = $1
        ORDER BY e.priority ASC
    "#;

    let rows = db.query_raw(query, &[&flow_id]).await?;

    Ok(rows
        .into_iter()
        .map(|row: tokio_postgres::Row| {
            serde_json::json!({
                "id": row.get::<_, Uuid>(0),
                "realm_id": row.get::<_, Uuid>(1),
                "flow_id": row.get::<_, Uuid>(2),
                "authenticator_id": row.get::<_, Option<Uuid>>(3),
                "requirement": row.get::<_, String>(4),
                "priority": row.get::<_, i32>(5),
                "parent_flow_id": row.get::<_, Option<Uuid>>(6),
                "created_at": row.get::<_, chrono::DateTime<Utc>>(7).to_rfc3339(),
                "updated_at": row.get::<_, chrono::DateTime<Utc>>(8).to_rfc3339(),
                "authenticator_name": row.get::<_, Option<String>>(9),
                "authenticator_type": row.get::<_, Option<String>>(10),
            })
        })
        .collect())
}

pub async fn record_execution_result(
    db: &Database,
    execution_id: Uuid,
    session_id: Option<Uuid>,
    user_id: Option<Uuid>,
    status: String,
    error_message: Option<String>,
    duration_ms: i32,
    attempt_count: i32,
) -> Result<Uuid> {
    let query = r#"
        INSERT INTO authenticator_execution_results (id, execution_id, session_id, user_id, status, error_message, duration_ms, attempt_count, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id
    "#;

    let result_id = Uuid::new_v4();
    let now = Utc::now();

    let rows = db
        .query_raw(
            query,
            &[
                &result_id,
                &execution_id,
                &session_id,
                &user_id,
                &status,
                &error_message,
                &duration_ms,
                &attempt_count,
                &now,
            ],
        )
        .await?;

    Ok(rows[0].get::<_, Uuid>(0))
}

pub async fn update_authenticator_config(
    db: &Database,
    authenticator_id: Uuid,
    config: JsonValue,
) -> Result<()> {
    let query = r#"
        UPDATE authenticator_configs
        SET config = $1, updated_at = $2
        WHERE id = $3
    "#;

    let now = Utc::now();
    db.execute(query, &[&config, &now, &authenticator_id])
        .await?;

    Ok(())
}

pub async fn set_authenticator_enabled(
    db: &Database,
    authenticator_id: Uuid,
    enabled: bool,
) -> Result<()> {
    let query = r#"
        UPDATE authenticator_configs
        SET enabled = $1, updated_at = $2
        WHERE id = $3
    "#;

    let now = Utc::now();
    db.execute(query, &[&enabled, &now, &authenticator_id])
        .await?;

    Ok(())
}

pub async fn delete_authenticator(db: &Database, authenticator_id: Uuid) -> Result<()> {
    let query = r#"
        DELETE FROM authenticator_configs WHERE id = $1
    "#;

    db.execute(query, &[&authenticator_id]).await?;

    Ok(())
}

pub async fn update_execution_requirement(
    db: &Database,
    execution_id: Uuid,
    requirement: String,
) -> Result<()> {
    let query = r#"
        UPDATE authenticator_executions
        SET requirement = $1, updated_at = $2
        WHERE id = $3
    "#;

    let now = Utc::now();
    db.execute(query, &[&requirement, &now, &execution_id])
        .await?;

    Ok(())
}

pub async fn get_execution_statistics(
    db: &Database,
    realm_id: Uuid,
    from_date: Option<chrono::DateTime<Utc>>,
    to_date: Option<chrono::DateTime<Utc>>,
) -> Result<JsonValue> {
    let mut query = String::from(
        r#"
        SELECT
            COUNT(*) as total_attempts,
            COUNT(*) FILTER (WHERE status = 'SUCCESS') as successful_attempts,
            COUNT(*) FILTER (WHERE status = 'FAILED') as failed_attempts,
            COUNT(DISTINCT user_id) as unique_users,
            AVG(duration_ms) as avg_duration_ms
        FROM authenticator_execution_results r
        JOIN authenticator_executions e ON r.execution_id = e.id
        WHERE e.realm_id = $1
    "#,
    );

    let mut param_idx = 2;
    let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&realm_id];

    let from_owned;
    let to_owned;

    if let Some(ref from) = from_date {
        from_owned = *from;
        query.push_str(&format!(" AND r.created_at >= ${}", param_idx));
        params.push(&from_owned);
        param_idx += 1;
    }

    if let Some(ref to) = to_date {
        to_owned = *to;
        query.push_str(&format!(" AND r.created_at <= ${}", param_idx));
        params.push(&to_owned);
    }

    let rows = db.query_raw(&query, &params).await?;

    if rows.is_empty() {
        return Ok(serde_json::json!({
            "total_attempts": 0,
            "successful_attempts": 0,
            "failed_attempts": 0,
            "unique_users": 0,
            "avg_duration_ms": 0,
        }));
    }

    let row = &rows[0];
    Ok(serde_json::json!({
        "total_attempts": row.get::<_, i64>(0),
        "successful_attempts": row.get::<_, i64>(1),
        "failed_attempts": row.get::<_, i64>(2),
        "unique_users": row.get::<_, i64>(3),
        "avg_duration_ms": row.get::<_, Option<f64>>(4).unwrap_or(0.0),
    }))
}
