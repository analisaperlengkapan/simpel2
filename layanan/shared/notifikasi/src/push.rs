use crate::config::AppConfig;
use crate::error::AppError;
use crate::models::{Notification, NotificationRecipient};
use deadpool_postgres::Pool;
use reqwest::Client;
use serde_json::json;
use uuid::Uuid;

pub struct PushService {
    #[allow(dead_code)]
    pub config: AppConfig,
    #[allow(dead_code)]
    pub pool: Pool,
    #[allow(dead_code)]
    pub client: Client,
}

impl PushService {
    pub fn new(config: AppConfig, pool: Pool) -> Self {
        Self {
            config,
            pool,
            client: Client::new(),
        }
    }

    #[allow(dead_code)]
    pub async fn send_push(
        &self,
        device_token: &str,
        title: &str,
        body: &str,
        data: serde_json::Value,
    ) -> Result<Notification, AppError> {
        let notif_id = Uuid::new_v4();
        // Insert notification DB (pending)
        let sql = r#"INSERT INTO notifikasi.notifications (id, channel, status, subject, body, created_at)
            VALUES ($1, 'push', 'pending', $2, $3, NOW()) RETURNING id, channel, status, subject, body, created_at, sent_at, error_message"#;
        let row = self
            .pool
            .get()
            .await?
            .query_one(sql, &[&notif_id, &title, &body])
            .await?;
        let notif: Notification = row.into();
        // Insert recipient
        let sql_recipient = r#"INSERT INTO notifikasi.notification_recipients (id, notification_id, recipient, recipient_type, status)
            VALUES ($1, $2, $3, 'device_token', 'pending')"#;
        self.pool
            .get()
            .await?
            .execute(sql_recipient, &[&Uuid::new_v4(), &notif_id, &device_token])
            .await?;
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
        let sql_update_notif = r#"UPDATE notifikasi.notifications SET status = $1, sent_at = NOW(), error_message = $2 WHERE id = $3"#;
        self.pool
            .get()
            .await?
            .execute(sql_update_notif, &[&status, &error_message, &notif_id])
            .await?;
        let sql_update_recipient = r#"UPDATE notifikasi.notification_recipients SET status = $1, sent_at = NOW(), error_message = $2 WHERE notification_id = $3 AND recipient = $4"#;
        self.pool
            .get()
            .await?
            .execute(
                sql_update_recipient,
                &[&status, &error_message, &notif_id, &device_token],
            )
            .await?;
        Ok(notif)
    }

    #[allow(dead_code)]
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
            let sql_recipient_status = r#"SELECT id, notification_id, recipient, recipient_type, status, sent_at, error_message FROM notifikasi.notification_recipients WHERE recipient = $1 ORDER BY sent_at DESC LIMIT 1"#;
            let row = self
                .pool
                .get()
                .await?
                .query_one(sql_recipient_status, &[&token])
                .await?;
            let rec: NotificationRecipient = row.into();
            results.push(rec);
        }
        Ok(results)
    }

    #[allow(dead_code)]
    pub async fn get_status(&self, notification_id: Uuid) -> Result<Notification, AppError> {
        let sql = r#"SELECT id, channel, status, subject, body, created_at, sent_at, error_message FROM notifikasi.notifications WHERE id = $1"#;
        let row = self
            .pool
            .get()
            .await?
            .query_one(sql, &[&notification_id])
            .await?;
        let notif: Notification = row.into();
        Ok(notif)
    }
}
