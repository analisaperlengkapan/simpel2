//! Audit logger with database persistence

use lib_core::audit::{AuditEvent, AuditLogEntry, AuditLogFilter, AuditSeverity};
use lib_core::context::RequestContext;
use lib_core::correlation::CorrelationId;
use lib_core::error::{CommonError, Result};

use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use tokio_postgres::Row;
use uuid::Uuid;

/// Audit logger with database persistence
pub struct AuditLogger {
    db_pool: Pool,
    service_name: String,
}

impl AuditLogger {
    pub fn new(db_pool: Pool, service_name: String) -> Self {
        Self {
            db_pool,
            service_name,
        }
    }

    /// Log an audit event to the database
    pub async fn log(&self, entry: AuditLogEntry) -> Result<Uuid> {
        let client: deadpool_postgres::Object = self.db_pool.get().await.map_err(|e| {
            CommonError::Database(format!("Failed to get database connection: {}", e))
        })?;

        let query = r#"
            INSERT INTO audit_log (
                id, timestamp, service, action, event_data, severity,
                actor_id, request_id, ip_address, user_agent, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id
        "#;

        let event_data = serde_json::to_value(&entry.event)
            .map_err(|e| CommonError::Serialization(format!("Failed to serialize event: {}", e)))?;

        let actor_id = entry.event.actor_id();
        let request_id = entry
            .context
            .as_ref()
            .map(|c| c.request_id.as_str().to_string());
        let ip_address = entry.context.as_ref().and_then(|c| c.ip_address.clone());
        let user_agent = entry.context.as_ref().and_then(|c| c.user_agent.clone());

        let row: Row = client
            .query_one(
                query,
                &[
                    &entry.id,
                    &entry.timestamp,
                    &entry.service,
                    &entry.event.action(),
                    &event_data,
                    &entry.severity.as_str(),
                    &actor_id,
                    &request_id,
                    &ip_address,
                    &user_agent,
                    &entry.metadata,
                ],
            )
            .await
            .map_err(|e| CommonError::Database(format!("Failed to insert audit log: {}", e)))?;

        let id: Uuid = row.get(0);
        Ok(id)
    }

    /// Log an event with automatic service name
    pub async fn log_event(
        &self,
        event: AuditEvent,
        severity: AuditSeverity,
        context: Option<RequestContext>,
    ) -> Result<Uuid> {
        let mut entry = AuditLogEntry::new(self.service_name.clone(), event, severity);
        if let Some(ctx) = context {
            entry = entry.with_context(ctx);
        }
        self.log(entry).await
    }

    /// Query audit logs with filters (simplified version)
    pub async fn query(&self, filter: AuditLogFilter) -> Result<Vec<AuditLogEntry>> {
        let client: deadpool_postgres::Object = self.db_pool.get().await.map_err(|e| {
            CommonError::Database(format!("Failed to get database connection: {}", e))
        })?;

        let query = r#"
            SELECT id, timestamp, service, action, event_data, severity,
                   actor_id, request_id, ip_address, user_agent, metadata
            FROM audit_log
            ORDER BY timestamp DESC
            LIMIT $1
        "#;

        let limit = filter.limit.unwrap_or(100);

        let rows: Vec<Row> = client
            .query(query, &[&limit])
            .await
            .map_err(|e| CommonError::Database(format!("Failed to query audit logs: {}", e)))?;

        let entries = rows
            .into_iter()
            .map(|row| self.row_to_entry(row))
            .collect::<Result<Vec<_>>>()?;

        Ok(entries)
    }

    /// Get audit log entry by ID
    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<AuditLogEntry>> {
        let client: deadpool_postgres::Object = self.db_pool.get().await.map_err(|e| {
            CommonError::Database(format!("Failed to get database connection: {}", e))
        })?;

        let query = r#"
            SELECT id, timestamp, service, action, event_data, severity,
                   actor_id, request_id, ip_address, user_agent, metadata
            FROM audit_log
            WHERE id = $1
        "#;

        let row: Option<Row> = client
            .query_opt(query, &[&id])
            .await
            .map_err(|e| CommonError::Database(format!("Failed to get audit log: {}", e)))?;

        match row {
            Some(row) => Ok(Some(self.row_to_entry(row)?)),
            None => Ok(None),
        }
    }

    /// Count audit logs matching filter (simplified version)
    pub async fn count(&self, _filter: AuditLogFilter) -> Result<i64> {
        let client: deadpool_postgres::Object = self.db_pool.get().await.map_err(|e| {
            CommonError::Database(format!("Failed to get database connection: {}", e))
        })?;

        let query = "SELECT COUNT(*) FROM audit_log";

        let row: Row = client
            .query_one(query, &[])
            .await
            .map_err(|e| CommonError::Database(format!("Failed to count audit logs: {}", e)))?;

        let count: i64 = row.get(0);
        Ok(count)
    }

    fn row_to_entry(&self, row: Row) -> Result<AuditLogEntry> {
        let id: Uuid = row.get(0);
        let timestamp: DateTime<Utc> = row.get(1);
        let service: String = row.get(2);
        let _action: String = row.get(3);
        let event_data: serde_json::Value = row.get(4);
        let severity_str: String = row.get(5);
        let _actor_id: Option<Uuid> = row.get(6);
        let request_id: Option<String> = row.get(7);
        let ip_address: Option<String> = row.get(8);
        let user_agent: Option<String> = row.get(9);
        let metadata: serde_json::Value = row.get(10);

        let event: AuditEvent = serde_json::from_value(event_data).map_err(|e| {
            CommonError::Deserialization(format!("Failed to deserialize event: {}", e))
        })?;

        let severity = match severity_str.as_str() {
            "info" => AuditSeverity::Info,
            "notice" => AuditSeverity::Notice,
            "warning" => AuditSeverity::Warning,
            "error" => AuditSeverity::Error,
            "critical" => AuditSeverity::Critical,
            "alert" => AuditSeverity::Alert,
            "emergency" => AuditSeverity::Emergency,
            _ => AuditSeverity::Info,
        };

        let context = if request_id.is_some() || ip_address.is_some() || user_agent.is_some() {
            Some(RequestContext {
                request_id: request_id
                    .map(CorrelationId::new_from_string)
                    .unwrap_or_default(),
                ip_address,
                user_agent,
                start_time: chrono::Utc::now(),
            })
        } else {
            None
        };

        Ok(AuditLogEntry {
            id,
            timestamp,
            service,
            event,
            severity,
            context,
            metadata,
        })
    }
}
