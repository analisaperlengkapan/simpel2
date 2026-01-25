//! # Data Models for Perlengkapan Service

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
use uuid::Uuid;
use validator::Validate;

// ============ Aset Models ============

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Aset {
    pub id: Uuid,
    pub nama: String,
    pub kategori: String,
    pub kode_bmn: String,
    pub kondisi: String,
    pub lokasi: String,
    pub nilai_perolehan: Option<f64>,
    pub tanggal_perolehan: Option<NaiveDate>,
    pub status: String,
    pub keterangan: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

impl Aset {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            nama: row.get("nama"),
            kategori: row.get("kategori"),
            kode_bmn: row.get("kode_bmn"),
            kondisi: row.get("kondisi"),
            lokasi: row.get("lokasi"),
            nilai_perolehan: row.get("nilai_perolehan"),
            tanggal_perolehan: row.get("tanggal_perolehan"),
            status: row.get("status"),
            keterangan: row.get("keterangan"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            created_by: row.get("created_by"),
            updated_by: row.get("updated_by"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateAsetRequest {
    #[validate(length(min = 1, max = 255))]
    pub nama: String,
    #[validate(length(min = 1, max = 100))]
    pub kategori: String,
    #[validate(length(min = 1, max = 50))]
    pub kode_bmn: String,
    #[validate(length(min = 1, max = 50))]
    pub kondisi: String,
    #[validate(length(min = 1, max = 255))]
    pub lokasi: String,
    pub nilai_perolehan: Option<f64>,
    pub tanggal_perolehan: Option<NaiveDate>,
    pub keterangan: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateAsetRequest {
    #[validate(length(min = 1, max = 255))]
    pub nama: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub kategori: Option<String>,
    #[validate(length(min = 1, max = 50))]
    pub kondisi: Option<String>,
    #[validate(length(min = 1, max = 255))]
    pub lokasi: Option<String>,
    pub nilai_perolehan: Option<f64>,
    pub tanggal_perolehan: Option<NaiveDate>,
    #[validate(length(min = 1, max = 50))]
    pub status: Option<String>,
    pub keterangan: Option<String>,
}

// ============ Pengadaan Models ============

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pengadaan {
    pub id: Uuid,
    pub judul: String,
    pub deskripsi: Option<String>,
    pub jenis: String,
    pub status: String,
    pub anggaran: Option<f64>,
    pub target_selesai: Option<NaiveDate>,
    pub pic_user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

impl Pengadaan {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            judul: row.get("judul"),
            deskripsi: row.get("deskripsi"),
            jenis: row.get("jenis"),
            status: row.get("status"),
            anggaran: row.get("anggaran"),
            target_selesai: row.get("target_selesai"),
            pic_user_id: row.get("pic_user_id"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            created_by: row.get("created_by"),
            updated_by: row.get("updated_by"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreatePengadaanRequest {
    #[validate(length(min = 1, max = 255))]
    pub judul: String,
    pub deskripsi: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub jenis: String,
    pub anggaran: Option<f64>,
    pub target_selesai: Option<NaiveDate>,
    pub pic_user_id: Option<Uuid>,
}

// ============ Analisis Kebutuhan Models ============

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

// ============ Dashboard Models ============

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_aset: i64,
    pub total_pengadaan: i64,
    pub total_analisis: i64,
    pub aset_aktif: i64,
    pub pengadaan_berjalan: i64,
    pub analisis_pending: i64,
}

// ============ Response Models ============

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
    pub message: String,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T, message: String) -> Self {
        Self {
            success: true,
            data,
            message,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub success: bool,
    pub data: Vec<T>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
    pub total_pages: i32,
    pub message: String,
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, total: i64, page: i32, per_page: i32, message: String) -> Self {
        let total_pages = ((total as f64) / (per_page as f64)).ceil() as i32;
        Self {
            success: true,
            data,
            total,
            page,
            per_page,
            total_pages,
            message,
        }
    }
}
