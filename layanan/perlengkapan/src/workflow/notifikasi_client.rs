// ============================================================================
// Notifikasi Service gRPC Client
// Description: Client for calling notifikasi service from workflow engine
// Requirements: REQ-N003, REQ-W011
// ============================================================================

use tonic::transport::Channel;
use uuid::Uuid;

// Re-export generated proto types
pub use crate::workflow::notifikasi_proto::notification_service_client::NotificationServiceClient;
pub use crate::workflow::notifikasi_proto::{SendNotificationRequest, SendNotificationResponse};

/// Notifikasi client error types
#[derive(Debug, thiserror::Error)]
pub enum NotifikasiClientError {
    #[error("gRPC transport error: {0}")]
    TransportError(#[from] tonic::transport::Error),

    #[error("gRPC status error: {0}")]
    StatusError(#[from] tonic::Status),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, NotifikasiClientError>;

/// Notification priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Urgent,
}

impl NotificationPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Normal => "normal",
            Self::High => "high",
            Self::Urgent => "urgent",
        }
    }
}

/// Notification type for workflow transitions
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkflowNotificationType {
    /// Workflow state transition notification
    WorkflowTransition {
        entity_type: String,
        entity_id: String,
        from_state: String,
        to_state: String,
        transition_by: String,
        catatan: Option<String>,
    },
    /// Approval required notification
    ApprovalRequired {
        entity_type: String,
        entity_id: String,
        current_state: String,
        required_role: String,
        deadline: Option<String>,
    },
    /// Approval completed notification
    ApprovalCompleted {
        entity_type: String,
        entity_id: String,
        approved_by: String,
        document_url: Option<String>,
    },
    /// Rejection notification
    Rejected {
        entity_type: String,
        entity_id: String,
        rejected_by: String,
        reason: Option<String>,
    },
    /// Revision required notification
    RevisionRequired {
        entity_type: String,
        entity_id: String,
        requested_by: String,
        notes: Option<String>,
    },
    /// SLA breach escalation notification (to approver)
    SlaBreachEscalation {
        entity_type: String,
        entity_id: String,
        current_state: String,
        sla_deadline: String,
        breach_duration_minutes: i64,
        days_overdue: i32,
    },
    /// SLA breach informational notification (to requester)
    SlaBreachInfo {
        entity_type: String,
        entity_id: String,
        current_state: String,
        sla_deadline: String,
        breach_duration_minutes: i64,
    },
}

/// Notifikasi service gRPC client wrapper
#[derive(Clone)]
pub struct NotifikasiClient {
    client: NotificationServiceClient<Channel>,
}

impl NotifikasiClient {
    /// Create a new notifikasi client
    pub async fn new(endpoint: &str) -> Result<Self> {
        let channel = Channel::from_shared(endpoint.to_string())
            .map_err(|e| {
                NotifikasiClientError::InvalidResponse(format!("Invalid endpoint URI: {}", e))
            })?
            .connect()
            .await?;

        let client = NotificationServiceClient::new(channel);

        Ok(Self { client })
    }

    /// Send a notification to a user
    pub async fn send_notification(
        &mut self,
        user_id: Uuid,
        notification_type: WorkflowNotificationType,
        priority: NotificationPriority,
    ) -> Result<SendNotificationResponse> {
        // Serialize notification data
        let notification_data = serde_json::to_string(&notification_type)?;

        let request = SendNotificationRequest {
            user_id: user_id.to_string(),
            notification_type: "workflow".to_string(),
            notification_data,
            priority: priority.as_str().to_string(),
            channels: vec!["in_app".to_string()],
        };

        let response = self.client.send_notification(request).await?;

        Ok(response.into_inner())
    }

    /// Send notification to multiple users
    pub async fn send_notification_to_multiple(
        &mut self,
        user_ids: Vec<Uuid>,
        notification_type: WorkflowNotificationType,
        priority: NotificationPriority,
    ) -> Result<Vec<SendNotificationResponse>> {
        let mut responses = Vec::new();

        for user_id in user_ids {
            match self
                .send_notification(user_id, notification_type.clone(), priority)
                .await
            {
                Ok(response) => responses.push(response),
                Err(e) => {
                    // Log error but continue with other users
                    tracing::error!(
                        user_id = %user_id,
                        error = %e,
                        "Failed to send notification to user"
                    );
                }
            }
        }

        Ok(responses)
    }
}

/// Notification send result
#[derive(Debug, Clone)]
pub struct NotificationSendResult {
    pub notification_id: Uuid,
    pub success: bool,
    pub message: String,
}

impl TryFrom<SendNotificationResponse> for NotificationSendResult {
    type Error = NotifikasiClientError;

    fn try_from(response: SendNotificationResponse) -> Result<Self> {
        let notification_id = Uuid::parse_str(&response.notification_id).map_err(|e| {
            NotifikasiClientError::InvalidResponse(format!("Invalid notification_id: {}", e))
        })?;

        Ok(Self {
            notification_id,
            success: response.success,
            message: response.message,
        })
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
    fn test_workflow_notification_type_serialization() {
        let notification = WorkflowNotificationType::WorkflowTransition {
            entity_type: "kebutuhan_bmn".to_string(),
            entity_id: Uuid::new_v4().to_string(),
            from_state: "DRAFT".to_string(),
            to_state: "SUBMITTED".to_string(),
            transition_by: "user-123".to_string(),
            catatan: Some("Test transition".to_string()),
        };

        let json = serde_json::to_string(&notification).unwrap();
        assert!(json.contains("workflow_transition"));
        assert!(json.contains("kebutuhan_bmn"));
    }
}
