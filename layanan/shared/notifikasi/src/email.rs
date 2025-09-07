use crate::config::AppConfig;
use crate::error::AppError;
use crate::models::{Notification, NotificationRecipient};
use lettre::{message::Mailbox, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use sqlx::PgPool;
use uuid::Uuid;

pub struct EmailService {
    pub config: AppConfig,
    pub pool: PgPool,
    pub mailer: AsyncSmtpTransport<Tokio1Executor>,
}

impl EmailService {
    pub fn new(config: AppConfig, pool: PgPool) -> Self {
        let creds = lettre::transport::smtp::authentication::Credentials::new(
            config.smtp_username.clone(),
            config.smtp_password.clone(),
        );
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_host)
            .unwrap()
            .port(config.smtp_port)
            .credentials(creds)
            .build();
        Self {
            config,
            pool,
            mailer,
        }
    }

    pub async fn send_email(
        &self,
        recipient: &str,
        subject: &str,
        body: &str,
    ) -> Result<Notification, AppError> {
        let email = Message::builder()
            .from(self.config.smtp_from.parse::<Mailbox>().unwrap())
            .to(recipient.parse::<Mailbox>().unwrap())
            .subject(subject)
            .body(body.to_string())
            .map_err(|e| AppError::Email(e.to_string()))?;
        let notif_id = Uuid::new_v4();
        // Insert notification DB (pending)
        let notif = sqlx::query_as!(
            Notification,
            r#"INSERT INTO notifikasi.notifications (id, channel, status, subject, body, created_at)
            VALUES ($1, 'email', 'pending', $2, $3, NOW()) RETURNING *"#,
            notif_id,
            subject,
            body
        )
        .fetch_one(&self.pool)
        .await?;
        // Insert recipient
        sqlx::query!(
            r#"INSERT INTO notifikasi.notification_recipients (id, notification_id, recipient, recipient_type, status)
            VALUES ($1, $2, $3, 'email', 'pending')"#,
            Uuid::new_v4(), notif_id, recipient
        ).execute(&self.pool).await?;
        // Kirim email
        let result = self.mailer.send(email).await;
        let status = if result.is_ok() { "sent" } else { "failed" };
        let error_message = result.as_ref().err().map(|e| e.to_string());
        // Update status DB
        sqlx::query!(
            r#"UPDATE notifikasi.notifications SET status = $1, sent_at = NOW(), error_message = $2 WHERE id = $3"#,
            status, error_message, notif_id
        ).execute(&self.pool).await?;
        sqlx::query!(
            r#"UPDATE notifikasi.notification_recipients SET status = $1, sent_at = NOW(), error_message = $2 WHERE notification_id = $3 AND recipient = $4"#,
            status, error_message, notif_id, recipient
        ).execute(&self.pool).await?;
        Ok(notif)
    }

    pub async fn send_batch_emails(
        &self,
        recipients: Vec<String>,
        subject: &str,
        body: &str,
    ) -> Result<Vec<NotificationRecipient>, AppError> {
        let mut results = Vec::new();
        for recipient in recipients {
            let _ = self.send_email(&recipient, subject, body).await; // Ignore error per recipient
            let rec = sqlx::query_as!(NotificationRecipient,
                r#"SELECT * FROM notifikasi.notification_recipients WHERE recipient = $1 ORDER BY sent_at DESC LIMIT 1"#,
                recipient
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
