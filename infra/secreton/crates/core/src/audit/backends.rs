//! Audit log storage backends

use super::*;
use async_trait::async_trait;
use deadpool_postgres::Pool;
use std::collections::HashMap;
use std::path::Path;

/// PostgreSQL backend for audit logs
pub struct PostgreSqlBackend {
    pool: Pool,
}

impl PostgreSqlBackend {
    /// Create a new PostgreSQL backend
    pub async fn new(pool: Pool) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Create audit logs table if it doesn't exist
        let client = pool.get().await?;
        client
            .execute(
                "CREATE TABLE IF NOT EXISTS audit_logs (
                    id UUID PRIMARY KEY,
                    timestamp TIMESTAMPTZ NOT NULL,
                    action TEXT NOT NULL,
                    actor_id TEXT,
                    resource_type TEXT,
                    resource_id TEXT,
                    status TEXT NOT NULL,
                    ip TEXT,
                    user_agent TEXT,
                    metadata JSONB,
                    namespace TEXT
                )",
                &[],
            )
            .await?;

        // Ensure namespace column exists (for migration of existing tables)
        client
            .execute(
                "ALTER TABLE audit_logs ADD COLUMN IF NOT EXISTS namespace TEXT",
                &[],
            )
            .await?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl AuditBackend for PostgreSqlBackend {
    async fn log(&self, entry: AuditLog) -> Result<(), AuditError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| AuditError::LoggingError(e.to_string()))?;

        let metadata = serde_json::to_string(&entry.metadata)
            .map_err(|e| AuditError::LoggingError(e.to_string()))?;

        client
            .execute(
                "INSERT INTO audit_logs (
                id, timestamp, action, actor_id, resource_type,
                resource_id, status, ip, user_agent, metadata, namespace
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
                &[
                    &entry.id,
                    &entry.timestamp,
                    &entry.action,
                    &entry.actor.as_ref().map(|id| id.to_string()),
                    &entry.resource_type,
                    &entry.resource_id,
                    &match entry.status {
                        AuditStatus::Success => "success",
                        AuditStatus::Failure => "failure",
                        AuditStatus::Denied => "denied",
                    },
                    &entry.ip,
                    &entry.user_agent,
                    &metadata,
                    &entry.namespace,
                ],
            )
            .await
            .map_err(|e| AuditError::LoggingError(e.to_string()))?;

        Ok(())
    }

    async fn query(&self, query: AuditQuery) -> Result<Vec<AuditLog>, AuditError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| AuditError::LoggingError(e.to_string()))?;

        let mut query_str = "SELECT id, timestamp, action, actor_id, resource_type, resource_id, status, ip, user_agent, metadata, namespace FROM audit_logs WHERE 1=1".to_string();
        let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = Vec::new();
        let mut param_idx = 1;

        if let Some(action) = query.action {
            query_str.push_str(&format!(" AND action = ${}", param_idx));
            params.push(Box::new(action));
            param_idx += 1;
        }

        if let Some(actor) = query.actor {
            query_str.push_str(&format!(" AND actor_id = ${}", param_idx));
            params.push(Box::new(actor));
            param_idx += 1;
        }

        if let Some(resource_type) = query.resource_type {
            query_str.push_str(&format!(" AND resource_type = ${}", param_idx));
            params.push(Box::new(resource_type));
            param_idx += 1;
        }

        if let Some(resource_id) = query.resource_id {
            query_str.push_str(&format!(" AND resource_id = ${}", param_idx));
            params.push(Box::new(resource_id));
            param_idx += 1;
        }

        if let Some(status) = query.status {
            let status_str = match status {
                AuditStatus::Success => "success",
                AuditStatus::Failure => "failure",
                AuditStatus::Denied => "denied",
            };
            query_str.push_str(&format!(" AND status = ${}", param_idx));
            params.push(Box::new(status_str.to_string()));
            param_idx += 1;
        }

        if let Some(start_time) = query.start_time {
            query_str.push_str(&format!(" AND timestamp >= ${}", param_idx));
            params.push(Box::new(start_time));
            param_idx += 1;
        }

        if let Some(end_time) = query.end_time {
            query_str.push_str(&format!(" AND timestamp <= ${}", param_idx));
            params.push(Box::new(end_time));
            param_idx += 1;
        }

        if let Some(namespace) = query.namespace {
            query_str.push_str(&format!(" AND namespace = ${}", param_idx));
            params.push(Box::new(namespace));
            param_idx += 1;
        }

        // Order by timestamp desc
        query_str.push_str(" ORDER BY timestamp DESC");

        if let Some(limit) = query.limit {
            query_str.push_str(&format!(" LIMIT ${}", param_idx));
            params.push(Box::new(limit as i64));
            param_idx += 1;
        }

        if let Some(offset) = query.offset {
            query_str.push_str(&format!(" OFFSET ${}", param_idx));
            params.push(Box::new(offset as i64));
        }

        let db_params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let rows = client
            .query(&query_str, &db_params)
            .await
            .map_err(|e| AuditError::LoggingError(e.to_string()))?;

        let mut logs = Vec::new();
        for row in rows {
            let status_str: String = row.get("status");
            let status = match status_str.as_str() {
                "success" => AuditStatus::Success,
                "failure" => AuditStatus::Failure,
                "denied" => AuditStatus::Denied,
                _ => AuditStatus::Failure, // Fallback
            };

            let metadata_val: Option<serde_json::Value> = row.try_get("metadata").ok();
            let metadata = if let Some(val) = metadata_val {
                 serde_json::from_value(val).unwrap_or_default()
            } else {
                 if let Ok(s) = row.try_get::<_, String>("metadata") {
                     serde_json::from_str(&s).unwrap_or_default()
                 } else {
                     HashMap::new()
                 }
            };

            logs.push(AuditLog {
                id: row.get("id"),
                timestamp: row.get("timestamp"),
                action: row.get("action"),
                actor: row.get("actor_id"),
                resource_type: row.get("resource_type"),
                resource_id: row.get("resource_id"),
                status,
                ip: row.get("ip"),
                user_agent: row.get("user_agent"),
                namespace: row.try_get("namespace").ok(),
                metadata,
            });
        }

        Ok(logs)
    }
}

/// Console backend for development
pub struct ConsoleBackend;

#[async_trait]
impl AuditBackend for ConsoleBackend {
    async fn log(&self, entry: AuditLog) -> Result<(), AuditError> {
        println!("[AUDIT] {:?}", entry);
        Ok(())
    }
}

/// File-based backend
pub struct FileBackend {
    file: tokio::sync::Mutex<tokio::fs::File>,
}

impl FileBackend {
    /// Create a new file backend
    pub async fn new(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .await?;

        Ok(Self {
            file: tokio::sync::Mutex::new(file),
        })
    }
}

#[async_trait]
impl AuditBackend for FileBackend {
    async fn log(&self, entry: AuditLog) -> Result<(), AuditError> {
        let mut file = self.file.lock().await;
        let line =
            serde_json::to_string(&entry).map_err(|e| AuditError::LoggingError(e.to_string()))?;

        use tokio::io::AsyncWriteExt;
        file.write_all(line.as_bytes())
            .await
            .map_err(|e| AuditError::LoggingError(e.to_string()))?;
        file.write_all(b"\n")
            .await
            .map_err(|e| AuditError::LoggingError(e.to_string()))?;

        Ok(())
    }
}
