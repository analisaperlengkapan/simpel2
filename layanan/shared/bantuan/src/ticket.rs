use crate::models::{SupportTicket, TicketComment};
use crate::error::AppError;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;

pub struct TicketService {
    pub pool: PgPool,
}

impl TicketService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // CRUD Ticket
    pub async fn create_ticket(&self, user_id: Uuid, subject: &str, description: Option<&str>, priority: Option<&str>, category_id: Option<Uuid>, spam_score: f32) -> Result<SupportTicket, AppError> {
        let ticket = sqlx::query_as!(SupportTicket,
            r#"INSERT INTO bantuan.support_tickets (id, user_id, subject, description, priority, category_id, status, created_at, updated_at, spam_score)
            VALUES ($1, $2, $3, $4, $5, $6, 'open', NOW(), NOW(), $7) RETURNING *"#,
            Uuid::new_v4(), user_id, subject, description, priority, category_id, spam_score
        ).fetch_one(&self.pool).await?;
        Ok(ticket)
    }
    pub async fn get_ticket(&self, id: Uuid) -> Result<SupportTicket, AppError> {
        let ticket = sqlx::query_as!(SupportTicket,
            r#"SELECT * FROM bantuan.support_tickets WHERE id = $1"#,
            id
        ).fetch_one(&self.pool).await?;
        Ok(ticket)
    }
    pub async fn update_ticket(&self, id: Uuid, subject: &str, description: Option<&str>, priority: Option<&str>, category_id: Option<Uuid>, status: &str) -> Result<SupportTicket, AppError> {
        let ticket = sqlx::query_as!(SupportTicket,
            r#"UPDATE bantuan.support_tickets SET subject = $1, description = $2, priority = $3, category_id = $4, status = $5, updated_at = NOW() WHERE id = $6 RETURNING *"#,
            subject, description, priority, category_id, status, id
        ).fetch_one(&self.pool).await?;
        Ok(ticket)
    }
    pub async fn delete_ticket(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query!(
            r#"DELETE FROM bantuan.support_tickets WHERE id = $1"#,
            id
        ).execute(&self.pool).await?;
        Ok(())
    }
    pub async fn list_tickets(&self, user_id: Option<Uuid>, status: Option<&str>) -> Result<Vec<SupportTicket>, AppError> {
        let tickets = if let Some(uid) = user_id {
            sqlx::query_as!(SupportTicket,
                r#"SELECT * FROM bantuan.support_tickets WHERE user_id = $1 AND ($2::text IS NULL OR status = $2) ORDER BY created_at DESC"#,
                uid, status
            ).fetch_all(&self.pool).await?
        } else {
            sqlx::query_as!(SupportTicket,
                r#"SELECT * FROM bantuan.support_tickets WHERE ($1::text IS NULL OR status = $1) ORDER BY created_at DESC"#,
                status
            ).fetch_all(&self.pool).await?
        };
        Ok(tickets)
    }
    // Komentar Ticket
    pub async fn add_comment(&self, ticket_id: Uuid, user_id: Uuid, content: &str) -> Result<TicketComment, AppError> {
        let comment = sqlx::query_as!(TicketComment,
            r#"INSERT INTO bantuan.ticket_comments (id, ticket_id, user_id, content, created_at)
            VALUES ($1, $2, $3, $4, NOW()) RETURNING *"#,
            Uuid::new_v4(), ticket_id, user_id, content
        ).fetch_one(&self.pool).await?;
        Ok(comment)
    }
    pub async fn list_comments(&self, ticket_id: Uuid) -> Result<Vec<TicketComment>, AppError> {
        let comments = sqlx::query_as!(TicketComment,
            r#"SELECT * FROM bantuan.ticket_comments WHERE ticket_id = $1 ORDER BY created_at ASC"#,
            ticket_id
        ).fetch_all(&self.pool).await?;
        Ok(comments)
    }
    // Update status
    pub async fn update_status(&self, id: Uuid, status: &str) -> Result<SupportTicket, AppError> {
        let ticket = sqlx::query_as!(SupportTicket,
            r#"UPDATE bantuan.support_tickets SET status = $1, updated_at = NOW(), closed_at = CASE WHEN $1 = 'closed' THEN NOW() ELSE closed_at END WHERE id = $2 RETURNING *"#,
            status, id
        ).fetch_one(&self.pool).await?;
        Ok(ticket)
    }
    // Pencarian tiket
    pub async fn search_tickets(&self, query: &str, max_results: u32) -> Result<Vec<SupportTicket>, AppError> {
        let tickets = sqlx::query_as!(SupportTicket,
            r#"SELECT * FROM bantuan.support_tickets WHERE (subject ILIKE $1 OR description ILIKE $1) ORDER BY created_at DESC LIMIT $2"#,
            format!('%{}%', query), max_results as i64
        ).fetch_all(&self.pool).await?;
        Ok(tickets)
    }
} 