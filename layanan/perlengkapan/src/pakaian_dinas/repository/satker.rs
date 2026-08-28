use super::PakaianDinasRepository;
use crate::pakaian_dinas::models::*;
use crate::shared::error::{AppError, AppResult, bad_request};
use crate::shared::satker_scope::{BoxedParam, SatkerScope, as_refs};
use uuid::Uuid;

/// `scope` as a trailing `AND` over a participation row's own satker.
///
/// A campaign is nationwide, but a satker's RESPONSE to it — who they listed,
/// what sizes, where it sits in the workflow — is that satker's data. Both
/// queries below used to return every participant to any authenticated caller.
fn scope_and(scope: &SatkerScope, params: &mut Vec<BoxedParam>) -> String {
    crate::shared::satker_scope::scope_and(scope, "ps.satker_id", params)
}

impl PakaianDinasRepository {
    pub async fn get_pengajuan_satker_list(
        &self,
        pengajuan_id: Uuid,
        page: i32,
        per_page: i32,
        scope: &SatkerScope,
    ) -> AppResult<(Vec<PengajuanSatker>, i64)> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let offset = (page - 1) * per_page;

        // The COUNT carries the scope too. A scoped page beside an unscoped
        // total still leaks: it tells the caller exactly how many participants
        // they are not being shown.
        let mut count_params: Vec<BoxedParam> = vec![Box::new(pengajuan_id)];
        let count_scope = scope_and(scope, &mut count_params);
        let count_row = client
            .query_one(
                &format!(
                    "SELECT COUNT(*) as total \
                     FROM perlengkapan.pengajuan_pakaian_dinas_satker ps \
                     WHERE ps.pengajuan_id = $1{count_scope}"
                ),
                &as_refs(&count_params),
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let total: i64 = count_row.get("total");

        // LIMIT/OFFSET are bound BEFORE the scope so their $2/$3 stay put and
        // the scope predicate takes $4 — see the note on the shared helper for
        // why placeholder order is the part no type can check.
        let mut params: Vec<BoxedParam> = vec![
            Box::new(pengajuan_id),
            Box::new(per_page as i64),
            Box::new(offset as i64),
        ];
        let scope_sql = scope_and(scope, &mut params);
        let rows = client
            .query(
                &format!(
                    r#"
                SELECT ps.*, s.nama_satker as satker_nama, s.kode_satker as satker_kode,
                       (SELECT COUNT(*) FROM perlengkapan.pengajuan_pakaian_dinas_satker_pegawai WHERE pengajuan_satker_id = ps.id) as total_pegawai
                FROM perlengkapan.pengajuan_pakaian_dinas_satker ps
                LEFT JOIN integrasi.mysimkari_satker s ON ps.satker_id = s.kode_satker
                WHERE ps.pengajuan_id = $1{scope_sql}
                ORDER BY s.nama_satker ASC
                LIMIT $2 OFFSET $3
                "#
                ),
                &as_refs(&params),
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let items: Vec<PengajuanSatker> = rows.iter().map(PengajuanSatker::from_row).collect();
        Ok((items, total))
    }

    pub async fn get_pengajuan_satker_by_id(
        &self,
        id: Uuid,
        scope: &SatkerScope,
    ) -> AppResult<PengajuanSatker> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let mut params: Vec<BoxedParam> = vec![Box::new(id)];
        let scope_sql = scope_and(scope, &mut params);
        let row = client
            .query_opt(
                &format!(
                    r#"
                SELECT ps.*, s.nama_satker as satker_nama, s.kode_satker as satker_kode,
                       (SELECT COUNT(*) FROM perlengkapan.pengajuan_pakaian_dinas_satker_pegawai WHERE pengajuan_satker_id = ps.id) as total_pegawai
                FROM perlengkapan.pengajuan_pakaian_dinas_satker ps
                LEFT JOIN integrasi.mysimkari_satker s ON ps.satker_id = s.kode_satker
                WHERE ps.id = $1{scope_sql}
                "#
                ),
                &as_refs(&params),
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?
            // Out of scope answers the same NotFound as absent: telling them
            // apart confirms the row exists under another satker (#93).
            .ok_or_else(|| AppError::NotFound("Pengajuan satker tidak ditemukan".to_string()))?;

        Ok(PengajuanSatker::from_row(&row))
    }

    // ============ Employee Sizes ============
}
