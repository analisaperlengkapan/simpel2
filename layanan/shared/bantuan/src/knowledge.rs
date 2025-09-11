use crate::models::KnowledgeArticle;
use crate::error::AppError;
use deadpool_postgres::Pool;
use uuid::Uuid;

pub struct KnowledgeService {
    pub pool: Pool,
}

impl KnowledgeService {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    // CRUD Artikel
    pub async fn create_article(&self, title: &str, content: &str, category_id: Option<Uuid>, tags: Vec<String>) -> Result<KnowledgeArticle, AppError> {
        let client = self.pool.get().await?;
        let id = Uuid::new_v4();
        let row = client.query_one(
            r#"INSERT INTO bantuan.knowledge_articles (id, title, content, category_id, tags, created_at, updated_at, is_active)
            VALUES ($1, $2, $3, $4, $5, NOW(), NOW(), TRUE) RETURNING *"#,
            &[&id, &title, &content, &category_id, &tags]
        ).await?;
        Ok(KnowledgeArticle::from(&row))
    }
    pub async fn get_article(&self, id: Uuid) -> Result<KnowledgeArticle, AppError> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            r#"SELECT * FROM bantuan.knowledge_articles WHERE id = $1"#,
            &[&id]
        ).await?;
        Ok(KnowledgeArticle::from(&row))
    }
    pub async fn update_article(&self, id: Uuid, title: &str, content: &str, category_id: Option<Uuid>, tags: Vec<String>) -> Result<KnowledgeArticle, AppError> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            r#"UPDATE bantuan.knowledge_articles SET title = $1, content = $2, category_id = $3, tags = $4, updated_at = NOW() WHERE id = $5 RETURNING *"#,
            &[&title, &content, &category_id, &tags, &id]
        ).await?;
        Ok(KnowledgeArticle::from(&row))
    }
    pub async fn delete_article(&self, id: Uuid) -> Result<(), AppError> {
        let client = self.pool.get().await?;
        client.execute(
            r#"DELETE FROM bantuan.knowledge_articles WHERE id = $1"#,
            &[&id]
        ).await?;
        Ok(())
    }
    pub async fn list_articles(&self, category_id: Option<Uuid>) -> Result<Vec<KnowledgeArticle>, AppError> {
        let client = self.pool.get().await?;
        let rows = if let Some(cat_id) = category_id {
            client.query(
                r#"SELECT * FROM bantuan.knowledge_articles WHERE category_id = $1 ORDER BY created_at DESC"#,
                &[&cat_id]
            ).await?
        } else {
            client.query(
                r#"SELECT * FROM bantuan.knowledge_articles ORDER BY created_at DESC"#,
                &[]
            ).await?
        };
        let arts = rows.into_iter().map(|row| KnowledgeArticle::from(&row)).collect();
        Ok(arts)
    }
    // Search
    pub async fn search(&self, query: &str, max_results: u32) -> Result<Vec<KnowledgeArticle>, AppError> {
        let client = self.pool.get().await?;
        let search_pattern = format!("%{}%", query);
        let rows = client.query(
            r#"SELECT * FROM bantuan.knowledge_articles WHERE (title ILIKE $1 OR content ILIKE $1 OR $1 = ANY(tags)) AND is_active = TRUE ORDER BY updated_at DESC LIMIT $2"#,
            &[&search_pattern, &(max_results as i64)]
        ).await?;
        let arts = rows.into_iter().map(|row| KnowledgeArticle::from(&row)).collect();
        Ok(arts)
    }
    // Get related
    pub async fn get_related(&self, article_id: Uuid, limit: u32) -> Result<Vec<KnowledgeArticle>, AppError> {
        let art = self.get_article(article_id).await?;
        let tags = art.tags.unwrap_or_default();
        let client = self.pool.get().await?;
        let rows = client.query(
            r#"SELECT * FROM bantuan.knowledge_articles WHERE id != $1 AND tags && $2::text[] AND is_active = TRUE ORDER BY updated_at DESC LIMIT $3"#,
            &[&article_id, &tags, &(limit as i64)]
        ).await?;
        let arts = rows.into_iter().map(|row| KnowledgeArticle::from(&row)).collect();
        Ok(arts)
    }
    // Export/import (backup/migrasi)
    pub async fn export_articles(&self) -> Result<Vec<KnowledgeArticle>, AppError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            r#"SELECT * FROM bantuan.knowledge_articles ORDER BY created_at ASC"#,
            &[]
        ).await?;
        let arts = rows.into_iter().map(|row| KnowledgeArticle::from(&row)).collect();
        Ok(arts)
    }
    pub async fn import_articles(&self, articles: Vec<KnowledgeArticle>) -> Result<(), AppError> {
        let client = self.pool.get().await?;
        for art in articles {
            let _ = client.execute(
                r#"INSERT INTO bantuan.knowledge_articles (id, title, content, category_id, tags, created_at, updated_at, is_active)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8) ON CONFLICT (id) DO NOTHING"#,
                &[&art.id, &art.title, &art.content, &art.category_id, &art.tags.unwrap_or_default(), &art.created_at, &art.updated_at, &art.is_active]
            ).await;
        }
        Ok(())
    }
}
