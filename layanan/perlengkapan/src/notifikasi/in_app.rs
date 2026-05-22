use super::error::AppError;
use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// In-app notification types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NotificationType {
    WorkflowStateChange {
        entity_id: Uuid,
        entity_type: String,
        from_state: String,
        to_state: String,
    },
    DocumentReady {
        document_id: Uuid,
        document_type: String,
    },
    SLABreach {
        entity_id: Uuid,
        current_state: String,
        elapsed_minutes: i64,
    },
    IzinExpiry {
        izin_id: Uuid,
        days_until_expiry: i32,
    },
}

/// Notification priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Urgent,
}

impl NotificationPriority {
    pub fn as_str(&self) -> &str {
        match self {
            NotificationPriority::Low => "low",
            NotificationPriority::Normal => "normal",
            NotificationPriority::High => "high",
            NotificationPriority::Urgent => "urgent",
        }
    }
}

/// Notification category for visual styling
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationCategory {
    Info,
    Warning,
    Error,
    Success,
    System,
}

impl NotificationCategory {
    pub fn as_str(&self) -> &str {
        match self {
            NotificationCategory::Info => "info",
            NotificationCategory::Warning => "warning",
            NotificationCategory::Error => "error",
            NotificationCategory::Success => "success",
            NotificationCategory::System => "system",
        }
    }
}

/// In-app notification model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InAppNotification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub notification_type: String,
    pub title: String,
    pub message: String,
    pub priority: String,
    pub category: String,
    pub action_url: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub read: bool,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// In-app notification channel service
#[derive(Clone)]
pub struct InAppNotificationChannel {
    pool: Pool,
}

impl InAppNotificationChannel {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// Send an in-app notification to a user
    pub async fn send(
        &self,
        user_id: Uuid,
        notification_type: NotificationType,
        priority: NotificationPriority,
    ) -> Result<Uuid, AppError> {
        let notification_id = Uuid::new_v4();

        let (title, message, category, action_url) = self.format_notification(&notification_type);
        let notification_type_str = self.notification_type_to_string(&notification_type);
        let metadata = serde_json::to_value(&notification_type)?;

        let query = r#"
            INSERT INTO notifikasi.in_app_notifications
            (id, user_id, notification_type, title, message, priority, category, action_url, metadata, read, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, false, NOW())
        "#;

        self.pool
            .get()
            .await?
            .execute(
                query,
                &[
                    &notification_id,
                    &user_id,
                    &notification_type_str,
                    &title,
                    &message,
                    &priority.as_str(),
                    &category.as_str(),
                    &action_url,
                    &metadata,
                ],
            )
            .await?;

        tracing::info!(
            "In-app notification {} sent to user {}",
            notification_id,
            user_id
        );

        Ok(notification_id)
    }

    /// Get unread notification count for a user
    pub async fn get_unread_count(&self, user_id: Uuid) -> Result<i64, AppError> {
        let query = r#"
            SELECT COUNT(*) as count
            FROM notifikasi.in_app_notifications
            WHERE user_id = $1 AND read = false
        "#;

        let client = self.pool.get().await?;
        let row = client.query_one(query, &[&user_id]).await?;
        let count: i64 = row.get("count");

        Ok(count)
    }

    /// Mark a notification as read
    pub async fn mark_as_read(&self, notification_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let query = r#"
            UPDATE notifikasi.in_app_notifications
            SET read = true, read_at = NOW()
            WHERE id = $1 AND user_id = $2
        "#;

        self.pool
            .get()
            .await?
            .execute(query, &[&notification_id, &user_id])
            .await?;

        tracing::info!(
            "Notification {} marked as read for user {}",
            notification_id,
            user_id
        );

        Ok(())
    }

    /// Mark all notifications as read for a user
    pub async fn mark_all_as_read(&self, user_id: Uuid) -> Result<u64, AppError> {
        let query = r#"
            UPDATE notifikasi.in_app_notifications
            SET read = true, read_at = NOW()
            WHERE user_id = $1 AND read = false
        "#;

        let count = self.pool.get().await?.execute(query, &[&user_id]).await?;

        tracing::info!(
            "Marked {} notifications as read for user {}",
            count,
            user_id
        );

        Ok(count)
    }

    /// Get notifications for a user with pagination
    pub async fn get_notifications(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
        unread_only: bool,
    ) -> Result<Vec<InAppNotification>, AppError> {
        let query = if unread_only {
            r#"
                SELECT id, user_id, notification_type, title, message, priority, category,
                       action_url, metadata, read, read_at, created_at, expires_at
                FROM notifikasi.in_app_notifications
                WHERE user_id = $1 AND read = false
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
            "#
        } else {
            r#"
                SELECT id, user_id, notification_type, title, message, priority, category,
                       action_url, metadata, read, read_at, created_at, expires_at
                FROM notifikasi.in_app_notifications
                WHERE user_id = $1
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
            "#
        };

        let client = self.pool.get().await?;
        let rows = client.query(query, &[&user_id, &limit, &offset]).await?;

        let notifications = rows
            .into_iter()
            .map(|row| InAppNotification {
                id: row.get("id"),
                user_id: row.get("user_id"),
                notification_type: row.get("notification_type"),
                title: row.get("title"),
                message: row.get("message"),
                priority: row.get("priority"),
                category: row.get("category"),
                action_url: row.get("action_url"),
                metadata: row.get("metadata"),
                read: row.get("read"),
                read_at: row.get("read_at"),
                created_at: row.get("created_at"),
                expires_at: row.get("expires_at"),
            })
            .collect();

        Ok(notifications)
    }

    /// Delete old read notifications (cleanup)
    pub async fn cleanup_old_notifications(&self, days: i32) -> Result<u64, AppError> {
        let query = r#"
            DELETE FROM notifikasi.in_app_notifications
            WHERE read = true AND read_at < NOW() - INTERVAL '1 day' * $1
        "#;

        let count = self.pool.get().await?.execute(query, &[&days]).await?;

        tracing::info!("Cleaned up {} old notifications", count);

        Ok(count)
    }

    /// Format notification content based on type
    fn format_notification(
        &self,
        notification_type: &NotificationType,
    ) -> (String, String, NotificationCategory, Option<String>) {
        match notification_type {
            NotificationType::WorkflowStateChange {
                entity_type,
                to_state,
                entity_id,
                ..
            } => {
                let title = format!("{} Status Changed", entity_type);
                let message = format!("Status changed to: {}", to_state);
                let action_url = Some(format!("/workflow/{}", entity_id));
                (title, message, NotificationCategory::Info, action_url)
            }
            NotificationType::DocumentReady {
                document_type,
                document_id,
            } => {
                let title = "Document Ready".to_string();
                let message = format!("Your {} is ready for download", document_type);
                let action_url = Some(format!("/documents/{}", document_id));
                (title, message, NotificationCategory::Success, action_url)
            }
            NotificationType::SLABreach {
                current_state,
                elapsed_minutes,
                entity_id,
            } => {
                let title = "SLA Breach".to_string();
                let message = format!(
                    "Item in {} state has exceeded SLA by {} minutes",
                    current_state, elapsed_minutes
                );
                let action_url = Some(format!("/workflow/{}", entity_id));
                (title, message, NotificationCategory::Warning, action_url)
            }
            NotificationType::IzinExpiry {
                days_until_expiry,
                izin_id,
            } => {
                let title = "Izin Expiring Soon".to_string();
                let message = format!("Izin will expire in {} days", days_until_expiry);
                let action_url = Some(format!("/izin/{}", izin_id));
                let category = if *days_until_expiry <= 7 {
                    NotificationCategory::Warning
                } else {
                    NotificationCategory::Info
                };
                (title, message, category, action_url)
            }
        }
    }

    /// Convert notification type to string
    fn notification_type_to_string(&self, notification_type: &NotificationType) -> String {
        match notification_type {
            NotificationType::WorkflowStateChange { .. } => "workflow_state_change".to_string(),
            NotificationType::DocumentReady { .. } => "document_ready".to_string(),
            NotificationType::SLABreach { .. } => "sla_breach".to_string(),
            NotificationType::IzinExpiry { .. } => "izin_expiry".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_priority_as_str() {
        assert_eq!(NotificationPriority::Low.as_str(), "low");
        assert_eq!(NotificationPriority::Normal.as_str(), "normal");
        assert_eq!(NotificationPriority::High.as_str(), "high");
        assert_eq!(NotificationPriority::Urgent.as_str(), "urgent");
    }

    #[test]
    fn test_notification_category_as_str() {
        assert_eq!(NotificationCategory::Info.as_str(), "info");
        assert_eq!(NotificationCategory::Warning.as_str(), "warning");
        assert_eq!(NotificationCategory::Error.as_str(), "error");
        assert_eq!(NotificationCategory::Success.as_str(), "success");
        assert_eq!(NotificationCategory::System.as_str(), "system");
    }
}
