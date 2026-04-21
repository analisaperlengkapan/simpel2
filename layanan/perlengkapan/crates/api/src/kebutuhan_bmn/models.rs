//! # Kebutuhan BMN Data Models
//!
//! Data models for BMN needs analysis system.
//! Includes domain entities, DTOs, and workflow enums.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio_postgres::Row;
use uuid::Uuid;
use validator::Validate;

// ============================================================================
// Workflow Status Enum
// ============================================================================

/// Workflow status codes for BMN needs requests
///
/// Flow:
///   Draft(2000) → InputBarang(2001) → SubmitWilayah(2002) → SubmitPusat(2004) → AnalisisKelayakan(2005) → Approved(2006)/Rejected(2007) → Completed(2008)
///   Validator Wilayah can return to RevisiSatker(2003)
///   Validator Pusat approves or rejects (no revision back)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum KebutuhanBmnStatus {
    /// New request in draft state (Validator Pusat creates period)
    Draft = 2000,
    /// Operator Satker inputting goods + attachments
    InputBarang = 2001,
    /// Operator Satker submitted to Validator Wilayah
    SubmitWilayah = 2002,
    /// Returned to Operator Satker for revision (by Validator Wilayah)
    RevisiSatker = 2003,
    /// Validator Wilayah forwarded to Validator Pusat
    SubmitPusat = 2004,
    /// Validator Pusat analyzing feasibility (SIMAN + MySIMKARI data)
    AnalisisKelayakan = 2005,
    /// Request approved by Validator Pusat
    Approved = 2006,
    /// Request rejected by Validator Pusat
    Rejected = 2007,
    /// Process completed
    Completed = 2008,
    /// Request cancelled
    Cancelled = 2009,
    /// Returned to Validator Wilayah by Validator Pusat for corrections
    RevisiWilayah = 2010,
}

impl KebutuhanBmnStatus {
    /// Convert from database integer code
    pub fn from_code(code: i32) -> Option<Self> {
        match code {
            2000 => Some(Self::Draft),
            2001 => Some(Self::InputBarang),
            2002 => Some(Self::SubmitWilayah),
            2003 => Some(Self::RevisiSatker),
            2004 => Some(Self::SubmitPusat),
            2005 => Some(Self::AnalisisKelayakan),
            2006 => Some(Self::Approved),
            2007 => Some(Self::Rejected),
            2008 => Some(Self::Completed),
            2009 => Some(Self::Cancelled),
            2010 => Some(Self::RevisiWilayah),
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
            Self::InputBarang => "Input Barang",
            Self::SubmitWilayah => "Diajukan ke Validator Wilayah",
            Self::RevisiSatker => "Revisi Satker",
            Self::SubmitPusat => "Diajukan ke Validator Pusat",
            Self::AnalisisKelayakan => "Analisis Kelayakan",
            Self::Approved => "Disetujui",
            Self::Rejected => "Ditolak",
            Self::Completed => "Selesai",
            Self::Cancelled => "Dibatalkan",
            Self::RevisiWilayah => "Revisi Wilayah",
        }
    }

    /// Check if transition to target status is allowed
    ///
    /// Flow: Draft → InputBarang → SubmitWilayah → SubmitPusat → AnalisisKelayakan → Approved/Rejected
    /// Validator Wilayah: SubmitWilayah → SubmitPusat (forward) or RevisiSatker (return)
    /// Validator Pusat: AnalisisKelayakan → Approved or Rejected (NO revision back)
    pub fn can_transition_to(&self, target: Self) -> bool {
        use KebutuhanBmnStatus::*;
        match self {
            Draft => matches!(target, InputBarang | Cancelled),
            InputBarang => matches!(target, SubmitWilayah | Cancelled),
            SubmitWilayah => matches!(target, SubmitPusat | RevisiSatker),
            RevisiSatker => matches!(target, SubmitWilayah | Cancelled),
            SubmitPusat => matches!(target, AnalisisKelayakan),
            AnalisisKelayakan => matches!(target, Approved | Rejected | RevisiWilayah),
            RevisiWilayah => matches!(target, SubmitPusat),
            Approved => matches!(target, Completed),
            Rejected | Completed | Cancelled => false,
        }
    }

    /// Get allowed next statuses for workflow UI
    pub fn allowed_transitions(&self) -> Vec<Self> {
        use KebutuhanBmnStatus::*;
        match self {
            Draft => vec![InputBarang, Cancelled],
            InputBarang => vec![SubmitWilayah, Cancelled],
            SubmitWilayah => vec![SubmitPusat, RevisiSatker],
            RevisiSatker => vec![SubmitWilayah, Cancelled],
            SubmitPusat => vec![AnalisisKelayakan],
            AnalisisKelayakan => vec![Approved, Rejected, RevisiWilayah],
            RevisiWilayah => vec![SubmitPusat],
            Approved => vec![Completed],
            Rejected | Completed | Cancelled => vec![],
        }
    }

    /// Convert status to workflow engine state name
    pub fn to_state_name(&self) -> &'static str {
        match self {
            Self::Draft => "DRAFT",
            Self::InputBarang => "INPUT_BARANG",
            Self::SubmitWilayah => "SUBMIT_WILAYAH",
            Self::RevisiSatker => "REVISI_SATKER",
            Self::SubmitPusat => "SUBMIT_PUSAT",
            Self::AnalisisKelayakan => "ANALISIS_KELAYAKAN",
            Self::Approved => "APPROVED",
            Self::Rejected => "REJECTED",
            Self::Completed => "COMPLETED",
            Self::Cancelled => "CANCELLED",
            Self::RevisiWilayah => "REVISI_WILAYAH",
        }
    }

    /// Convert workflow engine state name to status
    pub fn from_state_name(name: &str) -> Option<Self> {
        match name {
            "DRAFT" => Some(Self::Draft),
            "INPUT_BARANG" => Some(Self::InputBarang),
            "SUBMIT_WILAYAH" => Some(Self::SubmitWilayah),
            "SUBMIT_SATKER" => Some(Self::SubmitWilayah), // backward compat
            "REVISI_SATKER" => Some(Self::RevisiSatker),
            "SUBMIT_PUSAT" => Some(Self::SubmitPusat),
            "ANALISIS_KELAYAKAN" => Some(Self::AnalisisKelayakan),
            "PENYUSUNAN_PRIORITAS" => Some(Self::AnalisisKelayakan), // backward compat
            "APPROVED" => Some(Self::Approved),
            "REJECTED" => Some(Self::Rejected),
            "COMPLETED" => Some(Self::Completed),
            "CANCELLED" => Some(Self::Cancelled),
            "REVISI_WILAYAH" => Some(Self::RevisiWilayah),
            _ => None,
        }
    }
}

impl Default for KebutuhanBmnStatus {
    fn default() -> Self {
        Self::Draft
    }
}

// ============================================================================
// Satker Selection Type
// ============================================================================

/// Options for satker selection in a request
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PilihanSatker {
    Semua,
    Sebagian,
}

impl PilihanSatker {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "sebagian" => Self::Sebagian,
            _ => Self::Semua,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Semua => "semua",
            Self::Sebagian => "sebagian",
        }
    }
}

impl Default for PilihanSatker {
    fn default() -> Self {
        Self::Semua
    }
}

// ============================================================================
// Main Entity: Pengajuan Kebutuhan BMN
// ============================================================================

/// Main entity for BMN needs analysis request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PengajuanKebutuhanBmn {
    pub id: Uuid,
    pub nama: String,
    pub deskripsi: Option<String>,
    pub tahun: i32,
    pub tgl_mulai: NaiveDate,
    pub tgl_selesai: NaiveDate,
    pub pilihan_satker: PilihanSatker,
    pub id_jenis_asset: Value,
    pub is_appv_daskrimti: bool,
    pub status_kode: i32,
    pub status: KebutuhanBmnStatus,
    // Report generation
    pub laporan_url: Option<String>,
    pub laporan_format: Option<String>,
    pub laporan_generated_at: Option<DateTime<Utc>>,
    // Audit
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i32,
}

impl PengajuanKebutuhanBmn {
    /// Create from database row
    pub fn from_row(row: &Row) -> Self {
        let status_kode: i32 = row.get("status_kode");
        Self {
            id: row.get("id"),
            nama: row.get("nama"),
            deskripsi: row.get("deskripsi"),
            tahun: row.get("tahun"),
            tgl_mulai: row.get("tgl_mulai"),
            tgl_selesai: row.get("tgl_selesai"),
            pilihan_satker: PilihanSatker::from_str(
                row.get::<_, String>("pilihan_satker").as_str(),
            ),
            id_jenis_asset: row.get("id_jenis_asset"),
            is_appv_daskrimti: row.get("is_appv_daskrimti"),
            status_kode,
            status: KebutuhanBmnStatus::from_code(status_kode).unwrap_or_default(),
            laporan_url: row.try_get("laporan_url").ok().flatten(),
            laporan_format: row.try_get("laporan_format").ok().flatten(),
            laporan_generated_at: row.try_get("laporan_generated_at").ok().flatten(),
            created_by: row.get("created_by"),
            updated_by: row.get("updated_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            version: row.get("version"),
        }
    }
}

// ============================================================================
// Entity: Pengajuan Kebutuhan BMN Asset
// ============================================================================

/// Asset type included in a BMN request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PengajuanKebutuhanBmnAsset {
    pub id: Uuid,
    pub pengajuan_id: Uuid,
    pub kode_barang: Option<String>,
    pub nm_barang: Option<String>,
    pub ms_jenis_asset_id: Option<i32>,
    pub keterangan: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PengajuanKebutuhanBmnAsset {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            pengajuan_id: row.get("pengajuan_id"),
            kode_barang: row.get("kode_barang"),
            nm_barang: row.get("nm_barang"),
            ms_jenis_asset_id: row.get("ms_jenis_asset_id"),
            keterangan: row.get("keterangan"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

// ============================================================================
// Entity: Pengajuan Kebutuhan BMN Satker
// ============================================================================

/// Per-satker tracking for BMN request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PengajuanKebutuhanBmnSatker {
    pub id: Uuid,
    pub pengajuan_id: Uuid,
    pub ms_satker_id: String,
    pub ms_satker_pusat_id: Option<String>,
    pub nm_satker: Option<String>,
    pub status_kode: i32,
    pub status: KebutuhanBmnStatus,
    pub prioritas: i32,
    // Lampiran & catatan operator satker
    pub catatan_satker: Option<String>,
    pub lampiran_surat_permohonan: Option<String>,
    pub lampiran_pendukung: Value,
    // Validator wilayah
    pub catatan_validator_wilayah: Option<String>,
    pub validator_wilayah_id: Option<Uuid>,
    pub tanggal_submit_wilayah: Option<DateTime<Utc>>,
    // Validator pusat
    pub catatan_validator_pusat: Option<String>,
    pub validator_pusat_id: Option<Uuid>,
    pub tanggal_submit_pusat: Option<DateTime<Utc>>,
    // Analysis data (from SIMAN & MySIMKARI)
    pub data_eksisting_siman: Value,
    pub data_pegawai_mysimkari: Value,
    pub rekap_eselon: Value,
    pub rekap_non_eselon: Value,
    pub hasil_analisis: Value,
    pub is_approved: Option<bool>,
    pub alasan_keputusan: Option<String>,
    // Audit
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PengajuanKebutuhanBmnSatker {
    pub fn from_row(row: &Row) -> Self {
        let status_kode: i32 = row.get("status_kode");
        Self {
            id: row.get("id"),
            pengajuan_id: row.get("pengajuan_id"),
            ms_satker_id: row.get("ms_satker_id"),
            ms_satker_pusat_id: row.get("ms_satker_pusat_id"),
            nm_satker: row.get("nm_satker"),
            status_kode,
            status: KebutuhanBmnStatus::from_code(status_kode).unwrap_or_default(),
            prioritas: row.get("prioritas"),
            catatan_satker: row.try_get("catatan_satker").ok().flatten(),
            lampiran_surat_permohonan: row.try_get("lampiran_surat_permohonan").ok().flatten(),
            lampiran_pendukung: row
                .try_get("lampiran_pendukung")
                .unwrap_or(serde_json::json!([])),
            catatan_validator_wilayah: row.try_get("catatan_validator_wilayah").ok().flatten(),
            validator_wilayah_id: row.try_get("validator_wilayah_id").ok().flatten(),
            tanggal_submit_wilayah: row.try_get("tanggal_submit_wilayah").ok().flatten(),
            catatan_validator_pusat: row.try_get("catatan_validator_pusat").ok().flatten(),
            validator_pusat_id: row.try_get("validator_pusat_id").ok().flatten(),
            tanggal_submit_pusat: row.try_get("tanggal_submit_pusat").ok().flatten(),
            data_eksisting_siman: row
                .try_get("data_eksisting_siman")
                .unwrap_or(serde_json::json!({})),
            data_pegawai_mysimkari: row
                .try_get("data_pegawai_mysimkari")
                .unwrap_or(serde_json::json!({})),
            rekap_eselon: row.try_get("rekap_eselon").unwrap_or(serde_json::json!({})),
            rekap_non_eselon: row
                .try_get("rekap_non_eselon")
                .unwrap_or(serde_json::json!({})),
            hasil_analisis: row
                .try_get("hasil_analisis")
                .unwrap_or(serde_json::json!({})),
            is_approved: row.try_get("is_approved").ok().flatten(),
            alasan_keputusan: row.try_get("alasan_keputusan").ok().flatten(),
            created_by: row.get("created_by"),
            updated_by: row.get("updated_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

// ============================================================================
// Entity: Pengajuan Kebutuhan BMN Satker Barang
// ============================================================================

/// Individual goods requested by a satker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PengajuanKebutuhanBmnBarang {
    pub id: Uuid,
    pub pengajuan_satker_id: Uuid,
    pub nama: String,
    pub kode_barang: Option<String>,
    pub jumlah: i32,
    pub satuan: String,
    pub jml_setuju: i32,
    pub alasan: Option<String>,
    pub keterangan: Option<String>,
    pub prioritas: i32,
    pub skor: f64,
    pub file_pendukung: Value,
    pub existing_count: i32,
    pub existing_condition: Option<String>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PengajuanKebutuhanBmnBarang {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            pengajuan_satker_id: row.get("pengajuan_satker_id"),
            nama: row.get("nama"),
            kode_barang: row.get("kode_barang"),
            jumlah: row.get("jumlah"),
            satuan: row.get("satuan"),
            jml_setuju: row.get("jml_setuju"),
            alasan: row.get("alasan"),
            keterangan: row.get("keterangan"),
            prioritas: row.get("prioritas"),
            skor: row.try_get::<_, f64>("skor").unwrap_or(0.0),
            file_pendukung: row.get("file_pendukung"),
            existing_count: row.get("existing_count"),
            existing_condition: row.get("existing_condition"),
            created_by: row.get("created_by"),
            updated_by: row.get("updated_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

// ============================================================================
// Entity: Pengajuan Kebutuhan BMN Satker Aktivitas
// ============================================================================

/// Workflow history audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PengajuanKebutuhanBmnAktivitas {
    pub id: Uuid,
    pub pengajuan_satker_id: Uuid,
    pub from_status_kode: Option<i32>,
    pub to_status_kode: i32,
    pub user_id: Option<Uuid>,
    pub nip: Option<String>,
    pub nama: Option<String>,
    pub pangkat: Option<String>,
    pub jabatan: Option<String>,
    pub role: Option<String>,
    pub aksi: String,
    pub komentar: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl PengajuanKebutuhanBmnAktivitas {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            pengajuan_satker_id: row.get("pengajuan_satker_id"),
            from_status_kode: row.get("from_status_kode"),
            to_status_kode: row.get("to_status_kode"),
            user_id: row.get("user_id"),
            nip: row.get("nip"),
            nama: row.get("nama"),
            pangkat: row.get("pangkat"),
            jabatan: row.get("jabatan"),
            role: row.get("role"),
            aksi: row.get("aksi"),
            komentar: row.get("komentar"),
            created_at: row.get("created_at"),
        }
    }
}

// ============================================================================
// Summary View Model
// ============================================================================

/// Summary data for dashboard and lists
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KebutuhanBmnSummary {
    pub id: Uuid,
    pub nama: String,
    pub tahun: i32,
    pub status_kode: i32,
    pub status_nama: String,
    pub total_satker: i64,
    pub total_barang: i64,
    pub total_jumlah_diminta: i64,
    pub total_jumlah_disetujui: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl KebutuhanBmnSummary {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            nama: row.get("nama"),
            tahun: row.get("tahun"),
            status_kode: row.get("status_kode"),
            status_nama: row
                .get::<_, Option<String>>("status_nama")
                .unwrap_or_default(),
            total_satker: row.get::<_, i64>("total_satker"),
            total_barang: row.get::<_, i64>("total_barang"),
            total_jumlah_diminta: row.get::<_, i64>("total_jumlah_diminta"),
            total_jumlah_disetujui: row.get::<_, i64>("total_jumlah_disetujui"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

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

    #[serde(default)]
    pub satker_ids: Vec<String>,

    #[serde(default)]
    pub asset_types: Vec<CreateAssetTypeRequest>,
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

/// Response with analysis data for validator pusat
#[derive(Debug, Clone, Serialize)]
pub struct AnalisisDataResponse {
    pub satker: PengajuanKebutuhanBmnSatker,
    pub barang_list: Vec<PengajuanKebutuhanBmnBarang>,
    pub data_eksisting_siman: Value,
    pub data_pegawai: DataPegawaiRekap,
    pub summary: AnalisisSummary,
}

/// Rekap data pegawai from MySIMKARI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPegawaiRekap {
    pub total_pegawai: i64,
    pub rekap_eselon: Vec<RekapEselonItem>,
    pub rekap_non_eselon: Vec<RekapNonEselonItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RekapEselonItem {
    pub tingkat_eselon: String,
    pub jumlah: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RekapNonEselonItem {
    pub golongan: String,
    pub pangkat: String,
    pub is_jaksa: bool,
    pub jumlah: i64,
}

// ============================================================================
// Response DTOs
// ============================================================================

/// Response with pengajuan and related data
#[derive(Debug, Clone, Serialize)]
pub struct PengajuanDetailResponse {
    #[serde(flatten)]
    pub pengajuan: PengajuanKebutuhanBmn,
    pub assets: Vec<PengajuanKebutuhanBmnAsset>,
    pub satkers: Vec<PengajuanKebutuhanBmnSatker>,
    pub allowed_transitions: Vec<WorkflowTransitionInfo>,
}

/// Information about allowed workflow transitions
#[derive(Debug, Clone, Serialize)]
pub struct WorkflowTransitionInfo {
    pub status_kode: i32,
    pub status_nama: String,
    pub requires_comment: bool,
}

/// Response for satker with its goods
#[derive(Debug, Clone, Serialize)]
pub struct SatkerWithBarangResponse {
    #[serde(flatten)]
    pub satker: PengajuanKebutuhanBmnSatker,
    pub barang_list: Vec<PengajuanKebutuhanBmnBarang>,
    pub total_barang: i64,
    pub total_jumlah: i64,
}

/// Response for feasibility analysis (enhanced with pegawai data)
#[derive(Debug, Clone, Serialize)]
pub struct AnalisisKelayakanResponse {
    pub satker: PengajuanKebutuhanBmnSatker,
    pub barang_list: Vec<BarangWithExistingInventory>,
    pub data_pegawai: Option<DataPegawaiRekap>,
    pub integrasi_sync: Option<IntegrasiSyncMetadata>,
    pub summary: AnalisisSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrasiSyncMetadata {
    pub mysimkari: IntegrasiSyncStatus,
    pub siman: IntegrasiSyncStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrasiSyncStatus {
    pub source: String,
    pub state: String,
    pub last_sync_at: Option<String>,
    pub next_sync_at: Option<String>,
    pub records_synced: i64,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BarangWithExistingInventory {
    #[serde(flatten)]
    pub barang: PengajuanKebutuhanBmnBarang,
    pub existing_assets: Vec<ExistingAssetInfo>,
    pub gap: i32, // jumlah - existing_count
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExistingAssetInfo {
    pub no_aset: String,
    pub nama_aset: String,
    pub kondisi: String,
    pub lokasi: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnalisisSummary {
    pub total_diminta: i64,
    pub total_existing: i64,
    pub total_gap: i64,
    pub kelayakan_persen: f64,
}

// ============================================================================
// Dashboard Statistics
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KebutuhanBmnDashboardStats {
    pub total_pengajuan: i64,
    pub pengajuan_draft: i64,
    pub pengajuan_in_progress: i64,
    pub pengajuan_completed: i64,
    pub total_satker_terlibat: i64,
    pub total_barang_diminta: i64,
    pub total_barang_disetujui: i64,
    pub by_tahun: Vec<StatsByTahun>,
    pub by_status: Vec<StatsByStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsByTahun {
    pub tahun: i32,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsByStatus {
    pub status_kode: i32,
    pub status_nama: String,
    pub total: i64,
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

/// Result of a single item in batch operation
#[derive(Debug, Clone, Serialize)]
pub struct BatchOperationItemResult {
    pub kebutuhan_id: Uuid,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Response for batch operations
#[derive(Debug, Clone, Serialize)]
pub struct BatchOperationResponse {
    pub batch_id: Uuid,
    pub total_items: usize,
    pub successful_items: usize,
    pub failed_items: usize,
    pub results: Vec<BatchOperationItemResult>,
    pub operation_type: String,
    pub executed_at: DateTime<Utc>,
    pub executed_by: Option<Uuid>,
}

// ============================================================================
// Unit Tests for Models
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_from_code() {
        assert_eq!(
            KebutuhanBmnStatus::from_code(2000),
            Some(KebutuhanBmnStatus::Draft)
        );
        assert_eq!(
            KebutuhanBmnStatus::from_code(2008),
            Some(KebutuhanBmnStatus::Completed)
        );
        assert_eq!(KebutuhanBmnStatus::from_code(9999), None);
    }

    #[test]
    fn test_status_to_code() {
        assert_eq!(KebutuhanBmnStatus::Draft.to_code(), 2000);
        assert_eq!(KebutuhanBmnStatus::Completed.to_code(), 2008);
    }

    #[test]
    fn test_status_transitions() {
        let draft = KebutuhanBmnStatus::Draft;
        assert!(draft.can_transition_to(KebutuhanBmnStatus::InputBarang));
        assert!(draft.can_transition_to(KebutuhanBmnStatus::Cancelled));
        assert!(!draft.can_transition_to(KebutuhanBmnStatus::Approved));

        // Operator satker submits to wilayah, not directly to pusat
        let input = KebutuhanBmnStatus::InputBarang;
        assert!(input.can_transition_to(KebutuhanBmnStatus::SubmitWilayah));
        assert!(!input.can_transition_to(KebutuhanBmnStatus::SubmitPusat));

        // Validator wilayah can forward or return
        let submit_wil = KebutuhanBmnStatus::SubmitWilayah;
        assert!(submit_wil.can_transition_to(KebutuhanBmnStatus::SubmitPusat));
        assert!(submit_wil.can_transition_to(KebutuhanBmnStatus::RevisiSatker));

        // Validator pusat approves or rejects only
        let analisis = KebutuhanBmnStatus::AnalisisKelayakan;
        assert!(analisis.can_transition_to(KebutuhanBmnStatus::Approved));
        assert!(analisis.can_transition_to(KebutuhanBmnStatus::Rejected));
        assert!(!analisis.can_transition_to(KebutuhanBmnStatus::RevisiSatker));

        let completed = KebutuhanBmnStatus::Completed;
        assert!(!completed.can_transition_to(KebutuhanBmnStatus::Draft));
        assert!(completed.allowed_transitions().is_empty());
    }

    #[test]
    fn test_pilihan_satker() {
        assert_eq!(PilihanSatker::from_str("semua"), PilihanSatker::Semua);
        assert_eq!(PilihanSatker::from_str("sebagian"), PilihanSatker::Sebagian);
        assert_eq!(PilihanSatker::from_str("unknown"), PilihanSatker::Semua);
        assert_eq!(PilihanSatker::Sebagian.as_str(), "sebagian");
    }

    #[test]
    fn test_status_labels() {
        assert_eq!(KebutuhanBmnStatus::Draft.label(), "Draft");
        assert_eq!(
            KebutuhanBmnStatus::AnalisisKelayakan.label(),
            "Analisis Kelayakan"
        );
    }
}
