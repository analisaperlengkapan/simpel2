// ============================================================================
// Workflow Monitoring Module
// Description: Workflow metrics, bottleneck detection, and monitoring dashboard
// Requirements: REQ-W008
// ============================================================================

use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Workflow metrics for monitoring dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMetrics {
    /// Total number of active workflows
    pub total_active: i64,

    /// Workflows by state
    pub by_state: HashMap<String, i64>,

    /// Average processing time per state (in minutes)
    pub avg_processing_time: HashMap<String, f64>,

    /// SLA breach count
    pub sla_breaches: i64,

    /// Workflows approaching SLA deadline (within 25% of limit)
    pub approaching_sla: i64,

    /// Bottleneck states (states with high volume and long processing time)
    pub bottlenecks: Vec<BottleneckInfo>,

    /// Timestamp when metrics were calculated
    pub calculated_at: DateTime<Utc>,
}

/// Bottleneck information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BottleneckInfo {
    /// State name
    pub state: String,

    /// Number of workflows in this state
    pub count: i64,

    /// Average time spent in this state (in minutes)
    pub avg_time_minutes: f64,

    /// SLA limit for this state (in minutes)
    pub sla_limit_minutes: Option<u32>,

    /// Percentage of SLA limit used on average
    pub sla_usage_percent: Option<f64>,
}

// `WorkflowHistory` / `StateDuration` and their two accessors
// (`get_workflow_history`, `get_state_durations`) were removed here: verified
// zero consumers anywhere — no handler, no route, no test, no frontend. Both
// queried the same never-created `perlengkapan.kebutuhan_bmn` / `aktivitas_id`
// shape as the rest of this file, so they were dead code that would have
// errored the moment anything called them. Broken AND unreachable is the one
// combination with no argument for keeping it.

/// Workflow monitoring service
pub struct WorkflowMonitor {
    /// Database connection pool
    db_pool: Pool,
}

/// SLA budget per workflow state, in minutes, keyed by `ms_workflow_status.kode`.
///
/// Keyed by CODE, not by name. The previous version matched on strings
/// ('SUBMIT_SATKER', 'ANALISIS_KELAYAKAN', 'PENYUSUNAN_PRIORITAS') that appear
/// in no enum, no table and no seed — `PENYUSUNAN_PRIORITAS` has no analogue in
/// the workflow at all. Codes come from `KebutuhanBmnStatus` and are the values
/// actually stored in `status_kode`, so they cannot drift from a rename.
///
/// SLA applies only where a REVIEWER owes the next action. Draft/Input Barang
/// are the operator's own desk, the Revisi states are back with the submitter,
/// and the rest are terminal — none of those are anyone's queue, so putting a
/// clock on them would report backlog that no one can clear.
///
/// The durations preserve the original author's intent (2 days to review a
/// submission, 3 days to analyse). They are a starting point pending product
/// input, not a derived truth — see the PR discussion.
const SLA_BUDGET_MINUTES: &[(i32, i32)] = &[
    (2002, 2880), // Diajukan ke Validator Wilayah — 2 days
    (2004, 2880), // Diajukan ke Validator Pusat   — 2 days
    (2005, 4320), // Analisis Kelayakan            — 3 days
];

/// Render `SLA_BUDGET_MINUTES` as a SQL `VALUES` body so the SQL and the Rust
/// share ONE definition. The old code kept the thresholds in two places — a
/// hard-coded SQL list and a Rust `match` — and they had already drifted apart.
///
/// Interpolation is safe by construction: every value is an `i32` from a `const`
/// in this file; no caller input reaches it.
fn sla_values_clause() -> String {
    SLA_BUDGET_MINUTES
        .iter()
        .map(|(kode, minutes)| format!("({kode},{minutes})"))
        .collect::<Vec<_>>()
        .join(",")
}

/// Shared CTE: the current state of every per-satker response, and when it was
/// entered. The activity trail records the state each row moved INTO
/// (`to_status_kode`) at `created_at`, so the latest activity timestamp is the
/// current state's entry time; rows that never moved fall back to their own
/// `created_at`.
const CURRENT_STATE_CTE: &str = r#"
    current_state AS (
        SELECT ps.id,
               ps.satker_id,
               ps.satker_nama,
               ps.status_kode,
               COALESCE(MAX(a.created_at), ps.created_at) AS entered_at
        FROM perlengkapan.pengajuan_kebutuhan_bmn_satker ps
        LEFT JOIN perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas a
               ON a.pengajuan_satker_id = ps.id
        GROUP BY ps.id, ps.satker_id, ps.satker_nama, ps.status_kode, ps.created_at
    )
"#;

impl WorkflowMonitor {
    /// Create a new workflow monitor
    pub fn new(db_pool: Pool) -> Self {
        Self { db_pool }
    }

    /// Get workflow metrics for monitoring dashboard
    ///
    /// Requirements: REQ-W008
    pub async fn get_metrics(&self) -> Result<WorkflowMetrics, MonitoringError> {
        let client = self.db_pool.get().await?;

        // Active workflows by state. The unit is the PER-SATKER response, which
        // is what actually moves through the workflow.
        //
        // Terminality is read from `ms_workflow_status.is_terminal` rather than a
        // hard-coded name list. The old query filtered
        // `status NOT IN ('REJECTED','CANCELLED','COMPLETED')` against a TEXT
        // `status` column on `perlengkapan.kebutuhan_bmn` — a table NO migration
        // creates, whose real counterpart stores `status_kode integer`. Every
        // call 500'd, so the monitoring page never rendered.
        let by_state_query = r#"
            SELECT COALESCE(w.nama, 'Kode ' || ps.status_kode::text) AS state,
                   COUNT(*) AS count
            FROM perlengkapan.pengajuan_kebutuhan_bmn_satker ps
            LEFT JOIN perlengkapan.ms_workflow_status w
                   ON w.modul = 'kebutuhan_bmn' AND w.kode = ps.status_kode
            WHERE COALESCE(w.is_terminal, FALSE) = FALSE
            GROUP BY w.nama, ps.status_kode
        "#;

        let by_state_rows = client.query(by_state_query, &[]).await?;
        let mut by_state = HashMap::new();
        // Derived from the same rows rather than re-counted in a second query:
        // two independent COUNTs can disagree under concurrent writes, and a
        // total that contradicts its own breakdown is worse than no total.
        let mut total_active: i64 = 0;
        for row in by_state_rows {
            let status: String = row.get("state");
            let count: i64 = row.get("count");
            total_active += count;
            by_state.insert(status, count);
        }

        // Average time spent in each state, from the activity trail.
        //
        // The old query joined `ms_aktivitas_bmn ON a1.aktivitas_id` and
        // self-joined on `pengajuan_id`; this table has NEITHER column (it keys
        // on `pengajuan_satker_id` and records `to_status_kode`). The state label
        // therefore comes from ms_workflow_status, and the "next activity"
        // lateral matches on the correct key.
        //
        // ::FLOAT8 is load-bearing: EXTRACT(EPOCH ...) is NUMERIC on PostgreSQL
        // 14+ and AVG() over numeric stays numeric, which tokio_postgres cannot
        // read into f64 — it panics rather than erroring, and the release profile
        // sets `panic = "abort"`, so it would take the whole service down.
        let avg_time_query = r#"
            WITH state_durations AS (
                SELECT
                    COALESCE(w.nama, 'Kode ' || a1.to_status_kode::text) AS state,
                    EXTRACT(EPOCH FROM (COALESCE(nx.created_at, NOW()) - a1.created_at)) / 60
                        AS duration_minutes
                FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas a1
                LEFT JOIN perlengkapan.ms_workflow_status w
                       ON w.modul = 'kebutuhan_bmn' AND w.kode = a1.to_status_kode
                LEFT JOIN LATERAL (
                    SELECT n.created_at
                    FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas n
                    WHERE n.pengajuan_satker_id = a1.pengajuan_satker_id
                      AND n.created_at > a1.created_at
                    ORDER BY n.created_at ASC
                    LIMIT 1
                ) nx ON TRUE
            )
            SELECT state, AVG(duration_minutes)::FLOAT8 AS avg_duration
            FROM state_durations
            GROUP BY state
        "#;

        let avg_time_rows = client.query(avg_time_query, &[]).await?;
        let mut avg_processing_time = HashMap::new();
        for row in avg_time_rows {
            let state: String = row.get("state");
            let avg_duration: f64 = row.get("avg_duration");
            avg_processing_time.insert(state, avg_duration);
        }

        // SLA breaches AND near-breaches in ONE pass.
        //
        // They were two queries whose thresholds were written out twice (2 days
        // vs 36 hours, 3 days vs 54 hours — i.e. 75%). Keeping one budget per
        // state and deriving the warning band from it means the two numbers
        // cannot contradict each other, and "approaching" excludes rows that
        // have already breached instead of double-counting them.
        let sla_query = format!(
            r#"
            WITH sla(status_kode, sla_minutes) AS (VALUES {values}),
            {current_state}
            SELECT
                COUNT(*) FILTER (
                    WHERE NOW() - cs.entered_at > make_interval(mins => s.sla_minutes)
                ) AS breached,
                COUNT(*) FILTER (
                    WHERE NOW() - cs.entered_at > make_interval(mins => (s.sla_minutes * 3) / 4)
                      AND NOW() - cs.entered_at <= make_interval(mins => s.sla_minutes)
                ) AS approaching
            FROM current_state cs
            JOIN sla s ON s.status_kode = cs.status_kode
            "#,
            values = sla_values_clause(),
            current_state = CURRENT_STATE_CTE,
        );

        let sla_row = client.query_one(sla_query.as_str(), &[]).await?;
        let sla_breaches: i64 = sla_row.get("breached");
        let approaching_sla: i64 = sla_row.get("approaching");

        // Detect bottlenecks
        let bottlenecks = self.detect_bottlenecks().await?;

        Ok(WorkflowMetrics {
            total_active,
            by_state,
            avg_processing_time,
            sla_breaches,
            approaching_sla,
            bottlenecks,
            calculated_at: Utc::now(),
        })
    }

    /// Detect workflow bottlenecks
    ///
    /// A bottleneck is defined as a state with:
    /// - High volume (more than 10% of active workflows)
    /// - Long processing time (more than 75% of SLA limit)
    ///
    /// Requirements: REQ-W008
    pub async fn detect_bottlenecks(&self) -> Result<Vec<BottleneckInfo>, MonitoringError> {
        let client = self.db_pool.get().await?;

        // The 10%-volume and 75%-of-SLA rules are both expressed in SQL now. They
        // used to straddle the boundary: the volume rule in SQL, the SLA rule in a
        // Rust `match` carrying its own copy of the budgets. Only states with a
        // budget can be bottlenecks, so the JOIN also does the filtering the
        // `match`'s `_ => (None, None)` arm used to.
        let query = format!(
            r#"
            WITH sla(status_kode, sla_minutes) AS (VALUES {values}),
            {current_state},
            active AS (
                SELECT cs.*, w.nama
                FROM current_state cs
                LEFT JOIN perlengkapan.ms_workflow_status w
                       ON w.modul = 'kebutuhan_bmn' AND w.kode = cs.status_kode
                WHERE COALESCE(w.is_terminal, FALSE) = FALSE
            ),
            state_stats AS (
                SELECT
                    COALESCE(a.nama, 'Kode ' || a.status_kode::text) AS state,
                    a.status_kode,
                    COUNT(*) AS count,
                    AVG(EXTRACT(EPOCH FROM (NOW() - a.entered_at)) / 60)::FLOAT8
                        AS avg_time_minutes
                FROM active a
                GROUP BY a.nama, a.status_kode
                HAVING COUNT(*)::numeric > (SELECT COUNT(*) * 0.1 FROM active)
            )
            SELECT ss.state, ss.count, ss.avg_time_minutes, s.sla_minutes
            FROM state_stats ss
            JOIN sla s ON s.status_kode = ss.status_kode
            WHERE ss.avg_time_minutes > s.sla_minutes * 0.75
            ORDER BY ss.count DESC, ss.avg_time_minutes DESC
            "#,
            values = sla_values_clause(),
            current_state = CURRENT_STATE_CTE,
        );

        let rows = client.query(query.as_str(), &[]).await?;

        let bottlenecks = rows
            .into_iter()
            .map(|row| {
                let avg_time_minutes: f64 = row.get("avg_time_minutes");
                let sla_limit: i32 = row.get("sla_minutes");
                BottleneckInfo {
                    state: row.get("state"),
                    count: row.get("count"),
                    avg_time_minutes,
                    sla_limit_minutes: Some(sla_limit as u32),
                    sla_usage_percent: Some((avg_time_minutes / f64::from(sla_limit)) * 100.0),
                }
            })
            .collect();

        Ok(bottlenecks)
    }

    /// List every workflow currently sitting in a non-terminal state, oldest
    /// first (longest-waiting at the top, which is what a monitoring list is for).
    ///
    /// Replaces `get_workflows_by_state(state, ..)`. That signature forced the
    /// caller to supply state NAMES, and its only caller looped over six
    /// hard-coded strings ("DRAFT", "SUBMIT_SATKER", "PENYUSUNAN_PRIORITAS", …)
    /// of which NONE exist — the real names are Indonesian and live in
    /// `ms_workflow_status` ("Draft", "Diajukan ke Validator Wilayah", …). So the
    /// endpoint issued six queries against a phantom table and, had the table
    /// existed, would still have matched nothing. Deriving the set from
    /// `is_terminal` removes the list that could be wrong.
    pub async fn list_active_workflows(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<WorkflowSummary>, MonitoringError> {
        let client = self.db_pool.get().await?;

        // `jumlah_barang` replaces the old kode_barang/nama_barang pair: items
        // live on `..._satker_barang`, MANY per response, so a single code/name
        // could only ever have been one arbitrary row of several. A count is
        // both honest at this grain and more useful in a queue view.
        let query = format!(
            r#"
            WITH {current_state}
            SELECT cs.id,
                   cs.satker_id,
                   COALESCE(cs.satker_nama, cs.satker_id) AS satker_nama,
                   COALESCE(w.nama, 'Kode ' || cs.status_kode::text) AS status,
                   cs.entered_at AS state_entered_at,
                   (EXTRACT(EPOCH FROM (NOW() - cs.entered_at)) / 60)::FLOAT8
                       AS time_in_state_minutes,
                   (SELECT COUNT(*)
                      FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_barang b
                     WHERE b.pengajuan_satker_id = cs.id)::bigint AS jumlah_barang
            FROM current_state cs
            LEFT JOIN perlengkapan.ms_workflow_status w
                   ON w.modul = 'kebutuhan_bmn' AND w.kode = cs.status_kode
            WHERE COALESCE(w.is_terminal, FALSE) = FALSE
            ORDER BY cs.entered_at ASC
            LIMIT $1 OFFSET $2
            "#,
            current_state = CURRENT_STATE_CTE,
        );

        let rows = client.query(query.as_str(), &[&limit, &offset]).await?;

        let workflows = rows
            .into_iter()
            .map(|row| {
                let time_in_state: f64 = row.get("time_in_state_minutes");
                WorkflowSummary {
                    id: row.get("id"),
                    satker_id: row.get("satker_id"),
                    satker_nama: row.get("satker_nama"),
                    jumlah_barang: row.get("jumlah_barang"),
                    status: row.get("status"),
                    state_entered_at: row.get("state_entered_at"),
                    time_in_state_minutes: time_in_state as i64,
                }
            })
            .collect();

        Ok(workflows)
    }
}

/// Workflow summary for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSummary {
    /// Per-satker response ID
    pub id: Uuid,

    /// MySIMKARI `kode_satker`. TEXT, not a UUID — the column it comes from is
    /// `varchar`, so the previous `Uuid` here could never have decoded.
    pub satker_id: String,

    /// Satker name, falling back to the code when the snapshot is absent
    pub satker_nama: String,

    /// How many requested items this response carries
    pub jumlah_barang: i64,

    /// Current status
    pub status: String,

    /// When entered current state
    pub state_entered_at: DateTime<Utc>,

    /// Time spent in current state (in minutes)
    pub time_in_state_minutes: i64,
}

/// Monitoring errors
#[derive(Debug, thiserror::Error)]
pub enum MonitoringError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] tokio_postgres::Error),

    #[error("Pool error: {0}")]
    PoolError(#[from] deadpool_postgres::PoolError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bottleneck_info_serialization() {
        let bottleneck = BottleneckInfo {
            state: "SUBMIT_SATKER".to_string(),
            count: 150,
            avg_time_minutes: 2500.0,
            sla_limit_minutes: Some(2880),
            sla_usage_percent: Some(86.8),
        };

        let json = serde_json::to_string(&bottleneck).unwrap();
        assert!(json.contains("SUBMIT_SATKER"));
        assert!(json.contains("150"));
        assert!(json.contains("2500"));
    }

    #[test]
    fn test_workflow_metrics_structure() {
        let mut by_state = HashMap::new();
        by_state.insert("DRAFT".to_string(), 10);
        by_state.insert("SUBMIT_SATKER".to_string(), 25);

        let metrics = WorkflowMetrics {
            total_active: 35,
            by_state,
            avg_processing_time: HashMap::new(),
            sla_breaches: 5,
            approaching_sla: 8,
            bottlenecks: vec![],
            calculated_at: Utc::now(),
        };

        assert_eq!(metrics.total_active, 35);
        assert_eq!(metrics.sla_breaches, 5);
    }
}
