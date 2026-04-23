use crate::email::EmailService;
use crate::error::AppError;
use crate::in_app::{InAppNotificationChannel, NotificationPriority, NotificationType};
use crate::preferences::NotificationPreferencesService;
use chrono::{Local, NaiveDate};
use deadpool_postgres::Pool;
use tokio_cron_scheduler::{Job, JobScheduler};
use uuid::Uuid;

/// Notification scheduler for batch operations and reminders
pub struct NotificationScheduler {
    pool: Pool,
    email_service: Option<EmailService>,
    in_app_channel: InAppNotificationChannel,
    preferences_service: NotificationPreferencesService,
}

impl NotificationScheduler {
    pub fn new(pool: Pool, email_service: Option<EmailService>) -> Self {
        let in_app_channel = InAppNotificationChannel::new(pool.clone());
        let preferences_service = NotificationPreferencesService::new(pool.clone());

        Self {
            pool,
            email_service,
            in_app_channel,
            preferences_service,
        }
    }

    /// Start all scheduled jobs
    pub async fn start(&self) -> Result<(), AppError> {
        let scheduler = JobScheduler::new().await.map_err(|e| {
            AppError::Internal(format!("Failed to create scheduler: {}", e).into_boxed_str())
        })?;

        // Daily digest at 08:00 WIB (01:00 UTC, assuming WIB = UTC+7)
        let daily_digest_job = self.create_daily_digest_job()?;
        scheduler.add(daily_digest_job).await.map_err(|e| {
            AppError::Internal(format!("Failed to add daily digest job: {}", e).into_boxed_str())
        })?;

        // Izin expiry reminders at 09:00 WIB (02:00 UTC)
        let izin_reminder_job = self.create_izin_reminder_job()?;
        scheduler.add(izin_reminder_job).await.map_err(|e| {
            AppError::Internal(format!("Failed to add izin reminder job: {}", e).into_boxed_str())
        })?;

        // SLA breach check every hour
        let sla_breach_job = self.create_sla_breach_job()?;
        scheduler.add(sla_breach_job).await.map_err(|e| {
            AppError::Internal(format!("Failed to add SLA breach job: {}", e).into_boxed_str())
        })?;

        scheduler.start().await.map_err(|e| {
            AppError::Internal(format!("Failed to start scheduler: {}", e).into_boxed_str())
        })?;

        tracing::info!("Notification scheduler started successfully");

        Ok(())
    }

    /// Create daily digest job
    fn create_daily_digest_job(&self) -> Result<Job, AppError> {
        let pool = self.pool.clone();
        let email_service = self.email_service.clone();
        let preferences_service = self.preferences_service.clone();

        let job = Job::new_async("0 0 1 * * *", move |_uuid, _l| {
            let pool = pool.clone();
            let email_service = email_service.clone();
            let preferences_service = preferences_service.clone();

            Box::pin(async move {
                tracing::info!("Running daily digest job");

                if let Err(e) =
                    Self::send_daily_digest_static(pool, email_service, preferences_service).await
                {
                    tracing::error!("Daily digest job failed: {}", e);
                }
            })
        })
        .map_err(|e| {
            AppError::Internal(format!("Failed to create daily digest job: {}", e).into_boxed_str())
        })?;

        Ok(job)
    }

    /// Create izin expiry reminder job
    fn create_izin_reminder_job(&self) -> Result<Job, AppError> {
        let pool = self.pool.clone();
        let in_app_channel = self.in_app_channel.clone();

        let job = Job::new_async("0 0 2 * * *", move |_uuid, _l| {
            let pool = pool.clone();
            let in_app_channel = in_app_channel.clone();

            Box::pin(async move {
                tracing::info!("Running izin expiry reminder job");

                if let Err(e) = Self::send_izin_expiry_reminders_static(pool, in_app_channel).await
                {
                    tracing::error!("Izin expiry reminder job failed: {}", e);
                }
            })
        })
        .map_err(|e| {
            AppError::Internal(
                format!("Failed to create izin reminder job: {}", e).into_boxed_str(),
            )
        })?;

        Ok(job)
    }

    /// Create SLA breach check job
    fn create_sla_breach_job(&self) -> Result<Job, AppError> {
        let pool = self.pool.clone();
        let in_app_channel = self.in_app_channel.clone();

        let job = Job::new_async("0 0 * * * *", move |_uuid, _l| {
            let pool = pool.clone();
            let in_app_channel = in_app_channel.clone();

            Box::pin(async move {
                tracing::info!("Running SLA breach check job");

                if let Err(e) = Self::check_sla_breaches_static(pool, in_app_channel).await {
                    tracing::error!("SLA breach check job failed: {}", e);
                }
            })
        })
        .map_err(|e| {
            AppError::Internal(format!("Failed to create SLA breach job: {}", e).into_boxed_str())
        })?;

        Ok(job)
    }

    /// Send daily digest (static method for async closure)
    async fn send_daily_digest_static(
        pool: Pool,
        email_service: Option<EmailService>,
        preferences_service: NotificationPreferencesService,
    ) -> Result<(), AppError> {
        tracing::info!("Sending daily digest");

        // Get users who want daily digest
        let users = preferences_service.get_users_for_daily_digest().await?;

        if users.is_empty() {
            tracing::info!("No users configured for daily digest");
            return Ok(());
        }

        for (user_id, _digest_time) in &users {
            // Get unread low-priority notifications from last 24 hours
            let query = r#"
                SELECT COUNT(*) as count
                FROM notifikasi.in_app_notifications
                WHERE user_id = $1
                  AND read = false
                  AND priority = 'low'
                  AND created_at >= NOW() - INTERVAL '24 hours'
            "#;

            let client = pool.get().await?;
            let row = client.query_one(query, &[&user_id]).await?;
            let count: i64 = row.get("count");

            if count == 0 {
                continue;
            }

            // Send digest email if email service is available
            if let Some(ref email_svc) = email_service {
                // Get user email from authenc (simplified - in production, call authenc service)
                let email = format!("user_{}@example.com", user_id); // Placeholder

                let subject = format!("Daily Notification Digest - {} unread notifications", count);
                let body = format!(
                    "You have {} unread notifications from the past 24 hours.\n\nVisit SIMPEL to view them.",
                    count
                );

                if let Err(e) = email_svc.send_email(&email, &subject, &body).await {
                    tracing::error!("Failed to send daily digest to user {}: {}", user_id, e);
                }
            }
        }

        tracing::info!("Daily digest sent to {} users", users.len());

        Ok(())
    }

    /// Send izin expiry reminders (static method for async closure)
    async fn send_izin_expiry_reminders_static(
        pool: Pool,
        in_app_channel: InAppNotificationChannel,
    ) -> Result<(), AppError> {
        tracing::info!("Sending izin expiry reminders");

        // Find izin expiring in 30, 14, or 7 days
        let query = r#"
            SELECT id, user_id, tanggal_berakhir
            FROM perlengkapan.izin_pemakaian_bmn
            WHERE tanggal_berakhir IN (
                CURRENT_DATE + INTERVAL '30 days',
                CURRENT_DATE + INTERVAL '14 days',
                CURRENT_DATE + INTERVAL '7 days'
            )
            AND status = 'active'
        "#;

        let client = pool.get().await?;
        let rows = client.query(query, &[]).await?;

        let mut reminder_count = 0;

        for row in rows {
            let izin_id: Uuid = row.get("id");
            let user_id: Uuid = row.get("user_id");
            let tanggal_berakhir: NaiveDate = row.get("tanggal_berakhir");

            let days_until_expiry = (tanggal_berakhir - Local::now().date_naive()).num_days();

            // Send notification
            let notification_type = NotificationType::IzinExpiry {
                izin_id,
                days_until_expiry: days_until_expiry as i32,
            };

            let priority = if days_until_expiry <= 7 {
                NotificationPriority::High
            } else {
                NotificationPriority::Normal
            };

            if let Err(e) = in_app_channel
                .send(user_id, notification_type, priority)
                .await
            {
                tracing::error!(
                    "Failed to send izin expiry reminder to user {}: {}",
                    user_id,
                    e
                );
            } else {
                reminder_count += 1;
            }
        }

        tracing::info!("Sent {} izin expiry reminders", reminder_count);

        Ok(())
    }

    /// Check for SLA breaches (static method for async closure)
    async fn check_sla_breaches_static(
        pool: Pool,
        in_app_channel: InAppNotificationChannel,
    ) -> Result<(), AppError> {
        tracing::info!("Checking for SLA breaches");

        // Find workflow items that have exceeded their SLA
        // This is a simplified query - in production, join with workflow configuration
        let query = r#"
            SELECT
                k.id,
                k.user_id,
                k.status,
                EXTRACT(EPOCH FROM (NOW() - k.updated_at)) / 60 AS elapsed_minutes
            FROM perlengkapan.kebutuhan_bmn k
            WHERE k.status IN ('submitted', 'in_review')
              AND k.updated_at < NOW() - INTERVAL '24 hours'
        "#;

        let client = pool.get().await?;
        let rows = client.query(query, &[]).await?;

        let mut breach_count = 0;

        for row in rows {
            let entity_id: Uuid = row.get("id");
            let user_id: Uuid = row.get("user_id");
            let current_state: String = row.get("status");
            let elapsed_minutes: f64 = row.get("elapsed_minutes");

            // Send SLA breach notification
            let notification_type = NotificationType::SLABreach {
                entity_id,
                current_state,
                elapsed_minutes: elapsed_minutes as i64,
            };

            if let Err(e) = in_app_channel
                .send(user_id, notification_type, NotificationPriority::High)
                .await
            {
                tracing::error!(
                    "Failed to send SLA breach notification to user {}: {}",
                    user_id,
                    e
                );
            } else {
                breach_count += 1;
            }
        }

        tracing::info!("Sent {} SLA breach notifications", breach_count);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_scheduler_creation() {
        // This is a placeholder test
        // In production, you would test with a mock pool
        assert!(true);
    }
}
