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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum KebutuhanBmnStatus {
    /// New request in draft state
    Draft = 2000,
    /// Pelaksana Satker inputting goods
    InputBarang = 2001,
    /// Satker submitted to Validator
    SubmitSatker = 2002,
    /// Returned to Satker for revision
    RevisiSatker = 2003,
    /// Validator Pusat analyzing feasibility
    AnalisisKelayakan = 2004,
    /// Validator Pusat setting priorities
    PenyusunanPrioritas = 2005,
    /// Request approved
    Approved = 2006,
    /// Request rejected
    Rejected = 2007,
    /// Process completed
    Completed = 2008,
    /// Request cancelled
    Cancelled = 2009,
}

impl KebutuhanBmnStatus {
    /// Convert from database integer code
    pub fn from_code(code: i32) -> Option<Self> {
        match code {
            2000 => Some(Self::Draft),
            2001 => Some(Self::InputBarang),
            2002 => Some(Self::SubmitSatker),
            2003 => Some(Self::RevisiSatker),
            2004 => Some(Self::AnalisisKelayakan),
            2005 => Some(Self::PenyusunanPrioritas),
            2006 => Some(Self::Approved),
            2007 => Some(Self::Rejected),
            2008 => Some(Self::Completed),
            2009 => Some(Self::Cancelled),
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
            Self::SubmitSatker => "Diajukan ke Validator",
            Self::RevisiSatker => "Revisi Satker",
            Self::AnalisisKelayakan => "Analisis Kelayakan",
            Self::PenyusunanPrioritas => "Penyusunan Prioritas",
            Self::Approved => "Disetujui",
            Self::Rejected => "Ditolak",
            Self::Completed => "Selesai",
            Self::Cancelled => "Dibatalkan",
        }
    }

    /// Check if transition to target status is allowed
    pub fn can_transition_to(&self, target: Self) -> bool {
        use KebutuhanBmnStatus::*;
        match self {
            Draft => matches!(target, InputBarang | Cancelled),
            InputBarang => matches!(target, SubmitSatker | Cancelled),
            SubmitSatker => matches!(target, AnalisisKelayakan | RevisiSatker | Rejected),
            RevisiSatker => matches!(target, SubmitSatker | Cancelled),
            AnalisisKelayakan => matches!(target, PenyusunanPrioritas | RevisiSatker | Rejected),
            PenyusunanPrioritas => matches!(target, Approved | Rejected),
            Approved => matches!(target, Completed),
            Rejected | Completed | Cancelled => false,
        }
    }

    /// Get allowed next statuses for workflow UI
    pub fn allowed_transitions(&self) -> Vec<Self> {
        use KebutuhanBmnStatus::*;
        match self {
            Draft => vec![InputBarang, Cancelled],
            InputBarang => vec![SubmitSatker, Cancelled],
            SubmitSatker => vec![AnalisisKelayakan, RevisiSatker, Rejected],
            RevisiSatker => vec![SubmitSatker, Cancelled],
            AnalisisKelayakan => vec![PenyusunanPrioritas, RevisiSatker, Rejected],
            PenyusunanPrioritas => vec![Approved, Rejected],
            Approved => vec![Completed],
            Rejected | Completed | Cancelled => vec![],
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

/// Response for feasibility analysis
#[derive(Debug, Clone, Serialize)]
pub struct AnalisisKelayakanResponse {
    pub satker: PengajuanKebutuhanBmnSatker,
    pub barang_list: Vec<BarangWithExistingInventory>,
    pub summary: AnalisisSummary,
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
