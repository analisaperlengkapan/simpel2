use crate::config::AppConfig;
use crate::error::AppError;
use crate::models::{Notification, NotificationRecipient};
use deadpool_postgres::Pool;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor, message::Mailbox};
use uuid::Uuid;

#[allow(dead_code)]
pub struct EmailService {
    pub config: AppConfig,
    pub pool: Pool,
    pub mailer: AsyncSmtpTransport<Tokio1Executor>,
}

impl EmailService {
    #[allow(dead_code)]
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

    #[allow(dead_code)]
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
            .map_err(|e| AppError::Email(e.to_string().into_boxed_str()))?;
        let notif_id = Uuid::new_v4();
        // Insert notification DB (pending)
        let sql = r#"INSERT INTO notifikasi.notifications (id, channel, status, subject, body, created_at)
            VALUES ($1, 'email', 'pending', $2, $3, NOW()) RETURNING id, channel, status, subject, body, created_at, sent_at, error_message"#;
        let row = self
            .pool
            .get()
            .await?
            .query_one(sql, &[&notif_id, &subject, &body])
            .await?;
        let notif: Notification = row.into();
        // Insert recipient
        let sql_recipient = r#"INSERT INTO notifikasi.notification_recipients (id, notification_id, recipient, recipient_type, status)
            VALUES ($1, $2, $3, 'email', 'pending')"#;
        self.pool
            .get()
            .await?
            .execute(sql_recipient, &[&Uuid::new_v4(), &notif_id, &recipient])
            .await?;
        // Kirim email
        let result = self.mailer.send(email).await;
        let status = if result.is_ok() { "sent" } else { "failed" };
        let error_message = result.as_ref().err().map(|e| e.to_string());
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
                &[&status, &error_message, &notif_id, &recipient],
            )
            .await?;
        Ok(notif)
    }

    #[allow(dead_code)]
    pub async fn send_batch_emails(
        &self,
        recipients: Vec<String>,
        subject: &str,
        body: &str,
    ) -> Result<Vec<NotificationRecipient>, AppError> {
        let mut results = Vec::new();
        for recipient in recipients {
            let _ = self.send_email(&recipient, subject, body).await; // Ignore error per recipient
            let sql_recipient_status = r#"SELECT id, notification_id, recipient, recipient_type, status, sent_at, error_message FROM notifikasi.notification_recipients WHERE recipient = $1 ORDER BY sent_at DESC LIMIT 1"#;
            let row = self
                .pool
                .get()
                .await?
                .query_one(sql_recipient_status, &[&recipient])
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
