use crate::config::AppConfig;
use crate::error::AppError;
use crate::models::{Notification, NotificationRecipient};
use deadpool_postgres::Pool;
use lettre::{message::Mailbox, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use uuid::Uuid;

pub struct EmailService {
    pub config: AppConfig,
    pub pool: Pool,
    pub mailer: AsyncSmtpTransport<Tokio1Executor>,
}

impl EmailService {
    pub fn new(config: AppConfig, pool: Pool) -> Self {
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
        let client = self.pool.get().await?;
        let row = client
            .query_one(
                "INSERT INTO notifikasi.notifications (id, channel, status, subject, body, created_at)
                 VALUES ($1, 'email', 'pending', $2, $3, NOW()) RETURNING *",
                &[&notif_id, &subject, &body],
            )
            .await?;
        let notif = Notification::from_row(&row);
        // Insert recipient
        client
            .execute(
                "INSERT INTO notifikasi.notification_recipients (id, notification_id, recipient, recipient_type, status)
                 VALUES ($1, $2, $3, 'email', 'pending')",
                &[&Uuid::new_v4(), &notif_id, &recipient],
            )
            .await?;
        // Kirim email
        let result = self.mailer.send(email).await;
        let status = if result.is_ok() { "sent" } else { "failed" };
        let error_message = result.as_ref().err().map(|e| e.to_string());
        // Update status DB
        client
            .execute(
                "UPDATE notifikasi.notifications SET status = $1, sent_at = NOW(), error_message = $2 WHERE id = $3",
                &[&status, &error_message, &notif_id],
            )
            .await?;
        client
            .execute(
                "UPDATE notifikasi.notification_recipients SET status = $1, sent_at = NOW(), error_message = $2 WHERE notification_id = $3 AND recipient = $4",
                &[&status, &error_message, &notif_id, &recipient],
            )
            .await?;
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
            let client = self.pool.get().await?;
            let row = client
                .query_one(
                    "SELECT * FROM notifikasi.notification_recipients WHERE recipient = $1 ORDER BY sent_at DESC LIMIT 1",
                    &[&recipient],
                )
                .await?;
            let rec = NotificationRecipient::from_row(&row);
            results.push(rec);
        }
        Ok(results)
    }

    pub async fn get_status(&self, notification_id: Uuid) -> Result<Notification, AppError> {
        let client = self.pool.get().await?;
        let row = client
            .query_one(
                "SELECT * FROM notifikasi.notifications WHERE id = $1",
                &[&notification_id],
            )
            .await?;
        let notif = Notification::from_row(&row);
        Ok(notif)
    }
}
