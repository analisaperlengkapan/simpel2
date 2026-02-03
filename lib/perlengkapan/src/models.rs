//! # Shared Data Models for Perlengkapan Service

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use validator::Validate;

// ============ Aset Models (Mapped to integrasi.siman_aset) ============

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

// ============ Dashboard Models ============

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DashboardStats {
    pub total_aset: i64,
    pub total_nilai_aset: f64,
    pub total_satker: i64,
    pub aset_baik: i64,
    pub aset_rusak: i64,
    pub categories: Vec<CategoryStat>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CategoryStat {
    pub category: String,
    pub count: i64,
    pub value: f64,
}

// ============ Pengadaan Models (Local) ============

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
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

// ============ Pengadaan Sub-Documents ============

// HPS
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PengadaanHps {
    pub id: Uuid,
    pub pengadaan_id: Uuid,
    pub no_hps: String,
    pub tgl_hps: NaiveDate,
    pub nip_penandatangan: String,
    pub nama_penandatangan: String,
    pub pangkat_penandatangan: String,
    pub barang: Value, // JSONB
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct CreatePengadaanHpsRequest {
    pub pengadaan_id: Uuid,
    #[validate(length(min = 1, max = 100))]
    pub no_hps: String,
    pub tgl_hps: NaiveDate,
    pub nip_penandatangan: String,
    pub nama_penandatangan: String,
    pub pangkat_penandatangan: String,
    pub barang: Value,
}

// SKPPBJ
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PengadaanSkppbj {
    pub id: Uuid,
    pub pengadaan_id: Uuid,
    pub nama_penandatangan: String,
    pub nip_penandatangan: String,
    pub pangkat_penandatangan: String,
    pub jabatan_penandatangan: String,
    pub alamat: String,
    pub tgl_skppbj: NaiveDate,
    pub penyedia: Value, // JSONB
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct CreatePengadaanSkppbjRequest {
    pub pengadaan_id: Uuid,
    pub nama_penandatangan: String,
    pub nip_penandatangan: String,
    pub pangkat_penandatangan: String,
    pub jabatan_penandatangan: String,
    pub alamat: String,
    pub tgl_skppbj: NaiveDate,
    pub penyedia: Value,
}

// SPK
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PengadaanSpk {
    pub id: Uuid,
    pub pengadaan_id: Uuid,
    pub no_spk: String,
    pub no_permintaan: String,
    pub tgl_permintaan: NaiveDate,
    pub no_ba: String,
    pub tgl_ba: NaiveDate,
    pub tgl_mulai: NaiveDate,
    pub tgl_spk: NaiveDate,
    pub tgl_selesai: NaiveDate,
    pub nama_penyedia: String,
    pub keterangan: Option<String>,
    pub instruksi: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct CreatePengadaanSpkRequest {
    pub pengadaan_id: Uuid,
    pub no_spk: String,
    pub no_permintaan: String,
    pub tgl_permintaan: NaiveDate,
    pub no_ba: String,
    pub tgl_ba: NaiveDate,
    pub tgl_mulai: NaiveDate,
    pub tgl_spk: NaiveDate,
    pub tgl_selesai: NaiveDate,
    pub nama_penyedia: String,
    pub keterangan: Option<String>,
    pub instruksi: Option<String>,
}

// Ringkasan
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PengadaanRingkasan {
    pub id: Uuid,
    pub pengadaan_id: Uuid,
    pub no_dipa: String,
    pub tgl_dipa: NaiveDate,
    pub cara_pembayaran: String,
    pub alamat_penyedia: String,
    pub nama_bank: String,
    pub kantor_bank: String,
    pub no_rek: String,
    pub npwp: String,
    pub sanksi: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct CreatePengadaanRingkasanRequest {
    pub pengadaan_id: Uuid,
    pub no_dipa: String,
    pub tgl_dipa: NaiveDate,
    pub cara_pembayaran: String,
    pub alamat_penyedia: String,
    pub nama_bank: String,
    pub kantor_bank: String,
    pub no_rek: String,
    pub npwp: String,
    pub sanksi: Option<String>,
}

// Kontrak
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PengadaanKontrak {
    pub id: Uuid,
    pub pengadaan_id: Uuid,
    pub no_kontrak: String,
    pub tgl_kontrak: NaiveDate,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct CreatePengadaanKontrakRequest {
    pub pengadaan_id: Uuid,
    pub no_kontrak: String,
    pub tgl_kontrak: NaiveDate,
}

// BAST
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PengadaanBast {
    pub id: Uuid,
    pub pengadaan_id: Uuid,
    pub no_bast: String,
    pub tgl_bast: NaiveDate,
    pub nama_pejabat: String,
    pub nip_pejabat: String,
    pub pangkat_pejabat: String,
    pub jabatan_pejabat: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct CreatePengadaanBastRequest {
    pub pengadaan_id: Uuid,
    pub no_bast: String,
    pub tgl_bast: NaiveDate,
    pub nama_pejabat: String,
    pub nip_pejabat: String,
    pub pangkat_pejabat: String,
    pub jabatan_pejabat: String,
}

// Nodis
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PengadaanNodis {
    pub id: Uuid,
    pub pengadaan_id: Uuid,
    pub no_nodis: String,
    pub tgl_nodis: NaiveDate,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct CreatePengadaanNodisRequest {
    pub pengadaan_id: Uuid,
    pub no_nodis: String,
    pub tgl_nodis: NaiveDate,
}

// ============ Analisis Kebutuhan Models (Local) ============

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct CreatePemakaianRequest {
    pub asset_id: Uuid,
    #[validate(length(min = 1, max = 100))]
    pub piminjam_nama: String,
    pub tanggal_mulai: NaiveDate,
    pub tanggal_selesai: Option<NaiveDate>,
    pub keperluan: Option<String>,
}

// ============ Hibah Models (Local) ============

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct CreateHibahRequest {
    pub asset_id: Uuid,
    #[validate(length(min = 1, max = 255))]
    pub pemberi: String,
    #[validate(length(min = 1, max = 255))]
    pub penerima: String,
    pub tanggal_hibah: NaiveDate,
    pub keterangan: Option<String>,
}

// ============ Mutasi Models (Local) ============

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
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

// ============ Penghapusan Models (Local) ============

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct CreatePenghapusanRequest {
    pub asset_id: Uuid,
    pub tanggal_penghapusan: NaiveDate,
    #[validate(length(min = 1))]
    pub alasan: String,
    #[validate(length(min = 1, max = 100))]
    pub metode_penghapusan: String,
    pub nilai_residu: Option<f64>,
}

// ============ Pengalihan Models (Local) ============

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
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

// ============ Pemeliharaan Models (Local) ============

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Pemeliharaan {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub jenis_pemeliharaan: String, // Rutin, Perbaikan, etc.
    pub biaya: Option<f64>,
    pub tanggal_mulai: NaiveDate,
    pub tanggal_selesai: Option<NaiveDate>,
    pub pelaksana: String, // Vendor or Internal
    pub status: String,    // Terjadwal, Proses, Selesai
    pub keterangan: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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
