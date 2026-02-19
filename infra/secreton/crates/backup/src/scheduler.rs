//! Backup scheduler for automated backups

use chrono::Utc;
use cron::Schedule;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};

use crate::error::{BackupError, Result};
use crate::manager::BackupManager;

/// Backup scheduler for automated backups
pub struct BackupScheduler {
    schedule: Schedule,
    manager: Arc<BackupManager>,
    running: Arc<RwLock<bool>>,
}

impl BackupScheduler {
    /// Create a new backup scheduler
    pub fn new(cron_expression: &str, manager: Arc<BackupManager>) -> Result<Self> {
        let schedule = Schedule::from_str(cron_expression).map_err(|e| {
            BackupError::Scheduler(format!("Invalid cron expression '{}': {}", cron_expression, e))
        })?;

        Ok(Self {
            schedule,
            manager,
            running: Arc::new(RwLock::new(false)),
        })
    }

    /// Start the scheduler
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Err(BackupError::Scheduler(
                "Scheduler is already running".to_string(),
            ));
        }
        *running = true;
        drop(running);

        tracing::info!("Backup scheduler started");

        // Spawn background task
        let schedule = self.schedule.clone();
        let manager = self.manager.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            loop {
                // Check if still running
                if !*running.read().await {
                    tracing::info!("Backup scheduler stopped");
                    break;
                }

                // Get next scheduled time
                let now = Utc::now();
                let next = match schedule.upcoming(Utc).next() {
                    Some(next) => next,
                    None => {
                        tracing::error!("Failed to get next scheduled time");
                        sleep(Duration::from_secs(60)).await;
                        continue;
                    }
                };

                // Calculate duration until next backup
                let duration = match (next - now).to_std() {
                    Ok(d) => d,
                    Err(e) => {
                        tracing::error!("Failed to calculate duration: {}", e);
                        sleep(Duration::from_secs(60)).await;
                        continue;
                    }
                };

                tracing::info!(
                    next_backup = %next,
                    duration_seconds = duration.as_secs(),
                    "Next backup scheduled"
                );

                // Sleep until next backup time
                sleep(duration).await;

                // Check if still running after sleep
                if !*running.read().await {
                    break;
                }

                // Execute backup
                tracing::info!("Starting scheduled backup");
                match manager.create_backup().await {
                    Ok(backup_id) => {
                        tracing::info!(
                            backup_id = %backup_id,
                            "Scheduled backup completed successfully"
                        );
                    }
                    Err(e) => {
                        tracing::error!(
                            error = %e,
                            "Scheduled backup failed"
                        );
                        // TODO: Send alert/notification
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop the scheduler
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if !*running {
            return Err(BackupError::Scheduler(
                "Scheduler is not running".to_string(),
            ));
        }
        *running = false;

        tracing::info!("Backup scheduler stopping");

        Ok(())
    }

    /// Check if scheduler is running
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// Get the next scheduled backup time
    pub fn next_backup_time(&self) -> Option<chrono::DateTime<Utc>> {
        self.schedule.upcoming(Utc).next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BackupConfig;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_scheduler_creation() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = BackupConfig::default();
        config.storage_config = crate::types::StorageConfig::Local(
            crate::types::LocalStorageConfig {
                path: temp_dir.path().to_string_lossy().to_string(),
            },
        );

        let manager = Arc::new(BackupManager::new(config).await.unwrap());

        // Valid cron expression
        let scheduler = BackupScheduler::new("0 2 * * *", manager.clone());
        assert!(scheduler.is_ok());

        // Invalid cron expression
        let scheduler = BackupScheduler::new("invalid", manager);
        assert!(scheduler.is_err());
    }

    #[tokio::test]
    async fn test_next_backup_time() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = BackupConfig::default();
        config.storage_config = crate::types::StorageConfig::Local(
            crate::types::LocalStorageConfig {
                path: temp_dir.path().to_string_lossy().to_string(),
            },
        );

        let manager = Arc::new(BackupManager::new(config).await.unwrap());
        let scheduler = BackupScheduler::new("0 2 * * *", manager).unwrap();

        let next = scheduler.next_backup_time();
        assert!(next.is_some());

        let next_time = next.unwrap();
        assert!(next_time > Utc::now());
    }

    #[tokio::test]
    async fn test_scheduler_start_stop() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = BackupConfig::default();
        config.storage_config = crate::types::StorageConfig::Local(
            crate::types::LocalStorageConfig {
                path: temp_dir.path().to_string_lossy().to_string(),
            },
        );

        let manager = Arc::new(BackupManager::new(config).await.unwrap());
        let scheduler = BackupScheduler::new("0 2 * * *", manager).unwrap();

        // Initially not running
        assert!(!scheduler.is_running().await);

        // Start
        scheduler.start().await.unwrap();
        assert!(scheduler.is_running().await);

        // Cannot start twice
        assert!(scheduler.start().await.is_err());

        // Stop
        scheduler.stop().await.unwrap();

        // Give it a moment to stop
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Cannot stop twice
        assert!(scheduler.stop().await.is_err());
    }
}
