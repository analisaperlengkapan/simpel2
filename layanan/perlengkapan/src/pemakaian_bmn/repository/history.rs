use super::PemakaianBmnRepository;
use crate::pemakaian_bmn::models::*;
use crate::shared::error::{AppError, AppResult};
use crate::shared::repo::PoolExt;
use crate::shared::satker_scope::{BoxedParam, SatkerScope};

/// `scope` as a trailing `AND` over this table's `satker_code` — see the note
/// on the shared helper it binds.
/// `alias` is mandatory since V014: these queries now join the permit to its
/// items, so two relations are in scope and an unqualified `satker_code` would
/// bind by Postgres' resolution order rather than by intent. The subquery
/// binds its OWN alias, which is the whole point — an outer binding there
/// would scope the current holder by the wrong row.
fn scope_and(scope: &SatkerScope, alias: &str, params: &mut Vec<BoxedParam>) -> String {
    crate::shared::satker_scope::scope_and(scope, &format!("{alias}.satker_code"), params)
}

impl PemakaianBmnRepository {
    /// Get permit history for a BMN, restricted to the caller's satker scope.
    ///
    /// A permit history is a named person's record of holding a named asset, so
    /// it is satker data, not reference data. Out of scope the answer is
    /// `NotFound` rather than `Forbidden`: 403 would confirm the NUP exists
    /// somewhere, which is the existence oracle #93 closed for satker detail.
    ///
    /// Note the aggregation is keyed on `bmn_nup` alone, which is not an asset
    /// identity — that takes kode satker + kode barang + NUP. Scoping narrows
    /// the collision but does NOT remove it, and the margin is smaller than
    /// that sentence used to imply: measured on the real SIMAN snapshot, 41,6%
    /// of (satker, NUP) pairs point at more than one kode barang, up to 425.
    /// So even an operator scoped to a single satker can see the histories of
    /// several different assets merged into one.
    ///
    /// Left keyed on NUP here on purpose: fixing it needs a UI decision first
    /// (how a cross-satker role names the asset it means), so it is tracked
    /// with the other read-side site listed in the `repository::lookup` module
    /// header rather than half-fixed here.
    /// Requirements: REQ-P012
    pub async fn get_bmn_usage_history(
        &self,
        bmn_nup: &str,
        scope: &SatkerScope,
    ) -> AppResult<BmnUsageStats> {
        let client = self.pool.client().await?;
        let mut params: Vec<BoxedParam> = vec![Box::new(bmn_nup.to_string())];
        // Two bindings, two params: the outer query narrows `p`, the current-
        // holder subquery narrows its own `p2`. Same value, stated twice, so
        // neither depends on name resolution reaching outward.
        let scope_sql = scope_and(scope, "p", &mut params);
        let scope_sub = scope_and(scope, "p2", &mut params);
        let refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let query = format!(
            r#"
            -- Counted over ITEMS since V014: an asset's history is the
            -- history of the rows naming it, and past the first asset those
            -- rows were invisible to the old parent-only query.
            SELECT
                i.bmn_nup,
                i.bmn_nama_barang,
                COUNT(*) as total_permits,
                COUNT(*) FILTER (WHERE p.status = 'ACTIVE') as active_permits,
                -- SUM over an int4 difference yields int8, not int4. Reading it
                -- as i32 panicked on the first row with any history at all.
                SUM(i.tanggal_selesai - i.tanggal_mulai)::bigint as total_days_used,
                (SELECT p2.pegawai_nama
                 FROM perlengkapan.pemakaian_bmn_items i2
                 JOIN perlengkapan.izin_pemakaian_bmn p2 ON p2.id = i2.izin_pemakaian_id
                 WHERE i2.bmn_nup = $1 AND p2.status = 'ACTIVE'{scope_sub} LIMIT 1) as current_holder
            FROM perlengkapan.pemakaian_bmn_items i
            JOIN perlengkapan.izin_pemakaian_bmn p ON p.id = i.izin_pemakaian_id
            WHERE i.bmn_nup = $1{scope_sql}
            GROUP BY i.bmn_nup, i.bmn_nama_barang
        "#
        );

        let row = client
            .query_opt(&query, &refs)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound(format!("No usage history for BMN: {}", bmn_nup)))?;

        // Get history entries. `id` breaks the created_at tie so paging and
        // repeat reads stay stable.
        let history_query = format!(
            r#"
            SELECT p.id, p.nomor_izin, i.tanggal_mulai, i.tanggal_selesai,
                   p.status, p.created_at
            FROM perlengkapan.pemakaian_bmn_items i
            JOIN perlengkapan.izin_pemakaian_bmn p ON p.id = i.izin_pemakaian_id
            WHERE i.bmn_nup = $1{scope_sql}
            ORDER BY p.created_at DESC, p.id ASC
            LIMIT 10
        "#
        );

        let history_rows = client
            .query(&history_query, &refs)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let permit_history = history_rows
            .into_iter()
            .map(|row| PermitHistoryEntry::from_row(&row))
            .collect();

        Ok(BmnUsageStats {
            bmn_nup: row.get("bmn_nup"),
            bmn_nama: row.get("bmn_nama_barang"),
            total_permits: row.get("total_permits"),
            active_permits: row.get("active_permits"),
            total_days_used: row.get::<_, Option<i64>>("total_days_used").unwrap_or(0),
            current_holder: row.get("current_holder"),
            permit_history,
        })
    }

    /// Get permit history for a pegawai, restricted to the caller's satker
    /// scope.
    ///
    /// Unscoped, this endpoint let anyone holding any monitoring role enumerate
    /// a named employee's asset history from their NIP alone.
    /// Requirements: REQ-P012
    pub async fn get_pegawai_usage_history(
        &self,
        pegawai_nip: &str,
        scope: &SatkerScope,
    ) -> AppResult<PegawaiUsageStats> {
        let client = self.pool.client().await?;
        let mut params: Vec<BoxedParam> = vec![Box::new(pegawai_nip.to_string())];
        let scope_sql = scope_and(scope, "p", &mut params);
        let refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let query = format!(
            r#"
            -- Permit-level: this counts permits, not assets, so V014 changed
            -- nothing here but the alias the scope binds to.
            SELECT
                p.pegawai_nip,
                p.pegawai_nama,
                COUNT(*) as total_permits,
                COUNT(*) FILTER (WHERE p.status = 'ACTIVE') as active_permits
            FROM perlengkapan.izin_pemakaian_bmn p
            WHERE p.pegawai_nip = $1{scope_sql}
            GROUP BY p.pegawai_nip, p.pegawai_nama
        "#
        );

        let row = client
            .query_opt(&query, &refs)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| {
                AppError::NotFound(format!("No usage history for pegawai: {}", pegawai_nip))
            })?;

        // Get history entries
        let history_query = format!(
            r#"
            SELECT p.id, p.nomor_izin, p.tanggal_mulai, p.tanggal_selesai,
                   p.status, p.created_at
            FROM perlengkapan.izin_pemakaian_bmn p
            WHERE p.pegawai_nip = $1{scope_sql}
            ORDER BY p.created_at DESC, p.id ASC
            LIMIT 10
        "#
        );

        let history_rows = client
            .query(&history_query, &refs)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let permit_history = history_rows
            .into_iter()
            .map(|row| PermitHistoryEntry::from_row(&row))
            .collect();

        Ok(PegawaiUsageStats {
            pegawai_nip: row.get("pegawai_nip"),
            pegawai_nama: row.get("pegawai_nama"),
            total_permits: row.get("total_permits"),
            active_permits: row.get("active_permits"),
            permit_history,
        })
    }
}
