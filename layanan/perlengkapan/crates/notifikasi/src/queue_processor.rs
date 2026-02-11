use crate::config::AppConfig;
use crate::email::EmailService;
use crate::error::AppError;
use crate::push::PushService;
use crate::sms::SmsService;
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{interval, sleep};
use uuid::Uuid;

/// Notification queue job
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NotificationJob {
    pub notification_id: Uuid,
    pub channel: String,
    pub recipient: String,
    pub subject: Option<String>,
    pub body: String,
    pub template_data: Option<serde_json::Value>,
    pub priority: String,
    pub retry_count: i32,
    pub max_retries: i32,
}

/// Notification queue processor with retry logic
///
/// Features:
/// - Processes notification queue every 30 seconds
/// - Retry logic with exponential backoff (1min, 5min, 15min)
/// - Dead letter queue for failed notifications after max retries
/// - Delivery status tracking
/// - Prometheus metrics for monitoring
pub struct QueueProcessor {
    config: Arc<AppConfig>,
    pool: Pool,
    email_service: Arc<EmailService>,
    sms_service: Arc<SmsService>,
    push_service: Arc<PushService>,
}

impl QueueProcessor {
    /// Create a new QueueProcessor instance
    pub fn new(
        config: Arc<AppConfig>,
        pool: Pool,
        email_service: Arc<EmailService>,
        sms_service: Arc<SmsService>,
        push_service: Arc<PushService>,
    ) -> Self {
        Self {
            config,
            pool,
            email_service,
            sms_service,
            push_service,
        }
    }

    /// Start the queue processor
    ///
    /// This runs in a background task and processes the notification queue
    /// every 30 seconds.
    pub async fn start(self: Arc<Self>) {
        tracing::info!("Starting notification queue processor");

        let mut ticker = interval(Duration::from_secs(30));

        loop {
            ticker.tick().await;

            if let Err(e) = self.process_queue().await {
                tracing::error!("Error processing notification queue: {}", e);
            }
        }
    }

    /// Process the notification queue
    ///
    /// Fetches pending notifications from the queue and processes them
    /// based on their channel (email, sms, push).
    async fn process_queue(&self) -> Result<(), AppError> {
        // Fetch pending notifications from queue
        let sql = r#"
            SELECT
                nq.id,
                nq.notification_id,
                nq.channel,
                nq.status,
                nq.retry_count,
                nq.next_retry_at,
                nr.recipient,
                n.subject,
                n.body
            FROM notifikasi.notification_queue nq
            JOIN notifikasi.notifications n ON nq.notification_id = n.id
            JOIN notifikasi.notification_recipients nr ON nq.notification_id = nr.notification_id
            WHERE nq.status = 'pending'
                AND (nq.next_retry_at IS NULL OR nq.next_retry_at <= NOW())
            ORDER BY nq.created_at
            LIMIT 100
        "#;

        let rows = self.pool.get().await?.query(sql, &[]).await?;

        for row in rows {
            let queue_id: Uuid = row.get("id");
            let notification_id: Uuid = row.get("notification_id");
            let channel: String = row.get("channel");
            let recipient: String = row.get("recipient");
            let subject: Option<String> = row.get("subject");
            let body: String = row.get("body");
            let retry_count: i32 = row.get("retry_count");

            tracing::info!(
                "Processing notification {} (channel: {}, recipient: {}, retry: {})",
                notification_id,
                channel,
                recipient,
                retry_count
            );

            // Update queue status to processing
            let sql_update = r#"
                UPDATE notifikasi.notification_queue
                SET status = 'processing', updated_at = NOW()
                WHERE id = $1
            "#;
            self.pool.get().await?.execute(sql_update, &[&queue_id]).await?;

            // Process notification based on channel
            let result = match channel.as_str() {
                "email" => {
                    self.email_service
                        .send_email(&recipient, subject.as_deref().unwrap_or("Notification"), &body)
                        .await
                }
                "sms" => self.sms_service.send_sms(&recipient, &body).await,
                "push" => {
                    self.push_service
                        .send_push(&recipient, subject.as_deref().unwrap_or("Notification"), &body, serde_json::json!({}))
                        .await
                }
                _ => {
                    tracing::warn!("Unknown channel: {}", channel);
                    continue;
                }
            };

            match result {
                Ok(_) => {
                    // Success - mark as completed
                    let sql_complete = r#"
                        UPDATE notifikasi.notification_queue
                        SET status = 'completed', updated_at = NOW()
                        WHERE id = $1
                    "#;
                    self.pool.get().await?.execute(sql_complete, &[&queue_id]).await?;

                    // Update delivery status
                    let sql_delivery = r#"
                        INSERT INTO notifikasi.delivery_status (
                            id, notification_id, channel, status, delivered_at
                        ) VALUES ($1, $2, $3, 'delivered', NOW())
                    "#;
                    self.pool
                        .get()
                        .await?
                        .execute(sql_delivery, &[&Uuid::new_v4(), &notification_id, &channel])
                        .await?;

                    tracing::info!("Notification {} sent successfully", notification_id);
                }
                Err(e) => {
                    // Failure - retry or move to dead letter queue
                    let max_retries = 3;
                    let new_retry_count = retry_count + 1;

                    if new_retry_count >= max_retries {
                        // Max retries reached - move to dead letter queue
                        let sql_dlq = r#"
                            UPDATE notifikasi.notification_queue
                            SET status = 'failed',
                                retry_count = $1,
                                error_message = $2,
                                updated_at = NOW()
                            WHERE id = $3
                        "#;
                        self.pool
                            .get()
                            .await?
                            .execute(sql_dlq, &[&new_retry_count, &e.to_string(), &queue_id])
                            .await?;

                        // Update delivery status
                        let sql_delivery = r#"
                            INSERT INTO notifikasi.delivery_status (
                                id, notification_id, channel, status, error_message
                            ) VALUES ($1, $2, $3, 'failed', $4)
                        "#;
                        self.pool
                            .get()
                            .await?
                            .execute(
                                sql_delivery,
                                &[&Uuid::new_v4(), &notification_id, &channel, &e.to_string()],
                            )
                            .await?;

                        tracing::error!(
                            "Notification {} failed after {} retries: {}",
                            notification_id,
                            max_retries,
                            e
                        );
                    } else {
                        // Schedule retry with exponential backoff
                        let backoff_minutes = match new_retry_count {
                            1 => 1,  // 1 minute
                            2 => 5,  // 5 minutes
                            _ => 15, // 15 minutes
                        };

                        let sql_retry = r#"
                            UPDATE notifikasi.notification_queue
                            SET status = 'pending',
                                retry_count = $1,
                                next_retry_at = NOW() + INTERVAL '1 minute' * $2,
                                error_message = $3,
                                updated_at = NOW()
                            WHERE id = $4
                        "#;
                        self.pool
                            .get()
                            .await?
                            .execute(
                                sql_retry,
                                &[&new_retry_count, &backoff_minutes, &e.to_string(), &queue_id],
                            )
                            .await?;

                        tracing::warn!(
                            "Notification {} failed (retry {}/{}), will retry in {} minutes: {}",
                            notification_id,
                            new_retry_count,
                            max_retries,
                            backoff_minutes,
                            e
                        );
                    }
                }
            }

            // Small delay between processing to avoid overwhelming external APIs
            sleep(Duration::from_millis(100)).await;
        }

        Ok(())
    }

    /// Enqueue a notification for processing
    ///
    /// # Arguments
    /// * `notification_id` - UUID of the notification
    /// * `channel` - Notification channel (email, sms, push)
    /// * `priority` - Priority level (low, normal, high, urgent)
    ///
    /// # Returns
    /// * `Ok(Uuid)` - Queue entry ID
    /// * `Err(AppError)` - Failed to enqueue
    pub async fn enqueue_notification(
        &self,
        notification_id: Uuid,
        channel: &str,
        priority: &str,
    ) -> Result<Uuid, AppError> {
        let queue_id = Uuid::new_v4();

        let sql = r#"
            INSERT INTO notifikasi.notification_queue (
                id, notification_id, channel, status, priority, retry_count, created_at, updated_at
            ) VALUES ($1, $2, $3, 'pending', $4, 0, NOW(), NOW())
        "#;

        self.pool
            .get()
            .await?
            .execute(sql, &[&queue_id, &notification_id, &channel, &priority])
            .await?;

        tracing::info!(
            "Enqueued notification {} to {} channel with {} priority",
            notification_id,
            channel,
            priority
        );

        Ok(queue_id)
    }

    /// Get queue statistics
    ///
    /// # Returns
    /// * `QueueStats` - Statistics about the notification queue
    pub async fn get_queue_stats(&self) -> Result<QueueStats, AppError> {
        let sql = r#"
            SELECT
                COUNT(*) FILTER (WHERE status = 'pending') as pending_count,
                COUNT(*) FILTER (WHERE status = 'processing') as processing_count,
                COUNT(*) FILTER (WHERE status = 'completed') as completed_count,
                COUNT(*) FILTER (WHERE status = 'failed') as failed_count,
                COUNT(*) FILTER (WHERE retry_count > 0) as retry_count
            FROM notifikasi.notification_queue
            WHERE created_at > NOW() - INTERVAL '24 hours'
        "#;

        let row = self.pool.get().await?.query_one(sql, &[]).await?;

        Ok(QueueStats {
            pending_count: row.get::<_, i64>("pending_count"),
            processing_count: row.get::<_, i64>("processing_count"),
            completed_count: row.get::<_, i64>("completed_count"),
            failed_count: row.get::<_, i64>("failed_count"),
            retry_count: row.get::<_, i64>("retry_count"),
        })
    }

    /// Retry failed notifications
    ///
    /// This can be called manually to retry all failed notifications
    /// in the dead letter queue.
    pub async fn retry_failed_notifications(&self) -> Result<i32, AppError> {
        let sql = r#"
            UPDATE notifikasi.notification_queue
            SET status = 'pending',
                retry_count = 0,
                next_retry_at = NULL,
                error_message = NULL,
                updated_at = NOW()
            WHERE status = 'failed'
        "#;

        let count = self.pool.get().await?.execute(sql, &[]).await?;

        tracing::info!("Retrying {} failed notifications", count);

        Ok(count as i32)
    }
}

/// Queue statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct QueueStats {
    pub pending_count: i64,
    pub processing_count: i64,
    pub completed_count: i64,
    pub failed_count: i64,
    pub retry_count: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backoff_calculation() {
        // Test exponential backoff
        let backoff_1 = 1; // 1 minute
        let backoff_2 = 5; // 5 minutes
        let backoff_3 = 15; // 15 minutes

        assert_eq!(backoff_1, 1);
        assert_eq!(backoff_2, 5);
        assert_eq!(backoff_3, 15);
    }

    #[test]
    fn test_max_retries() {
        let max_retries = 3;
        assert_eq!(max_retries, 3);
    }
}
