// Dashboard repository for database queries

use crate::dashboard::models::*;
use crate::shared::error::AppError;
// Which half of `integrasi.siman_aset` the SIMAN ingest actually writes, and in
// what format. Shared with bank_aset and pemakaian_bmn so a fix in one place
// cannot leave the others reading the dead half — which is exactly what
// happened here after the condition column was corrected but `kode_barang`
// and `kategori_aset` were not.
use crate::bank_aset::AsetScope;
use crate::shared::satker_scope::{BoxedParam, SatkerScope, as_refs, scope_and};
use crate::shared::siman_columns::{ASSET_KODE_BARANG_SQL, ASSET_KONDISI_BAIK_PREDICATE};
use deadpool_postgres::Pool;
use std::collections::HashMap;

/// `AsetScope` as an extra `AND` over `integrasi.siman_aset.kdsatker_keu`.
///
/// Sibling of [`scope_and`], which does the same for the MySIMKARI-keyed
/// workflow tables. Two scopes are needed on this one module because the
/// dashboard aggregates BOTH sides: perlengkapan's own workflow rows (keyed by
/// MySIMKARI `kode_satker`) and SIMAN assets (keyed by the disjoint finance
/// code `kdsatker_keu`). Filtering one and not the other is how a "scoped"
/// dashboard still reports 624 533 national assets.
fn aset_and(scope: &AsetScope, params: &mut Vec<BoxedParam>) -> String {
    match scope.push_condition(params) {
        Some(cond) => format!(" AND {cond}"),
        None => String::new(),
    }
}

/// `AsetScope` as a complete `WHERE` clause (empty for the unrestricted tier).
fn aset_where(scope: &AsetScope, params: &mut Vec<BoxedParam>) -> String {
    match scope.push_condition(params) {
        Some(cond) => format!(" WHERE {cond}"),
        None => String::new(),
    }
}

/// Fetch kebutuhan metrics from database
pub async fn fetch_kebutuhan_metrics(
    db_pool: &Pool,
    params: &DashboardParams,
    scope: &SatkerScope,
) -> Result<KebutuhanMetrics, AppError> {
    let client = db_pool.get().await?;

    // Total by status. The unit of "a kebutuhan" is the PER-SATKER response
    // (`pengajuan_kebutuhan_bmn_satker`), not the campaign — the campaign carries
    // its own separate status. The human label comes from `ms_workflow_status`
    // rather than being hard-coded, so it cannot drift from the seed.
    let mut sp: Vec<BoxedParam> = vec![Box::new(params.tahun_anggaran)];
    let sp_scope = scope_and(scope, "ps.satker_id", &mut sp);
    let status_query = format!(
        r#"
        SELECT COALESCE(w.nama, 'Kode ' || ps.status_kode::text) AS status,
               COUNT(*) AS count
        FROM perlengkapan.pengajuan_kebutuhan_bmn_satker ps
        JOIN perlengkapan.pengajuan_kebutuhan_bmn p ON p.id = ps.pengajuan_id
        LEFT JOIN perlengkapan.ms_workflow_status w
               ON w.modul = 'kebutuhan_bmn' AND w.kode = ps.status_kode
        WHERE p.tahun = $1{sp_scope}
        GROUP BY status
    "#
    );

    let rows = client.query(status_query.as_str(), &as_refs(&sp)).await?;

    let total_by_status: HashMap<String, i64> = rows
        .into_iter()
        .map(|row| (row.get("status"), row.get("count")))
        .collect();

    // Total by satker (top 10). `ps.satker_id` holds the MySIMKARI `kode_satker`,
    // so the name resolves from the SoT (`integrasi.mysimkari_satker`) — NOT from
    // `authenc.satkers`, which is an IAM read-model and whose column is `name`
    // anyway (the old query selected a non-existent `s.nama`).
    let mut kp: Vec<BoxedParam> = vec![Box::new(params.tahun_anggaran)];
    let kp_scope = scope_and(scope, "ps.satker_id", &mut kp);
    let satker_query = format!(
        r#"
        SELECT ps.satker_id,
               COALESCE(ms.nama_satker, ps.satker_nama, ps.satker_id) AS satker_nama,
               COUNT(*) AS count
        FROM perlengkapan.pengajuan_kebutuhan_bmn_satker ps
        JOIN perlengkapan.pengajuan_kebutuhan_bmn p ON p.id = ps.pengajuan_id
        LEFT JOIN integrasi.mysimkari_satker ms ON ms.kode_satker = ps.satker_id
        WHERE p.tahun = $1{kp_scope}
        -- Group by the SOURCE columns, not the output alias: `satker_nama` is
        -- also an input column here (ps.satker_nama), so `GROUP BY satker_nama`
        -- binds to the input and leaves ms.nama_satker ungrouped.
        GROUP BY ps.satker_id, ms.nama_satker, ps.satker_nama
        ORDER BY count DESC
        LIMIT 10
    "#
    );

    let rows = client.query(satker_query.as_str(), &as_refs(&kp)).await?;

    let total_by_satker: Vec<SatkerCount> = rows
        .into_iter()
        .map(|row| SatkerCount {
            satker_id: row.get("satker_id"),
            satker_nama: row.get("satker_nama"),
            count: row.get("count"),
        })
        .collect();

    // Total by tahun (last 5 years), counted over per-satker responses.
    let mut tp: Vec<BoxedParam> = Vec::new();
    // `WHERE TRUE` so the scope can be spliced in unconditionally: this query
    // has no filter of its own, and the unrestricted tier contributes the empty
    // string.
    let tp_scope = scope_and(scope, "ps.satker_id", &mut tp);
    let tahun_query = format!(
        r#"
        SELECT p.tahun AS tahun_anggaran, COUNT(*) AS count
        FROM perlengkapan.pengajuan_kebutuhan_bmn_satker ps
        JOIN perlengkapan.pengajuan_kebutuhan_bmn p ON p.id = ps.pengajuan_id
        WHERE TRUE{tp_scope}
        GROUP BY p.tahun
        ORDER BY p.tahun DESC
        LIMIT 5
    "#
    );

    let rows = client.query(tahun_query.as_str(), &as_refs(&tp)).await?;

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
    scope: &SatkerScope,
    aset_scope: &AsetScope,
) -> Result<Vec<GapAnalysisResult>, AppError> {
    let client = db_pool.get().await?;

    // Requested items live on `..._satker_barang` (nama / kode_barang / jumlah);
    // the year filter has to climb to the campaign. Aggregated per kode_barang so
    // one row per asset type, which is what "top N gaps" means.
    //
    // The SIMAN side had TWO independent reasons to match nothing, so every gap
    // came back equal to the full requested quantity — the exact symptom the
    // condition-column comment below says was already fixed:
    //
    //   1. `sa.kode_barang` is populated in 0 of 624 533 rows. The ingest derives
    //      its INSERT columns from the SIMAN payload's keys
    //      (`layanan/integrasi/src/db.rs`), and SIMAN sends `kd_brg`.
    //   2. FORMAT: `kd_brg` is ten digits with NO dots (3050201002) while the
    //      request side stores the dotted presentation form (3.05.02.01.002), so
    //      fixing the column alone still matches nothing. Both sides are
    //      normalised at comparison time.
    //
    // Measured on staging: 0 -> 36 787 and 1 902 existing good assets for the
    // two requested codes that have any.
    //
    // Shape change: the per-request `LEFT JOIN LATERAL` scanned all 624 533 rows
    // ONCE PER REQUESTED CODE — fine while the predicate matched nothing and an
    // index on the all-NULL `kode_barang` answered instantly, but ~1.9 s for
    // three codes once it started matching, growing linearly with the campaign.
    // Pre-aggregating SIMAN once and joining costs one scan regardless: 776 ms
    // for the same result.
    // Both halves are scoped, and both must be: the requested side is keyed by
    // MySIMKARI `kode_satker`, the SIMAN side by the disjoint finance code, so
    // one scope cannot cover the other. `LIMIT` binds LAST because the scope
    // predicates occupy the earlier placeholders.
    let mut params: Vec<BoxedParam> = Vec::new();
    let req_scope = scope_and(scope, "ps.satker_id", &mut params);
    let good_scope = aset_and(aset_scope, &mut params);
    params.push(Box::new(limit));
    let limit_idx = params.len();

    let query = format!(
        r#"
        WITH requested AS (
            SELECT b.kode_barang,
                   MIN(b.nama) AS nama_barang,
                   SUM(b.jumlah)::bigint AS standard_quantity
            FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_barang b
            JOIN perlengkapan.pengajuan_kebutuhan_bmn_satker ps ON ps.id = b.pengajuan_satker_id
            JOIN perlengkapan.pengajuan_kebutuhan_bmn p ON p.id = ps.pengajuan_id
            WHERE p.tahun = EXTRACT(YEAR FROM CURRENT_DATE)::int
              AND b.kode_barang IS NOT NULL{req_scope}
            GROUP BY b.kode_barang
        ),
        good AS (
            -- Both sides are normalised, but only the REQUEST side is load-
            -- bearing today: `kd_brg` holds undotted digits in 624 528 of
            -- 624 533 rows, so stripping dots here is a no-op and reverting it
            -- does NOT turn the test red (unlike the other three halves, which
            -- do). It is kept anyway rather than trimmed as dead weight,
            -- because dotted values HAVE occurred in this column — 5 rows on
            -- staging carry them — so this defends against a format the column
            -- has actually held, not a hypothetical one.
            SELECT replace({kode_barang}, '.', '') AS kb,
                   COUNT(*)::bigint AS good_count
            FROM integrasi.siman_aset
            -- see fetch_asset_utilization: `kondisi` alone is NULL in every
            -- environment fed by ur_kondisi, which silently made every gap
            -- equal to the full requested quantity.
            WHERE {kondisi_baik}{good_scope}
            GROUP BY 1
        )
        SELECT
            r.kode_barang,
            r.nama_barang,
            r.standard_quantity,
            COALESCE(g.good_count, 0) AS existing_good_quantity,
            r.standard_quantity - COALESCE(g.good_count, 0) AS gap
        FROM requested r
        LEFT JOIN good g ON g.kb = replace(r.kode_barang, '.', '')
        ORDER BY gap DESC
        LIMIT ${limit_idx}
    "#,
        kode_barang = ASSET_KODE_BARANG_SQL,
        kondisi_baik = ASSET_KONDISI_BAIK_PREDICATE
    );

    let rows = client.query(query.as_str(), &as_refs(&params)).await?;

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
    scope: &SatkerScope,
) -> Result<PakaianDinasMetrics, AppError> {
    let client = db_pool.get().await?;

    // Total by jenis — the campaign references the master by id; the readable
    // name lives on `ms_jenis_pakaian_dinas`.
    //
    // A campaign is not owned by one satker, so this counts the campaigns the
    // caller can SEE — the same predicate the pengajuan list uses
    // (`pakaian_dinas::scope::campaign_visibility_condition`), so the card and
    // the list behind it cannot disagree about how many campaigns exist.
    let mut jp: Vec<BoxedParam> = vec![Box::new(params.tahun_anggaran)];
    let jp_scope = crate::pakaian_dinas::scope::campaign_visibility_condition(scope, &mut jp)
        .map(|c| format!(" AND {c}"))
        .unwrap_or_default();
    let jenis_query = format!(
        r#"
        SELECT COALESCE(j.nama, '(tanpa jenis)') AS jenis_pakaian, COUNT(*) AS count
        FROM perlengkapan.pengajuan_pakaian_dinas p
        LEFT JOIN perlengkapan.ms_jenis_pakaian_dinas j ON j.id = p.jenis_pakaian_dinas_id
        WHERE p.tahun = $1{jp_scope}
        GROUP BY jenis_pakaian
    "#
    );

    let rows = client.query(jenis_query.as_str(), &as_refs(&jp)).await?;

    let total_by_jenis: HashMap<String, i64> = rows
        .into_iter()
        .map(|row| (row.get("jenis_pakaian"), row.get("count")))
        .collect();

    // Total by ukuran — sizes are recorded per employee per garment, so this
    // climbs pegawai_ukuran -> satker -> campaign to reach the year.
    //
    // Sizes ARE per-satker (`pengajuan_pakaian_dinas_satker.satker_id` holds the
    // MySIMKARI kode_satker since V006/#94), so this one filters the column
    // directly rather than through campaign visibility: an operator who can see
    // a nationwide campaign still must not read another satker's body sizes.
    let mut up: Vec<BoxedParam> = vec![Box::new(params.tahun_anggaran)];
    let up_scope = scope_and(scope, "ps.satker_id", &mut up);
    let ukuran_query = format!(
        r#"
        SELECT u.ukuran, COUNT(*) AS count
        FROM perlengkapan.pengajuan_pakaian_dinas_satker_pegawai_ukuran u
        JOIN perlengkapan.pengajuan_pakaian_dinas_satker ps ON ps.id = u.pengajuan_satker_id
        JOIN perlengkapan.pengajuan_pakaian_dinas p ON p.id = ps.pengajuan_id
        WHERE p.tahun = $1 AND u.ukuran IS NOT NULL{up_scope}
        GROUP BY u.ukuran
    "#
    );

    let rows = client.query(ukuran_query.as_str(), &as_refs(&up)).await?;

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
pub async fn fetch_workflow_metrics(
    db_pool: &Pool,
    scope: &SatkerScope,
) -> Result<WorkflowMetrics, AppError> {
    let client = db_pool.get().await?;

    // Average processing time for completed per-satker responses. "Completed" is
    // status_kode 2008 (models/status.rs). The old query filtered a TEXT
    // `status = 'COMPLETED'` that exists nowhere in this schema.
    // The `::FLOAT8` is load-bearing, not defensive noise. `EXTRACT(EPOCH ...)`
    // returns NUMERIC on PostgreSQL 14+ (it was float8 before), and AVG() over
    // numeric stays numeric. tokio_postgres has no `FromSql<f64>` for NUMERIC, so
    // reading it into an f64 PANICS instead of returning Err — and because the
    // release profile sets `panic = "abort"` (root Cargo.toml), that panic takes
    // the entire service down rather than failing the one request. Verified in
    // CI: this exact query aborted layanan-perlengkapan mid-run, which is why 12
    // unrelated e2e tests then failed with ENOTFOUND.
    let mut ap: Vec<BoxedParam> = Vec::new();
    let ap_scope = scope_and(scope, "ps.satker_id", &mut ap);
    let avg_time_query = format!(
        r#"
        SELECT COALESCE(AVG(EXTRACT(EPOCH FROM (ps.updated_at - ps.created_at)) / 3600), 0)::FLOAT8 AS avg_hours
        FROM perlengkapan.pengajuan_kebutuhan_bmn_satker ps
        WHERE ps.status_kode = 2008{ap_scope}
    "#
    );

    let row = client
        .query_one(avg_time_query.as_str(), &as_refs(&ap))
        .await?;
    let average_processing_time_hours: f64 = row.get("avg_hours");

    // Bottlenecks (states with longest average time)
    // This is a simplified version - in production, you'd use window functions
    // The activity table carries no satker of its own, so the scope reaches it
    // through the per-satker response it belongs to. Without that join the
    // bottleneck card kept reporting the whole country's queue depth to a
    // single satker's operator.
    let mut bp: Vec<BoxedParam> = Vec::new();
    let bp_scope = scope_and(scope, "ps.satker_id", &mut bp);
    let bottleneck_query = format!(
        r#"
        SELECT
            -- The activity row records the workflow STATE it moved into
            -- (`to_status_kode`); it has no `aktivitas_id`, and this table has no
            -- relation to `ms_aktivitas_bmn` at all — the old query joined both,
            -- so it errored before it could return anything. Labels come from
            -- ms_workflow_status; the code is cast to text because
            -- BottleneckInfo.state is a String and the kode column is INTEGER.
            COALESCE(w.nama, 'Kode ' || ka.to_status_kode::text) AS state,
            -- ::FLOAT8 for the same NUMERIC-vs-f64 reason as avg_time_query above.
            COALESCE(AVG(EXTRACT(EPOCH FROM (NOW() - ka.created_at)) / 3600), 0)::FLOAT8 as avg_hours,
            COUNT(*) as count
        FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas ka
        JOIN perlengkapan.pengajuan_kebutuhan_bmn_satker ps ON ps.id = ka.pengajuan_satker_id
        LEFT JOIN perlengkapan.ms_workflow_status w
               ON w.modul = 'kebutuhan_bmn' AND w.kode = ka.to_status_kode
        WHERE ka.created_at > NOW() - INTERVAL '30 days'{bp_scope}
        GROUP BY w.nama, ka.to_status_kode
        HAVING AVG(EXTRACT(EPOCH FROM (NOW() - ka.created_at)) / 3600) > 24
        ORDER BY avg_hours DESC
        LIMIT 5
    "#
    );

    let rows = client
        .query(bottleneck_query.as_str(), &as_refs(&bp))
        .await?;

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
    let mut lp: Vec<BoxedParam> = Vec::new();
    let lp_scope = scope_and(scope, "ps.satker_id", &mut lp);
    let sla_breach_query = format!(
        r#"
        SELECT COUNT(*) AS count
        FROM perlengkapan.pengajuan_kebutuhan_bmn_satker ps
        LEFT JOIN perlengkapan.ms_workflow_status w
               ON w.modul = 'kebutuhan_bmn' AND w.kode = ps.status_kode
        WHERE ps.created_at < NOW() - INTERVAL '2 days'
          AND COALESCE(w.is_terminal, FALSE) = FALSE{lp_scope}
    "#
    );

    let row = client
        .query_one(sla_breach_query.as_str(), &as_refs(&lp))
        .await?;
    let sla_breaches_today: i64 = row.get("count");

    Ok(WorkflowMetrics {
        average_processing_time_hours,
        bottlenecks,
        sla_breaches_today,
    })
}

/// Fetch asset utilization from SIMAN data
pub async fn fetch_asset_utilization(
    db_pool: &Pool,
    aset_scope: &AsetScope,
) -> Result<AssetUtilization, AppError> {
    let client = db_pool.get().await?;

    // Two fixes over the previous form, both of which killed the process rather
    // than erroring (release profile is `panic = "abort"`):
    //
    //  * COALESCE around SUM(): with no GROUP BY, an EMPTY siman_aset still yields
    //    one row, and SUM() over zero rows is NULL. Reading NULL into i64 panics.
    //    That is the state of every environment before its first SIMAN sync.
    //  * the condition column: `kondisi` is only populated when the SIMAN API
    //    returns a KONDISI field (siman/transform.rs:107); the e2e seed and much
    //    real data carry `ur_kondisi` instead, so `kondisi = 'BAIK'` counted zero
    //    good assets. integrasi's own gRPC reader coalesces both
    //    (grpc/service.rs:972) — that is the SoT owner's canonical form, so use it.
    let mut params: Vec<BoxedParam> = Vec::new();
    let scope_sql = aset_where(aset_scope, &mut params);
    let query = format!(
        r#"
        SELECT
            COUNT(*) AS total_assets,
            COALESCE(SUM(
                CASE WHEN UPPER(COALESCE(kondisi, ur_kondisi, '')) = 'BAIK'
                     THEN 1 ELSE 0 END
            ), 0) AS assets_in_good_condition
        FROM integrasi.siman_aset{scope_sql}
    "#
    );

    let row = client.query_one(query.as_str(), &as_refs(&params)).await?;

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
    scope: &SatkerScope,
) -> Result<ModuleStatusMetrics, AppError> {
    let client = db_pool.get().await?;
    // `satker_code` is the authoritative owner column added by V003 — the same
    // one the pemakaian list and detail are scoped on (#66, #875).
    let mut params: Vec<BoxedParam> = Vec::new();
    let scope_sql = scope_and(scope, "satker_code", &mut params);
    let sql = format!(
        "SELECT status, COUNT(*) AS count
             FROM perlengkapan.izin_pemakaian_bmn
             WHERE TRUE{scope_sql}
             GROUP BY status"
    );
    let rows = client.query(sql.as_str(), &as_refs(&params)).await?;
    Ok(rows_to_status_metrics(rows))
}

/// Rekap status Usulan SK Penghapusan BMN (Fase 2.7) — COUNT per `status`.
pub async fn fetch_penghapusan_status_metrics(
    db_pool: &Pool,
    scope: &SatkerScope,
) -> Result<ModuleStatusMetrics, AppError> {
    let client = db_pool.get().await?;
    let mut params: Vec<BoxedParam> = Vec::new();
    let scope_sql = scope_and(scope, "satker_code", &mut params);
    let sql = format!(
        "SELECT status, COUNT(*) AS count
             FROM perlengkapan.penghapusan_bmn
             WHERE TRUE{scope_sql}
             GROUP BY status"
    );
    let rows = client.query(sql.as_str(), &as_refs(&params)).await?;
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
