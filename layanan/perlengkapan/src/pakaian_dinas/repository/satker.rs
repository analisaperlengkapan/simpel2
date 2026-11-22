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

/// The `AND` clauses for `filter`, binding their parameters onto `params` as it goes.
///
/// Called before [`scope_and`] on every query that uses both, so the scope's `$n` is the
/// next free one whatever the filter happened to add. The search looks at the satker's
/// name and its code (what an operator knows it by).
fn filter_and(filter: &SatkerListFilter, params: &mut Vec<BoxedParam>) -> String {
    let mut sql = String::new();
    if let Some(pattern) = &filter.q_pattern {
        params.push(Box::new(pattern.clone()));
        let n = params.len();
        sql.push_str(&format!(
            " AND (s.nama_satker ILIKE ${n} OR ps.satker_id ILIKE ${n})"
        ));
    }
    if !filter.statuses.is_empty() {
        params.push(Box::new(filter.statuses.clone()));
        let n = params.len();
        sql.push_str(&format!(" AND ps.aktivitas_id = ANY(${n})"));
    }
    sql
}

impl PakaianDinasRepository {
    /// How many satker rows stand at one of `statuses`, inside the caller's scope.
    /// What the dashboard's "Perlu Tindakan" shows for pakaian dinas: the rows
    /// waiting on the caller's role.
    pub async fn hitung_menunggu(&self, statuses: &[i32], scope: &SatkerScope) -> AppResult<i64> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let mut params: Vec<BoxedParam> = vec![Box::new(statuses.to_vec())];
        let scope_sql = scope_and(scope, &mut params);
        let sql = format!(
            "SELECT COUNT(*) FROM perlengkapan.pengajuan_pakaian_dinas_satker ps \
             WHERE ps.aktivitas_id = ANY($1){scope_sql}"
        );
        let row = client
            .query_one(&sql, &as_refs(&params))
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        Ok(row.get(0))
    }

    pub async fn get_pengajuan_satker_list(
        &self,
        pengajuan_id: Uuid,
        page: i32,
        per_page: i32,
        filter: &SatkerListFilter,
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
        //
        // The filter's parameters are bound BEFORE the scope in both queries, for the
        // reason below: the scope takes whatever `$n` comes next.
        let mut count_params: Vec<BoxedParam> = vec![Box::new(pengajuan_id)];
        let count_filter = filter_and(filter, &mut count_params);
        let count_scope = scope_and(scope, &mut count_params);
        let count_row = client
            .query_one(
                &format!(
                    "SELECT COUNT(*) as total \
                     FROM perlengkapan.pengajuan_pakaian_dinas_satker ps \
                     LEFT JOIN integrasi.mysimkari_satker s ON ps.satker_id = s.kode_satker \
                     WHERE ps.pengajuan_id = $1{count_filter}{count_scope}"
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
        let filter_sql = filter_and(filter, &mut params);
        let scope_sql = scope_and(scope, &mut params);
        let rows = client
            .query(
                &format!(
                    r#"
                SELECT ps.*, s.nama_satker as satker_nama, s.kode_satker as satker_kode,
                       (SELECT COUNT(*) FROM perlengkapan.pengajuan_pakaian_dinas_satker_pegawai WHERE pengajuan_satker_id = ps.id) as total_pegawai
                FROM perlengkapan.pengajuan_pakaian_dinas_satker ps
                LEFT JOIN integrasi.mysimkari_satker s ON ps.satker_id = s.kode_satker
                WHERE ps.pengajuan_id = $1{filter_sql}{scope_sql}
                ORDER BY s.nama_satker ASC NULLS LAST, ps.satker_id ASC
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
