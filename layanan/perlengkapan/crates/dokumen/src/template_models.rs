use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocumentTemplate {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub template_type: String,
    pub content: String,
    pub format: String,
    pub output_format: String,
    pub version: i32,
    pub is_active: bool,
    pub parent_template_id: Option<Uuid>,
    pub variables: serde_json::Value,
    pub sample_data: Option<serde_json::Value>,
    pub letterhead_config: Option<serde_json::Value>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_by: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
}

impl From<&Row> for DocumentTemplate {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            template_type: row.get("template_type"),
            content: row.get("content"),
            format: row.get("format"),
            output_format: row.get("output_format"),
            version: row.get("version"),
            is_active: row.get("is_active"),
            parent_template_id: row.get("parent_template_id"),
            variables: row.get("variables"),
            sample_data: row.get("sample_data"),
            letterhead_config: row.get("letterhead_config"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_by: row.get("updated_by"),
            updated_at: row.get("updated_at"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeneratedDocument {
    pub id: Uuid,
    pub template_id: Uuid,
    pub document_number: String,
    pub filename: String,
    pub storage_path: String,
    pub format: String,
    pub size: i64,
    pub checksum: String,
    pub generated_data: serde_json::Value,
    pub generated_by: Uuid,
    pub generated_at: DateTime<Utc>,
    pub status: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&Row> for GeneratedDocument {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            template_id: row.get("template_id"),
            document_number: row.get("document_number"),
            filename: row.get("filename"),
            storage_path: row.get("storage_path"),
            format: row.get("format"),
            size: row.get("size"),
            checksum: row.get("checksum"),
            generated_data: row.get("generated_data"),
            generated_by: row.get("generated_by"),
            generated_at: row.get("generated_at"),
            status: row.get("status"),
            metadata: row.get("metadata"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TemplateVersion {
    pub id: Uuid,
    pub template_id: Uuid,
    pub version: i32,
    pub content: String,
    pub variables: serde_json::Value,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub change_notes: Option<String>,
}

impl From<&Row> for TemplateVersion {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            template_id: row.get("template_id"),
            version: row.get("version"),
            content: row.get("content"),
            variables: row.get("variables"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            change_notes: row.get("change_notes"),
        }
    }
}

// Request/Response DTOs
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTemplateRequest {
    pub name: String,
    pub description: Option<String>,
    pub template_type: String,
    pub content: String,
    pub format: String,
    pub output_format: String,
    pub variables: serde_json::Value,
    pub sample_data: Option<serde_json::Value>,
    pub letterhead_config: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTemplateRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub format: Option<String>,
    pub output_format: Option<String>,
    pub variables: Option<serde_json::Value>,
    pub sample_data: Option<serde_json::Value>,
    pub letterhead_config: Option<serde_json::Value>,
    pub change_notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplatePreviewRequest {
    pub template_id: Uuid,
    pub data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplatePreviewResponse {
    pub html: String,
    pub variables_used: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateDocumentRequest {
    pub template_id: Uuid,
    pub data: serde_json::Value,
    pub output_format: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateDocumentResponse {
    pub document_id: Uuid,
    pub document_number: String,
    pub filename: String,
    pub download_url: String,
    pub checksum: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListTemplatesQuery {
    pub template_type: Option<String>,
    pub is_active: Option<bool>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListTemplatesResponse {
    pub templates: Vec<DocumentTemplate>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}
