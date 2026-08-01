use super::PakaianDinasRepository;
use crate::pakaian_dinas::models::*;
use crate::shared::error::{AppError, AppResult, bad_request};
use uuid::Uuid;

impl PakaianDinasRepository {
    pub async fn get_pengajuan_satker_list(
        &self,
        pengajuan_id: Uuid,
        page: i32,
        per_page: i32,
    ) -> AppResult<(Vec<PengajuanSatker>, i64)> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let offset = (page - 1) * per_page;

        let count_row = client
            .query_one(
                "SELECT COUNT(*) as total FROM perlengkapan.pengajuan_pakaian_dinas_satker WHERE pengajuan_id = $1",
                &[&pengajuan_id],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let total: i64 = count_row.get("total");

        let rows = client
            .query(
                r#"
                SELECT ps.*, s.nama as satker_nama, s.kode as satker_kode,
                       (SELECT COUNT(*) FROM perlengkapan.pengajuan_pakaian_dinas_satker_pegawai WHERE pengajuan_satker_id = ps.id) as total_pegawai
                FROM perlengkapan.pengajuan_pakaian_dinas_satker ps
                LEFT JOIN integrasi.mysimkari_satker s ON ps.satker_id = s.kode_satker
                WHERE ps.pengajuan_id = $1
                ORDER BY s.nama ASC
                LIMIT $2 OFFSET $3
                "#,
                &[&pengajuan_id, &(per_page as i64), &(offset as i64)],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let items: Vec<PengajuanSatker> = rows.iter().map(PengajuanSatker::from_row).collect();
        Ok((items, total))
    }

    pub async fn get_pengajuan_satker_by_id(&self, id: Uuid) -> AppResult<PengajuanSatker> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let row = client
            .query_opt(
                r#"
                SELECT ps.*, s.nama as satker_nama, s.kode as satker_kode,
                       (SELECT COUNT(*) FROM perlengkapan.pengajuan_pakaian_dinas_satker_pegawai WHERE pengajuan_satker_id = ps.id) as total_pegawai
                FROM perlengkapan.pengajuan_pakaian_dinas_satker ps
                LEFT JOIN integrasi.mysimkari_satker s ON ps.satker_id = s.kode_satker
                WHERE ps.id = $1
                "#,
                &[&id],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Pengajuan satker tidak ditemukan".to_string()))?;

        Ok(PengajuanSatker::from_row(&row))
    }

    // ============ Employee Sizes ============
}
