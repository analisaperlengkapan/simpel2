use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::*;

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
    /// V029 (#24): `true` jika `barang_list`/`summary` berasal dari snapshot
    /// beku saat Operator Satker submit (bukan fetch SIMAN live). Memberi
    /// tahu Validator Wilayah & Pusat bahwa data yg mereka lihat PERSIS sama
    /// dgn yg dilihat operator — transparansi & konsistensi dari hulu.
    pub is_snapshot: bool,
    /// Timestamp RFC3339 saat snapshot dibekukan (`None` jika data live).
    pub snapshot_at: Option<String>,
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
// V029 (Fase 1.6+ / #24) — Snapshot beku analisis kelayakan saat submit
// ============================================================================
//
// Disimpan di kolom JSONB `pengajuan_kebutuhan_bmn_satker.analisis_snapshot_at_submit`
// (ditambahkan V034). Saat Operator Satker submit ke wilayah, hasil analisis
// (usulan ↔ eksisting SIMAN + kondisi + gap) dibekukan. Validator Wilayah &
// Pusat membaca snapshot ini alih-alih fetch SIMAN ulang dari nol, sehingga
// semua aktor melihat angka yg sama (SIMAN bisa berubah antar-waktu).
//
// Struct ini sengaja self-contained (Serialize+Deserialize) dan tidak
// mem-`flatten` record barang penuh, agar payload snapshot stabil terhadap
// perubahan skema `PengajuanKebutuhanBmnBarang`.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalisisSnapshot {
    /// RFC3339 — kapan snapshot dibekukan (saat submit ke wilayah).
    pub snapshot_at: String,
    pub summary: AnalisisSnapshotSummary,
    pub barang: Vec<AnalisisSnapshotBarang>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalisisSnapshotSummary {
    pub total_diminta: i64,
    pub total_existing: i64,
    pub total_gap: i64,
    pub kelayakan_persen: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalisisSnapshotBarang {
    pub barang_id: Uuid,
    pub existing_count: i32,
    pub gap: i32,
    pub recommendation: String,
    pub existing_assets: Vec<AnalisisSnapshotAsset>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalisisSnapshotAsset {
    pub no_aset: String,
    pub nama_aset: String,
    pub kondisi: String,
    pub lokasi: Option<String>,
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
// Laporan Rekap (E-5) — cross-campaign recap for /kebutuhan-bmn/laporan
// ============================================================================

/// One row of the Laporan Kebutuhan BMN recap: a single requested item
/// (`pengajuan_kebutuhan_bmn_satker_barang`) resolved against its owning
/// satker and campaign.
///
/// Status is the **satker-level** `status_kode` (where a request actually
/// sits in the workflow), not the campaign-level one; the label is joined
/// from `ms_aktivitas_bmn`, the same lookup `vw_kebutuhan_bmn_summary` uses.
#[derive(Debug, Clone, Serialize)]
pub struct RekapLaporanRow {
    pub pengajuan_id: Uuid,
    pub pengajuan_nama: String,
    pub tahun: i32,
    pub satker_id: String,
    pub satker_nama: Option<String>,
    pub kode_barang: Option<String>,
    pub nama_barang: String,
    pub satuan: Option<String>,
    pub jumlah: i32,
    pub jml_setuju: i32,
    pub status_kode: i32,
    pub status_nama: Option<String>,
}
