/// Database operations for audit logging
use crate::Database;
use authenc_types::Result;
use authenc_types::domain::audit::AuditEvent;
use uuid::Uuid;

pub async fn create_audit_log(db: &Database, event: &AuditEvent) -> Result<()> {
    let event_id = Uuid::new_v4();

    let query = r#"
        INSERT INTO audit_logs (
            id, timestamp, event_type, user_id, session_id,
            client_id, resource_type, resource_id, action,
            status, details, ip_address, user_agent,
            location_data, error_message, request_id, correlation_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
    "#;

    db.execute(
        query,
        &[
            &event_id,
            &event.timestamp,
            &event.event_type,
            &event.user_id,
            &event.session_id,
            &event.client_id,
            &event.resource_type,
            &event.resource_id,
            &event.action,
            &event.status,
            &event
                .details
                .as_ref()
                .map(|v| serde_json::to_string(v).unwrap_or_default()),
            &event.ip_address,
            &event.user_agent,
            &event.location_data,
            &event.error_message,
            &event.request_id,
            &event.correlation_id,
        ],
    )
    .await?;

    Ok(())
}

pub async fn get_audit_logs(
    db: &Database,
    user_id: Option<Uuid>,
    event_type: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<AuditEvent>> {
    let query = r#"
        SELECT
            id, timestamp, event_type, user_id, session_id,
            client_id, resource_type, resource_id, action,
            status, details, ip_address, user_agent,
            location_data, error_message, request_id, correlation_id
        FROM audit_logs
        WHERE ($1::uuid IS NULL OR user_id = $1)
        AND ($2::text IS NULL OR event_type = $2)
        ORDER BY timestamp DESC
        LIMIT $3 OFFSET $4
    "#;

    let rows = db
        .query(query, &[&user_id, &event_type, &limit, &offset])
        .await?;
    // Convert rows to Vec<AuditEvent>
    rows.into_iter()
        .map(|row: tokio_postgres::Row| row.try_into())
        .collect::<Result<Vec<AuditEvent>>>()
}

pub async fn get_audit_log_count(
    db: &Database,
    user_id: Option<Uuid>,
    event_type: Option<&str>,
) -> Result<i64> {
    let query = r#"
        SELECT COUNT(*) FROM audit_logs
        WHERE ($1::uuid IS NULL OR user_id = $1)
        AND ($2::text IS NULL OR event_type = $2)
    "#;

    let client = db.get_connection().await?;
    let count: i64 = client
        .query_one(query, &[&user_id, &event_type])
        .await?
        .try_get(0)?;
    Ok(count)
}

pub async fn cleanup_old_logs(db: &Database) -> Result<u64> {
    let query = "DELETE FROM audit_logs WHERE timestamp < NOW() - INTERVAL '90 days'";
    db.execute(query, &[]).await
}
