// ============================================================================
// SLA Monitoring Module
// Description: Service Level Agreement monitoring and escalation
// Requirements: REQ-W003
// ============================================================================

use crate::metrics;
use crate::workflow::config::WorkflowConfig;
use crate::workflow::notifikasi_client::{
    NotificationPriority, NotifikasiClient, WorkflowNotificationType,
};
use chrono::{DateTime, Duration, Utc};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// SLA breach information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaBreachInfo {
    /// Entity ID
    pub entity_id: Uuid,

    /// Current state
    pub current_state: String,

    /// When the entity entered this state
    pub state_entered_at: DateTime<Utc>,

    /// SLA deadline
    pub sla_deadline: DateTime<Utc>,

    /// How long past the deadline (in minutes)
    pub breach_duration_minutes: i64,

    /// Elapsed time in current state (in minutes)
    pub elapsed_minutes: i64,

    /// SLA limit (in minutes)
    pub sla_limit_minutes: u32,
}

/// SLA monitoring service
pub struct SlaMonitor {
    /// Workflow configuration
    config: WorkflowConfig,

    /// Database connection pool
    db_pool: Pool,

    /// Notification service client (optional - for production use)
    notifikasi_client: Option<NotifikasiClient>,
}

impl SlaMonitor {
    /// Create a new SLA monitor
    pub fn new(config: WorkflowConfig, db_pool: Pool) -> Self {
        Self {
            config,
            db_pool,
            notifikasi_client: None,
        }
    }

    /// Create a new SLA monitor with notification service
    pub fn with_notifikasi(
        config: WorkflowConfig,
        db_pool: Pool,
        notifikasi_client: NotifikasiClient,
    ) -> Self {
        Self {
            config,
            db_pool,
            notifikasi_client: Some(notifikasi_client),
        }
    }

    /// Check SLA for a specific entity
    ///
    /// Returns Some(SlaBreachInfo) if SLA is breached, None otherwise
    ///
    /// Requirements: REQ-W003
    pub async fn check_sla(&self, entity_id: Uuid) -> Result<Option<SlaBreachInfo>, SlaError> {
        let client = self.db_pool.get().await?;

        // Get current state and when it was entered
        let query = r#"
            SELECT
                kb.status,
                a.created_at as state_entered_at
            FROM perlengkapan.kebutuhan_bmn kb
            LEFT JOIN LATERAL (
                SELECT created_at
                FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas
                WHERE pengajuan_id = kb.id
                ORDER BY created_at DESC
                LIMIT 1
            ) a ON true
            WHERE kb.id = $1
        "#;

        let row = client
            .query_opt(query, &[&entity_id])
            .await?
            .ok_or_else(|| SlaError::EntityNotFound(entity_id))?;

        let current_state: String = row.get("status");
        let state_entered_at: DateTime<Utc> = row.get("state_entered_at");

        // Get SLA limit for this state
        let sla_limit_minutes = match self.config.get_sla_minutes(&current_state) {
            Some(minutes) => minutes,
            None => {
                // No SLA defined for this state
                return Ok(None);
            }
        };

        // Calculate elapsed time
        let now = Utc::now();
        let elapsed = now.signed_duration_since(state_entered_at);
        let elapsed_minutes = elapsed.num_minutes();

        // Calculate SLA deadline
        let sla_deadline = state_entered_at + Duration::minutes(sla_limit_minutes as i64);

        // Check if SLA is breached
        if now > sla_deadline {
            let breach_duration = now.signed_duration_since(sla_deadline);
            let breach_duration_minutes = breach_duration.num_minutes();

            Ok(Some(SlaBreachInfo {
                entity_id,
                current_state,
                state_entered_at,
                sla_deadline,
                breach_duration_minutes,
                elapsed_minutes,
                sla_limit_minutes,
            }))
        } else {
            Ok(None)
        }
    }

    /// Check SLA for all entities in non-terminal states
    ///
    /// Returns a list of all entities with SLA breaches
    pub async fn check_all_sla(&self) -> Result<Vec<SlaBreachInfo>, SlaError> {
        let client = self.db_pool.get().await?;

        // Get all entities in non-terminal states
        let query = r#"
            SELECT
                kb.id,
                kb.status,
                a.created_at as state_entered_at
            FROM perlengkapan.kebutuhan_bmn kb
            LEFT JOIN LATERAL (
                SELECT created_at
                FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas
                WHERE pengajuan_id = kb.id
                ORDER BY created_at DESC
                LIMIT 1
            ) a ON true
            WHERE kb.status NOT IN ('REJECTED', 'CANCELLED', 'COMPLETED')
        "#;

        let rows = client.query(query, &[]).await?;

        let mut breaches = Vec::new();
        let now = Utc::now();

        for row in rows {
            let entity_id: Uuid = row.get("id");
            let current_state: String = row.get("status");
            let state_entered_at: DateTime<Utc> = row.get("state_entered_at");

            // Get SLA limit for this state
            let sla_limit_minutes = match self.config.get_sla_minutes(&current_state) {
                Some(minutes) => minutes,
                None => continue, // No SLA defined for this state
            };

            // Calculate elapsed time
            let elapsed = now.signed_duration_since(state_entered_at);
            let elapsed_minutes = elapsed.num_minutes();

            // Calculate SLA deadline
            let sla_deadline = state_entered_at + Duration::minutes(sla_limit_minutes as i64);

            // Check if SLA is breached
            if now > sla_deadline {
                let breach_duration = now.signed_duration_since(sla_deadline);
                let breach_duration_minutes = breach_duration.num_minutes();

                breaches.push(SlaBreachInfo {
                    entity_id,
                    current_state,
                    state_entered_at,
                    sla_deadline,
                    breach_duration_minutes,
                    elapsed_minutes,
                    sla_limit_minutes,
                });
            }
        }

        Ok(breaches)
    }

    /// Send escalation notification for SLA breach
    ///
    /// Sends notifications to:
    /// 1. The current approver (escalation to supervisor)
    /// 2. The requester (informational)
    ///
    /// Requirements: REQ-W003, REQ-N008, NFR-M004
    pub async fn escalate_sla_breach(&self, breach: &SlaBreachInfo) -> Result<(), SlaError> {
        // Record SLA breach metrics
        metrics::workflow_sla_breaches_total()
            .with_label_values(&["kebutuhan_bmn", &breach.current_state])
            .inc();

        metrics::workflow_sla_breach_duration()
            .with_label_values(&["kebutuhan_bmn", &breach.current_state])
            .observe(breach.breach_duration_minutes as f64);

        // Get approver and requester user IDs
        let (approver_id, requester_id) = self.get_approver_and_requester(breach.entity_id).await?;

        // Send notification via notification service if available
        if let Some(mut client) = self.notifikasi_client.clone() {
            // Send escalation notification to approver
            let escalation_notification = WorkflowNotificationType::SlaBreachEscalation {
                entity_type: "kebutuhan_bmn".to_string(),
                entity_id: breach.entity_id.to_string(),
                current_state: breach.current_state.clone(),
                sla_deadline: breach.sla_deadline.to_rfc3339(),
                breach_duration_minutes: breach.breach_duration_minutes,
                days_overdue: (breach.breach_duration_minutes / (24 * 60)) as i32,
            };

            match client
                .send_notification(
                    approver_id,
                    escalation_notification.clone(),
                    NotificationPriority::Urgent,
                )
                .await
            {
                Ok(response) => {
                    tracing::info!(
                        entity_id = %breach.entity_id,
                        approver_id = %approver_id,
                        notification_id = %response.notification_id,
                        "SLA breach escalation notification sent to approver"
                    );

                    // Record successful escalation
                    metrics::workflow_escalations_total()
                        .with_label_values(&["kebutuhan_bmn", &breach.current_state, "success"])
                        .inc();
                }
                Err(e) => {
                    tracing::error!(
                        entity_id = %breach.entity_id,
                        approver_id = %approver_id,
                        error = %e,
                        "Failed to send SLA breach escalation notification to approver"
                    );

                    // Record failed escalation
                    metrics::workflow_escalations_total()
                        .with_label_values(&["kebutuhan_bmn", &breach.current_state, "error"])
                        .inc();

                    return Err(SlaError::NotificationError(e.to_string()));
                }
            }

            // Send informational notification to requester
            let requester_notification = WorkflowNotificationType::SlaBreachInfo {
                entity_type: "kebutuhan_bmn".to_string(),
                entity_id: breach.entity_id.to_string(),
                current_state: breach.current_state.clone(),
                sla_deadline: breach.sla_deadline.to_rfc3339(),
                breach_duration_minutes: breach.breach_duration_minutes,
            };

            match client
                .send_notification(
                    requester_id,
                    requester_notification,
                    NotificationPriority::Normal,
                )
                .await
            {
                Ok(response) => {
                    tracing::info!(
                        entity_id = %breach.entity_id,
                        requester_id = %requester_id,
                        notification_id = %response.notification_id,
                        "SLA breach informational notification sent to requester"
                    );
                }
                Err(e) => {
                    // Log error but don't fail - requester notification is informational
                    tracing::error!(
                        entity_id = %breach.entity_id,
                        requester_id = %requester_id,
                        error = %e,
                        "Failed to send SLA breach informational notification to requester"
                    );
                }
            }
        } else {
            // Fallback: log the escalation (for testing/development)
            tracing::warn!(
                entity_id = %breach.entity_id,
                current_state = %breach.current_state,
                breach_duration_minutes = %breach.breach_duration_minutes,
                sla_limit_minutes = %breach.sla_limit_minutes,
                approver_id = %approver_id,
                requester_id = %requester_id,
                "SLA breach detected - escalation notification would be sent (notification service not configured)"
            );

            // Record escalation as success (logged)
            metrics::workflow_escalations_total()
                .with_label_values(&["kebutuhan_bmn", &breach.current_state, "success"])
                .inc();
        }

        // Log escalation in workflow activity
        self.log_sla_escalation(breach).await?;

        Ok(())
    }

    /// Get approver and requester user IDs for an entity
    async fn get_approver_and_requester(&self, entity_id: Uuid) -> Result<(Uuid, Uuid), SlaError> {
        let client = self.db_pool.get().await?;

        let query = r#"
            SELECT
                kb.created_by as requester_id,
                COALESCE(
                    (SELECT user_id
                     FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas
                     WHERE pengajuan_id = kb.id
                     AND aktivitas_id IN (
                         SELECT id FROM perlengkapan.ms_aktivitas_bmn
                         WHERE kode IN ('REVIEWED', 'APPROVED')
                     )
                     ORDER BY created_at DESC
                     LIMIT 1),
                    kb.created_by
                ) as approver_id
            FROM perlengkapan.kebutuhan_bmn kb
            WHERE kb.id = $1
        "#;

        let row = client
            .query_opt(query, &[&entity_id])
            .await?
            .ok_or_else(|| SlaError::EntityNotFound(entity_id))?;

        let requester_id: Uuid = row.get("requester_id");
        let approver_id: Uuid = row.get("approver_id");

        Ok((approver_id, requester_id))
    }

    /// Log SLA escalation in workflow activity
    async fn log_sla_escalation(&self, breach: &SlaBreachInfo) -> Result<(), SlaError> {
        let client = self.db_pool.get().await?;

        let query = r#"
            INSERT INTO perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas
            (pengajuan_id, aktivitas_id, user_id, catatan, created_at)
            VALUES (
                $1,
                (SELECT id FROM perlengkapan.ms_aktivitas_bmn WHERE kode = 'SLA_BREACH' LIMIT 1),
                '00000000-0000-0000-0000-000000000000'::uuid,
                $2,
                NOW()
            )
        "#;

        let catatan = format!(
            "SLA breach detected: {} minutes overdue (limit: {} minutes)",
            breach.breach_duration_minutes, breach.sla_limit_minutes
        );

        client
            .execute(query, &[&breach.entity_id, &catatan])
            .await?;

        Ok(())
    }

    /// Monitor SLA and send escalation notifications for all breaches
    ///
    /// This method should be called periodically (e.g., every hour) by a scheduler
    pub async fn monitor_and_escalate(&self) -> Result<usize, SlaError> {
        let breaches = self.check_all_sla().await?;
        let breach_count = breaches.len();

        for breach in breaches {
            if let Err(e) = self.escalate_sla_breach(&breach).await {
                tracing::error!(
                    entity_id = %breach.entity_id,
                    error = %e,
                    "Failed to escalate SLA breach"
                );
            }
        }

        if breach_count > 0 {
            tracing::info!(
                breach_count = %breach_count,
                "SLA monitoring completed - escalations sent"
            );
        }

        Ok(breach_count)
    }

    /// Get SLA status for an entity (not breached yet, but approaching deadline)
    pub async fn get_sla_status(&self, entity_id: Uuid) -> Result<SlaStatus, SlaError> {
        let client = self.db_pool.get().await?;

        // Get current state and when it was entered
        let query = r#"
            SELECT
                kb.status,
                a.created_at as state_entered_at
            FROM perlengkapan.kebutuhan_bmn kb
            LEFT JOIN LATERAL (
                SELECT created_at
                FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas
                WHERE pengajuan_id = kb.id
                ORDER BY created_at DESC
                LIMIT 1
            ) a ON true
            WHERE kb.id = $1
        "#;

        let row = client
            .query_opt(query, &[&entity_id])
            .await?
            .ok_or_else(|| SlaError::EntityNotFound(entity_id))?;

        let current_state: String = row.get("status");
        let state_entered_at: DateTime<Utc> = row.get("state_entered_at");

        // Get SLA limit for this state
        let sla_limit_minutes = match self.config.get_sla_minutes(&current_state) {
            Some(minutes) => minutes,
            None => {
                // No SLA defined for this state
                return Ok(SlaStatus::NoSla);
            }
        };

        // Calculate elapsed time and remaining time
        let now = Utc::now();
        let elapsed = now.signed_duration_since(state_entered_at);
        let _elapsed_minutes = elapsed.num_minutes();

        let sla_deadline = state_entered_at + Duration::minutes(sla_limit_minutes as i64);
        let remaining = sla_deadline.signed_duration_since(now);
        let remaining_minutes = remaining.num_minutes();

        if remaining_minutes < 0 {
            // SLA breached
            Ok(SlaStatus::Breached {
                breach_duration_minutes: -remaining_minutes,
            })
        } else if remaining_minutes < (sla_limit_minutes as i64 / 4) {
            // Less than 25% time remaining - critical
            Ok(SlaStatus::Critical {
                remaining_minutes: remaining_minutes as u32,
            })
        } else if remaining_minutes < (sla_limit_minutes as i64 / 2) {
            // Less than 50% time remaining - warning
            Ok(SlaStatus::Warning {
                remaining_minutes: remaining_minutes as u32,
            })
        } else {
            // Normal
            Ok(SlaStatus::Normal {
                remaining_minutes: remaining_minutes as u32,
            })
        }
    }
}

/// SLA status for an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum SlaStatus {
    /// No SLA defined for current state
    NoSla,

    /// SLA is normal (more than 50% time remaining)
    Normal { remaining_minutes: u32 },

    /// SLA is in warning state (25-50% time remaining)
    Warning { remaining_minutes: u32 },

    /// SLA is in critical state (less than 25% time remaining)
    Critical { remaining_minutes: u32 },

    /// SLA has been breached
    Breached { breach_duration_minutes: i64 },
}

/// SLA monitoring errors
#[derive(Debug, thiserror::Error)]
pub enum SlaError {
    #[error("Entity not found: {0}")]
    EntityNotFound(Uuid),

    #[error("Database error: {0}")]
    DatabaseError(#[from] tokio_postgres::Error),

    #[error("Pool error: {0}")]
    PoolError(#[from] deadpool_postgres::PoolError),

    #[error("Notification error: {0}")]
    NotificationError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::config::WorkflowConfig;

    #[test]
    fn test_sla_status_serialization() {
        let status = SlaStatus::Normal {
            remaining_minutes: 120,
        };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("normal"));
        assert!(json.contains("120"));

        let status = SlaStatus::Breached {
            breach_duration_minutes: 30,
        };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("breached"));
        assert!(json.contains("30"));
    }
}
