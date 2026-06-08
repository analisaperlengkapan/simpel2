// ============================================================================
// SLA Escalation Scheduler Module
// Description: Periodic scheduler for SLA breach detection and auto-escalation
// Requirements: REQ-W003, REQ-N008
// ============================================================================

use crate::contracts::NotificationSender;
use crate::workflow::config::WorkflowConfig;
use crate::workflow::sla::SlaMonitor;
use deadpool_postgres::Pool;
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info, warn};

/// SLA escalation scheduler configuration
#[derive(Debug, Clone)]
pub struct SlaSchedulerConfig {
    /// Cron expression for SLA check interval (default: every 15 minutes)
    pub check_interval_cron: String,

    /// Whether to enable the scheduler
    pub enabled: bool,
}

impl Default for SlaSchedulerConfig {
    fn default() -> Self {
        Self {
            // Run every 15 minutes: "0 */15 * * * *" (sec min hour day month weekday)
            check_interval_cron: "0 */15 * * * *".to_string(),
            enabled: true,
        }
    }
}

impl SlaSchedulerConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let check_interval_cron = std::env::var("SLA_CHECK_INTERVAL_CRON")
            .unwrap_or_else(|_| "0 */15 * * * *".to_string());

        let enabled = std::env::var("SLA_SCHEDULER_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        Self {
            check_interval_cron,
            enabled,
        }
    }
}

/// SLA escalation scheduler
pub struct SlaEscalationScheduler {
    config: SlaSchedulerConfig,
    db_pool: Pool,
    notifier: Option<Arc<dyn NotificationSender>>,
}

impl SlaEscalationScheduler {
    /// Create a new SLA escalation scheduler
    pub fn new(db_pool: Pool) -> Self {
        Self {
            config: SlaSchedulerConfig::from_env(),
            db_pool,
            notifier: None,
        }
    }

    /// Create a new SLA escalation scheduler with custom configuration
    pub fn with_config(config: SlaSchedulerConfig, db_pool: Pool) -> Self {
        Self {
            config,
            db_pool,
            notifier: None,
        }
    }

    /// Inject the notification sender used for escalation notifications.
    pub fn with_notifier(mut self, notifier: Arc<dyn NotificationSender>) -> Self {
        self.notifier = Some(notifier);
        self
    }

    /// Start the scheduler
    ///
    /// This method spawns a background task that runs the SLA check job
    /// according to the configured cron schedule.
    ///
    /// Requirements: REQ-W003, REQ-N008
    pub fn start(self) -> anyhow::Result<()> {
        if !self.config.enabled {
            info!("SLA escalation scheduler is disabled");
            return Ok(());
        }

        info!(
            "Starting SLA escalation scheduler with interval: {}",
            self.config.check_interval_cron
        );

        let db_pool = self.db_pool.clone();
        let notifier = self.notifier.clone();
        let cron_expr = self.config.check_interval_cron.clone();

        tokio::spawn(async move {
            match Self::run_scheduler(db_pool, notifier, cron_expr).await {
                Ok(_) => {
                    info!("SLA escalation scheduler stopped gracefully");
                }
                Err(e) => {
                    error!("SLA escalation scheduler error: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Run the scheduler loop
    async fn run_scheduler(
        db_pool: Pool,
        notifier: Option<Arc<dyn NotificationSender>>,
        cron_expr: String,
    ) -> anyhow::Result<()> {
        let scheduler = JobScheduler::new().await?;

        // Create the SLA check job
        let db_pool_clone = db_pool.clone();
        let notifier_clone = notifier.clone();

        let job = Job::new_async(cron_expr.as_str(), move |_uuid, _lock| {
            let db_pool = db_pool_clone.clone();
            let notifier = notifier_clone.clone();

            Box::pin(async move {
                if let Err(e) = Self::check_and_escalate_sla_breaches(db_pool, notifier).await {
                    error!("SLA check job failed: {}", e);
                }
            })
        })?;

        scheduler.add(job).await?;
        scheduler.start().await?;

        info!("SLA escalation scheduler started successfully");

        // Keep the scheduler running
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    }

    /// Check for SLA breaches and escalate them
    ///
    /// This method:
    /// 1. Creates SLA monitors for each workflow type (Kebutuhan, Pemakaian, Penghapusan)
    /// 2. Checks all entities for SLA breaches
    /// 3. Escalates breached entities by sending notifications
    /// 4. Logs escalation activities
    ///
    /// Requirements: REQ-W003, REQ-N008, NFR-M004
    async fn check_and_escalate_sla_breaches(
        db_pool: Pool,
        notifier: Option<Arc<dyn NotificationSender>>,
    ) -> anyhow::Result<()> {
        info!("Starting SLA breach check and escalation");

        let start = std::time::Instant::now();
        let mut total_breaches = 0;
        let mut total_escalations = 0;

        // Check Kebutuhan BMN SLA breaches
        match Self::check_workflow_sla(
            "kebutuhan_bmn",
            WorkflowConfig::default_kebutuhan_bmn(),
            db_pool.clone(),
            notifier.clone(),
        )
        .await
        {
            Ok((breaches, escalations)) => {
                total_breaches += breaches;
                total_escalations += escalations;
                info!(
                    "Kebutuhan BMN: {} breaches detected, {} escalations sent",
                    breaches, escalations
                );
            }
            Err(e) => {
                error!("Failed to check Kebutuhan BMN SLA: {}", e);
            }
        }

        // Check Pemakaian BMN SLA breaches
        match Self::check_workflow_sla(
            "pemakaian_bmn",
            WorkflowConfig::default_pemakaian_bmn(),
            db_pool.clone(),
            notifier.clone(),
        )
        .await
        {
            Ok((breaches, escalations)) => {
                total_breaches += breaches;
                total_escalations += escalations;
                info!(
                    "Pemakaian BMN: {} breaches detected, {} escalations sent",
                    breaches, escalations
                );
            }
            Err(e) => {
                error!("Failed to check Pemakaian BMN SLA: {}", e);
            }
        }

        // Check Penghapusan BMN SLA breaches
        match Self::check_workflow_sla(
            "penghapusan_bmn",
            WorkflowConfig::default_penghapusan_bmn(),
            db_pool.clone(),
            notifier.clone(),
        )
        .await
        {
            Ok((breaches, escalations)) => {
                total_breaches += breaches;
                total_escalations += escalations;
                info!(
                    "Penghapusan BMN: {} breaches detected, {} escalations sent",
                    breaches, escalations
                );
            }
            Err(e) => {
                error!("Failed to check Penghapusan BMN SLA: {}", e);
            }
        }

        let duration = start.elapsed();

        if total_breaches > 0 {
            warn!(
                "SLA check completed: {} total breaches detected, {} escalations sent in {:?}",
                total_breaches, total_escalations, duration
            );
        } else {
            info!(
                "SLA check completed: No breaches detected in {:?}",
                duration
            );
        }

        // Record metrics
        crate::shared::metrics::workflow_sla_check_duration()
            .with_label_values(&["all"])
            .observe(duration.as_secs_f64());
        crate::shared::metrics::workflow_sla_check_total()
            .with_label_values(&["success"])
            .inc();

        Ok(())
    }

    /// Check SLA for a specific workflow type
    async fn check_workflow_sla(
        workflow_type: &str,
        config: WorkflowConfig,
        db_pool: Pool,
        notifier: Option<Arc<dyn NotificationSender>>,
    ) -> anyhow::Result<(usize, usize)> {
        // Create SLA monitor
        let monitor = if let Some(notifier) = notifier {
            SlaMonitor::with_notifier(config, db_pool, notifier)
        } else {
            SlaMonitor::new(config, db_pool)
        };

        // Check all SLA breaches
        let breaches = monitor
            .check_all_sla()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to check SLA for {}: {}", workflow_type, e))?;

        let breach_count = breaches.len();
        let mut escalation_count = 0;

        // Escalate each breach
        for breach in breaches {
            match monitor.escalate_sla_breach(&breach).await {
                Ok(_) => {
                    escalation_count += 1;
                }
                Err(e) => {
                    error!(
                        "Failed to escalate SLA breach for {} entity {}: {}",
                        workflow_type, breach.entity_id, e
                    );
                }
            }
        }

        Ok((breach_count, escalation_count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sla_scheduler_config_default() {
        let config = SlaSchedulerConfig::default();
        assert_eq!(config.check_interval_cron, "0 */15 * * * *");
        assert!(config.enabled);
    }

    #[test]
    fn test_sla_scheduler_config_from_env() {
        unsafe {
            std::env::set_var("SLA_CHECK_INTERVAL_CRON", "0 */30 * * * *");
            std::env::set_var("SLA_SCHEDULER_ENABLED", "false");
        }

        let config = SlaSchedulerConfig::from_env();
        assert_eq!(config.check_interval_cron, "0 */30 * * * *");
        assert!(!config.enabled);

        // Cleanup
        unsafe {
            std::env::remove_var("SLA_CHECK_INTERVAL_CRON");
            std::env::remove_var("SLA_SCHEDULER_ENABLED");
        }
    }
}
