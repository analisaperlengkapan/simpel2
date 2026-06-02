//! # Kebutuhan BMN Services
//!
//! Business logic layer for BMN needs analysis system.
//! Handles workflow management, validation, and integration with external services.
//!
//! ## SIMAN Integration
//! This service integrates with SIMAN (Sistem Informasi Manajemen Aset Negara)
//! to fetch existing BMN inventory for feasibility analysis.

use chrono::Datelike;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;
use validator::Validate;

use crate::shared::error::{AppError, AppResult};
use crate::shared::grpc::clients::AuthencClient;
use crate::shared::grpc::clients::IntegrasiClient;
use crate::shared::grpc::clients::integrasi::v1::{DataSource, SyncState};
use crate::workflow::engine::{TransitionRequest, WorkflowEngine};

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
    #[allow(dead_code)]
    authenc_client: AuthencClient,
    integrasi_client: Option<IntegrasiClient>,
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
            integrasi_client: None,
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
            integrasi_client: None,
            siman: Some(Arc::new(siman)),
            workflow_engine: Arc::new(workflow_engine),
        }
    }

    /// Set layanan-integrasi gRPC client after construction
    pub fn with_integrasi_client(mut self, integrasi_client: IntegrasiClient) -> Self {
        self.integrasi_client = Some(integrasi_client);
        self
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
    ) -> AppResult<(Vec<KebutuhanBmnSummary>, i64)> {
        self.repository
            .get_all_pengajuan(page, per_page, filter)
            .await
    }

    /// V029 (Fase 1.7): Daftar nama wilayah Kejaksaan Tinggi distinct dari
    /// `integrasi.mysimkari_satker.wilayah`. Dipakai FE untuk dropdown
    /// "Scope satker = wilayah".
    pub async fn list_wilayah(&self) -> AppResult<Vec<String>> {
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
        client_ip: String,
    ) -> AppResult<PengajuanKebutuhanBmn> {
        let current = self.repository.get_pengajuan_by_id(id).await?;
        let target_status = KebutuhanBmnStatus::from_code(request.target_status)
            .ok_or_else(|| AppError::BadRequest("Invalid target status code".to_string()))?;

        // Convert status codes to state names for workflow engine
        let from_state = current.status.to_state_name();
        let to_state = target_status.to_state_name();

        // Validate transition using workflow engine
        if !self
            .workflow_engine
            .config()
            .is_valid_transition(from_state, to_state)
        {
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
            ip_address: client_ip.clone(),
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
        if !self
            .workflow_engine
            .config()
            .is_valid_transition(from_state, to_state)
        {
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
    pub fn get_allowed_transitions(
        &self,
        current_status: KebutuhanBmnStatus,
    ) -> Vec<WorkflowTransitionInfo> {
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
    // Satker Workflow Operations (Validator Wilayah & Pusat)
    // ========================================================================

    /// Operator Satker submits to Validator Wilayah
    /// Updates satker with lampiran and catatan, transitions to SubmitWilayah
    pub async fn submit_satker_to_wilayah(
        &self,
        satker_id: Uuid,
        request: SubmitKebutuhanSatkerRequest,
        user_id: Option<Uuid>,
        user_info: Option<UserInfo>,
    ) -> AppResult<PengajuanKebutuhanBmnSatker> {
        let current = self.repository.get_satker_by_id(satker_id).await?;

        // Only allow from InputBarang or RevisiSatker
        if !matches!(
            current.status,
            KebutuhanBmnStatus::InputBarang | KebutuhanBmnStatus::RevisiSatker
        ) {
            return Err(AppError::BadRequest(
                "Pengajuan hanya bisa dikirim saat status Input Barang atau Revisi".to_string(),
            ));
        }

        // Update satker with lampiran and catatan
        self.repository
            .update_satker_submit_data(
                satker_id,
                request.catatan_satker.clone(),
                Some(request.lampiran_surat_permohonan.clone()),
                Some(request.lampiran_pendukung.clone()),
            )
            .await?;

        // V029 (#24): bekukan snapshot analisis (usulan ↔ eksisting SIMAN +
        // kondisi + gap) saat submit. Validator Wilayah & Pusat membaca
        // snapshot ini (lihat `get_analisis_kelayakan`) sehingga melihat data
        // konsisten dgn operator — tidak ada fetch SIMAN ulang yg bisa drift.
        // Best-effort: kegagalan snapshot tidak memblok submit.
        let (barang_list, _) = self
            .repository
            .get_satker_barang(satker_id, 1, 1000, None)
            .await?;
        let (_, _, snapshot) = self.compute_live_analisis(barang_list).await;
        match serde_json::to_value(&snapshot) {
            Ok(v) => {
                if let Err(e) = self.repository.save_analisis_snapshot(satker_id, &v).await {
                    warn!(
                        "Gagal menyimpan snapshot analisis utk satker {}: {}",
                        satker_id, e
                    );
                }
            }
            Err(e) => warn!("Gagal serialize snapshot analisis: {}", e),
        }

        info!(
            "Operator Satker submitting {} to Validator Wilayah",
            satker_id
        );

        // Transition to SubmitWilayah
        let transition_request = WorkflowTransitionRequest {
            target_status: KebutuhanBmnStatus::SubmitWilayah.to_code(),
            komentar: request.catatan_satker.clone(),
        };

        self.transition_satker_status(satker_id, transition_request, user_id, user_info)
            .await
    }

    /// Validator Wilayah action: forward to pusat or return to operator
    pub async fn validator_wilayah_action(
        &self,
        satker_id: Uuid,
        request: ValidatorWilayahActionRequest,
        user_id: Option<Uuid>,
        user_info: Option<UserInfo>,
    ) -> AppResult<PengajuanKebutuhanBmnSatker> {
        let current = self.repository.get_satker_by_id(satker_id).await?;

        if current.status != KebutuhanBmnStatus::SubmitWilayah {
            return Err(AppError::BadRequest(
                "Aksi Validator Wilayah hanya bisa dilakukan saat status Submit Wilayah"
                    .to_string(),
            ));
        }

        // Update validator wilayah info
        self.repository
            .update_satker_validator_wilayah(satker_id, user_id, request.catatan.clone())
            .await?;

        match request.aksi.as_str() {
            "forward" => {
                info!(
                    "Validator Wilayah forwarding {} to Validator Pusat",
                    satker_id
                );
                let transition_request = WorkflowTransitionRequest {
                    target_status: KebutuhanBmnStatus::SubmitPusat.to_code(),
                    komentar: request.catatan,
                };
                self.transition_satker_status(satker_id, transition_request, user_id, user_info)
                    .await
            }
            "return" => {
                if request.catatan.is_none() {
                    return Err(AppError::BadRequest(
                        "Catatan diperlukan saat mengembalikan ke Operator Satker".to_string(),
                    ));
                }
                info!(
                    "Validator Wilayah returning {} to Operator Satker",
                    satker_id
                );
                let transition_request = WorkflowTransitionRequest {
                    target_status: KebutuhanBmnStatus::RevisiSatker.to_code(),
                    komentar: request.catatan,
                };
                self.transition_satker_status(satker_id, transition_request, user_id, user_info)
                    .await
            }
            _ => Err(AppError::BadRequest(
                "Action harus 'forward' atau 'return'".to_string(),
            )),
        }
    }

    /// Validator Pusat makes final decision: approve or reject
    /// NO revision/return - only approve or reject with alasan
    pub async fn validator_pusat_keputusan(
        &self,
        satker_id: Uuid,
        request: ValidatorPusatKeputusanRequest,
        user_id: Option<Uuid>,
        user_info: Option<UserInfo>,
    ) -> AppResult<PengajuanKebutuhanBmnSatker> {
        let current = self.repository.get_satker_by_id(satker_id).await?;

        if !matches!(
            current.status,
            KebutuhanBmnStatus::SubmitPusat | KebutuhanBmnStatus::AnalisisKelayakan
        ) {
            return Err(AppError::BadRequest(
                "Keputusan Validator Pusat hanya bisa dibuat saat status Submit Pusat atau Analisis Kelayakan".to_string(),
            ));
        }

        let sync_risky = self.is_integrasi_sync_risky().await;
        let override_requested = request.override_darurat.unwrap_or(false);
        let override_reason = request
            .override_reason
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string);

        if sync_risky {
            if !override_requested {
                return Err(AppError::BadRequest(
                    "Keputusan Validator Pusat dikunci karena sinkronisasi data integrasi bermasalah. Gunakan override darurat terotorisasi.".to_string(),
                ));
            }

            if !Self::is_admin_user(&user_info) {
                return Err(AppError::Authorization(
                    "Override darurat hanya boleh dilakukan oleh admin".to_string(),
                ));
            }

            if override_reason.as_ref().map(|s| s.len()).unwrap_or(0) < 20 {
                return Err(AppError::BadRequest(
                    "Alasan override darurat wajib diisi minimal 20 karakter".to_string(),
                ));
            }
        }

        let keputusan_alasan = if sync_risky && override_requested {
            let base_reason = request
                .alasan
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string);
            let override_note =
                format!("[OVERRIDE DARURAT] {}", override_reason.unwrap_or_default());

            match base_reason {
                Some(reason) => Some(format!("{}\n\n{}", reason, override_note)),
                None => Some(override_note),
            }
        } else {
            request.alasan.clone()
        };

        // Update validator pusat info
        self.repository
            .update_satker_validator_pusat(
                satker_id,
                user_id,
                keputusan_alasan.clone(),
                request.is_approved,
            )
            .await?;

        let target_status = if request.is_approved {
            info!(
                "Validator Pusat approving kebutuhan BMN satker {}",
                satker_id
            );
            KebutuhanBmnStatus::Approved
        } else {
            info!(
                "Validator Pusat rejecting kebutuhan BMN satker {}",
                satker_id
            );
            KebutuhanBmnStatus::Rejected
        };

        let transition_request = WorkflowTransitionRequest {
            target_status: target_status.to_code(),
            komentar: keputusan_alasan,
        };

        self.transition_satker_status(satker_id, transition_request, user_id, user_info)
            .await
    }

    fn is_admin_user(user_info: &Option<UserInfo>) -> bool {
        user_info
            .as_ref()
            .and_then(|u| u.role.as_deref())
            .map(|role| {
                let normalized = role.to_ascii_lowercase();
                normalized == "admin"
                    || normalized == "super_admin"
                    || normalized == "administrator"
                    || normalized.starts_with("admin_")
            })
            .unwrap_or(false)
    }

    async fn is_integrasi_sync_risky(&self) -> bool {
        let Some(sync) = self.get_integrasi_sync_metadata().await else {
            return false;
        };

        let mysimkari_risky = sync.mysimkari.state.contains("FAILED")
            || sync
                .mysimkari
                .error_message
                .as_deref()
                .map(|e| !e.trim().is_empty())
                .unwrap_or(false);

        let siman_risky = sync.siman.state.contains("FAILED")
            || sync
                .siman
                .error_message
                .as_deref()
                .map(|e| !e.trim().is_empty())
                .unwrap_or(false);

        mysimkari_risky || siman_risky
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

        // V029 (Fase 1.6): Allowed-list BMN enforcement. Cek kode_barang
        // request masuk dlm whitelist Validator Pusat. Kosong / NULL =
        // legacy mode (semua boleh). Pengajuan_id resolve via satker.
        if let Some(kode) = &request.kode_barang {
            let allowed = self
                .repository
                .is_bmn_allowed_for_pengajuan(satker.pengajuan_id, kode)
                .await?;
            if !allowed {
                return Err(AppError::BadRequest(format!(
                    "Barang dgn kode_barang '{}' tidak diizinkan untuk pengajuan ini. Periksa daftar BMN yg ditetapkan Validator Pusat.",
                    kode
                )));
            }
        }

        info!("Creating barang for satker {}: {}", satker_id, request.nama);
        self.repository
            .create_barang(satker_id, request, user_id)
            .await
    }

    /// V029 (Fase 1.6): list allowed BMN utk pengajuan — dipakai FE
    /// dropdown saat Operator Satker input barang.
    pub async fn list_bmn_referensi(
        &self,
        pengajuan_id: Uuid,
    ) -> AppResult<Vec<PengajuanBmnReferensi>> {
        self.repository.list_bmn_referensi(pengajuan_id).await
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

        // V029 (#24): jika satker sudah melewati submit operator, baca snapshot
        // beku (di-freeze saat submit) alih-alih fetch SIMAN live — sehingga
        // Validator Wilayah & Pusat melihat angka yg PERSIS sama dgn operator.
        let snapshot = if satker.status.is_post_operator_submit() {
            self.repository
                .get_analisis_snapshot(satker_id)
                .await
                .ok()
                .flatten()
                .and_then(|v| serde_json::from_value::<AnalisisSnapshot>(v).ok())
        } else {
            None
        };

        let (barang_with_inventory, summary, is_snapshot, snapshot_at) = match snapshot {
            Some(snap) => {
                let at = snap.snapshot_at.clone();
                let (b, s) = Self::build_analisis_from_snapshot(barang_list, &snap);
                (b, s, true, Some(at))
            }
            None => {
                let (b, s, _) = self.compute_live_analisis(barang_list).await;
                (b, s, false, None)
            }
        };

        // Fetch pegawai data from MySIMKARI for final analysis by Validator Pusat
        let data_pegawai = self
            .get_mysimkari_pegawai_data(&satker.ms_satker_id)
            .await
            .ok();

        let integrasi_sync = self.get_integrasi_sync_metadata().await;

        Ok(AnalisisKelayakanResponse {
            satker,
            barang_list: barang_with_inventory,
            summary,
            data_pegawai,
            integrasi_sync,
            is_snapshot,
            snapshot_at,
        })
    }

    /// V029 (#24): hitung analisis kelayakan LIVE dari SIMAN per barang.
    /// Mengembalikan (barang+inventory, summary, snapshot-beku siap simpan).
    /// Dipakai saat operator masih mengedit (live) dan saat submit (untuk
    /// membekukan snapshot).
    async fn compute_live_analisis(
        &self,
        barang_list: Vec<PengajuanKebutuhanBmnBarang>,
    ) -> (
        Vec<BarangWithExistingInventory>,
        AnalisisSummary,
        AnalisisSnapshot,
    ) {
        let mut total_diminta: i64 = 0;
        let mut total_existing: i64 = 0;
        let mut barang_with_inventory = Vec::new();
        let mut snap_barang = Vec::new();

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

            snap_barang.push(AnalisisSnapshotBarang {
                barang_id: barang.id,
                existing_count,
                gap,
                recommendation: recommendation.clone(),
                existing_assets: existing_assets
                    .iter()
                    .map(|a| AnalisisSnapshotAsset {
                        no_aset: a.no_aset.clone(),
                        nama_aset: a.nama_aset.clone(),
                        kondisi: a.kondisi.clone(),
                        lokasi: a.lokasi.clone(),
                    })
                    .collect(),
            });

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

        let summary = AnalisisSummary {
            total_diminta,
            total_existing,
            total_gap,
            kelayakan_persen,
        };
        let snapshot = AnalisisSnapshot {
            snapshot_at: chrono::Utc::now().to_rfc3339(),
            summary: AnalisisSnapshotSummary {
                total_diminta,
                total_existing,
                total_gap,
                kelayakan_persen,
            },
            barang: snap_barang,
        };

        (barang_with_inventory, summary, snapshot)
    }

    /// V029 (#24): rekonstruksi analisis dari snapshot beku — fungsi MURNI
    /// (testable tanpa SIMAN/DB). Baris barang tetap dari DB (beku setelah
    /// submit), tetapi `existing_count`/`gap`/`recommendation`/`existing_assets`
    /// diambil dari snapshot agar konsisten dgn yg dilihat operator. Barang
    /// yg tak ada di snapshot (mis. ditambahkan setelah submit saat revisi)
    /// jatuh ke `existing_count` tersimpan sebagai fallback.
    fn build_analisis_from_snapshot(
        barang_list: Vec<PengajuanKebutuhanBmnBarang>,
        snapshot: &AnalisisSnapshot,
    ) -> (Vec<BarangWithExistingInventory>, AnalisisSummary) {
        let mut out = Vec::new();
        for barang in barang_list {
            match snapshot.barang.iter().find(|b| b.barang_id == barang.id) {
                Some(snap) => {
                    let existing_assets = snap
                        .existing_assets
                        .iter()
                        .map(|a| ExistingAssetInfo {
                            no_aset: a.no_aset.clone(),
                            nama_aset: a.nama_aset.clone(),
                            kondisi: a.kondisi.clone(),
                            lokasi: a.lokasi.clone(),
                        })
                        .collect();
                    out.push(BarangWithExistingInventory {
                        existing_assets,
                        gap: snap.gap,
                        recommendation: snap.recommendation.clone(),
                        barang,
                    });
                }
                None => {
                    let gap = barang.jumlah - barang.existing_count;
                    out.push(BarangWithExistingInventory {
                        existing_assets: vec![],
                        gap,
                        recommendation: String::new(),
                        barang,
                    });
                }
            }
        }

        let summary = AnalisisSummary {
            total_diminta: snapshot.summary.total_diminta,
            total_existing: snapshot.summary.total_existing,
            total_gap: snapshot.summary.total_gap,
            kelayakan_persen: snapshot.summary.kelayakan_persen,
        };
        (out, summary)
    }

    /// Fetch MySIMKARI pegawai data for satker analysis
    async fn get_mysimkari_pegawai_data(&self, satker_id: &str) -> AppResult<DataPegawaiRekap> {
        let integrasi_client = match &self.integrasi_client {
            Some(client) => client,
            None => {
                warn!(
                    "Integrasi client not configured, returning empty MySIMKARI rekap for satker {}",
                    satker_id
                );
                return Ok(DataPegawaiRekap {
                    total_pegawai: 0,
                    rekap_eselon: vec![],
                    rekap_non_eselon: vec![],
                });
            }
        };

        let mut current_page = 1;
        let per_page = 200;
        let mut all_items = Vec::new();
        let mut total_pegawai = 0_i64;

        loop {
            let response = integrasi_client
                .get_mysimkari_pegawai(satker_id, current_page, per_page)
                .await
                .map_err(|e| {
                    AppError::Internal(format!(
                        "Failed to fetch MySIMKARI pegawai data for satker {}: {}",
                        satker_id, e
                    ))
                })?;

            if total_pegawai == 0 {
                total_pegawai = response
                    .pagination
                    .as_ref()
                    .map(|p| p.total_items)
                    .unwrap_or(response.items.len() as i64);
            }

            let total_pages = response
                .pagination
                .as_ref()
                .map(|p| p.total_pages)
                .unwrap_or(current_page);

            if response.items.is_empty() {
                break;
            }

            all_items.extend(response.items);

            if current_page >= total_pages {
                break;
            }
            current_page += 1;
        }

        let mut eselon_counts: HashMap<String, i64> = HashMap::new();
        let mut non_eselon_counts: HashMap<(String, String, bool), i64> = HashMap::new();

        for pegawai in all_items {
            if let Some(tingkat_eselon) =
                Self::extract_eselon_level(&pegawai.jabatan, &pegawai.extra_fields)
            {
                *eselon_counts.entry(tingkat_eselon).or_insert(0) += 1;
                continue;
            }

            let golongan = if pegawai.golongan.trim().is_empty() {
                "Tidak Diketahui".to_string()
            } else {
                pegawai.golongan.clone()
            };

            let pangkat = if pegawai.pangkat.trim().is_empty() {
                "Tidak Diketahui".to_string()
            } else {
                pegawai.pangkat.clone()
            };

            let is_jaksa = pegawai.jabatan.to_lowercase().contains("jaksa");
            *non_eselon_counts
                .entry((golongan, pangkat, is_jaksa))
                .or_insert(0) += 1;
        }

        let mut rekap_eselon: Vec<RekapEselonItem> = eselon_counts
            .into_iter()
            .map(|(tingkat_eselon, jumlah)| RekapEselonItem {
                tingkat_eselon,
                jumlah,
            })
            .collect();
        rekap_eselon.sort_by(|a, b| a.tingkat_eselon.cmp(&b.tingkat_eselon));

        let mut rekap_non_eselon: Vec<RekapNonEselonItem> = non_eselon_counts
            .into_iter()
            .map(
                |((golongan, pangkat, is_jaksa), jumlah)| RekapNonEselonItem {
                    golongan,
                    pangkat,
                    is_jaksa,
                    jumlah,
                },
            )
            .collect();
        rekap_non_eselon.sort_by(|a, b| {
            a.golongan
                .cmp(&b.golongan)
                .then(a.pangkat.cmp(&b.pangkat))
                .then(a.is_jaksa.cmp(&b.is_jaksa))
        });

        Ok(DataPegawaiRekap {
            total_pegawai,
            rekap_eselon,
            rekap_non_eselon,
        })
    }

    fn extract_eselon_level(
        jabatan: &str,
        extra_fields: &HashMap<String, String>,
    ) -> Option<String> {
        let from_extra = extra_fields
            .get("tingkat_eselon")
            .or_else(|| extra_fields.get("eselon"))
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        if from_extra.is_some() {
            return from_extra;
        }

        let jabatan_lc = jabatan.to_lowercase();
        let patterns = [
            ("eselon i", "Eselon I"),
            ("eselon ii", "Eselon II"),
            ("eselon iii", "Eselon III"),
            ("eselon iv", "Eselon IV"),
            ("eselon v", "Eselon V"),
        ];

        for (needle, label) in patterns {
            if jabatan_lc.contains(needle) {
                return Some(label.to_string());
            }
        }

        None
    }

    async fn get_integrasi_sync_metadata(&self) -> Option<IntegrasiSyncMetadata> {
        let integrasi_client = self.integrasi_client.as_ref()?;

        let mysimkari =
            Self::fetch_sync_status(integrasi_client, DataSource::Mysimkari, "mysimkari").await;
        let siman = Self::fetch_sync_status(integrasi_client, DataSource::Siman, "siman").await;

        Some(IntegrasiSyncMetadata { mysimkari, siman })
    }

    async fn fetch_sync_status(
        integrasi_client: &IntegrasiClient,
        source: DataSource,
        source_name: &str,
    ) -> IntegrasiSyncStatus {
        match integrasi_client.get_sync_status(source).await {
            Ok(status) => {
                let state = SyncState::try_from(status.state)
                    .map(|s| s.as_str_name().to_string())
                    .unwrap_or_else(|_| "SYNC_STATE_UNSPECIFIED".to_string());

                IntegrasiSyncStatus {
                    source: source_name.to_string(),
                    state,
                    last_sync_at: if status.last_sync_at.trim().is_empty() {
                        None
                    } else {
                        Some(status.last_sync_at)
                    },
                    next_sync_at: if status.next_sync_at.trim().is_empty() {
                        None
                    } else {
                        Some(status.next_sync_at)
                    },
                    records_synced: status.records_synced,
                    error_message: if status.error_message.trim().is_empty() {
                        None
                    } else {
                        Some(status.error_message)
                    },
                }
            }
            Err(e) => {
                // NOTE: A failed gRPC probe (network timeout, service restart, DNS)
                // must NOT block Validator Pusat business decisions. We only set
                // error_message when the sync *itself* reports a failure (handled
                // in the Ok branch above). Probe failures are logged but treated
                // as "unknown" rather than "risky".
                warn!(
                    "Failed to fetch sync status for {} from Integrasi (probe failure, not blocking decisions): {}",
                    source_name, e
                );
                IntegrasiSyncStatus {
                    source: source_name.to_string(),
                    state: "SYNC_STATE_UNSPECIFIED".to_string(),
                    last_sync_at: None,
                    next_sync_at: None,
                    records_synced: 0,
                    error_message: None,
                }
            }
        }
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
        client_ip: String,
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
            let result = match self
                .process_single_approval(
                    *kebutuhan_id,
                    &request.komentar,
                    user_id,
                    user_info.clone(),
                    client_ip.clone(),
                )
                .await
            {
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
            batch_id,
            successful_count,
            request.kebutuhan_ids.len()
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
        client_ip: String,
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
            let result = match self
                .process_single_rejection(
                    *kebutuhan_id,
                    &request.komentar,
                    user_id,
                    user_info.clone(),
                    client_ip.clone(),
                )
                .await
            {
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
            batch_id,
            successful_count,
            request.kebutuhan_ids.len()
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
        client_ip: String,
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
            let result = match self
                .process_single_status_update(
                    *kebutuhan_id,
                    request.target_status,
                    &request.komentar,
                    user_id,
                    user_info.clone(),
                    client_ip.clone(),
                )
                .await
            {
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
                    warn!(
                        "Failed to update status for kebutuhan {}: {}",
                        kebutuhan_id, e
                    );
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
            batch_id,
            successful_count,
            request.kebutuhan_ids.len()
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
        client_ip: String,
    ) -> AppResult<()> {
        let transition_request = WorkflowTransitionRequest {
            target_status: KebutuhanBmnStatus::Approved.to_code(),
            komentar: komentar.clone(),
        };

        self.transition_pengajuan_status(
            kebutuhan_id,
            transition_request,
            user_id,
            user_info,
            client_ip,
        )
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
        client_ip: String,
    ) -> AppResult<()> {
        let transition_request = WorkflowTransitionRequest {
            target_status: KebutuhanBmnStatus::Rejected.to_code(),
            komentar: Some(komentar.to_string()),
        };

        self.transition_pengajuan_status(
            kebutuhan_id,
            transition_request,
            user_id,
            user_info,
            client_ip,
        )
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
        client_ip: String,
    ) -> AppResult<()> {
        let transition_request = WorkflowTransitionRequest {
            target_status,
            komentar: komentar.clone(),
        };

        self.transition_pengajuan_status(
            kebutuhan_id,
            transition_request,
            user_id,
            user_info,
            client_ip,
        )
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
            wilayah_id: None,
            satker_ids: vec![],
            asset_types: vec![],
            bmn_referensi_diizinkan: vec![],
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

    // ========================================================================
    // V029 (#24) — Snapshot analisis kelayakan
    // ========================================================================

    fn fixture_barang(
        id: Uuid,
        nama: &str,
        jumlah: i32,
        existing_count: i32,
    ) -> PengajuanKebutuhanBmnBarang {
        let now = chrono::Utc::now();
        PengajuanKebutuhanBmnBarang {
            id,
            pengajuan_satker_id: Uuid::new_v4(),
            nama: nama.to_string(),
            kode_barang: None,
            jumlah,
            satuan: "Unit".to_string(),
            jml_setuju: 0,
            alasan: None,
            keterangan: None,
            prioritas: 0,
            skor: 0.0,
            file_pendukung: serde_json::json!([]),
            existing_count,
            existing_condition: None,
            created_by: None,
            updated_by: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Snapshot harus round-trip lewat JSON (kolom JSONB) tanpa kehilangan
    /// data — properti inti agar Wilayah/Pusat membaca angka yg sama.
    #[test]
    fn analisis_snapshot_round_trips_through_json() {
        let snap = AnalisisSnapshot {
            snapshot_at: "2026-06-02T10:00:00Z".to_string(),
            summary: AnalisisSnapshotSummary {
                total_diminta: 10,
                total_existing: 4,
                total_gap: 6,
                kelayakan_persen: 40.0,
            },
            barang: vec![AnalisisSnapshotBarang {
                barang_id: Uuid::new_v4(),
                existing_count: 4,
                gap: 6,
                recommendation: "Pengadaan disarankan".to_string(),
                existing_assets: vec![AnalisisSnapshotAsset {
                    no_aset: "A1".to_string(),
                    nama_aset: "Kendaraan".to_string(),
                    kondisi: "Baik".to_string(),
                    lokasi: Some("Gudang".to_string()),
                }],
            }],
        };
        let v = serde_json::to_value(&snap).unwrap();
        let back: AnalisisSnapshot = serde_json::from_value(v).unwrap();
        assert_eq!(back.snapshot_at, snap.snapshot_at);
        assert_eq!(back.summary.total_gap, 6);
        assert_eq!(back.barang.len(), 1);
        assert_eq!(back.barang[0].existing_assets[0].kondisi, "Baik");
    }

    /// Rebuild dari snapshot harus memakai angka SNAPSHOT (existing/gap),
    /// BUKAN `existing_count` live di baris barang — itulah inti konsistensi.
    #[test]
    fn build_from_snapshot_prefers_snapshot_over_live_existing() {
        let id = Uuid::new_v4();
        // Baris barang DB punya existing_count=99 (mis. SIMAN berubah setelah submit).
        let barang = vec![fixture_barang(id, "Laptop", 10, 99)];
        let snap = AnalisisSnapshot {
            snapshot_at: "2026-06-02T10:00:00Z".to_string(),
            summary: AnalisisSnapshotSummary {
                total_diminta: 10,
                total_existing: 3,
                total_gap: 7,
                kelayakan_persen: 30.0,
            },
            barang: vec![AnalisisSnapshotBarang {
                barang_id: id,
                existing_count: 3,
                gap: 7,
                recommendation: "Beku".to_string(),
                existing_assets: vec![],
            }],
        };
        let (out, summary) = KebutuhanBmnService::build_analisis_from_snapshot(barang, &snap);
        assert_eq!(out.len(), 1);
        // gap dari snapshot (7), bukan jumlah - live existing_count (10-99).
        assert_eq!(out[0].gap, 7);
        assert_eq!(out[0].recommendation, "Beku");
        assert_eq!(summary.total_existing, 3);
        assert_eq!(summary.total_gap, 7);
    }

    /// Barang yg tak ada di snapshot (ditambah saat revisi) jatuh ke fallback
    /// stored existing_count, bukan panik / hilang.
    #[test]
    fn build_from_snapshot_falls_back_for_unknown_barang() {
        let snapshot_id = Uuid::new_v4();
        let extra_id = Uuid::new_v4();
        let barang = vec![
            fixture_barang(snapshot_id, "A", 5, 2),
            fixture_barang(extra_id, "B", 8, 3), // tidak ada di snapshot
        ];
        let snap = AnalisisSnapshot {
            snapshot_at: "2026-06-02T10:00:00Z".to_string(),
            summary: AnalisisSnapshotSummary {
                total_diminta: 5,
                total_existing: 2,
                total_gap: 3,
                kelayakan_persen: 40.0,
            },
            barang: vec![AnalisisSnapshotBarang {
                barang_id: snapshot_id,
                existing_count: 2,
                gap: 3,
                recommendation: "Snap".to_string(),
                existing_assets: vec![],
            }],
        };
        let (out, _) = KebutuhanBmnService::build_analisis_from_snapshot(barang, &snap);
        assert_eq!(out.len(), 2);
        let b = out.iter().find(|x| x.barang.id == extra_id).unwrap();
        // fallback: jumlah(8) - stored existing_count(3) = 5
        assert_eq!(b.gap, 5);
    }

    /// Gate snapshot: hanya state PASCA submit operator yg memakai snapshot.
    #[test]
    fn post_submit_gate_matches_expected_states() {
        use KebutuhanBmnStatus::*;
        // Masih edit → live.
        assert!(!Draft.is_post_operator_submit());
        assert!(!InputBarang.is_post_operator_submit());
        assert!(!RevisiSatker.is_post_operator_submit());
        // Sudah submit → snapshot.
        assert!(SubmitWilayah.is_post_operator_submit());
        assert!(SubmitPusat.is_post_operator_submit());
        assert!(AnalisisKelayakan.is_post_operator_submit());
        assert!(Approved.is_post_operator_submit());
        assert!(Completed.is_post_operator_submit());
    }
}
