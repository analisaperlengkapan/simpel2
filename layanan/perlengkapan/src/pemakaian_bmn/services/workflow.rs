use super::PemakaianBmnService;
use crate::pemakaian_bmn::models::*;
use crate::shared::error::{AppError, AppResult};
use tracing::{info, warn};
use uuid::Uuid;

impl PemakaianBmnService {
    pub async fn validator_satker_forward(
        &self,
        id: Uuid,
        validator_id: Uuid,
        validator_nama: String,
        expected_version: i32,
        catatan: Option<String>,
    ) -> AppResult<IzinPemakaianBmn> {
        self.repository
            .validator_satker_forward(
                id,
                validator_id,
                &validator_nama,
                expected_version,
                catatan.as_deref(),
            )
            .await
    }

    pub async fn validator_satker_return(
        &self,
        id: Uuid,
        validator_id: Uuid,
        validator_nama: String,
        expected_version: i32,
        catatan: String,
    ) -> AppResult<IzinPemakaianBmn> {
        self.repository
            .validator_satker_return(
                id,
                validator_id,
                &validator_nama,
                expected_version,
                &catatan,
            )
            .await
    }

    pub async fn approver_satker_approve(
        &self,
        id: Uuid,
        approver_id: Uuid,
        approver_nama: String,
        expected_version: i32,
        catatan: Option<String>,
    ) -> AppResult<IzinPemakaianBmn> {
        let approved = self
            .repository
            .approver_satker_approve(
                id,
                approver_id,
                &approver_nama,
                expected_version,
                catatan.as_deref(),
            )
            .await?;
        // Auto-activate: generate nomor_izin + SK Izin 2-halaman.
        // Kegagalan aktivasi tidak boleh me-rollback approval — biar
        // operator dapat retry aktivasi manual jika misal SIMAN sedang down.
        match self.activate_permit(approved.id, approver_id).await {
            Ok(active) => Ok(active),
            Err(e) => {
                tracing::warn!(
                    "approver_satker_approve: approval sukses tapi activate_permit gagal: {}. Approval tetap dipertahankan; aktivasi dapat di-retry.",
                    e
                );
                Ok(approved)
            }
        }
    }

    pub async fn approver_satker_return(
        &self,
        id: Uuid,
        approver_id: Uuid,
        approver_nama: String,
        expected_version: i32,
        catatan: String,
    ) -> AppResult<IzinPemakaianBmn> {
        self.repository
            .approver_satker_return(id, approver_id, &approver_nama, expected_version, &catatan)
            .await
    }

    pub async fn operator_resubmit(
        &self,
        id: Uuid,
        operator_id: Uuid,
        operator_nama: String,
        expected_version: i32,
    ) -> AppResult<IzinPemakaianBmn> {
        self.repository
            .operator_resubmit(id, operator_id, &operator_nama, expected_version)
            .await
    }

    /// Activate a permit (generate permit number and set to ACTIVE)
    ///
    /// Requirements: REQ-P005, REQ-P006
    pub async fn activate_permit(&self, id: Uuid, user_id: Uuid) -> AppResult<IzinPemakaianBmn> {
        let current = self.repository.get_by_id(id).await?;

        // Can only activate APPROVED permits
        if current.status != "APPROVED" {
            return Err(AppError::BadRequest(
                "Hanya izin dengan status APPROVED yang dapat diaktifkan".to_string(),
            ));
        }

        // Generate permit number (REQ-P005)
        let nomor_izin = self.repository.generate_permit_number(id).await?;
        info!("Generated permit number {} for permit {}", nomor_izin, id);

        // Transition to ACTIVE
        let transition_request = WorkflowTransitionRequest {
            target_status: "ACTIVE".to_string(),
            catatan: Some(format!("Izin diaktifkan dengan nomor {}", nomor_izin)),
        };

        // System-driven activation — pass a sentinel IP so the audit row
        // is unambiguous about the originator. Any human-driven transition
        // arrives through the handler with a real ClientIp.
        let mut permit = self
            .transition_permit_status(id, transition_request, user_id, "system".to_string())
            .await?;

        // Generate permit document (REQ-P006)
        if let Some(docs) = &self.docs {
            info!("Generating permit document for permit {}", id);

            match self.generate_permit_document(&permit, docs.as_ref()).await {
                Ok((document_id, document_url)) => {
                    info!(
                        "Successfully generated document {} for permit {}",
                        document_id, id
                    );

                    // Update permit with document reference
                    permit = self
                        .repository
                        .update_document_fields(id, document_id, document_url)
                        .await?;
                }
                Err(e) => {
                    warn!(
                        "Failed to generate document for permit {}: {}. Manual generation required.",
                        id, e
                    );
                    // Don't fail activation if document generation fails
                    // Document can be generated manually later
                }
            }
        } else {
            warn!(
                "Document generator not configured, skipping document generation for permit {}",
                id
            );
        }

        Ok(permit)
    }

    /// Generate permit document via the [`DocumentGenerator`] port.
    ///
    /// Requirements: REQ-P006, REQ-D002, REQ-D004
    async fn generate_permit_document(
        &self,
        permit: &IzinPemakaianBmn,
        docs: &dyn lib_perlengkapan::contracts::DocumentGenerator,
    ) -> AppResult<(Uuid, String)> {
        // Prepare document data
        let mut document_data = serde_json::json!({
            "nomor_izin": permit.nomor_izin,
            "pegawai_nip": permit.pegawai_nip,
            "pegawai_nama": permit.pegawai_nama,
            "pegawai_jabatan": permit.pegawai_jabatan,
            "pegawai_satker_nama": permit.pegawai_satker_nama,
            "jenis_bmn": permit.jenis_bmn,
            "bmn_nup": permit.bmn_nup,
            "bmn_kode_barang": permit.bmn_kode_barang,
            "bmn_nama_barang": permit.bmn_nama_barang,
            "bmn_merk": permit.bmn_merk,
            "bmn_tahun_perolehan": permit.bmn_tahun_perolehan,
            "no_polisi": permit.no_polisi,
            "no_bpkb": permit.no_bpkb,
            "no_rangka": permit.no_rangka,
            "no_mesin": permit.no_mesin,
            "alamat": permit.alamat,
            "luas_tanah": permit.luas_tanah,
            "luas_bangunan": permit.luas_bangunan,
            "serial_number": permit.serial_number,
            "tanggal_mulai": permit.tanggal_mulai.to_string(),
            "tanggal_selesai": permit.tanggal_selesai.to_string(),
            "keperluan": permit.keperluan,
            "lokasi_pemakaian": permit.lokasi_pemakaian,
            "approved_by_nama": permit.approved_by_nama,
            "approved_at": permit.approved_at.map(|d| d.to_rfc3339()),
        });
        if let Some(obj) = document_data.as_object_mut() {
            obj.insert(
                "_workflow_meta".to_string(),
                serde_json::json!({
                    "entity_type": "pemakaian_bmn",
                    "entity_id": permit.id.to_string(),
                    "document_type": "surat_izin_pemakaian",
                }),
            );
        }

        // Template ID is config-driven. Operators seed the real template
        // UUID into `dokumen.document_templates` (template_type =
        // `surat_izin_pemakaian`) and expose its UUID via the environment
        // variable below. The fallback UUID is a well-known sentinel used
        // by integration tests / dev seeds; production deployments must
        // override it.
        let template_id_str = std::env::var("PEMAKAIAN_SURAT_IZIN_TEMPLATE_ID")
            .unwrap_or_else(|_| "00000000-0000-0000-0000-000000000001".to_string());
        let template_id = Uuid::parse_str(&template_id_str).map_err(|e| {
            AppError::Internal(format!(
                "Invalid PEMAKAIAN_SURAT_IZIN_TEMPLATE_ID ({}): {}",
                template_id_str, e
            ))
        })?;

        // Call the document generator with retry logic
        let mut retry_count = 0;
        let max_retries = 3;

        loop {
            let request = lib_perlengkapan::contracts::DocumentRequest {
                template_id: template_id.to_string(),
                format: lib_perlengkapan::contracts::DocumentFormat::Pdf,
                data: document_data.clone(),
                locale: None,
                requested_by: None,
            };

            match docs.generate(request).await {
                Ok(artifact) => {
                    return Ok((artifact.document_id, artifact.storage_key));
                }
                Err(e) => {
                    retry_count += 1;
                    if retry_count >= max_retries {
                        return Err(AppError::Internal(format!(
                            "Document generation failed after {} retries: {}",
                            max_retries, e
                        )));
                    }

                    warn!(
                        "Document generation attempt {} failed: {}. Retrying...",
                        retry_count, e
                    );

                    // Wait before retry (exponential backoff)
                    tokio::time::sleep(tokio::time::Duration::from_secs(2u64.pow(retry_count)))
                        .await;
                }
            }
        }
    }
}
