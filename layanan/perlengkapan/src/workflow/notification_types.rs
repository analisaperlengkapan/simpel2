//! Workflow notification domain types.
//!
//! Previously lived alongside the deleted `notifikasi_client.rs` gRPC client.
//! The workflow engine produces values of [`WorkflowNotificationType`] +
//! [`NotificationPriority`] and the [`to_notification_message`] adapter
//! converts them into the
//! [`crate::contracts::NotificationMessage`] DTO that the
//! [`NotificationSender`](crate::contracts::NotificationSender)
//! trait consumes.

use crate::contracts::{
    NotificationChannel, NotificationMessage, NotificationPriority as ContractPriority,
};
use uuid::Uuid;

/// Notification priority levels used by the workflow engine.
///
/// Wire-distinct from the contract's
/// [`NotificationPriority`](crate::contracts::NotificationPriority)
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

/// Human name for a module slug (`entity_type`).
///
/// `entity_type` is the workflow config's `name`, an internal slug like
/// `penghapusan_bmn`. It was being interpolated straight into notification
/// titles, so the inbox read "penghapusan_bmn: berpindah ke SUBMIT_PUSAT".
/// Unknown slugs are title-cased rather than passed through, so a module added
/// later degrades to "Foo Bar" instead of shouting `foo_bar` at a user.
pub fn module_label(entity_type: &str) -> String {
    match entity_type {
        "kebutuhan_bmn" => "Kebutuhan BMN".to_string(),
        "pemakaian_bmn" => "Pemakaian BMN".to_string(),
        "penghapusan_bmn" => "Penghapusan BMN".to_string(),
        "pakaian_dinas" => "Pakaian Dinas".to_string(),
        other => humanize_token(other),
    }
}

/// Human name for a workflow state, resolved through the module's own status
/// enum — the same `label()` the rest of the UI shows.
///
/// Delegating to the enums rather than keeping a table here is deliberate:
/// every hand-written copy of this mapping in the repo had drifted from the
/// enum it copied — the `ms_aktivitas_bmn` seed, `WorkflowStateCode`, and the
/// frontend's `STATUS_OPTIONS` all named 2004 "Analisis Kelayakan" when the
/// enum has it at 2005. A fifth copy here would have been the fourth to rot.
///
/// `pakaian_dinas` is deliberately absent: its `AktivitasStatus` has a
/// `label()` but no `from_state_name`, so its states take the title-cased
/// path. That is the honest outcome — imprecise prose, never a raw
/// `SUBMIT_PUSAT` — and adding a lookup it does not have would be inventing
/// one more copy to keep in step.
pub fn state_label(entity_type: &str, state_name: &str) -> String {
    use crate::kebutuhan_bmn::models::KebutuhanBmnStatus;
    use crate::pemakaian_bmn::models::PemakaianBmnStatus;
    use crate::penghapusan_bmn::models::PenghapusanBmnStatus;

    let mapped = match entity_type {
        "kebutuhan_bmn" => KebutuhanBmnStatus::from_state_name(state_name).map(|s| s.label()),
        "pemakaian_bmn" => PemakaianBmnStatus::from_state_name(state_name).map(|s| s.label()),
        "penghapusan_bmn" => PenghapusanBmnStatus::from_state_name(state_name).map(|s| s.label()),
        _ => None,
    };
    mapped
        .map(str::to_string)
        .unwrap_or_else(|| humanize_token(state_name))
}

/// `SUBMIT_PUSAT` / `penghapusan_bmn` -> `Submit Pusat` / `Penghapusan Bmn`.
///
/// Only ever reached for a token no enum claims; it keeps an unmapped value
/// legible instead of letting SCREAMING_SNAKE_CASE reach a user.
fn humanize_token(token: &str) -> String {
    token
        .split(['_', '-'])
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + &c.as_str().to_lowercase(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Human name for an RBAC role slug (`validator_wilayah` -> "Validator
/// Wilayah"). Same reasoning as [`module_label`]: the slug is an internal
/// identifier and has no business being read by a user.
fn role_label(role: &str) -> String {
    match role {
        "operator_satker" => "Operator Satker".to_string(),
        "validator_satker" => "Validator Satker".to_string(),
        "approver_satker" => "Approver Satker".to_string(),
        "validator_wilayah" => "Validator Wilayah".to_string(),
        "validator_pusat" => "Validator Pusat".to_string(),
        "admin" => "Administrator".to_string(),
        other => humanize_token(other),
    }
}

/// How to name whoever caused a transition.
///
/// The workflow engine only has `user_id`, so the body used to end in a bare
/// UUID ("oleh 33333333-3333-4333-8333-333333333333"). We will not invent a
/// name we do not have, and we will not print 36 hex characters at a reader:
/// a bare id yields `None` and the caller drops the clause entirely. The
/// scheduled paths pass the literal "system", which does name an actor.
fn actor_label(actor: &str) -> Option<String> {
    if actor.eq_ignore_ascii_case("system") {
        return Some("sistem".to_string());
    }
    if Uuid::parse_str(actor).is_ok() {
        return None;
    }
    let trimmed = actor.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
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
            } => format!(
                "{}: berpindah ke {}",
                module_label(entity_type),
                state_label(entity_type, to_state)
            ),
            Self::ApprovalRequired { entity_type, .. } => {
                format!("Persetujuan diperlukan: {}", module_label(entity_type))
            }
            Self::ApprovalCompleted { entity_type, .. } => {
                format!("Disetujui: {}", module_label(entity_type))
            }
            Self::Rejected { entity_type, .. } => {
                format!("Ditolak: {}", module_label(entity_type))
            }
            Self::RevisionRequired { entity_type, .. } => {
                format!("Perlu revisi: {}", module_label(entity_type))
            }
            Self::SlaBreachEscalation { entity_type, .. } => {
                format!("SLA terlampaui (eskalasi): {}", module_label(entity_type))
            }
            Self::SlaBreachInfo { entity_type, .. } => {
                format!("SLA terlampaui: {}", module_label(entity_type))
            }
        }
    }

    /// One-line body suitable for in-app preview; rich rendering is handled
    /// by the notifikasi template engine using `variables`.
    pub fn body(&self) -> String {
        match self {
            Self::WorkflowTransition {
                entity_type,
                from_state,
                to_state,
                transition_by,
                ..
            } => {
                let movement = format!(
                    "{} ➜ {}",
                    state_label(entity_type, from_state),
                    state_label(entity_type, to_state)
                );
                match actor_label(transition_by) {
                    Some(who) => format!("{} oleh {}", movement, who),
                    None => format!("{}.", movement),
                }
            }
            Self::ApprovalRequired { required_role, .. } => {
                format!("Menunggu tindakan dari {}.", role_label(required_role))
            }
            Self::ApprovalCompleted { approved_by, .. } => match actor_label(approved_by) {
                Some(who) => format!("Disetujui oleh {}.", who),
                None => "Pengajuan telah disetujui.".to_string(),
            },
            Self::Rejected {
                rejected_by,
                reason,
                ..
            } => {
                let head = match actor_label(rejected_by) {
                    Some(who) => format!("Ditolak oleh {}", who),
                    None => "Pengajuan ditolak".to_string(),
                };
                match reason {
                    Some(r) => format!("{}: {}", head, r),
                    None => format!("{}.", head),
                }
            }
            Self::RevisionRequired {
                requested_by,
                notes,
                ..
            } => {
                let head = match actor_label(requested_by) {
                    Some(who) => format!("Revisi diminta oleh {}", who),
                    None => "Pengajuan dikembalikan untuk revisi".to_string(),
                };
                match notes {
                    Some(n) => format!("{}: {}", head, n),
                    None => format!("{}.", head),
                }
            }
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
/// [`NotificationSender`](crate::contracts::NotificationSender)
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
        recipient_email: None,
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

    /// The inbox read "penghapusan_bmn: berpindah ke SUBMIT_PUSAT". Both
    /// halves were internal identifiers.
    #[test]
    fn a_transition_title_names_the_module_and_the_state_in_indonesian() {
        let n = WorkflowNotificationType::WorkflowTransition {
            entity_type: "kebutuhan_bmn".into(),
            entity_id: Uuid::new_v4().to_string(),
            from_state: "SUBMIT_WILAYAH".into(),
            to_state: "SUBMIT_PUSAT".into(),
            transition_by: Uuid::new_v4().to_string(),
            catatan: None,
        };

        assert_eq!(
            n.title(),
            "Kebutuhan BMN: berpindah ke Diajukan ke Validator Pusat"
        );
        assert_eq!(
            n.body(),
            "Diajukan ke Validator Wilayah ➜ Diajukan ke Validator Pusat."
        );
    }

    /// A bare user id is not a name. We would rather say nothing about the
    /// actor than print 36 hex characters at a reader.
    #[test]
    fn a_uuid_actor_is_dropped_but_a_named_one_is_kept() {
        assert_eq!(actor_label(&Uuid::new_v4().to_string()), None);
        assert_eq!(actor_label("system"), Some("sistem".to_string()));
        assert_eq!(actor_label("Budi"), Some("Budi".to_string()));
        assert_eq!(actor_label("   "), None);
    }

    /// Canary: without the mapping these assertions must fail. If a future
    /// refactor makes `state_label` an identity function, the first assertion
    /// still passes on the unmapped branch — so assert the *mapped* value is
    /// different from the token it came from.
    #[test]
    fn an_unmapped_token_degrades_to_prose_never_to_screaming_snake_case() {
        // Mapped: resolves through the module enum.
        let mapped = state_label("kebutuhan_bmn", "ANALISIS_KELAYAKAN");
        assert_eq!(mapped, "Analisis Kelayakan");

        // Unmapped module: no enum claims it, so it is title-cased.
        let unmapped = state_label("modul_yang_belum_ada", "SUBMIT_PUSAT");
        assert_eq!(unmapped, "Submit Pusat");

        // Whatever the path, a raw token must never survive to a user.
        for label in [mapped, unmapped, module_label("modul_baru")] {
            assert!(
                !label.contains('_') && label != label.to_uppercase(),
                "raw identifier reached the label: {label}"
            );
        }
    }

    #[test]
    fn role_slugs_are_named_too() {
        let n = WorkflowNotificationType::ApprovalRequired {
            entity_type: "penghapusan_bmn".into(),
            entity_id: Uuid::new_v4().to_string(),
            current_state: "SUBMIT_WILAYAH".into(),
            required_role: "validator_wilayah".into(),
            deadline: None,
        };
        assert_eq!(n.title(), "Persetujuan diperlukan: Penghapusan BMN");
        assert_eq!(n.body(), "Menunggu tindakan dari Validator Wilayah.");
    }
}
