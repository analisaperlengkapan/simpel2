//! `GET /admin/status-sinkronisasi`: are the master-data syncs running, and is anything wrong.
//!
//! Every form that takes a NIP or picks an asset reads data copied from MySIMKARI and SIMAN by
//! integrasi's CronJobs. When those stop, nothing here fails: lookups quietly answer from an
//! older copy, or find nobody. This endpoint is how an administrator sees that, without cluster
//! access.
//!
//! It reads `integrasi.sync_status`, which every run of the scheduler writes
//! (`layanan/integrasi/src/sync_run.rs`): `running` at the start with the CronJob's schedule and
//! next run time, then `completed` (and the rows now in the table this service reads) or `failed`
//! and the reason. integrasi's HTTP port used to have four unauthenticated monitoring endpoints
//! over `api_log`; they were removed because they could not see a run that failed before an HTTP
//! answer, and because this belongs behind the application's login and roles.

use axum::{Json, extract::State};
use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::shared::error::AppResult;
use crate::shared::middleware::Claims;
use lib_perlengkapan::response::ApiResponse;

/// A run is overdue when its next scheduled time is this far in the past and no newer run has
/// started. The CronJob controller starts a job within seconds; an hour leaves room for a node
/// that was busy, without hiding a schedule that did not fire.
pub const GRACE_MINUTES: i32 = 60;

/// One source and part (`module`/`endpoint` of `integrasi.sync_status`).
#[derive(Debug, Clone, Serialize)]
pub struct SyncStatusEntry {
    /// `MySIMKARI`, `SIMAN`, `MonSAKTI_ADM`.
    pub sumber: String,
    /// `satker`, `pegawai`, `aset`, … (what the row stands for).
    pub bagian: String,
    /// The judgement an administrator acts on, computed here so every reader agrees:
    /// `BELUM_PERNAH` (never ran), `BERJALAN` (running), `BERHASIL` (last run completed),
    /// `GAGAL` (last run failed), `TERLAMBAT` (the next scheduled run did not start).
    pub keadaan: String,
    /// `last_sync_status` as recorded: `pending`, `running`, `completed`, `failed`.
    pub status_terakhir: String,
    pub mulai_terakhir: Option<DateTime<Utc>>,
    pub selesai_terakhir: Option<DateTime<Utc>>,
    /// End of the last run that completed. After a failure this says how old the data is.
    pub berhasil_terakhir: Option<DateTime<Utc>>,
    /// The CronJob schedule, e.g. `0 */6 * * * Asia/Jakarta`.
    pub jadwal: Option<String>,
    pub jadwal_berikutnya: Option<DateTime<Utc>>,
    /// Rows in the table this service reads, counted after the last successful run.
    pub jumlah_data: i32,
    /// Why the last run failed. `None` after a success.
    pub galat: Option<String>,
    pub total_berhasil: i32,
    pub total_gagal: i32,
}

/// GET /admin/status-sinkronisasi (admin only)
pub async fn list_sync_status(
    State(pool): State<deadpool_postgres::Pool>,
    claims: Claims,
) -> AppResult<Json<ApiResponse<Vec<SyncStatusEntry>>>> {
    claims.require_admin()?;

    let client = pool.get().await?;
    let rows = client
        .query(
            "SELECT module, endpoint, \
                    CASE \
                      WHEN last_sync_started_at IS NULL THEN 'BELUM_PERNAH' \
                      WHEN next_sync_scheduled_at IS NOT NULL \
                           AND next_sync_scheduled_at + make_interval(mins => $1) < now() \
                        THEN 'TERLAMBAT' \
                      WHEN last_sync_status = 'running' THEN 'BERJALAN' \
                      WHEN last_sync_status = 'failed' THEN 'GAGAL' \
                      WHEN last_sync_status = 'completed' THEN 'BERHASIL' \
                      ELSE 'BELUM_PERNAH' \
                    END AS keadaan, \
                    COALESCE(last_sync_status, 'pending') AS status_terakhir, \
                    last_sync_started_at, last_sync_completed_at, last_success_at, \
                    sync_frequency, next_sync_scheduled_at, \
                    COALESCE(last_sync_records_count, 0) AS jumlah_data, \
                    last_sync_error, \
                    COALESCE(total_syncs_completed, 0) AS total_berhasil, \
                    COALESCE(total_syncs_failed, 0) AS total_gagal \
               FROM integrasi.sync_status \
              ORDER BY module, endpoint",
            &[&GRACE_MINUTES],
        )
        .await?;

    let entries: Vec<SyncStatusEntry> = rows
        .iter()
        .map(|r| SyncStatusEntry {
            sumber: r.get("module"),
            bagian: r.get("endpoint"),
            keadaan: r.get("keadaan"),
            status_terakhir: r.get("status_terakhir"),
            mulai_terakhir: r.get("last_sync_started_at"),
            selesai_terakhir: r.get("last_sync_completed_at"),
            berhasil_terakhir: r.get("last_success_at"),
            jadwal: r.get("sync_frequency"),
            jadwal_berikutnya: r.get("next_sync_scheduled_at"),
            jumlah_data: r.get("jumlah_data"),
            galat: r.get("last_sync_error"),
            total_berhasil: r.get("total_berhasil"),
            total_gagal: r.get("total_gagal"),
        })
        .collect();
    let count = entries.len();
    Ok(Json(ApiResponse::success(
        entries,
        format!("{count} sumber sinkronisasi"),
    )))
}
