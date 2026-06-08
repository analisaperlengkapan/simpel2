use super::error::AppError;
use super::models::{SupportTicket, TicketComment};
use crate::contracts::{
    AuditSink, NotificationChannel, NotificationMessage, NotificationPriority, NotificationSender,
};
use deadpool_postgres::Pool;
use lib_perlengkapan::audit::{AuditAction, AuditEvent};
use std::sync::Arc;
use uuid::Uuid;

/// Service that owns the lifecycle of bantuan support tickets and their
/// comments. Optionally fans out user-facing notifications (in-app) on key
/// events through the [`NotificationSender`] port — same plumbing the
/// workflow engine uses, so /notifikasi center surfaces them uniformly.
/// Also optionally writes audit rows through [`AuditSink`] so the cross-
/// module `perlengkapan.audit_log` table captures who-did-what-when on
/// every ticket lifecycle event.
pub struct TicketService {
    pub pool: Pool,
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

    /// Inject the notification sender. Without this, the service stays a pure
    /// CRUD repository (matches its prior behaviour for tests/dev).
    pub fn with_notification_sender(mut self, notifier: Arc<dyn NotificationSender>) -> Self {
        self.notifier = Some(notifier);
        self
    }

    /// Inject the audit sink. Without this, ticket lifecycle events are not
    /// written to `perlengkapan.audit_log` (useful for tests / dev).
    pub fn with_audit_sink(mut self, audit: Arc<dyn AuditSink>) -> Self {
        self.audit = Some(audit);
        self
    }

    /// Best-effort audit emission. Failures are logged inside the sink and
    /// never propagate — losing one row must not abort a ticket transition.
    fn audit_event(
        &self,
        action: AuditAction,
        action_name: Option<&str>,
        actor: Uuid,
        ticket_id: Uuid,
        success: bool,
        message: Option<String>,
    ) {
        let audit = match self.audit.as_ref() {
            Some(a) => a.clone(),
            None => return,
        };
        let mut event = AuditEvent::new("bantuan", action, "tiket")
            .actor(actor, format!("user:{actor}"))
            .resource_id(ticket_id.to_string());
        if let Some(name) = action_name {
            event.action_name = Some(name.to_string());
        }
        event.success = success;
        if let Some(m) = message {
            event.message = Some(m);
        }
        tokio::spawn(async move {
            if let Err(e) = audit.log(event).await {
                tracing::warn!(error = %e, "bantuan ticket audit dispatch failed");
            }
        });
    }

    fn notify_event(
        &self,
        event: &str,
        recipient: Uuid,
        ticket: &SupportTicket,
        title: String,
        body: String,
        priority: NotificationPriority,
    ) {
        let notifier = match self.notifier.as_ref() {
            Some(n) => n.clone(),
            None => return,
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
        // Fire and forget — failures are logged inside NotifikasiService::send;
        // a ticket transition must never fail just because the notifier is
        // having a bad day.
        tokio::spawn(async move {
            if let Err(e) = notifier.send(msg).await {
                tracing::warn!(error = %e, "tiket notification dispatch failed");
            }
        });
    }

    // CRUD Ticket
    pub async fn create_ticket(
        &self,
        user_id: Uuid,
        subject: &str,
        description: Option<&str>,
        priority: Option<&str>,
        category_id: Option<Uuid>,
        spam_score: f32,
    ) -> Result<SupportTicket, AppError> {
        let client = self.pool.get().await?;
        let id = Uuid::new_v4();
        let row = client.query_one(
            r#"INSERT INTO bantuan.support_tickets (id, user_id, subject, description, priority, category_id, status, created_at, updated_at, spam_score)
            VALUES ($1, $2, $3, $4, $5, $6, 'open', NOW(), NOW(), $7) RETURNING *"#,
            &[&id, &user_id, &subject, &description, &priority, &category_id, &spam_score]
        ).await?;
        let ticket = SupportTicket::from(&row);

        // Notify the requester that their ticket has been received.
        self.notify_event(
            "tiket.dibuat",
            ticket.user_id,
            &ticket,
            format!("Tiket #{} dibuat", ticket.id),
            format!("Tiket bantuan Anda \"{}\" telah dibuat.", ticket.subject),
            NotificationPriority::Medium,
        );

        // Audit: a new bantuan ticket has been created.
        self.audit_event(
            AuditAction::Create,
            Some("tiket.create"),
            ticket.user_id,
            ticket.id,
            true,
            Some(format!("subject={}", ticket.subject)),
        );

        Ok(ticket)
    }
    pub async fn get_ticket(&self, id: Uuid) -> Result<SupportTicket, AppError> {
        let client = self.pool.get().await?;
        let row = client
            .query_one(
                r#"SELECT * FROM bantuan.support_tickets WHERE id = $1"#,
                &[&id],
            )
            .await?;
        Ok(SupportTicket::from(&row))
    }
    pub async fn update_ticket(
        &self,
        id: Uuid,
        subject: &str,
        description: Option<&str>,
        priority: Option<&str>,
        category_id: Option<Uuid>,
        status: &str,
    ) -> Result<SupportTicket, AppError> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            r#"UPDATE bantuan.support_tickets SET subject = $1, description = $2, priority = $3, category_id = $4, status = $5, updated_at = NOW() WHERE id = $6 RETURNING *"#,
            &[&subject, &description, &priority, &category_id, &status, &id]
        ).await?;
        let ticket = SupportTicket::from(&row);

        // Notify the requester that their ticket has been updated.
        self.notify_event(
            "tiket.diupdate",
            ticket.user_id,
            &ticket,
            format!("Tiket #{} diperbarui", ticket.id),
            format!(
                "Tiket bantuan \"{}\" diperbarui (status: {}).",
                ticket.subject, ticket.status
            ),
            NotificationPriority::Low,
        );

        // Audit: ticket fields / status changed.
        self.audit_event(
            AuditAction::Update,
            Some("tiket.update"),
            ticket.user_id,
            ticket.id,
            true,
            Some(format!("status={}", ticket.status)),
        );

        Ok(ticket)
    }
    pub async fn delete_ticket(&self, id: Uuid) -> Result<(), AppError> {
        let client = self.pool.get().await?;
        client
            .execute(
                r#"DELETE FROM bantuan.support_tickets WHERE id = $1"#,
                &[&id],
            )
            .await?;
        // Audit: a ticket has been hard-deleted. Actor isn't visible at this
        // call site so we record `Uuid::nil()` as the actor — the handler
        // layer is the right place to plug in the real claims.user_id once
        // bantuan routes get mounted into the unified app.
        self.audit_event(
            AuditAction::Delete,
            Some("tiket.delete"),
            Uuid::nil(),
            id,
            true,
            None,
        );
        Ok(())
    }
    pub async fn list_tickets(
        &self,
        user_id: Option<Uuid>,
        status: Option<&str>,
    ) -> Result<Vec<SupportTicket>, AppError> {
        let client = self.pool.get().await?;
        let rows = if let Some(uid) = user_id {
            client.query(
                r#"SELECT * FROM bantuan.support_tickets WHERE user_id = $1 AND ($2::text IS NULL OR status = $2) ORDER BY created_at DESC"#,
                &[&uid, &status]
            ).await?
        } else {
            client.query(
                r#"SELECT * FROM bantuan.support_tickets WHERE ($1::text IS NULL OR status = $1) ORDER BY created_at DESC"#,
                &[&status]
            ).await?
        };
        Ok(rows
            .into_iter()
            .map(|row| SupportTicket::from(&row))
            .collect())
    }
    // Komentar Ticket
    pub async fn add_comment(
        &self,
        ticket_id: Uuid,
        user_id: Uuid,
        content: &str,
    ) -> Result<TicketComment, AppError> {
        let client = self.pool.get().await?;
        let id = Uuid::new_v4();
        let row = client
            .query_one(
                r#"INSERT INTO bantuan.ticket_comments (id, ticket_id, user_id, content, created_at)
            VALUES ($1, $2, $3, $4, NOW()) RETURNING *"#,
                &[&id, &ticket_id, &user_id, &content],
            )
            .await?;
        let comment = TicketComment::from(&row);

        // Notify the ticket owner (if different from the commenter) that the
        // ticket has a new comment.
        if let Ok(ticket) = self.get_ticket(ticket_id).await
            && ticket.user_id != user_id
        {
            self.notify_event(
                "tiket.komentar",
                ticket.user_id,
                &ticket,
                format!("Komentar baru pada tiket #{}", ticket.id),
                format!("Tiket \"{}\" mendapat komentar baru.", ticket.subject),
                NotificationPriority::Medium,
            );
        }

        // Audit: a new comment was posted on this ticket by `user_id`.
        self.audit_event(
            AuditAction::Custom,
            Some("tiket.comment.create"),
            user_id,
            ticket_id,
            true,
            None,
        );

        Ok(comment)
    }
    pub async fn list_comments(&self, ticket_id: Uuid) -> Result<Vec<TicketComment>, AppError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            r#"SELECT * FROM bantuan.ticket_comments WHERE ticket_id = $1 ORDER BY created_at ASC"#,
            &[&ticket_id]
        ).await?;
        Ok(rows
            .into_iter()
            .map(|row| TicketComment::from(&row))
            .collect())
    }
    // Update status
    pub async fn update_status(&self, id: Uuid, status: &str) -> Result<SupportTicket, AppError> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            r#"UPDATE bantuan.support_tickets SET status = $1, updated_at = NOW(), closed_at = CASE WHEN $1 = 'closed' THEN NOW() ELSE closed_at END WHERE id = $2 RETURNING *"#,
            &[&status, &id]
        ).await?;
        let ticket = SupportTicket::from(&row);

        // Surface terminal status transitions to the requester. "resolved" and
        // "closed" map to the same notification; intermediate moves (e.g.
        // "in_progress") still fire the generic diupdate event via
        // update_ticket above.
        let event = match status {
            "resolved" => Some(("tiket.resolved", "diselesaikan")),
            "closed" => Some(("tiket.closed", "ditutup")),
            _ => None,
        };
        if let Some((event_name, label)) = event {
            self.notify_event(
                event_name,
                ticket.user_id,
                &ticket,
                format!("Tiket #{} {}", ticket.id, label),
                format!("Tiket bantuan \"{}\" telah {}.", ticket.subject, label),
                NotificationPriority::High,
            );
        }

        // Audit: terminal status transitions (resolved / closed) earn their
        // own audit row; intermediate moves are covered by `update_ticket`.
        let action = match status {
            "resolved" => Some((AuditAction::Approve, "tiket.resolved")),
            "closed" => Some((AuditAction::Cancel, "tiket.closed")),
            "open" | "in_progress" | "pending" => {
                Some((AuditAction::Update, "tiket.status_change"))
            }
            _ => None,
        };
        if let Some((kind, name)) = action {
            self.audit_event(
                kind,
                Some(name),
                ticket.user_id,
                ticket.id,
                true,
                Some(format!("status={status}")),
            );
        }

        Ok(ticket)
    }
    // Pencarian tiket
    pub async fn search_tickets(
        &self,
        query: &str,
        max_results: u32,
    ) -> Result<Vec<SupportTicket>, AppError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            r#"SELECT * FROM bantuan.support_tickets WHERE (subject ILIKE $1 OR description ILIKE $1) ORDER BY created_at DESC LIMIT $2"#,
            &[&format!("%{}%", query), &(max_results as i64)]
        ).await?;
        Ok(rows
            .into_iter()
            .map(|row| SupportTicket::from(&row))
            .collect())
    }
}
