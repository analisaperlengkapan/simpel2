use crate::services::audit_log_sink::AuditLogSink;
use anyhow::Result;
use authenc_types::domain::audit_log::AuditLog;
use std::sync::Arc;

/// The three values `audit_logs_status_check` accepts. Anything else becomes
/// `warning`: losing the precise outcome label beats losing the whole event.
fn normalize_status(raw: &str) -> &'static str {
    match raw {
        "success" => "success",
        "failure" => "failure",
        "warning" => "warning",
        other => {
            tracing::warn!("audit log status {other:?} is out of domain; recorded as 'warning'");
            "warning"
        }
    }
}

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
    /// The same reasoning covers `status`, which is constrained to
    /// success/failure/warning: an out-of-domain value is recorded as
    /// `warning` (and logged) rather than dropping the event entirely.
    pub async fn add_log(&self, log: &AuditLog) -> Result<()> {
        let user_id: Option<uuid::Uuid> = log.user_id.as_ref().and_then(|id| id.parse().ok());
        let client_id: Option<uuid::Uuid> = log.client_id.as_ref().and_then(|id| id.parse().ok());
        // details is jsonb — pass a JSON value, wrapping non-JSON detail
        // strings so a malformed detail can never fail the audit write.
        let details: Option<serde_json::Value> = log.detail.as_ref().map(|d| {
            serde_json::from_str(d).unwrap_or_else(|_| serde_json::json!({ "detail": d }))
        });
        let status = normalize_status(&log.status);

        // `ip_address` is an `inet` column, and PostgreSQL does NOT quietly turn
        // a non-address into NULL: `'x'::inet`, `'1.2.3.4:80'::inet`,
        // `'[::1]:443'::inet` and a comma-joined chain each raise
        // `invalid input syntax for type inet`, which fails the WHOLE insert
        // (verified against PostgreSQL 16). The previous version cast in SQL
        // on the strength of a comment claiming the opposite — so a caller
        // sending `X-Forwarded-For: x` could make its own audit row vanish
        // (the error was only logged). Parse in Rust; a value that is not an
        // address is stored as NULL, and the original text stays in `details`.
        let ip_address: Option<std::net::IpAddr> = log
            .ip_address
            .as_deref()
            .and_then(lib_backend::client_ip::parse_ip_token);

        self.db
            .execute(
                r#"INSERT INTO audit_logs ("timestamp", event_type, user_id, client_id, action, status, details, ip_address)
                   VALUES ($1, $2,
                           (SELECT id FROM users WHERE id = $3),
                           (SELECT id FROM oauth2_clients WHERE id = $4),
                           $5, $6, $7, $8)"#,
                &[
                    &log.timestamp,
                    &log.event,
                    &user_id,
                    &client_id,
                    &log.event,
                    &status,
                    &details,
                    &ip_address,
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

#[cfg(test)]
mod tests {
    use super::normalize_status;

    #[test]
    fn keeps_the_three_values_the_check_constraint_allows() {
        assert_eq!(normalize_status("success"), "success");
        assert_eq!(normalize_status("failure"), "failure");
        assert_eq!(normalize_status("warning"), "warning");
    }

    #[test]
    fn maps_out_of_domain_values_to_warning() {
        // "unknown" was the real value that made every insert violate
        // audit_logs_status_check, so the audit trail stayed empty.
        assert_eq!(normalize_status("unknown"), "warning");
        assert_eq!(normalize_status(""), "warning");
        assert_eq!(normalize_status("SUCCESS"), "warning");
    }
}
