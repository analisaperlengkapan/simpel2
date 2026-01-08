use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePerkaraRequest {
    pub nomor_perkara: String,
    pub judul: String,
    pub tanggal_kejadian: DateTime<Utc>,
    pub status: String,
    pub deskripsi: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerkaraStats {
    pub total: i32,
    pub spdp: i32,
    pub p21: i32,
    pub tahap_2: i32,
}
