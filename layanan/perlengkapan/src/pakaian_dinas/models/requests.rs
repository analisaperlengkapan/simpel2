use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

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
    pub spesifikasi_ids: Vec<Uuid>, // Selected specifications
    /// Selected satkers (if `pilihan_satker = "sebagian"`), as MySIMKARI
    /// `kode_satker` — not the bigint surrogate id (V006/#94).
    pub satker_ids: Option<Vec<String>>,
    /// Wilayah Kejaksaan Tinggi (#19) — wajib jika `pilihan_satker = "wilayah"`.
    /// Satker di-resolve otomatis dari `integrasi.mysimkari_satker.wilayah`.
    #[serde(default)]
    pub wilayah_id: Option<String>,
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
/// Request DTO for workflow action (approve/reject)
#[derive(Debug, Deserialize, Validate)]
pub struct ValidatorActionRequest {
    pub pengajuan_satker_id: Uuid,
    pub aksi: String, // "approve" or "reject"
    pub komentar: Option<String>,
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
/// Full profile upsert — used by satker operators to set the reporting
/// fields that MySIMKARI does not carry (eselon, gender, jenis pegawai,
/// hijab flag, mapped unit kerja) alongside sizes. Called during the
/// pakaian dinas wizard and by the admin bulk-import.
#[derive(Debug, Deserialize, Validate)]
pub struct UpsertPegawaiProfileRequest {
    #[validate(length(min = 1, max = 30, message = "NIP tidak valid"))]
    pub nip: String,
    pub nama: Option<String>,
    pub pangkat: Option<String>,
    pub jabatan: Option<String>,
    pub eselon: Option<String>,
    /// "L" or "P"
    pub jenis_kelamin: Option<String>,
    /// "TU", "Jaksa", etc.
    pub jenis_pegawai: Option<String>,
    #[serde(default)]
    pub with_hijab: bool,
    pub mapped_unit_kerja: Option<String>,
    pub kode_satker: Option<String>,
    pub ukuran_baju: Option<String>,
    pub ukuran_celana: Option<String>,
    pub ukuran_sepatu: Option<String>,
}
/// Report filter options
#[derive(Debug, Clone, Deserialize, Default)]
pub struct LaporanFilter {
    pub pengajuan_id: Option<Uuid>,
    pub tahun: Option<i32>,
    /// MySIMKARI `kode_satker` (V006/#94).
    pub satker_id: Option<String>,
    pub jenis_kelamin: Option<String>,
    pub eselon: Option<String>,
    pub jenis: Option<String>,
}
// ============ Utility Functions ============

fn default_true() -> bool {
    true
}
