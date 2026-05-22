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
    pub nama_aset: Option<String>,   // ur_sskel or nama
    pub kode_barang: Option<String>, // kd_brg
    pub merk: Option<String>,
    pub tipe: Option<String>,
    pub kondisi: Option<String>,       // ur_kondisi
    pub lokasi: Option<String>,        // alamat
    pub satker: Option<String>,        // nama_satker
    pub nilai_perolehan: Option<f64>,  // rph_aset
    pub tgl_perolehan: Option<String>, // tgl_perlh
    pub updated_at: DateTime<Utc>,
}

impl Asset {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            kategori_aset: row.get("kategori_aset"),
            no_aset: row.get("no_aset"),
            nama_aset: row
                .try_get("ur_sskel")
                .ok()
                .or_else(|| row.try_get("nama").ok()),
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

// ============ Penghapusan Models (Local) ============

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Penghapusan {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub tanggal_penghapusan: NaiveDate,
    pub alasan: String,
    pub metode_penghapusan: String,
    pub status: String,
    pub nilai_residu: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

impl Penghapusan {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            asset_id: row.get("asset_id"),
            tanggal_penghapusan: row.get("tanggal_penghapusan"),
            alasan: row.get("alasan"),
            metode_penghapusan: row.get("metode_penghapusan"),
            status: row.get("status"),
            nilai_residu: row.get("nilai_residu"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            created_by: row.get("created_by"),
            updated_by: row.get("updated_by"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreatePenghapusanRequest {
    pub asset_id: Uuid,
    pub tanggal_penghapusan: NaiveDate,
    #[validate(length(min = 1))]
    pub alasan: String,
    #[validate(length(min = 1, max = 100))]
    pub metode_penghapusan: String,
    pub nilai_residu: Option<f64>,
}

// ============ Response Models ============
//
// `ApiResponse<T>` + `PaginatedResponse<T>` live in `lib_perlengkapan` so
// the frontend (Leptos WASM) and the backend share one definition + one
// wire format. The constructors are inherent impls on the lib types, so
// callers keep using `ApiResponse::success(...)` / `PaginatedResponse::new(...)`
// unchanged after the re-export.

pub use lib_perlengkapan::response::{ApiResponse, PaginatedResponse};
