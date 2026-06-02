//! # Pemakaian BMN Data Models
//!
//! Data models for BMN usage permit system.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// ============================================================================
// Workflow Status Enum
// ============================================================================

/// Workflow status codes for BMN usage permits
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
#[derive(Default)]
pub enum PemakaianBmnStatus {
    /// New permit request in draft state
    #[default]
    Draft = 3000,
    /// Submitted to Validator Satker (internal satker validation step)
    Submitted = 3001,
    /// Approved by Approver Satker (Pengguna Barang Satker)
    Approved = 3002,
    /// Rejected (legacy terminal — alur baru pakai RevisiOperator)
    Rejected = 3003,
    /// Permit is active
    Active = 3004,
    /// Permit has expired
    Expired = 3005,
    /// Permit was revoked
    Revoked = 3006,
    /// Request cancelled
    Cancelled = 3007,
    /// V035 (Fase 1.5): forwarded by Validator Satker, awaiting Approver Satker
    SubmittedApproverSatker = 3010,
    /// V035 (Fase 1.5): returned to Operator for revision (catatan wajib)
    RevisiOperator = 3011,
}

impl PemakaianBmnStatus {
    /// Convert from database integer code
    pub fn from_code(code: i32) -> Option<Self> {
        match code {
            3000 => Some(Self::Draft),
            3001 => Some(Self::Submitted),
            3002 => Some(Self::Approved),
            3003 => Some(Self::Rejected),
            3004 => Some(Self::Active),
            3005 => Some(Self::Expired),
            3006 => Some(Self::Revoked),
            3007 => Some(Self::Cancelled),
            3010 => Some(Self::SubmittedApproverSatker),
            3011 => Some(Self::RevisiOperator),
            _ => None,
        }
    }

    /// Convert to database integer code
    pub fn to_code(self) -> i32 {
        self as i32
    }

    /// Get human-readable label
    pub fn label(&self) -> &'static str {
        match self {
            Self::Draft => "Draft",
            Self::Submitted => "Menunggu Validator Satker",
            Self::SubmittedApproverSatker => "Menunggu Approver Satker",
            Self::RevisiOperator => "Revisi Operator",
            Self::Approved => "Disetujui",
            Self::Rejected => "Ditolak",
            Self::Active => "Aktif",
            Self::Expired => "Kadaluarsa",
            Self::Revoked => "Dicabut",
            Self::Cancelled => "Dibatalkan",
        }
    }

    /// Convert status to workflow engine state name
    pub fn to_state_name(&self) -> &'static str {
        match self {
            Self::Draft => "DRAFT",
            Self::Submitted => "SUBMITTED",
            Self::SubmittedApproverSatker => "SUBMITTED_APPROVER_SATKER",
            Self::RevisiOperator => "REVISI_OPERATOR",
            Self::Approved => "APPROVED",
            Self::Rejected => "REJECTED",
            Self::Active => "ACTIVE",
            Self::Expired => "EXPIRED",
            Self::Revoked => "REVOKED",
            Self::Cancelled => "CANCELLED",
        }
    }

    /// Convert workflow engine state name to status
    pub fn from_state_name(name: &str) -> Option<Self> {
        match name {
            "DRAFT" => Some(Self::Draft),
            "SUBMITTED" => Some(Self::Submitted),
            "SUBMITTED_APPROVER_SATKER" => Some(Self::SubmittedApproverSatker),
            "REVISI_OPERATOR" => Some(Self::RevisiOperator),
            "APPROVED" => Some(Self::Approved),
            "REJECTED" => Some(Self::Rejected),
            "ACTIVE" => Some(Self::Active),
            "EXPIRED" => Some(Self::Expired),
            "REVOKED" => Some(Self::Revoked),
            "CANCELLED" => Some(Self::Cancelled),
            _ => None,
        }
    }

    /// Check if transition to target status is allowed.
    ///
    /// V035 (Fase 1.5): alur baru = `Draft → Submitted (ValidatorSatker)
    /// → SubmittedApproverSatker → Approved → Active`. `Submitted/SubmittedApproverSatker
    /// → RevisiOperator` (catatan wajib). `RevisiOperator → Submitted` (re-submit).
    ///
    /// Legacy path `Submitted → Approved` direct masih diizinkan agar record
    /// `approved_via_legacy_flow` tidak putus di tengah jalan saat ada migrasi
    /// data. Handler baru tidak boleh memanggil transisi ini.
    pub fn can_transition_to(&self, target: Self) -> bool {
        use PemakaianBmnStatus::*;
        match self {
            Draft => matches!(target, Submitted | Cancelled),
            // Alur baru: ValidatorSatker → ApproverSatker | RevisiOperator.
            // Alur legacy: Submitted → Approved (direct) tetap valid utk
            // back-compat; ditolak di layer handler utk record baru.
            Submitted => matches!(
                target,
                SubmittedApproverSatker | RevisiOperator | Approved | Rejected
            ),
            SubmittedApproverSatker => matches!(target, Approved | RevisiOperator),
            RevisiOperator => matches!(target, Submitted | Cancelled),
            Approved => matches!(target, Active),
            Active => matches!(target, Expired | Revoked),
            Rejected | Expired | Revoked | Cancelled => false,
        }
    }
}

// ============================================================================
// BMN Type Enum
// ============================================================================

/// Types of BMN that can be requested for usage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JenisBmn {
    /// Motor vehicle (car, motorcycle)
    KendaraanBermotor,
    /// Government housing
    RumahNegara,
    /// Laptop/computer
    Laptop,
    /// Other equipment
    Lainnya,
}

impl JenisBmn {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::KendaraanBermotor => "KENDARAAN_BERMOTOR",
            Self::RumahNegara => "RUMAH_NEGARA",
            Self::Laptop => "LAPTOP",
            Self::Lainnya => "LAINNYA",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "KENDARAAN_BERMOTOR" => Some(Self::KendaraanBermotor),
            "RUMAH_NEGARA" => Some(Self::RumahNegara),
            "LAPTOP" => Some(Self::Laptop),
            "LAINNYA" => Some(Self::Lainnya),
            _ => None,
        }
    }

    /// Get required fields for this BMN type
    pub fn required_fields(&self) -> Vec<&'static str> {
        match self {
            Self::KendaraanBermotor => vec!["no_polisi", "merk", "tahun"],
            Self::RumahNegara => vec!["alamat", "luas_tanah", "luas_bangunan"],
            Self::Laptop => vec!["merk", "serial_number"],
            Self::Lainnya => vec![],
        }
    }
}

// ============================================================================
// Main Entity: Izin Pemakaian BMN
// ============================================================================

/// Main entity for BMN usage permit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IzinPemakaianBmn {
    pub id: Uuid,
    pub nomor_izin: Option<String>,

    // Pegawai Information
    pub pegawai_nip: String,
    pub pegawai_nama: String,
    pub pegawai_satker_id: Uuid,
    pub pegawai_satker_nama: String,
    pub pegawai_jabatan: Option<String>,
    pub pegawai_golongan: Option<String>,
    pub pegawai_pangkat: Option<String>,
    pub pegawai_unit_kerja: Option<String>,
    pub foto_pegawai: Option<String>,

    // BMN Information (primary, kept for backward compat)
    pub jenis_bmn: String,
    pub bmn_nup: String,
    pub bmn_kode_barang: String,
    pub bmn_nama_barang: String,
    pub bmn_merk: Option<String>,
    pub bmn_tahun_perolehan: Option<i32>,

    // Vehicle-specific fields
    pub no_polisi: Option<String>,
    pub no_bpkb: Option<String>,
    pub no_stnk: Option<String>,
    pub no_rangka: Option<String>,
    pub no_mesin: Option<String>,

    // Housing-specific fields
    pub alamat: Option<String>,
    pub luas_tanah: Option<f64>,
    pub luas_bangunan: Option<f64>,

    // Laptop-specific fields
    pub serial_number: Option<String>,
    pub spesifikasi: Option<serde_json::Value>,

    // Permit Details
    pub tanggal_mulai: NaiveDate,
    pub tanggal_selesai: NaiveDate,
    pub keperluan: String,
    pub lokasi_pemakaian: Option<String>,

    // Renewal
    pub is_renewal: bool,
    pub previous_permit_id: Option<Uuid>,

    // Supporting Documents
    pub file_pendukung: Option<serde_json::Value>,

    // Document Generation & Upload
    pub document_id: Option<Uuid>,
    pub document_url: Option<String>,
    pub konsep_surat_url: Option<String>, // Generated DOCX concept (editable)
    pub konsep_surat_generated_at: Option<DateTime<Utc>>,
    pub konsep_surat_pdf_url: Option<String>, // Generated PDF concept (final, side-by-side with DOCX)
    pub konsep_surat_pdf_generated_at: Option<DateTime<Utc>>,
    pub signed_pdf_url: Option<String>, // Uploaded signed PDF
    pub signed_pdf_uploaded_at: Option<DateTime<Utc>>,
    pub is_completed: bool,

    // Workflow Status
    pub status: String,
    pub catatan_approval: Option<String>,
    pub catatan_revocation: Option<String>,

    // Approval Information (legacy field — Pimpinan langsung)
    pub approved_by: Option<Uuid>,
    pub approved_by_nama: Option<String>,
    pub approved_at: Option<DateTime<Utc>>,

    // V035 (Fase 1.5): Internal-satker 3-step approval audit fields
    pub validator_satker_id: Option<Uuid>,
    pub validator_satker_nama: Option<String>,
    pub tanggal_validasi_satker: Option<DateTime<Utc>>,
    pub catatan_validator_satker: Option<String>,
    pub approver_satker_id: Option<Uuid>,
    pub approver_satker_nama: Option<String>,
    pub tanggal_approval_satker: Option<DateTime<Utc>>,
    pub catatan_approver_satker: Option<String>,
    pub approved_via_legacy_flow: bool,
    pub version: i32,

    // Revocation Information
    pub revoked_by: Option<Uuid>,
    pub revoked_by_nama: Option<String>,
    pub revoked_at: Option<DateTime<Utc>>,

    // Audit Fields
    pub created_by: Uuid,
    pub created_by_nama: String,
    pub updated_by: Option<Uuid>,
    pub updated_by_nama: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // Multi-BMN items (loaded separately)
    #[serde(default)]
    pub bmn_items: Vec<PemakaianBmnItem>,
}

// ============================================================================
// Entity: Pemakaian BMN Item (Multi-BMN per permit)
// ============================================================================

/// Individual BMN item in a usage permit (supports multiple BMN per pegawai)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PemakaianBmnItem {
    pub id: Uuid,
    pub izin_pemakaian_id: Uuid,
    pub bmn_nup: String,
    pub bmn_kode_barang: String,
    pub bmn_nama_barang: String,
    pub bmn_merk: Option<String>,
    pub bmn_tahun_perolehan: Option<i32>,
    pub bmn_kondisi: Option<String>,
    pub detail_bmn: serde_json::Value, // Type-specific details
    pub keterangan: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PemakaianBmnItem {
    pub fn from_row(row: &tokio_postgres::Row) -> Self {
        Self {
            id: row.get("id"),
            izin_pemakaian_id: row.get("izin_pemakaian_id"),
            bmn_nup: row.get("bmn_nup"),
            bmn_kode_barang: row.get("bmn_kode_barang"),
            bmn_nama_barang: row.get("bmn_nama_barang"),
            bmn_merk: row.try_get("bmn_merk").ok().flatten(),
            bmn_tahun_perolehan: row.try_get("bmn_tahun_perolehan").ok().flatten(),
            bmn_kondisi: row.try_get("bmn_kondisi").ok().flatten(),
            detail_bmn: row.try_get("detail_bmn").unwrap_or(serde_json::json!({})),
            keterangan: row.try_get("keterangan").ok().flatten(),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

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

// ============================================================================
// Response DTOs
// ============================================================================

/// Information about allowed workflow transitions
#[derive(Debug, Clone, Serialize)]
pub struct WorkflowTransitionInfo {
    pub status: String,
    pub label: String,
    pub requires_comment: bool,
}

/// Response with permit and allowed transitions
#[derive(Debug, Clone, Serialize)]
pub struct IzinPemakaianDetailResponse {
    #[serde(flatten)]
    pub izin: IzinPemakaianBmn,
    pub allowed_transitions: Vec<WorkflowTransitionInfo>,
    pub days_until_expiry: Option<i64>,
    pub is_expiring_soon: bool,
    pub can_generate_konsep: bool,
    pub can_upload_signed_pdf: bool,
}

/// Paginated list response
#[derive(Debug, Clone, Serialize)]
pub struct PaginatedPermitsResponse {
    pub data: Vec<IzinPemakaianBmn>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

/// BMN availability check response
#[derive(Debug, Clone, Serialize)]
pub struct BmnAvailabilityResponse {
    pub bmn_nup: String,
    pub is_available: bool,
    pub active_permit_id: Option<Uuid>,
    pub active_permit_holder: Option<String>,
    pub active_permit_expires: Option<NaiveDate>,
}

// ─── Fase 1.11: Cek pegawai + cek BMN (with period) ──────────────────────

/// Info pegawai dari `integrasi.mysimkari_pegawai` (cache MySIMKARI).
#[derive(Debug, Clone, Serialize)]
pub struct PegawaiInfo {
    pub nip: String,
    pub nama: Option<String>,
    pub jabatan: Option<String>,
    pub pangkat: Option<String>,
    pub satker_id: Option<String>,
    pub nama_satker: Option<String>,
    pub foto: Option<String>,
}

/// Hasil cek pegawai-in-satker (Fase 1.11). Dipakai oleh form pemakaian
/// BMN: operator input NIP → sistem auto-lookup + tampilkan info pegawai
/// + pemakaian aktif + histori. Validator Satker & Approver Satker juga
/// melihat info yang sama (transparansi sejak hulu).
#[derive(Debug, Clone, Serialize)]
pub struct CekPegawaiResponse {
    pub pegawai: PegawaiInfo,
    /// Pemakaian BMN saat ini aktif utk pegawai ini (jika ada).
    pub pemakaian_aktif: Vec<PemakaianAktifEntry>,
    /// Histori pemakaian BMN pegawai (status: Expired/Revoked/Completed).
    pub histori_pemakaian: Vec<PemakaianHistoriEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PemakaianAktifEntry {
    pub permit_id: Uuid,
    pub nomor_izin: Option<String>,
    pub bmn_nup: String,
    pub bmn_nama_barang: String,
    pub tanggal_mulai: NaiveDate,
    pub tanggal_selesai: NaiveDate,
}

#[derive(Debug, Clone, Serialize)]
pub struct PemakaianHistoriEntry {
    pub permit_id: Uuid,
    pub nomor_izin: Option<String>,
    pub bmn_nup: String,
    pub bmn_nama_barang: String,
    pub status: String,
    pub tanggal_mulai: NaiveDate,
    pub tanggal_selesai: NaiveDate,
}

/// Status cek ketersediaan BMN per periode (Fase 1.11).
///
/// `Available`: BMN bebas utk periode yg diminta.
/// `Sequential`: Ada izin aktif tapi berakhir sebelum periode usulan
/// dimulai → diizinkan (operator dpt submit).
/// `Overlap`: Ada izin aktif yg overlap dgn periode usulan → tolak.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BmnCheckStatus {
    Available,
    PemakaianBerurutan {
        existing_holder: String,
        existing_sampai_tgl: NaiveDate,
    },
    Overlap {
        existing_holder: String,
        existing_sampai_tgl: NaiveDate,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct CekBmnResponse {
    pub bmn_nup: String,
    pub bmn_info: Option<BmnRefInfo>,
    #[serde(flatten)]
    pub status: BmnCheckStatus,
}

#[derive(Debug, Clone, Serialize)]
pub struct BmnRefInfo {
    pub nup: String,
    pub kode_barang: Option<String>,
    pub nama_barang: Option<String>,
    pub merk: Option<String>,
    pub tahun_perolehan: Option<String>,
    pub kondisi: Option<String>,
}

/// Permit history entry
#[derive(Debug, Clone, Serialize)]
pub struct PermitHistoryEntry {
    pub id: Uuid,
    pub nomor_izin: Option<String>,
    pub tanggal_mulai: NaiveDate,
    pub tanggal_selesai: NaiveDate,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

/// Usage statistics for a BMN
#[derive(Debug, Clone, Serialize)]
pub struct BmnUsageStats {
    pub bmn_nup: String,
    pub bmn_nama: String,
    pub total_permits: i64,
    pub active_permits: i64,
    pub total_days_used: i64,
    pub current_holder: Option<String>,
    pub permit_history: Vec<PermitHistoryEntry>,
}

/// Usage statistics for a pegawai
#[derive(Debug, Clone, Serialize)]
pub struct PegawaiUsageStats {
    pub pegawai_nip: String,
    pub pegawai_nama: String,
    pub total_permits: i64,
    pub active_permits: i64,
    pub permit_history: Vec<PermitHistoryEntry>,
}

// ============================================================================
// Monitoring Dashboard Models
// ============================================================================

/// Active usage monitoring dashboard data
/// Requirements: REQ-P011
#[derive(Debug, Clone, Serialize)]
pub struct ActiveUsageMonitoringDashboard {
    pub total_active_permits: i64,
    pub permits_by_jenis_bmn: Vec<PermitsByJenisBmn>,
    pub permits_by_satker: Vec<PermitsBySatker>,
    pub expiring_soon: Vec<ExpiringPermitInfo>,
    pub recent_activations: Vec<RecentActivationInfo>,
}

/// Permits grouped by BMN type
#[derive(Debug, Clone, Serialize)]
pub struct PermitsByJenisBmn {
    pub jenis_bmn: String,
    pub count: i64,
    pub percentage: f64,
}

/// Permits grouped by satker
#[derive(Debug, Clone, Serialize)]
pub struct PermitsBySatker {
    pub satker_id: Uuid,
    pub satker_nama: String,
    pub active_permits: i64,
}

/// Permit expiring soon information
#[derive(Debug, Clone, Serialize)]
pub struct ExpiringPermitInfo {
    pub id: Uuid,
    pub nomor_izin: Option<String>,
    pub bmn_nama: String,
    pub pegawai_nama: String,
    pub tanggal_selesai: NaiveDate,
    pub days_until_expiry: i64,
}

/// Recent activation information
#[derive(Debug, Clone, Serialize)]
pub struct RecentActivationInfo {
    pub id: Uuid,
    pub nomor_izin: Option<String>,
    pub bmn_nama: String,
    pub pegawai_nama: String,
    pub activated_at: DateTime<Utc>,
}

/// BMN utilization report
/// Requirements: REQ-P013
#[derive(Debug, Clone, Serialize)]
pub struct BmnUtilizationReport {
    pub total_bmn: i64,
    pub bmn_with_active_permits: i64,
    pub bmn_without_permits: i64,
    pub utilization_rate: f64,
    pub bmn_by_type: Vec<BmnUtilizationByType>,
    pub top_utilized_bmn: Vec<TopUtilizedBmn>,
    pub underutilized_bmn: Vec<UnderutilizedBmn>,
}

/// BMN utilization by type
#[derive(Debug, Clone, Serialize)]
pub struct BmnUtilizationByType {
    pub jenis_bmn: String,
    pub total_bmn: i64,
    pub utilized_bmn: i64,
    pub utilization_rate: f64,
}

/// Top utilized BMN
#[derive(Debug, Clone, Serialize)]
pub struct TopUtilizedBmn {
    pub bmn_nup: String,
    pub bmn_nama: String,
    pub jenis_bmn: String,
    pub total_permits: i64,
    pub total_days_used: i64,
    pub current_holder: Option<String>,
}

/// Underutilized BMN
#[derive(Debug, Clone, Serialize)]
pub struct UnderutilizedBmn {
    pub bmn_nup: String,
    pub bmn_nama: String,
    pub jenis_bmn: String,
    pub last_used_date: Option<NaiveDate>,
    pub days_since_last_use: Option<i64>,
}

/// Query parameters for monitoring dashboard
#[derive(Debug, Clone, Deserialize)]
pub struct MonitoringDashboardQuery {
    pub satker_id: Option<Uuid>,
    pub jenis_bmn: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}

/// Tiga kartu agregat headline dashboard monitoring Pemakaian BMN (Fase 2.6).
///
/// Stakeholder (Validator Wilayah & Pusat, read-only) eksplisit meminta tiga
/// kartu ini berdampingan: **sedang dipakai / tidak dipakai / akan expired**.
/// Sebelumnya nilai-nilai ini tersebar di dua endpoint berbeda; endpoint
/// `/monitoring/summary` menyatukannya jadi satu panggilan murah.
#[derive(Debug, Clone, Serialize)]
pub struct MonitoringSummaryCards {
    /// BMN dgn izin pemakaian berstatus ACTIVE (sesuai filter satker/jenis).
    pub sedang_dipakai: i64,
    /// Izin ACTIVE yg `tanggal_selesai` jatuh dalam 30 hari ke depan.
    pub akan_expired_30d: i64,
    /// BMN (kondisi BAIK di SIMAN) yg TIDAK sedang dipakai = total − terpakai.
    /// `None` bila SIMAN tidak tersedia, atau bila ada filter satker/jenis
    /// (data SIMAN tidak ter-scope per-satker di sini, agar tidak menyesatkan).
    pub tidak_dipakai: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_from_code() {
        assert_eq!(
            PemakaianBmnStatus::from_code(3000),
            Some(PemakaianBmnStatus::Draft)
        );
        assert_eq!(
            PemakaianBmnStatus::from_code(3004),
            Some(PemakaianBmnStatus::Active)
        );
        assert_eq!(PemakaianBmnStatus::from_code(9999), None);
    }

    #[test]
    fn test_status_transitions() {
        let draft = PemakaianBmnStatus::Draft;
        assert!(draft.can_transition_to(PemakaianBmnStatus::Submitted));
        assert!(!draft.can_transition_to(PemakaianBmnStatus::Active));

        let active = PemakaianBmnStatus::Active;
        assert!(active.can_transition_to(PemakaianBmnStatus::Expired));
        assert!(active.can_transition_to(PemakaianBmnStatus::Revoked));
        assert!(!active.can_transition_to(PemakaianBmnStatus::Draft));
    }

    /// V035 (Fase 1.5): alur internal-satker 3-step.
    /// Draft → Submitted (Operator) → SubmittedApproverSatker (Validator)
    /// → Approved (Approver Satker) → Active.
    /// RevisiOperator pintu balik dari Validator atau Approver.
    #[test]
    fn test_status_transitions_satker_3step() {
        use PemakaianBmnStatus::*;
        // Operator submit
        assert!(Draft.can_transition_to(Submitted));
        // Validator Satker forward → ApproverSatker
        assert!(Submitted.can_transition_to(SubmittedApproverSatker));
        // Validator Satker return → RevisiOperator
        assert!(Submitted.can_transition_to(RevisiOperator));
        // Approver Satker approve
        assert!(SubmittedApproverSatker.can_transition_to(Approved));
        // Approver Satker return ke Operator
        assert!(SubmittedApproverSatker.can_transition_to(RevisiOperator));
        // Operator re-submit
        assert!(RevisiOperator.can_transition_to(Submitted));
        // RevisiOperator dapat di-cancel
        assert!(RevisiOperator.can_transition_to(Cancelled));
        // Skip illegal: Draft langsung ke Approved tidak boleh
        assert!(!Draft.can_transition_to(Approved));
        assert!(!Draft.can_transition_to(SubmittedApproverSatker));
        // SubmittedApproverSatker tidak boleh langsung Active (harus via Approved → activate)
        assert!(!SubmittedApproverSatker.can_transition_to(Active));
        // RevisiOperator tidak boleh langsung lompat ke ApproverSatker
        assert!(!RevisiOperator.can_transition_to(SubmittedApproverSatker));
    }

    /// Helper utk test logic overlap di repository — direkstrak agar
    /// dapat di-unit-test tanpa DB. Match dgn implementasi di
    /// `check_bmn_availability_for_period` (repository.rs).
    fn classify(
        existing: &[(chrono::NaiveDate, chrono::NaiveDate, &str)],
        new_start: chrono::NaiveDate,
        new_end: chrono::NaiveDate,
    ) -> &'static str {
        let mut latest_end: Option<chrono::NaiveDate> = None;
        for (s, e, _) in existing {
            let overlap = !(*e < new_start || *s > new_end);
            if overlap {
                return "Overlap";
            }
            if latest_end.map(|prev| *e > prev).unwrap_or(true) {
                latest_end = Some(*e);
            }
        }
        if latest_end.is_some() {
            "PemakaianBerurutan"
        } else {
            "Available"
        }
    }

    #[test]
    fn bmn_check_status_available_when_no_existing() {
        let s = classify(
            &[],
            chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        );
        assert_eq!(s, "Available");
    }

    #[test]
    fn bmn_check_status_sequential_when_existing_ends_before_new_start() {
        // Existing 2026-01-01 .. 2026-06-02, new starts 2026-06-03 → sequential OK.
        let s = classify(
            &[(
                chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                chrono::NaiveDate::from_ymd_opt(2026, 6, 2).unwrap(),
                "pegawai_lain",
            )],
            chrono::NaiveDate::from_ymd_opt(2026, 6, 3).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        );
        assert_eq!(s, "PemakaianBerurutan");
    }

    #[test]
    fn bmn_check_status_overlap_when_periods_intersect() {
        // Existing 2026-01-01 .. 2026-06-30, new 2026-06-15 .. 2026-12-31 → overlap.
        let s = classify(
            &[(
                chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
                "pegawai_lain",
            )],
            chrono::NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        );
        assert_eq!(s, "Overlap");
    }

    #[test]
    fn bmn_check_status_overlap_when_new_inside_existing() {
        // Existing 2026-01-01 .. 2026-12-31, new 2026-05-01 .. 2026-06-30
        // (entirely inside) → overlap.
        let s = classify(
            &[(
                chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
                "pegawai_lain",
            )],
            chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
        );
        assert_eq!(s, "Overlap");
    }

    #[test]
    fn bmn_check_status_overlap_when_edge_equal() {
        // Existing ends 2026-06-02, new starts 2026-06-02 — same day = overlap
        // (sequential berarti new_start STRICTLY > existing_end).
        let s = classify(
            &[(
                chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                chrono::NaiveDate::from_ymd_opt(2026, 6, 2).unwrap(),
                "pegawai_lain",
            )],
            chrono::NaiveDate::from_ymd_opt(2026, 6, 2).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        );
        assert_eq!(s, "Overlap");
    }

    #[test]
    fn test_state_name_conversion() {
        assert_eq!(PemakaianBmnStatus::Draft.to_state_name(), "DRAFT");
        assert_eq!(PemakaianBmnStatus::Active.to_state_name(), "ACTIVE");

        assert_eq!(
            PemakaianBmnStatus::from_state_name("DRAFT"),
            Some(PemakaianBmnStatus::Draft)
        );
        assert_eq!(PemakaianBmnStatus::from_state_name("INVALID"), None);
    }

    #[test]
    fn test_jenis_bmn_required_fields() {
        let kendaraan = JenisBmn::KendaraanBermotor;
        assert!(kendaraan.required_fields().contains(&"no_polisi"));

        let rumah = JenisBmn::RumahNegara;
        assert!(rumah.required_fields().contains(&"alamat"));

        let laptop = JenisBmn::Laptop;
        assert!(laptop.required_fields().contains(&"serial_number"));
    }
}
