use super::PemakaianBmnRepository;
use crate::pemakaian_bmn::models::*;
use crate::shared::error::{AppError, AppResult};
use crate::shared::repo::PoolExt;
use uuid::Uuid;

impl PemakaianBmnRepository {
    /// Validator Satker meneruskan ke Approver Satker.
    /// State: SUBMITTED → SUBMITTED_APPROVER_SATKER (kode 3010).
    pub async fn validator_satker_forward(
        &self,
        id: Uuid,
        validator_id: Uuid,
        validator_nama: &str,
        expected_version: i32,
        catatan: Option<&str>,
    ) -> AppResult<IzinPemakaianBmn> {
        let client = self.pool.client().await?;
        let row_opt = client
            .query_opt(
                r#"
                UPDATE perlengkapan.izin_pemakaian_bmn
                SET status                   = 'SUBMITTED_APPROVER_SATKER',
                    status_kode              = 3010,
                    validator_satker_id      = $1,
                    validator_satker_nama    = $2,
                    tanggal_validasi_satker  = NOW(),
                    catatan_validator_satker = $3,
                    updated_by               = $1,
                    updated_by_nama          = $2,
                    updated_at               = NOW(),
                    version                  = version + 1
                WHERE id = $4 AND version = $5 AND status_kode = 3001
                RETURNING *
                "#,
                &[
                    &validator_id,
                    &validator_nama,
                    &catatan,
                    &id,
                    &expected_version,
                ],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        match row_opt {
            Some(row) => Ok(self.row_to_permit(row)),
            None => Err(AppError::Conflict(
                "Versi data sudah berubah atau status bukan SUBMITTED — silakan refresh".into(),
            )),
        }
    }

    /// Validator Satker mengembalikan ke Operator utk revisi.
    /// State: SUBMITTED → REVISI_OPERATOR (kode 3011).
    pub async fn validator_satker_return(
        &self,
        id: Uuid,
        validator_id: Uuid,
        validator_nama: &str,
        expected_version: i32,
        catatan: &str,
    ) -> AppResult<IzinPemakaianBmn> {
        if catatan.trim().is_empty() {
            return Err(AppError::BadRequest(
                "Catatan revisi wajib diisi saat mengembalikan ke Operator".into(),
            ));
        }
        let client = self.pool.client().await?;
        let row_opt = client
            .query_opt(
                r#"
                UPDATE perlengkapan.izin_pemakaian_bmn
                SET status                   = 'REVISI_OPERATOR',
                    status_kode              = 3011,
                    validator_satker_id      = $1,
                    validator_satker_nama    = $2,
                    tanggal_validasi_satker  = NOW(),
                    catatan_validator_satker = $3,
                    updated_by               = $1,
                    updated_by_nama          = $2,
                    updated_at               = NOW(),
                    version                  = version + 1
                WHERE id = $4 AND version = $5
                  AND status_kode IN (3001, 3010)
                RETURNING *
                "#,
                &[
                    &validator_id,
                    &validator_nama,
                    &catatan,
                    &id,
                    &expected_version,
                ],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        match row_opt {
            Some(row) => Ok(self.row_to_permit(row)),
            None => Err(AppError::Conflict(
                "Versi data sudah berubah atau status tidak valid utk revisi — silakan refresh"
                    .into(),
            )),
        }
    }

    /// Approver Satker (Pengguna Barang Satker) menyetujui — siap aktivasi.
    /// State: SUBMITTED_APPROVER_SATKER → APPROVED.
    pub async fn approver_satker_approve(
        &self,
        id: Uuid,
        approver_id: Uuid,
        approver_nama: &str,
        expected_version: i32,
        catatan: Option<&str>,
    ) -> AppResult<IzinPemakaianBmn> {
        let client = self.pool.client().await?;
        let row_opt = client
            .query_opt(
                r#"
                UPDATE perlengkapan.izin_pemakaian_bmn
                SET status                   = 'APPROVED',
                    status_kode              = 3002,
                    approver_satker_id       = $1,
                    approver_satker_nama     = $2,
                    tanggal_approval_satker  = NOW(),
                    catatan_approver_satker  = $3,
                    approved_by              = $1,
                    approved_by_nama         = $2,
                    approved_at              = NOW(),
                    catatan_approval         = $3,
                    updated_by               = $1,
                    updated_by_nama          = $2,
                    updated_at               = NOW(),
                    version                  = version + 1
                WHERE id = $4 AND version = $5 AND status_kode = 3010
                RETURNING *
                "#,
                &[
                    &approver_id,
                    &approver_nama,
                    &catatan,
                    &id,
                    &expected_version,
                ],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        match row_opt {
            Some(row) => Ok(self.row_to_permit(row)),
            None => Err(AppError::Conflict(
                "Versi data berubah atau status bukan SUBMITTED_APPROVER_SATKER — silakan refresh"
                    .into(),
            )),
        }
    }

    /// Approver Satker mengembalikan ke Operator utk revisi.
    /// State: SUBMITTED_APPROVER_SATKER → REVISI_OPERATOR.
    pub async fn approver_satker_return(
        &self,
        id: Uuid,
        approver_id: Uuid,
        approver_nama: &str,
        expected_version: i32,
        catatan: &str,
    ) -> AppResult<IzinPemakaianBmn> {
        // Re-use validator_satker_return — guard kolom yg dipakai sama,
        // tapi audit field-nya `approver_satker_*` agar jelas siapa yg
        // menolak. Tetap simpan ke `catatan_validator_satker` agar
        // Operator melihat catatan terbaru di field umum.
        if catatan.trim().is_empty() {
            return Err(AppError::BadRequest(
                "Catatan revisi wajib diisi saat mengembalikan ke Operator".into(),
            ));
        }
        let client = self.pool.client().await?;
        let row_opt = client
            .query_opt(
                r#"
                UPDATE perlengkapan.izin_pemakaian_bmn
                SET status                  = 'REVISI_OPERATOR',
                    status_kode             = 3011,
                    approver_satker_id      = $1,
                    approver_satker_nama    = $2,
                    tanggal_approval_satker = NOW(),
                    catatan_approver_satker = $3,
                    updated_by              = $1,
                    updated_by_nama         = $2,
                    updated_at              = NOW(),
                    version                 = version + 1
                WHERE id = $4 AND version = $5 AND status_kode = 3010
                RETURNING *
                "#,
                &[
                    &approver_id,
                    &approver_nama,
                    &catatan,
                    &id,
                    &expected_version,
                ],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        match row_opt {
            Some(row) => Ok(self.row_to_permit(row)),
            None => Err(AppError::Conflict(
                "Versi data berubah atau status bukan SUBMITTED_APPROVER_SATKER — silakan refresh"
                    .into(),
            )),
        }
    }

    /// Operator re-submit setelah revisi.
    /// State: REVISI_OPERATOR → SUBMITTED (kembali ke Validator Satker).
    pub async fn operator_resubmit(
        &self,
        id: Uuid,
        operator_id: Uuid,
        operator_nama: &str,
        expected_version: i32,
    ) -> AppResult<IzinPemakaianBmn> {
        let client = self.pool.client().await?;
        let row_opt = client
            .query_opt(
                r#"
                UPDATE perlengkapan.izin_pemakaian_bmn
                SET status          = 'SUBMITTED',
                    status_kode     = 3001,
                    updated_by      = $1,
                    updated_by_nama = $2,
                    updated_at      = NOW(),
                    version         = version + 1
                WHERE id = $3 AND version = $4 AND status_kode = 3011
                RETURNING *
                "#,
                &[&operator_id, &operator_nama, &id, &expected_version],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        match row_opt {
            Some(row) => Ok(self.row_to_permit(row)),
            None => Err(AppError::Conflict(
                "Versi data berubah atau status bukan REVISI_OPERATOR — silakan refresh".into(),
            )),
        }
    }
}
