//! Audit logging for Secreton Adhyaksa

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;
use uuid::Uuid;

mod backends;
mod middleware;

pub use backends::*;
pub use middleware::*;

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: Uuid,
    pub timestamp: chrono::DateTime<Utc>,
    pub action: String,
    pub actor: Option<String>,
    pub resource_type: String,
    pub resource_id: String,
    pub status: AuditStatus,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    /// Namespace ID for the resource (e.g., "satker-kja001")
    pub namespace: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Status of an audited action
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AuditStatus {
    Success,
    Failure,
    Denied,
}

/// Audit error type
#[derive(Error, Debug)]
pub enum AuditError {
    #[error("Audit logging failed: {0}")]
    LoggingError(String),
}

/// Trait for audit log backends
#[async_trait::async_trait]
pub trait AuditBackend: Send + Sync + 'static {
    async fn log(&self, entry: AuditLog) -> Result<(), AuditError>;
}

/// In-memory audit log backend (for testing/demo)
#[derive(Default)]
pub struct MemoryBackend {
    logs: Arc<parking_lot::RwLock<Vec<AuditLog>>>,
}

#[async_trait::async_trait]
impl AuditBackend for MemoryBackend {
    async fn log(&self, entry: AuditLog) -> Result<(), AuditError> {
        self.logs.write().push(entry);
        Ok(())
    }
}

impl MemoryBackend {
    /// Get all logs (for testing)
    pub fn logs(&self) -> Vec<AuditLog> {
        self.logs.read().clone()
    }
}

/// Main audit logger
#[derive(Clone)]
pub struct AuditLogger {
    backends: Vec<Arc<dyn AuditBackend>>,
}

impl AuditLogger {
    /// Create a new audit logger with the given backends
    pub fn new(backends: Vec<Arc<dyn AuditBackend>>) -> Self {
        Self { backends }
    }

    /// Log an audit event
    pub async fn log(&self, mut entry: AuditLog) -> Result<(), AuditError> {
        entry.id = Uuid::new_v4();
        entry.timestamp = Utc::now();

        for backend in &self.backends {
            if let Err(e) = backend.log(entry.clone()).await {
                tracing::error!("Failed to write to audit log: {}", e);
            }
        }

        Ok(())
    }

    /// Log a namespace-scoped audit event
    ///
    /// Convenience method for logging events with namespace information
    pub async fn log_with_namespace(
        &self,
        action: String,
        actor: Option<String>,
        resource_type: String,
        resource_id: String,
        namespace: String,
        status: AuditStatus,
        ip: Option<String>,
        user_agent: Option<String>,
        metadata: HashMap<String, String>,
    ) -> Result<(), AuditError> {
        let entry = AuditLog {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            action,
            actor,
            resource_type,
            resource_id,
            status,
            ip,
            user_agent,
            namespace: Some(namespace),
            metadata,
        };

        self.log(entry).await
    }
}

impl AuditLog {
    /// Create a new audit log entry builder
    pub fn builder() -> AuditLogBuilder {
        AuditLogBuilder::default()
    }
}

/// Builder for audit log entries
#[derive(Default)]
pub struct AuditLogBuilder {
    action: Option<String>,
    actor: Option<String>,
    resource_type: Option<String>,
    resource_id: Option<String>,
    status: Option<AuditStatus>,
    ip: Option<String>,
    user_agent: Option<String>,
    namespace: Option<String>,
    metadata: HashMap<String, String>,
}

impl AuditLogBuilder {
    pub fn action(mut self, action: impl Into<String>) -> Self {
        self.action = Some(action.into());
        self
    }

    pub fn actor(mut self, actor: impl Into<String>) -> Self {
        self.actor = Some(actor.into());
        self
    }

    pub fn resource_type(mut self, resource_type: impl Into<String>) -> Self {
        self.resource_type = Some(resource_type.into());
        self
    }

    pub fn resource_id(mut self, resource_id: impl Into<String>) -> Self {
        self.resource_id = Some(resource_id.into());
        self
    }

    pub fn status(mut self, status: AuditStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn ip(mut self, ip: impl Into<String>) -> Self {
        self.ip = Some(ip.into());
        self
    }

    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    pub fn namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = Some(namespace.into());
        self
    }

    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn build(self) -> Result<AuditLog, AuditError> {
        Ok(AuditLog {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            action: self
                .action
                .ok_or_else(|| AuditError::LoggingError("action is required".to_string()))?,
            actor: self.actor,
            resource_type: self.resource_type.ok_or_else(|| {
                AuditError::LoggingError("resource_type is required".to_string())
            })?,
            resource_id: self.resource_id.ok_or_else(|| {
                AuditError::LoggingError("resource_id is required".to_string())
            })?,
            status: self
                .status
                .ok_or_else(|| AuditError::LoggingError("status is required".to_string()))?,
            ip: self.ip,
            user_agent: self.user_agent,
            namespace: self.namespace,
         metadata: self.metadata,
        })
    }
}
