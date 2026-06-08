use super::PemakaianBmnService;
use crate::pemakaian_bmn::models::*;
use crate::shared::error::{AppError, AppResult};
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
    ) -> AppResult<IzinPemakaianBmn> {
        // Validate request
        request
            .validate()
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        // Validate BMN type-specific fields
        self.validate_bmn_type_fields(&request)?;

        // Check BMN availability (REQ-P002, REQ-P003)
        let availability = self
            .repository
            .check_bmn_availability(&request.bmn_nup)
            .await?;
        if !availability.is_available {
            return Err(AppError::BadRequest(format!(
                "BMN {} sedang digunakan oleh {} hingga {}",
                request.bmn_nup,
                availability.active_permit_holder.unwrap_or_default(),
                availability
                    .active_permit_expires
                    .map(|d| d.to_string())
                    .unwrap_or_default()
            )));
        }

        // Validate date range
        if request.tanggal_selesai <= request.tanggal_mulai {
            return Err(AppError::BadRequest(
                "Tanggal selesai harus lebih besar dari tanggal mulai".to_string(),
            ));
        }

        info!(
            "Creating new permit for BMN {} by pegawai {}",
            request.bmn_nup, request.pegawai_nip
        );

        // Create permit
        let mut permit = self
            .repository
            .create(request.clone(), user_id, user_nama)
            .await?;

        // Create additional BMN items if any (multi-BMN support)
        if !request.additional_bmn_items.is_empty() {
            for item in &request.additional_bmn_items {
                // Check availability for each additional BMN
                let avail = self
                    .repository
                    .check_bmn_availability(&item.bmn_nup)
                    .await?;
                if !avail.is_available {
                    return Err(AppError::BadRequest(format!(
                        "BMN {} sedang digunakan oleh {} hingga {}",
                        item.bmn_nup,
                        avail.active_permit_holder.unwrap_or_default(),
                        avail
                            .active_permit_expires
                            .map(|d| d.to_string())
                            .unwrap_or_default()
                    )));
                }
                self.repository
                    .create_bmn_item(permit.id, item.clone())
                    .await?;
            }
        }

        // Reload permit with bmn_items
        permit = self.repository.get_by_id(permit.id).await?;

        Ok(permit)
    }

    /// Filesystem path the konsep-surat route handler streams from.
    /// `format` is "docx" or "pdf".
    pub async fn konsep_surat_path(&self, id: Uuid, format: &str) -> AppResult<Option<String>> {
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
    ) -> AppResult<IzinPemakaianBmn> {
        let current = self.repository.get_by_id(id).await?;
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
            "jenis_bmn": current.jenis_bmn,
            "bmn_nup": current.bmn_nup,
            "bmn_kode_barang": current.bmn_kode_barang,
            "bmn_nama_barang": current.bmn_nama_barang,
            "tanggal_mulai": current.tanggal_mulai.to_string(),
            "tanggal_selesai": current.tanggal_selesai.to_string(),
            "keperluan": current.keperluan,
            "lokasi_pemakaian": current.lokasi_pemakaian,
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
            "/api/pembinaan/perlengkapan/pemakaian-bmn/{}/konsep-surat.docx",
            id
        );
        let pdf_url = format!(
            "/api/pembinaan/perlengkapan/pemakaian-bmn/{}/konsep-surat.pdf",
            id
        );

        self.repository
            .update_konsep_surat(id, &docx_url, &docx_path, &pdf_url, &pdf_path)
            .await?;

        info!(
            "Generated konsep surat (DOCX + PDF) for permit {} at {}",
            id, dir
        );
        self.repository.get_by_id(id).await
    }

    /// Upload signed PDF izin pemakaian BMN and mark as completed
    ///
    /// Requirements: REQ-P006
    pub async fn upload_signed_pdf(
        &self,
        id: Uuid,
        signed_pdf_url: String,
        _user_id: Uuid,
    ) -> AppResult<IzinPemakaianBmn> {
        let current = self.repository.get_by_id(id).await?;

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
        self.repository.get_by_id(id).await
    }
}
