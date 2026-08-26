use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
use uuid::Uuid;
use validator::Validate;

use super::*;

impl<'a> TryFrom<&'a Row> for KebutuhanBmnSummary {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(row: &'a Row) -> Result<Self, Self::Error> {
        Ok(Self::from_row(row))
    }
}

// ============================================================================
// Request DTOs
// ============================================================================

/// Request to create a new BMN needs request
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreatePengajuanRequest {
    #[validate(length(min = 3, max = 255, message = "Nama harus antara 3-255 karakter"))]
    pub nama: String,

    pub deskripsi: Option<String>,

    #[validate(range(min = 2020, max = 2100, message = "Tahun harus antara 2020-2100"))]
    pub tahun: i32,

    pub tgl_mulai: NaiveDate,
    pub tgl_selesai: NaiveDate,

    #[serde(default)]
    pub pilihan_satker: Option<String>,

    /// `kode_satker` Kejaksaan Tinggi penaung (match
    /// `integrasi.v_satker_wilayah.wilayah_code`). BUKAN
    /// `mysimkari_satker.wilayah` — kolom itu berisi I/II/III, 15 Kejati per nilai.
    /// WAJIB jika `pilihan_satker = "wilayah"`. Server akan resolve semua
    /// satker di wilayah tsb otomatis (V029, Fase 1.7).
    #[serde(default)]
    pub wilayah_id: Option<String>,

    #[serde(default)]
    pub satker_ids: Vec<String>,

    #[serde(default)]
    pub asset_types: Vec<CreateAssetTypeRequest>,

    /// V029 (Fase 1.6): Allowed-list BMN. Validator Pusat tetapkan saat
    /// create periode; Operator Satker hanya boleh input barang dari
    /// daftar ini. Jika kosong → tidak ada whitelist (legacy behaviour,
    /// operator bebas — dipertahankan utk backward compat data lama).
    #[serde(default)]
    pub bmn_referensi_diizinkan: Vec<CreateBmnReferensiRequest>,
}
/// V029 (Fase 1.6): Satu entry allowed BMN (request DTO).
#[derive(Debug, Clone, Deserialize)]
pub struct CreateBmnReferensiRequest {
    pub kode_barang: String,
    pub nama_barang: String,
    pub keterangan: Option<String>,
}
/// Request to create asset type for a request
#[derive(Debug, Clone, Deserialize)]
pub struct CreateAssetTypeRequest {
    pub kode_barang: Option<String>,
    pub nm_barang: Option<String>,
    pub ms_jenis_asset_id: Option<i32>,
    pub keterangan: Option<String>,
}
/// Request to update a BMN needs request
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdatePengajuanRequest {
    #[validate(length(min = 3, max = 255, message = "Nama harus antara 3-255 karakter"))]
    pub nama: Option<String>,

    pub deskripsi: Option<String>,
    pub tgl_mulai: Option<NaiveDate>,
    pub tgl_selesai: Option<NaiveDate>,
    pub pilihan_satker: Option<String>,
    pub wilayah_id: Option<String>,

    /// For optimistic locking
    pub version: i32,
}
/// Request to add goods to a satker
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateBarangRequest {
    #[validate(length(min = 1, max = 255, message = "Nama barang harus diisi"))]
    pub nama: String,

    pub kode_barang: Option<String>,

    #[validate(range(min = 1, message = "Jumlah minimal 1"))]
    pub jumlah: i32,

    #[serde(default = "default_satuan")]
    pub satuan: String,

    pub alasan: Option<String>,
    pub keterangan: Option<String>,

    #[serde(default)]
    pub file_pendukung: Vec<String>,
}
fn default_satuan() -> String {
    "Unit".to_string()
}
/// Request to update goods approval
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateBarangApprovalRequest {
    pub jml_setuju: i32,
    pub keterangan: Option<String>,
}
/// Request to set priority for goods
#[derive(Debug, Clone, Deserialize)]
pub struct SetPrioritasRequest {
    pub items: Vec<PrioritasItem>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct PrioritasItem {
    pub barang_id: Uuid,
    pub prioritas: i32,
    pub skor: Option<f64>,
}
/// Request to transition workflow status
#[derive(Debug, Clone, Deserialize)]
pub struct WorkflowTransitionRequest {
    pub target_status: i32,
    pub komentar: Option<String>,
}
/// Request for operator satker to submit kebutuhan with attachments
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct SubmitKebutuhanSatkerRequest {
    pub catatan_satker: Option<String>,
    #[validate(length(min = 1, message = "Surat permohonan wajib dilampirkan"))]
    pub lampiran_surat_permohonan: String,
    #[serde(default)]
    pub lampiran_pendukung: Vec<LampiranItem>,
}
/// Lampiran/attachment item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LampiranItem {
    pub nama: String,
    pub url: String,
    pub tipe: Option<String>,
}
/// Request for validator wilayah action
#[derive(Debug, Clone, Deserialize)]
pub struct ValidatorWilayahActionRequest {
    /// "forward" or "return"
    pub aksi: String,
    pub catatan: Option<String>,
}
/// Request for validator pusat decision
#[derive(Debug, Clone, Deserialize)]
pub struct ValidatorPusatKeputusanRequest {
    /// true = approve, false = reject
    pub is_approved: bool,
    pub alasan: Option<String>,
    #[serde(default)]
    pub override_darurat: Option<bool>,
    #[serde(default)]
    pub override_reason: Option<String>,
}
// ============================================================================
// Filter/Query Parameters
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct PengajuanFilter {
    pub tahun: Option<i32>,
    pub status_kode: Option<i32>,
    pub satker_id: Option<String>,
    pub search: Option<String>,
}
/// Filter for the Laporan Kebutuhan BMN recap (E-5). `status_kode` matches the
/// **satker-level** status, which is what the report lists per row.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RekapLaporanFilter {
    pub tahun: Option<i32>,
    pub status_kode: Option<i32>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct BarangFilter {
    pub kode_barang: Option<String>,
    pub prioritas_min: Option<i32>,
    pub search: Option<String>,
}
// ============================================================================
// Batch Operations
// ============================================================================

/// Request for batch approval of kebutuhan
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct BatchApproveRequest {
    #[validate(length(min = 1, max = 500, message = "Batch size must be between 1 and 500"))]
    pub kebutuhan_ids: Vec<Uuid>,

    pub komentar: Option<String>,
}
/// Request for batch rejection of kebutuhan
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct BatchRejectRequest {
    #[validate(length(min = 1, max = 500, message = "Batch size must be between 1 and 500"))]
    pub kebutuhan_ids: Vec<Uuid>,

    #[validate(length(min = 10, message = "Rejection reason must be at least 10 characters"))]
    pub komentar: String,
}
/// Request for batch status update
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct BatchUpdateStatusRequest {
    #[validate(length(min = 1, max = 500, message = "Batch size must be between 1 and 500"))]
    pub kebutuhan_ids: Vec<Uuid>,

    pub target_status: i32,
    pub komentar: Option<String>,
}
