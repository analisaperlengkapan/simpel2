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
        // Authoritative MySIMKARI satker_code of the creating operator (from JWT
        // claims, #66), persisted for RBAC scoping.
        satker_code: Option<String>,
    ) -> AppResult<IzinPemakaianBmn> {
        // Validate request
        request
            .validate()
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        // Validate BMN type-specific fields
        self.validate_bmn_type_fields(&request)?;

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

        // Check BMN availability (REQ-P002, REQ-P003).
        //
        // Ber-key pada identitas aset penuh — kode satker + kode barang + NUP.
        // Versi lama ber-key `bmn_nup` saja dgn `SatkerScope::All`, "supaya
        // jawabannya tak bergantung siapa yang bertanya". Niatnya benar,
        // kuncinya salah: 44.017 aset di 553 satker sama-sama ber-NUP `1`
        // (lihat header `repository::lookup`), jadi satu izin di mana pun
        // memblokir ratusan satker atas aset mereka sendiri — dan pesan error
        // di bawah menyebut nama pegawai satker lain sebagai pemegangnya.
        // Karena satker kini ikut jadi kunci, tabrakan yang ditemukan pasti
        // milik satker pemanggil: tak ada lagi yang bocor untuk diredaksi.
        if let Some(bentrok) = self
            .repository
            .find_booking_conflict(&request.bmn_nup, &request.bmn_kode_barang, &satker, None)
            .await?
        {
            return Err(AppError::BadRequest(format!(
                "BMN {} (kode barang {}) sedang digunakan oleh {} hingga {}",
                request.bmn_nup, request.bmn_kode_barang, bentrok.holder, bentrok.expires
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
            .create(request.clone(), user_id, user_nama, Some(satker.clone()))
            .await?;

        // Create additional BMN items if any (multi-BMN support)
        if !request.additional_bmn_items.is_empty() {
            for item in &request.additional_bmn_items {
                // Setiap BMN tambahan dicek dgn identitas aset penuh, sama
                // seperti BMN utama di atas.
                if let Some(bentrok) = self
                    .repository
                    .find_booking_conflict(&item.bmn_nup, &item.bmn_kode_barang, &satker, None)
                    .await?
                {
                    return Err(AppError::BadRequest(format!(
                        "BMN {} (kode barang {}) sedang digunakan oleh {} hingga {}",
                        item.bmn_nup, item.bmn_kode_barang, bentrok.holder, bentrok.expires
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
