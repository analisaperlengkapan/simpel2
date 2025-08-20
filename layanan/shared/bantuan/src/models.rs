use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub role: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct RbacPermission {
    pub id: Uuid,
    pub role: String,
    pub resource: String,
    pub action: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct FaqCategory {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct FaqArticle {
    pub id: Uuid,
    pub category_id: Uuid,
    pub title: String,
    pub content: String,
    pub tags: Option<Vec<String>>,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SupportTicket {
    pub id: Uuid,
    pub user_id: Uuid,
    pub subject: String,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub category_id: Option<Uuid>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub spam_score: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TicketComment {
    pub id: Uuid,
    pub ticket_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ChatbotConversation {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub context: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ChatbotMessage {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub sender: String,
    pub message: String,
    pub feedback: Option<i32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct KnowledgeArticle {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub category_id: Option<Uuid>,
    pub tags: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct HelpAnalytics {
    pub id: Uuid,
    pub metric: String,
    pub value: Option<f64>,
    pub details: Option<serde_json::Value>,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub resource: Option<String>,
    pub resource_id: Option<Uuid>,
    pub details: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct WebhookEvent {
    pub id: Uuid,
    pub event_type: String,
    pub payload: Option<serde_json::Value>,
    pub delivered: bool,
    pub delivered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ExportImportLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub resource: Option<String>,
    pub resource_id: Option<Uuid>,
    pub status: Option<String>,
    pub details: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct GdprRequest {
    pub id: Uuid,
    pub user_id: Uuid,
    pub request_type: String,
    pub status: String,
    pub details: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CaptchaLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub ip_address: Option<String>,
    pub success: bool,
    pub created_at: DateTime<Utc>,
} 