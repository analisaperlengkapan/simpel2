use chrono::NaiveDate;
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

// ============================================================================
// Request DTOs
// ============================================================================

/// Request to create a new BMN usage permit
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateIzinPemakaianRequest {
    #[validate(length(min = 1, message = "NIP pegawai harus diisi"))]
    pub pegawai_nip: String,

    pub pegawai_nama: String,
    pub pegawai_satker_id: Uuid,
    pub pegawai_satker_nama: String,
    pub pegawai_jabatan: Option<String>,
    pub pegawai_golongan: Option<String>,
    pub pegawai_pangkat: Option<String>,
    pub pegawai_unit_kerja: Option<String>,
    pub foto_pegawai: Option<String>,

    #[validate(length(min = 1, message = "Jenis BMN harus diisi"))]
    pub jenis_bmn: String,

    // Primary BMN (kept for backward compat + single-BMN cases)
    #[validate(length(min = 1, message = "NUP BMN harus diisi"))]
    pub bmn_nup: String,

    pub bmn_kode_barang: String,
    pub bmn_nama_barang: String,
    pub bmn_merk: Option<String>,
    pub bmn_tahun_perolehan: Option<i32>,

    // Vehicle-specific
    pub no_polisi: Option<String>,
    pub no_bpkb: Option<String>,
    pub no_stnk: Option<String>,
    pub no_rangka: Option<String>,
    pub no_mesin: Option<String>,

    // Housing-specific
    pub alamat: Option<String>,
    pub luas_tanah: Option<f64>,
    pub luas_bangunan: Option<f64>,

    // Laptop-specific
    pub serial_number: Option<String>,
    pub spesifikasi: Option<serde_json::Value>,

    pub tanggal_mulai: NaiveDate,
    pub tanggal_selesai: NaiveDate,

    #[validate(length(min = 10, message = "Keperluan minimal 10 karakter"))]
    pub keperluan: String,

    pub lokasi_pemakaian: Option<String>,
    pub file_pendukung: Option<serde_json::Value>,

    // Renewal
    pub is_renewal: Option<bool>,
    pub previous_permit_id: Option<Uuid>,

    // Additional BMN items (multi-BMN per pegawai)
    #[serde(default)]
    pub additional_bmn_items: Vec<CreateBmnItemRequest>,
}
/// Request to add a BMN item to a permit
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateBmnItemRequest {
    #[validate(length(min = 1, message = "NUP BMN harus diisi"))]
    pub bmn_nup: String,
    pub bmn_kode_barang: String,
    pub bmn_nama_barang: String,
    pub bmn_merk: Option<String>,
    pub bmn_tahun_perolehan: Option<i32>,
    pub bmn_kondisi: Option<String>,
    pub detail_bmn: Option<serde_json::Value>,
    pub keterangan: Option<String>,
}
/// Request to upload signed PDF
#[derive(Debug, Clone, Deserialize)]
pub struct UploadSignedPdfRequest {
    pub signed_pdf_url: String,
}
/// Request to generate konsep surat. Both DOCX and PDF are always produced
/// side-by-side; callers pick which artifact to download via the URL
/// extension (`konsep-surat.docx` vs `konsep-surat.pdf`).
#[derive(Debug, Clone, Deserialize, Default)]
pub struct GenerateKonsepSuratRequest {}
/// Request to update an existing permit (only in DRAFT status)
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdateIzinPemakaianRequest {
    pub tanggal_mulai: Option<NaiveDate>,
    pub tanggal_selesai: Option<NaiveDate>,

    #[validate(length(min = 10, message = "Keperluan minimal 10 karakter"))]
    pub keperluan: Option<String>,

    pub lokasi_pemakaian: Option<String>,
    pub file_pendukung: Option<serde_json::Value>,
}
/// Request to transition workflow status
#[derive(Debug, Clone, Deserialize)]
pub struct WorkflowTransitionRequest {
    pub target_status: String,
    pub catatan: Option<String>,
}
/// Request to revoke a permit
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct RevokePermitRequest {
    #[validate(length(min = 10, message = "Alasan pencabutan minimal 10 karakter"))]
    pub alasan: String,
}
// ============================================================================
// V035 (Fase 1.5): DTO untuk alur internal-satker 3-step.
// ============================================================================

/// Validator Satker / Approver Satker forward (lanjutkan ke step berikut).
///
/// `expected_version` adalah optimistic-lock token — FE harus mengirim
/// `permit.version` yg dia baca; backend menolak (409) jika sudah berubah.
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct SatkerForwardRequest {
    pub expected_version: i32,
    pub catatan: Option<String>,
}
/// Validator Satker / Approver Satker return (kembalikan ke Operator).
/// Catatan wajib + minimal 10 karakter agar Operator paham apa yg direvisi.
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct SatkerReturnRequest {
    pub expected_version: i32,
    #[validate(length(min = 10, message = "Catatan revisi minimal 10 karakter"))]
    pub catatan: String,
}
/// Operator re-submit setelah revisi. Tidak perlu catatan (perubahan data
/// sudah ter-record via PUT permit sebelumnya).
#[derive(Debug, Clone, Deserialize)]
pub struct OperatorResubmitRequest {
    pub expected_version: i32,
}
/// Request to renew a permit
#[derive(Debug, Clone, Deserialize)]
pub struct RenewPermitRequest {
    pub tanggal_mulai: NaiveDate,
    pub tanggal_selesai: NaiveDate,
    pub keperluan: String,
}
/// Query parameters for listing permits
#[derive(Debug, Clone, Deserialize)]
pub struct ListPermitsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub status: Option<String>,
    pub jenis_bmn: Option<String>,
    pub pegawai_nip: Option<String>,
    pub satker_id: Option<Uuid>,
    pub search: Option<String>,
}
/// Query parameters for monitoring dashboard
#[derive(Debug, Clone, Deserialize)]
pub struct MonitoringDashboardQuery {
    pub satker_id: Option<Uuid>,
    pub jenis_bmn: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}
