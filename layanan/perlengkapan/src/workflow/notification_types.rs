//! Workflow notification domain types.
//!
//! Previously lived alongside the deleted `notifikasi_client.rs` gRPC client.
//! The workflow engine produces values of [`WorkflowNotificationType`] +
//! [`NotificationPriority`] and the [`to_notification_message`] adapter
//! converts them into the
//! [`lib_perlengkapan::contracts::NotificationMessage`] DTO that the
//! [`NotificationSender`](lib_perlengkapan::contracts::NotificationSender)
//! trait consumes.

use lib_perlengkapan::contracts::{
    NotificationChannel, NotificationMessage, NotificationPriority as ContractPriority,
};
use uuid::Uuid;

/// Notification priority levels used by the workflow engine.
///
/// Wire-distinct from the contract's
/// [`NotificationPriority`](lib_perlengkapan::contracts::NotificationPriority)
/// only because the workflow needed a `Normal` variant historically; mapped
/// one-to-one in [`to_contract_priority`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Urgent,
}

impl NotificationPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Normal => "normal",
            Self::High => "high",
            Self::Urgent => "urgent",
        }
    }

    fn to_contract_priority(self) -> ContractPriority {
        match self {
            Self::Low => ContractPriority::Low,
            Self::Normal => ContractPriority::Medium,
            Self::High => ContractPriority::High,
            Self::Urgent => ContractPriority::Critical,
        }
    }
}

/// Notification type for workflow transitions.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkflowNotificationType {
    /// Workflow state transition notification.
    WorkflowTransition {
        entity_type: String,
        entity_id: String,
        from_state: String,
        to_state: String,
        transition_by: String,
        catatan: Option<String>,
    },
    /// Approval required notification.
    ApprovalRequired {
        entity_type: String,
        entity_id: String,
        current_state: String,
        required_role: String,
        deadline: Option<String>,
    },
    /// Approval completed notification.
    ApprovalCompleted {
        entity_type: String,
        entity_id: String,
        approved_by: String,
        document_url: Option<String>,
    },
    /// Rejection notification.
    Rejected {
        entity_type: String,
        entity_id: String,
        rejected_by: String,
        reason: Option<String>,
    },
    /// Revision required notification.
    RevisionRequired {
        entity_type: String,
        entity_id: String,
        requested_by: String,
        notes: Option<String>,
    },
    /// SLA breach escalation notification (to approver).
    SlaBreachEscalation {
        entity_type: String,
        entity_id: String,
        current_state: String,
        sla_deadline: String,
        breach_duration_minutes: i64,
        days_overdue: i32,
    },
    /// SLA breach informational notification (to requester).
    SlaBreachInfo {
        entity_type: String,
        entity_id: String,
        current_state: String,
        sla_deadline: String,
        breach_duration_minutes: i64,
    },
}

impl WorkflowNotificationType {
    /// Logical event name used as the Handlebars template id by the notifikasi
    /// module.
    pub fn event(&self) -> &'static str {
        match self {
            Self::WorkflowTransition { .. } => "workflow.transition",
            Self::ApprovalRequired { .. } => "workflow.approval_required",
            Self::ApprovalCompleted { .. } => "workflow.approval_completed",
            Self::Rejected { .. } => "workflow.rejected",
            Self::RevisionRequired { .. } => "workflow.revision_required",
            Self::SlaBreachEscalation { .. } => "workflow.sla_breach_escalation",
            Self::SlaBreachInfo { .. } => "workflow.sla_breach_info",
        }
    }

    /// Short human-readable title.
    pub fn title(&self) -> String {
        match self {
            Self::WorkflowTransition {
                entity_type,
                to_state,
                ..
            } => format!("{}: berpindah ke {}", entity_type, to_state),
            Self::ApprovalRequired { entity_type, .. } => {
                format!("Persetujuan diperlukan: {}", entity_type)
            }
            Self::ApprovalCompleted { entity_type, .. } => {
                format!("Disetujui: {}", entity_type)
            }
            Self::Rejected { entity_type, .. } => format!("Ditolak: {}", entity_type),
            Self::RevisionRequired { entity_type, .. } => {
                format!("Perlu revisi: {}", entity_type)
            }
            Self::SlaBreachEscalation { entity_type, .. } => {
                format!("SLA terlampaui (eskalasi): {}", entity_type)
            }
            Self::SlaBreachInfo { entity_type, .. } => {
                format!("SLA terlampaui: {}", entity_type)
            }
        }
    }

    /// One-line body suitable for in-app preview; rich rendering is handled
    /// by the notifikasi template engine using `variables`.
    pub fn body(&self) -> String {
        match self {
            Self::WorkflowTransition {
                from_state,
                to_state,
                transition_by,
                ..
            } => format!("{} ➜ {} oleh {}", from_state, to_state, transition_by),
            Self::ApprovalRequired { required_role, .. } => {
                format!("Menunggu tindakan dari {}.", required_role)
            }
            Self::ApprovalCompleted { approved_by, .. } => {
                format!("Disetujui oleh {}.", approved_by)
            }
            Self::Rejected {
                rejected_by,
                reason,
                ..
            } => match reason {
                Some(r) => format!("Ditolak oleh {}: {}", rejected_by, r),
                None => format!("Ditolak oleh {}.", rejected_by),
            },
            Self::RevisionRequired {
                requested_by,
                notes,
                ..
            } => match notes {
                Some(n) => format!("Revisi diminta oleh {}: {}", requested_by, n),
                None => format!("Revisi diminta oleh {}.", requested_by),
            },
            Self::SlaBreachEscalation {
                breach_duration_minutes,
                days_overdue,
                ..
            } => format!(
                "SLA terlampaui {} menit ({} hari).",
                breach_duration_minutes, days_overdue
            ),
            Self::SlaBreachInfo {
                breach_duration_minutes,
                ..
            } => format!("SLA terlampaui {} menit.", breach_duration_minutes),
        }
    }
}

/// Convert a workflow-domain notification into the cross-module
/// [`NotificationMessage`] DTO consumed by the
/// [`NotificationSender`](lib_perlengkapan::contracts::NotificationSender)
/// trait.
pub fn to_notification_message(
    recipient: Uuid,
    notification: &WorkflowNotificationType,
    priority: NotificationPriority,
) -> NotificationMessage {
    NotificationMessage {
        event: notification.event().to_string(),
        recipient_user_id: recipient,
        channels: vec![NotificationChannel::InApp],
        priority: priority.to_contract_priority(),
        title: notification.title(),
        body: notification.body(),
        variables: serde_json::to_value(notification).ok(),
        deeplink: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn priority_as_str_round_trip() {
        assert_eq!(NotificationPriority::Low.as_str(), "low");
        assert_eq!(NotificationPriority::Normal.as_str(), "normal");
        assert_eq!(NotificationPriority::High.as_str(), "high");
        assert_eq!(NotificationPriority::Urgent.as_str(), "urgent");
    }

    #[test]
    fn workflow_notification_serializes_with_tag() {
        let n = WorkflowNotificationType::WorkflowTransition {
            entity_type: "kebutuhan_bmn".into(),
            entity_id: Uuid::new_v4().to_string(),
            from_state: "DRAFT".into(),
            to_state: "SUBMITTED".into(),
            transition_by: "user-123".into(),
            catatan: Some("Test transition".into()),
        };

        let json = serde_json::to_string(&n).unwrap();
        assert!(json.contains("workflow_transition"));
        assert!(json.contains("kebutuhan_bmn"));
    }
}
