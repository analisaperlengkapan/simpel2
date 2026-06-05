//! Data models for the Analisis Kebutuhan feature.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnalisisKebutuhan {
    pub id: Uuid,
    pub judul: String,
    pub kategori: String,
    pub deskripsi: Option<String>,
    pub prioritas: String,
    pub status: String,
    pub estimasi_biaya: Option<f64>,
    pub justifikasi: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

impl AnalisisKebutuhan {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            judul: row.get("judul"),
            kategori: row.get("kategori"),
            deskripsi: row.get("deskripsi"),
            prioritas: row.get("prioritas"),
            status: row.get("status"),
            estimasi_biaya: row.get("estimasi_biaya"),
            justifikasi: row.get("justifikasi"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            created_by: row.get("created_by"),
            updated_by: row.get("updated_by"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateAnalisisRequest {
    #[validate(length(min = 1, max = 255))]
    pub judul: String,
    #[validate(length(min = 1, max = 100))]
    pub kategori: String,
    pub deskripsi: Option<String>,
    #[validate(length(min = 1, max = 50))]
    pub prioritas: String,
    pub estimasi_biaya: Option<f64>,
    pub justifikasi: Option<String>,
}
