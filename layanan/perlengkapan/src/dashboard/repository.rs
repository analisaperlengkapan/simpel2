// Dashboard repository for database queries

use crate::dashboard::models::*;
use crate::shared::error::AppError;
use deadpool_postgres::Pool;
use std::collections::HashMap;

/// Fetch kebutuhan metrics from database
pub async fn fetch_kebutuhan_metrics(
    db_pool: &Pool,
    params: &DashboardParams,
) -> Result<KebutuhanMetrics, AppError> {
    let client = db_pool.get().await?;

    // Total by status. The unit of "a kebutuhan" is the PER-SATKER response
    // (`pengajuan_kebutuhan_bmn_satker`), not the campaign — the campaign carries
    // its own separate status. The human label comes from `ms_workflow_status`
    // rather than being hard-coded, so it cannot drift from the seed.
    let status_query = r#"
        SELECT COALESCE(w.nama, 'Kode ' || ps.status_kode::text) AS status,
               COUNT(*) AS count
        FROM perlengkapan.pengajuan_kebutuhan_bmn_satker ps
        JOIN perlengkapan.pengajuan_kebutuhan_bmn p ON p.id = ps.pengajuan_id
        LEFT JOIN perlengkapan.ms_workflow_status w
               ON w.modul = 'kebutuhan_bmn' AND w.kode = ps.status_kode
        WHERE p.tahun = $1
        GROUP BY status
    "#;

    let rows = client
        .query(status_query, &[&params.tahun_anggaran])
        .await?;

    let total_by_status: HashMap<String, i64> = rows
        .into_iter()
        .map(|row| (row.get("status"), row.get("count")))
        .collect();

    // Total by satker (top 10). `ps.satker_id` holds the MySIMKARI `kode_satker`,
    // so the name resolves from the SoT (`integrasi.mysimkari_satker`) — NOT from
    // `authenc.satkers`, which is an IAM read-model and whose column is `name`
    // anyway (the old query selected a non-existent `s.nama`).
    let satker_query = r#"
        SELECT ps.satker_id,
               COALESCE(ms.nama_satker, ps.satker_nama, ps.satker_id) AS satker_nama,
               COUNT(*) AS count
        FROM perlengkapan.pengajuan_kebutuhan_bmn_satker ps
        JOIN perlengkapan.pengajuan_kebutuhan_bmn p ON p.id = ps.pengajuan_id
        LEFT JOIN integrasi.mysimkari_satker ms ON ms.kode_satker = ps.satker_id
        WHERE p.tahun = $1
        -- Group by the SOURCE columns, not the output alias: `satker_nama` is
        -- also an input column here (ps.satker_nama), so `GROUP BY satker_nama`
        -- binds to the input and leaves ms.nama_satker ungrouped.
        GROUP BY ps.satker_id, ms.nama_satker, ps.satker_nama
        ORDER BY count DESC
        LIMIT 10
    "#;

    let rows = client
        .query(satker_query, &[&params.tahun_anggaran])
        .await?;

    let total_by_satker: Vec<SatkerCount> = rows
        .into_iter()
        .map(|row| SatkerCount {
            satker_id: row.get("satker_id"),
            satker_nama: row.get("satker_nama"),
            count: row.get("count"),
        })
        .collect();

    // Total by tahun (last 5 years), counted over per-satker responses.
    let tahun_query = r#"
        SELECT p.tahun AS tahun_anggaran, COUNT(*) AS count
        FROM perlengkapan.pengajuan_kebutuhan_bmn_satker ps
        JOIN perlengkapan.pengajuan_kebutuhan_bmn p ON p.id = ps.pengajuan_id
        GROUP BY p.tahun
        ORDER BY p.tahun DESC
        LIMIT 5
    "#;

    let rows = client.query(tahun_query, &[]).await?;

    let total_by_tahun: HashMap<i32, i64> = rows
        .into_iter()
        .map(|row| (row.get("tahun_anggaran"), row.get("count")))
        .collect();

    Ok(KebutuhanMetrics {
        total_by_status,
        total_by_satker,
        total_by_tahun,
    })
}

/// Fetch gap analysis (top N asset types with largest gaps)
pub async fn fetch_gap_analysis(
    db_pool: &Pool,
    limit: i64,
) -> Result<Vec<GapAnalysisResult>, AppError> {
    let client = db_pool.get().await?;

    // Requested items live on `..._satker_barang` (nama / kode_barang / jumlah);
    // the year filter has to climb to the campaign. Aggregated per kode_barang so
    // one row per asset type, which is what "top N gaps" means.
    let query = r#"
        WITH requested AS (
            SELECT b.kode_barang,
                   MIN(b.nama) AS nama_barang,
                   SUM(b.jumlah)::bigint AS standard_quantity
            FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_barang b
            JOIN perlengkapan.pengajuan_kebutuhan_bmn_satker ps ON ps.id = b.pengajuan_satker_id
            JOIN perlengkapan.pengajuan_kebutuhan_bmn p ON p.id = ps.pengajuan_id
            WHERE p.tahun = EXTRACT(YEAR FROM CURRENT_DATE)::int
              AND b.kode_barang IS NOT NULL
            GROUP BY b.kode_barang
        )
        SELECT
            r.kode_barang,
            r.nama_barang,
            r.standard_quantity,
            COALESCE(g.good_count, 0) AS existing_good_quantity,
            r.standard_quantity - COALESCE(g.good_count, 0) AS gap
        FROM requested r
        LEFT JOIN LATERAL (
            SELECT COUNT(*)::bigint AS good_count
            FROM integrasi.siman_aset sa
            WHERE sa.kode_barang = r.kode_barang
              AND sa.kondisi = 'BAIK'
        ) g ON TRUE
        ORDER BY gap DESC
        LIMIT $1
    "#;

    let rows = client.query(query, &[&limit]).await?;

    let gap_analysis: Vec<GapAnalysisResult> = rows
        .into_iter()
        .map(|row| GapAnalysisResult {
            kode_barang: row.get("kode_barang"),
            nama_barang: row.get("nama_barang"),
            // SUM()/COUNT() are int8; the DTO fields are i32, so read as i64 and
            // narrow explicitly. Reading an int8 column straight into i32 panics
            // (the same trap as MysimkariPegawai::id in #94).
            standard_quantity: row.get::<_, i64>("standard_quantity") as i32,
            existing_good_quantity: row.get::<_, i64>("existing_good_quantity") as i32,
            gap: row.get::<_, i64>("gap") as i32,
        })
        .collect();

    Ok(gap_analysis)
}

/// Fetch pakaian dinas metrics
pub async fn fetch_pakaian_dinas_metrics(
    db_pool: &Pool,
    params: &DashboardParams,
) -> Result<PakaianDinasMetrics, AppError> {
    let client = db_pool.get().await?;

    // Total by jenis — the campaign references the master by id; the readable
    // name lives on `ms_jenis_pakaian_dinas`.
    let jenis_query = r#"
        SELECT COALESCE(j.nama, '(tanpa jenis)') AS jenis_pakaian, COUNT(*) AS count
        FROM perlengkapan.pengajuan_pakaian_dinas p
        LEFT JOIN perlengkapan.ms_jenis_pakaian_dinas j ON j.id = p.jenis_pakaian_dinas_id
        WHERE p.tahun = $1
        GROUP BY jenis_pakaian
    "#;

    let rows = client.query(jenis_query, &[&params.tahun_anggaran]).await?;

    let total_by_jenis: HashMap<String, i64> = rows
        .into_iter()
        .map(|row| (row.get("jenis_pakaian"), row.get("count")))
        .collect();

    // Total by ukuran — sizes are recorded per employee per garment, so this
    // climbs pegawai_ukuran -> satker -> campaign to reach the year.
    let ukuran_query = r#"
        SELECT u.ukuran, COUNT(*) AS count
        FROM perlengkapan.pengajuan_pakaian_dinas_satker_pegawai_ukuran u
        JOIN perlengkapan.pengajuan_pakaian_dinas_satker ps ON ps.id = u.pengajuan_satker_id
        JOIN perlengkapan.pengajuan_pakaian_dinas p ON p.id = ps.pengajuan_id
        WHERE p.tahun = $1 AND u.ukuran IS NOT NULL
        GROUP BY u.ukuran
    "#;

    let rows = client
        .query(ukuran_query, &[&params.tahun_anggaran])
        .await?;

    let total_by_ukuran: HashMap<String, i64> = rows
        .into_iter()
        .map(|row| (row.get("ukuran"), row.get("count")))
        .collect();

    Ok(PakaianDinasMetrics {
        total_by_jenis,
        total_by_ukuran,
    })
}

/// Fetch workflow performance metrics
pub async fn fetch_workflow_metrics(db_pool: &Pool) -> Result<WorkflowMetrics, AppError> {
    let client = db_pool.get().await?;

    // Average processing time for completed per-satker responses. "Completed" is
    // status_kode 2008 (models/status.rs). The old query filtered a TEXT
    // `status = 'COMPLETED'` that exists nowhere in this schema.
    let avg_time_query = r#"
        SELECT COALESCE(AVG(EXTRACT(EPOCH FROM (updated_at - created_at)) / 3600), 0) AS avg_hours
        FROM perlengkapan.pengajuan_kebutuhan_bmn_satker
        WHERE status_kode = 2008
    "#;

    let row = client.query_one(avg_time_query, &[]).await?;
    let average_processing_time_hours: f64 = row.get("avg_hours");

    // Bottlenecks (states with longest average time)
    // This is a simplified version - in production, you'd use window functions
    let bottleneck_query = r#"
        SELECT
            -- The activity row records the workflow STATE it moved into
            -- (`to_status_kode`); it has no `aktivitas_id`, and this table has no
            -- relation to `ms_aktivitas_bmn` at all — the old query joined both,
            -- so it errored before it could return anything. Labels come from
            -- ms_workflow_status; the code is cast to text because
            -- BottleneckInfo.state is a String and the kode column is INTEGER.
            COALESCE(w.nama, 'Kode ' || ka.to_status_kode::text) AS state,
            COALESCE(AVG(EXTRACT(EPOCH FROM (NOW() - ka.created_at)) / 3600), 0) as avg_hours,
            COUNT(*) as count
        FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas ka
        LEFT JOIN perlengkapan.ms_workflow_status w
               ON w.modul = 'kebutuhan_bmn' AND w.kode = ka.to_status_kode
        WHERE ka.created_at > NOW() - INTERVAL '30 days'
        GROUP BY w.nama, ka.to_status_kode
        HAVING AVG(EXTRACT(EPOCH FROM (NOW() - ka.created_at)) / 3600) > 24
        ORDER BY avg_hours DESC
        LIMIT 5
    "#;

    let rows = client.query(bottleneck_query, &[]).await?;

    let bottlenecks: Vec<BottleneckInfo> = rows
        .into_iter()
        .map(|row| BottleneckInfo {
            state: row.get("state"),
            average_time_hours: row.get("avg_hours"),
            count: row.get("count"),
        })
        .collect();

    // SLA breaches today (older than 2 days and still in a non-terminal state).
    // Terminality is read from `ms_workflow_status.is_terminal` instead of a
    // hard-coded name list: the old query filtered 'COMPLETED'/'ARCHIVED'/
    // 'CANCELLED'/'REJECTED', none of which are values this system ever stores
    // (the states are integer codes 2000-2010).
    let sla_breach_query = r#"
        SELECT COUNT(*) AS count
        FROM perlengkapan.pengajuan_kebutuhan_bmn_satker ps
        LEFT JOIN perlengkapan.ms_workflow_status w
               ON w.modul = 'kebutuhan_bmn' AND w.kode = ps.status_kode
        WHERE ps.created_at < NOW() - INTERVAL '2 days'
          AND COALESCE(w.is_terminal, FALSE) = FALSE
    "#;

    let row = client.query_one(sla_breach_query, &[]).await?;
    let sla_breaches_today: i64 = row.get("count");

    Ok(WorkflowMetrics {
        average_processing_time_hours,
        bottlenecks,
        sla_breaches_today,
    })
}

/// Fetch asset utilization from SIMAN data
pub async fn fetch_asset_utilization(db_pool: &Pool) -> Result<AssetUtilization, AppError> {
    let client = db_pool.get().await?;

    let query = r#"
        SELECT
            COUNT(*) as total_assets,
            SUM(CASE WHEN kondisi = 'BAIK' THEN 1 ELSE 0 END) as assets_in_good_condition
        FROM integrasi.siman_aset
    "#;

    let row = client.query_one(query, &[]).await?;

    let total_assets: i64 = row.get("total_assets");
    let assets_in_good_condition: i64 = row.get("assets_in_good_condition");

    let utilization_percentage = if total_assets > 0 {
        (assets_in_good_condition as f64 / total_assets as f64) * 100.0
    } else {
        0.0
    };

    Ok(AssetUtilization {
        total_assets,
        assets_in_good_condition,
        utilization_percentage,
    })
}

/// Rekap status Pemakaian BMN (Fase 2.7) — COUNT per `status`.
pub async fn fetch_pemakaian_status_metrics(
    db_pool: &Pool,
) -> Result<ModuleStatusMetrics, AppError> {
    let client = db_pool.get().await?;
    let rows = client
        .query(
            "SELECT status, COUNT(*) AS count
             FROM perlengkapan.izin_pemakaian_bmn
             GROUP BY status",
            &[],
        )
        .await?;
    Ok(rows_to_status_metrics(rows))
}

/// Rekap status Usulan SK Penghapusan BMN (Fase 2.7) — COUNT per `status`.
pub async fn fetch_penghapusan_status_metrics(
    db_pool: &Pool,
) -> Result<ModuleStatusMetrics, AppError> {
    let client = db_pool.get().await?;
    let rows = client
        .query(
            "SELECT status, COUNT(*) AS count
             FROM perlengkapan.penghapusan_bmn
             GROUP BY status",
            &[],
        )
        .await?;
    Ok(rows_to_status_metrics(rows))
}

/// Map baris (status, count) menjadi `ModuleStatusMetrics`.
fn rows_to_status_metrics(rows: Vec<tokio_postgres::Row>) -> ModuleStatusMetrics {
    let mut total_by_status: HashMap<String, i64> = HashMap::new();
    let mut total: i64 = 0;
    for row in rows {
        let status: String = row.get("status");
        let count: i64 = row.get("count");
        total += count;
        total_by_status.insert(status, count);
    }
    ModuleStatusMetrics {
        total_by_status,
        total,
    }
}

/// Fetch the lightweight SIMAN summary card (`/dashboard/stats`).
pub async fn fetch_dashboard_stats(db_pool: &Pool) -> Result<DashboardStats, AppError> {
    let client = db_pool
        .get()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to get database connection: {}", e)))?;

    // Query v_siman_summary_total with explicit casts to be safe.
    let summary = client
        .query_one(
            "SELECT
                total_aset,
                COALESCE(total_nilai_perolehan, 0)::FLOAT8 as total_nilai,
                total_satker,
                total_baik,
                total_rusak
              FROM integrasi.v_siman_summary_total",
            &[],
        )
        .await
        .map_err(|e| AppError::Database(format!("Failed to query summary with cast: {}", e)))?;

    let total_aset: i64 = summary.get("total_aset");
    let total_nilai_aset: f64 = summary.get("total_nilai");
    let total_satker: i64 = summary.get("total_satker");
    let aset_baik: i64 = summary.get("total_baik");
    let aset_rusak: i64 = summary.get("total_rusak");

    let cat_rows = client
        .query(
            "SELECT kategori_aset, total_aset, COALESCE(total_nilai_perolehan, 0)::FLOAT8 as total_nilai FROM integrasi.v_siman_summary_per_kategori ORDER BY total_aset DESC",
            &[],
        )
        .await
        .map_err(|e| AppError::Database(format!("Failed to query categories: {}", e)))?;

    let categories = cat_rows
        .iter()
        .map(|row| CategoryStat {
            category: row.get("kategori_aset"),
            count: row.get("total_aset"),
            value: row.get("total_nilai"),
        })
        .collect();

    Ok(DashboardStats {
        total_aset,
        total_nilai_aset,
        total_satker,
        aset_baik,
        aset_rusak,
        categories,
    })
}
