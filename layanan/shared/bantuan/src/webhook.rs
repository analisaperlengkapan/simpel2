use crate::models::WebhookEvent;
use crate::error::AppError;
use deadpool_postgres::Pool;
use chrono::Utc;
use uuid::Uuid;
use reqwest::Client;
use serde_json::Value;

pub struct WebhookService {
    pub pool: Pool,
    pub client: Client,
}

impl WebhookService {
    pub fn new(pool: Pool) -> Self {
        Self { pool, client: Client::new() }
    }

    pub async fn create_event(&self, event_type: &str, payload: &Value) -> Result<WebhookEvent, AppError> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            r#"INSERT INTO bantuan.webhook_events (id, event_type, payload, delivered, created_at)
            VALUES ($1, $2, $3, FALSE, NOW()) RETURNING *"#,
            &[&Uuid::new_v4(), &event_type, &payload]
        ).await?;
        Ok(WebhookEvent::from(&row))
    }
    pub async fn deliver_event(&self, event_id: Uuid, url: &str) -> Result<(), AppError> {
        let client = self.pool.get().await?;
        let row = client.query_one(
            r#"SELECT * FROM bantuan.webhook_events WHERE id = $1"#,
            &[&event_id]
        ).await?;
        let event = WebhookEvent::from(&row);
        let resp = self.client.post(url)
            .json(&event.payload)
            .send().await;
        let delivered = resp.as_ref().map(|r| r.status().is_success()).unwrap_or(false);
        let now = Utc::now();
        client.execute(
            r#"UPDATE bantuan.webhook_events SET delivered = $1, delivered_at = $2 WHERE id = $3"#,
            &[&delivered, &now, &event_id]
        ).await?;
        Ok(())
    }
    pub async fn list_events(&self, event_type: Option<&str>, delivered: Option<bool>, limit: i64) -> Result<Vec<WebhookEvent>, AppError> {
        let client = self.pool.get().await?;
        let rows = if let Some(et) = event_type {
            client.query(
                r#"SELECT * FROM bantuan.webhook_events WHERE event_type = $1 AND ($2::bool IS NULL OR delivered = $2) ORDER BY created_at DESC LIMIT $3"#,
                &[&et, &delivered, &limit]
            ).await?
        } else {
            client.query(
                r#"SELECT * FROM bantuan.webhook_events WHERE ($1::bool IS NULL OR delivered = $1) ORDER BY created_at DESC LIMIT $2"#,
                &[&delivered, &limit]
            ).await?
        };
        let events = rows.into_iter().map(|row| WebhookEvent::from(&row)).collect();
        Ok(events)
    }
    pub async fn retry_event(&self, event_id: Uuid, url: &str) -> Result<(), AppError> {
        self.deliver_event(event_id, url).await
    }
}
