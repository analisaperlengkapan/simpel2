use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
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

impl From<Row> for Notification {
    fn from(row: Row) -> Self {
        Self {
            id: row.get("id"),
            channel: row.get("channel"),
            template_id: row.get("template_id"),
            event_id: row.get("event_id"),
            status: row.get("status"),
            subject: row.get("subject"),
            body: row.get("body"),
            payload: row.get("payload"),
            created_at: row.get("created_at"),
            sent_at: row.get("sent_at"),
            delivered_at: row.get("delivered_at"),
            read_at: row.get("read_at"),
            error_message: row.get("error_message"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
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

impl From<Row> for NotificationTemplate {
    fn from(row: Row) -> Self {
        Self {
            id: row.get("id"),
            name: row.get("name"),
            content: row.get("content"),
            variables: row.get("variables"),
            version: row.get("version"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            is_active: row.get("is_active"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct NotificationChannel {
    pub id: Uuid,
    pub channel: String,
    pub config: Option<serde_json::Value>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct NotificationEvent {
    pub id: Uuid,
    pub event_type: String,
    pub reference_id: Option<Uuid>,
    pub payload: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
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

impl From<Row> for NotificationRecipient {
    fn from(row: Row) -> Self {
        Self {
            id: row.get("id"),
            notification_id: row.get("notification_id"),
            recipient: row.get("recipient"),
            recipient_type: row.get("recipient_type"),
            status: row.get("status"),
            sent_at: row.get("sent_at"),
            delivered_at: row.get("delivered_at"),
            read_at: row.get("read_at"),
            error_message: row.get("error_message"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct DeliveryLog {
    pub id: Uuid,
    pub notification_id: Uuid,
    pub recipient: String,
    pub channel: String,
    pub status: String,
    pub message: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Consent {
    pub id: Uuid,
    pub user_id: Uuid,
    pub channel: String,
    pub consented: bool,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Optout {
    pub id: Uuid,
    pub recipient: String,
    pub channel: String,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
}
