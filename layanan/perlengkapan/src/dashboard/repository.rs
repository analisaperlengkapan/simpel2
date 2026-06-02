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

    // Total by status
    let status_query = r#"
        SELECT status, COUNT(*) as count
        FROM perlengkapan.kebutuhan_bmn
        WHERE tahun_anggaran = $1
        GROUP BY status
    "#;

    let rows = client
        .query(status_query, &[&params.tahun_anggaran])
        .await?;

    let total_by_status: HashMap<String, i64> = rows
        .into_iter()
        .map(|row| (row.get("status"), row.get("count")))
        .collect();

    // Total by satker (top 10)
    let satker_query = r#"
        SELECT k.satker_id, s.nama as satker_nama, COUNT(*) as count
        FROM perlengkapan.kebutuhan_bmn k
        JOIN authenc.satkers s ON k.satker_id = s.id
        WHERE k.tahun_anggaran = $1
        GROUP BY k.satker_id, s.nama
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

    // Total by tahun (last 5 years)
    let tahun_query = r#"
        SELECT tahun_anggaran, COUNT(*) as count
        FROM perlengkapan.kebutuhan_bmn
        GROUP BY tahun_anggaran
        ORDER BY tahun_anggaran DESC
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

    let query = r#"
        SELECT
            k.kode_barang,
            k.nama_barang,
            k.jumlah_kebutuhan AS standard_quantity,
            COALESCE(
                (SELECT COUNT(*)
                 FROM integrasi.siman_aset_tanah sa
                 WHERE sa.kode_barang = k.kode_barang
                   AND sa.kondisi = 'BAIK'),
                0
            ) AS existing_good_quantity,
            k.jumlah_kebutuhan - COALESCE(
                (SELECT COUNT(*)
                 FROM integrasi.siman_aset_tanah sa
                 WHERE sa.kode_barang = k.kode_barang
                   AND sa.kondisi = 'BAIK'),
                0
            ) AS gap
        FROM perlengkapan.kebutuhan_bmn k
        WHERE k.tahun_anggaran = EXTRACT(YEAR FROM CURRENT_DATE)
        ORDER BY gap DESC
        LIMIT $1
    "#;

    let rows = client.query(query, &[&limit]).await?;

    let gap_analysis: Vec<GapAnalysisResult> = rows
        .into_iter()
        .map(|row| GapAnalysisResult {
            kode_barang: row.get("kode_barang"),
            nama_barang: row.get("nama_barang"),
            standard_quantity: row.get("standard_quantity"),
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

    // Total by jenis
    let jenis_query = r#"
        SELECT jenis_pakaian, COUNT(*) as count
        FROM perlengkapan.pakaian_dinas
        WHERE tahun_anggaran = $1
        GROUP BY jenis_pakaian
    "#;

    let rows = client.query(jenis_query, &[&params.tahun_anggaran]).await?;

    let total_by_jenis: HashMap<String, i64> = rows
        .into_iter()
        .map(|row| (row.get("jenis_pakaian"), row.get("count")))
        .collect();

    // Total by ukuran
    let ukuran_query = r#"
        SELECT ukuran, COUNT(*) as count
        FROM perlengkapan.pakaian_dinas
        WHERE tahun_anggaran = $1
        GROUP BY ukuran
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

    // Average processing time for completed items
    let avg_time_query = r#"
        SELECT COALESCE(AVG(EXTRACT(EPOCH FROM (updated_at - created_at)) / 3600), 0) as avg_hours
        FROM perlengkapan.kebutuhan_bmn
        WHERE status = 'COMPLETED'
    "#;

    let row = client.query_one(avg_time_query, &[]).await?;
    let average_processing_time_hours: f64 = row.get("avg_hours");

    // Bottlenecks (states with longest average time)
    // This is a simplified version - in production, you'd use window functions
    let bottleneck_query = r#"
        SELECT
            a.kode as state,
            COALESCE(AVG(EXTRACT(EPOCH FROM (NOW() - ka.created_at)) / 3600), 0) as avg_hours,
            COUNT(*) as count
        FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas ka
        JOIN perlengkapan.ms_aktivitas_bmn a ON ka.aktivitas_id = a.id
        WHERE ka.created_at > NOW() - INTERVAL '30 days'
        GROUP BY a.kode
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

    // SLA breaches today (items older than 2 days in non-terminal states)
    let sla_breach_query = r#"
        SELECT COUNT(*) as count
        FROM perlengkapan.kebutuhan_bmn k
        WHERE k.created_at < NOW() - INTERVAL '2 days'
          AND k.status NOT IN ('COMPLETED', 'ARCHIVED', 'CANCELLED', 'REJECTED')
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
        FROM integrasi.siman_aset_tanah
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
