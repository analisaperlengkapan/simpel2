//! Helpdesk ticket service.
//!
//! Owns the lifecycle of `bantuan.support_tickets` + `bantuan.ticket_comments`
//! and fans out to the notification/audit ports so a ticket behaves like every
//! other tracked object in the service.
//!
//! # Authorization model
//!
//! Identity is **never** taken from client input. Callers pass a [`TicketActor`]
//! built from JWT claims by the handler layer; the reporter is
//! `actor.user_id`, and reads are constrained by [`TicketAccess`]:
//!
//! * `TicketAccess::Own` — ordinary users see only tickets they filed.
//! * `TicketAccess::All` — helpdesk staff (admin roles) see every ticket.
//!
//! The earlier draft of this module read `user_id` from the request payload and
//! query string, which let a caller file a ticket *as* someone else and read
//! anyone's tickets by id. Routing every access through `TicketActor` is what
//! keeps that from coming back.

use super::models::{SupportTicket, TicketComment};
use crate::contracts::{
    AuditSink, NotificationChannel, NotificationMessage, NotificationPriority, NotificationSender,
};
use crate::shared::error::{AppError, AppResult};
use crate::shared::middleware::Claims;
use deadpool_postgres::Pool;
use lib_perlengkapan::audit::{AuditAction, AuditEvent};
use std::sync::Arc;
use uuid::Uuid;

/// Ticket statuses, in lifecycle order. Mirrored by the
/// `support_tickets_status_valid` CHECK in V004.
pub const STATUS_OPEN: &str = "open";
pub const STATUS_IN_PROGRESS: &str = "in_progress";
pub const STATUS_RESOLVED: &str = "resolved";
pub const STATUS_CLOSED: &str = "closed";

/// Priorities. Mirrored by the `support_tickets_priority_valid` CHECK in V004.
pub const PRIORITIES: [&str; 4] = ["low", "normal", "high", "urgent"];

/// Statuses that count as terminal — the two that carry a `closed_at`.
fn is_terminal(status: &str) -> bool {
    matches!(status, STATUS_RESOLVED | STATUS_CLOSED)
}

/// Legal status transitions.
///
/// Encoded here rather than left to callers so an invalid move is a 400 instead
/// of a silently-wrong row. A resolved ticket may reopen (the reporter says it
/// isn't actually fixed); a closed one is final.
pub fn can_transition_to(from: &str, to: &str) -> bool {
    match from {
        STATUS_OPEN => matches!(to, STATUS_IN_PROGRESS | STATUS_RESOLVED | STATUS_CLOSED),
        STATUS_IN_PROGRESS => matches!(to, STATUS_RESOLVED | STATUS_CLOSED | STATUS_OPEN),
        STATUS_RESOLVED => matches!(to, STATUS_CLOSED | STATUS_OPEN),
        STATUS_CLOSED => false,
        _ => false,
    }
}

/// Who is acting, derived from JWT claims — never from the request body.
#[derive(Debug, Clone)]
pub struct TicketActor {
    pub user_id: Uuid,
    pub satker_code: Option<String>,
    pub is_staff: bool,
}

impl TicketActor {
    /// Build from verified claims. `is_staff` (helpdesk) maps to the admin
    /// roles — the same set [`Claims::require_admin`] accepts.
    pub fn from_claims(claims: &Claims) -> Self {
        Self {
            user_id: claims.user_id,
            satker_code: claims.satker_code.clone(),
            is_staff: matches!(claims.role.as_str(), "admin" | "admin_pusat" | "superadmin"),
        }
    }

    /// What this actor is allowed to read.
    pub fn access(&self) -> TicketAccess {
        if self.is_staff {
            TicketAccess::All
        } else {
            TicketAccess::Own(self.user_id)
        }
    }

    /// Assert this actor is helpdesk staff. Used for the triage surface
    /// (moving status).
    pub fn require_staff(&self) -> AppResult<()> {
        if self.is_staff {
            Ok(())
        } else {
            Err(AppError::Authorization(
                "Akses ditolak: aksi ini memerlukan role admin (helpdesk)".to_string(),
            ))
        }
    }
}

/// Read scope for ticket queries.
#[derive(Debug, Clone, Copy)]
pub enum TicketAccess {
    /// Only tickets filed by this user.
    Own(Uuid),
    /// Every ticket — helpdesk staff.
    All,
}

/// Service owning ticket + comment lifecycle.
pub struct TicketService {
    pool: Pool,
    notifier: Option<Arc<dyn NotificationSender>>,
    audit: Option<Arc<dyn AuditSink>>,
}

impl TicketService {
    pub fn new(pool: Pool) -> Self {
        Self {
            pool,
            notifier: None,
            audit: None,
        }
    }

    /// Inject the notification sender. Without it the service is a pure CRUD
    /// repository.
    pub fn with_notification_sender(mut self, notifier: Arc<dyn NotificationSender>) -> Self {
        self.notifier = Some(notifier);
        self
    }

    /// Inject the audit sink. Without it ticket events are not written to
    /// `perlengkapan.audit_log`.
    pub fn with_audit_sink(mut self, audit: Arc<dyn AuditSink>) -> Self {
        self.audit = Some(audit);
        self
    }

    /// Best-effort audit emission — losing one row must not abort a ticket
    /// transition, so failures are logged inside the sink and never propagate.
    fn audit_event(
        &self,
        action: AuditAction,
        action_name: &str,
        actor: Uuid,
        ticket_id: Uuid,
        message: Option<String>,
    ) {
        let Some(audit) = self.audit.as_ref().cloned() else {
            return;
        };
        let mut event = AuditEvent::new("bantuan", action, "tiket")
            .actor(actor, format!("user:{actor}"))
            .resource_id(ticket_id.to_string());
        event.action_name = Some(action_name.to_string());
        event.success = true;
        event.message = message;
        tokio::spawn(async move {
            if let Err(e) = audit.log(event).await {
                tracing::warn!(error = %e, "bantuan ticket audit dispatch failed");
            }
        });
    }

    /// Fire-and-forget notification — a ticket transition must never fail
    /// because the notifier is having a bad day.
    fn notify(
        &self,
        event: &str,
        recipient: Uuid,
        ticket: &SupportTicket,
        title: String,
        body: String,
        priority: NotificationPriority,
    ) {
        let Some(notifier) = self.notifier.as_ref().cloned() else {
            return;
        };
        let msg = NotificationMessage {
            event: event.to_string(),
            recipient_user_id: recipient,
            channels: vec![NotificationChannel::InApp],
            priority,
            title,
            body,
            variables: serde_json::to_value(ticket).ok(),
            deeplink: Some(format!("/bantuan/tiket/{}", ticket.id)),
            recipient_email: None,
        };
        tokio::spawn(async move {
            if let Err(e) = notifier.send(msg).await {
                tracing::warn!(error = %e, "tiket notification dispatch failed");
            }
        });
    }

    /// File a new ticket. The reporter is `actor` — the caller cannot name
    /// someone else.
    pub async fn create_ticket(
        &self,
        actor: &TicketActor,
        subject: &str,
        description: Option<&str>,
        priority: Option<&str>,
    ) -> AppResult<SupportTicket> {
        let subject = subject.trim();
        if subject.is_empty() {
            return Err(AppError::BadRequest("Subjek wajib diisi".to_string()));
        }
        if subject.chars().count() > 200 {
            return Err(AppError::BadRequest(
                "Subjek maksimal 200 karakter".to_string(),
            ));
        }
        let description = description.map(str::trim).filter(|d| !d.is_empty());
        if description.is_none() {
            return Err(AppError::BadRequest("Pesan wajib diisi".to_string()));
        }
        let priority = priority.unwrap_or("normal");
        if !PRIORITIES.contains(&priority) {
            return Err(AppError::BadRequest(format!(
                "Prioritas '{priority}' tidak dikenal (gunakan: {})",
                PRIORITIES.join(", ")
            )));
        }

        let client = self.pool.get().await?;
        let id = Uuid::new_v4();
        let row = client
            .query_one(
                r#"INSERT INTO bantuan.support_tickets
                     (id, user_id, satker_code, subject, description, priority, status)
                   VALUES ($1, $2, $3, $4, $5, $6, 'open')
                   RETURNING *"#,
                &[
                    &id,
                    &actor.user_id,
                    &actor.satker_code,
                    &subject,
                    &description,
                    &priority,
                ],
            )
            .await?;
        let ticket = SupportTicket::from(&row);

        self.notify(
            "tiket.dibuat",
            ticket.user_id,
            &ticket,
            "Tiket bantuan diterima".to_string(),
            format!("Tiket bantuan Anda \"{}\" telah dibuat.", ticket.subject),
            NotificationPriority::Medium,
        );
        self.audit_event(
            AuditAction::Create,
            "tiket.create",
            ticket.user_id,
            ticket.id,
            Some(format!("subject={}", ticket.subject)),
        );

        Ok(ticket)
    }

    /// Fetch one ticket, enforcing the actor's read scope.
    ///
    /// A ticket the actor may not read reports `NotFound`, not `Authorization`:
    /// a 403 on a real id would confirm that id exists to someone with no
    /// business knowing it does.
    pub async fn get_ticket(&self, actor: &TicketActor, id: Uuid) -> AppResult<SupportTicket> {
        let client = self.pool.get().await?;
        let row = match actor.access() {
            TicketAccess::All => {
                client
                    .query_opt(
                        r#"SELECT * FROM bantuan.support_tickets WHERE id = $1"#,
                        &[&id],
                    )
                    .await?
            }
            TicketAccess::Own(uid) => {
                client
                    .query_opt(
                        r#"SELECT * FROM bantuan.support_tickets
                           WHERE id = $1 AND user_id = $2"#,
                        &[&id, &uid],
                    )
                    .await?
            }
        };
        row.as_ref()
            .map(SupportTicket::from)
            .ok_or_else(|| AppError::NotFound(format!("Tiket {id} tidak ditemukan")))
    }

    /// List tickets visible to the actor, newest first.
    pub async fn list_tickets(
        &self,
        actor: &TicketActor,
        status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> AppResult<Vec<SupportTicket>> {
        let limit = limit.clamp(1, 200);
        let offset = offset.max(0);
        let client = self.pool.get().await?;
        let rows = match actor.access() {
            TicketAccess::All => {
                client
                    .query(
                        r#"SELECT * FROM bantuan.support_tickets
                           WHERE ($1::text IS NULL OR status = $1)
                           ORDER BY created_at DESC
                           LIMIT $2 OFFSET $3"#,
                        &[&status, &limit, &offset],
                    )
                    .await?
            }
            TicketAccess::Own(uid) => {
                client
                    .query(
                        r#"SELECT * FROM bantuan.support_tickets
                           WHERE user_id = $1
                             AND ($2::text IS NULL OR status = $2)
                           ORDER BY created_at DESC
                           LIMIT $3 OFFSET $4"#,
                        &[&uid, &status, &limit, &offset],
                    )
                    .await?
            }
        };
        Ok(rows.iter().map(SupportTicket::from).collect())
    }

    /// Move a ticket's status. Helpdesk staff only, and only along a legal edge.
    pub async fn update_status(
        &self,
        actor: &TicketActor,
        id: Uuid,
        status: &str,
    ) -> AppResult<SupportTicket> {
        actor.require_staff()?;
        let current = self.get_ticket(actor, id).await?;
        if !can_transition_to(&current.status, status) {
            return Err(AppError::BadRequest(format!(
                "Transisi status '{}' → '{}' tidak diizinkan",
                current.status, status
            )));
        }

        let client = self.pool.get().await?;
        // closed_at is kept in lock-step with the terminal statuses so the
        // support_tickets_closed_at_matches_status CHECK always holds.
        let row = client
            .query_one(
                r#"UPDATE bantuan.support_tickets
                   SET status = $1,
                       updated_at = now(),
                       closed_at = CASE WHEN $2 THEN now() ELSE NULL END
                   WHERE id = $3
                   RETURNING *"#,
                &[&status, &is_terminal(status), &id],
            )
            .await?;
        let ticket = SupportTicket::from(&row);

        let label = match status {
            STATUS_IN_PROGRESS => "sedang ditangani",
            STATUS_RESOLVED => "diselesaikan",
            STATUS_CLOSED => "ditutup",
            _ => "dibuka kembali",
        };
        self.notify(
            &format!("tiket.{status}"),
            ticket.user_id,
            &ticket,
            format!("Tiket bantuan {label}"),
            format!("Tiket bantuan \"{}\" telah {}.", ticket.subject, label),
            if is_terminal(status) {
                NotificationPriority::High
            } else {
                NotificationPriority::Medium
            },
        );
        self.audit_event(
            match status {
                STATUS_RESOLVED => AuditAction::Approve,
                STATUS_CLOSED => AuditAction::Cancel,
                _ => AuditAction::Update,
            },
            "tiket.status_change",
            actor.user_id,
            ticket.id,
            Some(format!("{} -> {}", current.status, status)),
        );

        Ok(ticket)
    }

    /// Post a comment. The author is `actor`; visibility follows `get_ticket`,
    /// so a user cannot comment on a ticket they cannot read.
    pub async fn add_comment(
        &self,
        actor: &TicketActor,
        ticket_id: Uuid,
        content: &str,
    ) -> AppResult<TicketComment> {
        let content = content.trim();
        if content.is_empty() {
            return Err(AppError::BadRequest(
                "Komentar tidak boleh kosong".to_string(),
            ));
        }
        let ticket = self.get_ticket(actor, ticket_id).await?;
        if ticket.status == STATUS_CLOSED {
            return Err(AppError::BadRequest(
                "Tiket sudah ditutup — tidak dapat dikomentari".to_string(),
            ));
        }

        let client = self.pool.get().await?;
        let id = Uuid::new_v4();
        let row = client
            .query_one(
                r#"INSERT INTO bantuan.ticket_comments (id, ticket_id, user_id, content)
                   VALUES ($1, $2, $3, $4)
                   RETURNING *"#,
                &[&id, &ticket_id, &actor.user_id, &content],
            )
            .await?;
        let comment = TicketComment::from(&row);

        // Tell the reporter when someone *else* (i.e. helpdesk) replies.
        if ticket.user_id != actor.user_id {
            self.notify(
                "tiket.komentar",
                ticket.user_id,
                &ticket,
                "Balasan baru pada tiket bantuan".to_string(),
                format!("Tiket \"{}\" mendapat balasan baru.", ticket.subject),
                NotificationPriority::Medium,
            );
        }
        self.audit_event(
            AuditAction::Custom,
            "tiket.comment.create",
            actor.user_id,
            ticket_id,
            None,
        );

        Ok(comment)
    }

    /// List a ticket's comments oldest-first. Gated by `get_ticket`, so the
    /// read scope is enforced before any comment row is returned.
    pub async fn list_comments(
        &self,
        actor: &TicketActor,
        ticket_id: Uuid,
    ) -> AppResult<Vec<TicketComment>> {
        self.get_ticket(actor, ticket_id).await?;
        let client = self.pool.get().await?;
        let rows = client
            .query(
                r#"SELECT * FROM bantuan.ticket_comments
                   WHERE ticket_id = $1
                   ORDER BY created_at ASC"#,
                &[&ticket_id],
            )
            .await?;
        Ok(rows.iter().map(TicketComment::from).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_ticket_can_be_picked_up_resolved_or_closed() {
        assert!(can_transition_to(STATUS_OPEN, STATUS_IN_PROGRESS));
        assert!(can_transition_to(STATUS_OPEN, STATUS_RESOLVED));
        assert!(can_transition_to(STATUS_OPEN, STATUS_CLOSED));
    }

    #[test]
    fn resolved_ticket_can_reopen_but_closed_is_final() {
        assert!(can_transition_to(STATUS_RESOLVED, STATUS_OPEN));
        assert!(can_transition_to(STATUS_RESOLVED, STATUS_CLOSED));
        // Closed is terminal — nothing leaves it.
        assert!(!can_transition_to(STATUS_CLOSED, STATUS_OPEN));
        assert!(!can_transition_to(STATUS_CLOSED, STATUS_IN_PROGRESS));
        assert!(!can_transition_to(STATUS_CLOSED, STATUS_RESOLVED));
    }

    #[test]
    fn self_transition_and_unknown_status_are_rejected() {
        assert!(!can_transition_to(STATUS_OPEN, STATUS_OPEN));
        assert!(!can_transition_to(STATUS_OPEN, "bogus"));
        assert!(!can_transition_to("bogus", STATUS_OPEN));
    }

    #[test]
    fn terminal_statuses_are_exactly_the_ones_carrying_closed_at() {
        assert!(is_terminal(STATUS_RESOLVED));
        assert!(is_terminal(STATUS_CLOSED));
        assert!(!is_terminal(STATUS_OPEN));
        assert!(!is_terminal(STATUS_IN_PROGRESS));
    }

    #[test]
    fn staff_read_everything_and_users_only_their_own() {
        let uid = Uuid::new_v4();
        let user = TicketActor {
            user_id: uid,
            satker_code: Some("0200010".to_string()),
            is_staff: false,
        };
        let staff = TicketActor {
            user_id: Uuid::new_v4(),
            satker_code: None,
            is_staff: true,
        };
        assert!(matches!(user.access(), TicketAccess::Own(u) if u == uid));
        assert!(matches!(staff.access(), TicketAccess::All));
        assert!(user.require_staff().is_err());
        assert!(staff.require_staff().is_ok());
    }
}
