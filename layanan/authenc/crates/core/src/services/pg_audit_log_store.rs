impl Clone for PgAuditLogStore {
    fn clone(&self) -> Self {
        PgAuditLogStore {
            db: self.db.clone(),
        }
    }
}
use anyhow::Result;
use authenc_types::{domain::audit_log::AuditLog, ClientId, UserId};
use std::sync::Arc;
use crate::services::audit_log_sink::AuditLogSink;

/// PostgreSQL-based audit log store implementation
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
    /// # Arguments
    /// * `log` - The audit log entry to store
    pub async fn add_log(&self, log: &AuditLog) -> Result<()> {
        let ts: std::time::SystemTime = log.timestamp.into();
        self.db.execute(
            "INSERT INTO audit_logs (timestamp, event_type, user_id, client_id, status, details) VALUES ($1, $2, $3, $4, $5, $6)",
            &[
                &ts,
                &log.event,
                &log.user_id.as_ref().and_then(|id| std::str::FromStr::from_str(id).ok()).unwrap_or(uuid::Uuid::nil()), // Try mapping String to UUID, fallback appropriately
                &log.client_id.as_ref().and_then(|id| std::str::FromStr::from_str(id).ok()).unwrap_or(uuid::Uuid::nil()),// Map string IDs to UUID or null equivalent
                &log.status,
                &log.detail,
            ],
        ).await.map_err(|e| anyhow::anyhow!("DB execution error: {}", e))?;
        Ok(())
    }

    /// Get all audit log entries ordered by timestamp descending
    pub async fn all(&self) -> Result<Vec<AuditLog>> {
        let rows = self.db.query("SELECT timestamp, event_type, user_id, client_id, status, details FROM audit_logs ORDER BY timestamp DESC", &[]).await.map_err(|e| anyhow::anyhow!("DB query error: {}", e))?;
        Ok(rows
            .into_iter()
            .map(|row| {
                let ts: std::time::SystemTime = row.get(0);
                let timestamp: chrono::DateTime<chrono::Utc> = ts.into();
                // Map UUID to String representations based on domain type
                let user_uuid: Option<uuid::Uuid> = row.try_get(2).ok();
                let client_uuid: Option<uuid::Uuid> = row.try_get(3).ok();

                AuditLog {
                    timestamp,
                    event: row.get(1),
                    user_id: user_uuid.map(|id| id.to_string()),
                    client_id: client_uuid.map(|id| id.to_string()),
                    status: row.get(4),
                    detail: row.get(5),
                }
            })
            .collect())
    }
}

impl AuditLogSink for PgAuditLogStore {
    fn send(&self, log: &AuditLog) {
        let store = self.clone();
        let log = log.clone();
        tokio::spawn(async move {
            let _ = store.add_log(&log).await;
        });
    }
}
