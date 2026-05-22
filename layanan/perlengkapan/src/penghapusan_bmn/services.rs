// ============================================================================
// Penghapusan BMN Services
// Description: Business logic for Usulan SK Penghapusan BMN workflow
// Requirements: REQ-W001, REQ-W004, REQ-D002, REQ-N001
//
// Flow:
//   Operator Satker: Draft → SubmitWilayah (with lampiran)
//   Validator Wilayah: SubmitWilayah → SubmitPusat (forward) or ReturnedToOperator (return)
//   Operator Satker: ReturnedToOperator → SubmitWilayah (re-submit)
//   Validator Pusat: SubmitPusat → VerifikasiPusat → KonsepSKGenerated (generate DOCX)
//   Validator Pusat: KonsepSKGenerated → SKSigned (upload signed PDF)
//   System: SKSigned → Completed
// ============================================================================

use super::models::*;
use super::repository::PenghapusanBmnRepository;
use crate::errors::AppResult;
use crate::workflow::engine::{TransitionRequest, WorkflowEngine};
use deadpool_postgres::Pool;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

pub struct PenghapusanBmnService {
    repository: PenghapusanBmnRepository,
    workflow_engine: Arc<WorkflowEngine>,
    docs: Option<Arc<dyn lib_perlengkapan::contracts::DocumentGenerator>>,
}

impl PenghapusanBmnService {
    pub fn new(pool: Pool, workflow_engine: Arc<WorkflowEngine>) -> Self {
        Self {
            repository: PenghapusanBmnRepository::new(pool),
            workflow_engine,
            docs: None,
        }
    }

    /// Inject the document generator (used by the konsep SK dual-format flow).
    pub fn with_document_generator(
        mut self,
        docs: Arc<dyn lib_perlengkapan::contracts::DocumentGenerator>,
    ) -> Self {
        self.docs = Some(docs);
        self
    }

    /// Create a new Usulan SK Penghapusan BMN (by Operator Satker)
    pub async fn create(
        &self,
        request: CreatePenghapusanBmnRequest,
        created_by: Uuid,
    ) -> AppResult<PenghapusanBmn> {
        info!(
            "Creating Usulan SK Penghapusan BMN for asset: {}",
            request.nama_barang
        );
        self.repository.create(request, created_by).await
    }

    /// Get penghapusan BMN by ID
    pub async fn get_by_id(&self, id: Uuid) -> AppResult<PenghapusanBmn> {
        self.repository.get_by_id(id).await
    }

    /// Get penghapusan BMN detail with allowed transitions
    pub async fn get_detail(&self, id: Uuid) -> AppResult<PenghapusanBmnDetailResponse> {
        let penghapusan = self.repository.get_by_id(id).await?;
        let status = PenghapusanBmnStatus::from_state_name(&penghapusan.status).unwrap_or_default();

        let transitions: Vec<PenghapusanTransitionInfo> = status
            .allowed_transitions()
            .into_iter()
            .map(|s| PenghapusanTransitionInfo {
                status_kode: s.to_code(),
                status_nama: s.label().to_string(),
                requires_comment: matches!(
                    s,
                    PenghapusanBmnStatus::ReturnedToOperator | PenghapusanBmnStatus::Rejected
                ),
            })
            .collect();

        let can_generate_sk = matches!(status, PenghapusanBmnStatus::VerifikasiPusat);
        let can_upload_signed_sk = matches!(status, PenghapusanBmnStatus::KonsepSKGenerated);

        Ok(PenghapusanBmnDetailResponse {
            penghapusan,
            allowed_transitions: transitions,
            can_generate_sk,
            can_upload_signed_sk,
        })
    }

    /// List penghapusan BMN with filters and pagination
    pub async fn list(
        &self,
        filters: PenghapusanBmnFilters,
        page: i32,
        per_page: i32,
    ) -> AppResult<(Vec<PenghapusanBmn>, i64)> {
        self.repository.list(filters, page, per_page).await
    }

    /// Update penghapusan BMN (only in Draft/ReturnedToOperator status)
    pub async fn update(
        &self,
        id: Uuid,
        request: UpdatePenghapusanBmnRequest,
    ) -> AppResult<PenghapusanBmn> {
        let current = self.repository.get_by_id(id).await?;
        let status = PenghapusanBmnStatus::from_state_name(&current.status).unwrap_or_default();

        if !matches!(
            status,
            PenghapusanBmnStatus::Draft | PenghapusanBmnStatus::ReturnedToOperator
        ) {
            return Err(crate::errors::AppError::WorkflowError(
                "Hanya bisa diubah saat status Draft atau Dikembalikan ke Operator".into(),
            ));
        }

        self.repository.update(id, request).await
    }

    /// Delete penghapusan BMN (only in Draft status)
    pub async fn delete(&self, id: Uuid) -> AppResult<()> {
        let current = self.repository.get_by_id(id).await?;
        let status = PenghapusanBmnStatus::from_state_name(&current.status).unwrap_or_default();

        if !matches!(status, PenghapusanBmnStatus::Draft) {
            return Err(crate::errors::AppError::WorkflowError(
                "Hanya bisa dihapus saat status Draft".into(),
            ));
        }

        self.repository.delete(id).await
    }

    /// Operator Satker submits to Validator Wilayah
    pub async fn submit_to_wilayah(
        &self,
        id: Uuid,
        user_id: Uuid,
        catatan: Option<String>,
    ) -> AppResult<PenghapusanBmn> {
        self.transition(
            id,
            PenghapusanBmnStatus::SubmitWilayah
                .to_state_name()
                .to_string(),
            user_id,
            catatan,
            "submit_to_wilayah".to_string(),
        )
        .await
    }

    /// Validator Wilayah forwards to Validator Pusat
    pub async fn forward_to_pusat(
        &self,
        id: Uuid,
        validator_id: Uuid,
        catatan: Option<String>,
    ) -> AppResult<PenghapusanBmn> {
        // Update validator wilayah info
        self.repository
            .update_validator_wilayah(id, validator_id, catatan.clone())
            .await?;

        self.transition(
            id,
            PenghapusanBmnStatus::SubmitPusat
                .to_state_name()
                .to_string(),
            validator_id,
            catatan,
            "forward_to_pusat".to_string(),
        )
        .await
    }

    /// Validator Wilayah returns to Operator Satker
    pub async fn return_to_operator(
        &self,
        id: Uuid,
        validator_id: Uuid,
        catatan: Option<String>,
    ) -> AppResult<PenghapusanBmn> {
        self.repository
            .update_validator_wilayah(id, validator_id, catatan.clone())
            .await?;

        self.transition(
            id,
            PenghapusanBmnStatus::ReturnedToOperator
                .to_state_name()
                .to_string(),
            validator_id,
            catatan,
            "return_to_operator".to_string(),
        )
        .await
    }

    /// Filesystem path the konsep-sk route handler streams from. `format` is
    /// "docx" or "pdf".
    pub async fn konsep_sk_path(
        &self,
        id: Uuid,
        format: &str,
    ) -> AppResult<Option<String>> {
        self.repository.konsep_sk_path(id, format).await
    }

    /// Validator Pusat generates konsep SK in BOTH DOCX (editable) and PDF
    /// (final) formats. Files land under
    /// `${DOCUMENT_STORAGE_PATH}/penghapusan-bmn/{id}/konsep-sk.{ext}`; the
    /// public download URLs are persisted in `konsep_sk_url` (DOCX) and
    /// `konsep_sk_pdf_url` (PDF).
    pub async fn generate_konsep_sk(
        &self,
        id: Uuid,
        validator_id: Uuid,
    ) -> AppResult<PenghapusanBmn> {
        let penghapusan = self.repository.get_by_id(id).await?;
        let status = PenghapusanBmnStatus::from_state_name(&penghapusan.status).unwrap_or_default();

        if !matches!(status, PenghapusanBmnStatus::VerifikasiPusat) {
            return Err(crate::errors::AppError::WorkflowError(
                "Konsep SK hanya bisa digenerate saat status Verifikasi Pusat".into(),
            ));
        }

        let docs = self.docs.as_ref().ok_or_else(|| {
            crate::errors::AppError::Internal(
                "DocumentGenerator port not wired into PenghapusanBmnService".into(),
            )
        })?;

        let template_id = std::env::var("KONSEP_SK_TEMPLATE_ID").unwrap_or_else(|_| {
            "00000000-0000-0000-0000-000000000002".to_string()
        });
        let data = serde_json::json!({
            "id": id.to_string(),
            "satker_id": penghapusan.satker_id,
            "nama_barang": penghapusan.nama_barang,
            "alasan": penghapusan.alasan,
            "status": penghapusan.status,
            "approval_date": chrono::Utc::now().format("%d %B %Y").to_string(),
        });

        let storage_root = std::env::var("DOCUMENT_STORAGE_PATH")
            .unwrap_or_else(|_| "/tmp/perlengkapan/docs".to_string());
        let dir = format!("{}/penghapusan-bmn/{}", storage_root, id);
        tokio::fs::create_dir_all(&dir).await.map_err(|e| {
            crate::errors::AppError::Internal(format!("mkdir {}: {}", dir, e))
        })?;
        let docx_path = format!("{}/konsep-sk.docx", dir);
        let pdf_path = format!("{}/konsep-sk.pdf", dir);

        let docx_request = lib_perlengkapan::contracts::DocumentRequest {
            template_id: template_id.clone(),
            format: lib_perlengkapan::contracts::DocumentFormat::Docx,
            data: data.clone(),
            locale: None,
            requested_by: None,
        };
        let pdf_request = lib_perlengkapan::contracts::DocumentRequest {
            template_id,
            format: lib_perlengkapan::contracts::DocumentFormat::Pdf,
            data,
            locale: None,
            requested_by: None,
        };

        let docx_bytes = docs.preview(docx_request).await.map_err(|e| {
            crate::errors::AppError::Internal(format!("konsep SK DOCX render failed: {}", e))
        })?;
        tokio::fs::write(&docx_path, &docx_bytes).await.map_err(|e| {
            crate::errors::AppError::Internal(format!("write {}: {}", docx_path, e))
        })?;

        let pdf_bytes = docs.preview(pdf_request).await.map_err(|e| {
            crate::errors::AppError::Internal(format!("konsep SK PDF render failed: {}", e))
        })?;
        tokio::fs::write(&pdf_path, &pdf_bytes).await.map_err(|e| {
            crate::errors::AppError::Internal(format!("write {}: {}", pdf_path, e))
        })?;

        let docx_url = format!(
            "/api/pembinaan/perlengkapan/penghapusan-bmn/{}/konsep-sk.docx",
            id
        );
        let pdf_url = format!(
            "/api/pembinaan/perlengkapan/penghapusan-bmn/{}/konsep-sk.pdf",
            id
        );

        self.repository
            .update_konsep_sk(id, &docx_url, &docx_path, &pdf_url, &pdf_path)
            .await?;

        // Transition to KonsepSKGenerated
        self.transition(
            id,
            PenghapusanBmnStatus::KonsepSKGenerated
                .to_state_name()
                .to_string(),
            validator_id,
            Some("Konsep Usulan SK Penghapusan BMN berhasil digenerate (DOCX + PDF)".to_string()),
            "generate_konsep_sk".to_string(),
        )
        .await
    }

    /// Validator Pusat uploads signed SK PDF
    pub async fn upload_signed_sk(
        &self,
        id: Uuid,
        validator_id: Uuid,
        signed_sk_pdf_url: String,
    ) -> AppResult<PenghapusanBmn> {
        let penghapusan = self.repository.get_by_id(id).await?;
        let status = PenghapusanBmnStatus::from_state_name(&penghapusan.status).unwrap_or_default();

        if !matches!(status, PenghapusanBmnStatus::KonsepSKGenerated) {
            return Err(crate::errors::AppError::WorkflowError(
                "SK hanya bisa diupload setelah konsep SK digenerate".into(),
            ));
        }

        // Update signed SK PDF URL
        self.repository
            .update_signed_sk(id, &signed_sk_pdf_url)
            .await?;

        // Transition to SKSigned then Completed
        self.transition(
            id,
            PenghapusanBmnStatus::SKSigned.to_state_name().to_string(),
            validator_id,
            Some("Usulan SK Penghapusan BMN telah ditandatangani".to_string()),
            "upload_signed_sk".to_string(),
        )
        .await?;

        // Auto-complete
        self.transition(
            id,
            PenghapusanBmnStatus::Completed.to_state_name().to_string(),
            validator_id,
            Some("Proses Usulan SK Penghapusan BMN selesai".to_string()),
            "complete".to_string(),
        )
        .await
    }

    /// Perform workflow transition
    pub async fn transition(
        &self,
        id: Uuid,
        to_state: String,
        user_id: Uuid,
        catatan: Option<String>,
        ip_address: String,
    ) -> AppResult<PenghapusanBmn> {
        let penghapusan = self.repository.get_by_id(id).await?;

        let transition_request = TransitionRequest {
            entity_id: id,
            from_state: penghapusan.status.clone(),
            to_state: to_state.clone(),
            user_id,
            catatan,
            ip_address,
        };

        self.workflow_engine
            .transition(transition_request)
            .await
            .map_err(|e| crate::errors::AppError::WorkflowError(e.to_string()))?;

        self.repository.update_status(id, &to_state).await?;
        self.repository.get_by_id(id).await
    }
}
