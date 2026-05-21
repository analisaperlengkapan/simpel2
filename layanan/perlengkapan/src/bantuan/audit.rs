use super::error::AppError;
use super::models::AuditLog;
use chrono::Utc;
use deadpool_postgres::Pool;
use serde_json::Value;
use uuid::Uuid;

#[allow(dead_code)]
pub async fn insert_audit_log(
    pool: &Pool,
    user_id: Option<Uuid>,
    action: &str,
    resource: Option<&str>,
    resource_id: Option<Uuid>,
    details: &Value,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
) -> Result<(), AppError> {
    let client = pool.get().await?;
    client.execute(
        r#"INSERT INTO bantuan.audit_logs (id, user_id, action, resource, resource_id, details, ip_address, user_agent, timestamp)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
        &[&Uuid::new_v4(), &user_id, &action, &resource, &resource_id, &details, &ip_address, &user_agent, &Utc::now()]
    ).await?;
    Ok(())
}

pub async fn query_audit_logs(
    pool: &Pool,
    user_id: Option<Uuid>,
    action: Option<&str>,
    resource: Option<&str>,
    limit: i64,
) -> Result<Vec<AuditLog>, AppError> {
    let client = pool.get().await?;
    let rows = if let Some(uid) = user_id {
        client.query(
            r#"SELECT * FROM bantuan.audit_logs WHERE user_id = $1 AND ($2::text IS NULL OR action = $2) AND ($3::text IS NULL OR resource = $3) ORDER BY timestamp DESC LIMIT $4"#,
            &[&uid, &action, &resource, &limit]
        ).await?
    } else {
        client.query(
            r#"SELECT * FROM bantuan.audit_logs WHERE ($1::text IS NULL OR action = $1) AND ($2::text IS NULL OR resource = $2) ORDER BY timestamp DESC LIMIT $3"#,
            &[&action, &resource, &limit]
        ).await?
    };
    let logs = rows.into_iter().map(|row| AuditLog::from(&row)).collect();
    Ok(logs)
}
