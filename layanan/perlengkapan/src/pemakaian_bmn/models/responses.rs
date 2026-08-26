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
    /// Imperative label for the control that performs this move
    /// (`PemakaianBmnAction::action_label`). `label` names the resulting state;
    /// a button needs the command form. Server-authored so the workflow
    /// vocabulary has one home and the FE renders it verbatim.
    pub action_label: String,
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
/// INTERNAL page carrier (repository -> service -> handler). **Not a wire
/// shape.** `list_permits` converts this into the shared
/// `lib_perlengkapan::response::PaginatedResponse`, which is what the FE
/// fetcher and the e2e specs are typed against. Serializing this struct
/// directly puts the rows at `data.data` and breaks both — do not return it
/// from a handler.
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
/// Permits grouped by satker.
///
/// Keyed on the authoritative MySIMKARI `satker_code` (V003). The previous
/// `satker_id: Uuid` was the legacy client-supplied `pegawai_satker_id`, which
/// carried no foreign key and no relation to the code the caller authenticates
/// with — so grouping on it split one satker across as many buckets as its rows
/// had distinct UUIDs, and could not be reconciled with the caller's scope.
#[derive(Debug, Clone, Serialize)]
pub struct PermitsBySatker {
    pub satker_code: Option<String>,
    pub satker_nama: Option<String>,
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
    /// BMN kelas ber-izin (kondisi Baik di SIMAN) yg TIDAK sedang dipakai =
    /// total − terpakai, **dalam scope pemanggil**: pusat/admin nasional,
    /// validator wilayah sebatas wilayahnya, operator sebatas satkernya.
    ///
    /// `None` hanya bila SIMAN tidak terjangkau (best-effort — kartu lain tetap
    /// tersaji) atau bila pemanggil memfilter `jenis_bmn`, yang tidak punya
    /// padanan tepat di sisi SIMAN.
    pub tidak_dipakai: Option<i64>,
}

/// Satu baris "siapa memakai BMN apa" pada dashboard monitoring.
///
/// Kolomnya mengikuti permintaan stakeholder: satker mana, nama barangnya, NUP
/// berapa, siapa pegawai yang memakai, dan berapa jangka waktu pemakaiannya.
///
/// Catatan penamaan (koreksi stakeholder): `nama_barang` adalah nama standar
/// yang melekat pada kode barang, sedangkan `merk`/`tipe` adalah penamaan bebas
/// milik operator SIMAN. Keduanya dipisah agar tidak tertukar — satu kolom
/// gabungan akan menampilkan label bebas seolah-olah nama resmi barang.
///
/// Identitas aset di sini adalah tiga serangkai **kode satker + kode barang +
/// NUP**, jadi ketiganya ikut, bukan NUP saja.
#[derive(Debug, Clone, Serialize)]
pub struct PemakaianBmnMonitoringRow {
    pub id: Uuid,
    pub nomor_izin: Option<String>,
    /// MySIMKARI `kode_satker` — kolom otoritatif hasil turunan klaim saat
    /// izin dibuat (V003), bukan UUID `pegawai_satker_id` warisan.
    pub satker_code: Option<String>,
    pub satker_nama: Option<String>,
    pub kode_barang: String,
    /// Nama standar barang sesuai kode barang.
    pub nama_barang: String,
    /// NUP — Nomor Urut Pendaftaran.
    pub nup: String,
    /// Label bebas dari operator SIMAN (merk/tipe), bila ada.
    pub merk_tipe: Option<String>,
    pub jenis_bmn: String,
    pub pegawai_nip: String,
    pub pegawai_nama: String,
    pub pegawai_jabatan: Option<String>,
    pub tanggal_mulai: NaiveDate,
    pub tanggal_selesai: NaiveDate,
    /// Jangka waktu pemakaian dalam hari (selesai − mulai).
    pub durasi_hari: i64,
    /// Sisa hari sampai izin berakhir. Negatif = sudah lewat tanggal selesai
    /// namun izin belum di-expire — kondisi yang memang perlu terlihat.
    pub sisa_hari: i64,
    pub status: String,
}

/// Halaman hasil listing monitoring pemakaian BMN.
#[derive(Debug, Clone, Serialize)]
pub struct PemakaianBmnMonitoringPage {
    pub data: Vec<PemakaianBmnMonitoringRow>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}
