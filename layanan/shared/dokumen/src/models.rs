use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Document {
    pub id: Uuid,
    pub filename: String,
    pub content_type: String,
    pub size: i64,
    pub storage_path: String,
    pub owner_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_archived: bool,
    pub checksum: Option<String>,
    pub encrypted: bool,
    pub current_version: i32,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DocumentVersion {
    pub id: Uuid,
    pub document_id: Uuid,
    pub version: i32,
    pub storage_path: String,
    pub size: i64,
    pub created_at: DateTime<Utc>,
    pub checksum: Option<String>,
    pub encrypted: bool,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DocumentTag {
    pub id: Uuid,
    pub document_id: Uuid,
    pub tag: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DocumentPermission {
    pub id: Uuid,
    pub document_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub granted_by: Option<Uuid>,
    pub granted_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    pub id: Uuid,
    pub document_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub details: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct OcrResult {
    pub id: Uuid,
    pub document_id: Uuid,
    pub status: String,
    pub text: Option<String>,
    pub accuracy: Option<f32>,
    pub processed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ArchiveCollection {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub owner_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ArchiveDocument {
    pub id: Uuid,
    pub collection_id: Uuid,
    pub document_id: Uuid,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct VirusScanLog {
    pub id: Uuid,
    pub document_id: Uuid,
    pub scanned_at: DateTime<Utc>,
    pub result: String,
    pub details: Option<String>,
} 