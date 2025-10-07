use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub role: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

impl From<&Row> for User {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            username: row.get("username"),
            email: row.get("email"),
            role: row.get("role"),
            password_hash: row.get("password_hash"),
            created_at: row.get("created_at"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RbacPermission {
    pub id: Uuid,
    pub role: String,
    pub resource: String,
    pub action: String,
}

impl From<&Row> for RbacPermission {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            role: row.get("role"),
            resource: row.get("resource"),
            action: row.get("action"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FaqCategory {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl From<&Row> for FaqCategory {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            parent_id: row.get("parent_id"),
            created_at: row.get("created_at"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
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

impl From<&Row> for FaqArticle {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            category_id: row.get("category_id"),
            title: row.get("title"),
            content: row.get("content"),
            tags: row.get("tags"),
            version: row.get("version"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            is_active: row.get("is_active"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
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

impl From<&Row> for SupportTicket {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            user_id: row.get("user_id"),
            subject: row.get("subject"),
            description: row.get("description"),
            priority: row.get("priority"),
            category_id: row.get("category_id"),
            status: row.get("status"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            closed_at: row.get("closed_at"),
            spam_score: row.get("spam_score"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TicketComment {
    pub id: Uuid,
    pub ticket_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

impl From<&Row> for TicketComment {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            ticket_id: row.get("ticket_id"),
            user_id: row.get("user_id"),
            content: row.get("content"),
            created_at: row.get("created_at"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatbotConversation {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub context: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

impl From<&Row> for ChatbotConversation {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            user_id: row.get("user_id"),
            context: row.get("context"),
            created_at: row.get("created_at"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatbotMessage {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub sender: String,
    pub message: String,
    pub feedback: Option<i32>,
    pub created_at: DateTime<Utc>,
}

impl From<&Row> for ChatbotMessage {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            conversation_id: row.get("conversation_id"),
            sender: row.get("sender"),
            message: row.get("message"),
            feedback: row.get("feedback"),
            created_at: row.get("created_at"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
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

impl From<&Row> for KnowledgeArticle {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            title: row.get("title"),
            content: row.get("content"),
            category_id: row.get("category_id"),
            tags: row.get("tags"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            is_active: row.get("is_active"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HelpAnalytics {
    pub id: Uuid,
    pub metric: String,
    pub value: Option<f64>,
    pub details: Option<serde_json::Value>,
    pub recorded_at: DateTime<Utc>,
}

impl From<&Row> for HelpAnalytics {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            metric: row.get("metric"),
            value: row.get("value"),
            details: row.get("details"),
            recorded_at: row.get("recorded_at"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
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

impl From<&Row> for AuditLog {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            user_id: row.get("user_id"),
            action: row.get("action"),
            resource: row.get("resource"),
            resource_id: row.get("resource_id"),
            details: row.get("details"),
            ip_address: row.get("ip_address"),
            user_agent: row.get("user_agent"),
            timestamp: row.get("timestamp"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookEvent {
    pub id: Uuid,
    pub event_type: String,
    pub payload: Option<serde_json::Value>,
    pub delivered: bool,
    pub delivered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<&Row> for WebhookEvent {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            event_type: row.get("event_type"),
            payload: row.get("payload"),
            delivered: row.get("delivered"),
            delivered_at: row.get("delivered_at"),
            created_at: row.get("created_at"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
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

impl From<&Row> for ExportImportLog {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            user_id: row.get("user_id"),
            action: row.get("action"),
            resource: row.get("resource"),
            resource_id: row.get("resource_id"),
            status: row.get("status"),
            details: row.get("details"),
            created_at: row.get("created_at"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GdprRequest {
    pub id: Uuid,
    pub user_id: Uuid,
    pub request_type: String,
    pub status: String,
    pub details: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

impl From<&Row> for GdprRequest {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            user_id: row.get("user_id"),
            request_type: row.get("request_type"),
            status: row.get("status"),
            details: row.get("details"),
            created_at: row.get("created_at"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CaptchaLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub ip_address: Option<String>,
    pub success: bool,
    pub created_at: DateTime<Utc>,
}

impl From<&Row> for CaptchaLog {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            user_id: row.get("user_id"),
            ip_address: row.get("ip_address"),
            success: row.get("success"),
            created_at: row.get("created_at"),
        }
    }
}
