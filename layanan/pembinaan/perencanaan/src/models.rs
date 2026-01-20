use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RencanaPengadaan {
    pub id: Uuid,
    pub nama_kegiatan: String,
    pub kode_rekening: String,
    pub pagu_anggaran: i64,
    pub tanggal_mulai: NaiveDate,
    pub tanggal_selesai: NaiveDate,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRencanaRequest {
    pub nama_kegiatan: String,
    pub kode_rekening: String,
    pub pagu_anggaran: i64,
    pub tanggal_mulai: NaiveDate,
    pub tanggal_selesai: NaiveDate,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRencanaRequest {
    pub nama_kegiatan: Option<String>,
    pub kode_rekening: Option<String>,
    pub pagu_anggaran: Option<i64>,
    pub tanggal_mulai: Option<NaiveDate>,
    pub tanggal_selesai: Option<NaiveDate>,
    pub status: Option<String>,
}

impl RencanaPengadaan {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            nama_kegiatan: row.get("nama_kegiatan"),
            kode_rekening: row.get("kode_rekening"),
            pagu_anggaran: row.get("pagu_anggaran"),
            tanggal_mulai: row.get("tanggal_mulai"),
            tanggal_selesai: row.get("tanggal_selesai"),
            status: row.get("status"),
            created_at: row.get("created_at"),
        }
    }
}
