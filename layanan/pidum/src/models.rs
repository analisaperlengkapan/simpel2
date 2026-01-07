use chrono::{DateTime, Utc};
use garde::Validate;
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Perkara {
    pub id: Uuid,
    pub nomor_perkara: String,
    pub judul: String,
    pub tanggal_kejadian: DateTime<Utc>,
    pub status: String,
    pub deskripsi: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Row> for Perkara {
    fn from(row: Row) -> Self {
        Self {
            id: row.get("id"),
            nomor_perkara: row.get("nomor_perkara"),
            judul: row.get("judul"),
            tanggal_kejadian: row.get("tanggal_kejadian"),
            status: row.get("status"),
            deskripsi: row.get("deskripsi"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreatePerkaraRequest {
    #[garde(length(min = 1, max = 50))]
    pub nomor_perkara: String,
    #[garde(length(min = 1, max = 200))]
    pub judul: String,
    #[garde(skip)]
    pub tanggal_kejadian: DateTime<Utc>,
    #[garde(length(min = 1))]
    pub status: String,
    #[garde(skip)]
    pub deskripsi: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePerkaraRequest {
    #[garde(length(min = 1, max = 200))]
    pub judul: Option<String>,
    #[garde(length(min = 1))]
    pub status: Option<String>,
    #[garde(skip)]
    pub deskripsi: Option<String>,
}
