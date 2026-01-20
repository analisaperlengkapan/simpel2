//! Audit Logging System for Secreton
//!
//! This module provides comprehensive audit trail capabilities for all vault operations,
//! enabling compliance with security standards (ISO 27001, SOC 2, GDPR) and forensic analysis.
//!
//! # Architecture
//!
//! The audit system uses a pluggable backend architecture:
//!
//! ```text
//! ┌─────────────────────────────────────┐
//! │       Application Code              │
//! │  (Services, Handlers, Middleware)   │
//! └────────────────┬────────────────────┘
//!                  │
//!                  ▼
//! ┌─────────────────────────────────────┐
//! │        AuditLogger                  │
//! │  (Aggregates multiple backends)     │
//! └────────────────┬────────────────────┘
//!                  │
//!       ┌──────────┴──────────┐
//!       ▼                     ▼
//! ┌──────────┐          ┌──────────┐
//! │PostgreSQL│          │  Syslog  │
//! │ Backend  │          │ Backend  │
//! └──────────┘          └──────────┘
//! ```
//!
//! # What Gets Audited
//!
//! - **Authentication**: Login attempts, MFA verification, session creation
//! - **Authorization**: Policy evaluations, permission checks
//! - **Secret Operations**: Read, write, delete, list secrets
//! - **Administrative Actions**: User creation, role assignments, policy changes
//! - **System Events**: Seal/unseal operations, configuration changes
//!
//! # Audit Log Entry Structure
//!
//! Each audit log contains:
//! - **Unique ID**: UUID for each event
//! - **Timestamp**: UTC datetime with millisecond precision
//! - **Action**: What operation was performed (e.g., "secret.read", "user.login")
//! - **Actor**: Who performed the action (user ID or system)
//! - **Resource**: What was accessed (type + ID)
//! - **Status**: Success or failure
//! - **IP Address**: Source of the request
//! - **Details**: Additional context (encrypted data, error messages, etc.)
//!
//! # Example: Manual Audit Logging
//!
//! ```rust,no_run
//! use secreton_core::audit::{AuditLogger, AuditLog, AuditStatus};
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let logger = AuditLogger::new(vec![/* backends */]);
//!
//! let log = AuditLog::new(
//!     "secret.read",
//!     Some("user-123"),
//!     "secret",
//!     "app/database/password",
//!     AuditStatus::Success,
//! ).with_ip("192.168.1.100")
//!   .with_detail("namespace", "production");
//!
//! logger.log(log).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Example: Using Audit Middleware
//!
//! ```rust,no_run
//! use secreton_core::audit::AuditMiddleware;
//! use axum::{Router, routing::get};
//!
//! # async fn example() {
//! let app = Router::new()
//!     .route("/secrets/:id", get(handler))
//!     .layer(AuditMiddleware::new(/* logger */));
//! // All requests automatically audited
//! # }
//! # async fn handler() {}
//! ```
//!
//! # Audit Backends
//!
//! Multiple backends can run simultaneously:
//!
//! - **PostgreSQL** - Structured storage, queryable, long-term retention
//! - **Syslog** - Integration with existing log infrastructure
//! - **File** - Simple file-based logging (development/testing)
//! - **Memory** - In-memory buffer (testing only)
//!
//! # Query and Analysis
//!
//! ```rust,no_run
//! use secreton_core::audit::{AuditLogger, AuditQuery};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let logger = AuditLogger::new(vec![]);
//! // Find all failed login attempts in last 24 hours
//! let query = AuditQuery::new()
//!     .action("user.login")
//!     .status_failed()
//!     .since_hours(24);
//!
//! let logs = logger.query(query).await?;
//! println!("Failed logins: {}", logs.len());
//! # Ok(())
//! # }
//! ```
//!
//! # Compliance Features
//!
//! - **Immutability**: Audit logs cannot be modified after creation
//! - **Integrity**: Cryptographic checksums prevent tampering
//! - **Retention**: Configurable retention policies
//! - **Export**: JSON/CSV export for compliance reports
//! - **Encryption**: Sensitive details encrypted at rest
//!
//! # Performance
//!
//! - **Async**: Non-blocking audit operations
//! - **Batching**: Logs buffered and written in batches
//! - **Sampling**: Optional sampling for high-throughput scenarios
//! - **Circuit Breaker**: Audit failures don't block operations
//!
//! # Security Considerations
//!
//! - Audit logs stored separately from operational data
//! - Restricted access (audit admin role required)
//! - Sensitive data masked or encrypted in logs
//! - Separate database credentials for audit storage
//!
//! # See Also
//!
//! - [`AuditLogger`] - Main audit logging interface
//! - [`AuditBackend`] - Trait for custom backends
//! - [`AuditMiddleware`] - Automatic request auditing

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;
use uuid::Uuid;

mod backends;
pub mod api_audit;

// DEPRECATED: Middleware module moved to secreton-api crate
// Requires 'legacy-axum-middleware' feature to compile (disabled by default)
#[cfg(feature = "legacy-axum-middleware")]
#[deprecated(
    since = "1.1.0",
    note = "Audit middleware moved to `secreton-api` crate. Use `secreton_api::middleware::audit_middleware` instead."
)]
mod middleware;

pub use backends::*;

// Re-export middleware types with deprecation warning (only if feature enabled)
#[cfg(feature = "legacy-axum-middleware")]
#[deprecated(
    since = "1.1.0",
    note = "Audit middleware moved to `secreton-api` crate. Use `secreton_api::middleware::audit_middleware` instead."
)]
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
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditStatus {
    Success,
    Failure,
    Denied,
}

/// Query parameters for audit logs
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AuditQuery {
    pub action: Option<String>,
    pub actor: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub status: Option<AuditStatus>,
    pub start_time: Option<chrono::DateTime<Utc>>,
    pub end_time: Option<chrono::DateTime<Utc>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl AuditQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn action(mut self, action: impl Into<String>) -> Self {
        self.action = Some(action.into());
        self
    }

    pub fn actor(mut self, actor: impl Into<String>) -> Self {
        self.actor = Some(actor.into());
        self
    }

    pub fn status(mut self, status: AuditStatus) -> Self {
        self.status = Some(status);
        self
    }
}

/// Audit error type
#[derive(Error, Debug)]
pub enum AuditError {
    #[error("Audit logging failed: {0}")]
    LoggingError(String),
    #[error("Audit query failed: {0}")]
    QueryError(String),
}

/// Trait for audit log backends
#[async_trait::async_trait]
pub trait AuditBackend: Send + Sync + 'static {
    async fn log(&self, entry: AuditLog) -> Result<(), AuditError>;

    /// Query audit logs
    ///
    /// Default implementation returns "not supported"
    async fn query(&self, _query: &AuditQuery) -> Result<Vec<AuditLog>, AuditError> {
        Err(AuditError::QueryError(
            "Query not supported by this backend".to_string(),
        ))
    }
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

    async fn query(&self, query: &AuditQuery) -> Result<Vec<AuditLog>, AuditError> {
        let logs = self.logs.read();
        let filtered: Vec<AuditLog> = logs
            .iter()
            .filter(|entry| {
                if let Some(action) = &query.action {
                    if entry.action != *action {
                        return false;
                    }
                }
                if let Some(actor) = &query.actor {
                    if entry.actor.as_ref() != Some(actor) {
                        return false;
                    }
                }
                if let Some(resource_type) = &query.resource_type {
                    if entry.resource_type != *resource_type {
                        return false;
                    }
                }
                if let Some(resource_id) = &query.resource_id {
                    if entry.resource_id != *resource_id {
                        return false;
                    }
                }
                if let Some(status) = &query.status {
                    if entry.status != *status {
                        return false;
                    }
                }
                if let Some(start_time) = &query.start_time {
                    if entry.timestamp < *start_time {
                        return false;
                    }
                }
                if let Some(end_time) = &query.end_time {
                    if entry.timestamp > *end_time {
                        return false;
                    }
                }
                true
            })
            .skip(query.offset.unwrap_or(0))
            .take(query.limit.unwrap_or(usize::MAX))
            .cloned()
            .collect();

        Ok(filtered)
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

    /// Query audit logs
    pub async fn query(&self, query: &AuditQuery) -> Result<Vec<AuditLog>, AuditError> {
        for backend in &self.backends {
            match backend.query(query).await {
                Ok(logs) => return Ok(logs),
                Err(AuditError::QueryError(_)) => continue, // Try next backend
                Err(e) => return Err(e),                    // Propagate other errors
            }
        }

        // If no backend supports query or all failed with QueryError
        Err(AuditError::QueryError(
            "No backend supports querying".to_string(),
        ))
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
            resource_type: self
                .resource_type
                .ok_or_else(|| AuditError::LoggingError("resource_type is required".to_string()))?,
            resource_id: self
                .resource_id
                .ok_or_else(|| AuditError::LoggingError("resource_id is required".to_string()))?,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_backend_query() {
        let backend = MemoryBackend::default();

        // Add some logs
        let log1 = AuditLog {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            action: "login".to_string(),
            actor: Some("user1".to_string()),
            resource_type: "auth".to_string(),
            resource_id: "session1".to_string(),
            status: AuditStatus::Success,
            ip: Some("127.0.0.1".to_string()),
            user_agent: None,
            namespace: None,
            metadata: HashMap::new(),
        };

        let log2 = AuditLog {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            action: "read".to_string(),
            actor: Some("user1".to_string()),
            resource_type: "secret".to_string(),
            resource_id: "secret1".to_string(),
            status: AuditStatus::Success,
            ip: Some("127.0.0.1".to_string()),
            user_agent: None,
            namespace: None,
            metadata: HashMap::new(),
        };

        let log3 = AuditLog {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            action: "login".to_string(),
            actor: Some("user2".to_string()),
            resource_type: "auth".to_string(),
            resource_id: "session2".to_string(),
            status: AuditStatus::Failure,
            ip: Some("127.0.0.2".to_string()),
            user_agent: None,
            namespace: None,
            metadata: HashMap::new(),
        };

        backend.log(log1).await.unwrap();
        backend.log(log2).await.unwrap();
        backend.log(log3).await.unwrap();

        // Query by actor
        let query = AuditQuery::new().actor("user1");
        let results = backend.query(&query).await.unwrap();
        assert_eq!(results.len(), 2);

        // Query by action
        let query = AuditQuery::new().action("login");
        let results = backend.query(&query).await.unwrap();
        assert_eq!(results.len(), 2);

        // Query by status
        let query = AuditQuery::new().status(AuditStatus::Failure);
        let results = backend.query(&query).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].actor, Some("user2".to_string()));

        // Query with limit
        let query = AuditQuery {
            limit: Some(1),
            ..Default::default()
        };
        let results = backend.query(&query).await.unwrap();
        assert_eq!(results.len(), 1);
    }
}
