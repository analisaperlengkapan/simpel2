//! # Pakaian Dinas Data Models
//!
//! Data models for official uniform management system.
//! Includes master data, transaction data, and workflow tracking.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
use uuid::Uuid;
use validator::Validate;

// ============ Enums ============

/// Gender options for uniform specifications
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Gender {
    L,     // Laki-laki (Male)
    P,     // Perempuan (Female)
    Semua, // All genders
}

impl Gender {
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "L" => Gender::L,
            "P" => Gender::P,
            _ => Gender::Semua,
        }
    }

    pub fn to_db_string(&self) -> &'static str {
        match self {
            Gender::L => "L",
            Gender::P => "P",
            Gender::Semua => "SEMUA",
        }
    }
}

/// Ukuran group for clothing categories
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum UkuranGroup {
    Baju,
    Celana,
    Sepatu,
}

impl UkuranGroup {
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "BAJU" => UkuranGroup::Baju,
            "CELANA" => UkuranGroup::Celana,
            "SEPATU" => UkuranGroup::Sepatu,
            _ => UkuranGroup::Baju,
        }
    }

    pub fn to_db_string(&self) -> &'static str {
        match self {
            UkuranGroup::Baju => "BAJU",
            UkuranGroup::Celana => "CELANA",
            UkuranGroup::Sepatu => "SEPATU",
        }
    }
}

/// Workflow status codes for pakaian dinas requests
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AktivitasStatus {
    Input = 1000,
    SubmitToValidator = 1001,
    SubmitToValidatorWilayah = 1012,
    RevisiPelaksana = 1003,
    RevisiSatker = 1005,
    RevisiWilayah = 1007,
    SubmitToPusat = 1004,
    SubmitToPusatFromWilayah = 1010,
    Selesai = 1008,
    StartKejagung = 1009,
    StartNonKejagung = 1011,
}

impl AktivitasStatus {
    pub fn from_i32(code: i32) -> Option<Self> {
        match code {
            1000 => Some(AktivitasStatus::Input),
            1001 => Some(AktivitasStatus::SubmitToValidator),
            1003 => Some(AktivitasStatus::RevisiPelaksana),
            1004 => Some(AktivitasStatus::SubmitToPusat),
            1005 => Some(AktivitasStatus::RevisiSatker),
            1007 => Some(AktivitasStatus::RevisiWilayah),
            1008 => Some(AktivitasStatus::Selesai),
            1009 => Some(AktivitasStatus::StartKejagung),
            1010 => Some(AktivitasStatus::SubmitToPusatFromWilayah),
            1011 => Some(AktivitasStatus::StartNonKejagung),
            1012 => Some(AktivitasStatus::SubmitToValidatorWilayah),
            _ => None,
        }
    }

    pub fn to_i32(&self) -> i32 {
        match self {
            AktivitasStatus::Input => 1000,
            AktivitasStatus::SubmitToValidator => 1001,
            AktivitasStatus::SubmitToValidatorWilayah => 1012,
            AktivitasStatus::RevisiPelaksana => 1003,
            AktivitasStatus::RevisiSatker => 1005,
            AktivitasStatus::RevisiWilayah => 1007,
            AktivitasStatus::SubmitToPusat => 1004,
            AktivitasStatus::SubmitToPusatFromWilayah => 1010,
            AktivitasStatus::Selesai => 1008,
            AktivitasStatus::StartKejagung => 1009,
            AktivitasStatus::StartNonKejagung => 1011,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            AktivitasStatus::Input => "Input",
            AktivitasStatus::SubmitToValidator => "Diajukan ke Validator",
            AktivitasStatus::SubmitToValidatorWilayah => "Diajukan ke Validator Wilayah",
            AktivitasStatus::RevisiPelaksana => "Revisi Pelaksana",
            AktivitasStatus::RevisiSatker => "Revisi Satker",
            AktivitasStatus::RevisiWilayah => "Revisi Wilayah",
            AktivitasStatus::SubmitToPusat => "Diajukan ke Pusat",
            AktivitasStatus::SubmitToPusatFromWilayah => "Diajukan ke Pusat dari Wilayah",
            AktivitasStatus::Selesai => "Selesai",
            AktivitasStatus::StartKejagung => "Mulai (Kejagung)",
            AktivitasStatus::StartNonKejagung => "Mulai (Non-Kejagung)",
        }
    }
}

// ============ Master Data Models ============

/// Master table: Jenis Pakaian Dinas (Uniform Types)
/// Example: PDH, PDL, Toga Jaksa
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JenisPakaianDinas {
    pub id: Uuid,
    pub nama: String,
    pub deskripsi: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl JenisPakaianDinas {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            nama: row.get("nama"),
            deskripsi: row.try_get("deskripsi").ok(),
            is_active: row.try_get("is_active").unwrap_or(true),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

/// Request DTO for creating/updating Jenis Pakaian Dinas
#[derive(Debug, Deserialize, Validate)]
pub struct CreateJenisPakaianDinasRequest {
    #[validate(length(
        min = 1,
        max = 255,
        message = "Nama harus diisi dan maksimal 255 karakter"
    ))]
    pub nama: String,
    pub deskripsi: Option<String>,
    #[serde(default = "default_true")]
    pub is_active: bool,
}

/// Master table: Spesifikasi Pakaian Dinas (Uniform Specifications)
/// Example: Kemeja PDH, Celana PDH, Sepatu Dinas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpesifikasiPakaianDinas {
    pub id: Uuid,
    pub jenis_pakaian_dinas_id: Uuid,
    pub nama: String,
    pub gender: String,
    pub ukuran_group: String,
    pub deskripsi: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Joined fields
    pub jenis_pakaian_nama: Option<String>,
}

impl SpesifikasiPakaianDinas {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            jenis_pakaian_dinas_id: row.get("jenis_pakaian_dinas_id"),
            nama: row.get("nama"),
            gender: row.get("gender"),
            ukuran_group: row.get("ukuran_group"),
            deskripsi: row.try_get("deskripsi").ok(),
            is_active: row.try_get("is_active").unwrap_or(true),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            jenis_pakaian_nama: row.try_get("jenis_pakaian_nama").ok(),
        }
    }
}

/// Request DTO for creating/updating Spesifikasi
#[derive(Debug, Deserialize, Validate)]
pub struct CreateSpesifikasiRequest {
    pub jenis_pakaian_dinas_id: Uuid,
    #[validate(length(min = 1, max = 255, message = "Nama harus diisi"))]
    pub nama: String,
    #[validate(length(min = 1, max = 10, message = "Gender harus diisi"))]
    pub gender: String, // L, P, or SEMUA
    #[validate(length(min = 1, max = 50, message = "Ukuran group harus diisi"))]
    pub ukuran_group: String, // BAJU, CELANA, or SEPATU
    pub deskripsi: Option<String>,
    #[serde(default = "default_true")]
    pub is_active: bool,
}

/// Photo attachment for specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpesifikasiFoto {
    pub id: Uuid,
    pub spesifikasi_id: Uuid,
    pub path: String,
    pub filename: String,
    pub created_at: DateTime<Utc>,
}

impl SpesifikasiFoto {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            spesifikasi_id: row.get("spesifikasi_id"),
            path: row.get("path"),
            filename: row.get("filename"),
            created_at: row.get("created_at"),
        }
    }
}

/// Master table: SubSpesifikasi Pakaian Dinas
/// Example: Kemeja Lengan Panjang, Kemeja Lengan Pendek
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubSpesifikasiPakaianDinas {
    pub id: Uuid,
    pub spesifikasi_id: Uuid,
    pub nama: String,
    pub gender: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Joined fields
    pub spesifikasi_nama: Option<String>,
}

impl SubSpesifikasiPakaianDinas {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            spesifikasi_id: row.get("spesifikasi_id"),
            nama: row.get("nama"),
            gender: row.get("gender"),
            is_active: row.try_get("is_active").unwrap_or(true),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            spesifikasi_nama: row.try_get("spesifikasi_nama").ok(),
        }
    }
}

/// Request DTO for creating SubSpesifikasi
#[derive(Debug, Deserialize, Validate)]
pub struct CreateSubSpesifikasiRequest {
    pub spesifikasi_id: Uuid,
    #[validate(length(min = 1, max = 255, message = "Nama harus diisi"))]
    pub nama: String,
    #[validate(length(min = 1, max = 10, message = "Gender harus diisi"))]
    pub gender: String,
    #[serde(default = "default_true")]
    pub is_active: bool,
}

/// Master table: Ukuran (Sizes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ukuran {
    pub ukuran: String,
    pub group: String,
    pub urutan: i32,
}

impl Ukuran {
    pub fn from_row(row: &Row) -> Self {
        Self {
            ukuran: row.get("ukuran"),
            group: row.get("group"),
            urutan: row.try_get("urutan").unwrap_or(0),
        }
    }
}

// ============ Transaction Models ============

/// Main request header for uniform procurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PengajuanPakaianDinas {
    pub id: Uuid,
    pub nama: String,
    pub deskripsi: Option<String>,
    pub tgl_mulai: Option<NaiveDate>,
    pub tgl_selesai: Option<NaiveDate>,
    pub is_reguler: bool,
    pub tahun: i32,
    pub pilihan_satker: String, // "all" or "sebagian"
    pub dengan_unit_kerja: bool,
    pub jenis_pakaian_dinas_id: Option<Uuid>,
    pub aktivitas_id: i32,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Joined/computed fields
    pub jenis_pakaian_nama: Option<String>,
    pub aktivitas_label: Option<String>,
    pub total_satker: Option<i64>,
    pub satker_selesai: Option<i64>,
}

impl PengajuanPakaianDinas {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            nama: row.get("nama"),
            deskripsi: row.try_get("deskripsi").ok(),
            tgl_mulai: row.try_get("tgl_mulai").ok(),
            tgl_selesai: row.try_get("tgl_selesai").ok(),
            is_reguler: row.try_get("is_reguler").unwrap_or(true),
            tahun: row.get("tahun"),
            pilihan_satker: row.get("pilihan_satker"),
            dengan_unit_kerja: row.try_get("dengan_unit_kerja").unwrap_or(false),
            jenis_pakaian_dinas_id: row.try_get("jenis_pakaian_dinas_id").ok(),
            aktivitas_id: row.get("aktivitas_id"),
            created_by: row.try_get("created_by").ok(),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            jenis_pakaian_nama: row.try_get("jenis_pakaian_nama").ok(),
            aktivitas_label: row.try_get("aktivitas_label").ok(),
            total_satker: row.try_get("total_satker").ok(),
            satker_selesai: row.try_get("satker_selesai").ok(),
        }
    }

    /// Check if the submission period is still open
    pub fn is_open(&self) -> bool {
        if !self.is_reguler {
            return true; // Non-regular requests are always open
        }
        match self.tgl_selesai {
            Some(end_date) => {
                let today = chrono::Local::now().date_naive();
                today <= end_date
            }
            None => true,
        }
    }
}

/// Request DTO for creating Pengajuan
#[derive(Debug, Deserialize, Validate)]
pub struct CreatePengajuanRequest {
    #[validate(length(min = 1, max = 255, message = "Nama pengajuan harus diisi"))]
    pub nama: String,
    pub deskripsi: Option<String>,
    pub tgl_mulai: Option<NaiveDate>,
    pub tgl_selesai: Option<NaiveDate>,
    #[serde(default = "default_true")]
    pub is_reguler: bool,
    pub tahun: Option<i32>,
    #[validate(length(min = 1, message = "Pilihan satker harus diisi"))]
    pub pilihan_satker: String,
    #[serde(default)]
    pub dengan_unit_kerja: bool,
    pub jenis_pakaian_dinas_id: Option<Uuid>,
    pub spesifikasi_ids: Vec<Uuid>,    // Selected specifications
    pub satker_ids: Option<Vec<Uuid>>, // Selected satkers (if pilihan_satker = "sebagian")
}

/// Selected satkers for a pengajuan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PengajuanSatkerTerpilih {
    pub pengajuan_id: Uuid,
    pub satker_id: Uuid,
    pub satker_pusat_id: Option<Uuid>,
    pub is_show_in_form: bool,
    // Joined fields
    pub satker_nama: Option<String>,
    pub satker_kode: Option<String>,
}

impl PengajuanSatkerTerpilih {
    pub fn from_row(row: &Row) -> Self {
        Self {
            pengajuan_id: row.get("pengajuan_id"),
            satker_id: row.get("satker_id"),
            satker_pusat_id: row.try_get("satker_pusat_id").ok(),
            is_show_in_form: row.try_get("is_show_in_form").unwrap_or(true),
            satker_nama: row.try_get("satker_nama").ok(),
            satker_kode: row.try_get("satker_kode").ok(),
        }
    }
}

/// Clothing items selected for a pengajuan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PengajuanPakaian {
    pub id: Uuid,
    pub pengajuan_id: Uuid,
    pub jenis_pakaian_id: Uuid,
    pub jenis_pakaian_nama: String,
    pub spesifikasi_id: Uuid,
    pub spesifikasi_nama: String,
    pub spesifikasi_ukuran_group: String,
    pub subspesifikasi_id: Option<Uuid>,
    pub subspesifikasi_nama: Option<String>,
    pub subspesifikasi_gender: Option<String>,
}

impl PengajuanPakaian {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            pengajuan_id: row.get("pengajuan_id"),
            jenis_pakaian_id: row.get("jenis_pakaian_id"),
            jenis_pakaian_nama: row.get("jenis_pakaian_nama"),
            spesifikasi_id: row.get("spesifikasi_id"),
            spesifikasi_nama: row.get("spesifikasi_nama"),
            spesifikasi_ukuran_group: row.get("spesifikasi_ukuran_group"),
            subspesifikasi_id: row.try_get("subspesifikasi_id").ok(),
            subspesifikasi_nama: row.try_get("subspesifikasi_nama").ok(),
            subspesifikasi_gender: row.try_get("subspesifikasi_gender").ok(),
        }
    }
}

/// Per-satker submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PengajuanSatker {
    pub id: Uuid,
    pub pengajuan_id: Uuid,
    pub satker_id: Uuid,
    pub satker_pusat_id: Option<Uuid>,
    pub id_kejati: Option<Uuid>,
    pub id_kejari: Option<Uuid>,
    pub id_cabjari: Option<Uuid>,
    pub aktivitas_id: i32,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Joined fields
    pub satker_nama: Option<String>,
    pub satker_kode: Option<String>,
    pub aktivitas_label: Option<String>,
    pub total_pegawai: Option<i64>,
}

impl PengajuanSatker {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            pengajuan_id: row.get("pengajuan_id"),
            satker_id: row.get("satker_id"),
            satker_pusat_id: row.try_get("satker_pusat_id").ok(),
            id_kejati: row.try_get("id_kejati").ok(),
            id_kejari: row.try_get("id_kejari").ok(),
            id_cabjari: row.try_get("id_cabjari").ok(),
            aktivitas_id: row.get("aktivitas_id"),
            created_by: row.try_get("created_by").ok(),
            updated_by: row.try_get("updated_by").ok(),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            satker_nama: row.try_get("satker_nama").ok(),
            satker_kode: row.try_get("satker_kode").ok(),
            aktivitas_label: row.try_get("aktivitas_label").ok(),
            total_pegawai: row.try_get("total_pegawai").ok(),
        }
    }
}

/// Employee data within a satker submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PengajuanSatkerPegawai {
    pub id: Uuid,
    pub pengajuan_satker_id: Uuid,
    pub nip: String,
    pub nama: String,
    pub pangkat: Option<String>,
    pub jabatan: Option<String>,
    pub eselon: Option<String>,
    pub jenis_kelamin: String,
    pub gol_kd: Option<String>,
    pub jenis: Option<String>, // Jaksa/TU/etc
    pub with_hijab: bool,
    pub created_at: DateTime<Utc>,
}

impl PengajuanSatkerPegawai {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            pengajuan_satker_id: row.get("pengajuan_satker_id"),
            nip: row.get("nip"),
            nama: row.get("nama"),
            pangkat: row.try_get("pangkat").ok(),
            jabatan: row.try_get("jabatan").ok(),
            eselon: row.try_get("eselon").ok(),
            jenis_kelamin: row.get("jenis_kelamin"),
            gol_kd: row.try_get("gol_kd").ok(),
            jenis: row.try_get("jenis").ok(),
            with_hijab: row.try_get("with_hijab").unwrap_or(false),
            created_at: row.get("created_at"),
        }
    }
}

/// Request DTO for creating/updating employee with sizes
#[derive(Debug, Deserialize, Validate)]
pub struct CreatePegawaiUkuranRequest {
    #[validate(length(min = 1, message = "NIP harus diisi"))]
    pub nip: String,
    #[validate(length(min = 1, message = "Nama harus diisi"))]
    pub nama: String,
    pub pangkat: Option<String>,
    pub jabatan: Option<String>,
    pub eselon: Option<String>,
    #[validate(length(min = 1, max = 1, message = "Jenis kelamin harus L atau P"))]
    pub jenis_kelamin: String,
    pub gol_kd: Option<String>,
    pub jenis: Option<String>,
    #[serde(default)]
    pub with_hijab: bool,
    pub ukuran: Vec<PegawaiUkuranItem>,
}

/// Size entry for a specific clothing item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PegawaiUkuranItem {
    pub pakaian_id: Uuid,
    pub ukuran: String,
}

/// Employee's clothing size within a submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PengajuanSatkerPegawaiUkuran {
    pub pengajuan_satker_id: Uuid,
    pub pegawai_id: Uuid,
    pub pakaian_id: Uuid,
    pub ukuran: String,
    // Joined fields
    pub pakaian_nama: Option<String>,
    pub ukuran_group: Option<String>,
}

impl PengajuanSatkerPegawaiUkuran {
    pub fn from_row(row: &Row) -> Self {
        Self {
            pengajuan_satker_id: row.get("pengajuan_satker_id"),
            pegawai_id: row.get("pegawai_id"),
            pakaian_id: row.get("pakaian_id"),
            ukuran: row.get("ukuran"),
            pakaian_nama: row.try_get("pakaian_nama").ok(),
            ukuran_group: row.try_get("ukuran_group").ok(),
        }
    }
}

/// Workflow activity history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PengajuanSatkerAktivitas {
    pub id: Uuid,
    pub pengajuan_satker_id: Uuid,
    pub aktivitas_id: i32,
    pub komentar: Option<String>,
    pub nip: Option<String>,
    pub nama: Option<String>,
    pub pangkat: Option<String>,
    pub jabatan: Option<String>,
    pub role: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl PengajuanSatkerAktivitas {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            pengajuan_satker_id: row.get("pengajuan_satker_id"),
            aktivitas_id: row.get("aktivitas_id"),
            komentar: row.try_get("komentar").ok(),
            nip: row.try_get("nip").ok(),
            nama: row.try_get("nama").ok(),
            pangkat: row.try_get("pangkat").ok(),
            jabatan: row.try_get("jabatan").ok(),
            role: row.try_get("role").ok(),
            created_at: row.get("created_at"),
        }
    }
}

/// Request DTO for workflow action (approve/reject)
#[derive(Debug, Deserialize, Validate)]
pub struct ValidatorActionRequest {
    pub pengajuan_satker_id: Uuid,
    pub aksi: String, // "approve" or "reject"
    pub komentar: Option<String>,
}

// ============ Persistent Employee Size Models ============

/// Persistent record of employee's uniform sizes
/// Updated when a request is approved
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PegawaiPakaianDinas {
    pub nip: String,
    pub nama: Option<String>,
    pub ukuran_baju: Option<String>,
    pub ukuran_celana: Option<String>,
    pub ukuran_sepatu: Option<String>,
    pub with_hijab: bool,
    pub pangkat: Option<String>,
    pub jabatan: Option<String>,
    pub status: Option<String>,
    pub last_pengajuan_satker_pegawai_id: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
}

impl PegawaiPakaianDinas {
    pub fn from_row(row: &Row) -> Self {
        Self {
            nip: row.get("nip"),
            nama: row.try_get("nama").ok(),
            ukuran_baju: row.try_get("ukuran_baju").ok(),
            ukuran_celana: row.try_get("ukuran_celana").ok(),
            ukuran_sepatu: row.try_get("ukuran_sepatu").ok(),
            with_hijab: row.try_get("with_hijab").unwrap_or(false),
            pangkat: row.try_get("pangkat").ok(),
            jabatan: row.try_get("jabatan").ok(),
            status: row.try_get("status").ok(),
            last_pengajuan_satker_pegawai_id: row.try_get("last_pengajuan_satker_pegawai_id").ok(),
            updated_at: row.get("updated_at"),
        }
    }
}

/// Request DTO for personal uniform sizes (self-service)
#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePersonalUkuranRequest {
    #[validate(length(min = 1, message = "Ukuran baju harus diisi"))]
    pub ukuran_baju: String,
    #[validate(length(min = 1, message = "Ukuran celana harus diisi"))]
    pub ukuran_celana: String,
    #[validate(length(min = 1, message = "Ukuran sepatu harus diisi"))]
    pub ukuran_sepatu: String,
    #[serde(default)]
    pub with_hijab: bool,
}

// ============ Report Models ============

/// Summary report by size
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaporanRekapUkuran {
    pub pakaian_nama: String,
    pub ukuran_group: String,
    pub ukuran: String,
    pub jumlah_laki: i64,
    pub jumlah_perempuan: i64,
    pub jumlah_total: i64,
}

impl LaporanRekapUkuran {
    pub fn from_row(row: &Row) -> Self {
        Self {
            pakaian_nama: row.get("pakaian_nama"),
            ukuran_group: row.get("ukuran_group"),
            ukuran: row.get("ukuran"),
            jumlah_laki: row.get("jumlah_laki"),
            jumlah_perempuan: row.get("jumlah_perempuan"),
            jumlah_total: row.get("jumlah_total"),
        }
    }
}

/// Detail list report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaporanDaftarPegawai {
    pub nip: String,
    pub nama: String,
    pub satker_nama: String,
    pub jabatan: Option<String>,
    pub pangkat: Option<String>,
    pub jenis_kelamin: String,
    pub gol_kd: Option<String>,
    pub jenis: Option<String>,
    pub eselon: Option<String>,
    pub ukuran_baju: Option<String>,
    pub ukuran_celana: Option<String>,
    pub ukuran_sepatu: Option<String>,
    pub with_hijab: bool,
}

impl LaporanDaftarPegawai {
    pub fn from_row(row: &Row) -> Self {
        Self {
            nip: row.get("nip"),
            nama: row.get("nama"),
            satker_nama: row.get("satker_nama"),
            jabatan: row.try_get("jabatan").ok(),
            pangkat: row.try_get("pangkat").ok(),
            jenis_kelamin: row.get("jenis_kelamin"),
            gol_kd: row.try_get("gol_kd").ok(),
            jenis: row.try_get("jenis").ok(),
            eselon: row.try_get("eselon").ok(),
            ukuran_baju: row.try_get("ukuran_baju").ok(),
            ukuran_celana: row.try_get("ukuran_celana").ok(),
            ukuran_sepatu: row.try_get("ukuran_sepatu").ok(),
            with_hijab: row.try_get("with_hijab").unwrap_or(false),
        }
    }
}

/// Report filter options
#[derive(Debug, Clone, Deserialize, Default)]
pub struct LaporanFilter {
    pub pengajuan_id: Option<Uuid>,
    pub tahun: Option<i32>,
    pub satker_id: Option<Uuid>,
    pub jenis_kelamin: Option<String>,
    pub eselon: Option<String>,
    pub jenis: Option<String>,
}

// ============ MySIMKARI Integration Models ============

/// Employee data from MySIMKARI integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MysimkariPegawai {
    pub id: i32,
    pub nama: String,
    pub nip: String,
    pub no_hp: Option<String>,
    pub email_dinas: Option<String>,
    pub bidang: Option<String>,
    pub foto: Option<String>,
    pub jk: String,
    pub agama: Option<String>,
    pub nrp: Option<String>,
    pub jabatan: Option<String>,
    pub golpang: Option<String>,
    pub jenis_jabatan_terakhir: Option<String>,
    pub jabat_tmt: Option<String>,
    pub eselon: Option<String>,
    pub nama_satker: Option<String>,
    pub gol_kd: Option<String>,
    pub satker_id: Option<Uuid>,
}

impl MysimkariPegawai {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            nama: row.get("nama"),
            nip: row.get("nip"),
            no_hp: row.try_get("no_hp").ok(),
            email_dinas: row.try_get("email_dinas").ok(),
            bidang: row.try_get("bidang").ok(),
            foto: row.try_get("foto").ok(),
            jk: row.try_get("jk").unwrap_or_else(|_| "L".to_string()),
            agama: row.try_get("agama").ok(),
            nrp: row.try_get("nrp").ok(),
            jabatan: row.try_get("jabatan").ok(),
            golpang: row.try_get("golpang").ok(),
            jenis_jabatan_terakhir: row.try_get("jenis_jabatan_terakhir").ok(),
            jabat_tmt: row.try_get("jabat_tmt").ok(),
            eselon: row.try_get("eselon").ok(),
            nama_satker: row.try_get("nama_satker").ok(),
            gol_kd: row.try_get("gol_kd").ok(),
            satker_id: row.try_get("satker_id").ok(),
        }
    }

    /// Convert to pegawai for pengajuan submission
    pub fn to_pengajuan_pegawai(
        &self,
        existing_sizes: Option<&PegawaiPakaianDinas>,
    ) -> CreatePegawaiUkuranRequest {
        CreatePegawaiUkuranRequest {
            nip: self.nip.clone(),
            nama: self.nama.clone(),
            pangkat: self.golpang.clone(),
            jabatan: self.jabatan.clone(),
            eselon: self.eselon.clone(),
            jenis_kelamin: self.jk.clone(),
            gol_kd: self.gol_kd.clone(),
            jenis: self.jenis_jabatan_terakhir.clone(),
            with_hijab: existing_sizes.map(|s| s.with_hijab).unwrap_or(false),
            ukuran: vec![], // Will be filled from existing sizes or user input
        }
    }
}

// ============ Utility Functions ============

fn default_true() -> bool {
    true
}

// ============ Tests ============

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gender_conversion() {
        assert_eq!(Gender::from_str("L"), Gender::L);
        assert_eq!(Gender::from_str("P"), Gender::P);
        assert_eq!(Gender::from_str("SEMUA"), Gender::Semua);
        assert_eq!(Gender::from_str("semua"), Gender::Semua);
        assert_eq!(Gender::from_str("unknown"), Gender::Semua);
    }

    #[test]
    fn test_ukuran_group_conversion() {
        assert_eq!(UkuranGroup::from_str("BAJU"), UkuranGroup::Baju);
        assert_eq!(UkuranGroup::from_str("CELANA"), UkuranGroup::Celana);
        assert_eq!(UkuranGroup::from_str("SEPATU"), UkuranGroup::Sepatu);
        assert_eq!(UkuranGroup::from_str("baju"), UkuranGroup::Baju);
    }

    #[test]
    fn test_aktivitas_status_conversion() {
        assert_eq!(
            AktivitasStatus::from_i32(1000),
            Some(AktivitasStatus::Input)
        );
        assert_eq!(
            AktivitasStatus::from_i32(1008),
            Some(AktivitasStatus::Selesai)
        );
        assert_eq!(AktivitasStatus::from_i32(9999), None);
        assert_eq!(AktivitasStatus::Input.to_i32(), 1000);
    }

    #[test]
    fn test_aktivitas_label() {
        assert_eq!(AktivitasStatus::Input.label(), "Input");
        assert_eq!(AktivitasStatus::Selesai.label(), "Selesai");
    }

    #[test]
    fn test_pengajuan_is_open() {
        let mut pengajuan = PengajuanPakaianDinas {
            id: Uuid::new_v4(),
            nama: "Test".to_string(),
            deskripsi: None,
            tgl_mulai: None,
            tgl_selesai: None,
            is_reguler: true,
            tahun: 2026,
            pilihan_satker: "all".to_string(),
            dengan_unit_kerja: false,
            jenis_pakaian_dinas_id: None,
            aktivitas_id: 1000,
            created_by: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            jenis_pakaian_nama: None,
            aktivitas_label: None,
            total_satker: None,
            satker_selesai: None,
        };

        // No end date = open
        assert!(pengajuan.is_open());

        // Non-regular = always open
        pengajuan.is_reguler = false;
        pengajuan.tgl_selesai = Some(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap());
        assert!(pengajuan.is_open());

        // Regular with past end date = closed
        pengajuan.is_reguler = true;
        assert!(!pengajuan.is_open());

        // Regular with future end date = open
        pengajuan.tgl_selesai = Some(NaiveDate::from_ymd_opt(2030, 12, 31).unwrap());
        assert!(pengajuan.is_open());
    }
}
