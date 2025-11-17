//! Top-level event functions for database operations
//! These functions were at the top level of operations_legacy.rs

use crate::{
    database::Database,
    error::{AuthencError, Result},
    models::events::{AdminEvent, AuthDetails, Event, EventType, OperationType, ResourceType},
    spi::events::{AdminEventOperationType, AdminEventQuery, EventQuery},
};

use log::error;
use serde_json;
use std::collections::HashMap;
use uuid::Uuid;

/// Store a user event in the database
pub async fn store_event(db: &Database, event: &Event) -> Result<()> {
    let details_json = serde_json::to_string(&event.details).map_err(|e| {
        error!("Failed to serialize event details: {}", e);
        AuthencError::validation("Failed to serialize event details")
    })?;

    let query = r#"
            INSERT INTO events (
                id, time, event_type, realm_id, realm_name, client_id,
                user_id, session_id, ip_address, error, details
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#;

    db.execute(
        query,
        &[
            &Uuid::parse_str(&event.id)
                .map_err(|_| AuthencError::validation("Invalid event ID"))?,
            &event.time,
            &event.event_type.as_str(),
            &event.realm_id,
            &event.realm_name,
            &event.client_id,
            &event.user_id,
            &event.session_id,
            &event.ip_address,
            &event.error,
            &details_json,
        ],
    )
    .await
    .map_err(|e| {
        error!("Failed to store event: {}", e);
        AuthencError::database("Failed to store event")
    })?;

    Ok(())
}

/// Store an admin event in the database
pub async fn store_admin_event(db: &Database, event: &AdminEvent) -> Result<()> {
    let query = r#"
            INSERT INTO admin_events (
                id, time, realm_id, realm_name, auth_user_id, auth_username,
                auth_ip_address, auth_user_agent, resource_type, operation_type,
                resource_path, representation, error
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        "#;

    db.execute(
        query,
        &[
            &Uuid::parse_str(&event.id)
                .map_err(|_| AuthencError::validation("Invalid admin event ID"))?,
            &event.time,
            &event.realm_id,
            &event.realm_name,
            &Uuid::parse_str(&event.auth_details.user_id)
                .map_err(|_| AuthencError::validation("Invalid auth user ID"))?,
            &event.auth_details.username,
            &event.auth_details.ip_address,
            &event.auth_details.user_agent,
            &event.resource_type.as_str(),
            &event.operation_type.as_str(),
            &event.resource_path,
            &event.representation,
            &event.error,
        ],
    )
    .await
    .map_err(|e| {
        error!("Failed to store admin event: {}", e);
        AuthencError::database("Failed to store admin event")
    })?;

    Ok(())
}

/// Query events based on the provided query parameters
pub async fn query_events(db: &Database, query: &EventQuery) -> Result<Vec<Event>> {
    let mut conditions = Vec::new();
    let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = Vec::new();
    let mut param_index = 1;

    // Build WHERE conditions
    if let Some(realm_id) = &query.realm_id {
        conditions.push(format!("realm_id = ${}", param_index));
        params.push(Box::new(realm_id.clone()));
        param_index += 1;
    }

    if let Some(user_id) = &query.user_id {
        conditions.push(format!("user_id = ${}", param_index));
        params.push(Box::new(user_id.clone()));
        param_index += 1;
    }

    if let Some(client_id) = &query.client_id {
        conditions.push(format!("client_id = ${}", param_index));
        params.push(Box::new(client_id.clone()));
        param_index += 1;
    }

    if let Some(event_types) = &query.event_types {
        if let Some(event_type) = event_types.first() {
            conditions.push(format!("event_type = ${}", param_index));
            params.push(Box::new(event_type.as_str()));
            param_index += 1;
        }
    }

    if let Some(from_date) = &query.date_from {
        conditions.push(format!("time >= ${}", param_index));
        params.push(Box::new(from_date.clone()));
        param_index += 1;
    }

    if let Some(to_date) = &query.date_to {
        conditions.push(format!("time <= ${}", param_index));
        params.push(Box::new(to_date.clone()));
        param_index += 1;
    }

    if let Some(ip_address) = &query.ip_address {
        conditions.push(format!("ip_address = ${}", param_index));
        params.push(Box::new(ip_address.clone()));
        param_index += 1;
    }

    // SECURITY: This is SAFE from SQL injection because:
    // 1. The where_clause only contains parameterized query placeholders ($1, $2, etc.)
    // 2. Actual user data is passed via params vector and properly escaped by tokio_postgres
    // 3. The format! is only building query structure, NOT interpolating user data
    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let limit = query.max_results.unwrap_or(100).min(1000);
    let offset = query.first_result.unwrap_or(0);

    // SECURITY NOTE: Using format! here is safe because:
    // - where_clause contains only SQL structure with $N placeholders
    // - param_index values are integers generated internally
    // - All user data goes through parameterized queries (params vector)
    let query_sql = format!(
        r#"
            SELECT
                id, time, event_type, realm_id, realm_name, client_id,
                user_id, session_id, ip_address, error, details
            FROM events
            {}
            ORDER BY time DESC
            LIMIT ${} OFFSET ${}
            "#,
        where_clause,
        param_index,
        param_index + 1
    );

    params.push(Box::new(limit as i64));
    params.push(Box::new(offset as i64));

    let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
        .iter()
        .map(|p| &**p as &(dyn tokio_postgres::types::ToSql + Sync))
        .collect();

    let rows: Vec<tokio_postgres::Row> = db
        .query(&query_sql, param_refs.as_slice())
        .await
        .map_err(|e| {
            error!("Failed to query events: {}", e);
            AuthencError::database("Failed to query events")
        })?;

    let mut events = Vec::new();
    for row in rows {
        let details_json: String = row.get(10);
        let details: HashMap<String, String> =
            serde_json::from_str(&details_json).unwrap_or_default();

        let event_type_str = row.get::<_, String>(2);
        match EventType::from_str(&event_type_str) {
            Ok(event_type) => {
                events.push(Event {
                    id: row.get::<_, Uuid>(0).to_string(),
                    time: row.get(1),
                    event_type,
                    realm_id: row.get(3),
                    realm_name: row.get(4),
                    client_id: row.get(5),
                    user_id: row.get(6),
                    session_id: row.get(7),
                    ip_address: row.get(8),
                    error: row.get(9),
                    details,
                });
            }
            Err(_) => {
                error!(
                    "Unknown event type '{}' in event row with id '{}'. Skipping event.",
                    event_type_str,
                    row.get::<_, Uuid>(0)
                );
                // Optionally, you could collect these in a separate vector for reporting
                continue;
            }
        }

    Ok(events)
}

/// Query admin events based on the provided query parameters
pub async fn query_admin_events(db: &Database, query: &AdminEventQuery) -> Result<Vec<AdminEvent>> {
    let mut conditions = Vec::new();
    let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = Vec::new();
    let mut param_index = 1;

    // Build WHERE conditions
    if let Some(realm_id) = &query.realm_id {
        conditions.push(format!("realm_id = ${}", param_index));
        params.push(Box::new(realm_id.clone()));
        param_index += 1;
    }

    if let Some(auth_user_id) = &query.auth_user_id {
        conditions.push(format!("auth_user_id = ${}", param_index));
        params
            .push(Box::new(Uuid::parse_str(auth_user_id).map_err(|_| {
                AuthencError::validation("Invalid auth user ID")
            })?));
        param_index += 1;
    }

    if let Some(resource_type) = &query.resource_type {
        conditions.push(format!("resource_type = ${}", param_index));
        params.push(Box::new(resource_type.as_str()));
        param_index += 1;
    }

    if let Some(operation_type) = &query.operation_type {
        conditions.push(format!("operation_type = ${}", param_index));
        params.push(Box::new(match operation_type {
            AdminEventOperationType::Create => "CREATE",
            AdminEventOperationType::Update => "UPDATE",
            AdminEventOperationType::Delete => "DELETE",
            AdminEventOperationType::Action => "ACTION",
        }));
        param_index += 1;
    }

    if let Some(from_date) = &query.date_from {
        conditions.push(format!("time >= ${}", param_index));
        params.push(Box::new(from_date.clone()));
        param_index += 1;
    }

    if let Some(to_date) = &query.date_to {
        conditions.push(format!("time <= ${}", param_index));
        params.push(Box::new(to_date.clone()));
        param_index += 1;
    }

    if let Some(resource_path) = &query.resource_type {
        conditions.push(format!("resource_path LIKE ${}", param_index));
        params.push(Box::new(format!("%{}%", resource_path)));
        param_index += 1;
    }

    // SECURITY: This is SAFE from SQL injection because:
    // 1. The where_clause only contains parameterized query placeholders ($1, $2, etc.)
    // 2. Actual user data is passed via params vector and properly escaped by tokio_postgres
    // 3. The format! is only building query structure, NOT interpolating user data
    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let limit = query.max_results.unwrap_or(100).min(1000);
    let offset = query.first_result.unwrap_or(0);

    // SECURITY NOTE: Using format! here is safe because:
    // - where_clause contains only SQL structure with $N placeholders
    // - param_index values are integers generated internally
    // - All user data goes through parameterized queries (params vector)
    let query_sql = format!(
        r#"
            SELECT
                id, time, realm_id, realm_name, auth_user_id, auth_username,
                auth_ip_address, auth_user_agent, resource_type, operation_type,
                resource_path, representation, error
            FROM admin_events
            {}
            ORDER BY time DESC
            LIMIT ${} OFFSET ${}
            "#,
        where_clause,
        param_index,
        param_index + 1
    );

    params.push(Box::new(limit as i64));
    params.push(Box::new(offset as i64));

    let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
        .iter()
        .map(|p| &**p as &(dyn tokio_postgres::types::ToSql + Sync))
        .collect();

    let rows: Vec<tokio_postgres::Row> = db
        .query(&query_sql, param_refs.as_slice())
        .await
        .map_err(|e| {
            error!("Failed to query admin events: {}", e);
            AuthencError::database("Failed to query admin events")
        })?;

    let mut events = Vec::new();
    for row in rows {
        events.push(AdminEvent {
            id: row.get::<_, Uuid>(0).to_string(),
            time: row.get(1),
            realm_id: row.get(2),
            realm_name: row.get(3),
            auth_details: AuthDetails {
                user_id: row
                    .get::<_, Option<Uuid>>(4)
                    .map(|id| id.to_string())
                    .unwrap_or_default(),
                username: row.get(5),
                ip_address: row.get(6),
                user_agent: row.get(7),
            },
            resource_type: ResourceType::from_str(&row.get::<_, String>(8))
                .unwrap_or(ResourceType::User),
            operation_type: OperationType::from_str(&row.get::<_, String>(9))
                .unwrap_or(OperationType::Create),
            resource_path: row.get(10),
            representation: row.get(11),
            error: row.get(12),
        });
    }

    Ok(events)
}

/// Clear old events based on retention policy
pub async fn clear_old_events(db: &Database, retention_days: i32) -> Result<i64> {
    let query = "DELETE FROM events WHERE time < NOW() - INTERVAL '1 day' * $1";

    let deleted = db.execute(query, &[&retention_days]).await.map_err(|e| {
        error!("Failed to clear old events: {}", e);
        AuthencError::database("Failed to clear old events")
    })?;

    Ok(deleted as i64)
}

/// Clear old admin events based on retention policy
pub async fn clear_old_admin_events(db: &Database, retention_days: i32) -> Result<i64> {
    let query = "DELETE FROM admin_events WHERE time < NOW() - INTERVAL '1 day' * $1";

    let deleted = db.execute(query, &[&retention_days]).await.map_err(|e| {
        error!("Failed to clear old admin events: {}", e);
        AuthencError::database("Failed to clear old admin events")
    })?;

    Ok(deleted as i64)
}
