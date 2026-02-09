//! Shared domain models for Perlengkapan
//!
//! These models are designed to work in both backend (with tokio-postgres)
//! and frontend (with WASM) environments.

use chrono::{DateTime, NaiveDate, Utc};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[cfg(feature = "backend")]
use tokio_postgres::Row;

// ============ Aset Models ============

/// Base asset representation shared across all perlengkapan services
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Asset {
    pub id: Uuid,
    pub kategori_aset: String,
    pub no_aset: String,
    pub nama_aset: Option<String>,
    pub kode_barang: Option<String>,
    pub merk: Option<String>,
    pub tipe: Option<String>,
    pub kondisi: Option<String>,
    pub lokasi: Option<String>,
    pub satker: Option<String>,
    pub nilai_perolehan: Option<f64>,
    pub tgl_perolehan: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(feature = "backend")]
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
            nilai_perolehan: row.try_get("rph_aset").ok(),
            tgl_perolehan: row.try_get("tgl_perlh").ok(),
            updated_at: row.try_get("updated_at").unwrap_or_else(|_| Utc::now()),
        }
    }
}

// ============ Dashboard Models ============

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DashboardStats {
    pub total_aset: i64,
    pub total_nilai_aset: f64,
    pub total_satker: i64,
    pub aset_baik: i64,
    pub aset_rusak: i64,
    pub categories: Vec<CategoryStat>,
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CategoryStat {
    pub category: String,
    pub count: i64,
    pub value: f64,
}

// ============ Pengadaan Models ============

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

#[cfg(feature = "backend")]
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

#[derive(Debug, Validate)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

#[cfg(feature = "backend")]
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

#[derive(Debug, Validate)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

// ============ Pemakaian Models ============

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Pemakaian {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub piminjam_nama: String,
    pub tanggal_mulai: NaiveDate,
    pub tanggal_selesai: Option<NaiveDate>,
    pub status: String,
    pub keperluan: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

#[cfg(feature = "backend")]
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

#[derive(Debug, Validate)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CreatePemakaianRequest {
    pub asset_id: Uuid,
    #[validate(length(min = 1, max = 100))]
    pub piminjam_nama: String,
    pub tanggal_mulai: NaiveDate,
    pub tanggal_selesai: Option<NaiveDate>,
    pub keperluan: Option<String>,
}

// ============ Hibah Models ============

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

#[cfg(feature = "backend")]
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

#[derive(Debug, Validate)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CreateHibahRequest {
    pub asset_id: Uuid,
    #[validate(length(min = 1, max = 255))]
    pub pemberi: String,
    #[validate(length(min = 1, max = 255))]
    pub penerima: String,
    pub tanggal_hibah: NaiveDate,
    pub keterangan: Option<String>,
}

// ============ Mutasi Models ============

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Mutasi {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub asal_satker: String,
    pub tujuan_satker: String,
    pub penanggung_jawab: String,
    pub tanggal_mutasi: NaiveDate,
    pub status: String,
    pub keterangan: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

#[cfg(feature = "backend")]
impl Mutasi {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            asset_id: row.get("asset_id"),
            asal_satker: row.get("asal_satker"),
            tujuan_satker: row.get("tujuan_satker"),
            penanggung_jawab: row.get("penanggung_jawab"),
            tanggal_mutasi: row.get("tanggal_mutasi"),
            status: row.get("status"),
            keterangan: row.get("keterangan"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            created_by: row.get("created_by"),
            updated_by: row.get("updated_by"),
        }
    }
}

#[derive(Debug, Validate)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CreateMutasiRequest {
    pub asset_id: Uuid,
    #[validate(length(min = 1, max = 255))]
    pub asal_satker: String,
    #[validate(length(min = 1, max = 255))]
    pub tujuan_satker: String,
    #[validate(length(min = 1, max = 255))]
    pub penanggung_jawab: String,
    pub tanggal_mutasi: NaiveDate,
    pub keterangan: Option<String>,
}

// ============ Penghapusan Models ============

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

#[cfg(feature = "backend")]
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

#[derive(Debug, Validate)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CreatePenghapusanRequest {
    pub asset_id: Uuid,
    pub tanggal_penghapusan: NaiveDate,
    #[validate(length(min = 1))]
    pub alasan: String,
    #[validate(length(min = 1, max = 100))]
    pub metode_penghapusan: String,
    pub nilai_residu: Option<f64>,
}

// ============ Pengalihan Models ============

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Pengalihan {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub pihak_lama: String,
    pub pihak_baru: String,
    pub tanggal_pengalihan: NaiveDate,
    pub dasar_pengalihan: Option<String>,
    pub status: String,
    pub keterangan: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

#[cfg(feature = "backend")]
impl Pengalihan {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            asset_id: row.get("asset_id"),
            pihak_lama: row.get("pihak_lama"),
            pihak_baru: row.get("pihak_baru"),
            tanggal_pengalihan: row.get("tanggal_pengalihan"),
            dasar_pengalihan: row.get("dasar_pengalihan"),
            status: row.get("status"),
            keterangan: row.get("keterangan"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            created_by: row.get("created_by"),
            updated_by: row.get("updated_by"),
        }
    }
}

#[derive(Debug, Validate)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CreatePengalihanRequest {
    pub asset_id: Uuid,
    #[validate(length(min = 1, max = 255))]
    pub pihak_lama: String,
    #[validate(length(min = 1, max = 255))]
    pub pihak_baru: String,
    pub tanggal_pengalihan: NaiveDate,
    pub dasar_pengalihan: Option<String>,
    pub keterangan: Option<String>,
}

// ============ Pemeliharaan Models ============

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Pemeliharaan {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub jenis_pemeliharaan: String,
    pub biaya: Option<f64>,
    pub tanggal_mulai: NaiveDate,
    pub tanggal_selesai: Option<NaiveDate>,
    pub pelaksana: String,
    pub status: String,
    pub keterangan: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

#[cfg(feature = "backend")]
impl Pemeliharaan {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            asset_id: row.get("asset_id"),
            jenis_pemeliharaan: row.get("jenis_pemeliharaan"),
            biaya: row.get("biaya"),
            tanggal_mulai: row.get("tanggal_mulai"),
            tanggal_selesai: row.get("tanggal_selesai"),
            pelaksana: row.get("pelaksana"),
            status: row.get("status"),
            keterangan: row.get("keterangan"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            created_by: row.get("created_by"),
            updated_by: row.get("updated_by"),
        }
    }
}

#[derive(Debug, Validate)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CreatePemeliharaanRequest {
    pub asset_id: Uuid,
    #[validate(length(min = 1, max = 100))]
    pub jenis_pemeliharaan: String,
    pub biaya: Option<f64>,
    pub tanggal_mulai: NaiveDate,
    pub tanggal_selesai: Option<NaiveDate>,
    #[validate(length(min = 1, max = 255))]
    pub pelaksana: String,
    pub keterangan: Option<String>,
}

// ============ Response Models ============

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
