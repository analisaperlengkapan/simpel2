use super::error::AppError;
use super::models::{SupportTicket, TicketComment};
use deadpool_postgres::Pool;
use lib_perlengkapan::contracts::{
    NotificationChannel, NotificationMessage, NotificationPriority, NotificationSender,
};
use std::sync::Arc;
use uuid::Uuid;

/// Service that owns the lifecycle of bantuan support tickets and their
/// comments. Optionally fans out user-facing notifications (in-app) on key
/// events through the [`NotificationSender`] port — same plumbing the
/// workflow engine uses, so /notifikasi center surfaces them uniformly.
pub struct TicketService {
    pub pool: Pool,
    notifier: Option<Arc<dyn NotificationSender>>,
}

impl TicketService {
    pub fn new(pool: Pool) -> Self {
        Self {
            pool,
            notifier: None,
        }
    }

    /// Inject the notification sender. Without this, the service stays a pure
    /// CRUD repository (matches its prior behaviour for tests/dev).
    pub fn with_notification_sender(mut self, notifier: Arc<dyn NotificationSender>) -> Self {
        self.notifier = Some(notifier);
        self
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
        if let Ok(ticket) = self.get_ticket(ticket_id).await {
            if ticket.user_id != user_id {
                self.notify_event(
                    "tiket.komentar",
                    ticket.user_id,
                    &ticket,
                    format!("Komentar baru pada tiket #{}", ticket.id),
                    format!("Tiket \"{}\" mendapat komentar baru.", ticket.subject),
                    NotificationPriority::Medium,
                );
            }
        }

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
