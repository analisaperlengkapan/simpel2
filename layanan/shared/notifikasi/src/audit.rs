use crate::error::AppError;
use sqlx::PgPool;
use uuid::Uuid;
use serde_json::Value;
use chrono::Utc;

pub async fn insert_audit_log(
    pool: &PgPool,
    notification_id: Option<Uuid>,
    user_id: Option<Uuid>,
    action: &str,
    details: &Value,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
) -> Result<(), AppError> {
    sqlx::query!(
        r#"INSERT INTO notifikasi.delivery_logs (id, notification_id, recipient, channel, status, message, timestamp)
        VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
        Uuid::new_v4(),
        notification_id,
        user_id.map(|u| u.to_string()),
        action,
        "audit",
        details.to_string(),
        Utc::now()
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn query_audit_logs(
    pool: &PgPool,
    notification_id: Option<Uuid>,
    user_id: Option<Uuid>,
    limit: i64
) -> Result<Vec<serde_json::Value>, AppError> {
    let rows = if let Some(nid) = notification_id {
        sqlx::query!(
            r#"SELECT * FROM notifikasi.delivery_logs WHERE notification_id = $1 ORDER BY timestamp DESC LIMIT $2"#,
            nid, limit
        ).fetch_all(pool).await?
    } else if let Some(uid) = user_id {
        sqlx::query!(
            r#"SELECT * FROM notifikasi.delivery_logs WHERE recipient = $1 ORDER BY timestamp DESC LIMIT $2"#,
            uid.to_string(), limit
        ).fetch_all(pool).await?
    } else {
        sqlx::query!(
            r#"SELECT * FROM notifikasi.delivery_logs ORDER BY timestamp DESC LIMIT $1"#,
            limit
        ).fetch_all(pool).await?
    };
    let logs = rows.into_iter().map(|row| {
        serde_json::json!({
            "id": row.id,
            "notification_id": row.notification_id,
            "recipient": row.recipient,
            "channel": row.channel,
            "status": row.status,
            "message": row.message,
            "timestamp": row.timestamp
        })
    }).collect();
    Ok(logs)
} 