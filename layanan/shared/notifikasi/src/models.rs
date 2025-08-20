use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Notification {
    pub id: Uuid,
    pub channel: String,
    pub template_id: Option<Uuid>,
    pub event_id: Option<Uuid>,
    pub status: String,
    pub subject: Option<String>,
    pub body: Option<String>,
    pub payload: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub sent_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub read_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct NotificationTemplate {
    pub id: Uuid,
    pub name: String,
    pub content: String,
    pub variables: Option<serde_json::Value>,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct NotificationChannel {
    pub id: Uuid,
    pub channel: String,
    pub config: Option<serde_json::Value>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct NotificationEvent {
    pub id: Uuid,
    pub event_type: String,
    pub reference_id: Option<Uuid>,
    pub payload: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct NotificationRecipient {
    pub id: Uuid,
    pub notification_id: Uuid,
    pub recipient: String,
    pub recipient_type: String,
    pub status: String,
    pub sent_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub read_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DeliveryLog {
    pub id: Uuid,
    pub notification_id: Uuid,
    pub recipient: String,
    pub channel: String,
    pub status: String,
    pub message: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Consent {
    pub id: Uuid,
    pub user_id: Uuid,
    pub channel: String,
    pub consented: bool,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Optout {
    pub id: Uuid,
    pub recipient: String,
    pub channel: String,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
} 