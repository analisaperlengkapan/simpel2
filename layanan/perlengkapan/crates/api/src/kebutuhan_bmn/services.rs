//! # Kebutuhan BMN Services
//!
//! Business logic layer for BMN needs analysis system.
//! Handles workflow management, validation, and integration with external services.
//!
//! ## SIMAN Integration
//! This service integrates with SIMAN (Sistem Informasi Manajemen Aset Negara)
//! to fetch existing BMN inventory for feasibility analysis.

use chrono::Datelike;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use validator::Validate;

use crate::errors::{AppError, AppResult};
use crate::grpc_clients::AuthencClient;
use crate::workflow::engine::{WorkflowEngine, TransitionRequest};

use super::models::*;
use super::repository::{KebutuhanBmnRepository, PgKebutuhanBmnRepository, UserInfo};
use super::siman_integration::{SimanAsset, SimanIntegration};

// ============================================================================
// Service Interface
// ============================================================================

/// Service for BMN needs analysis business logic
#[derive(Clone)]
pub struct KebutuhanBmnService {
    repository: Arc<PgKebutuhanBmnRepository>,
    authenc_client: AuthencClient,
    siman: Option<Arc<SimanIntegration>>,
    workflow_engine: Arc<WorkflowEngine>,
}

impl KebutuhanBmnService {
    /// Create a new service instance
    pub fn new(
        repository: PgKebutuhanBmnRepository,
        authenc_client: AuthencClient,
        workflow_engine: WorkflowEngine,
    ) -> Self {
        Self {
            repository: Arc::new(repository),
            authenc_client,
            siman: None,
            workflow_engine: Arc::new(workflow_engine),
        }
    }

    /// Create a new service instance with SIMAN integration
    pub fn with_siman(
        repository: PgKebutuhanBmnRepository,
        authenc_client: AuthencClient,
        siman: SimanIntegration,
        workflow_engine: WorkflowEngine,
    ) -> Self {
        Self {
            repository: Arc::new(repository),
            authenc_client,
            siman: Some(Arc::new(siman)),
            workflow_engine: Arc::new(workflow_engine),
        }
    }

    /// Set SIMAN integration after construction
    pub fn set_siman(&mut self, siman: SimanIntegration) {
        self.siman = Some(Arc::new(siman));
    }

    // ========================================================================
    // Pengajuan Operations
    // ========================================================================

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
        if request.tahun != request.tgl_mulai.year() as i32 {
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
    ) -> AppResult<(Vec<KebutuhanBmnSummary>, i64)> {
        self.repository
            .get_all_pengajuan(page, per_page, filter)
            .await
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
        if let (Some(tgl_mulai), Some(tgl_selesai)) = (request.tgl_mulai, request.tgl_selesai) {
            if tgl_selesai < tgl_mulai {
                return Err(AppError::BadRequest(
                    "Tanggal selesai harus setelah tanggal mulai".to_string(),
                ));
            }
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

    /// Transition pengajuan to a new status using workflow engine
    ///
    /// This method now uses the centralized workflow engine for:
    /// - Transition validation
    /// - Role-based authorization
    /// - Audit logging
    /// - Activity tracking
    ///
    /// Requirements: REQ-K004, REQ-W004, REQ-W005
    pub async fn transition_pengajuan_status(
        &self,
        id: Uuid,
        request: WorkflowTransitionRequest,
        user_id: Option<Uuid>,
        user_info: Option<UserInfo>,
    ) -> AppResult<PengajuanKebutuhanBmn> {
        let current = self.repository.get_pengajuan_by_id(id).await?;
        let target_status = KebutuhanBmnStatus::from_code(request.target_status)
            .ok_or_else(|| AppError::BadRequest("Invalid target status code".to_string()))?;

        // Convert status codes to state names for workflow engine
        let from_state = current.status.to_state_name();
        let to_state = target_status.to_state_name();

        // Validate transition using workflow engine
        if !self.workflow_engine.config().is_valid_transition(from_state, to_state) {
            return Err(AppError::BadRequest(format!(
                "Cannot transition from {} to {}",
                current.status.label(),
                target_status.label()
            )));
        }

        // Some transitions require comment
        let requires_comment = matches!(
            target_status,
            KebutuhanBmnStatus::RevisiSatker | KebutuhanBmnStatus::Rejected
        );

        if requires_comment && request.komentar.is_none() {
            return Err(AppError::BadRequest(
                "Komentar diperlukan untuk transisi ini".to_string(),
            ));
        }

        info!(
            "Transitioning pengajuan {} from {} to {} via workflow engine",
            id,
            current.status.label(),
            target_status.label()
        );

        // Use workflow engine for transition
        let transition_request = TransitionRequest {
            entity_id: id,
            from_state: from_state.to_string(),
            to_state: to_state.to_string(),
            user_id: user_id.unwrap_or_else(Uuid::nil),
            catatan: request.komentar.clone(),
            ip_address: "0.0.0.0".to_string(), // TODO: Get from request context
        };

        // Execute transition through workflow engine
        let _transition_result = self
            .workflow_engine
            .transition(transition_request)
            .await
            .map_err(|e| AppError::Internal(format!("Workflow transition failed: {}", e)))?;

        // Get updated pengajuan
        let updated = self.repository.get_pengajuan_by_id(id).await?;

        // Update all satkers to new status and create activity logs
        let satkers = self.repository.get_pengajuan_satkers(id).await?;
        for satker in satkers {
            self.repository
                .update_satker_status(satker.id, target_status.to_code(), user_id)
                .await?;

            // Create activity log for each satker
            self.repository
                .create_aktivitas(
                    satker.id,
                    Some(current.status_kode),
                    target_status.to_code(),
                    &format!("Transition to {}", target_status.label()),
                    request.komentar.clone(),
                    user_id,
                    user_info.clone(),
                )
                .await?;
        }

        Ok(updated)
    }

    /// Transition a single satker's status using workflow engine
    ///
    /// Requirements: REQ-K004, REQ-W004, REQ-W005
    pub async fn transition_satker_status(
        &self,
        satker_id: Uuid,
        request: WorkflowTransitionRequest,
        user_id: Option<Uuid>,
        user_info: Option<UserInfo>,
    ) -> AppResult<PengajuanKebutuhanBmnSatker> {
        let current = self.repository.get_satker_by_id(satker_id).await?;
        let target_status = KebutuhanBmnStatus::from_code(request.target_status)
            .ok_or_else(|| AppError::BadRequest("Invalid target status code".to_string()))?;

        // Convert status codes to state names for workflow engine
        let from_state = current.status.to_state_name();
        let to_state = target_status.to_state_name();

        // Validate transition using workflow engine
        if !self.workflow_engine.config().is_valid_transition(from_state, to_state) {
            return Err(AppError::BadRequest(format!(
                "Cannot transition from {} to {}",
                current.status.label(),
                target_status.label()
            )));
        }

        info!(
            "Transitioning satker {} from {} to {} via workflow engine",
            satker_id,
            current.status.label(),
            target_status.label()
        );

        // Update status
        let updated = self
            .repository
            .update_satker_status(satker_id, target_status.to_code(), user_id)
            .await?;

        // Create activity log
        self.repository
            .create_aktivitas(
                satker_id,
                Some(current.status_kode),
                target_status.to_code(),
                &format!("Transition to {}", target_status.label()),
                request.komentar.clone(),
                user_id,
                user_info,
            )
            .await?;

        Ok(updated)
    }

    /// Get allowed next states for a pengajuan using workflow engine
    ///
    /// Requirements: REQ-K004
    pub fn get_allowed_transitions(&self, current_status: KebutuhanBmnStatus) -> Vec<WorkflowTransitionInfo> {
        let current_state = current_status.to_state_name();
        let next_states = self.workflow_engine.get_next_states(current_state);

        next_states
            .into_iter()
            .filter_map(|state_name| {
                KebutuhanBmnStatus::from_state_name(&state_name).map(|status| {
                    WorkflowTransitionInfo {
                        status_kode: status.to_code(),
                        status_nama: status.label().to_string(),
                        requires_comment: matches!(
                            status,
                            KebutuhanBmnStatus::RevisiSatker | KebutuhanBmnStatus::Rejected
                        ),
                    }
                })
            })
            .collect()
    }

    // ========================================================================
    // Satker Operations
    // ========================================================================

    /// Get satkers for a pengajuan
    pub async fn get_pengajuan_satkers(
        &self,
        pengajuan_id: Uuid,
    ) -> AppResult<Vec<PengajuanKebutuhanBmnSatker>> {
        self.repository.get_pengajuan_satkers(pengajuan_id).await
    }

    /// Get satker with its barang list
    pub async fn get_satker_with_barang(
        &self,
        satker_id: Uuid,
        page: i32,
        per_page: i32,
    ) -> AppResult<SatkerWithBarangResponse> {
        let satker = self.repository.get_satker_by_id(satker_id).await?;
        let (barang_list, total_barang) = self
            .repository
            .get_satker_barang(satker_id, page, per_page, None)
            .await?;

        let total_jumlah: i64 = barang_list.iter().map(|b| b.jumlah as i64).sum();

        Ok(SatkerWithBarangResponse {
            satker,
            barang_list,
            total_barang,
            total_jumlah,
        })
    }

    /// Add a satker to pengajuan
    pub async fn add_satker_to_pengajuan(
        &self,
        pengajuan_id: Uuid,
        satker_id: &str,
        satker_name: Option<String>,
        user_id: Option<Uuid>,
    ) -> AppResult<PengajuanKebutuhanBmnSatker> {
        // Verify pengajuan exists and is editable
        let pengajuan = self.repository.get_pengajuan_by_id(pengajuan_id).await?;
        if pengajuan.status != KebutuhanBmnStatus::Draft {
            return Err(AppError::BadRequest(
                "Satker hanya dapat ditambahkan pada pengajuan dengan status Draft".to_string(),
            ));
        }

        self.repository
            .create_pengajuan_satker(pengajuan_id, satker_id, satker_name, user_id)
            .await
    }

    // ========================================================================
    // Barang Operations
    // ========================================================================

    /// Add a barang to a satker
    pub async fn create_barang(
        &self,
        satker_id: Uuid,
        request: CreateBarangRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<PengajuanKebutuhanBmnBarang> {
        // Validate request
        request
            .validate()
            .map_err(|e| AppError::BadRequest(format!("Validation error: {}", e)))?;

        // Verify satker is in input mode
        let satker = self.repository.get_satker_by_id(satker_id).await?;
        if !matches!(
            satker.status,
            KebutuhanBmnStatus::InputBarang | KebutuhanBmnStatus::RevisiSatker
        ) {
            return Err(AppError::BadRequest(
                "Barang hanya dapat ditambahkan saat status Input Barang atau Revisi".to_string(),
            ));
        }

        info!("Creating barang for satker {}: {}", satker_id, request.nama);
        self.repository
            .create_barang(satker_id, request, user_id)
            .await
    }

    /// Update barang approval (jml_setuju)
    pub async fn update_barang_approval(
        &self,
        barang_id: Uuid,
        request: UpdateBarangApprovalRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<PengajuanKebutuhanBmnBarang> {
        let barang = self.repository.get_barang_by_id(barang_id).await?;

        // Validate jml_setuju doesn't exceed jumlah
        if request.jml_setuju > barang.jumlah {
            return Err(AppError::BadRequest(
                "Jumlah disetujui tidak boleh melebihi jumlah diminta".to_string(),
            ));
        }

        if request.jml_setuju < 0 {
            return Err(AppError::BadRequest(
                "Jumlah disetujui tidak boleh negatif".to_string(),
            ));
        }

        self.repository
            .update_barang_approval(barang_id, request, user_id)
            .await
    }

    /// Delete a barang
    pub async fn delete_barang(&self, barang_id: Uuid, user_id: Option<Uuid>) -> AppResult<()> {
        // Check if barang's satker is in editable status
        let barang = self.repository.get_barang_by_id(barang_id).await?;
        let satker = self
            .repository
            .get_satker_by_id(barang.pengajuan_satker_id)
            .await?;

        if !matches!(
            satker.status,
            KebutuhanBmnStatus::InputBarang | KebutuhanBmnStatus::RevisiSatker
        ) {
            return Err(AppError::BadRequest(
                "Barang hanya dapat dihapus saat status Input Barang atau Revisi".to_string(),
            ));
        }

        info!("Deleting barang: {} by user {:?}", barang_id, user_id);
        self.repository.delete_barang(barang_id).await
    }

    /// Set priorities for multiple barang
    pub async fn set_barang_prioritas(&self, request: SetPrioritasRequest) -> AppResult<()> {
        // Validate all items have valid prioritas
        for item in &request.items {
            if item.prioritas < 0 {
                return Err(AppError::BadRequest(
                    "Prioritas tidak boleh negatif".to_string(),
                ));
            }
        }

        info!("Setting prioritas for {} items", request.items.len());
        self.repository.set_barang_prioritas(request.items).await
    }

    // ========================================================================
    // Analisis Kelayakan
    // ========================================================================

    /// Get feasibility analysis for a satker
    /// Compares requested goods against existing SIMAN inventory
    pub async fn get_analisis_kelayakan(
        &self,
        satker_id: Uuid,
    ) -> AppResult<AnalisisKelayakanResponse> {
        let satker = self.repository.get_satker_by_id(satker_id).await?;
        let (barang_list, _) = self
            .repository
            .get_satker_barang(satker_id, 1, 1000, None)
            .await?;

        let mut total_diminta: i64 = 0;
        let mut total_existing: i64 = 0;
        let mut barang_with_inventory = Vec::new();

        for barang in barang_list {
            total_diminta += barang.jumlah as i64;

            // Fetch existing assets from SIMAN if integration is available
            let (existing_count, existing_assets) = if let Some(ref siman) = self.siman {
                let matching_assets = siman
                    .get_matching_assets(&barang.nama, None, 20)
                    .await
                    .unwrap_or_default();

                let count = matching_assets.len() as i32;
                let assets_info: Vec<ExistingAssetInfo> = matching_assets
                    .into_iter()
                    .map(|a| ExistingAssetInfo {
                        no_aset: a.no_aset,
                        nama_aset: a.nama_aset,
                        kondisi: a.kondisi,
                        lokasi: a.lokasi,
                    })
                    .collect();

                (count, assets_info)
            } else {
                // Fallback to stored existing_count from barang table
                (barang.existing_count, vec![])
            };

            total_existing += existing_count as i64;

            let gap = barang.jumlah - existing_count;
            let recommendation = self.generate_recommendation(gap, barang.jumlah);

            barang_with_inventory.push(BarangWithExistingInventory {
                barang,
                existing_assets,
                gap,
                recommendation,
            });
        }

        let total_gap = total_diminta - total_existing;
        let kelayakan_persen = if total_diminta > 0 {
            (total_existing as f64 / total_diminta as f64) * 100.0
        } else {
            100.0
        };

        Ok(AnalisisKelayakanResponse {
            satker,
            barang_list: barang_with_inventory,
            summary: AnalisisSummary {
                total_diminta,
                total_existing,
                total_gap,
                kelayakan_persen,
            },
        })
    }

    /// Generate recommendation based on gap analysis
    fn generate_recommendation(&self, gap: i32, jumlah_diminta: i32) -> String {
        if gap <= 0 {
            "Tidak perlu pengadaan - stok mencukupi".to_string()
        } else if gap < jumlah_diminta / 2 {
            format!("Perlu pengadaan {} unit (sebagian)", gap)
        } else if gap == jumlah_diminta {
            format!("Perlu pengadaan {} unit (tidak ada stok)", gap)
        } else {
            format!("Perlu pengadaan {} unit (prioritas tinggi)", gap)
        }
    }

    /// Search for existing assets from SIMAN
    /// Returns similar assets from inventory for a specific barang
    pub async fn search_siman_assets(
        &self,
        search_term: &str,
        kategori: Option<&str>,
        limit: usize,
    ) -> AppResult<Vec<SimanAsset>> {
        let siman = self
            .siman
            .as_ref()
            .ok_or_else(|| AppError::Internal("SIMAN integration not configured".to_string()))?;

        siman
            .get_matching_assets(search_term, kategori, limit)
            .await
    }

    /// Get satker asset summary from SIMAN
    pub async fn get_satker_siman_summary(
        &self,
        satker_id: &str,
    ) -> AppResult<super::siman_integration::SatkerAssetSummary> {
        let siman = self
            .siman
            .as_ref()
            .ok_or_else(|| AppError::Internal("SIMAN integration not configured".to_string()))?;

        siman.get_satker_asset_summary(satker_id).await
    }

    // ========================================================================
    // Dashboard & Statistics
    // ========================================================================

    /// Get dashboard statistics
    pub async fn get_dashboard_stats(&self) -> AppResult<KebutuhanBmnDashboardStats> {
        self.repository.get_dashboard_stats().await
    }

    /// Get workflow history for a satker
    pub async fn get_satker_aktivitas(
        &self,
        satker_id: Uuid,
    ) -> AppResult<Vec<PengajuanKebutuhanBmnAktivitas>> {
        self.repository.get_satker_aktivitas(satker_id).await
    }

    // ========================================================================
    // Search Operations
    // ========================================================================

    /// Search kebutuhan BMN with full-text search and filters
    ///
    /// Uses PostgreSQL's Indonesian text search configuration for relevance ranking
    /// Requirements: REQ-K005
    pub async fn search_kebutuhan(
        &self,
        query: lib_perlengkapan::search::SearchQuery,
    ) -> AppResult<lib_perlengkapan::search::SearchResults<KebutuhanBmnSummary>> {
        use lib_perlengkapan::search::SearchEngineDb;

        let search_engine = SearchEngineDb::new(self.repository.pool().clone());

        search_engine
            .search_kebutuhan(query)
            .await
            .map_err(|e| AppError::Internal(format!("Search failed: {}", e)))
    }

    /// Get search suggestions based on partial query
    ///
    /// Returns top N most relevant suggestions for autocomplete
    /// Requirements: REQ-K005
    pub async fn get_search_suggestions(
        &self,
        partial_query: &str,
        limit: i32,
    ) -> AppResult<Vec<String>> {
        use lib_perlengkapan::search::SearchEngineDb;

        let search_engine = SearchEngineDb::new(self.repository.pool().clone());

        search_engine
            .get_suggestions(partial_query, limit)
            .await
            .map_err(|e| AppError::Internal(format!("Suggestions failed: {}", e)))
    }

    // ========================================================================
    // Batch Operations
    // ========================================================================

    /// Batch approve multiple kebutuhan
    ///
    /// Processes each item independently - failures don't affect other items
    /// Requirements: REQ-K004
    pub async fn batch_approve_kebutuhan(
        &self,
        request: super::models::BatchApproveRequest,
        user_id: Option<Uuid>,
        user_info: Option<UserInfo>,
    ) -> AppResult<super::models::BatchOperationResponse> {
        use super::models::{BatchOperationItemResult, BatchOperationResponse};
        use chrono::Utc;

        // Validate batch size
        if request.kebutuhan_ids.is_empty() {
            return Err(AppError::BadRequest("Batch cannot be empty".to_string()));
        }
        if request.kebutuhan_ids.len() > 500 {
            return Err(AppError::BadRequest(
                "Batch size cannot exceed 500 items".to_string(),
            ));
        }

        let batch_id = Uuid::new_v4();
        let operation_type = "batch_approve".to_string();
        let executed_at = Utc::now();

        info!(
            "Starting batch approve operation {} for {} items by user {:?}",
            batch_id,
            request.kebutuhan_ids.len(),
            user_id
        );

        // Process each item independently
        let mut results = Vec::new();
        let mut successful_count = 0;
        let mut failed_count = 0;

        for kebutuhan_id in &request.kebutuhan_ids {
            let result = match self.process_single_approval(*kebutuhan_id, &request.komentar, user_id, user_info.clone()).await {
                Ok(_) => {
                    successful_count += 1;
                    BatchOperationItemResult {
                        kebutuhan_id: *kebutuhan_id,
                        success: true,
                        error_message: None,
                    }
                }
                Err(e) => {
                    failed_count += 1;
                    warn!("Failed to approve kebutuhan {}: {}", kebutuhan_id, e);
                    BatchOperationItemResult {
                        kebutuhan_id: *kebutuhan_id,
                        success: false,
                        error_message: Some(e.to_string()),
                    }
                }
            };
            results.push(result);
        }

        // Audit log the batch operation
        self.log_batch_operation(
            batch_id,
            &operation_type,
            request.kebutuhan_ids.len(),
            successful_count,
            failed_count,
            user_id,
        )
        .await?;

        info!(
            "Batch approve operation {} completed: {}/{} successful",
            batch_id, successful_count, request.kebutuhan_ids.len()
        );

        Ok(BatchOperationResponse {
            batch_id,
            total_items: request.kebutuhan_ids.len(),
            successful_items: successful_count,
            failed_items: failed_count,
            results,
            operation_type,
            executed_at,
            executed_by: user_id,
        })
    }

    /// Batch reject multiple kebutuhan
    ///
    /// Processes each item independently - failures don't affect other items
    /// Requirements: REQ-K004
    pub async fn batch_reject_kebutuhan(
        &self,
        request: super::models::BatchRejectRequest,
        user_id: Option<Uuid>,
        user_info: Option<UserInfo>,
    ) -> AppResult<super::models::BatchOperationResponse> {
        use super::models::{BatchOperationItemResult, BatchOperationResponse};
        use chrono::Utc;

        // Validate batch size
        if request.kebutuhan_ids.is_empty() {
            return Err(AppError::BadRequest("Batch cannot be empty".to_string()));
        }
        if request.kebutuhan_ids.len() > 500 {
            return Err(AppError::BadRequest(
                "Batch size cannot exceed 500 items".to_string(),
            ));
        }

        let batch_id = Uuid::new_v4();
        let operation_type = "batch_reject".to_string();
        let executed_at = Utc::now();

        info!(
            "Starting batch reject operation {} for {} items by user {:?}",
            batch_id,
            request.kebutuhan_ids.len(),
            user_id
        );

        // Process each item independently
        let mut results = Vec::new();
        let mut successful_count = 0;
        let mut failed_count = 0;

        for kebutuhan_id in &request.kebutuhan_ids {
            let result = match self.process_single_rejection(*kebutuhan_id, &request.komentar, user_id, user_info.clone()).await {
                Ok(_) => {
                    successful_count += 1;
                    BatchOperationItemResult {
                        kebutuhan_id: *kebutuhan_id,
                        success: true,
                        error_message: None,
                    }
                }
                Err(e) => {
                    failed_count += 1;
                    warn!("Failed to reject kebutuhan {}: {}", kebutuhan_id, e);
                    BatchOperationItemResult {
                        kebutuhan_id: *kebutuhan_id,
                        success: false,
                        error_message: Some(e.to_string()),
                    }
                }
            };
            results.push(result);
        }

        // Audit log the batch operation
        self.log_batch_operation(
            batch_id,
            &operation_type,
            request.kebutuhan_ids.len(),
            successful_count,
            failed_count,
            user_id,
        )
        .await?;

        info!(
            "Batch reject operation {} completed: {}/{} successful",
            batch_id, successful_count, request.kebutuhan_ids.len()
        );

        Ok(BatchOperationResponse {
            batch_id,
            total_items: request.kebutuhan_ids.len(),
            successful_items: successful_count,
            failed_items: failed_count,
            results,
            operation_type,
            executed_at,
            executed_by: user_id,
        })
    }

    /// Batch update status for multiple kebutuhan
    ///
    /// Processes each item independently - failures don't affect other items
    /// Requirements: REQ-K004
    pub async fn batch_update_status(
        &self,
        request: super::models::BatchUpdateStatusRequest,
        user_id: Option<Uuid>,
        user_info: Option<UserInfo>,
    ) -> AppResult<super::models::BatchOperationResponse> {
        use super::models::{BatchOperationItemResult, BatchOperationResponse};
        use chrono::Utc;

        // Validate batch size
        if request.kebutuhan_ids.is_empty() {
            return Err(AppError::BadRequest("Batch cannot be empty".to_string()));
        }
        if request.kebutuhan_ids.len() > 500 {
            return Err(AppError::BadRequest(
                "Batch size cannot exceed 500 items".to_string(),
            ));
        }

        // Validate target status
        let target_status = KebutuhanBmnStatus::from_code(request.target_status)
            .ok_or_else(|| AppError::BadRequest("Invalid target status code".to_string()))?;

        let batch_id = Uuid::new_v4();
        let operation_type = format!("batch_update_status_{}", target_status.label());
        let executed_at = Utc::now();

        info!(
            "Starting batch update status operation {} for {} items to {} by user {:?}",
            batch_id,
            request.kebutuhan_ids.len(),
            target_status.label(),
            user_id
        );

        // Process each item independently
        let mut results = Vec::new();
        let mut successful_count = 0;
        let mut failed_count = 0;

        for kebutuhan_id in &request.kebutuhan_ids {
            let result = match self.process_single_status_update(
                *kebutuhan_id,
                request.target_status,
                &request.komentar,
                user_id,
                user_info.clone(),
            ).await {
                Ok(_) => {
                    successful_count += 1;
                    BatchOperationItemResult {
                        kebutuhan_id: *kebutuhan_id,
                        success: true,
                        error_message: None,
                    }
                }
                Err(e) => {
                    failed_count += 1;
                    warn!("Failed to update status for kebutuhan {}: {}", kebutuhan_id, e);
                    BatchOperationItemResult {
                        kebutuhan_id: *kebutuhan_id,
                        success: false,
                        error_message: Some(e.to_string()),
                    }
                }
            };
            results.push(result);
        }

        // Audit log the batch operation
        self.log_batch_operation(
            batch_id,
            &operation_type,
            request.kebutuhan_ids.len(),
            successful_count,
            failed_count,
            user_id,
        )
        .await?;

        info!(
            "Batch update status operation {} completed: {}/{} successful",
            batch_id, successful_count, request.kebutuhan_ids.len()
        );

        Ok(BatchOperationResponse {
            batch_id,
            total_items: request.kebutuhan_ids.len(),
            successful_items: successful_count,
            failed_items: failed_count,
            results,
            operation_type,
            executed_at,
            executed_by: user_id,
        })
    }

    // ========================================================================
    // Batch Operation Helpers
    // ========================================================================

    /// Process approval for a single kebutuhan
    async fn process_single_approval(
        &self,
        kebutuhan_id: Uuid,
        komentar: &Option<String>,
        user_id: Option<Uuid>,
        user_info: Option<UserInfo>,
    ) -> AppResult<()> {
        let transition_request = WorkflowTransitionRequest {
            target_status: KebutuhanBmnStatus::Approved.to_code(),
            komentar: komentar.clone(),
        };

        self.transition_pengajuan_status(kebutuhan_id, transition_request, user_id, user_info)
            .await?;

        Ok(())
    }

    /// Process rejection for a single kebutuhan
    async fn process_single_rejection(
        &self,
        kebutuhan_id: Uuid,
        komentar: &str,
        user_id: Option<Uuid>,
        user_info: Option<UserInfo>,
    ) -> AppResult<()> {
        let transition_request = WorkflowTransitionRequest {
            target_status: KebutuhanBmnStatus::Rejected.to_code(),
            komentar: Some(komentar.to_string()),
        };

        self.transition_pengajuan_status(kebutuhan_id, transition_request, user_id, user_info)
            .await?;

        Ok(())
    }

    /// Process status update for a single kebutuhan
    async fn process_single_status_update(
        &self,
        kebutuhan_id: Uuid,
        target_status: i32,
        komentar: &Option<String>,
        user_id: Option<Uuid>,
        user_info: Option<UserInfo>,
    ) -> AppResult<()> {
        let transition_request = WorkflowTransitionRequest {
            target_status,
            komentar: komentar.clone(),
        };

        self.transition_pengajuan_status(kebutuhan_id, transition_request, user_id, user_info)
            .await?;

        Ok(())
    }

    /// Log batch operation to audit trail
    async fn log_batch_operation(
        &self,
        batch_id: Uuid,
        operation_type: &str,
        total_items: usize,
        successful_items: usize,
        failed_items: usize,
        user_id: Option<Uuid>,
    ) -> AppResult<()> {
        let query = r#"
            INSERT INTO perlengkapan.batch_operation_log (
                batch_id,
                operation_type,
                total_items,
                successful_items,
                failed_items,
                user_id,
                created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, NOW())
        "#;

        self.repository
            .pool()
            .get()
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .execute(
                query,
                &[
                    &batch_id,
                    &operation_type,
                    &(total_items as i32),
                    &(successful_items as i32),
                    &(failed_items as i32),
                    &user_id,
                ],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn create_valid_request() -> CreatePengajuanRequest {
        CreatePengajuanRequest {
            nama: "Test Pengajuan".to_string(),
            deskripsi: Some("Test description".to_string()),
            tahun: 2026,
            tgl_mulai: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            tgl_selesai: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
            pilihan_satker: Some("semua".to_string()),
            satker_ids: vec![],
            asset_types: vec![],
        }
    }

    #[test]
    fn test_validate_request() {
        let request = create_valid_request();
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_validate_request_short_name() {
        let mut request = create_valid_request();
        request.nama = "AB".to_string(); // Too short
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_validate_request_invalid_tahun() {
        let mut request = create_valid_request();
        request.tahun = 2019; // Before 2020
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_validate_barang_request() {
        let request = CreateBarangRequest {
            nama: "Test Barang".to_string(),
            kode_barang: Some("123".to_string()),
            jumlah: 10,
            satuan: "Unit".to_string(),
            alasan: Some("Test".to_string()),
            keterangan: None,
            file_pendukung: vec![],
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_validate_barang_empty_name() {
        let request = CreateBarangRequest {
            nama: "".to_string(),
            kode_barang: None,
            jumlah: 10,
            satuan: "Unit".to_string(),
            alasan: None,
            keterangan: None,
            file_pendukung: vec![],
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_validate_barang_zero_jumlah() {
        let request = CreateBarangRequest {
            nama: "Test".to_string(),
            kode_barang: None,
            jumlah: 0, // Invalid
            satuan: "Unit".to_string(),
            alasan: None,
            keterangan: None,
            file_pendukung: vec![],
        };
        assert!(request.validate().is_err());
    }
}
