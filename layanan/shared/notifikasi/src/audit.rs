use crate::error::AppError;
use chrono::Utc;
use deadpool_postgres::Pool;
use serde_json::Value;
use uuid::Uuid;

pub async fn insert_audit_log(
    pool: &Pool,
    notification_id: Option<Uuid>,
    user_id: Option<Uuid>,
    action: &str,
    details: &Value,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
) -> Result<(), AppError> {
    let client = pool.get().await?;
    client
        .execute(
            "INSERT INTO notifikasi.delivery_logs (id, notification_id, recipient, channel, status, message, timestamp)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
            &[
                &Uuid::new_v4(),
                &notification_id,
                &user_id.map(|u| u.to_string()),
                &action,
                &"audit",
                &details.to_string(),
                &Utc::now(),
            ],
        )
        .await?;
    Ok(())
}

pub async fn query_audit_logs(
    pool: &Pool,
    notification_id: Option<Uuid>,
    user_id: Option<Uuid>,
    limit: i64,
) -> Result<Vec<serde_json::Value>, AppError> {
    let client = pool.get().await?;
    let rows = if let Some(nid) = notification_id {
        client
            .query(
                "SELECT id, notification_id, recipient, channel, status, message, timestamp
                 FROM notifikasi.delivery_logs WHERE notification_id = $1 ORDER BY timestamp DESC LIMIT $2",
                &[&nid, &limit],
            )
            .await?
    } else if let Some(uid) = user_id {
        client
            .query(
                "SELECT id, notification_id, recipient, channel, status, message, timestamp
                 FROM notifikasi.delivery_logs WHERE recipient = $1 ORDER BY timestamp DESC LIMIT $2",
                &[&uid.to_string(), &limit],
            )
            .await?
    } else {
        client
            .query(
                "SELECT id, notification_id, recipient, channel, status, message, timestamp
                 FROM notifikasi.delivery_logs ORDER BY timestamp DESC LIMIT $1",
                &[&limit],
            )
            .await?
    };

    let logs = rows
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "id": row.get::<_, Uuid>(0),
                "notification_id": row.get::<_, Option<Uuid>>(1),
                "recipient": row.get::<_, Option<String>>(2),
                "channel": row.get::<_, String>(3),
                "status": row.get::<_, String>(4),
                "message": row.get::<_, Option<String>>(5),
                "timestamp": row.get::<_, chrono::DateTime<Utc>>(6)
            })
        })
        .collect();
    Ok(logs)
}
