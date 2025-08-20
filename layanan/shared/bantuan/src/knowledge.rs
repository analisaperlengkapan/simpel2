use crate::models::KnowledgeArticle;
use crate::error::AppError;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;

pub struct KnowledgeService {
    pub pool: PgPool,
}

impl KnowledgeService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // CRUD Artikel
    pub async fn create_article(&self, title: &str, content: &str, category_id: Option<Uuid>, tags: Vec<String>) -> Result<KnowledgeArticle, AppError> {
        let art = sqlx::query_as!(KnowledgeArticle,
            r#"INSERT INTO bantuan.knowledge_articles (id, title, content, category_id, tags, created_at, updated_at, is_active)
            VALUES ($1, $2, $3, $4, $5, NOW(), NOW(), TRUE) RETURNING *"#,
            Uuid::new_v4(), title, content, category_id, &tags
        ).fetch_one(&self.pool).await?;
        Ok(art)
    }
    pub async fn get_article(&self, id: Uuid) -> Result<KnowledgeArticle, AppError> {
        let art = sqlx::query_as!(KnowledgeArticle,
            r#"SELECT * FROM bantuan.knowledge_articles WHERE id = $1"#,
            id
        ).fetch_one(&self.pool).await?;
        Ok(art)
    }
    pub async fn update_article(&self, id: Uuid, title: &str, content: &str, category_id: Option<Uuid>, tags: Vec<String>) -> Result<KnowledgeArticle, AppError> {
        let art = sqlx::query_as!(KnowledgeArticle,
            r#"UPDATE bantuan.knowledge_articles SET title = $1, content = $2, category_id = $3, tags = $4, updated_at = NOW() WHERE id = $5 RETURNING *"#,
            title, content, category_id, &tags, id
        ).fetch_one(&self.pool).await?;
        Ok(art)
    }
    pub async fn delete_article(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query!(
            r#"DELETE FROM bantuan.knowledge_articles WHERE id = $1"#,
            id
        ).execute(&self.pool).await?;
        Ok(())
    }
    pub async fn list_articles(&self, category_id: Option<Uuid>) -> Result<Vec<KnowledgeArticle>, AppError> {
        let arts = if let Some(cat_id) = category_id {
            sqlx::query_as!(KnowledgeArticle,
                r#"SELECT * FROM bantuan.knowledge_articles WHERE category_id = $1 ORDER BY created_at DESC"#,
                cat_id
            ).fetch_all(&self.pool).await?
        } else {
            sqlx::query_as!(KnowledgeArticle,
                r#"SELECT * FROM bantuan.knowledge_articles ORDER BY created_at DESC"#
            ).fetch_all(&self.pool).await?
        };
        Ok(arts)
    }
    // Search
    pub async fn search(&self, query: &str, max_results: u32) -> Result<Vec<KnowledgeArticle>, AppError> {
        let arts = sqlx::query_as!(KnowledgeArticle,
            r#"SELECT * FROM bantuan.knowledge_articles WHERE (title ILIKE $1 OR content ILIKE $1 OR $1 = ANY(tags)) AND is_active = TRUE ORDER BY updated_at DESC LIMIT $2"#,
            format!('%{}%', query), max_results as i64
        ).fetch_all(&self.pool).await?;
        Ok(arts)
    }
    // Get related
    pub async fn get_related(&self, article_id: Uuid, limit: u32) -> Result<Vec<KnowledgeArticle>, AppError> {
        let art = self.get_article(article_id).await?;
        let tags = art.tags.unwrap_or_default();
        let arts = sqlx::query_as!(KnowledgeArticle,
            r#"SELECT * FROM bantuan.knowledge_articles WHERE id != $1 AND tags && $2::text[] AND is_active = TRUE ORDER BY updated_at DESC LIMIT $3"#,
            article_id, &tags, limit as i64
        ).fetch_all(&self.pool).await?;
        Ok(arts)
    }
    // Export/import (backup/migrasi)
    pub async fn export_articles(&self) -> Result<Vec<KnowledgeArticle>, AppError> {
        let arts = sqlx::query_as!(KnowledgeArticle,
            r#"SELECT * FROM bantuan.knowledge_articles ORDER BY created_at ASC"#
        ).fetch_all(&self.pool).await?;
        Ok(arts)
    }
    pub async fn import_articles(&self, articles: Vec<KnowledgeArticle>) -> Result<(), AppError> {
        for art in articles {
            let _ = sqlx::query!(
                r#"INSERT INTO bantuan.knowledge_articles (id, title, content, category_id, tags, created_at, updated_at, is_active)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8) ON CONFLICT (id) DO NOTHING"#,
                art.id, art.title, art.content, art.category_id, &art.tags.unwrap_or_default(), art.created_at, art.updated_at, art.is_active
            ).execute(&self.pool).await;
        }
        Ok(())
    }
} 