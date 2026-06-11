//! Audit log storage backends

use super::*;
use async_trait::async_trait;
use deadpool_postgres::Pool;
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
                    metadata JSONB
                )",
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
                resource_id, status, ip, user_agent, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
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
                ],
            )
            .await
            .map_err(|e| AuditError::LoggingError(e.to_string()))?;

        Ok(())
    }

    async fn query(&self, query: &AuditQuery) -> Result<Vec<AuditLog>, AuditError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| AuditError::QueryError(e.to_string()))?;

        let mut sql = "SELECT id, timestamp, action, actor_id, resource_type, resource_id, status, ip, user_agent, metadata FROM audit_logs WHERE 1=1".to_string();
        let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = Vec::new();
        let mut param_idx = 1;

        if let Some(action) = &query.action {
            sql.push_str(&format!(" AND action = ${}", param_idx));
            params.push(Box::new(action));
            param_idx += 1;
        }

        if let Some(actor) = &query.actor {
            sql.push_str(&format!(" AND actor_id = ${}", param_idx));
            params.push(Box::new(actor));
            param_idx += 1;
        }

        if let Some(resource_type) = &query.resource_type {
            sql.push_str(&format!(" AND resource_type = ${}", param_idx));
            params.push(Box::new(resource_type));
            param_idx += 1;
        }

        if let Some(resource_id) = &query.resource_id {
            sql.push_str(&format!(" AND resource_id = ${}", param_idx));
            params.push(Box::new(resource_id));
            param_idx += 1;
        }

        if let Some(status) = &query.status {
            sql.push_str(&format!(" AND status = ${}", param_idx));
            let status_str = match status {
                AuditStatus::Success => "success",
                AuditStatus::Failure => "failure",
                AuditStatus::Denied => "denied",
            };
            params.push(Box::new(status_str));
            param_idx += 1;
        }

        if let Some(start_time) = &query.start_time {
            sql.push_str(&format!(" AND timestamp >= ${}", param_idx));
            params.push(Box::new(start_time));
            param_idx += 1;
        }

        if let Some(end_time) = &query.end_time {
            sql.push_str(&format!(" AND timestamp <= ${}", param_idx));
            params.push(Box::new(end_time));
            param_idx += 1;
        }

        sql.push_str(" ORDER BY timestamp DESC");

        if let Some(limit) = query.limit {
            sql.push_str(&format!(" LIMIT ${}", param_idx));
            params.push(Box::new(limit as i64));
            param_idx += 1;
        }

        if let Some(offset) = query.offset {
            sql.push_str(&format!(" OFFSET ${}", param_idx));
            params.push(Box::new(offset as i64));
        }

        let params_slice: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let rows = client
            .query(&sql, &params_slice)
            .await
            .map_err(|e| AuditError::QueryError(e.to_string()))?;

        let mut logs = Vec::new();
        for row in rows {
            let status_str: String = row.get("status");
            let status = match status_str.as_str() {
                "success" => AuditStatus::Success,
                "failure" => AuditStatus::Failure,
                "denied" => AuditStatus::Denied,
                _ => AuditStatus::Failure, // Fallback
            };

            let metadata_val: serde_json::Value = row.get("metadata");
            let metadata: HashMap<String, String> =
                serde_json::from_value(metadata_val).unwrap_or_default();

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
                namespace: None,
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
