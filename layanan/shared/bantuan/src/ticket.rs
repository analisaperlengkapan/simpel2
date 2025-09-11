use crate::models::{SupportTicket, TicketComment};
use crate::error::AppError;
use deadpool_postgres::Pool;
use uuid::Uuid;

pub struct TicketService {
    pub pool: Pool,
}

impl TicketService {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    // CRUD Ticket
    pub async fn create_ticket(&self, user_id: Uuid, subject: &str, description: Option<&str>, priority: Option<&str>, category_id: Option<Uuid>, spam_score: f32) -> Result<SupportTicket, AppError> {
        let client = self.pool.get().await?;
        let id = Uuid::new_v4();
        let row = client.query_one(
            r#"INSERT INTO bantuan.support_tickets (id, user_id, subject, description, priority, category_id, status, created_at, updated_at, spam_score)
            VALUES ($1, $2, $3, $4, $5, $6, 'open', NOW(), NOW(), $7) RETURNING *"#,
            &[&id, &user_id, &subject, &description, &priority, &category_id, &spam_score]
        ).await?;
        Ok(SupportTicket::from(&row))
    }
    pub async fn get_ticket(&self, id: Uuid) -> Result<SupportTicket, AppError> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            r#"SELECT * FROM bantuan.support_tickets WHERE id = $1"#,
            &[&id]
        ).await?;
        Ok(SupportTicket::from(&row))
    }
    pub async fn update_ticket(&self, id: Uuid, subject: &str, description: Option<&str>, priority: Option<&str>, category_id: Option<Uuid>, status: &str) -> Result<SupportTicket, AppError> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            r#"UPDATE bantuan.support_tickets SET subject = $1, description = $2, priority = $3, category_id = $4, status = $5, updated_at = NOW() WHERE id = $6 RETURNING *"#,
            &[&subject, &description, &priority, &category_id, &status, &id]
        ).await?;
        Ok(SupportTicket::from(&row))
    }
    pub async fn delete_ticket(&self, id: Uuid) -> Result<(), AppError> {
        let client = self.pool.get().await?;
        client.execute(
            r#"DELETE FROM bantuan.support_tickets WHERE id = $1"#,
            &[&id]
        ).await?;
        Ok(())
    }
    pub async fn list_tickets(&self, user_id: Option<Uuid>, status: Option<&str>) -> Result<Vec<SupportTicket>, AppError> {
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
        Ok(rows.into_iter().map(|row| SupportTicket::from(&row)).collect())
    }
    // Komentar Ticket
    pub async fn add_comment(&self, ticket_id: Uuid, user_id: Uuid, content: &str) -> Result<TicketComment, AppError> {
        let client = self.pool.get().await?;
        let id = Uuid::new_v4();
        let row = client.query_one(
            r#"INSERT INTO bantuan.ticket_comments (id, ticket_id, user_id, content, created_at)
            VALUES ($1, $2, $3, $4, NOW()) RETURNING *"#,
            &[&id, &ticket_id, &user_id, &content]
        ).await?;
        Ok(TicketComment::from(&row))
    }
    pub async fn list_comments(&self, ticket_id: Uuid) -> Result<Vec<TicketComment>, AppError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            r#"SELECT * FROM bantuan.ticket_comments WHERE ticket_id = $1 ORDER BY created_at ASC"#,
            &[&ticket_id]
        ).await?;
        Ok(rows.into_iter().map(|row| TicketComment::from(&row)).collect())
    }
    // Update status
    pub async fn update_status(&self, id: Uuid, status: &str) -> Result<SupportTicket, AppError> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            r#"UPDATE bantuan.support_tickets SET status = $1, updated_at = NOW(), closed_at = CASE WHEN $1 = 'closed' THEN NOW() ELSE closed_at END WHERE id = $2 RETURNING *"#,
            &[&status, &id]
        ).await?;
        Ok(SupportTicket::from(&row))
    }
    // Pencarian tiket
    pub async fn search_tickets(&self, query: &str, max_results: u32) -> Result<Vec<SupportTicket>, AppError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            r#"SELECT * FROM bantuan.support_tickets WHERE (subject ILIKE $1 OR description ILIKE $1) ORDER BY created_at DESC LIMIT $2"#,
            &[&format!("%{}%", query), &(max_results as i64)]
        ).await?;
        Ok(rows.into_iter().map(|row| SupportTicket::from(&row)).collect())
    }
}
