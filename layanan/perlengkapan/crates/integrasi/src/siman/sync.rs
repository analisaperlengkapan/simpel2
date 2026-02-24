//! SIMAN Sync Service
//!
//! This module provides scheduled synchronization of SIMAN asset data with the local database.
//! It implements:
//! - Full sync (daily at 02:00 WIB)
//! - Incremental sync (every 6 hours)
//! - Sync status tracking
//! - Error handling and logging

use crate::client::MonsaktiClient;
use crate::error::MonsaktiError;
use crate::siman::endpoints::{CircuitBreaker, fetch_all_assets_with_pagination};
use crate::siman::models::SimanAssetCategory;
use crate::storage::StorageStrategy;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info, warn};

/// Sync status for tracking synchronization state
#[derive(Debug, Clone)]
pub struct SyncStatus {
    /// Last successful sync timestamp
    pub last_sync: Option<DateTime<Utc>>,
    /// Last sync attempt timestamp
    pub last_attempt: DateTime<Utc>,
    /// Whether the last sync was successful
    pub success: bool,
    /// Error message if sync failed
    pub error_message: Option<String>,
    /// Number of records synced
    pub records_synced: usize,
    /// Number of records failed
    pub records_failed: usize,
}

impl SyncStatus {
    /// Create a new sync status
    pub fn new() -> Self {
        Self {
            last_sync: None,
            last_attempt: Utc::now(),
            success: false,
            error_message: None,
            records_synced: 0,
            records_failed: 0,
        }
    }

    /// Mark sync as successful
    pub fn mark_success(&mut self, records_synced: usize, records_failed: usize) {
        self.last_sync = Some(Utc::now());
        self.last_attempt = Utc::now();
        self.success = true;
        self.error_message = None;
        self.records_synced = records_synced;
        self.records_failed = records_failed;
    }

    /// Mark sync as failed
    pub fn mark_failure(&mut self, error: String) {
        self.last_attempt = Utc::now();
        self.success = false;
        self.error_message = Some(error);
    }
}

impl Default for SyncStatus {
    fn default() -> Self {
        Self::new()
    }
}

/// SIMAN Sync Service
///
/// Manages scheduled synchronization of SIMAN asset data
pub struct SimanSyncService {
    client: Arc<Mutex<MonsaktiClient>>,
    storage: Arc<StorageStrategy>,
    scheduler: JobScheduler,
    sync_status: Arc<Mutex<SyncStatus>>,
    circuit_breaker: Arc<CircuitBreaker>,
}

impl SimanSyncService {
    /// Create a new SIMAN sync service
    ///
    /// # Arguments
    /// * `client` - MonsaktiClient for API calls
    /// * `storage` - Storage strategy for persisting data
    pub async fn new(
        client: MonsaktiClient,
        storage: StorageStrategy,
    ) -> Result<Self, MonsaktiError> {
        let scheduler = JobScheduler::new()
            .await
            .map_err(|e| MonsaktiError::ApiError(format!("Failed to create scheduler: {}", e)))?;

        let circuit_breaker = Arc::new(CircuitBreaker::new(
            5,                        // failure threshold
            Duration::from_secs(300), // 5 minute timeout
        ));

        Ok(Self {
            client: Arc::new(Mutex::new(client)),
            storage: Arc::new(storage),
            scheduler,
            sync_status: Arc::new(Mutex::new(SyncStatus::new())),
            circuit_breaker,
        })
    }

    /// Start the sync service with scheduled jobs
    pub async fn start(&mut self) -> Result<(), MonsaktiError> {
        info!("Starting SIMAN Sync Service...");

        // Add full sync job (daily at 02:00 WIB)
        self.add_full_sync_job().await?;

        // Add incremental sync job (every 6 hours)
        self.add_incremental_sync_job().await?;

        // Start the scheduler
        self.scheduler
            .start()
            .await
            .map_err(|e| MonsaktiError::ApiError(format!("Failed to start scheduler: {}", e)))?;

        info!("✅ SIMAN Sync Service started successfully");
        Ok(())
    }

    /// Add full sync job (daily at 02:00 WIB)
    async fn add_full_sync_job(&mut self) -> Result<(), MonsaktiError> {
        // Cron expression for 02:00 WIB (UTC+7)
        // 02:00 WIB = 19:00 UTC (previous day)
        let schedule = "0 0 19 * * *"; // 19:00 UTC = 02:00 WIB

        info!("📅 Scheduling SIMAN full sync: {} (02:00 WIB)", schedule);

        let client = Arc::clone(&self.client);
        let storage = Arc::clone(&self.storage);
        let sync_status = Arc::clone(&self.sync_status);
        let circuit_breaker = Arc::clone(&self.circuit_breaker);

        let job = Job::new_async(schedule, move |_uuid, _lock| {
            let client = Arc::clone(&client);
            let storage = Arc::clone(&storage);
            let sync_status = Arc::clone(&sync_status);
            let circuit_breaker = Arc::clone(&circuit_breaker);

            Box::pin(async move {
                info!("🚀 Starting SIMAN full sync...");

                match Self::perform_full_sync(client, storage, circuit_breaker).await {
                    Ok((success_count, failed_count)) => {
                        let mut status = sync_status.lock().await;
                        status.mark_success(success_count, failed_count);
                        info!(
                            "✅ SIMAN full sync completed: {} success, {} failed",
                            success_count, failed_count
                        );
                    }
                    Err(e) => {
                        let mut status = sync_status.lock().await;
                        status.mark_failure(e.to_string());
                        error!("❌ SIMAN full sync failed: {}", e);
                    }
                }
            })
        })
        .map_err(|e| MonsaktiError::ApiError(format!("Failed to create full sync job: {}", e)))?;

        self.scheduler
            .add(job)
            .await
            .map_err(|e| MonsaktiError::ApiError(format!("Failed to add full sync job: {}", e)))?;

        Ok(())
    }

    /// Add incremental sync job (every 6 hours)
    async fn add_incremental_sync_job(&mut self) -> Result<(), MonsaktiError> {
        // Cron expression for every 6 hours
        let schedule = "0 0 */6 * * *";

        info!("📅 Scheduling SIMAN incremental sync: {}", schedule);

        let client = Arc::clone(&self.client);
        let storage = Arc::clone(&self.storage);
        let sync_status = Arc::clone(&self.sync_status);
        let circuit_breaker = Arc::clone(&self.circuit_breaker);

        let job = Job::new_async(schedule, move |_uuid, _lock| {
            let client = Arc::clone(&client);
            let storage = Arc::clone(&storage);
            let sync_status = Arc::clone(&sync_status);
            let circuit_breaker = Arc::clone(&circuit_breaker);

            Box::pin(async move {
                info!("🚀 Starting SIMAN incremental sync...");

                match Self::perform_incremental_sync(client, storage, circuit_breaker).await {
                    Ok((success_count, failed_count)) => {
                        let mut status = sync_status.lock().await;
                        status.mark_success(success_count, failed_count);
                        info!(
                            "✅ SIMAN incremental sync completed: {} success, {} failed",
                            success_count, failed_count
                        );
                    }
                    Err(e) => {
                        let mut status = sync_status.lock().await;
                        status.mark_failure(e.to_string());
                        error!("❌ SIMAN incremental sync failed: {}", e);
                    }
                }
            })
        })
        .map_err(|e| {
            MonsaktiError::ApiError(format!("Failed to create incremental sync job: {}", e))
        })?;

        self.scheduler.add(job).await.map_err(|e| {
            MonsaktiError::ApiError(format!("Failed to add incremental sync job: {}", e))
        })?;

        Ok(())
    }

    /// Perform full sync of all SIMAN categories
    async fn perform_full_sync(
        client: Arc<Mutex<MonsaktiClient>>,
        storage: Arc<StorageStrategy>,
        circuit_breaker: Arc<CircuitBreaker>,
    ) -> Result<(usize, usize), MonsaktiError> {
        info!("📊 Starting full sync for all SIMAN categories");

        let categories = SimanAssetCategory::all();
        let mut total_success = 0;
        let mut total_failed = 0;

        for category in categories {
            // Check circuit breaker before each category
            if let Err(e) = circuit_breaker.can_proceed().await {
                warn!(
                    "Circuit breaker open, skipping category {}: {}",
                    category.description(),
                    e
                );
                continue;
            }

            info!("  → Syncing category: {}", category.description());

            let mut client_guard = client.lock().await;
            match fetch_all_assets_with_pagination(&mut *client_guard, &storage, category).await {
                Ok((success, failed)) => {
                    total_success += success;
                    total_failed += failed;
                    circuit_breaker.record_success().await;
                    info!(
                        "  ✓ {}: {} success, {} failed",
                        category.description(),
                        success,
                        failed
                    );
                }
                Err(e) => {
                    circuit_breaker.record_failure().await;
                    error!("  ✗ {}: {}", category.description(), e);
                    total_failed += 1;
                }
            }
        }

        info!(
            "📊 Full sync completed: {} total success, {} total failed",
            total_success, total_failed
        );

        Ok((total_success, total_failed))
    }

    /// Perform incremental sync (only changed data)
    ///
    /// Note: SIMAN API doesn't support incremental sync natively,
    /// so we implement a lightweight version by syncing only high-priority categories
    async fn perform_incremental_sync(
        client: Arc<Mutex<MonsaktiClient>>,
        storage: Arc<StorageStrategy>,
        circuit_breaker: Arc<CircuitBreaker>,
    ) -> Result<(usize, usize), MonsaktiError> {
        info!("📊 Starting incremental sync for high-priority SIMAN categories");

        // High-priority categories for incremental sync
        let categories = vec![
            SimanAssetCategory::Tanah,
            SimanAssetCategory::GedungBangunan,
            SimanAssetCategory::AngkutanBermotor,
            SimanAssetCategory::NonTIK,
            SimanAssetCategory::KhususTIK,
        ];

        let mut total_success = 0;
        let mut total_failed = 0;

        for category in categories {
            // Check circuit breaker before each category
            if let Err(e) = circuit_breaker.can_proceed().await {
                warn!(
                    "Circuit breaker open, skipping category {}: {}",
                    category.description(),
                    e
                );
                continue;
            }

            info!("  → Syncing category: {}", category.description());

            let mut client_guard = client.lock().await;
            match fetch_all_assets_with_pagination(&mut *client_guard, &storage, category).await {
                Ok((success, failed)) => {
                    total_success += success;
                    total_failed += failed;
                    circuit_breaker.record_success().await;
                    info!(
                        "  ✓ {}: {} success, {} failed",
                        category.description(),
                        success,
                        failed
                    );
                }
                Err(e) => {
                    circuit_breaker.record_failure().await;
                    error!("  ✗ {}: {}", category.description(), e);
                    total_failed += 1;
                }
            }
        }

        info!(
            "📊 Incremental sync completed: {} total success, {} total failed",
            total_success, total_failed
        );

        Ok((total_success, total_failed))
    }

    /// Get current sync status
    pub async fn get_sync_status(&self) -> SyncStatus {
        self.sync_status.lock().await.clone()
    }

    /// Trigger manual sync
    pub async fn trigger_manual_sync(&self) -> Result<(usize, usize), MonsaktiError> {
        info!("🔄 Manual sync triggered");
        Self::perform_full_sync(
            Arc::clone(&self.client),
            Arc::clone(&self.storage),
            Arc::clone(&self.circuit_breaker),
        )
        .await
    }

    /// Shutdown the sync service
    pub async fn shutdown(mut self) -> Result<(), MonsaktiError> {
        info!("Shutting down SIMAN Sync Service...");
        self.scheduler
            .shutdown()
            .await
            .map_err(|e| MonsaktiError::ApiError(format!("Failed to shutdown scheduler: {}", e)))?;
        info!("✅ SIMAN Sync Service shutdown complete");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_status_creation() {
        let status = SyncStatus::new();
        assert!(!status.success);
        assert!(status.last_sync.is_none());
        assert_eq!(status.records_synced, 0);
        assert_eq!(status.records_failed, 0);
    }

    #[test]
    fn test_sync_status_mark_success() {
        let mut status = SyncStatus::new();
        status.mark_success(100, 5);

        assert!(status.success);
        assert!(status.last_sync.is_some());
        assert_eq!(status.records_synced, 100);
        assert_eq!(status.records_failed, 5);
        assert!(status.error_message.is_none());
    }

    #[test]
    fn test_sync_status_mark_failure() {
        let mut status = SyncStatus::new();
        status.mark_failure("Test error".to_string());

        assert!(!status.success);
        assert_eq!(status.error_message, Some("Test error".to_string()));
    }
}
