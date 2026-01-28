//! # Data Models for Perlengkapan Service

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
use uuid::Uuid;
use validator::Validate;

// ============ Aset Models (Mapped to integrasi.siman_aset) ============

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Asset {
    pub id: Uuid,
    pub kategori_aset: String,
    pub no_aset: String,
    pub nama_aset: Option<String>, // ur_sskel or nama
    pub kode_barang: Option<String>, // kd_brg
    pub merk: Option<String>,
    pub tipe: Option<String>,
    pub kondisi: Option<String>, // ur_kondisi
    pub lokasi: Option<String>, // alamat
    pub satker: Option<String>, // nama_satker
    pub nilai_perolehan: Option<f64>, // rph_aset
    pub tgl_perolehan: Option<String>, // tgl_perlh
    pub updated_at: DateTime<Utc>,
}

impl Asset {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            kategori_aset: row.get("kategori_aset"),
            no_aset: row.get("no_aset"),
            nama_aset: row.try_get("ur_sskel").ok().or_else(|| row.try_get("nama").ok()),
            kode_barang: row.try_get("kd_brg").ok(),
            merk: row.try_get("merk").ok(),
            tipe: row.try_get("tipe").ok(),
            kondisi: row.try_get("ur_kondisi").ok(),
            lokasi: row.try_get("alamat").ok(),
            satker: row.try_get("nama_satker").ok(),
            nilai_perolehan: row.try_get("rph_aset").ok(), // In table it is TEXT or Numeric? Migration said TEXT for some money fields but usually mapped to f64 via casts
            tgl_perolehan: row.try_get("tgl_perlh").ok(),
            updated_at: row.try_get("updated_at").unwrap_or_else(|_| Utc::now()),
        }
    }
}

// ============ Dashboard Models ============

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_aset: i64,
    pub total_nilai_aset: f64,
    pub total_satker: i64,
    pub aset_baik: i64,
    pub aset_rusak: i64,
    pub categories: Vec<CategoryStat>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryStat {
    pub category: String,
    pub count: i64,
    pub value: f64,
}

// ============ Pengadaan Models (Local) ============

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

// ============ Analisis Kebutuhan Models (Local) ============

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

// ============ Pemakaian Models (Local) ============

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pemakaian {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub piminjam_nama: String,
    pub tanggal_mulai: NaiveDate,
    pub tanggal_selesai: Option<NaiveDate>,
    pub status: String, // dipinjam, kembali
    pub keperluan: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

impl Pemakaian {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            asset_id: row.get("asset_id"),
            piminjam_nama: row.get("piminjam_nama"),
            tanggal_mulai: row.get("tanggal_mulai"),
            tanggal_selesai: row.get("tanggal_selesai"),
            status: row.get("status"),
            keperluan: row.get("keperluan"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            created_by: row.get("created_by"),
            updated_by: row.get("updated_by"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreatePemakaianRequest {
    pub asset_id: Uuid,
    #[validate(length(min = 1, max = 100))]
    pub piminjam_nama: String,
    pub tanggal_mulai: NaiveDate,
    pub tanggal_selesai: Option<NaiveDate>,
    pub keperluan: Option<String>,
}

// ============ Hibah Models (Local) ============

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Hibah {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub pemberi: String,
    pub penerima: String,
    pub tanggal_hibah: NaiveDate,
    pub keterangan: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

impl Hibah {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            asset_id: row.get("asset_id"),
            pemberi: row.get("pemberi"),
            penerima: row.get("penerima"),
            tanggal_hibah: row.get("tanggal_hibah"),
            keterangan: row.get("keterangan"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            created_by: row.get("created_by"),
            updated_by: row.get("updated_by"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateHibahRequest {
    pub asset_id: Uuid,
    #[validate(length(min = 1, max = 255))]
    pub pemberi: String,
    #[validate(length(min = 1, max = 255))]
    pub penerima: String,
    pub tanggal_hibah: NaiveDate,
    pub keterangan: Option<String>,
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
        let total_pages = if per_page > 0 {
            ((total as f64) / (per_page as f64)).ceil() as i32
        } else {
            0
        };
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
