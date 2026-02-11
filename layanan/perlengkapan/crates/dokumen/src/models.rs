use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
use uuid::Uuid;

// Re-export template models
pub use crate::template_models::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
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

impl From<&Row> for Document {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            filename: row.get("filename"),
            content_type: row.get("content_type"),
            size: row.get("size"),
            storage_path: row.get("storage_path"),
            owner_id: row.get("owner_id"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            is_archived: row.get("is_archived"),
            checksum: row.get("checksum"),
            encrypted: row.get("encrypted"),
            current_version: row.get("current_version"),
            metadata: row.get("metadata"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

impl From<&Row> for DocumentVersion {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            document_id: row.get("document_id"),
            version: row.get("version"),
            storage_path: row.get("storage_path"),
            size: row.get("size"),
            created_at: row.get("created_at"),
            checksum: row.get("checksum"),
            encrypted: row.get("encrypted"),
            metadata: row.get("metadata"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocumentTag {
    pub id: Uuid,
    pub document_id: Uuid,
    pub tag: String,
    pub created_at: DateTime<Utc>,
}

impl From<&Row> for DocumentTag {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            document_id: row.get("document_id"),
            tag: row.get("tag"),
            created_at: row.get("created_at"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocumentPermission {
    pub id: Uuid,
    pub document_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub granted_by: Option<Uuid>,
    pub granted_at: DateTime<Utc>,
}

impl From<&Row> for DocumentPermission {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            document_id: row.get("document_id"),
            user_id: row.get("user_id"),
            role: row.get("role"),
            granted_by: row.get("granted_by"),
            granted_at: row.get("granted_at"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

impl From<&Row> for AuditLog {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            document_id: row.get("document_id"),
            user_id: row.get("user_id"),
            action: row.get("action"),
            details: row.get("details"),
            ip_address: row.get("ip_address"),
            user_agent: row.get("user_agent"),
            timestamp: row.get("timestamp"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OcrResult {
    pub id: Uuid,
    pub document_id: Uuid,
    pub status: String,
    pub text: Option<String>,
    pub accuracy: Option<f32>,
    pub processed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

impl From<&Row> for OcrResult {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            document_id: row.get("document_id"),
            status: row.get("status"),
            text: row.get("text"),
            accuracy: row.get("accuracy"),
            processed_at: row.get("processed_at"),
            error_message: row.get("error_message"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArchiveCollection {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub owner_id: Option<Uuid>,
}

impl From<&Row> for ArchiveCollection {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            created_at: row.get("created_at"),
            owner_id: row.get("owner_id"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArchiveDocument {
    pub id: Uuid,
    pub collection_id: Uuid,
    pub document_id: Uuid,
    pub added_at: DateTime<Utc>,
}

impl From<&Row> for ArchiveDocument {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            collection_id: row.get("collection_id"),
            document_id: row.get("document_id"),
            added_at: row.get("added_at"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VirusScanLog {
    pub id: Uuid,
    pub document_id: Uuid,
    pub scanned_at: DateTime<Utc>,
    pub result: String,
    pub details: Option<String>,
}
