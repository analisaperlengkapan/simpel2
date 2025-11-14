/// Admin Console Advanced Features operations (for admin dashboard, monitoring, audit logging)
use crate::{database::Database, error::Result};
use chrono::{DateTime, Duration, Utc};
use serde_json::Value as JsonValue;
use uuid::Uuid;

#[allow(clippy::too_many_arguments)]
pub async fn log_admin_operation(
    db: &Database,
    realm_id: Uuid,
    admin_user_id: Option<Uuid>,
    admin_username: &str,
    admin_ip_address: Option<&str>,
    operation_type: &str,
    resource_type: &str,
    resource_id: Option<&str>,
    resource_name: Option<&str>,
    action: &str,
    status: &str,
    error_message: Option<&str>,
    request_method: Option<&str>,
    request_path: Option<&str>,
    request_body: Option<&JsonValue>,
    response_status: Option<i32>,
    response_body: Option<&JsonValue>,
    duration_ms: Option<i32>,
    user_agent: Option<&str>,
    session_id: Option<Uuid>,
) -> Result<Uuid> {
    let query = r#"
        INSERT INTO admin_audit_log (
            realm_id, admin_user_id, admin_username, admin_ip_address,
            operation_type, resource_type, resource_id, resource_name,
            action, status, error_message, request_method, request_path,
            request_body, response_status, response_body, duration_ms,
            user_agent, session_id
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
        RETURNING id
    "#;

    let ip_parsed = admin_ip_address.and_then(|ip| ip.parse::<std::net::IpAddr>().ok());

    let rows = db
        .query_raw(
            query,
            &[
                &realm_id,
                &admin_user_id,
                &admin_username,
                &ip_parsed,
                &operation_type,
                &resource_type,
                &resource_id,
                &resource_name,
                &action,
                &status,
                &error_message,
                &request_method,
                &request_path,
                &request_body,
                &response_status,
                &response_body,
                &duration_ms,
                &user_agent,
                &session_id,
            ],
        )
        .await?;

    Ok(rows[0].get(0))
}

pub async fn query_admin_audit_log(
    db: &Database,
    realm_id: Uuid,
    admin_user_id: Option<Uuid>,
    operation_type: Option<String>,
    resource_type: Option<String>,
    status: Option<String>,
    from_date: Option<DateTime<Utc>>,
    to_date: Option<DateTime<Utc>>,
    offset: i64,
    limit: i64,
) -> Result<Vec<JsonValue>> {
    let mut where_clauses = vec![String::from("realm_id = $1")];
    let mut param_index = 2;

    if admin_user_id.is_some() {
        where_clauses.push(format!("admin_user_id = ${}", param_index));
        param_index += 1;
    }

    if operation_type.is_some() {
        where_clauses.push(format!("operation_type = ${}", param_index));
        param_index += 1;
    }

    if resource_type.is_some() {
        where_clauses.push(format!("resource_type = ${}", param_index));
        param_index += 1;
    }

    if status.is_some() {
        where_clauses.push(format!("status = ${}", param_index));
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

    let where_clause = where_clauses.join(" AND ");

    let query = format!(
        r#"
        SELECT
            id, admin_user_id, admin_username, admin_ip_address, operation_type,
            resource_type, resource_id, resource_name, action, status,
            error_message, duration_ms, created_at
        FROM admin_audit_log
        WHERE {}
        ORDER BY created_at DESC
        LIMIT ${} OFFSET ${}
    "#,
        where_clause,
        param_index,
        param_index + 1
    );

    let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&realm_id];

    if let Some(ref user_id) = admin_user_id {
        params.push(user_id);
    }
    if let Some(ref op_type) = operation_type {
        params.push(op_type);
    }
    if let Some(ref res_type) = resource_type {
        params.push(res_type);
    }
    if let Some(ref st) = status {
        params.push(st);
    }
    if let Some(ref from) = from_date {
        params.push(from);
    }
    if let Some(ref to) = to_date {
        params.push(to);
    }

    params.push(&limit);
    params.push(&offset);

    let rows = db.query_raw(&query, &params).await?;

    Ok(rows
        .into_iter()
        .map(|row: tokio_postgres::Row| {
            serde_json::json!({
                "id": row.get::<_, Uuid>(0),
                "admin_user_id": row.get::<_, Option<Uuid>>(1),
                "admin_username": row.get::<_, String>(2),
                "admin_ip_address": row.get::<_, Option<std::net::IpAddr>>(3),
                "operation_type": row.get::<_, String>(4),
                "resource_type": row.get::<_, String>(5),
                "resource_id": row.get::<_, Option<String>>(6),
                "resource_name": row.get::<_, Option<String>>(7),
                "action": row.get::<_, String>(8),
                "status": row.get::<_, String>(9),
                "error_message": row.get::<_, Option<String>>(10),
                "duration_ms": row.get::<_, Option<i32>>(11),
                "created_at": row.get::<_, DateTime<Utc>>(12),
            })
        })
        .collect())
}

pub async fn record_dashboard_metric(
    db: &Database,
    realm_id: Uuid,
    metric_type: &str,
    metric_name: &str,
    metric_value: f64,
    metric_unit: Option<&str>,
    aggregation_period: &str,
    period_start: DateTime<Utc>,
    period_end: DateTime<Utc>,
    metadata: Option<&JsonValue>,
) -> Result<Uuid> {
    let query = r#"
        INSERT INTO admin_dashboard_metrics (
            realm_id, metric_type, metric_name, metric_value, metric_unit,
            aggregation_period, period_start, period_end, metadata
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        ON CONFLICT (realm_id, metric_type, metric_name, period_start)
        DO UPDATE SET
            metric_value = EXCLUDED.metric_value,
            metric_unit = EXCLUDED.metric_unit,
            period_end = EXCLUDED.period_end,
            metadata = EXCLUDED.metadata
        RETURNING id
    "#;

    let rows = db
        .query_raw(
            query,
            &[
                &realm_id,
                &metric_type,
                &metric_name,
                &metric_value,
                &metric_unit,
                &aggregation_period,
                &period_start,
                &period_end,
                &metadata,
            ],
        )
        .await?;

    Ok(rows[0].get(0))
}

pub async fn get_dashboard_metrics(
    db: &Database,
    realm_id: Uuid,
    metric_type: Option<&str>,
    aggregation_period: &str,
    from_date: DateTime<Utc>,
    to_date: DateTime<Utc>,
) -> Result<Vec<JsonValue>> {
    let query = if metric_type.is_some() {
        r#"
            SELECT
                id, metric_type, metric_name, metric_value, metric_unit,
                aggregation_period, period_start, period_end, metadata, created_at
            FROM admin_dashboard_metrics
            WHERE realm_id = $1 AND metric_type = $2 AND aggregation_period = $3
                AND period_start >= $4 AND period_end <= $5
            ORDER BY period_start
        "#
    } else {
        r#"
            SELECT
                id, metric_type, metric_name, metric_value, metric_unit,
                aggregation_period, period_start, period_end, metadata, created_at
            FROM admin_dashboard_metrics
            WHERE realm_id = $1 AND aggregation_period = $2
                AND period_start >= $3 AND period_end <= $4
            ORDER BY metric_type, period_start
        "#
    };

    let rows = if let Some(m_type) = metric_type {
        db.query_raw(
            query,
            &[
                &realm_id,
                &m_type,
                &aggregation_period,
                &from_date,
                &to_date,
            ],
        )
        .await?
    } else {
        db.query_raw(
            query,
            &[&realm_id, &aggregation_period, &from_date, &to_date],
        )
        .await?
    };

    Ok(rows
        .into_iter()
        .map(|row: tokio_postgres::Row| {
            serde_json::json!({
                "id": row.get::<_, Uuid>(0),
                "metric_type": row.get::<_, String>(1),
                "metric_name": row.get::<_, String>(2),
                "metric_value": row.get::<_, f64>(3),
                "metric_unit": row.get::<_, Option<String>>(4),
                "aggregation_period": row.get::<_, String>(5),
                "period_start": row.get::<_, DateTime<Utc>>(6),
                "period_end": row.get::<_, DateTime<Utc>>(7),
                "metadata": row.get::<_, Option<JsonValue>>(8),
                "created_at": row.get::<_, DateTime<Utc>>(9),
            })
        })
        .collect())
}

pub async fn create_admin_session(
    db: &Database,
    realm_id: Uuid,
    admin_user_id: Uuid,
    username: &str,
    session_token: &str,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
    login_method: &str,
    mfa_verified: bool,
    expires_in_seconds: i64,
) -> Result<Uuid> {
    let expires_at = Utc::now() + Duration::seconds(expires_in_seconds);

    let query = r#"
        INSERT INTO admin_console_sessions (
            realm_id, admin_user_id, username, session_token,
            ip_address, user_agent, login_method, mfa_verified, expires_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id
    "#;

    let ip_parsed = ip_address.and_then(|ip| ip.parse::<std::net::IpAddr>().ok());

    let rows = db
        .query_raw(
            query,
            &[
                &realm_id,
                &admin_user_id,
                &username,
                &session_token,
                &ip_parsed,
                &user_agent,
                &login_method,
                &mfa_verified,
                &expires_at,
            ],
        )
        .await?;

    Ok(rows[0].get(0))
}

pub async fn validate_admin_session(
    db: &Database,
    session_token: &str,
) -> Result<Option<JsonValue>> {
    let query = r#"
        UPDATE admin_console_sessions
        SET last_activity_at = NOW()
        WHERE session_token = $1 AND is_active = TRUE AND expires_at > NOW()
        RETURNING id, realm_id, admin_user_id, username, mfa_verified, expires_at
    "#;

    match db.query_opt(query, &[&session_token]).await? {
        Some(row) => Ok(Some(serde_json::json!({
            "id": row.get::<_, Uuid>(0),
            "realm_id": row.get::<_, Uuid>(1),
            "admin_user_id": row.get::<_, Uuid>(2),
            "username": row.get::<_, String>(3),
            "mfa_verified": row.get::<_, bool>(4),
            "expires_at": row.get::<_, DateTime<Utc>>(5),
        }))),
        None => Ok(None),
    }
}

pub async fn terminate_admin_session(
    db: &Database,
    session_token: &str,
    logout_reason: &str,
) -> Result<bool> {
    let query = r#"
        UPDATE admin_console_sessions
        SET is_active = FALSE, terminated_at = NOW(), logout_reason = $1
        WHERE session_token = $2 AND is_active = TRUE
    "#;

    let rows_affected = db.execute(query, &[&logout_reason, &session_token]).await?;
    Ok(rows_affected > 0)
}

pub async fn create_admin_notification(
    db: &Database,
    realm_id: Uuid,
    notification_type: &str,
    title: &str,
    message: &str,
    target_admin_user_id: Option<Uuid>,
    target_role: Option<&str>,
    action_url: Option<&str>,
    action_label: Option<&str>,
    priority: i32,
    expires_in_seconds: Option<i64>,
    metadata: Option<&JsonValue>,
) -> Result<Uuid> {
    let expires_at = expires_in_seconds.map(|seconds| Utc::now() + Duration::seconds(seconds));

    let query = r#"
        INSERT INTO admin_notifications (
            realm_id, notification_type, title, message, target_admin_user_id,
            target_role, action_url, action_label, priority, expires_at, metadata
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING id
    "#;

    let rows = db
        .query_raw(
            query,
            &[
                &realm_id,
                &notification_type,
                &title,
                &message,
                &target_admin_user_id,
                &target_role,
                &action_url,
                &action_label,
                &priority,
                &expires_at,
                &metadata,
            ],
        )
        .await?;

    Ok(rows[0].get(0))
}

pub async fn get_admin_notifications(
    db: &Database,
    realm_id: Uuid,
    admin_user_id: Option<Uuid>,
    unread_only: bool,
) -> Result<Vec<JsonValue>> {
    let query = if unread_only {
        r#"
            SELECT
                id, notification_type, title, message, action_url, action_label,
                priority, is_read, created_at
            FROM admin_notifications
            WHERE realm_id = $1
                AND (target_admin_user_id = $2 OR target_admin_user_id IS NULL)
                AND is_read = FALSE
                AND (expires_at IS NULL OR expires_at > NOW())
            ORDER BY priority ASC, created_at DESC
        "#
    } else {
        r#"
            SELECT
                id, notification_type, title, message, action_url, action_label,
                priority, is_read, created_at
            FROM admin_notifications
            WHERE realm_id = $1
                AND (target_admin_user_id = $2 OR target_admin_user_id IS NULL)
                AND (expires_at IS NULL OR expires_at > NOW())
            ORDER BY priority ASC, created_at DESC
        "#
    };

    let rows = db.query_raw(query, &[&realm_id, &admin_user_id]).await?;

    Ok(rows
        .into_iter()
        .map(|row: tokio_postgres::Row| {
            serde_json::json!({
                "id": row.get::<_, Uuid>(0),
                "notification_type": row.get::<_, String>(1),
                "title": row.get::<_, String>(2),
                "message": row.get::<_, String>(3),
                "action_url": row.get::<_, Option<String>>(4),
                "action_label": row.get::<_, Option<String>>(5),
                "priority": row.get::<_, i32>(6),
                "is_read": row.get::<_, bool>(7),
                "created_at": row.get::<_, DateTime<Utc>>(8),
            })
        })
        .collect())
}

pub async fn mark_notification_read(
    db: &Database,
    notification_id: Uuid,
    admin_user_id: Uuid,
) -> Result<bool> {
    let query = r#"
        UPDATE admin_notifications
        SET is_read = TRUE, read_at = NOW(), read_by_user_id = $1
        WHERE id = $2
    "#;

    let rows_affected = db
        .execute(query, &[&admin_user_id, &notification_id])
        .await?;
    Ok(rows_affected > 0)
}

pub async fn get_admin_preferences(
    db: &Database,
    admin_user_id: Uuid,
    realm_id: Uuid,
) -> Result<JsonValue> {
    let query = r#"
        SELECT
            id, theme, language, timezone, items_per_page, compact_mode,
            sidebar_collapsed, email_notifications, desktop_notifications,
            notification_frequency, dashboard_layout, favorite_pages,
            developer_mode, show_advanced_options, preferences
        FROM admin_console_preferences
        WHERE admin_user_id = $1 AND realm_id = $2
    "#;

    match db.query_opt(query, &[&admin_user_id, &realm_id]).await? {
        Some(row) => Ok(serde_json::json!({
            "id": row.get::<_, Uuid>(0),
            "theme": row.get::<_, String>(1),
            "language": row.get::<_, String>(2),
            "timezone": row.get::<_, String>(3),
            "items_per_page": row.get::<_, i32>(4),
            "compact_mode": row.get::<_, bool>(5),
            "sidebar_collapsed": row.get::<_, bool>(6),
            "email_notifications": row.get::<_, bool>(7),
            "desktop_notifications": row.get::<_, bool>(8),
            "notification_frequency": row.get::<_, String>(9),
            "dashboard_layout": row.get::<_, Option<JsonValue>>(10),
            "favorite_pages": row.get::<_, Option<Vec<String>>>(11),
            "developer_mode": row.get::<_, bool>(12),
            "show_advanced_options": row.get::<_, bool>(13),
            "preferences": row.get::<_, Option<JsonValue>>(14),
        })),
        None => {
            // Create default preferences
            let insert_query = r#"
                INSERT INTO admin_console_preferences (admin_user_id, realm_id)
                VALUES ($1, $2)
                RETURNING id
            "#;
            let rows = db
                .query_raw(insert_query, &[&admin_user_id, &realm_id])
                .await?;
            let id: Uuid = rows[0].get(0);

            Ok(serde_json::json!({
                "id": id,
                "theme": "light",
                "language": "en",
                "timezone": "UTC",
                "items_per_page": 25,
                "compact_mode": false,
                "sidebar_collapsed": false,
                "email_notifications": true,
                "desktop_notifications": true,
                "notification_frequency": "realtime",
                "dashboard_layout": null,
                "favorite_pages": null,
                "developer_mode": false,
                "show_advanced_options": false,
                "preferences": null,
            }))
        }
    }
}

pub async fn update_admin_preferences(
    db: &Database,
    admin_user_id: Uuid,
    realm_id: Uuid,
    preferences: &JsonValue,
) -> Result<bool> {
    let query = r#"
        UPDATE admin_console_preferences
        SET
            theme = COALESCE($3, theme),
            language = COALESCE($4, language),
            timezone = COALESCE($5, timezone),
            items_per_page = COALESCE($6, items_per_page),
            compact_mode = COALESCE($7, compact_mode),
            sidebar_collapsed = COALESCE($8, sidebar_collapsed),
            email_notifications = COALESCE($9, email_notifications),
            desktop_notifications = COALESCE($10, desktop_notifications),
            notification_frequency = COALESCE($11, notification_frequency),
            dashboard_layout = COALESCE($12, dashboard_layout),
            favorite_pages = COALESCE($13, favorite_pages),
            developer_mode = COALESCE($14, developer_mode),
            show_advanced_options = COALESCE($15, show_advanced_options),
            preferences = COALESCE($16, preferences),
            updated_at = NOW()
        WHERE admin_user_id = $1 AND realm_id = $2
    "#;

    let rows_affected = db
        .execute(
            query,
            &[
                &admin_user_id,
                &realm_id,
                &preferences.get("theme").and_then(|v| v.as_str()),
                &preferences.get("language").and_then(|v| v.as_str()),
                &preferences.get("timezone").and_then(|v| v.as_str()),
                &preferences
                    .get("items_per_page")
                    .and_then(|v| v.as_i64())
                    .map(|v| v as i32),
                &preferences.get("compact_mode").and_then(|v| v.as_bool()),
                &preferences
                    .get("sidebar_collapsed")
                    .and_then(|v| v.as_bool()),
                &preferences
                    .get("email_notifications")
                    .and_then(|v| v.as_bool()),
                &preferences
                    .get("desktop_notifications")
                    .and_then(|v| v.as_bool()),
                &preferences
                    .get("notification_frequency")
                    .and_then(|v| v.as_str()),
                &preferences.get("dashboard_layout"),
                &preferences.get("favorite_pages").and_then(|v| {
                    v.as_array().map(|arr| {
                        arr.iter()
                            .filter_map(|s| s.as_str().map(|s| s.to_string()))
                            .collect::<Vec<String>>()
                    })
                }),
                &preferences.get("developer_mode").and_then(|v| v.as_bool()),
                &preferences
                    .get("show_advanced_options")
                    .and_then(|v| v.as_bool()),
                &preferences.get("preferences"),
            ],
        )
        .await?;

    Ok(rows_affected > 0)
}
