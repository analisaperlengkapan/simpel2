use crate::config::AppConfig;
use crate::error::AppError;
use crate::models::{Notification, NotificationRecipient};
use reqwest::Client;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

pub struct PushService {
    pub config: AppConfig,
    pub pool: PgPool,
    pub client: Client,
}

impl PushService {
    pub fn new(config: AppConfig, pool: PgPool) -> Self {
        Self {
            config,
            pool,
            client: Client::new(),
        }
    }

    pub async fn send_push(
        &self,
        device_token: &str,
        title: &str,
        body: &str,
        data: serde_json::Value,
    ) -> Result<Notification, AppError> {
        let notif_id = Uuid::new_v4();
        // Insert notification DB (pending)
        let notif = sqlx::query_as!(
            Notification,
            r#"INSERT INTO notifikasi.notifications (id, channel, status, subject, body, created_at)
            VALUES ($1, 'push', 'pending', $2, $3, NOW()) RETURNING *"#,
            notif_id,
            title,
            body
        )
        .fetch_one(&self.pool)
        .await?;
        // Insert recipient
        sqlx::query!(
            r#"INSERT INTO notifikasi.notification_recipients (id, notification_id, recipient, recipient_type, status)
            VALUES ($1, $2, $3, 'device_token', 'pending')"#,
            Uuid::new_v4(), notif_id, device_token
        ).execute(&self.pool).await?;
        // Kirim push (FCM)
        let fcm_key = self.config.fcm_server_key.as_deref().unwrap_or("");
        let url = "https://fcm.googleapis.com/fcm/send";
        let payload = json!({
            "to": device_token,
            "notification": {"title": title, "body": body},
            "data": data
        });
        let resp = self
            .client
            .post(url)
            .bearer_auth(fcm_key)
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
        sqlx::query!(
            r#"UPDATE notifikasi.notifications SET status = $1, sent_at = NOW(), error_message = $2 WHERE id = $3"#,
            status, error_message, notif_id
        ).execute(&self.pool).await?;
        sqlx::query!(
            r#"UPDATE notifikasi.notification_recipients SET status = $1, sent_at = NOW(), error_message = $2 WHERE notification_id = $3 AND recipient = $4"#,
            status, error_message, notif_id, device_token
        ).execute(&self.pool).await?;
        Ok(notif)
    }

    pub async fn send_batch_push(
        &self,
        device_tokens: Vec<String>,
        title: &str,
        body: &str,
        data: serde_json::Value,
    ) -> Result<Vec<NotificationRecipient>, AppError> {
        let mut results = Vec::new();
        for token in device_tokens {
            let _ = self.send_push(&token, title, body, data.clone()).await;
            let rec = sqlx::query_as!(NotificationRecipient,
                r#"SELECT * FROM notifikasi.notification_recipients WHERE recipient = $1 ORDER BY sent_at DESC LIMIT 1"#,
                token
            ).fetch_one(&self.pool).await?;
            results.push(rec);
        }
        Ok(results)
    }

    pub async fn get_status(&self, notification_id: Uuid) -> Result<Notification, AppError> {
        let notif = sqlx::query_as!(
            Notification,
            r#"SELECT * FROM notifikasi.notifications WHERE id = $1"#,
            notification_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(notif)
    }
}
