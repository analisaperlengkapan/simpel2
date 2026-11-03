use super::PemakaianBmnService;
use crate::pemakaian_bmn::models::*;
use crate::shared::error::{AppError, AppResult};
use crate::shared::satker_scope::SatkerScope;
use crate::workflow::engine::TransitionRequest;
use tracing::{info, warn};
use uuid::Uuid;

/// How close to its end an ACTIVE permit must be before the detail page warns.
/// The warning text states the actual number of days, so this is the only place
/// the window is defined.
pub(crate) const EXPIRING_SOON_DAYS: i64 = 30;

/// Days left on a permit and whether to warn that it is nearly over.
///
/// Only a permit that is IN FORCE has a countdown. This used to count down from
/// `tanggal_selesai` for every status, so a REVOKED permit — withdrawn the day it
/// was superseded — still read "59 hari lagi", and a DRAFT whose dates were
/// merely requested read as if it were running. `(None, false)` for anything
/// else; the page renders that as "—".
///
/// A permit that is ACTIVE but already past its end date keeps a negative count
/// ("Lewat N hari"): the scheduler that flips it to EXPIRED has not run yet, and
/// the page should say so rather than go blank.
pub(crate) fn expiry_info(
    status: PemakaianBmnStatus,
    tanggal_selesai: chrono::NaiveDate,
    today: chrono::NaiveDate,
) -> (Option<i64>, bool) {
    if status != PemakaianBmnStatus::Active {
        return (None, false);
    }
    let days = (tanggal_selesai - today).num_days();
    (Some(days), (0..=EXPIRING_SOON_DAYS).contains(&days))
}

impl PemakaianBmnService {
    /// Get permit with allowed transitions
    ///
    /// Requirements: REQ-P001
    pub async fn get_permit_detail(
        &self,
        id: Uuid,
        scope: &SatkerScope,
    ) -> AppResult<IzinPemakaianDetailResponse> {
        // The LIST behind this has been scoped since #66; the detail was not.
        // Measured on staging, one satker's operator read another satker's
        // permit in full — the employee's name, the asset, the dates.
        let izin = self.repository.get_by_id(id, scope).await?;

        // Get allowed transitions from workflow engine
        let current_status =
            PemakaianBmnStatus::from_state_name(&izin.status).unwrap_or(PemakaianBmnStatus::Draft);
        let allowed_transitions = self.get_allowed_transitions(current_status);

        let (days_until_expiry, is_expiring_soon) = expiry_info(
            current_status,
            izin.tanggal_selesai,
            chrono::Utc::now().date_naive(),
        );

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
        scope: &SatkerScope,
    ) -> AppResult<IzinPemakaianBmn> {
        let current = self.repository.get_by_id(id, scope).await?;
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

        // Two renewals of one permit cannot both go live: the first replaces
        // the permit it renews, and the second would then be "renewing" a
        // permit that is already gone. Refused here with a sentence the caller
        // can act on; the trigger raises the same condition for the concurrent
        // case this read cannot see, and rolls the activation back.
        if target_status == PemakaianBmnStatus::Active
            && current.is_renewal
            && let Some(sebelumnya) = current.previous_permit_id
            && let Some(pengganti) = self.repository.superseded_by(sebelumnya).await?
            && pengganti != id
        {
            return Err(AppError::Conflict(format!(
                "Izin yang diperpanjang sudah digantikan oleh izin lain ({pengganti}); \
                 perpanjangan ini tidak dapat diaktifkan"
            )));
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
        let updated = self.repository.get_by_id(id, scope).await?;

        // Special handling for APPROVED -> ACTIVE transition
        if target_status == PemakaianBmnStatus::Approved {
            info!("Permit {} approved, can now be activated", id);
        }

        // Special handling for ACTIVE status
        if target_status == PemakaianBmnStatus::Active {
            info!("Permit {} is now active", id);

            // A renewal takes effect the moment it is active, and the permit
            // it renews is withdrawn at the same instant — otherwise both are
            // ACTIVE over the same assets and the conflict check reports the
            // old one against its own successor.
            //
            // Nothing to do here: `trg_izin_pemakaian_supersede` (V014) revokes
            // the predecessor in the same statement that wrote ACTIVE. It used
            // to be a second UPDATE issued from this function, after the
            // engine had already committed — so a failure between the two left
            // BOTH permits ACTIVE while the caller was told it had failed. The
            // engine owns its own connection, so no caller can join its
            // transaction; the trigger is the only place the two writes can be
            // one. It also covers every path to ACTIVE, including the audited
            // break-glass override, which comes through this function too.
            if updated.is_renewal
                && let Some(sebelumnya) = updated.previous_permit_id
            {
                info!(
                    "Renewal {} is active; permit {} withdrawn with it",
                    id, sebelumnya
                );
                // WHO is accountable. The trigger withdrew the permit as the
                // system's automatic consequence of the renewal and cannot know
                // the human behind the activation (the engine's connection
                // carries no actor), so `revoked_by` is still empty. Name them
                // now, as the previous path did. This is attribution only: the
                // withdrawal itself is already committed and atomic, and
                // failing the request here would tell the client an activation
                // failed that did not. So a miss is logged, not returned. An
                // activation that bypasses this function (a bare UPDATE) leaves
                // the field empty, which honestly reads "unknown", rather than
                // guessing from `updated_by`.
                if let Err(e) = self
                    .repository
                    .attribute_supersession(sebelumnya, id, user_id)
                    .await
                {
                    warn!(
                        "Permit {} was withdrawn by renewal {} but recording who did it failed: {}",
                        sebelumnya, id, e
                    );
                }
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn an_active_permit_counts_down_and_warns_inside_the_window() {
        assert_eq!(
            expiry_info(PemakaianBmnStatus::Active, d("2026-12-01"), d("2026-11-01")),
            (Some(30), true)
        );
        assert_eq!(
            expiry_info(PemakaianBmnStatus::Active, d("2026-12-02"), d("2026-11-01")),
            (Some(31), false),
            "one day outside the window must not warn"
        );
        assert_eq!(
            expiry_info(PemakaianBmnStatus::Active, d("2026-11-01"), d("2026-11-01")),
            (Some(0), true),
            "the last day is still a warning"
        );
    }

    #[test]
    fn an_active_permit_past_its_end_date_counts_negative_without_warning() {
        assert_eq!(
            expiry_info(PemakaianBmnStatus::Active, d("2026-10-30"), d("2026-11-01")),
            (Some(-2), false)
        );
    }

    /// The defect: every status counted down from its end date, so a withdrawn
    /// permit claimed it had 59 days left.
    #[test]
    fn a_permit_that_is_not_in_force_has_no_countdown() {
        for status in [
            PemakaianBmnStatus::Draft,
            PemakaianBmnStatus::Submitted,
            PemakaianBmnStatus::Revoked,
            PemakaianBmnStatus::Expired,
        ] {
            assert_eq!(
                expiry_info(status, d("2026-12-30"), d("2026-11-01")),
                (None, false),
                "{status:?} must not show a countdown"
            );
        }
    }
}
