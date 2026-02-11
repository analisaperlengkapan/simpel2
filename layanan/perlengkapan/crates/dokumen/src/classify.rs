use crate::config::AppConfig;
use crate::error::AppError;
use crate::models::DocumentTag;
use reqwest::Client;
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
        _document_id: Uuid,
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
        pool: &deadpool_postgres::Pool,
        document_id: Uuid,
        tags: Vec<String>,
    ) -> Result<(), AppError> {
        let client = pool.get().await?;
        for tag in tags {
            let id = Uuid::new_v4();
            client
                .execute(
                    "INSERT INTO dokumen.document_tags (id, document_id, tag, created_at) VALUES ($1, $2, $3, NOW()) ON CONFLICT DO NOTHING",
                    &[&id, &document_id, &tag],
                )
                .await?;
        }
        Ok(())
    }

    pub async fn get_tags(
        &self,
        pool: &deadpool_postgres::Pool,
        document_id: Uuid,
    ) -> Result<Vec<DocumentTag>, AppError> {
        let client = pool.get().await?;
        let rows = client
            .query(
                "SELECT * FROM dokumen.document_tags WHERE document_id = $1 ORDER BY created_at DESC",
                &[&document_id],
            )
            .await?;
        Ok(rows.iter().map(DocumentTag::from).collect())
    }

    pub async fn update_tags(
        &self,
        pool: &deadpool_postgres::Pool,
        document_id: Uuid,
        tags: Vec<String>,
    ) -> Result<(), AppError> {
        let client = pool.get().await?;
        client
            .execute(
                "DELETE FROM dokumen.document_tags WHERE document_id = $1",
                &[&document_id],
            )
            .await?;
        drop(client);
        self.add_tags(pool, document_id, tags).await
    }
}
