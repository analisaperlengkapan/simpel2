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
///
/// Enhanced to meet Requirement 12.1: Include timestamp, actor, action, resource, client IP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: Uuid,
    /// Timestamp in UTC (Requirement 12.1)
    pub timestamp: chrono::DateTime<Utc>,
    /// Action performed (Requirement 12.1)
    pub action: String,
    /// Actor/user performing the action (Requirement 12.1)
    pub actor: Option<String>,
    /// Resource type being accessed (Requirement 12.1)
    pub resource_type: String,
    /// Resource identifier (Requirement 12.1)
    pub resource_id: String,
    /// Status of the operation
    pub status: AuditStatus,
    /// Client IP address (Requirement 12.1)
    pub ip: Option<String>,
    /// User agent string for additional context
    pub user_agent: Option<String>,
    /// Namespace ID for the resource (e.g., "satker-kja001")
    pub namespace: Option<String>,
    /// Additional metadata for context
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
    #[error("Operation not supported by this backend")]
    NotSupported,
}

/// Query parameters for audit logs
#[derive(Debug, Clone, Default)]
pub struct AuditQuery {
    pub action: Option<String>,
    pub actor: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub status: Option<AuditStatus>,
    pub start_time: Option<chrono::DateTime<Utc>>,
    pub end_time: Option<chrono::DateTime<Utc>>,
    pub namespace: Option<String>,
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

    pub fn resource_type(mut self, resource_type: impl Into<String>) -> Self {
        self.resource_type = Some(resource_type.into());
        self
    }

    pub fn status(mut self, status: AuditStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn status_failed(mut self) -> Self {
        self.status = Some(AuditStatus::Failure);
        self
    }

    pub fn since_hours(mut self, hours: i64) -> Self {
        self.start_time = Some(Utc::now() - chrono::Duration::hours(hours));
        self
    }

    pub fn start_time(mut self, start_time: Option<chrono::DateTime<Utc>>) -> Self {
        self.start_time = start_time;
        self
    }

    pub fn end_time(mut self, end_time: Option<chrono::DateTime<Utc>>) -> Self {
        self.end_time = end_time;
        self
    }

    pub fn namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = Some(namespace.into());
        self
    }

    pub fn limit(mut self, limit: Option<usize>) -> Self {
        self.limit = limit;
        self
    }

    pub fn offset(mut self, offset: Option<usize>) -> Self {
        self.offset = offset;
        self
    }
}

/// Trait for audit log backends
#[async_trait::async_trait]
pub trait AuditBackend: Send + Sync + 'static {
    async fn log(&self, entry: AuditLog) -> Result<(), AuditError>;

    async fn query(&self, _query: AuditQuery) -> Result<Vec<AuditLog>, AuditError> {
        Err(AuditError::NotSupported)
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

    async fn query(&self, query: AuditQuery) -> Result<Vec<AuditLog>, AuditError> {
        let logs = self.logs.read();
        let filtered: Vec<AuditLog> = logs.iter().filter(|log| {
            if let Some(ref action) = query.action {
                if &log.action != action { return false; }
            }
            if let Some(ref actor) = query.actor {
                if log.actor.as_ref() != Some(actor) { return false; }
            }
            if let Some(ref resource_type) = query.resource_type {
                if &log.resource_type != resource_type { return false; }
            }
            if let Some(ref resource_id) = query.resource_id {
                if &log.resource_id != resource_id { return false; }
            }
            if let Some(status) = query.status {
                // Approximate status check since AuditStatus doesn't impl PartialEq automatically
                let log_status_str = match log.status {
                    AuditStatus::Success => "Success",
                    AuditStatus::Failure => "Failure",
                    AuditStatus::Denied => "Denied",
                };
                let query_status_str = match status {
                    AuditStatus::Success => "Success",
                    AuditStatus::Failure => "Failure",
                    AuditStatus::Denied => "Denied",
                };
                if log_status_str != query_status_str { return false; }
            }
            if let Some(start_time) = query.start_time {
                if log.timestamp < start_time { return false; }
            }
            if let Some(end_time) = query.end_time {
                if log.timestamp > end_time { return false; }
            }
            if let Some(ref namespace) = query.namespace {
                if log.namespace.as_ref() != Some(namespace) { return false; }
            }
            true
        }).cloned().collect();

        // Apply pagination
        let start = query.offset.unwrap_or(0);
        let end = start + query.limit.unwrap_or(filtered.len());

        if start >= filtered.len() {
            return Ok(vec![]);
        }

        let end = std::cmp::min(end, filtered.len());
        Ok(filtered[start..end].to_vec())
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
    pub async fn query(&self, query: AuditQuery) -> Result<Vec<AuditLog>, AuditError> {
        for backend in &self.backends {
            match backend.query(query.clone()).await {
                Ok(logs) => return Ok(logs),
                Err(AuditError::NotSupported) => continue,
                Err(e) => return Err(e),
            }
        }
        // If no backend supports querying or no backends are configured
        Ok(vec![])
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
