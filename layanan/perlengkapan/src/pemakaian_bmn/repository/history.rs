use super::PemakaianBmnRepository;
use crate::pemakaian_bmn::models::*;
use crate::shared::error::{AppError, AppResult};
use crate::shared::repo::PoolExt;
use crate::shared::satker_scope::SatkerScope;

type BoxedParam = Box<dyn tokio_postgres::types::ToSql + Sync + Send>;

/// `scope` as an extra `AND` clause, plus its bind parameters.
///
/// Returns an empty string for the unrestricted tier so the caller can splice
/// it in unconditionally.
fn scope_and(scope: &SatkerScope, params: &mut Vec<BoxedParam>) -> String {
    match scope.push_condition("satker_code", params) {
        Some(cond) => format!(" AND {cond}"),
        None => String::new(),
    }
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
        let scope_sql = scope_and(scope, &mut params);
        let refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let query = format!(
            r#"
            SELECT
                bmn_nup,
                bmn_nama_barang,
                COUNT(*) as total_permits,
                COUNT(*) FILTER (WHERE status = 'ACTIVE') as active_permits,
                -- SUM over an int4 difference yields int8, not int4. Reading it
                -- as i32 panicked on the first row with any history at all.
                SUM(tanggal_selesai - tanggal_mulai)::bigint as total_days_used,
                (SELECT pegawai_nama FROM perlengkapan.izin_pemakaian_bmn
                 WHERE bmn_nup = $1 AND status = 'ACTIVE'{scope_sql} LIMIT 1) as current_holder
            FROM perlengkapan.izin_pemakaian_bmn
            WHERE bmn_nup = $1{scope_sql}
            GROUP BY bmn_nup, bmn_nama_barang
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
            SELECT id, nomor_izin, tanggal_mulai, tanggal_selesai, status, created_at
            FROM perlengkapan.izin_pemakaian_bmn
            WHERE bmn_nup = $1{scope_sql}
            ORDER BY created_at DESC, id ASC
            LIMIT 10
        "#
        );

        let history_rows = client
            .query(&history_query, &refs)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let permit_history = history_rows
            .into_iter()
            .map(|row| PermitHistoryEntry {
                id: row.get("id"),
                nomor_izin: row.get("nomor_izin"),
                tanggal_mulai: row.get("tanggal_mulai"),
                tanggal_selesai: row.get("tanggal_selesai"),
                status: row.get("status"),
                created_at: row.get("created_at"),
            })
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
        let scope_sql = scope_and(scope, &mut params);
        let refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let query = format!(
            r#"
            SELECT
                pegawai_nip,
                pegawai_nama,
                COUNT(*) as total_permits,
                COUNT(*) FILTER (WHERE status = 'ACTIVE') as active_permits
            FROM perlengkapan.izin_pemakaian_bmn
            WHERE pegawai_nip = $1{scope_sql}
            GROUP BY pegawai_nip, pegawai_nama
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
            SELECT id, nomor_izin, tanggal_mulai, tanggal_selesai, status, created_at
            FROM perlengkapan.izin_pemakaian_bmn
            WHERE pegawai_nip = $1{scope_sql}
            ORDER BY created_at DESC, id ASC
            LIMIT 10
        "#
        );

        let history_rows = client
            .query(&history_query, &refs)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let permit_history = history_rows
            .into_iter()
            .map(|row| PermitHistoryEntry {
                id: row.get("id"),
                nomor_izin: row.get("nomor_izin"),
                tanggal_mulai: row.get("tanggal_mulai"),
                tanggal_selesai: row.get("tanggal_selesai"),
                status: row.get("status"),
                created_at: row.get("created_at"),
            })
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
