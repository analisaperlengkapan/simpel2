//! Audit event types shared across modules.
//!
//! The actual sink (postgres `audit_log` table writer) lives in the service
//! crate at `src/shared/audit.rs`, but the event shape and `AuditSink` trait
//! are here so any module can produce events without depending on the sink
//! implementation.

use chrono::{DateTime, Utc};
use uuid::Uuid;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuditAction {
    Create,
    Read,
    Update,
    Delete,
    Login,
    Logout,
    Approve,
    Reject,
    Submit,
    Cancel,
    Export,
    Import,
    Custom,
}

impl AuditAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Read => "read",
            Self::Update => "update",
            Self::Delete => "delete",
            Self::Login => "login",
            Self::Logout => "logout",
            Self::Approve => "approve",
            Self::Reject => "reject",
            Self::Submit => "submit",
            Self::Cancel => "cancel",
            Self::Export => "export",
            Self::Import => "import",
            Self::Custom => "custom",
        }
    }
}

/// A single audit log entry.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEvent {
    pub id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub actor_user_id: Option<Uuid>,
    pub actor_username: Option<String>,
    pub actor_ip: Option<String>,
    pub action: AuditAction,
    /// Free-form action name when `action == Custom` (e.g. "workflow.delegate").
    pub action_name: Option<String>,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub module: String,
    pub success: bool,
    pub message: Option<String>,
    #[cfg(feature = "serde")]
    pub metadata: Option<serde_json::Value>,
    #[cfg(not(feature = "serde"))]
    pub metadata: Option<String>,
}

impl AuditEvent {
    pub fn new(
        module: impl Into<String>,
        action: AuditAction,
        resource_type: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            actor_user_id: None,
            actor_username: None,
            actor_ip: None,
            action,
            action_name: None,
            resource_type: resource_type.into(),
            resource_id: None,
            module: module.into(),
            success: true,
            message: None,
            metadata: None,
        }
    }

    pub fn actor(mut self, user_id: Uuid, username: impl Into<String>) -> Self {
        self.actor_user_id = Some(user_id);
        self.actor_username = Some(username.into());
        self
    }

    pub fn ip(mut self, ip: impl Into<String>) -> Self {
        self.actor_ip = Some(ip.into());
        self
    }

    pub fn resource_id(mut self, id: impl Into<String>) -> Self {
        self.resource_id = Some(id.into());
        self
    }

    pub fn failure(mut self, msg: impl Into<String>) -> Self {
        self.success = false;
        self.message = Some(msg.into());
        self
    }

    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }
}
