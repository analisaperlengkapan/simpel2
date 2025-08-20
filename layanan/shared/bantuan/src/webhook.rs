use crate::models::WebhookEvent;
use crate::error::AppError;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use reqwest::Client;
use serde_json::Value;

pub struct WebhookService {
    pub pool: PgPool,
    pub client: Client,
}

impl WebhookService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool, client: Client::new() }
    }

    pub async fn create_event(&self, event_type: &str, payload: &Value) -> Result<WebhookEvent, AppError> {
        let event = sqlx::query_as!(WebhookEvent,
            r#"INSERT INTO bantuan.webhook_events (id, event_type, payload, delivered, created_at)
            VALUES ($1, $2, $3, FALSE, NOW()) RETURNING *"#,
            Uuid::new_v4(), event_type, payload
        ).fetch_one(&self.pool).await?;
        Ok(event)
    }
    pub async fn deliver_event(&self, event_id: Uuid, url: &str) -> Result<(), AppError> {
        let event = sqlx::query_as!(WebhookEvent,
            r#"SELECT * FROM bantuan.webhook_events WHERE id = $1"#,
            event_id
        ).fetch_one(&self.pool).await?;
        let resp = self.client.post(url)
            .json(&event.payload)
            .send().await;
        let delivered = resp.as_ref().map(|r| r.status().is_success()).unwrap_or(false);
        let now = Utc::now();
        sqlx::query!(
            r#"UPDATE bantuan.webhook_events SET delivered = $1, delivered_at = $2 WHERE id = $3"#,
            delivered, now, event_id
        ).execute(&self.pool).await?;
        Ok(())
    }
    pub async fn list_events(&self, event_type: Option<&str>, delivered: Option<bool>, limit: i64) -> Result<Vec<WebhookEvent>, AppError> {
        let events = if let Some(et) = event_type {
            sqlx::query_as!(WebhookEvent,
                r#"SELECT * FROM bantuan.webhook_events WHERE event_type = $1 AND ($2::bool IS NULL OR delivered = $2) ORDER BY created_at DESC LIMIT $3"#,
                et, delivered, limit
            ).fetch_all(&self.pool).await?
        } else {
            sqlx::query_as!(WebhookEvent,
                r#"SELECT * FROM bantuan.webhook_events WHERE ($1::bool IS NULL OR delivered = $1) ORDER BY created_at DESC LIMIT $2"#,
                delivered, limit
            ).fetch_all(&self.pool).await?
        };
        Ok(events)
    }
    pub async fn retry_event(&self, event_id: Uuid, url: &str) -> Result<(), AppError> {
        self.deliver_event(event_id, url).await
    }
} 