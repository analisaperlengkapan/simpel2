use super::KebutuhanBmnService;
use crate::kebutuhan_bmn::models::*;
use crate::kebutuhan_bmn::repository::KebutuhanBmnRepository;
use crate::shared::error::{AppError, AppResult};
use chrono::Datelike;
use tracing::{info, warn};
use uuid::Uuid;
use validator::Validate;

impl KebutuhanBmnService {
    /// Create a new BMN needs request
    pub async fn create_pengajuan(
        &self,
        request: CreatePengajuanRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<PengajuanDetailResponse> {
        // Validate request
        request
            .validate()
            .map_err(|e| AppError::BadRequest(format!("Validation error: {}", e)))?;

        // Validate date range
        if request.tgl_selesai < request.tgl_mulai {
            return Err(AppError::BadRequest(
                "Tanggal selesai harus setelah tanggal mulai".to_string(),
            ));
        }

        // Validate tahun matches dates
        if request.tahun != request.tgl_mulai.year() {
            warn!(
                "Tahun {} doesn't match tgl_mulai year {}",
                request.tahun,
                request.tgl_mulai.year()
            );
        }

        info!("Creating pengajuan kebutuhan BMN: {}", request.nama);
        let pengajuan = self.repository.create_pengajuan(request, user_id).await?;

        // Build response with related data
        self.get_pengajuan_detail(pengajuan.id).await
    }

    /// Get pengajuan with full details
    pub async fn get_pengajuan_detail(&self, id: Uuid) -> AppResult<PengajuanDetailResponse> {
        let pengajuan = self.repository.get_pengajuan_by_id(id).await?;
        let assets = self.repository.get_pengajuan_assets(id).await?;
        let satkers = self.repository.get_pengajuan_satkers(id).await?;

        // Get allowed transitions from workflow engine
        let allowed_transitions = self.get_allowed_transitions(pengajuan.status);

        Ok(PengajuanDetailResponse {
            pengajuan,
            assets,
            satkers,
            allowed_transitions,
        })
    }

    /// Get paginated list of pengajuan
    pub async fn get_all_pengajuan(
        &self,
        page: i32,
        per_page: i32,
        filter: Option<PengajuanFilter>,
        scope: &crate::shared::satker_scope::SatkerScope,
    ) -> AppResult<(Vec<KebutuhanBmnSummary>, i64)> {
        self.repository
            .get_all_pengajuan(page, per_page, filter, scope)
            .await
    }

    /// Laporan Kebutuhan BMN recap (E-5) — see
    /// [`super::super::repository::KebutuhanBmnRepository::get_rekap_laporan`].
    pub async fn get_rekap_laporan(
        &self,
        filter: RekapLaporanFilter,
        scope: &crate::shared::satker_scope::SatkerScope,
    ) -> AppResult<Vec<RekapLaporanRow>> {
        self.repository.get_rekap_laporan(filter, scope).await
    }

    /// V029 (Fase 1.7): Daftar Kejaksaan Tinggi dari `integrasi.v_satker_wilayah`
    /// — satu-satunya definisi tier wilayah. Dipakai FE untuk dropdown
    /// "Scope satker = wilayah".
    pub async fn list_wilayah(
        &self,
    ) -> AppResult<Vec<crate::kebutuhan_bmn::models::WilayahKejati>> {
        self.repository.list_wilayah().await
    }

    /// Update a pengajuan
    pub async fn update_pengajuan(
        &self,
        id: Uuid,
        request: UpdatePengajuanRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<PengajuanKebutuhanBmn> {
        // Validate request
        request
            .validate()
            .map_err(|e| AppError::BadRequest(format!("Validation error: {}", e)))?;

        // Check current status - only allow updates in draft
        let current = self.repository.get_pengajuan_by_id(id).await?;
        if current.status != KebutuhanBmnStatus::Draft {
            return Err(AppError::BadRequest(
                "Hanya pengajuan dengan status Draft yang dapat diubah".to_string(),
            ));
        }

        // Validate date range if both provided
        if let (Some(tgl_mulai), Some(tgl_selesai)) = (request.tgl_mulai, request.tgl_selesai)
            && tgl_selesai < tgl_mulai
        {
            return Err(AppError::BadRequest(
                "Tanggal selesai harus setelah tanggal mulai".to_string(),
            ));
        }

        info!("Updating pengajuan kebutuhan BMN: {}", id);
        self.repository.update_pengajuan(id, request, user_id).await
    }

    /// Delete a pengajuan (draft only)
    pub async fn delete_pengajuan(&self, id: Uuid) -> AppResult<()> {
        info!("Deleting pengajuan kebutuhan BMN: {}", id);
        self.repository.delete_pengajuan(id).await
    }

    // ========================================================================
    // Workflow Operations
    // ========================================================================
}
