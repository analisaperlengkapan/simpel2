use super::PemakaianBmnService;
use crate::pemakaian_bmn::models::*;
use crate::shared::error::{AppError, AppResult};
use crate::shared::satker_scope::SatkerScope;
use tracing::info;
use uuid::Uuid;
use validator::Validate;

impl PemakaianBmnService {
    /// Create a new permit request
    ///
    /// Requirements: REQ-P001, REQ-P002, REQ-P003
    pub async fn create_permit(
        &self,
        request: CreateIzinPemakaianRequest,
        user_id: Uuid,
        user_nama: String,
        // Authoritative MySIMKARI satker_code of the creating operator (from JWT
        // claims, #66), persisted for RBAC scoping.
        satker_code: Option<String>,
    ) -> AppResult<IzinPemakaianBmn> {
        // Validate request
        request
            .validate()
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        // Validate BMN type-specific fields

        // Sebuah izin melekat pada SATU satker: tanpa itu barisnya tak punya
        // identitas aset, tak terlihat oleh operator satker mana pun, dan tak
        // bisa ikut cek tabrakan. Tolak di depan alih-alih menulis baris yatim.
        let satker = satker_code
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                AppError::BadRequest(
                    "Tidak ada identitas satker pada sesi Anda; izin pemakaian \
                     harus melekat pada satu satker"
                        .to_string(),
                )
            })?
            .to_string();

        // Validate the permit's default period before anything derived from it.
        if request.tanggal_selesai <= request.tanggal_mulai {
            return Err(AppError::BadRequest(
                "Tanggal selesai harus lebih besar dari tanggal mulai".to_string(),
            ));
        }

        // Every asset is checked BEFORE the first row is written. Checking as
        // we insert would leave a half-written permit behind when asset three
        // turns out to be taken.
        self.validate_items(
            &request.items,
            request.tanggal_mulai,
            request.tanggal_selesai,
            &satker,
            None,
        )
        .await?;

        info!(
            "Creating new permit over {} BMN for pegawai {}",
            request.items.len(),
            request.pegawai_nip
        );

        // Header and items are written in one transaction inside `create`.
        let permit = self
            .repository
            .create(request, user_id, user_nama, Some(satker))
            .await?;

        Ok(permit)
    }

    /// Filesystem path the konsep-surat route handler streams from.
    /// `format` is "docx" or "pdf".
    pub async fn konsep_surat_path(
        &self,
        id: Uuid,
        format: &str,
        scope: &SatkerScope,
    ) -> AppResult<Option<String>> {
        // Reached by permit id and streams a document, so it needs the same
        // gate as the detail behind it.
        self.repository.get_by_id(id, scope).await?;
        self.repository.konsep_surat_path(id, format).await
    }

    /// Generate konsep surat izin pemakaian BMN in BOTH DOCX (editable) and
    /// PDF (final) formats. Both files land under
    /// `${DOCUMENT_STORAGE_PATH}/pemakaian-bmn/{id}/konsep-surat.{ext}` and
    /// their public download URLs are written to
    /// `konsep_surat_url` (DOCX) and `konsep_surat_pdf_url` (PDF).
    ///
    /// Requirements: REQ-P006
    pub async fn generate_konsep_surat(
        &self,
        id: Uuid,
        _user_id: Uuid,
        scope: &SatkerScope,
    ) -> AppResult<IzinPemakaianBmn> {
        let current = self.repository.get_by_id(id, scope).await?;
        let status = PemakaianBmnStatus::from_state_name(&current.status)
            .unwrap_or(PemakaianBmnStatus::Draft);

        // Can generate surat from Draft, Submitted, or Approved status
        if !matches!(
            status,
            PemakaianBmnStatus::Draft
                | PemakaianBmnStatus::Submitted
                | PemakaianBmnStatus::Approved
        ) {
            return Err(AppError::BadRequest(
                "Konsep surat hanya bisa digenerate pada status Draft, Submitted, atau Approved"
                    .to_string(),
            ));
        }

        let docs = self.docs.as_ref().ok_or_else(|| {
            AppError::Internal("DocumentGenerator port not wired into PemakaianBmnService".into())
        })?;

        // Template variables fed into both renders. We reuse the existing
        // permit payload so DOCX and PDF stay structurally identical.
        let template_id = std::env::var("KONSEP_SURAT_TEMPLATE_ID").unwrap_or_else(|_| {
            // Placeholder UUID used until the production template is seeded.
            "00000000-0000-0000-0000-000000000001".to_string()
        });
        let data = serde_json::json!({
            "nomor_izin": current.nomor_izin,
            "pegawai_nip": current.pegawai_nip,
            "pegawai_nama": current.pegawai_nama,
            "pegawai_jabatan": current.pegawai_jabatan,
            "pegawai_satker_nama": current.pegawai_satker_nama,
            // The asset LIST, so the template can loop. It used to be four
            // flat fields naming the permit's first asset, which is why the
            // letter only ever showed one however many were on the permit.
            // `jumlah_item` alongside it mirrors what penghapusan already
            // hands its own template.
            "items": serde_json::to_value(&current.bmn_items).unwrap_or(serde_json::Value::Null),
            "jumlah_item": current.bmn_items.len(),
            "tanggal_mulai": current.tanggal_mulai.to_string(),
            "tanggal_selesai": current.tanggal_selesai.to_string(),
            "keperluan": current.keperluan,
        });

        let storage_root = std::env::var("DOCUMENT_STORAGE_PATH")
            .unwrap_or_else(|_| "/tmp/perlengkapan/docs".to_string());
        let dir = format!("{}/pemakaian-bmn/{}", storage_root, id);
        tokio::fs::create_dir_all(&dir)
            .await
            .map_err(|e| AppError::Internal(format!("mkdir {}: {}", dir, e)))?;
        let docx_path = format!("{}/konsep-surat.docx", dir);
        let pdf_path = format!("{}/konsep-surat.pdf", dir);

        let docx_request = crate::contracts::DocumentRequest {
            template_id: template_id.clone(),
            format: crate::contracts::DocumentFormat::Docx,
            data: data.clone(),
            locale: None,
            requested_by: None,
        };
        let pdf_request = crate::contracts::DocumentRequest {
            template_id: template_id.clone(),
            format: crate::contracts::DocumentFormat::Pdf,
            data,
            locale: None,
            requested_by: None,
        };

        let docx_bytes = docs
            .preview(docx_request)
            .await
            .map_err(|e| AppError::Internal(format!("konsep surat DOCX render failed: {}", e)))?;
        tokio::fs::write(&docx_path, &docx_bytes)
            .await
            .map_err(|e| AppError::Internal(format!("write {}: {}", docx_path, e)))?;

        let pdf_bytes = docs
            .preview(pdf_request)
            .await
            .map_err(|e| AppError::Internal(format!("konsep surat PDF render failed: {}", e)))?;
        tokio::fs::write(&pdf_path, &pdf_bytes)
            .await
            .map_err(|e| AppError::Internal(format!("write {}: {}", pdf_path, e)))?;

        let docx_url = format!(
            "/api/v1/perlengkapan/pemakaian-bmn/{}/konsep-surat.docx",
            id
        );
        let pdf_url = format!("/api/v1/perlengkapan/pemakaian-bmn/{}/konsep-surat.pdf", id);

        self.repository
            .update_konsep_surat(id, &docx_url, &docx_path, &pdf_url, &pdf_path)
            .await?;

        info!(
            "Generated konsep surat (DOCX + PDF) for permit {} at {}",
            id, dir
        );
        self.repository.get_by_id(id, scope).await
    }

    /// Upload signed PDF izin pemakaian BMN and mark as completed
    ///
    /// Requirements: REQ-P006
    pub async fn upload_signed_pdf(
        &self,
        id: Uuid,
        signed_pdf_url: String,
        _user_id: Uuid,
        scope: &SatkerScope,
    ) -> AppResult<IzinPemakaianBmn> {
        let current = self.repository.get_by_id(id, scope).await?;

        // Konsep surat must have been generated first
        if current.konsep_surat_url.is_none() {
            return Err(AppError::BadRequest(
                "Konsep surat harus digenerate terlebih dahulu".to_string(),
            ));
        }

        // Update signed PDF and mark as completed
        self.repository
            .update_signed_pdf(id, &signed_pdf_url)
            .await?;

        info!(
            "Uploaded signed PDF for permit {}, marking as completed",
            id
        );
        self.repository.get_by_id(id, scope).await
    }
}
