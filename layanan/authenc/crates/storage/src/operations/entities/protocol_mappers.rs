/// Database operations for protocol mappers
use crate::Database;
use authenc_types::Result;
use chrono::Utc;
use serde_json::Value as JsonValue;
use uuid::Uuid;

pub async fn create_protocol_mapper(
    db: &Database,
    client_id: Option<Uuid>,
    realm_id: Uuid,
    name: String,
    protocol: String,
    mapper_type: String,
    config: JsonValue,
) -> Result<Uuid> {
    let query = r#"
        INSERT INTO protocol_mappers (id, client_id, realm_id, name, protocol, mapper_type, config, enabled, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, TRUE, $8, $9)
        RETURNING id
    "#;

    let mapper_id = Uuid::new_v4();
    let now = Utc::now();

    let rows = db
        .query(
            query,
            &[
                &mapper_id,
                &client_id,
                &realm_id,
                &name,
                &protocol,
                &mapper_type,
                &config,
                &now,
                &now,
            ],
        )
        .await?;

    Ok(rows[0].get::<_, Uuid>(0))
}

pub async fn get_client_mappers(
    db: &Database,
    client_id: Uuid,
    protocol: Option<String>,
) -> Result<Vec<JsonValue>> {
    let mut query = String::from(
        r#"
        SELECT id, client_id, realm_id, name, protocol, mapper_type, config, enabled, created_at, updated_at
        FROM protocol_mappers
        WHERE client_id = $1 AND enabled = TRUE
    "#,
    );

    let param_idx = 2;
    let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&client_id];

    let protocol_owned;
    if let Some(ref p) = protocol {
        protocol_owned = p.clone();
        query.push_str(&format!(" AND protocol = ${}", param_idx));
        params.push(&protocol_owned);
    }

    query.push_str(" ORDER BY name");

    let rows = db.query(&query, &params).await?;

    Ok(rows
        .into_iter()
        .map(|row: tokio_postgres::Row| {
            serde_json::json!({
                "id": row.get::<_, Uuid>(0),
                "client_id": row.get::<_, Option<Uuid>>(1),
                "realm_id": row.get::<_, Uuid>(2),
                "name": row.get::<_, String>(3),
                "protocol": row.get::<_, String>(4),
                "mapper_type": row.get::<_, String>(5),
                "config": row.get::<_, JsonValue>(6),
                "enabled": row.get::<_, bool>(7),
                "created_at": row.get::<_, chrono::DateTime<Utc>>(8).to_rfc3339(),
                "updated_at": row.get::<_, chrono::DateTime<Utc>>(9).to_rfc3339(),
            })
        })
        .collect())
}

pub async fn get_realm_mappers(
    db: &Database,
    realm_id: Uuid,
    protocol: Option<String>,
) -> Result<Vec<JsonValue>> {
    let mut query = String::from(
        r#"
        SELECT id, client_id, realm_id, name, protocol, mapper_type, config, enabled, created_at, updated_at
        FROM protocol_mappers
        WHERE realm_id = $1 AND client_id IS NULL AND enabled = TRUE
    "#,
    );

    let param_idx = 2;
    let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&realm_id];

    let protocol_owned;
    if let Some(ref p) = protocol {
        protocol_owned = p.clone();
        query.push_str(&format!(" AND protocol = ${}", param_idx));
        params.push(&protocol_owned);
    }

    query.push_str(" ORDER BY name");

    let rows = db.query(&query, &params).await?;

    Ok(rows
        .into_iter()
        .map(|row: tokio_postgres::Row| {
            serde_json::json!({
                "id": row.get::<_, Uuid>(0),
                "client_id": row.get::<_, Option<Uuid>>(1),
                "realm_id": row.get::<_, Uuid>(2),
                "name": row.get::<_, String>(3),
                "protocol": row.get::<_, String>(4),
                "mapper_type": row.get::<_, String>(5),
                "config": row.get::<_, JsonValue>(6),
                "enabled": row.get::<_, bool>(7),
                "created_at": row.get::<_, chrono::DateTime<Utc>>(8).to_rfc3339(),
                "updated_at": row.get::<_, chrono::DateTime<Utc>>(9).to_rfc3339(),
            })
        })
        .collect())
}

pub async fn update_mapper_config(db: &Database, mapper_id: Uuid, config: JsonValue) -> Result<()> {
    let query = r#"
        UPDATE protocol_mappers
        SET config = $1, updated_at = $2
        WHERE id = $3
    "#;

    let now = Utc::now();
    db.execute(query, &[&config, &now, &mapper_id]).await?;

    Ok(())
}

pub async fn set_mapper_enabled(db: &Database, mapper_id: Uuid, enabled: bool) -> Result<()> {
    let query = r#"
        UPDATE protocol_mappers
        SET enabled = $1, updated_at = $2
        WHERE id = $3
    "#;

    let now = Utc::now();
    db.execute(query, &[&enabled, &now, &mapper_id]).await?;

    Ok(())
}

pub async fn delete_mapper(db: &Database, mapper_id: Uuid) -> Result<()> {
    let query = r#"
        DELETE FROM protocol_mappers WHERE id = $1
    "#;

    db.execute(query, &[&mapper_id]).await?;

    Ok(())
}

pub async fn get_mapper_by_id(db: &Database, mapper_id: Uuid) -> Result<Option<JsonValue>> {
    let query = r#"
        SELECT id, client_id, realm_id, name, protocol, mapper_type, config, enabled, created_at, updated_at
        FROM protocol_mappers
        WHERE id = $1
    "#;

    let rows = db.query(query, &[&mapper_id]).await?;

    if rows.is_empty() {
        return Ok(None);
    }

    let row = &rows[0];
    Ok(Some(serde_json::json!({
        "id": row.get::<_, Uuid>(0),
        "client_id": row.get::<_, Option<Uuid>>(1),
        "realm_id": row.get::<_, Uuid>(2),
        "name": row.get::<_, String>(3),
        "protocol": row.get::<_, String>(4),
        "mapper_type": row.get::<_, String>(5),
        "config": row.get::<_, JsonValue>(6),
        "enabled": row.get::<_, bool>(7),
        "created_at": row.get::<_, chrono::DateTime<Utc>>(8).to_rfc3339(),
        "updated_at": row.get::<_, chrono::DateTime<Utc>>(9).to_rfc3339(),
    })))
}

pub async fn get_mapper_statistics(db: &Database, realm_id: Uuid) -> Result<JsonValue> {
    let query = r#"
        SELECT
            COUNT(*) FILTER (WHERE enabled = TRUE) as total_enabled,
            COUNT(*) FILTER (WHERE enabled = FALSE) as total_disabled,
            COUNT(DISTINCT protocol) as unique_protocols,
            COUNT(DISTINCT mapper_type) as unique_types,
            COUNT(*) FILTER (WHERE client_id IS NULL) as realm_level_mappers,
            COUNT(*) FILTER (WHERE client_id IS NOT NULL) as client_level_mappers
        FROM protocol_mappers
        WHERE realm_id = $1
    "#;

    let rows = db.query(query, &[&realm_id]).await?;

    if rows.is_empty() {
        return Ok(serde_json::json!({
            "total_enabled": 0,
            "total_disabled": 0,
            "unique_protocols": 0,
            "unique_types": 0,
            "realm_level_mappers": 0,
            "client_level_mappers": 0,
        }));
    }

    let row = &rows[0];
    Ok(serde_json::json!({
        "total_enabled": row.get::<_, i64>(0),
        "total_disabled": row.get::<_, i64>(1),
        "unique_protocols": row.get::<_, i64>(2),
        "unique_types": row.get::<_, i64>(3),
        "realm_level_mappers": row.get::<_, i64>(4),
        "client_level_mappers": row.get::<_, i64>(5),
    }))
}
