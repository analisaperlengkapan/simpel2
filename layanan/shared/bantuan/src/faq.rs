use crate::models::{FaqCategory, FaqArticle};
use crate::error::AppError;
use deadpool_postgres::Pool;
use uuid::Uuid;

pub struct FaqService {
    pub pool: Pool,
}

impl FaqService {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    // Kategori FAQ
    pub async fn create_category(&self, name: &str, description: Option<&str>, parent_id: Option<Uuid>) -> Result<FaqCategory, AppError> {
        let client = self.pool.get().await?;
        let id = Uuid::new_v4();
        let row = client.query_one(
            r#"INSERT INTO bantuan.faq_categories (id, name, description, parent_id, created_at)
            VALUES ($1, $2, $3, $4, NOW()) RETURNING *"#,
            &[&id, &name, &description, &parent_id]
        ).await?;
        Ok(FaqCategory::from(&row))
    }
    pub async fn get_category(&self, id: Uuid) -> Result<FaqCategory, AppError> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            r#"SELECT * FROM bantuan.faq_categories WHERE id = $1"#,
            &[&id]
        ).await?;
        Ok(FaqCategory::from(&row))
    }
    pub async fn update_category(&self, id: Uuid, name: &str, description: Option<&str>, parent_id: Option<Uuid>) -> Result<FaqCategory, AppError> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            r#"UPDATE bantuan.faq_categories SET name = $1, description = $2, parent_id = $3 WHERE id = $4 RETURNING *"#,
            &[&name, &description, &parent_id, &id]
        ).await?;
        Ok(FaqCategory::from(&row))
    }
    pub async fn delete_category(&self, id: Uuid) -> Result<(), AppError> {
        let client = self.pool.get().await?;
        client.execute(
            r#"DELETE FROM bantuan.faq_categories WHERE id = $1"#,
            &[&id]
        ).await?;
        Ok(())
    }
    pub async fn list_categories(&self) -> Result<Vec<FaqCategory>, AppError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            r#"SELECT * FROM bantuan.faq_categories ORDER BY name"#,
            &[]
        ).await?;
        Ok(rows.into_iter().map(|row| FaqCategory::from(&row)).collect())
    }

    // Artikel FAQ
    pub async fn create_article(&self, category_id: Uuid, title: &str, content: &str, tags: Vec<String>) -> Result<FaqArticle, AppError> {
        let client = self.pool.get().await?;
        let id = Uuid::new_v4();
        let row = client.query_one(
            r#"INSERT INTO bantuan.faq_articles (id, category_id, title, content, tags, version, created_at, updated_at, is_active)
            VALUES ($1, $2, $3, $4, $5, 1, NOW(), NOW(), TRUE) RETURNING *"#,
            &[&id, &category_id, &title, &content, &tags]
        ).await?;
        Ok(FaqArticle::from(&row))
    }
    pub async fn get_article(&self, id: Uuid) -> Result<FaqArticle, AppError> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            r#"SELECT * FROM bantuan.faq_articles WHERE id = $1"#,
            &[&id]
        ).await?;
        Ok(FaqArticle::from(&row))
    }
    pub async fn update_article(&self, id: Uuid, title: &str, content: &str, tags: Vec<String>) -> Result<FaqArticle, AppError> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            r#"UPDATE bantuan.faq_articles SET title = $1, content = $2, tags = $3, updated_at = NOW(), version = version + 1 WHERE id = $4 RETURNING *"#,
            &[&title, &content, &tags, &id]
        ).await?;
        Ok(FaqArticle::from(&row))
    }
    pub async fn delete_article(&self, id: Uuid) -> Result<(), AppError> {
        let client = self.pool.get().await?;
        client.execute(
            r#"DELETE FROM bantuan.faq_articles WHERE id = $1"#,
            &[&id]
        ).await?;
        Ok(())
    }
    pub async fn list_articles(&self, category_id: Option<Uuid>) -> Result<Vec<FaqArticle>, AppError> {
        let client = self.pool.get().await?;
        let rows = if let Some(cat_id) = category_id {
            client.query(
                r#"SELECT * FROM bantuan.faq_articles WHERE category_id = $1 ORDER BY created_at DESC"#,
                &[&cat_id]
            ).await?
        } else {
            client.query(
                r#"SELECT * FROM bantuan.faq_articles ORDER BY created_at DESC"#,
                &[]
            ).await?
        };
        Ok(rows.into_iter().map(|row| FaqArticle::from(&row)).collect())
    }
    // Search FAQ
    pub async fn search_faqs(&self, query: &str, max_results: u32) -> Result<Vec<FaqArticle>, AppError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            r#"SELECT * FROM bantuan.faq_articles WHERE (title ILIKE $1 OR content ILIKE $1 OR $1 = ANY(tags)) AND is_active = TRUE ORDER BY updated_at DESC LIMIT $2"#,
            &[&format!("%{}%", query), &(max_results as i64)]
        ).await?;
        Ok(rows.into_iter().map(|row| FaqArticle::from(&row)).collect())
    }
}
