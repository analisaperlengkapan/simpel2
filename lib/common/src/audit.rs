use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::context::RequestContext;

/// Standard severity levels for audit logs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuditSeverity {
    Info,
    Notice,
    Warning,
    Error,
    Critical,
    Alert,
    Emergency,
}

impl AuditSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Notice => "notice",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Critical => "critical",
            Self::Alert => "alert",
            Self::Emergency => "emergency",
        }
    }
}

/// Standard audit event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique ID for the audit event
    pub id: String,
    /// When the event occurred
    pub timestamp: DateTime<Utc>,
    /// Which service generated the event
    pub service: String,
    /// Type of action being audited (e.g. "login", "encrypt", "delete_user")
    pub action: String,
    /// Status of the action (success, failure, denied)
    pub status: String,
    /// Severity level
    pub severity: AuditSeverity,
    /// Actor who performed the action (User ID or "system")
    pub actor_id: String,
    /// Actor's email or username
    pub actor_name: Option<String>,
    /// Request context information
    pub context: Option<RequestContext>,
    /// Resource being acted upon (e.g. "user:123", "key:abc")
    pub resource: Option<String>,
    /// Additional detailed information (sanitized/masked)
    pub details: serde_json::Value,
}

impl AuditEvent {
    pub fn new(
        service: String,
        action: String,
        status: String,
        severity: AuditSeverity,
        actor_id: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            service,
            action,
            status,
            severity,
            actor_id,
            actor_name: None,
            context: None,
            resource: None,
            details: serde_json::json!({}),
        }
    }
}
