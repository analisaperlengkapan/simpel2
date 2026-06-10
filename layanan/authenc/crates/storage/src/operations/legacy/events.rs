/// Event System operations (for event-driven architecture and audit logging)
use crate::Database;
use authenc_types::Result;
use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use uuid::Uuid;

/// Parameters for registering an event listener.
pub struct NewEventListener<'a> {
    pub realm_id: Uuid,
    pub name: &'a str,
    pub listener_type: &'a str,
    pub enabled: bool,
    pub config: Option<&'a JsonValue>,
    pub event_types: Option<Vec<String>>,
    pub priority: i32,
    pub is_async: bool,
    pub retry_on_failure: bool,
    pub max_retries: i32,
}

pub async fn register_event_listener(db: &Database, params: NewEventListener<'_>) -> Result<Uuid> {
    let NewEventListener {
        realm_id,
        name,
        listener_type,
        enabled,
        config,
        event_types,
        priority,
        is_async,
        retry_on_failure,
        max_retries,
    } = params;
    let query = r#"
        INSERT INTO event_listeners (
            realm_id, name, listener_type, enabled, config,
            event_types, priority, is_async, retry_on_failure, max_retries
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id
    "#;

    let rows = db
        .query(
            query,
            &[
                &realm_id,
                &name,
                &listener_type,
                &enabled,
                &config,
                &event_types,
                &priority,
                &is_async,
                &retry_on_failure,
                &max_retries,
            ],
        )
        .await?;

    Ok(rows[0].get(0))
}

pub async fn get_enabled_listeners(db: &Database, realm_id: Uuid) -> Result<Vec<JsonValue>> {
    let query = r#"
        SELECT
            id, name, listener_type, config, event_types, priority,
            is_async, retry_on_failure, max_retries
        FROM event_listeners
        WHERE realm_id = $1 AND enabled = TRUE
        ORDER BY priority ASC
    "#;

    let rows = db.query(query, &[&realm_id]).await?;

    Ok(rows
        .into_iter()
        .map(|row: tokio_postgres::Row| {
            serde_json::json!({
                "id": row.get::<_, Uuid>(0),
                "name": row.get::<_, String>(1),
                "listener_type": row.get::<_, String>(2),
                "config": row.get::<_, Option<JsonValue>>(3),
                "event_types": row.get::<_, Option<Vec<String>>>(4),
                "priority": row.get::<_, i32>(5),
                "is_async": row.get::<_, bool>(6),
                "retry_on_failure": row.get::<_, bool>(7),
                "max_retries": row.get::<_, i32>(8),
            })
        })
        .collect())
}

#[allow(clippy::too_many_arguments)]
pub async fn log_event(
    db: &Database,
    realm_id: Uuid,
    event_type: &str,
    event_category: &str,
    resource_type: Option<&str>,
    resource_id: Option<&str>,
    resource_name: Option<&str>,
    user_id: Option<Uuid>,
    username: Option<&str>,
    event_data: Option<&JsonValue>,
    old_value: Option<&JsonValue>,
    new_value: Option<&JsonValue>,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
    session_id: Option<Uuid>,
    success: bool,
    error_message: Option<&str>,
    operation_id: Option<Uuid>,
    correlation_id: Option<Uuid>,
) -> Result<Uuid> {
    let query = r#"
        INSERT INTO event_log (
            realm_id, event_type, event_category, resource_type, resource_id,
            resource_name, user_id, username, event_data, old_value, new_value,
            ip_address, user_agent, session_id, success, error_message,
            operation_id, correlation_id
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
        RETURNING id
    "#;

    let ip_parsed = ip_address.and_then(|ip| ip.parse::<std::net::IpAddr>().ok());

    let rows = db
        .query(
            query,
            &[
                &realm_id,
                &event_type,
                &event_category,
                &resource_type,
                &resource_id,
                &resource_name,
                &user_id,
                &username,
                &event_data,
                &old_value,
                &new_value,
                &ip_parsed,
                &user_agent,
                &session_id,
                &success,
                &error_message,
                &operation_id,
                &correlation_id,
            ],
        )
        .await?;

    Ok(rows[0].get(0))
}

/// Filter parameters for querying the event log.
#[derive(Default)]
pub struct EventLogQuery {
    pub realm_id: Uuid,
    pub event_category: Option<String>,
    pub event_type: Option<String>,
    pub resource_type: Option<String>,
    pub user_id: Option<Uuid>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub success_only: Option<bool>,
    pub offset: i64,
    pub limit: i64,
}

pub async fn query_event_log(db: &Database, query_params: EventLogQuery) -> Result<Vec<JsonValue>> {
    let EventLogQuery {
        realm_id,
        event_category,
        event_type,
        resource_type,
        user_id,
        from_date,
        to_date,
        success_only,
        offset,
        limit,
    } = query_params;
    let mut where_clauses = vec![String::from("realm_id = $1")];
    let mut param_index = 2;

    if event_category.is_some() {
        where_clauses.push(format!("event_category = ${}", param_index));
        param_index += 1;
    }

    if event_type.is_some() {
        where_clauses.push(format!("event_type = ${}", param_index));
        param_index += 1;
    }

    if resource_type.is_some() {
        where_clauses.push(format!("resource_type = ${}", param_index));
        param_index += 1;
    }

    if user_id.is_some() {
        where_clauses.push(format!("user_id = ${}", param_index));
        param_index += 1;
    }

    if from_date.is_some() {
        where_clauses.push(format!("created_at >= ${}", param_index));
        param_index += 1;
    }

    if to_date.is_some() {
        where_clauses.push(format!("created_at <= ${}", param_index));
        param_index += 1;
    }

    if let Some(true) = success_only {
        where_clauses.push(String::from("success = TRUE"));
    }

    let where_clause = where_clauses.join(" AND ");

    let query = format!(
        r#"
        SELECT
            id, event_type, event_category, resource_type, resource_id,
            resource_name, user_id, username, success, error_message,
            correlation_id, created_at
        FROM event_log
        WHERE {}
        ORDER BY created_at DESC
        LIMIT ${} OFFSET ${}
    "#,
        where_clause,
        param_index,
        param_index + 1
    );

    let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&realm_id];

    if let Some(ref cat) = event_category {
        params.push(cat);
    }
    if let Some(ref et) = event_type {
        params.push(et);
    }
    if let Some(ref rt) = resource_type {
        params.push(rt);
    }
    if let Some(ref uid) = user_id {
        params.push(uid);
    }
    if let Some(ref from) = from_date {
        params.push(from);
    }
    if let Some(ref to) = to_date {
        params.push(to);
    }

    params.push(&limit);
    params.push(&offset);

    let rows = db.query(&query, &params).await?;

    Ok(rows
        .into_iter()
        .map(|row: tokio_postgres::Row| {
            serde_json::json!({
                "id": row.get::<_, Uuid>(0),
                "event_type": row.get::<_, String>(1),
                "event_category": row.get::<_, String>(2),
                "resource_type": row.get::<_, Option<String>>(3),
                "resource_id": row.get::<_, Option<String>>(4),
                "resource_name": row.get::<_, Option<String>>(5),
                "user_id": row.get::<_, Option<Uuid>>(6),
                "username": row.get::<_, Option<String>>(7),
                "success": row.get::<_, bool>(8),
                "error_message": row.get::<_, Option<String>>(9),
                "correlation_id": row.get::<_, Option<Uuid>>(10),
                "created_at": row.get::<_, DateTime<Utc>>(11),
            })
        })
        .collect())
}

pub async fn record_listener_execution(
    db: &Database,
    event_log_id: Uuid,
    listener_id: Uuid,
    success: bool,
    error_message: Option<&str>,
    duration_ms: i32,
    retry_count: i32,
    next_retry_at: Option<DateTime<Utc>>,
) -> Result<Uuid> {
    let query = r#"
        INSERT INTO event_listener_executions (
            event_log_id, listener_id, success, error_message,
            duration_ms, retry_count, next_retry_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id
    "#;

    let rows = db
        .query(
            query,
            &[
                &event_log_id,
                &listener_id,
                &success,
                &error_message,
                &duration_ms,
                &retry_count,
                &next_retry_at,
            ],
        )
        .await?;

    Ok(rows[0].get(0))
}

/// Parameters for registering an event webhook.
pub struct NewWebhook<'a> {
    pub listener_id: Uuid,
    pub realm_id: Uuid,
    pub url: &'a str,
    pub http_method: &'a str,
    pub auth_type: Option<&'a str>,
    pub auth_credentials: Option<&'a JsonValue>,
    pub custom_headers: Option<&'a JsonValue>,
    pub payload_template: Option<&'a str>,
    pub secret_key: Option<&'a str>,
    pub verify_ssl: bool,
    pub timeout_seconds: i32,
}

pub async fn register_webhook(db: &Database, params: NewWebhook<'_>) -> Result<Uuid> {
    let NewWebhook {
        listener_id,
        realm_id,
        url,
        http_method,
        auth_type,
        auth_credentials,
        custom_headers,
        payload_template,
        secret_key,
        verify_ssl,
        timeout_seconds,
    } = params;
    let query = r#"
        INSERT INTO event_webhooks (
            listener_id, realm_id, url, http_method, auth_type,
            auth_credentials, custom_headers, payload_template,
            secret_key, verify_ssl, timeout_seconds
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING id
    "#;

    let rows = db
        .query(
            query,
            &[
                &listener_id,
                &realm_id,
                &url,
                &http_method,
                &auth_type,
                &auth_credentials,
                &custom_headers,
                &payload_template,
                &secret_key,
                &verify_ssl,
                &timeout_seconds,
            ],
        )
        .await?;

    Ok(rows[0].get(0))
}

pub async fn get_event_statistics(
    db: &Database,
    realm_id: Uuid,
    from_date: DateTime<Utc>,
    to_date: DateTime<Utc>,
) -> Result<JsonValue> {
    let query = r#"
        SELECT
            COUNT(*) as total_events,
            COUNT(*) FILTER (WHERE success = TRUE) as successful_events,
            COUNT(*) FILTER (WHERE success = FALSE) as failed_events,
            COUNT(DISTINCT event_type) as unique_event_types,
            COUNT(DISTINCT user_id) as unique_users,
            COUNT(DISTINCT event_category) as unique_categories
        FROM event_log
        WHERE realm_id = $1 AND created_at >= $2 AND created_at <= $3
    "#;

    match db
        .query_opt(query, &[&realm_id, &from_date, &to_date])
        .await?
    {
        Some(row) => Ok(serde_json::json!({
            "total_events": row.get::<_, i64>(0),
            "successful_events": row.get::<_, i64>(1),
            "failed_events": row.get::<_, i64>(2),
            "unique_event_types": row.get::<_, i64>(3),
            "unique_users": row.get::<_, i64>(4),
            "unique_categories": row.get::<_, i64>(5),
        })),
        None => Ok(serde_json::json!({
            "total_events": 0,
            "successful_events": 0,
            "failed_events": 0,
            "unique_event_types": 0,
            "unique_users": 0,
            "unique_categories": 0,
        })),
    }
}

pub async fn get_failed_executions_for_retry(
    db: &Database,
    max_retry_count: i32,
) -> Result<Vec<JsonValue>> {
    let query = r#"
        SELECT
            ele.id, ele.event_log_id, ele.listener_id, ele.retry_count,
            el.realm_id, el.event_type, el.event_data
        FROM event_listener_executions ele
        JOIN event_log el ON ele.event_log_id = el.id
        JOIN event_listeners l ON ele.listener_id = l.id
        WHERE ele.success = FALSE
            AND l.retry_on_failure = TRUE
            AND ele.retry_count < $1
            AND (ele.next_retry_at IS NULL OR ele.next_retry_at <= NOW())
        ORDER BY ele.next_retry_at NULLS FIRST
        LIMIT 100
    "#;

    let rows = db.query(query, &[&max_retry_count]).await?;

    Ok(rows
        .into_iter()
        .map(|row: tokio_postgres::Row| {
            serde_json::json!({
                "execution_id": row.get::<_, Uuid>(0),
                "event_log_id": row.get::<_, Uuid>(1),
                "listener_id": row.get::<_, Uuid>(2),
                "retry_count": row.get::<_, i32>(3),
                "realm_id": row.get::<_, Uuid>(4),
                "event_type": row.get::<_, String>(5),
                "event_data": row.get::<_, Option<JsonValue>>(6),
            })
        })
        .collect())
}
