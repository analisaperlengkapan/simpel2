//! # Kebutuhan BMN Repository
//!
//! Database operations for BMN needs analysis system.
//! Uses PostgreSQL via tokio-postgres with connection pooling.

use async_trait::async_trait;
use deadpool_postgres::Pool;
use serde_json::Value;
use tracing::error;
use uuid::Uuid;

use super::models::*;
use crate::shared::error::{AppError, AppResult};

// ============================================================================
// Repository Trait
// ============================================================================

#[cfg_attr(test, mockall::automock)]
#[async_trait]
// Some repository methods carry many columns as positional params; bundling
// them into structs would only add indirection at the call sites.
#[allow(clippy::too_many_arguments)]
pub trait KebutuhanBmnRepository: Send + Sync {
    // Pengajuan CRUD
    async fn create_pengajuan(
        &self,
        request: CreatePengajuanRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<PengajuanKebutuhanBmn>;

    async fn get_pengajuan_by_id(&self, id: Uuid) -> AppResult<PengajuanKebutuhanBmn>;

    async fn get_all_pengajuan(
        &self,
        page: i32,
        per_page: i32,
        filter: Option<PengajuanFilter>,
        // Campaign-visibility scope (#66): restricts the list to RKBMN campaigns
        // that target the caller's satker/wilayah; cross-satker roles see all.
        scope: &crate::shared::satker_scope::SatkerScope,
    ) -> AppResult<(Vec<KebutuhanBmnSummary>, i64)>;

    /// Laporan Kebutuhan BMN recap (E-5): every requested item across the
    /// campaigns visible to `scope`, one row per
    /// `pengajuan_kebutuhan_bmn_satker_barang`.
    async fn get_rekap_laporan(
        &self,
        filter: RekapLaporanFilter,
        // Same campaign-visibility scope as the pengajuan list (#66/#71): a
        // report must never widen what its underlying list would show.
        scope: &crate::shared::satker_scope::SatkerScope,
    ) -> AppResult<Vec<RekapLaporanRow>>;

    async fn update_pengajuan(
        &self,
        id: Uuid,
        request: UpdatePengajuanRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<PengajuanKebutuhanBmn>;

    async fn delete_pengajuan(&self, id: Uuid) -> AppResult<()>;

    // Pengajuan Assets
    async fn get_pengajuan_assets(
        &self,
        pengajuan_id: Uuid,
    ) -> AppResult<Vec<PengajuanKebutuhanBmnAsset>>;
    async fn create_pengajuan_asset(
        &self,
        pengajuan_id: Uuid,
        request: CreateAssetTypeRequest,
    ) -> AppResult<PengajuanKebutuhanBmnAsset>;

    // Satker Operations
    async fn get_pengajuan_satkers(
        &self,
        pengajuan_id: Uuid,
    ) -> AppResult<Vec<PengajuanKebutuhanBmnSatker>>;
    async fn get_satker_by_id(&self, id: Uuid) -> AppResult<PengajuanKebutuhanBmnSatker>;
    async fn create_pengajuan_satker(
        &self,
        pengajuan_id: Uuid,
        satker_id: &str,
        satker_name: Option<String>,
        user_id: Option<Uuid>,
    ) -> AppResult<PengajuanKebutuhanBmnSatker>;
    async fn update_satker_status(
        &self,
        satker_id: Uuid,
        new_status: i32,
        user_id: Option<Uuid>,
    ) -> AppResult<PengajuanKebutuhanBmnSatker>;

    // Barang Operations
    async fn get_satker_barang(
        &self,
        satker_id: Uuid,
        page: i32,
        per_page: i32,
        filter: Option<BarangFilter>,
    ) -> AppResult<(Vec<PengajuanKebutuhanBmnBarang>, i64)>;
    async fn get_barang_by_id(&self, id: Uuid) -> AppResult<PengajuanKebutuhanBmnBarang>;
    async fn create_barang(
        &self,
        satker_id: Uuid,
        request: CreateBarangRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<PengajuanKebutuhanBmnBarang>;
    async fn update_barang_approval(
        &self,
        barang_id: Uuid,
        request: UpdateBarangApprovalRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<PengajuanKebutuhanBmnBarang>;
    async fn delete_barang(&self, id: Uuid) -> AppResult<()>;
    async fn set_barang_prioritas(&self, items: Vec<PrioritasItem>) -> AppResult<()>;

    // Workflow Aktivitas
    async fn get_satker_aktivitas(
        &self,
        satker_id: Uuid,
    ) -> AppResult<Vec<PengajuanKebutuhanBmnAktivitas>>;
    async fn create_aktivitas(
        &self,
        satker_id: Uuid,
        from_status: Option<i32>,
        to_status: i32,
        aksi: &str,
        komentar: Option<String>,
        user_id: Option<Uuid>,
        user_info: Option<UserInfo>,
    ) -> AppResult<PengajuanKebutuhanBmnAktivitas>;

    // Dashboard Statistics
    async fn get_dashboard_stats(&self) -> AppResult<KebutuhanBmnDashboardStats>;

    // Pengajuan Status Update
    async fn update_pengajuan_status(
        &self,
        id: Uuid,
        new_status: i32,
        user_id: Option<Uuid>,
    ) -> AppResult<PengajuanKebutuhanBmn>;

    // Satker Submit Data (lampiran, catatan) for SubmitWilayah
    async fn update_satker_submit_data(
        &self,
        satker_id: Uuid,
        catatan_satker: Option<String>,
        lampiran_surat_permohonan: Option<String>,
        lampiran_pendukung: Option<Vec<LampiranItem>>,
    ) -> AppResult<()>;

    // Validator Wilayah info update
    async fn update_satker_validator_wilayah(
        &self,
        satker_id: Uuid,
        validator_id: Option<Uuid>,
        catatan: Option<String>,
    ) -> AppResult<()>;

    /// V029 (#24): bekukan snapshot analisis kelayakan (usulan ↔ eksisting
    /// SIMAN) ke kolom JSONB `analisis_snapshot_at_submit` saat submit.
    async fn save_analisis_snapshot(&self, satker_id: Uuid, snapshot: &Value) -> AppResult<()>;

    /// V029 (#24): baca snapshot beku analisis kelayakan (None jika belum
    /// pernah disubmit / kolom NULL).
    async fn get_analisis_snapshot(&self, satker_id: Uuid) -> AppResult<Option<Value>>;

    // Validator Pusat info update
    async fn update_satker_validator_pusat(
        &self,
        satker_id: Uuid,
        validator_id: Option<Uuid>,
        catatan: Option<String>,
        is_approved: bool,
    ) -> AppResult<()>;

    // Update pengajuan laporan URL
    async fn update_pengajuan_laporan(
        &self,
        id: Uuid,
        laporan_url: &str,
        laporan_format: &str,
    ) -> AppResult<()>;

    /// V029 (Fase 1.7): Resolve daftar `kode_satker` yg termasuk dalam
    /// `wilayah` tertentu (Kejaksaan Tinggi). Sumber: tabel cache
    /// `integrasi.mysimkari_satker`. Kosong jika tidak ada satker /
    /// wilayah tidak dikenal.
    async fn list_satker_codes_by_wilayah(&self, wilayah: &str) -> AppResult<Vec<String>>;

    /// V029 (Fase 1.7): Daftar wilayah distinct yg ada di
    /// `integrasi.mysimkari_satker` — dipakai FE utk dropdown.
    async fn list_wilayah(&self) -> AppResult<Vec<String>>;

    // ========================================================================
    // V029 (Fase 1.6): Allowed-list BMN
    // ========================================================================

    /// Insert satu entry allowed BMN utk pengajuan.
    async fn insert_bmn_referensi(
        &self,
        pengajuan_id: Uuid,
        request: CreateBmnReferensiRequest,
    ) -> AppResult<PengajuanBmnReferensi>;

    /// List semua allowed BMN utk pengajuan.
    async fn list_bmn_referensi(&self, pengajuan_id: Uuid)
    -> AppResult<Vec<PengajuanBmnReferensi>>;

    /// Cek apakah kode_barang ada dlm allowed-list pengajuan.
    /// Return `true` jika allowed-list kosong (legacy mode: no whitelist
    /// enforcement) ATAU kode_barang ada di whitelist.
    async fn is_bmn_allowed_for_pengajuan(
        &self,
        pengajuan_id: Uuid,
        kode_barang: &str,
    ) -> AppResult<bool>;
}

/// User information for audit trail
#[derive(Debug, Clone)]
pub struct UserInfo {
    pub nip: Option<String>,
    pub nama: Option<String>,
    pub pangkat: Option<String>,
    pub jabatan: Option<String>,
    pub role: Option<String>,
}

// ============================================================================
// PostgreSQL Implementation
// ============================================================================

#[derive(Clone)]
pub struct PgKebutuhanBmnRepository {
    pool: Pool,
}

impl PgKebutuhanBmnRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// Get a reference to the database pool
    ///
    /// Used by search engine and other components that need direct pool access
    pub fn pool(&self) -> &Pool {
        &self.pool
    }

    async fn get_client(&self) -> AppResult<deadpool_postgres::Client> {
        self.pool.get().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AppError::Internal(format!("Database connection error: {}", e))
        })
    }
}

mod pg;

#[cfg(test)]
mod tests;
