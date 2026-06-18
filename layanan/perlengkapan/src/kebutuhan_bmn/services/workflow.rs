use super::KebutuhanBmnService;
use crate::kebutuhan_bmn::models::*;
use crate::kebutuhan_bmn::repository::{KebutuhanBmnRepository, UserInfo};
use crate::shared::error::{AppError, AppResult};
use crate::workflow::engine::TransitionRequest;
use tracing::{info, warn};
use uuid::Uuid;

impl KebutuhanBmnService {
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
        user_role: String,
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
            user_role: user_role.clone(),
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
        user_role: String,
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

        // Execute transition through workflow engine
        let _transition_result = self
            .workflow_engine
            .transition(TransitionRequest {
                entity_id: satker_id,
                from_state: from_state.to_string(),
                to_state: to_state.to_string(),
                user_id: user_id.unwrap_or_else(Uuid::nil),
                user_role,
                catatan: request.komentar.clone(),
                ip_address: "internal".to_string(),
            })
            .await
            .map_err(|e| AppError::Internal(format!("Workflow transition failed: {}", e)))?;

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
        user_role: String,
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

        self.transition_satker_status(
            satker_id,
            transition_request,
            user_id,
            user_info,
            user_role,
        )
        .await
    }

    /// Validator Wilayah action: forward to pusat or return to operator
    pub async fn validator_wilayah_action(
        &self,
        satker_id: Uuid,
        request: ValidatorWilayahActionRequest,
        user_id: Option<Uuid>,
        user_info: Option<UserInfo>,
        user_role: String,
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
                self.transition_satker_status(
                    satker_id,
                    transition_request,
                    user_id,
                    user_info,
                    user_role,
                )
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
                self.transition_satker_status(
                    satker_id,
                    transition_request,
                    user_id,
                    user_info,
                    user_role,
                )
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
        user_role: String,
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

        // Honor the config's mandatory feasibility-analysis gate
        // (SUBMIT_PUSAT → ANALISIS_KELAYAKAN → decision). A keputusan issued while
        // still in SubmitPusat first advances through AnalisisKelayakan so the
        // analysis step is recorded in the audit trail before the final decision,
        // rather than attempting the (intentionally) invalid SubmitPusat→Approved
        // transition directly.
        if current.status == KebutuhanBmnStatus::SubmitPusat {
            self.transition_satker_status(
                satker_id,
                WorkflowTransitionRequest {
                    target_status: KebutuhanBmnStatus::AnalisisKelayakan.to_code(),
                    komentar: keputusan_alasan.clone(),
                },
                user_id,
                user_info.clone(),
                user_role.clone(),
            )
            .await?;
        }

        let transition_request = WorkflowTransitionRequest {
            target_status: target_status.to_code(),
            komentar: keputusan_alasan,
        };

        self.transition_satker_status(
            satker_id,
            transition_request,
            user_id,
            user_info,
            user_role,
        )
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
}
