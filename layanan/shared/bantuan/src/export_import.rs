use crate::error::AppError;
use crate::models::{FaqArticle, KnowledgeArticle, SupportTicket};
use chrono::Utc;
use deadpool_postgres::Pool;
use serde_json::Value;
use uuid::Uuid;

pub struct ExportImportService {
    pub pool: Pool,
}

impl ExportImportService {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    pub async fn export_resource(&self, resource: &str) -> Result<Value, AppError> {
        let client = self.pool.get().await?;
        let data = match resource {
            "knowledge_articles" => {
                let rows = client
                    .query(
                        r#"SELECT * FROM bantuan.knowledge_articles ORDER BY created_at ASC"#,
                        &[],
                    )
                    .await?;
                let articles = rows
                    .into_iter()
                    .map(|row| KnowledgeArticle::from(&row))
                    .collect::<Vec<_>>();
                serde_json::to_value(articles)
                    .map_err(|_e| AppError::Validation("Internal error".to_string()))?
            }
            "faq_articles" => {
                let rows = client
                    .query(
                        r#"SELECT * FROM bantuan.faq_articles ORDER BY created_at ASC"#,
                        &[],
                    )
                    .await?;
                let articles = rows
                    .into_iter()
                    .map(|row| FaqArticle::from(&row))
                    .collect::<Vec<_>>();
                serde_json::to_value(articles)
                    .map_err(|_e| AppError::Validation("Internal error".to_string()))?
            }
            "support_tickets" => {
                let rows = client
                    .query(
                        r#"SELECT * FROM bantuan.support_tickets ORDER BY created_at ASC"#,
                        &[],
                    )
                    .await?;
                let tickets = rows
                    .into_iter()
                    .map(|row| SupportTicket::from(&row))
                    .collect::<Vec<_>>();
                serde_json::to_value(tickets)
                    .map_err(|_e| AppError::Validation("Internal error".to_string()))?
            }
            _ => return Err(AppError::BadRequest("Resource tidak didukung".to_string())),
        };
        Ok(data)
    }
    pub async fn import_resource(&self, resource: &str, data: Value) -> Result<(), AppError> {
        let client = self.pool.get().await?;
        match resource {
            "knowledge_articles" => {
                let articles: Vec<KnowledgeArticle> = serde_json::from_value(data)
                    .map_err(|e| AppError::BadRequest(e.to_string()))?;
                for art in articles {
                    let _ = client.execute(
                        r#"INSERT INTO bantuan.knowledge_articles (id, title, content, category_id, tags, created_at, updated_at, is_active)
                        VALUES ($1, $2, $3, $4, $5, $6, $7, $8) ON CONFLICT (id) DO NOTHING"#,
                        &[&art.id, &art.title, &art.content, &art.category_id, &art.tags.unwrap_or_default(), &art.created_at, &art.updated_at, &art.is_active]
                    ).await;
                }
            }
            "faq_articles" => {
                let articles: Vec<FaqArticle> = serde_json::from_value(data)
                    .map_err(|e| AppError::BadRequest(e.to_string()))?;
                for art in articles {
                    let _ = client.execute(
                        r#"INSERT INTO bantuan.faq_articles (id, category_id, title, content, tags, version, created_at, updated_at, is_active)
                        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) ON CONFLICT (id) DO NOTHING"#,
                        &[&art.id, &art.category_id, &art.title, &art.content, &art.tags.unwrap_or_default(), &art.version, &art.created_at, &art.updated_at, &art.is_active]
                    ).await;
                }
            }
            "support_tickets" => {
                let tickets: Vec<SupportTicket> = serde_json::from_value(data)
                    .map_err(|e| AppError::BadRequest(e.to_string()))?;
                for t in tickets {
                    let _ = client.execute(
                        r#"INSERT INTO bantuan.support_tickets (id, user_id, subject, description, priority, category_id, status, created_at, updated_at, closed_at, spam_score)
                        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) ON CONFLICT (id) DO NOTHING"#,
                        &[&t.id, &t.user_id, &t.subject, &t.description, &t.priority, &t.category_id, &t.status, &t.created_at, &t.updated_at, &t.closed_at, &t.spam_score]
                    ).await;
                }
            }
            _ => return Err(AppError::BadRequest("Resource tidak didukung".to_string())),
        }
        Ok(())
    }
    #[allow(dead_code)]
    pub async fn log_export_import(
        &self,
        user_id: Option<Uuid>,
        action: &str,
        resource: &str,
        resource_id: Option<Uuid>,
        status: &str,
        details: &Value,
    ) -> Result<(), AppError> {
        let client = self.pool.get().await?;
        client.execute(
            r#"INSERT INTO bantuan.export_import_logs (id, user_id, action, resource, resource_id, status, details, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
            &[&Uuid::new_v4(), &user_id, &action, &resource, &resource_id, &status, &details, &Utc::now()]
        ).await?;
        Ok(())
    }
}
