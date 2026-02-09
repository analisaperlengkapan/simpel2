use crate::config::AppConfig;
use crate::error::AppError;
use crate::models::DocumentTag;
use reqwest::Client;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

pub struct ClassifyService {
    pub config: AppConfig,
    client: Client,
}

impl ClassifyService {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    pub async fn classify_document(
        &self,
        document_id: Uuid,
        file_bytes: Vec<u8>,
    ) -> Result<Vec<String>, AppError> {
        let url = format!("{}/classify", self.config.ai_service_url);
        let mut req = self
            .client
            .post(&url)
            .header("Content-Type", "application/octet-stream");
        if let Some(ref key) = self.config.ai_service_api_key {
            req = req.header("x-api-key", key);
        }
        let resp = req
            .body(file_bytes)
            .send()
            .await
            .map_err(|e| AppError::Ai(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(AppError::Ai(format!(
                "AI classify gagal: {}",
                resp.status()
            )));
        }
        let tags: serde_json::Value = resp.json().await.map_err(|e| AppError::Ai(e.to_string()))?;
        let tags = tags["tags"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
        Ok(tags)
    }

    pub async fn add_tags(
        &self,
        pool: &PgPool,
        document_id: Uuid,
        tags: Vec<String>,
    ) -> Result<(), AppError> {
        for tag in tags {
            sqlx::query!(
                r#"INSERT INTO dokumen.document_tags (id, document_id, tag, created_at) VALUES ($1, $2, $3, NOW()) ON CONFLICT DO NOTHING"#,
                Uuid::new_v4(), document_id, tag
            ).execute(pool).await?;
        }
        Ok(())
    }

    pub async fn get_tags(
        &self,
        pool: &PgPool,
        document_id: Uuid,
    ) -> Result<Vec<DocumentTag>, AppError> {
        let tags = sqlx::query_as!(DocumentTag,
            r#"SELECT * FROM dokumen.document_tags WHERE document_id = $1 ORDER BY created_at DESC"#,
            document_id
        ).fetch_all(pool).await?;
        Ok(tags)
    }

    pub async fn update_tags(
        &self,
        pool: &PgPool,
        document_id: Uuid,
        tags: Vec<String>,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"DELETE FROM dokumen.document_tags WHERE document_id = $1"#,
            document_id
        )
        .execute(pool)
        .await?;
        self.add_tags(pool, document_id, tags).await
    }
}
