use super::PemakaianBmnService;
use crate::pemakaian_bmn::models::*;
use crate::shared::error::{AppError, AppResult};
use crate::shared::satker_scope::SatkerScope;
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

        // Same rules as create, through the same function. `All` here because
        // the caller's scope was applied by the handler that reached this, and
        // the repository re-checks DRAFT status before writing anything.
        if let Some(items) = request.items.as_deref() {
            let current = self.repository.get_by_id(id, &SatkerScope::All).await?;
            let satker = current.satker_code.clone().unwrap_or_default();
            let mulai = request.tanggal_mulai.unwrap_or(current.tanggal_mulai);
            let selesai = request.tanggal_selesai.unwrap_or(current.tanggal_selesai);
            // `Some(id)`: the permit's own rows must not be reported as
            // conflicting with themselves.
            self.validate_items(items, mulai, selesai, &satker, Some(id))
                .await?;
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
        scope: &crate::shared::satker_scope::SatkerScope,
    ) -> AppResult<PaginatedPermitsResponse> {
        self.repository.list(query, scope).await
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
