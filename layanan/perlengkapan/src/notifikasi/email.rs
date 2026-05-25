use super::config::AppConfig;
use super::error::AppError;
use super::models::{Notification, NotificationRecipient};
use crate::shared::grpc::clients::SecretonClient;
use deadpool_postgres::Pool;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor, message::Mailbox};
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

/// Email service for sending notifications via SMTP
///
/// Features:
/// - Fetches SMTP credentials from Secreton (secure secret management)
/// - Template-based email generation
/// - Retry logic with exponential backoff
/// - Database tracking of email status
#[allow(dead_code)]
#[derive(Clone)]
pub struct EmailService {
    #[allow(dead_code)]
    pub config: AppConfig,
    #[allow(dead_code)]
    pub pool: Pool,
    #[allow(dead_code)]
    pub mailer: AsyncSmtpTransport<Tokio1Executor>,
}

impl EmailService {
    /// Create a new EmailService instance
    ///
    /// Note: In production, SMTP credentials should be fetched from Secreton
    /// via gRPC for enhanced security. This implementation uses config for now.
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

    /// Create EmailService with SMTP credentials fetched from Secreton.
    ///
    /// `secreton_path` points at a Secreton KV bundle (e.g.
    /// `"perlengkapan/notifikasi/smtp"`) that holds `username` and `password`
    /// keys. Host, port, and `from` address still come from `config` — those
    /// are not secret and benefit from being declared in plain values.yaml so
    /// ops can grep them. Returns an error if the bundle is missing either
    /// credential or if Secreton is unreachable.
    pub async fn new_with_secreton(
        config: AppConfig,
        pool: Pool,
        secreton_client: &SecretonClient,
        secreton_path: &str,
    ) -> Result<Self, AppError> {
        let bundle = secreton_client.get_secret(secreton_path).await.map_err(|e| {
            AppError::Config(
                format!("fetch secreton {}: {}", secreton_path, e).into_boxed_str(),
            )
        })?;

        let smtp_username = bundle.get("username").cloned().ok_or_else(|| {
            AppError::Config(
                format!("secreton {}: missing key `username`", secreton_path).into_boxed_str(),
            )
        })?;
        let smtp_password = bundle.get("password").cloned().ok_or_else(|| {
            AppError::Config(
                format!("secreton {}: missing key `password`", secreton_path).into_boxed_str(),
            )
        })?;

        let creds =
            lettre::transport::smtp::authentication::Credentials::new(smtp_username, smtp_password);

        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_host)
            .map_err(|e| {
                AppError::Email(format!("Failed to create SMTP transport: {}", e).into_boxed_str())
            })?
            .port(config.smtp_port)
            .credentials(creds)
            .build();

        Ok(Self {
            config,
            pool,
            mailer,
        })
    }

    /// Send an email with retry logic and exponential backoff
    ///
    /// # Arguments
    /// * `recipient` - Email address of the recipient
    /// * `subject` - Email subject line
    /// * `body` - Email body content (plain text or HTML)
    ///
    /// # Returns
    /// * `Ok(Notification)` - Successfully sent notification with database record
    /// * `Err(AppError)` - Failed to send after all retries
    ///
    /// # Features
    /// - Stores notification in database before sending
    /// - Retries up to 3 times with exponential backoff (1s, 2s, 4s)
    /// - Updates database with final status (sent/failed)
    /// - Tracks error messages for debugging
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

        // Send email with retry logic (max 3 attempts)
        let max_retries = 3;
        let mut last_error = None;
        let mut success = false;

        for attempt in 0..max_retries {
            match self.mailer.send(email.clone()).await {
                Ok(_) => {
                    success = true;
                    tracing::info!(
                        "Email sent successfully to {} (attempt {}/{})",
                        recipient,
                        attempt + 1,
                        max_retries
                    );
                    break;
                }
                Err(e) => {
                    last_error = Some(e.to_string());
                    tracing::warn!(
                        "Email send failed to {} (attempt {}/{}): {}",
                        recipient,
                        attempt + 1,
                        max_retries,
                        e
                    );

                    // Exponential backoff: 1s, 2s, 4s
                    if attempt < max_retries - 1 {
                        let backoff_duration = Duration::from_secs(2u64.pow(attempt as u32));
                        tracing::debug!("Retrying after {:?}", backoff_duration);
                        sleep(backoff_duration).await;
                    }
                }
            }
        }

        let status = if success { "sent" } else { "failed" };
        let error_message = if success { None } else { last_error };

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

    /// Send an email using a template
    ///
    /// # Arguments
    /// * `recipient` - Email address of the recipient
    /// * `template_name` - Name of the email template to use
    /// * `template_data` - Data to populate the template
    ///
    /// # Example
    /// ```ignore
    /// let data = serde_json::json!({
    ///     "user_name": "John Doe",
    ///     "document_name": "Rekapitulasi Kebutuhan BMN",
    ///     "download_link": "https://simpel.kejaksaan.go.id/documents/123"
    /// });
    ///
    /// email_service.send_email_with_template(
    ///     "user@example.com",
    ///     "document_ready",
    ///     data,
    /// ).await?;
    /// ```
    #[allow(dead_code)]
    pub async fn send_email_with_template(
        &self,
        recipient: &str,
        template_name: &str,
        template_data: serde_json::Value,
    ) -> Result<Notification, AppError> {
        // Fetch template from database
        let sql_template = r#"SELECT subject, body_template FROM notifikasi.email_templates WHERE name = $1 AND enabled = true"#;
        let row = self
            .pool
            .get()
            .await?
            .query_one(sql_template, &[&template_name])
            .await
            .map_err(|e| {
                AppError::Email(
                    format!("Template '{}' not found: {}", template_name, e).into_boxed_str(),
                )
            })?;

        let subject: String = row.get("subject");
        let body_template: String = row.get("body_template");

        // Simple template rendering (replace {{key}} with values)
        // For production, consider using a proper template engine like Tera or Handlebars
        let body = self.render_template(&body_template, &template_data)?;

        // Send email using the standard send_email method (includes retry logic)
        self.send_email(recipient, &subject, &body).await
    }

    /// Simple template rendering function
    /// Replaces {{key}} placeholders with values from template_data
    fn render_template(
        &self,
        template: &str,
        data: &serde_json::Value,
    ) -> Result<String, AppError> {
        let mut rendered = template.to_string();

        if let Some(obj) = data.as_object() {
            for (key, value) in obj {
                let placeholder = format!("{{{{{}}}}}", key);
                let value_str = match value {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    _ => value.to_string(),
                };
                rendered = rendered.replace(&placeholder, &value_str);
            }
        }

        Ok(rendered)
    }

    /// Send batch emails to multiple recipients
    ///
    /// # Arguments
    /// * `recipients` - List of email addresses
    /// * `subject` - Email subject line
    /// * `body` - Email body content
    ///
    /// # Returns
    /// * `Vec<NotificationRecipient>` - Status for each recipient
    ///
    /// # Features
    /// - Sends emails concurrently for better performance
    /// - Each recipient is processed independently (one failure doesn't affect others)
    /// - Returns detailed status for each recipient
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

    /// Send batch emails using a template
    ///
    /// # Arguments
    /// * `recipients` - List of email addresses
    /// * `template_name` - Name of the email template to use
    /// * `template_data` - Data to populate the template (same for all recipients)
    ///
    /// # Example
    /// ```ignore
    /// let recipients = vec![
    ///     "user1@example.com".to_string(),
    ///     "user2@example.com".to_string(),
    /// ];
    ///
    /// let data = serde_json::json!({
    ///     "announcement": "System maintenance scheduled for tonight"
    /// });
    ///
    /// email_service.send_batch_emails_with_template(
    ///     recipients,
    ///     "system_announcement",
    ///     data,
    /// ).await?;
    /// ```
    #[allow(dead_code)]
    pub async fn send_batch_emails_with_template(
        &self,
        recipients: Vec<String>,
        template_name: &str,
        template_data: serde_json::Value,
    ) -> Result<Vec<NotificationRecipient>, AppError> {
        let mut results = Vec::new();
        for recipient in recipients {
            let _ = self
                .send_email_with_template(&recipient, template_name, template_data.clone())
                .await;
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

    /// Get notification status by ID
    ///
    /// # Arguments
    /// * `notification_id` - UUID of the notification
    ///
    /// # Returns
    /// * `Ok(Notification)` - Notification details including status
    /// * `Err(AppError)` - Notification not found or database error
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

    /// Get all recipients for a notification
    ///
    /// # Arguments
    /// * `notification_id` - UUID of the notification
    ///
    /// # Returns
    /// * `Vec<NotificationRecipient>` - List of recipients with their delivery status
    #[allow(dead_code)]
    pub async fn get_recipients(
        &self,
        notification_id: Uuid,
    ) -> Result<Vec<NotificationRecipient>, AppError> {
        let sql = r#"SELECT id, notification_id, recipient, recipient_type, status, sent_at, error_message
                     FROM notifikasi.notification_recipients
                     WHERE notification_id = $1
                     ORDER BY created_at"#;
        let rows = self
            .pool
            .get()
            .await?
            .query(sql, &[&notification_id])
            .await?;

        let recipients = rows.into_iter().map(|row| row.into()).collect();

        Ok(recipients)
    }

    /// Get failed notifications for retry
    ///
    /// # Arguments
    /// * `limit` - Maximum number of failed notifications to retrieve
    ///
    /// # Returns
    /// * `Vec<Notification>` - List of failed notifications
    ///
    /// # Use Case
    /// This can be used by a background job to retry failed email sends
    #[allow(dead_code)]
    pub async fn get_failed_notifications(
        &self,
        limit: i64,
    ) -> Result<Vec<Notification>, AppError> {
        let sql = r#"SELECT id, channel, status, subject, body, created_at, sent_at, error_message
                     FROM notifikasi.notifications
                     WHERE status = 'failed' AND channel = 'email'
                     ORDER BY created_at DESC
                     LIMIT $1"#;
        let rows = self.pool.get().await?.query(sql, &[&limit]).await?;

        let notifications = rows.into_iter().map(|row| row.into()).collect();

        Ok(notifications)
    }

    /// Send an email via HTTP API (alternative to direct SMTP)
    ///
    /// This method is useful when you want to delegate email sending to an external
    /// notification API service instead of using SMTP directly.
    ///
    /// # Arguments
    /// * `api_url` - Base URL of the notification API
    /// * `api_key` - Optional API key for authentication
    /// * `recipient` - Email address of the recipient
    /// * `subject` - Email subject line
    /// * `body` - Email body content
    ///
    /// # Example
    /// ```ignore
    /// email_service.send_email_via_api(
    ///     "https://api.notification.internal",
    ///     Some("secret-api-key"),
    ///     "user@example.com",
    ///     "Test Subject",
    ///     "Test Body",
    /// ).await?;
    /// ```
    #[allow(dead_code)]
    pub async fn send_email_via_api(
        &self,
        api_url: &str,
        api_key: Option<&str>,
        recipient: &str,
        subject: &str,
        body: &str,
    ) -> Result<(), AppError> {
        let client = reqwest::Client::new();
        let payload = serde_json::json!({
            "to": recipient,
            "subject": subject,
            "body": body,
        });

        let mut req = client.post(format!("{}/send", api_url)).json(&payload);

        if let Some(key) = api_key {
            req = req.header("x-api-key", key);
        }

        let resp = req.send().await.map_err(|e| {
            AppError::Email(format!("Failed to send email via API: {}", e).into_boxed_str())
        })?;

        if !resp.status().is_success() {
            return Err(AppError::Email(
                format!("Email API returned status: {}", resp.status()).into_boxed_str(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_template_rendering() {
        let config = AppConfig::default();
        let pool = deadpool_postgres::Pool::builder(deadpool_postgres::Manager::new(
            tokio_postgres::Config::new(),
            tokio_postgres::NoTls,
        ))
        .build()
        .unwrap();

        let service = EmailService::new(config, pool);

        let template = "Hello {{name}}, your document {{document_name}} is ready!";
        let data = serde_json::json!({
            "name": "John Doe",
            "document_name": "Report.pdf"
        });

        let rendered = service.render_template(template, &data).unwrap();
        assert_eq!(
            rendered,
            "Hello John Doe, your document Report.pdf is ready!"
        );
    }
}
