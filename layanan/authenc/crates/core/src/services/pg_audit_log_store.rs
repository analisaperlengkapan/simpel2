use crate::services::audit_log_sink::AuditLogSink;
use anyhow::Result;
use authenc_types::domain::audit_log::AuditLog;
use std::sync::Arc;

/// PostgreSQL-based audit log store implementation
#[derive(Clone)]
pub struct PgAuditLogStore {
    /// Database connection pool
    pub db: Arc<authenc_storage::Database>,
}

impl PgAuditLogStore {
    /// Create new PostgreSQL audit log store
    ///
    /// # Arguments
    /// * `db` - Shared database connection pool
    pub fn new(db: Arc<authenc_storage::Database>) -> Self {
        Self { db }
    }

    /// Add audit log entry to database
    ///
    /// The `audit_logs` table requires a NOT NULL `action`; the domain
    /// `AuditLog` only carries an event name, so the event doubles as the
    /// action. IDs that don't parse as UUIDs are stored as NULL — never as
    /// the nil UUID, which would fabricate an entity that doesn't exist —
    /// and `audit_logs` has FKs to users/oauth2_clients, so IDs that parse
    /// but reference nothing are nulled via subselect: an audit write must
    /// never be rejected because its subject is unknown or already deleted.
    pub async fn add_log(&self, log: &AuditLog) -> Result<()> {
        let user_id: Option<uuid::Uuid> = log.user_id.as_ref().and_then(|id| id.parse().ok());
        let client_id: Option<uuid::Uuid> = log.client_id.as_ref().and_then(|id| id.parse().ok());
        // details is jsonb — pass a JSON value, wrapping non-JSON detail
        // strings so a malformed detail can never fail the audit write.
        let details: Option<serde_json::Value> = log.detail.as_ref().map(|d| {
            serde_json::from_str(d).unwrap_or_else(|_| serde_json::json!({ "detail": d }))
        });

        self.db
            .execute(
                r#"INSERT INTO audit_logs ("timestamp", event_type, user_id, client_id, action, status, details)
                   VALUES ($1, $2,
                           (SELECT id FROM users WHERE id = $3),
                           (SELECT id FROM oauth2_clients WHERE id = $4),
                           $5, $6, $7)"#,
                &[
                    &log.timestamp,
                    &log.event,
                    &user_id,
                    &client_id,
                    &log.event,
                    &log.status,
                    &details,
                ],
            )
            .await
            .map_err(|e| anyhow::anyhow!("DB execution error: {}", e))?;
        Ok(())
    }
}

impl AuditLogSink for PgAuditLogStore {
    fn send(&self, log: &AuditLog) {
        let store = self.clone();
        let log = log.clone();
        tokio::spawn(async move {
            if let Err(e) = store.add_log(&log).await {
                tracing::error!("Failed to persist audit log event: {e}");
            }
        });
    }
}
