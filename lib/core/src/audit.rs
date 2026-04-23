//! Audit logging module for SIMPEL
//!
//! Provides comprehensive audit logging with database persistence,
//! query capabilities, and filtering.

use crate::context::RequestContext;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

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

impl fmt::Display for AuditSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Audit event types for different operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuditEvent {
    // Authentication events
    UserLogin {
        user_id: Uuid,
        username: String,
        success: bool,
        mfa_used: bool,
    },
    UserLogout {
        user_id: Uuid,
        username: String,
    },
    PasswordChanged {
        user_id: Uuid,
        username: String,
    },
    MfaEnabled {
        user_id: Uuid,
        username: String,
        method: String,
    },
    MfaDisabled {
        user_id: Uuid,
        username: String,
    },

    // User management events
    UserCreated {
        user_id: Uuid,
        username: String,
        created_by: Uuid,
    },
    UserUpdated {
        user_id: Uuid,
        username: String,
        updated_by: Uuid,
        changes: Vec<String>,
    },
    UserDeleted {
        user_id: Uuid,
        username: String,
        deleted_by: Uuid,
    },
    UserEnabled {
        user_id: Uuid,
        username: String,
        enabled_by: Uuid,
    },
    UserDisabled {
        user_id: Uuid,
        username: String,
        disabled_by: Uuid,
    },

    // Role and permission events
    RoleAssigned {
        user_id: Uuid,
        role_name: String,
        assigned_by: Uuid,
    },
    RoleRevoked {
        user_id: Uuid,
        role_name: String,
        revoked_by: Uuid,
    },
    PermissionGranted {
        user_id: Uuid,
        permission: String,
        granted_by: Uuid,
    },
    PermissionRevoked {
        user_id: Uuid,
        permission: String,
        revoked_by: Uuid,
    },

    // Secret management events
    SecretStored {
        path: String,
        version: u32,
        user_id: Uuid,
    },
    SecretRetrieved {
        path: String,
        version: Option<u32>,
        user_id: Uuid,
    },
    SecretDeleted {
        path: String,
        user_id: Uuid,
    },
    SecretRotated {
        path: String,
        old_version: u32,
        new_version: u32,
        user_id: Uuid,
    },

    // Workflow events
    WorkflowTransition {
        entity_id: Uuid,
        entity_type: String,
        from_state: String,
        to_state: String,
        user_id: Uuid,
    },
    WorkflowApproved {
        entity_id: Uuid,
        entity_type: String,
        approver_id: Uuid,
        comments: Option<String>,
    },
    WorkflowRejected {
        entity_id: Uuid,
        entity_type: String,
        rejector_id: Uuid,
        reason: String,
    },

    // Batch operations
    BatchOperation {
        batch_id: Uuid,
        operation: String,
        count: usize,
        success_count: usize,
        failure_count: usize,
        user_id: Uuid,
    },

    // Data access events
    DataExported {
        entity_type: String,
        record_count: usize,
        format: String,
        user_id: Uuid,
    },
    DataImported {
        entity_type: String,
        record_count: usize,
        format: String,
        user_id: Uuid,
    },

    // Security events
    AccessDenied {
        user_id: Uuid,
        resource: String,
        action: String,
        reason: String,
    },
    SuspiciousActivity {
        user_id: Option<Uuid>,
        activity_type: String,
        details: String,
    },
    SecurityPolicyViolation {
        user_id: Uuid,
        policy: String,
        violation: String,
    },

    // System events
    SystemStarted {
        service: String,
        version: String,
    },
    SystemStopped {
        service: String,
    },
    ConfigurationChanged {
        setting: String,
        changed_by: Uuid,
    },

    // Integration events
    IntegrationSync {
        service: String,
        status: String,
        records_synced: usize,
        duration_ms: u64,
    },
    IntegrationError {
        service: String,
        error: String,
    },

    // Custom event for extensibility
    Custom {
        action: String,
        details: serde_json::Value,
    },
}

impl AuditEvent {
    /// Get the action name for this event
    pub fn action(&self) -> &str {
        match self {
            Self::UserLogin { .. } => "user_login",
            Self::UserLogout { .. } => "user_logout",
            Self::PasswordChanged { .. } => "password_changed",
            Self::MfaEnabled { .. } => "mfa_enabled",
            Self::MfaDisabled { .. } => "mfa_disabled",
            Self::UserCreated { .. } => "user_created",
            Self::UserUpdated { .. } => "user_updated",
            Self::UserDeleted { .. } => "user_deleted",
            Self::UserEnabled { .. } => "user_enabled",
            Self::UserDisabled { .. } => "user_disabled",
            Self::RoleAssigned { .. } => "role_assigned",
            Self::RoleRevoked { .. } => "role_revoked",
            Self::PermissionGranted { .. } => "permission_granted",
            Self::PermissionRevoked { .. } => "permission_revoked",
            Self::SecretStored { .. } => "secret_stored",
            Self::SecretRetrieved { .. } => "secret_retrieved",
            Self::SecretDeleted { .. } => "secret_deleted",
            Self::SecretRotated { .. } => "secret_rotated",
            Self::WorkflowTransition { .. } => "workflow_transition",
            Self::WorkflowApproved { .. } => "workflow_approved",
            Self::WorkflowRejected { .. } => "workflow_rejected",
            Self::BatchOperation { .. } => "batch_operation",
            Self::DataExported { .. } => "data_exported",
            Self::DataImported { .. } => "data_imported",
            Self::AccessDenied { .. } => "access_denied",
            Self::SuspiciousActivity { .. } => "suspicious_activity",
            Self::SecurityPolicyViolation { .. } => "security_policy_violation",
            Self::SystemStarted { .. } => "system_started",
            Self::SystemStopped { .. } => "system_stopped",
            Self::ConfigurationChanged { .. } => "configuration_changed",
            Self::IntegrationSync { .. } => "integration_sync",
            Self::IntegrationError { .. } => "integration_error",
            Self::Custom { action, .. } => action,
        }
    }

    /// Get the actor (user) ID for this event, if applicable
    pub fn actor_id(&self) -> Option<Uuid> {
        match self {
            Self::UserLogin { user_id, .. }
            | Self::UserLogout { user_id, .. }
            | Self::PasswordChanged { user_id, .. }
            | Self::MfaEnabled { user_id, .. }
            | Self::MfaDisabled { user_id, .. }
            | Self::SecretStored { user_id, .. }
            | Self::SecretRetrieved { user_id, .. }
            | Self::SecretDeleted { user_id, .. }
            | Self::SecretRotated { user_id, .. }
            | Self::WorkflowTransition { user_id, .. }
            | Self::WorkflowApproved {
                approver_id: user_id,
                ..
            }
            | Self::WorkflowRejected {
                rejector_id: user_id,
                ..
            }
            | Self::BatchOperation { user_id, .. }
            | Self::DataExported { user_id, .. }
            | Self::DataImported { user_id, .. }
            | Self::AccessDenied { user_id, .. }
            | Self::SecurityPolicyViolation { user_id, .. }
            | Self::ConfigurationChanged {
                changed_by: user_id,
                ..
            } => Some(*user_id),
            Self::UserCreated { created_by, .. }
            | Self::UserUpdated {
                updated_by: created_by,
                ..
            }
            | Self::UserDeleted {
                deleted_by: created_by,
                ..
            }
            | Self::UserEnabled {
                enabled_by: created_by,
                ..
            }
            | Self::UserDisabled {
                disabled_by: created_by,
                ..
            }
            | Self::RoleAssigned {
                assigned_by: created_by,
                ..
            }
            | Self::RoleRevoked {
                revoked_by: created_by,
                ..
            }
            | Self::PermissionGranted {
                granted_by: created_by,
                ..
            }
            | Self::PermissionRevoked {
                revoked_by: created_by,
                ..
            } => Some(*created_by),
            Self::SuspiciousActivity { user_id, .. } => *user_id,
            _ => None,
        }
    }
}

/// Standard audit log entry structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    /// Unique ID for the audit event
    pub id: Uuid,
    /// When the event occurred
    pub timestamp: DateTime<Utc>,
    /// Which service generated the event
    pub service: String,
    /// The audit event with all details
    pub event: AuditEvent,
    /// Severity level
    pub severity: AuditSeverity,
    /// Request context information
    pub context: Option<RequestContext>,
    /// Additional metadata
    pub metadata: serde_json::Value,
}

impl AuditLogEntry {
    pub fn new(service: String, event: AuditEvent, severity: AuditSeverity) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            service,
            event,
            severity,
            context: None,
            metadata: serde_json::json!({}),
        }
    }

    pub fn with_context(mut self, context: RequestContext) -> Self {
        self.context = Some(context);
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Audit log query filter
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditLogFilter {
    pub service: Option<String>,
    pub action: Option<String>,
    pub actor_id: Option<Uuid>,
    pub severity: Option<AuditSeverity>,
    pub from_timestamp: Option<DateTime<Utc>>,
    pub to_timestamp: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
