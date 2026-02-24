use crate::error::AppError;
use chrono::{DateTime, NaiveTime, Utc};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User notification preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub id: Uuid,
    pub user_id: Uuid,
    pub in_app_enabled: bool,
    pub email_enabled: bool,
    pub sms_enabled: bool,
    pub push_enabled: bool,
    pub batch_non_urgent: bool,
    pub daily_digest_time: NaiveTime,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to update notification preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePreferencesRequest {
    pub in_app_enabled: Option<bool>,
    pub email_enabled: Option<bool>,
    pub sms_enabled: Option<bool>,
    pub push_enabled: Option<bool>,
    pub batch_non_urgent: Option<bool>,
    pub daily_digest_time: Option<String>, // Format: "HH:MM:SS"
}

/// Notification preferences service
#[derive(Clone)]
pub struct NotificationPreferencesService {
    pool: Pool,
}

impl NotificationPreferencesService {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// Get user notification preferences (create default if not exists)
    pub async fn get_preferences(
        &self,
        user_id: Uuid,
    ) -> Result<NotificationPreferences, AppError> {
        let query = r#"
            SELECT id, user_id, in_app_enabled, email_enabled, sms_enabled, push_enabled,
                   batch_non_urgent, daily_digest_time, created_at, updated_at
            FROM notifikasi.user_notification_preferences
            WHERE user_id = $1
        "#;

        let client = self.pool.get().await?;

        match client.query_opt(query, &[&user_id]).await? {
            Some(row) => Ok(NotificationPreferences {
                id: row.get("id"),
                user_id: row.get("user_id"),
                in_app_enabled: row.get("in_app_enabled"),
                email_enabled: row.get("email_enabled"),
                sms_enabled: row.get("sms_enabled"),
                push_enabled: row.get("push_enabled"),
                batch_non_urgent: row.get("batch_non_urgent"),
                daily_digest_time: row.get("daily_digest_time"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }),
            None => {
                // Create default preferences
                self.create_default_preferences(user_id).await
            }
        }
    }

    /// Create default notification preferences for a user
    async fn create_default_preferences(
        &self,
        user_id: Uuid,
    ) -> Result<NotificationPreferences, AppError> {
        let query = r#"
            INSERT INTO notifikasi.user_notification_preferences
            (id, user_id, in_app_enabled, email_enabled, sms_enabled, push_enabled,
             batch_non_urgent, daily_digest_time, created_at, updated_at)
            VALUES ($1, $2, true, true, false, true, false, '08:00:00', NOW(), NOW())
            RETURNING id, user_id, in_app_enabled, email_enabled, sms_enabled, push_enabled,
                      batch_non_urgent, daily_digest_time, created_at, updated_at
        "#;

        let pref_id = Uuid::new_v4();
        let client = self.pool.get().await?;
        let row = client.query_one(query, &[&pref_id, &user_id]).await?;

        tracing::info!(
            "Created default notification preferences for user {}",
            user_id
        );

        Ok(NotificationPreferences {
            id: row.get("id"),
            user_id: row.get("user_id"),
            in_app_enabled: row.get("in_app_enabled"),
            email_enabled: row.get("email_enabled"),
            sms_enabled: row.get("sms_enabled"),
            push_enabled: row.get("push_enabled"),
            batch_non_urgent: row.get("batch_non_urgent"),
            daily_digest_time: row.get("daily_digest_time"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    /// Update user notification preferences
    pub async fn update_preferences(
        &self,
        user_id: Uuid,
        updates: UpdatePreferencesRequest,
    ) -> Result<NotificationPreferences, AppError> {
        // Ensure preferences exist
        let _ = self.get_preferences(user_id).await?;

        let mut query_parts: Vec<String> = vec![
            "UPDATE notifikasi.user_notification_preferences SET updated_at = NOW()".to_string(),
        ];
        let mut param_index = 1;
        let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Send + Sync>> = vec![];

        if let Some(in_app_enabled) = updates.in_app_enabled {
            query_parts.push(format!("in_app_enabled = ${}", param_index));
            params.push(Box::new(in_app_enabled));
            param_index += 1;
        }

        if let Some(email_enabled) = updates.email_enabled {
            query_parts.push(format!("email_enabled = ${}", param_index));
            params.push(Box::new(email_enabled));
            param_index += 1;
        }

        if let Some(sms_enabled) = updates.sms_enabled {
            query_parts.push(format!("sms_enabled = ${}", param_index));
            params.push(Box::new(sms_enabled));
            param_index += 1;
        }

        if let Some(push_enabled) = updates.push_enabled {
            query_parts.push(format!("push_enabled = ${}", param_index));
            params.push(Box::new(push_enabled));
            param_index += 1;
        }

        if let Some(batch_non_urgent) = updates.batch_non_urgent {
            query_parts.push(format!("batch_non_urgent = ${}", param_index));
            params.push(Box::new(batch_non_urgent));
            param_index += 1;
        }

        if let Some(daily_digest_time) = updates.daily_digest_time {
            let time = NaiveTime::parse_from_str(&daily_digest_time, "%H:%M:%S").map_err(|e| {
                AppError::Internal(format!("Invalid time format: {}", e).into_boxed_str())
            })?;
            query_parts.push(format!("daily_digest_time = ${}", param_index));
            params.push(Box::new(time));
            param_index += 1;
        }

        // Build the full query
        let mut query = query_parts.join(", ");
        query.push_str(&format!(" WHERE user_id = ${}", param_index));
        params.push(Box::new(user_id));

        query.push_str(" RETURNING id, user_id, in_app_enabled, email_enabled, sms_enabled, push_enabled, batch_non_urgent, daily_digest_time, created_at, updated_at");

        // Execute update
        let client = self.pool.get().await?;
        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let row = client.query_one(&query, &param_refs[..]).await?;

        tracing::info!("Updated notification preferences for user {}", user_id);

        Ok(NotificationPreferences {
            id: row.get("id"),
            user_id: row.get("user_id"),
            in_app_enabled: row.get("in_app_enabled"),
            email_enabled: row.get("email_enabled"),
            sms_enabled: row.get("sms_enabled"),
            push_enabled: row.get("push_enabled"),
            batch_non_urgent: row.get("batch_non_urgent"),
            daily_digest_time: row.get("daily_digest_time"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    /// Check if a specific channel is enabled for a user
    pub async fn is_channel_enabled(&self, user_id: Uuid, channel: &str) -> Result<bool, AppError> {
        let preferences = self.get_preferences(user_id).await?;

        let enabled = match channel {
            "in_app" => preferences.in_app_enabled,
            "email" => preferences.email_enabled,
            "sms" => preferences.sms_enabled,
            "push" => preferences.push_enabled,
            _ => false,
        };

        Ok(enabled)
    }

    /// Get all users who have a specific channel enabled
    pub async fn get_users_with_channel_enabled(
        &self,
        channel: &str,
    ) -> Result<Vec<Uuid>, AppError> {
        let column = match channel {
            "in_app" => "in_app_enabled",
            "email" => "email_enabled",
            "sms" => "sms_enabled",
            "push" => "push_enabled",
            _ => return Ok(vec![]),
        };

        let query = format!(
            "SELECT user_id FROM notifikasi.user_notification_preferences WHERE {} = true",
            column
        );

        let client = self.pool.get().await?;
        let rows = client.query(&query, &[]).await?;

        let user_ids = rows.iter().map(|row| row.get("user_id")).collect();

        Ok(user_ids)
    }

    /// Get users who want daily digest
    pub async fn get_users_for_daily_digest(&self) -> Result<Vec<(Uuid, NaiveTime)>, AppError> {
        let query = r#"
            SELECT user_id, daily_digest_time
            FROM notifikasi.user_notification_preferences
            WHERE batch_non_urgent = true AND email_enabled = true
        "#;

        let client = self.pool.get().await?;
        let rows = client.query(query, &[]).await?;

        let users = rows
            .iter()
            .map(|row| (row.get("user_id"), row.get("daily_digest_time")))
            .collect();

        Ok(users)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    #[test]
    fn test_update_preferences_request_serialization() {
        let req = UpdatePreferencesRequest {
            in_app_enabled: Some(true),
            email_enabled: Some(false),
            sms_enabled: None,
            push_enabled: Some(true),
            batch_non_urgent: Some(false),
            daily_digest_time: Some("09:00:00".to_string()),
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("in_app_enabled"));
        assert!(json.contains("email_enabled"));
    }

    #[test]
    fn test_time_parsing() {
        let time_str = "08:00:00";
        let time = NaiveTime::parse_from_str(time_str, "%H:%M:%S").unwrap();
        assert_eq!(time.hour(), 8);
        assert_eq!(time.minute(), 0);
        assert_eq!(time.second(), 0);
    }
}
