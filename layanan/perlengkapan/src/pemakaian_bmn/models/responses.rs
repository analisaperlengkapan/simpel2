use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use uuid::Uuid;

use super::*;

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
/// BMN: operator input NIP → sistem auto-lookup, tampilkan info pegawai,
/// pemakaian aktif, dan histori. Validator Satker & Approver Satker juga
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
