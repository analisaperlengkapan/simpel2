use crate::error::AppError;
use crate::models::AuditLog;
use sqlx::PgPool;
use uuid::Uuid;
use serde_json::Value;
use chrono::Utc;

pub async fn insert_audit_log(
    pool: &PgPool,
    user_id: Option<Uuid>,
    document_id: Option<Uuid>,
    action: &str,
    details: &Value,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
) -> Result<(), AppError> {
    sqlx::query!(
        r#"INSERT INTO dokumen.audit_logs (id, document_id, user_id, action, details, ip_address, user_agent, timestamp)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
        Uuid::new_v4(),
        document_id,
        user_id,
        action,
        details,
        ip_address,
        user_agent,
        Utc::now()
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn query_audit_logs(
    pool: &PgPool,
    document_id: Option<Uuid>,
    user_id: Option<Uuid>,
    limit: i64
) -> Result<Vec<AuditLog>, AppError> {
    let logs = if let Some(doc_id) = document_id {
        sqlx::query_as!(AuditLog,
            r#"SELECT * FROM dokumen.audit_logs WHERE document_id = $1 ORDER BY timestamp DESC LIMIT $2"#,
            doc_id, limit
        )
        .fetch_all(pool)
        .await?
    } else if let Some(uid) = user_id {
        sqlx::query_as!(AuditLog,
            r#"SELECT * FROM dokumen.audit_logs WHERE user_id = $1 ORDER BY timestamp DESC LIMIT $2"#,
            uid, limit
        )
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as!(AuditLog,
            r#"SELECT * FROM dokumen.audit_logs ORDER BY timestamp DESC LIMIT $1"#,
            limit
        )
        .fetch_all(pool)
        .await?
    };
    Ok(logs)
} 