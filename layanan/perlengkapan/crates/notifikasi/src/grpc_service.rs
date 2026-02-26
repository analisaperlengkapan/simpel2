use crate::in_app::{InAppNotificationChannel, NotificationPriority, NotificationType};
use crate::preferences::{
    NotificationPreferencesService, UpdatePreferencesRequest as PrefsUpdateRequest,
};
use deadpool_postgres::Pool;
use tonic::{Request, Response, Status};
use uuid::Uuid;

// Include generated proto code
pub mod notifikasi_proto {
    tonic::include_proto!("notifikasi.v1");
}

use notifikasi_proto::{
    notification_service_server::{NotificationService, NotificationServiceServer},
    *,
};

/// gRPC service implementation for notifications
pub struct NotificationServiceImpl {
    pool: Pool,
    in_app_channel: InAppNotificationChannel,
    preferences_service: NotificationPreferencesService,
}

impl NotificationServiceImpl {
    pub fn new(pool: Pool) -> Self {
        let in_app_channel = InAppNotificationChannel::new(pool.clone());
        let preferences_service = NotificationPreferencesService::new(pool.clone());

        Self {
            pool,
            in_app_channel,
            preferences_service,
        }
    }

    /// Create the gRPC server
    pub fn into_server(self) -> NotificationServiceServer<Self> {
        NotificationServiceServer::new(self)
    }
}

#[tonic::async_trait]
impl NotificationService for NotificationServiceImpl {
    async fn send_notification(
        &self,
        request: Request<SendNotificationRequest>,
    ) -> Result<Response<SendNotificationResponse>, Status> {
        let req = request.into_inner();

        // Parse user_id
        let user_id = Uuid::parse_str(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {}", e)))?;

        // Parse notification data
        let notification_type: NotificationType = serde_json::from_str(&req.notification_data)
            .map_err(|e| Status::invalid_argument(format!("Invalid notification_data: {}", e)))?;

        // Parse priority
        let priority = match req.priority.as_str() {
            "low" => NotificationPriority::Low,
            "high" => NotificationPriority::High,
            "urgent" => NotificationPriority::Urgent,
            _ => NotificationPriority::Normal,
        };

        // Send notification (currently only in-app)
        let notification_id = self
            .in_app_channel
            .send(user_id, notification_type, priority)
            .await
            .map_err(|e| Status::internal(format!("Failed to send notification: {}", e)))?;

        Ok(Response::new(SendNotificationResponse {
            success: true,
            message: "Notification sent successfully".to_string(),
            notification_id: notification_id.to_string(),
        }))
    }

    async fn get_notifications(
        &self,
        request: Request<GetNotificationsRequest>,
    ) -> Result<Response<GetNotificationsResponse>, Status> {
        let req = request.into_inner();

        // Parse user_id
        let user_id = Uuid::parse_str(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {}", e)))?;

        // Get notifications
        let notifications = self
            .in_app_channel
            .get_notifications(
                user_id,
                req.limit as i64,
                req.offset as i64,
                req.unread_only,
            )
            .await
            .map_err(|e| Status::internal(format!("Failed to get notifications: {}", e)))?;

        // Convert to proto messages
        let proto_notifications: Vec<Notification> = notifications
            .iter()
            .map(|n| Notification {
                id: n.id.to_string(),
                user_id: n.user_id.to_string(),
                notification_type: n.notification_type.clone(),
                title: n.title.clone(),
                message: n.message.clone(),
                priority: n.priority.clone(),
                category: n.category.clone(),
                action_url: n.action_url.clone().unwrap_or_default(),
                metadata: n
                    .metadata
                    .as_ref()
                    .map(|m| m.to_string())
                    .unwrap_or_default(),
                read: n.read,
                read_at: n.read_at.map(|dt| dt.to_rfc3339()).unwrap_or_default(),
                created_at: n.created_at.to_rfc3339(),
                expires_at: n.expires_at.map(|dt| dt.to_rfc3339()).unwrap_or_default(),
            })
            .collect();

        Ok(Response::new(GetNotificationsResponse {
            notifications: proto_notifications,
            total_count: notifications.len() as i32,
        }))
    }

    async fn get_unread_count(
        &self,
        request: Request<GetUnreadCountRequest>,
    ) -> Result<Response<GetUnreadCountResponse>, Status> {
        let req = request.into_inner();

        // Parse user_id
        let user_id = Uuid::parse_str(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {}", e)))?;

        // Get unread count
        let count = self
            .in_app_channel
            .get_unread_count(user_id)
            .await
            .map_err(|e| Status::internal(format!("Failed to get unread count: {}", e)))?;

        Ok(Response::new(GetUnreadCountResponse {
            unread_count: count,
        }))
    }

    async fn mark_as_read(
        &self,
        request: Request<MarkAsReadRequest>,
    ) -> Result<Response<MarkAsReadResponse>, Status> {
        let req = request.into_inner();

        // Parse IDs
        let notification_id = Uuid::parse_str(&req.notification_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid notification_id: {}", e)))?;
        let user_id = Uuid::parse_str(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {}", e)))?;

        // Mark as read
        self.in_app_channel
            .mark_as_read(notification_id, user_id)
            .await
            .map_err(|e| Status::internal(format!("Failed to mark as read: {}", e)))?;

        Ok(Response::new(MarkAsReadResponse {
            success: true,
            message: "Notification marked as read".to_string(),
        }))
    }

    async fn get_preferences(
        &self,
        request: Request<GetPreferencesRequest>,
    ) -> Result<Response<GetPreferencesResponse>, Status> {
        let req = request.into_inner();

        // Parse user_id
        let user_id = Uuid::parse_str(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {}", e)))?;

        // Get preferences
        let prefs = self
            .preferences_service
            .get_preferences(user_id)
            .await
            .map_err(|e| Status::internal(format!("Failed to get preferences: {}", e)))?;

        // Convert to proto message
        let proto_prefs = NotificationPreferences {
            id: prefs.id.to_string(),
            user_id: prefs.user_id.to_string(),
            in_app_enabled: prefs.in_app_enabled,
            email_enabled: prefs.email_enabled,
            sms_enabled: prefs.sms_enabled,
            push_enabled: prefs.push_enabled,
            batch_non_urgent: prefs.batch_non_urgent,
            daily_digest_time: prefs.daily_digest_time.to_string(),
            created_at: prefs.created_at.to_rfc3339(),
            updated_at: prefs.updated_at.to_rfc3339(),
        };

        Ok(Response::new(GetPreferencesResponse {
            preferences: Some(proto_prefs),
        }))
    }

    async fn update_preferences(
        &self,
        request: Request<UpdatePreferencesRequest>,
    ) -> Result<Response<UpdatePreferencesResponse>, Status> {
        let req = request.into_inner();

        // Parse user_id
        let user_id = Uuid::parse_str(&req.user_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid user_id: {}", e)))?;

        // Build update request
        let updates = PrefsUpdateRequest {
            in_app_enabled: req.in_app_enabled,
            email_enabled: req.email_enabled,
            sms_enabled: req.sms_enabled,
            push_enabled: req.push_enabled,
            batch_non_urgent: req.batch_non_urgent,
            daily_digest_time: req.daily_digest_time,
        };

        // Update preferences
        let prefs = self
            .preferences_service
            .update_preferences(user_id, updates)
            .await
            .map_err(|e| Status::internal(format!("Failed to update preferences: {}", e)))?;

        // Convert to proto message
        let proto_prefs = NotificationPreferences {
            id: prefs.id.to_string(),
            user_id: prefs.user_id.to_string(),
            in_app_enabled: prefs.in_app_enabled,
            email_enabled: prefs.email_enabled,
            sms_enabled: prefs.sms_enabled,
            push_enabled: prefs.push_enabled,
            batch_non_urgent: prefs.batch_non_urgent,
            daily_digest_time: prefs.daily_digest_time.to_string(),
            created_at: prefs.created_at.to_rfc3339(),
            updated_at: prefs.updated_at.to_rfc3339(),
        };

        Ok(Response::new(UpdatePreferencesResponse {
            success: true,
            message: "Preferences updated successfully".to_string(),
            preferences: Some(proto_prefs),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_service_creation() {
        // This is a placeholder test
        // In production, you would test with a mock pool
        assert!(true);
    }
}
