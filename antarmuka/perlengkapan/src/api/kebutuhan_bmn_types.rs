#[allow(unused_imports)]
use super::common::*;
use serde_json::Value;

use serde::{Deserialize, Serialize};

// ============================================================================
// KEBUTUHAN BMN MODELS
// ============================================================================

/// Workflow status codes for BMN needs requests
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum KebutuhanBmnStatus {
    Draft = 2000,
    InputBarang = 2001,
    SubmitSatker = 2002,
    RevisiSatker = 2003,
    AnalisisKelayakan = 2004,
    PenyusunanPrioritas = 2005,
    Approved = 2006,
    Rejected = 2007,
    Completed = 2008,
    Cancelled = 2009,
}

impl KebutuhanBmnStatus {
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

    pub fn badge_class(&self) -> &'static str {
        match self {
            Self::Draft => "bg-slate-500/15 text-slate-300 ring-1 ring-slate-500/25",
            Self::InputBarang => "bg-info-500/15 text-info-300 ring-1 ring-info-500/25",
            Self::SubmitSatker => "bg-gold-500/15 text-gold-300 ring-1 ring-gold-500/25",
            Self::RevisiSatker => "bg-warning-500/15 text-warning-300 ring-1 ring-warning-500/25",
            Self::AnalisisKelayakan => "bg-purple-500/15 text-purple-300 ring-1 ring-purple-500/25",
            Self::PenyusunanPrioritas => {
                "bg-indigo-500/15 text-indigo-300 ring-1 ring-indigo-500/25"
            }
            Self::Approved => "bg-success-500/15 text-success-300 ring-1 ring-success-500/25",
            Self::Rejected => "bg-danger-500/15 text-danger-300 ring-1 ring-danger-500/25",
            Self::Completed => "bg-success-500/15 text-success-300 ring-1 ring-success-500/25",
            Self::Cancelled => "bg-slate-500/15 text-slate-400 ring-1 ring-slate-500/25",
        }
    }
}

impl Default for KebutuhanBmnStatus {
    fn default() -> Self {
        Self::Draft
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PilihanSatker {
    Semua,
    Sebagian,
}

impl Default for PilihanSatker {
    fn default() -> Self {
        Self::Semua
    }
}

/// Main entity for BMN needs analysis request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PengajuanKebutuhanBmn {
    pub id: String,
    pub nama: String,
    pub deskripsi: Option<String>,
    pub tahun: i32,
    pub tgl_mulai: String,
    pub tgl_selesai: String,
    pub pilihan_satker: PilihanSatker,
    pub id_jenis_asset: Value,
    pub is_appv_daskrimti: bool,
    pub status_kode: i32,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub version: i32,
}

/// Asset type included in a BMN request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PengajuanKebutuhanBmnAsset {
    pub id: String,
    pub pengajuan_id: String,
    pub kode_barang: Option<String>,
    pub nm_barang: Option<String>,
    pub ms_jenis_asset_id: Option<i32>,
    pub keterangan: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Per-satker tracking for BMN request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PengajuanKebutuhanBmnSatker {
    pub id: String,
    pub pengajuan_id: String,
    pub ms_satker_id: String,
    pub ms_satker_pusat_id: Option<String>,
    pub nm_satker: Option<String>,
    pub status_kode: i32,
    pub prioritas: i32,
    // Operator satker fields
    pub catatan_satker: Option<String>,
    pub lampiran_surat_permohonan: Option<String>,
    pub lampiran_pendukung: Option<Value>,
    // Validator Wilayah fields
    pub catatan_validator_wilayah: Option<String>,
    pub validator_wilayah_id: Option<String>,
    pub tanggal_submit_wilayah: Option<String>,
    // Validator Pusat fields
    pub catatan_validator_pusat: Option<String>,
    pub validator_pusat_id: Option<String>,
    pub tanggal_submit_pusat: Option<String>,
    // Analisis data (SIMAN + MySIMKARI)
    pub data_eksisting_siman: Option<Value>,
    pub data_pegawai_mysimkari: Option<Value>,
    pub rekap_eselon: Option<Value>,
    pub rekap_non_eselon: Option<Value>,
    pub hasil_analisis: Option<Value>,
    // Keputusan validator pusat
    pub is_approved: Option<bool>,
    pub alasan_keputusan: Option<String>,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Individual goods requested by a satker
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PengajuanKebutuhanBmnBarang {
    pub id: String,
    pub pengajuan_satker_id: String,
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
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Workflow history audit trail
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PengajuanKebutuhanBmnAktivitas {
    pub id: String,
    pub pengajuan_satker_id: String,
    pub from_status_kode: Option<i32>,
    pub to_status_kode: i32,
    pub user_id: Option<String>,
    pub nip: Option<String>,
    pub nama: Option<String>,
    pub pangkat: Option<String>,
    pub jabatan: Option<String>,
    pub role: Option<String>,
    pub aksi: String,
    pub komentar: Option<String>,
    pub created_at: String,
}

/// Summary data for dashboard and lists
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KebutuhanBmnSummary {
    pub id: String,
    pub nama: String,
    pub tahun: i32,
    pub status_kode: i32,
    pub status_nama: String,
    pub total_satker: i64,
    pub total_barang: i64,
    pub total_jumlah_diminta: i64,
    pub total_jumlah_disetujui: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// Workflow transition info
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowTransitionInfo {
    pub status_kode: i32,
    pub status_nama: String,
    pub requires_comment: bool,
}

/// Response with pengajuan and related data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PengajuanDetailResponse {
    #[serde(flatten)]
    pub pengajuan: PengajuanKebutuhanBmn,
    pub assets: Vec<PengajuanKebutuhanBmnAsset>,
    pub satkers: Vec<PengajuanKebutuhanBmnSatker>,
    pub allowed_transitions: Vec<WorkflowTransitionInfo>,
}

/// Response for satker with its goods
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SatkerWithBarangResponse {
    #[serde(flatten)]
    pub satker: PengajuanKebutuhanBmnSatker,
    pub barang_list: Vec<PengajuanKebutuhanBmnBarang>,
    pub total_barang: i64,
    pub total_jumlah: i64,
}

/// Response for feasibility analysis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalisisKelayakanResponse {
    pub satker: PengajuanKebutuhanBmnSatker,
    pub barang_list: Vec<BarangWithExistingInventory>,
    pub data_pegawai: Option<DataPegawaiRekap>,
    pub integrasi_sync: Option<IntegrasiSyncMetadata>,
    pub summary: AnalisisSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IntegrasiSyncMetadata {
    pub mysimkari: IntegrasiSyncStatus,
    pub siman: IntegrasiSyncStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IntegrasiSyncStatus {
    pub source: String,
    pub state: String,
    pub last_sync_at: Option<String>,
    pub next_sync_at: Option<String>,
    pub records_synced: i64,
    pub error_message: Option<String>,
}

/// Rekap data pegawai from MySIMKARI
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DataPegawaiRekap {
    pub total_pegawai: i64,
    pub rekap_eselon: Vec<RekapEselonItem>,
    pub rekap_non_eselon: Vec<RekapNonEselonItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RekapEselonItem {
    pub tingkat_eselon: String,
    pub jumlah: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RekapNonEselonItem {
    pub golongan: String,
    pub pangkat: String,
    pub is_jaksa: bool,
    pub jumlah: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BarangWithExistingInventory {
    #[serde(flatten)]
    pub barang: PengajuanKebutuhanBmnBarang,
    pub existing_assets: Vec<ExistingAssetInfo>,
    pub gap: i32,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExistingAssetInfo {
    pub no_aset: String,
    pub nama_aset: String,
    pub kondisi: String,
    pub lokasi: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalisisSummary {
    pub total_diminta: i64,
    pub total_existing: i64,
    pub total_gap: i64,
    pub kelayakan_persen: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatsByTahun {
    pub tahun: i32,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatsByStatus {
    pub status_kode: i32,
    pub status_nama: String,
    pub total: i64,
}

// ============================================================================
// SIMAN Integration Types
// ============================================================================

/// Asset data from SIMAN (Sistem Informasi Manajemen Aset Negara)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SimanAsset {
    pub no_aset: String,
    pub nama_aset: String,
    pub nup: Option<String>,
    pub kondisi: String,
    pub tahun_perolehan: Option<i32>,
    pub nilai_perolehan: Option<f64>,
    pub nilai_buku: Option<f64>,
    pub lokasi: Option<String>,
    pub kategori: String,
    pub satker_id: String,
    pub metadata: Option<serde_json::Value>,
}

/// Summary of existing assets for a satker from SIMAN
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SatkerAssetSummary {
    pub satker_id: String,
    pub satker_name: Option<String>,
    pub total_assets: i64,
    pub total_value: f64,
    pub by_category: Vec<CategoryAssetCount>,
    pub by_condition: Vec<ConditionAssetCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CategoryAssetCount {
    pub category: String,
    pub count: i64,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConditionAssetCount {
    pub condition: String,
    pub count: i64,
}

// Request DTOs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateKebutuhanBmnRequest {
    pub nama: String,
    pub deskripsi: Option<String>,
    pub tahun: i32,
    pub tgl_mulai: String,
    pub tgl_selesai: String,
    pub pilihan_satker: Option<String>,
    #[serde(default)]
    pub satker_ids: Vec<String>,
    #[serde(default)]
    pub asset_types: Vec<CreateAssetTypeRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAssetTypeRequest {
    pub kode_barang: Option<String>,
    pub nm_barang: Option<String>,
    pub ms_jenis_asset_id: Option<i32>,
    pub keterangan: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateKebutuhanBmnRequest {
    pub nama: Option<String>,
    pub deskripsi: Option<String>,
    pub tgl_mulai: Option<String>,
    pub tgl_selesai: Option<String>,
    pub pilihan_satker: Option<String>,
    pub version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateKebutuhanBmnBarangRequest {
    pub nama: String,
    pub kode_barang: Option<String>,
    pub jumlah: i32,
    pub satuan: String,
    pub alasan: Option<String>,
    pub keterangan: Option<String>,
    #[serde(default)]
    pub file_pendukung: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateBarangApprovalRequest {
    pub jml_setuju: i32,
    pub keterangan: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetPrioritasRequest {
    pub items: Vec<PrioritasItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrioritasItem {
    pub barang_id: String,
    pub prioritas: i32,
    pub skor: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTransitionRequest {
    pub target_status: i32,
    pub komentar: Option<String>,
}

/// Request for operator satker submitting to validator wilayah
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitKebutuhanSatkerRequest {
    pub catatan_satker: Option<String>,
    pub lampiran_surat_permohonan: String,
    #[serde(default)]
    pub lampiran_pendukung: Vec<LampiranItem>,
}

/// Lampiran item for file attachments
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LampiranItem {
    pub nama: String,
    pub url: String,
    pub tipe: Option<String>,
}

/// Request for validator wilayah action (forward or return)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KebutuhanValidatorWilayahActionRequest {
    /// "forward" or "return"
    pub aksi: String,
    pub catatan: Option<String>,
}

/// Request for validator pusat decision (approve or reject only)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorPusatKeputusanRequest {
    pub is_approved: bool,
    pub alasan: Option<String>,
    pub override_darurat: Option<bool>,
    pub override_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KebutuhanBmnQuery {
    pub tahun: Option<i32>,
    pub status_kode: Option<i32>,
    pub satker_id: Option<String>,
    pub search: Option<String>,
}
