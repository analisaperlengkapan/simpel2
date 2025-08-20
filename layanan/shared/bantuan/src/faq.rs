use crate::models::{FaqCategory, FaqArticle};
use crate::error::AppError;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;

pub struct FaqService {
    pub pool: PgPool,
}

impl FaqService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // Kategori FAQ
    pub async fn create_category(&self, name: &str, description: Option<&str>, parent_id: Option<Uuid>) -> Result<FaqCategory, AppError> {
        let cat = sqlx::query_as!(FaqCategory,
            r#"INSERT INTO bantuan.faq_categories (id, name, description, parent_id, created_at)
            VALUES ($1, $2, $3, $4, NOW()) RETURNING *"#,
            Uuid::new_v4(), name, description, parent_id
        ).fetch_one(&self.pool).await?;
        Ok(cat)
    }
    pub async fn get_category(&self, id: Uuid) -> Result<FaqCategory, AppError> {
        let cat = sqlx::query_as!(FaqCategory,
            r#"SELECT * FROM bantuan.faq_categories WHERE id = $1"#,
            id
        ).fetch_one(&self.pool).await?;
        Ok(cat)
    }
    pub async fn update_category(&self, id: Uuid, name: &str, description: Option<&str>, parent_id: Option<Uuid>) -> Result<FaqCategory, AppError> {
        let cat = sqlx::query_as!(FaqCategory,
            r#"UPDATE bantuan.faq_categories SET name = $1, description = $2, parent_id = $3 WHERE id = $4 RETURNING *"#,
            name, description, parent_id, id
        ).fetch_one(&self.pool).await?;
        Ok(cat)
    }
    pub async fn delete_category(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query!(
            r#"DELETE FROM bantuan.faq_categories WHERE id = $1"#,
            id
        ).execute(&self.pool).await?;
        Ok(())
    }
    pub async fn list_categories(&self) -> Result<Vec<FaqCategory>, AppError> {
        let cats = sqlx::query_as!(FaqCategory,
            r#"SELECT * FROM bantuan.faq_categories ORDER BY name"#
        ).fetch_all(&self.pool).await?;
        Ok(cats)
    }

    // Artikel FAQ
    pub async fn create_article(&self, category_id: Uuid, title: &str, content: &str, tags: Vec<String>) -> Result<FaqArticle, AppError> {
        let art = sqlx::query_as!(FaqArticle,
            r#"INSERT INTO bantuan.faq_articles (id, category_id, title, content, tags, version, created_at, updated_at, is_active)
            VALUES ($1, $2, $3, $4, $5, 1, NOW(), NOW(), TRUE) RETURNING *"#,
            Uuid::new_v4(), category_id, title, content, &tags
        ).fetch_one(&self.pool).await?;
        Ok(art)
    }
    pub async fn get_article(&self, id: Uuid) -> Result<FaqArticle, AppError> {
        let art = sqlx::query_as!(FaqArticle,
            r#"SELECT * FROM bantuan.faq_articles WHERE id = $1"#,
            id
        ).fetch_one(&self.pool).await?;
        Ok(art)
    }
    pub async fn update_article(&self, id: Uuid, title: &str, content: &str, tags: Vec<String>) -> Result<FaqArticle, AppError> {
        let art = sqlx::query_as!(FaqArticle,
            r#"UPDATE bantuan.faq_articles SET title = $1, content = $2, tags = $3, updated_at = NOW(), version = version + 1 WHERE id = $4 RETURNING *"#,
            title, content, &tags, id
        ).fetch_one(&self.pool).await?;
        Ok(art)
    }
    pub async fn delete_article(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query!(
            r#"DELETE FROM bantuan.faq_articles WHERE id = $1"#,
            id
        ).execute(&self.pool).await?;
        Ok(())
    }
    pub async fn list_articles(&self, category_id: Option<Uuid>) -> Result<Vec<FaqArticle>, AppError> {
        let arts = if let Some(cat_id) = category_id {
            sqlx::query_as!(FaqArticle,
                r#"SELECT * FROM bantuan.faq_articles WHERE category_id = $1 ORDER BY created_at DESC"#,
                cat_id
            ).fetch_all(&self.pool).await?
        } else {
            sqlx::query_as!(FaqArticle,
                r#"SELECT * FROM bantuan.faq_articles ORDER BY created_at DESC"#
            ).fetch_all(&self.pool).await?
        };
        Ok(arts)
    }
    // Search FAQ
    pub async fn search_faqs(&self, query: &str, max_results: u32) -> Result<Vec<FaqArticle>, AppError> {
        let arts = sqlx::query_as!(FaqArticle,
            r#"SELECT * FROM bantuan.faq_articles WHERE (title ILIKE $1 OR content ILIKE $1 OR $1 = ANY(tags)) AND is_active = TRUE ORDER BY updated_at DESC LIMIT $2"#,
            format!('%{}%', query), max_results as i64
        ).fetch_all(&self.pool).await?;
        Ok(arts)
    }
} 