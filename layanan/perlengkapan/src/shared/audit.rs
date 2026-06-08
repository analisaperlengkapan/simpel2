//! Concrete [`AuditSink`] implementation backed by
//! `perlengkapan.audit_log` (migration V028).
//!
//! Wired into [`crate::state::AppState`] as `Arc<dyn AuditSink>` so any
//! handler can record an `AuditEvent` without coupling to PostgreSQL or
//! knowing the schema. Failures are logged and swallowed inside the sink —
//! a transient DB hiccup must not abort the user's primary action just to
//! lose an audit row.

use async_trait::async_trait;
use deadpool_postgres::Pool;

use crate::contracts::AuditSink;
use lib_perlengkapan::audit::{AuditAction, AuditEvent};
use lib_perlengkapan::error::{ServiceError, ServiceResult};

/// PostgreSQL-backed `AuditSink`. Writes every event to
/// `perlengkapan.audit_log`.
#[derive(Clone)]
pub struct PgAuditSink {
    pool: Pool,
}

impl PgAuditSink {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    fn action_str(action: AuditAction) -> &'static str {
        action.as_str()
    }
}

#[async_trait]
impl AuditSink for PgAuditSink {
    async fn log(&self, event: AuditEvent) -> ServiceResult<()> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| ServiceError::DatabasePool(format!("audit pool: {e}")))?;

        let action_str = Self::action_str(event.action);
        let metadata = event.metadata.clone();

        client
            .execute(
                r#"
                INSERT INTO perlengkapan.audit_log
                    (id, occurred_at, actor_user_id, actor_username, actor_ip,
                     action, action_name, resource_type, resource_id, module,
                     success, message, metadata)
                VALUES
                    ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                "#,
                &[
                    &event.id,
                    &event.occurred_at,
                    &event.actor_user_id,
                    &event.actor_username,
                    &event.actor_ip,
                    &action_str,
                    &event.action_name,
                    &event.resource_type,
                    &event.resource_id,
                    &event.module,
                    &event.success,
                    &event.message,
                    &metadata,
                ],
            )
            .await
            .map_err(|e| ServiceError::database(format!("audit insert: {e}")))?;

        Ok(())
    }
}
