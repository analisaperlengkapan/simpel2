use crate::models::{ExportImportLog, KnowledgeArticle, FaqArticle, SupportTicket};
use crate::error::AppError;
use sqlx::PgPool;
use uuid::Uuid;
use serde_json::Value;
use chrono::Utc;

pub struct ExportImportService {
    pub pool: PgPool,
}

impl ExportImportService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn export_resource(&self, resource: &str) -> Result<Value, AppError> {
        let data = match resource {
            "knowledge_articles" => {
                let rows = sqlx::query_as!(KnowledgeArticle,
                    r#"SELECT * FROM bantuan.knowledge_articles ORDER BY created_at ASC"#
                ).fetch_all(&self.pool).await?;
                serde_json::to_value(rows).map_err(|e| AppError::Internal)?
            },
            "faq_articles" => {
                let rows = sqlx::query_as!(FaqArticle,
                    r#"SELECT * FROM bantuan.faq_articles ORDER BY created_at ASC"#
                ).fetch_all(&self.pool).await?;
                serde_json::to_value(rows).map_err(|e| AppError::Internal)?
            },
            "support_tickets" => {
                let rows = sqlx::query_as!(SupportTicket,
                    r#"SELECT * FROM bantuan.support_tickets ORDER BY created_at ASC"#
                ).fetch_all(&self.pool).await?;
                serde_json::to_value(rows).map_err(|e| AppError::Internal)?
            },
            _ => return Err(AppError::BadRequest("Resource tidak didukung".to_string())),
        };
        Ok(data)
    }
    pub async fn import_resource(&self, resource: &str, data: Value) -> Result<(), AppError> {
        match resource {
            "knowledge_articles" => {
                let articles: Vec<KnowledgeArticle> = serde_json::from_value(data).map_err(|e| AppError::BadRequest(e.to_string()))?;
                for art in articles {
                    let _ = sqlx::query!(
                        r#"INSERT INTO bantuan.knowledge_articles (id, title, content, category_id, tags, created_at, updated_at, is_active)
                        VALUES ($1, $2, $3, $4, $5, $6, $7, $8) ON CONFLICT (id) DO NOTHING"#,
                        art.id, art.title, art.content, art.category_id, &art.tags.unwrap_or_default(), art.created_at, art.updated_at, art.is_active
                    ).execute(&self.pool).await;
                }
            },
            "faq_articles" => {
                let articles: Vec<FaqArticle> = serde_json::from_value(data).map_err(|e| AppError::BadRequest(e.to_string()))?;
                for art in articles {
                    let _ = sqlx::query!(
                        r#"INSERT INTO bantuan.faq_articles (id, category_id, title, content, tags, version, created_at, updated_at, is_active)
                        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) ON CONFLICT (id) DO NOTHING"#,
                        art.id, art.category_id, art.title, art.content, &art.tags.unwrap_or_default(), art.version, art.created_at, art.updated_at, art.is_active
                    ).execute(&self.pool).await;
                }
            },
            "support_tickets" => {
                let tickets: Vec<SupportTicket> = serde_json::from_value(data).map_err(|e| AppError::BadRequest(e.to_string()))?;
                for t in tickets {
                    let _ = sqlx::query!(
                        r#"INSERT INTO bantuan.support_tickets (id, user_id, subject, description, priority, category_id, status, created_at, updated_at, closed_at, spam_score)
                        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) ON CONFLICT (id) DO NOTHING"#,
                        t.id, t.user_id, t.subject, t.description, t.priority, t.category_id, t.status, t.created_at, t.updated_at, t.closed_at, t.spam_score
                    ).execute(&self.pool).await;
                }
            },
            _ => return Err(AppError::BadRequest("Resource tidak didukung".to_string())),
        }
        Ok(())
    }
    pub async fn log_export_import(&self, user_id: Option<Uuid>, action: &str, resource: &str, resource_id: Option<Uuid>, status: &str, details: &Value) -> Result<(), AppError> {
        sqlx::query!(
            r#"INSERT INTO bantuan.export_import_logs (id, user_id, action, resource, resource_id, status, details, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
            Uuid::new_v4(), user_id, action, resource, resource_id, status, details, Utc::now()
        ).execute(&self.pool).await?;
        Ok(())
    }
} 