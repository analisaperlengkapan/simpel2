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

/// Workflow history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowHistory {
    /// Activity ID
    pub id: Uuid,

    /// Entity ID
    pub entity_id: Uuid,

    /// State name
    pub state: String,

    /// User who performed the action
    pub user_id: Uuid,

    /// Notes/comments
    pub catatan: Option<String>,

    /// Timestamp
    pub created_at: DateTime<Utc>,
}

/// Workflow state duration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDuration {
    /// State name
    pub state: String,

    /// Duration in minutes
    pub duration_minutes: i64,

    /// Start time
    pub started_at: DateTime<Utc>,

    /// End time (None if still in this state)
    pub ended_at: Option<DateTime<Utc>>,
}

/// Workflow monitoring service
pub struct WorkflowMonitor {
    /// Database connection pool
    db_pool: Pool,
}

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

        // Get total active workflows (non-terminal states)
        let total_query = r#"
            SELECT COUNT(*) as total
            FROM perlengkapan.kebutuhan_bmn
            WHERE status NOT IN ('REJECTED', 'CANCELLED', 'COMPLETED')
        "#;

        let total_row = client.query_one(total_query, &[]).await?;
        let total_active: i64 = total_row.get("total");

        // Get workflows by state
        let by_state_query = r#"
            SELECT status, COUNT(*) as count
            FROM perlengkapan.kebutuhan_bmn
            WHERE status NOT IN ('REJECTED', 'CANCELLED', 'COMPLETED')
            GROUP BY status
        "#;

        let by_state_rows = client.query(by_state_query, &[]).await?;
        let mut by_state = HashMap::new();
        for row in by_state_rows {
            let status: String = row.get("status");
            let count: i64 = row.get("count");
            by_state.insert(status, count);
        }

        // Get average processing time per state
        let avg_time_query = r#"
            WITH state_durations AS (
                SELECT
                    a1.pengajuan_id,
                    mab1.nama as state,
                    a1.created_at as start_time,
                    COALESCE(a2.created_at, NOW()) as end_time,
                    EXTRACT(EPOCH FROM (COALESCE(a2.created_at, NOW()) - a1.created_at)) / 60 as duration_minutes
                FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas a1
                JOIN perlengkapan.ms_aktivitas_bmn mab1 ON a1.aktivitas_id = mab1.id
                LEFT JOIN LATERAL (
                    SELECT created_at
                    FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas
                    WHERE pengajuan_id = a1.pengajuan_id
                      AND created_at > a1.created_at
                    ORDER BY created_at ASC
                    LIMIT 1
                ) a2 ON true
            )
            SELECT state, AVG(duration_minutes) as avg_duration
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

        // Get SLA breach count (simplified - would need SLA configuration)
        let sla_breach_query = r#"
            SELECT COUNT(*) as breach_count
            FROM perlengkapan.kebutuhan_bmn kb
            JOIN LATERAL (
                SELECT created_at
                FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas
                WHERE pengajuan_id = kb.id
                ORDER BY created_at DESC
                LIMIT 1
            ) a ON true
            WHERE kb.status NOT IN ('REJECTED', 'CANCELLED', 'COMPLETED')
              AND (
                  (kb.status = 'SUBMIT_SATKER' AND NOW() - a.created_at > INTERVAL '2 days') OR
                  (kb.status = 'ANALISIS_KELAYAKAN' AND NOW() - a.created_at > INTERVAL '3 days') OR
                  (kb.status = 'PENYUSUNAN_PRIORITAS' AND NOW() - a.created_at > INTERVAL '1 day')
              )
        "#;

        let sla_breach_row = client.query_one(sla_breach_query, &[]).await?;
        let sla_breaches: i64 = sla_breach_row.get("breach_count");

        // Get workflows approaching SLA (within 25% of limit)
        let approaching_sla_query = r#"
            SELECT COUNT(*) as approaching_count
            FROM perlengkapan.kebutuhan_bmn kb
            JOIN LATERAL (
                SELECT created_at
                FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas
                WHERE pengajuan_id = kb.id
                ORDER BY created_at DESC
                LIMIT 1
            ) a ON true
            WHERE kb.status NOT IN ('REJECTED', 'CANCELLED', 'COMPLETED')
              AND (
                  (kb.status = 'SUBMIT_SATKER' AND NOW() - a.created_at > INTERVAL '36 hours') OR
                  (kb.status = 'ANALISIS_KELAYAKAN' AND NOW() - a.created_at > INTERVAL '54 hours') OR
                  (kb.status = 'PENYUSUNAN_PRIORITAS' AND NOW() - a.created_at > INTERVAL '18 hours')
              )
        "#;

        let approaching_row = client.query_one(approaching_sla_query, &[]).await?;
        let approaching_sla: i64 = approaching_row.get("approaching_count");

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

        let query = r#"
            WITH state_stats AS (
                SELECT
                    kb.status as state,
                    COUNT(*) as count,
                    AVG(EXTRACT(EPOCH FROM (NOW() - a.created_at)) / 60) as avg_time_minutes
                FROM perlengkapan.kebutuhan_bmn kb
                JOIN LATERAL (
                    SELECT created_at
                    FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas
                    WHERE pengajuan_id = kb.id
                    ORDER BY created_at DESC
                    LIMIT 1
                ) a ON true
                WHERE kb.status NOT IN ('REJECTED', 'CANCELLED', 'COMPLETED')
                GROUP BY kb.status
                HAVING COUNT(*) > (
                    SELECT COUNT(*) * 0.1
                    FROM perlengkapan.kebutuhan_bmn
                    WHERE status NOT IN ('REJECTED', 'CANCELLED', 'COMPLETED')
                )
            )
            SELECT state, count, avg_time_minutes
            FROM state_stats
            ORDER BY count DESC, avg_time_minutes DESC
        "#;

        let rows = client.query(query, &[]).await?;

        let mut bottlenecks = Vec::new();
        for row in rows {
            let state: String = row.get("state");
            let count: i64 = row.get("count");
            let avg_time_minutes: f64 = row.get("avg_time_minutes");

            // Get SLA limit for this state (hardcoded for now, should come from config)
            let (sla_limit_minutes, sla_usage_percent) = match state.as_str() {
                "SUBMIT_SATKER" => {
                    let limit = 2880; // 2 days
                    let usage = (avg_time_minutes / limit as f64) * 100.0;
                    (Some(limit), Some(usage))
                }
                "ANALISIS_KELAYAKAN" => {
                    let limit = 4320; // 3 days
                    let usage = (avg_time_minutes / limit as f64) * 100.0;
                    (Some(limit), Some(usage))
                }
                "PENYUSUNAN_PRIORITAS" => {
                    let limit = 1440; // 1 day
                    let usage = (avg_time_minutes / limit as f64) * 100.0;
                    (Some(limit), Some(usage))
                }
                _ => (None, None),
            };

            // Only include if using more than 75% of SLA
            if let Some(usage) = sla_usage_percent {
                if usage > 75.0 {
                    bottlenecks.push(BottleneckInfo {
                        state,
                        count,
                        avg_time_minutes,
                        sla_limit_minutes,
                        sla_usage_percent,
                    });
                }
            }
        }

        Ok(bottlenecks)
    }

    /// Get workflow history for an entity
    ///
    /// Requirements: REQ-W008
    pub async fn get_workflow_history(
        &self,
        entity_id: Uuid,
    ) -> Result<Vec<WorkflowHistory>, MonitoringError> {
        let client = self.db_pool.get().await?;

        let query = r#"
            SELECT
                a.id,
                a.pengajuan_id as entity_id,
                mab.nama as state,
                a.user_id,
                a.catatan,
                a.created_at
            FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas a
            JOIN perlengkapan.ms_aktivitas_bmn mab ON a.aktivitas_id = mab.id
            WHERE a.pengajuan_id = $1
            ORDER BY a.created_at ASC
        "#;

        let rows = client.query(query, &[&entity_id]).await?;

        let history = rows
            .into_iter()
            .map(|row| WorkflowHistory {
                id: row.get("id"),
                entity_id: row.get("entity_id"),
                state: row.get("state"),
                user_id: row.get("user_id"),
                catatan: row.get("catatan"),
                created_at: row.get("created_at"),
            })
            .collect();

        Ok(history)
    }

    /// Get state durations for an entity
    ///
    /// Shows how long the entity spent in each state
    pub async fn get_state_durations(
        &self,
        entity_id: Uuid,
    ) -> Result<Vec<StateDuration>, MonitoringError> {
        let client = self.db_pool.get().await?;

        let query = r#"
            SELECT
                mab.nama as state,
                a1.created_at as started_at,
                a2.created_at as ended_at,
                EXTRACT(EPOCH FROM (COALESCE(a2.created_at, NOW()) - a1.created_at)) / 60 as duration_minutes
            FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas a1
            JOIN perlengkapan.ms_aktivitas_bmn mab ON a1.aktivitas_id = mab.id
            LEFT JOIN LATERAL (
                SELECT created_at
                FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas
                WHERE pengajuan_id = a1.pengajuan_id
                  AND created_at > a1.created_at
                ORDER BY created_at ASC
                LIMIT 1
            ) a2 ON true
            WHERE a1.pengajuan_id = $1
            ORDER BY a1.created_at ASC
        "#;

        let rows = client.query(query, &[&entity_id]).await?;

        let durations = rows
            .into_iter()
            .map(|row| {
                let duration_minutes: f64 = row.get("duration_minutes");
                StateDuration {
                    state: row.get("state"),
                    duration_minutes: duration_minutes as i64,
                    started_at: row.get("started_at"),
                    ended_at: row.get("ended_at"),
                }
            })
            .collect();

        Ok(durations)
    }

    /// Get workflows by state with pagination
    pub async fn get_workflows_by_state(
        &self,
        state: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<WorkflowSummary>, MonitoringError> {
        let client = self.db_pool.get().await?;

        let query = r#"
            SELECT
                kb.id,
                kb.satker_id,
                kb.kode_barang,
                kb.nama_barang,
                kb.status,
                a.created_at as state_entered_at,
                EXTRACT(EPOCH FROM (NOW() - a.created_at)) / 60 as time_in_state_minutes
            FROM perlengkapan.kebutuhan_bmn kb
            JOIN LATERAL (
                SELECT created_at
                FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas
                WHERE pengajuan_id = kb.id
                ORDER BY created_at DESC
                LIMIT 1
            ) a ON true
            WHERE kb.status = $1
            ORDER BY a.created_at ASC
            LIMIT $2 OFFSET $3
        "#;

        let rows = client.query(query, &[&state, &limit, &offset]).await?;

        let workflows = rows
            .into_iter()
            .map(|row| {
                let time_in_state: f64 = row.get("time_in_state_minutes");
                WorkflowSummary {
                    id: row.get("id"),
                    satker_id: row.get("satker_id"),
                    kode_barang: row.get("kode_barang"),
                    nama_barang: row.get("nama_barang"),
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
    /// Entity ID
    pub id: Uuid,

    /// Satker ID
    pub satker_id: Uuid,

    /// Kode barang
    pub kode_barang: String,

    /// Nama barang
    pub nama_barang: String,

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
