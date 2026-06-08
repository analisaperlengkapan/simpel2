use super::KebutuhanBmnService;
use crate::kebutuhan_bmn::models::*;
use crate::kebutuhan_bmn::repository::KebutuhanBmnRepository;
use crate::shared::error::{AppError, AppResult};
use tracing::info;
use uuid::Uuid;
use validator::Validate;

impl KebutuhanBmnService {
    /// Get satkers for a pengajuan
    pub async fn get_pengajuan_satkers(
        &self,
        pengajuan_id: Uuid,
    ) -> AppResult<Vec<PengajuanKebutuhanBmnSatker>> {
        self.repository.get_pengajuan_satkers(pengajuan_id).await
    }

    /// Get satker with its barang list
    pub async fn get_satker_with_barang(
        &self,
        satker_id: Uuid,
        page: i32,
        per_page: i32,
    ) -> AppResult<SatkerWithBarangResponse> {
        let satker = self.repository.get_satker_by_id(satker_id).await?;
        let (barang_list, total_barang) = self
            .repository
            .get_satker_barang(satker_id, page, per_page, None)
            .await?;

        let total_jumlah: i64 = barang_list.iter().map(|b| b.jumlah as i64).sum();

        Ok(SatkerWithBarangResponse {
            satker,
            barang_list,
            total_barang,
            total_jumlah,
        })
    }

    /// Add a satker to pengajuan
    pub async fn add_satker_to_pengajuan(
        &self,
        pengajuan_id: Uuid,
        satker_id: &str,
        satker_name: Option<String>,
        user_id: Option<Uuid>,
    ) -> AppResult<PengajuanKebutuhanBmnSatker> {
        // Verify pengajuan exists and is editable
        let pengajuan = self.repository.get_pengajuan_by_id(pengajuan_id).await?;
        if pengajuan.status != KebutuhanBmnStatus::Draft {
            return Err(AppError::BadRequest(
                "Satker hanya dapat ditambahkan pada pengajuan dengan status Draft".to_string(),
            ));
        }

        self.repository
            .create_pengajuan_satker(pengajuan_id, satker_id, satker_name, user_id)
            .await
    }

    // ========================================================================
    // Barang Operations
    // ========================================================================

    /// Add a barang to a satker
    pub async fn create_barang(
        &self,
        satker_id: Uuid,
        request: CreateBarangRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<PengajuanKebutuhanBmnBarang> {
        // Validate request
        request
            .validate()
            .map_err(|e| AppError::BadRequest(format!("Validation error: {}", e)))?;

        // Verify satker is in input mode
        let satker = self.repository.get_satker_by_id(satker_id).await?;
        if !matches!(
            satker.status,
            KebutuhanBmnStatus::InputBarang | KebutuhanBmnStatus::RevisiSatker
        ) {
            return Err(AppError::BadRequest(
                "Barang hanya dapat ditambahkan saat status Input Barang atau Revisi".to_string(),
            ));
        }

        // V029 (Fase 1.6): Allowed-list BMN enforcement. Cek kode_barang
        // request masuk dlm whitelist Validator Pusat. Kosong / NULL =
        // legacy mode (semua boleh). Pengajuan_id resolve via satker.
        if let Some(kode) = &request.kode_barang {
            let allowed = self
                .repository
                .is_bmn_allowed_for_pengajuan(satker.pengajuan_id, kode)
                .await?;
            if !allowed {
                return Err(AppError::BadRequest(format!(
                    "Barang dgn kode_barang '{}' tidak diizinkan untuk pengajuan ini. Periksa daftar BMN yg ditetapkan Validator Pusat.",
                    kode
                )));
            }
        }

        info!("Creating barang for satker {}: {}", satker_id, request.nama);
        self.repository
            .create_barang(satker_id, request, user_id)
            .await
    }

    /// V029 (Fase 1.6): list allowed BMN utk pengajuan — dipakai FE
    /// dropdown saat Operator Satker input barang.
    pub async fn list_bmn_referensi(
        &self,
        pengajuan_id: Uuid,
    ) -> AppResult<Vec<PengajuanBmnReferensi>> {
        self.repository.list_bmn_referensi(pengajuan_id).await
    }

    /// Update barang approval (jml_setuju)
    pub async fn update_barang_approval(
        &self,
        barang_id: Uuid,
        request: UpdateBarangApprovalRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<PengajuanKebutuhanBmnBarang> {
        let barang = self.repository.get_barang_by_id(barang_id).await?;

        // Validate jml_setuju doesn't exceed jumlah
        if request.jml_setuju > barang.jumlah {
            return Err(AppError::BadRequest(
                "Jumlah disetujui tidak boleh melebihi jumlah diminta".to_string(),
            ));
        }

        if request.jml_setuju < 0 {
            return Err(AppError::BadRequest(
                "Jumlah disetujui tidak boleh negatif".to_string(),
            ));
        }

        self.repository
            .update_barang_approval(barang_id, request, user_id)
            .await
    }

    /// Delete a barang
    pub async fn delete_barang(&self, barang_id: Uuid, user_id: Option<Uuid>) -> AppResult<()> {
        // Check if barang's satker is in editable status
        let barang = self.repository.get_barang_by_id(barang_id).await?;
        let satker = self
            .repository
            .get_satker_by_id(barang.pengajuan_satker_id)
            .await?;

        if !matches!(
            satker.status,
            KebutuhanBmnStatus::InputBarang | KebutuhanBmnStatus::RevisiSatker
        ) {
            return Err(AppError::BadRequest(
                "Barang hanya dapat dihapus saat status Input Barang atau Revisi".to_string(),
            ));
        }

        info!("Deleting barang: {} by user {:?}", barang_id, user_id);
        self.repository.delete_barang(barang_id).await
    }

    /// Set priorities for multiple barang
    pub async fn set_barang_prioritas(&self, request: SetPrioritasRequest) -> AppResult<()> {
        // Validate all items have valid prioritas
        for item in &request.items {
            if item.prioritas < 0 {
                return Err(AppError::BadRequest(
                    "Prioritas tidak boleh negatif".to_string(),
                ));
            }
        }

        info!("Setting prioritas for {} items", request.items.len());
        self.repository.set_barang_prioritas(request.items).await
    }

    // ========================================================================
    // Analisis Kelayakan
    // ========================================================================
}
