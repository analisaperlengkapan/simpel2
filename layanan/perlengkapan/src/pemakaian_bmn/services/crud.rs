use super::PemakaianBmnService;
use crate::pemakaian_bmn::models::*;
use crate::shared::error::{AppError, AppResult};
use tracing::info;
use uuid::Uuid;
use validator::Validate;

impl PemakaianBmnService {
    /// Update a permit (only in DRAFT status)
    ///
    /// Requirements: REQ-P001
    pub async fn update_permit(
        &self,
        id: Uuid,
        request: UpdateIzinPemakaianRequest,
        user_id: Uuid,
        user_nama: String,
    ) -> AppResult<IzinPemakaianBmn> {
        // Validate request
        request
            .validate()
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        // Validate date range if both dates provided
        if let (Some(start), Some(end)) = (request.tanggal_mulai, request.tanggal_selesai)
            && end <= start
        {
            return Err(AppError::BadRequest(
                "Tanggal selesai harus lebih besar dari tanggal mulai".to_string(),
            ));
        }

        info!("Updating permit {} by user {}", id, user_id);

        let permit = self
            .repository
            .update(id, request, user_id, user_nama)
            .await?;

        Ok(permit)
    }

    /// List permits with pagination and filters
    ///
    /// Requirements: REQ-P001, REQ-P011
    pub async fn list_permits(
        &self,
        query: ListPermitsQuery,
    ) -> AppResult<PaginatedPermitsResponse> {
        self.repository.list(query).await
    }

    // ========================================================================
    // V035 (Fase 1.5): Internal-satker 3-step approval orchestration.
    //
    // Workflow: Operator → Validator Satker → Approver Satker.
    // - `validator_satker_forward`  : SUBMITTED → SUBMITTED_APPROVER_SATKER
    // - `validator_satker_return`   : SUBMITTED → REVISI_OPERATOR
    // - `approver_satker_approve`   : SUBMITTED_APPROVER_SATKER → APPROVED (+ auto-activate)
    // - `approver_satker_return`    : SUBMITTED_APPROVER_SATKER → REVISI_OPERATOR
    // - `operator_resubmit`         : REVISI_OPERATOR → SUBMITTED
    //
    // Setelah APPROVED, service ini otomatis memanggil `activate_permit`
    // (yg melakukan advisory-lock + generate `nomor_izin` + render SK 2-hal)
    // — agar approver hanya menekan satu tombol.
    // ========================================================================
}
