use super::PemakaianBmnService;
use crate::pemakaian_bmn::models::*;
use crate::shared::error::{AppError, AppResult};
use crate::workflow::engine::TransitionRequest;
use tracing::info;
use uuid::Uuid;

impl PemakaianBmnService {
    /// Get permit with allowed transitions
    ///
    /// Requirements: REQ-P001
    pub async fn get_permit_detail(&self, id: Uuid) -> AppResult<IzinPemakaianDetailResponse> {
        let izin = self.repository.get_by_id(id).await?;

        // Get allowed transitions from workflow engine
        let current_status =
            PemakaianBmnStatus::from_state_name(&izin.status).unwrap_or(PemakaianBmnStatus::Draft);
        let allowed_transitions = self.get_allowed_transitions(current_status);

        // Calculate expiry info
        let (days_until_expiry, is_expiring_soon) = {
            let today = chrono::Utc::now().date_naive();
            let days = (izin.tanggal_selesai - today).num_days();
            let expiring = (0..=30).contains(&days);
            (Some(days), expiring)
        };

        Ok(IzinPemakaianDetailResponse {
            izin,
            allowed_transitions,
            days_until_expiry,
            is_expiring_soon,
            can_generate_konsep: false,
            can_upload_signed_pdf: false,
        })
    }

    /// Transition permit to a new status using workflow engine
    ///
    /// This method uses the centralized workflow engine for:
    /// - Transition validation
    /// - Role-based authorization
    /// - Audit logging
    /// - Activity tracking
    ///
    /// Requirements: REQ-P004, REQ-W004, REQ-W005
    /// Caller (handler) supplies the originating client IP so the workflow
    /// activity log gets a real address instead of a placeholder. Use the
    /// `ClientIp` axum extractor on the handler side.
    pub async fn transition_permit_status(
        &self,
        id: Uuid,
        request: WorkflowTransitionRequest,
        user_id: Uuid,
        user_role: String,
        client_ip: String,
    ) -> AppResult<IzinPemakaianBmn> {
        let current = self.repository.get_by_id(id).await?;
        let current_status =
            PemakaianBmnStatus::from_state_name(&current.status).ok_or_else(|| {
                AppError::BadRequest(format!("Unknown current status: {}", current.status))
            })?;
        let target_status = PemakaianBmnStatus::from_state_name(&request.target_status)
            .ok_or_else(|| {
                AppError::BadRequest(format!("Invalid target status: {}", request.target_status))
            })?;

        // Convert status codes to state names for workflow engine
        let from_state = current_status.to_state_name();
        let to_state = target_status.to_state_name();

        // Validate transition using workflow engine
        if !self
            .workflow_engine
            .config()
            .is_valid_transition(from_state, to_state)
        {
            return Err(AppError::BadRequest(format!(
                "Cannot transition from {} to {}",
                current_status.label(),
                target_status.label()
            )));
        }

        // Some transitions require comment
        let requires_comment = matches!(
            target_status,
            PemakaianBmnStatus::Rejected | PemakaianBmnStatus::Revoked
        );

        if requires_comment && request.catatan.is_none() {
            return Err(AppError::BadRequest(
                "Catatan diperlukan untuk transisi ini".to_string(),
            ));
        }

        info!(
            "Transitioning permit {} from {} to {} via workflow engine",
            id,
            current_status.label(),
            target_status.label()
        );

        // Use workflow engine for transition
        let transition_request = TransitionRequest {
            entity_id: id,
            from_state: from_state.to_string(),
            to_state: to_state.to_string(),
            user_id,
            user_role,
            catatan: request.catatan.clone(),
            ip_address: client_ip.clone(),
        };

        // Execute transition through workflow engine
        let _transition_result = self
            .workflow_engine
            .transition(transition_request)
            .await
            .map_err(|e| AppError::Internal(format!("Workflow transition failed: {}", e)))?;

        // The engine only writes the canonical `status`; re-sync the
        // denormalized `status_kode` so the internal-satker workflow methods
        // (which gate on status_kode) see the new state.
        self.repository
            .sync_status_kode(id, target_status.to_code())
            .await?;

        // Get updated permit
        let updated = self.repository.get_by_id(id).await?;

        // Special handling for APPROVED -> ACTIVE transition
        if target_status == PemakaianBmnStatus::Approved {
            info!("Permit {} approved, can now be activated", id);
        }

        // Special handling for ACTIVE status
        if target_status == PemakaianBmnStatus::Active {
            info!("Permit {} is now active", id);
            // TODO(pemakaian-expiry-scheduler): hand this permit to the
            // `PemakaianBmnScheduler` (already running, see main.rs) so
            // `send_expiry_reminder` / `send_expiry_notification` fire at
            // H-30 / H-7 / H-0. Currently the scheduler scans on its own
            // cron, so this enqueue is an optimisation, not a correctness
            // gap.
        }

        Ok(updated)
    }

    /// Get allowed next states for a permit using workflow engine
    ///
    /// Requirements: REQ-P004
    pub fn get_allowed_transitions(
        &self,
        current_status: PemakaianBmnStatus,
    ) -> Vec<WorkflowTransitionInfo> {
        let current_state = current_status.to_state_name();
        let next_states = self.workflow_engine.get_next_states(current_state);

        next_states
            .into_iter()
            .filter_map(|state_name| {
                let status = PemakaianBmnStatus::from_state_name(&state_name)?;
                // A move nothing can perform is not offered at all. `label()`
                // would still render, but there would be no action behind the
                // button — the failure mode this whole path exists to avoid.
                let action = crate::shared::policy::PemakaianBmnAction::for_transition(
                    current_state,
                    status.to_state_name(),
                )?;
                Some(WorkflowTransitionInfo {
                    status: status.to_state_name().to_string(),
                    label: status.label().to_string(),
                    action_label: action.action_label().to_string(),
                    requires_comment: matches!(
                        status,
                        PemakaianBmnStatus::Rejected | PemakaianBmnStatus::Revoked
                    ),
                })
            })
            .collect()
    }
}
