use super::error::AppError;
use super::models::AuditLog;
use chrono::Utc;
use serde_json::Value;
use uuid::Uuid;

pub async fn insert_audit_log(
    pool: &deadpool_postgres::Pool,
    user_id: Option<Uuid>,
    document_id: Option<Uuid>,
    action: &str,
    details: &Value,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
) -> Result<(), AppError> {
    let client = pool.get().await?;
    let id = Uuid::new_v4();
    let now = Utc::now();
    client
        .execute(
            "INSERT INTO dokumen.audit_logs (id, document_id, user_id, action, details, ip_address, user_agent, timestamp)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            &[&id, &document_id, &user_id, &action, &details, &ip_address, &user_agent, &now],
        )
        .await?;
    Ok(())
}

pub async fn query_audit_logs(
    pool: &deadpool_postgres::Pool,
    document_id: Option<Uuid>,
    user_id: Option<Uuid>,
    limit: i64,
) -> Result<Vec<AuditLog>, AppError> {
    let client = pool.get().await?;
    let rows = if let Some(doc_id) = document_id {
        client
            .query(
                "SELECT * FROM dokumen.audit_logs WHERE document_id = $1 ORDER BY timestamp DESC LIMIT $2",
                &[&doc_id, &limit],
            )
            .await?
    } else if let Some(uid) = user_id {
        client
            .query(
                "SELECT * FROM dokumen.audit_logs WHERE user_id = $1 ORDER BY timestamp DESC LIMIT $2",
                &[&uid, &limit],
            )
            .await?
    } else {
        client
            .query(
                "SELECT * FROM dokumen.audit_logs ORDER BY timestamp DESC LIMIT $1",
                &[&limit],
            )
            .await?
    };
    Ok(rows.iter().map(AuditLog::from).collect())
}
