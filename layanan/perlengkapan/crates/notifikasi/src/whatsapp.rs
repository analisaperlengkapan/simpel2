use crate::config::AppConfig;
use crate::error::AppError;
use crate::models::{Notification, NotificationRecipient};
use deadpool_postgres::Pool;
use reqwest::Client;
use serde_json::json;
use uuid::Uuid;

#[allow(dead_code)]
pub struct WhatsAppService {
    #[allow(dead_code)]
    pub config: AppConfig,
    #[allow(dead_code)]
    pub pool: Pool,
    #[allow(dead_code)]
    pub client: Client,
}

impl WhatsAppService {
    #[allow(dead_code)]
    pub fn new(config: AppConfig, pool: Pool) -> Self {
        Self {
            config,
            pool,
            client: Client::new(),
        }
    }

    #[allow(dead_code)]
    pub async fn send_whatsapp(
        &self,
        phone_number: &str,
        template_name: &str,
        variables: serde_json::Value,
    ) -> Result<Notification, AppError> {
        let notif_id = Uuid::new_v4();
        // Insert notification DB (pending)
        let row = self.pool.get().await?.query_one(
            r#"INSERT INTO notifikasi.notifications (id, channel, status, subject, body, created_at)
            VALUES ($1, 'whatsapp', 'pending', $2, $3, NOW()) RETURNING *"#,
            &[&notif_id, &template_name, &variables.to_string()]
        ).await?;
        let notif: Notification = row.into();
        // Insert recipient
        self.pool.get().await?.execute(
            r#"INSERT INTO notifikasi.notification_recipients (id, notification_id, recipient, recipient_type, status)
            VALUES ($1, $2, $3, 'phone', 'pending')"#,
            &[&Uuid::new_v4(), &notif_id, &phone_number]
        ).await?;
        // Kirim WhatsApp
        let api_url = self.config.whatsapp_api_url.as_deref().unwrap_or("");
        let token = self.config.whatsapp_access_token.as_deref().unwrap_or("");
        let phone_id = self
            .config
            .whatsapp_phone_number_id
            .as_deref()
            .unwrap_or("");
        let url = format!("{}/{}/messages", api_url, phone_id);
        let payload = json!({
            "messaging_product": "whatsapp",
            "to": phone_number,
            "type": "template",
            "template": {
                "name": template_name,
                "language": {"code": "id"},
                "components": [
                    {"type": "body", "parameters": variables.as_array().unwrap_or(&vec![])}
                ]
            }
        });
        let resp = self
            .client
            .post(&url)
            .bearer_auth(token)
            .json(&payload)
            .send()
            .await;
        let status = if let Ok(r) = &resp {
            if r.status().is_success() {
                "sent"
            } else {
                "failed"
            }
        } else {
            "failed"
        };
        let error_message = resp.as_ref().err().map(|e| e.to_string());
        // Update status DB
        self.pool.get().await?.execute(
            r#"UPDATE notifikasi.notifications SET status = $1, sent_at = NOW(), error_message = $2 WHERE id = $3"#,
            &[&status, &error_message, &notif_id]
        ).await?;
        self.pool.get().await?.execute(
            r#"UPDATE notifikasi.notification_recipients SET status = $1, sent_at = NOW(), error_message = $2 WHERE notification_id = $3 AND recipient = $4"#,
            &[&status, &error_message, &notif_id, &phone_number]
        ).await?;
        Ok(notif)
    }

    #[allow(dead_code)]
    pub async fn send_batch_whatsapp(
        &self,
        recipients: Vec<String>,
        template_name: &str,
        variables: serde_json::Value,
    ) -> Result<Vec<NotificationRecipient>, AppError> {
        let mut results = Vec::new();
        for recipient in recipients {
            let _ = self
                .send_whatsapp(&recipient, template_name, variables.clone())
                .await;
            let row = self.pool.get().await?.query_one(
                r#"SELECT * FROM notifikasi.notification_recipients WHERE recipient = $1 ORDER BY sent_at DESC LIMIT 1"#,
                &[&recipient]
            ).await?;
            let rec: NotificationRecipient = row.into();
            results.push(rec);
        }
        Ok(results)
    }

    #[allow(dead_code)]
    pub async fn get_status(&self, notification_id: Uuid) -> Result<Notification, AppError> {
        let row = self
            .pool
            .get()
            .await?
            .query_one(
                r#"SELECT * FROM notifikasi.notifications WHERE id = $1"#,
                &[&notification_id],
            )
            .await?;
        let notif: Notification = row.into();
        Ok(notif)
    }
}
