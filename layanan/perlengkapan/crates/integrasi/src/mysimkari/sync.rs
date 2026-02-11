use crate::client::MonsaktiClient;
use crate::error::MonsaktiError;
use crate::mysimkari::api::{
    get_pegawai_aktif, get_pegawai_mutasi, get_satker, MySIMKARICircuitBreaker,
};
use crate::storage::SyncStatusStorage;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info, warn};

/// Sync status for MySIMKARI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MySIMKARISyncStatus {
    pub last_full_sync: Option<DateTime<Utc>>,
    pub last_incremental_sync: Option<DateTime<Utc>>,
    pub last_sync_success: bool,
    pub last_sync_error: Option<String>,
    pub total_satker_synced: u64,
    pub total_pegawai_synced: u64,
    pub sync_in_progress: bool,
}

impl Default for MySIMKARISyncStatus {
    fn default() -> Self {
        Self {
            last_full_sync: None,
            last_incremental_sync: None,
            last_sync_success: false,
            last_sync_error: None,
            total_satker_synced: 0,
            total_pegawai_synced: 0,
            sync_in_progress: false,
        }
    }
}

/// MySIMKARI sync service with scheduler
pub struct MySIMKARISyncService {
    client: Arc<Mutex<MonsaktiClient>>,
    circuit_breaker: MySIMKARICircuitBreaker,
    storage: Arc<SyncStatusStorage>,
    status: Arc<Mutex<MySIMKARISyncStatus>>,
}

impl MySIMKARISyncService {
    pub fn new(
        client: MonsaktiClient,
        storage: SyncStatusStorage,
    ) -> Self {
        Self {
            client: Arc::new(Mutex::new(client)),
            circuit_breaker: MySIMKARICircuitBreaker::new(),
            storage: Arc::new(storage),
            status: Arc::new(Mutex::new(MySIMKARISyncStatus::default())),
        }
    }

    /// Start the scheduler for automatic syncs
    pub async fn start_scheduler(&self) -> Result<(), MonsaktiError> {
        let scheduler = JobScheduler::new()
            .await
            .map_err(|e| MonsaktiError::ConfigError(format!("Failed to create scheduler: {}", e)))?;

        // Full sync daily at 03:00 WIB (UTC+7 = 20:00 UTC previous day)
        let full_sync_job = self.create_full_sync_job();
        scheduler
            .add(full_sync_job)
            .await
            .map_err(|e| MonsaktiError::ConfigError(format!("Failed to add full sync job: {}", e)))?;

        // Incremental sync every 4 hours
        let incremental_sync_job = self.create_incremental_sync_job();
        scheduler
            .add(incremental_sync_job)
            .await
            .map_err(|e| MonsaktiError::ConfigError(format!("Failed to add incremental sync job: {}", e)))?;

        scheduler
            .start()
            .await
            .map_err(|e| MonsaktiError::ConfigError(format!("Failed to start scheduler: {}", e)))?;

        info!("MySIMKARI sync scheduler started");
        info!("  - Full sync: Daily at 03:00 WIB (20:00 UTC)");
        info!("  - Incremental sync: Every 4 hours");

        Ok(())
    }

    fn create_full_sync_job(&self) -> Job {
        let client = Arc::clone(&self.client);
        let circuit_breaker = self.circuit_breaker.clone();
        let storage = Arc::clone(&self.storage);
        let status = Arc::clone(&self.status);

        Job::new_async("0 0 20 * * *", move |_uuid, _l| {
            let client = Arc::clone(&client);
            let circuit_breaker = circuit_breaker.clone();
            let storage = Arc::clone(&storage);
            let status = Arc::clone(&status);

            Box::pin(async move {
                info!("Starting MySIMKARI full sync (scheduled)");
                let mut client_guard = client.lock().await;
                let mut status_guard = status.lock().await;

                match Self::execute_full_sync(
                    &mut *client_guard,
                    &circuit_breaker,
                    &storage,
                    &mut *status_guard,
                )
                .await
                {
                    Ok(_) => info!("MySIMKARI full sync completed successfully"),
                    Err(e) => error!("MySIMKARI full sync failed: {:?}", e),
                }
            })
        })
        .expect("Failed to create full sync job")
    }

    fn create_incremental_sync_job(&self) -> Job {
        let client = Arc::clone(&self.client);
        let circuit_breaker = self.circuit_breaker.clone();
        let storage = Arc::clone(&self.storage);
        let status = Arc::clone(&self.status);

        Job::new_async("0 0 */4 * * *", move |_uuid, _l| {
            let client = Arc::clone(&client);
            let circuit_breaker = circuit_breaker.clone();
            let storage = Arc::clone(&storage);
            let status = Arc::clone(&status);

            Box::pin(async move {
                info!("Starting MySIMKARI incremental sync (scheduled)");
                let mut client_guard = client.lock().await;
                let mut status_guard = status.lock().await;

                match Self::execute_incremental_sync(
                    &mut *client_guard,
                    &circuit_breaker,
                    &storage,
                    &mut *status_guard,
                )
                .await
                {
                    Ok(_) => info!("MySIMKARI incremental sync completed successfully"),
                    Err(e) => error!("MySIMKARI incremental sync failed: {:?}", e),
                }
            })
        })
        .expect("Failed to create incremental sync job")
    }

    /// Execute full sync manually
    pub async fn full_sync(&self) -> Result<(), MonsaktiError> {
        let mut client = self.client.lock().await;
        let mut status = self.status.lock().await;

        Self::execute_full_sync(&mut *client, &self.circuit_breaker, &self.storage, &mut *status)
            .await
    }

    /// Execute incremental sync manually
    pub async fn incremental_sync(&self) -> Result<(), MonsaktiError> {
        let mut client = self.client.lock().await;
        let mut status = self.status.lock().await;

        Self::execute_incremental_sync(
            &mut *client,
            &self.circuit_breaker,
            &self.storage,
            &mut *status,
        )
        .await
    }

    async fn execute_full_sync(
        client: &mut MonsaktiClient,
        circuit_breaker: &MySIMKARICircuitBreaker,
        storage: &SyncStatusStorage,
        status: &mut MySIMKARISyncStatus,
    ) -> Result<(), MonsaktiError> {
        if status.sync_in_progress {
            warn!("MySIMKARI sync already in progress, skipping");
            return Ok(());
        }

        status.sync_in_progress = true;
        let sync_start = Utc::now();

        info!("Starting MySIMKARI full sync");

        // Reset counters
        status.total_satker_synced = 0;
        status.total_pegawai_synced = 0;

        // Step 1: Sync satker data
        match Self::sync_satker_data(client, circuit_breaker, storage).await {
            Ok(count) => {
                info!("Synced {} satker records", count);
                status.total_satker_synced = count;
            }
            Err(e) => {
                error!("Failed to sync satker data: {:?}", e);
                status.last_sync_success = false;
                status.last_sync_error = Some(format!("Satker sync failed: {}", e));
                status.sync_in_progress = false;
                return Err(e);
            }
        }

        // Step 2: Sync all active pegawai data
        match Self::sync_all_pegawai_data(client, circuit_breaker, storage).await {
            Ok(count) => {
                info!("Synced {} pegawai records", count);
                status.total_pegawai_synced = count;
            }
            Err(e) => {
                error!("Failed to sync pegawai data: {:?}", e);
                status.last_sync_success = false;
                status.last_sync_error = Some(format!("Pegawai sync failed: {}", e));
                status.sync_in_progress = false;
                return Err(e);
            }
        }

        // Update status
        status.last_full_sync = Some(sync_start);
        status.last_sync_success = true;
        status.last_sync_error = None;
        status.sync_in_progress = false;

        // Save sync status to storage
        storage
            .save_sync_status("mysimkari", "full_sync", true, None)
            .await?;

        let duration = Utc::now() - sync_start;
        info!(
            "MySIMKARI full sync completed in {} seconds",
            duration.num_seconds()
        );

        Ok(())
    }

    async fn execute_incremental_sync(
        client: &mut MonsaktiClient,
        circuit_breaker: &MySIMKARICircuitBreaker,
        storage: &SyncStatusStorage,
        status: &mut MySIMKARISyncStatus,
    ) -> Result<(), MonsaktiError> {
        if status.sync_in_progress {
            warn!("MySIMKARI sync already in progress, skipping");
            return Ok(());
        }

        status.sync_in_progress = true;
        let sync_start = Utc::now();

        info!("Starting MySIMKARI incremental sync");

        // Get last sync time
        let last_sync = status
            .last_incremental_sync
            .or(status.last_full_sync)
            .unwrap_or_else(|| Utc::now() - Duration::hours(4));

        // Sync pegawai mutations since last sync
        let start_date = last_sync.format("%Y-%m-%d").to_string();
        let end_date = Utc::now().format("%Y-%m-%d").to_string();

        match Self::sync_pegawai_mutations(
            client,
            circuit_breaker,
            storage,
            &start_date,
            &end_date,
        )
        .await
        {
            Ok(count) => {
                info!("Synced {} pegawai mutations", count);
                status.total_pegawai_synced += count;
            }
            Err(e) => {
                error!("Failed to sync pegawai mutations: {:?}", e);
                status.last_sync_success = false;
                status.last_sync_error = Some(format!("Incremental sync failed: {}", e));
                status.sync_in_progress = false;
                return Err(e);
            }
        }

        // Update status
        status.last_incremental_sync = Some(sync_start);
        status.last_sync_success = true;
        status.last_sync_error = None;
        status.sync_in_progress = false;

        // Save sync status to storage
        storage
            .save_sync_status("mysimkari", "incremental_sync", true, None)
            .await?;

        let duration = Utc::now() - sync_start;
        info!(
            "MySIMKARI incremental sync completed in {} seconds",
            duration.num_seconds()
        );

        Ok(())
    }

    async fn sync_satker_data(
        client: &mut MonsaktiClient,
        circuit_breaker: &MySIMKARICircuitBreaker,
        storage: &SyncStatusStorage,
    ) -> Result<u64, MonsaktiError> {
        info!("Fetching satker data from MySIMKARI");

        let data = get_satker(client, circuit_breaker).await?;

        // Save to database
        let count = storage
            .save_mysimkari_data("satker", &data)
            .await?;

        Ok(count)
    }

    async fn sync_all_pegawai_data(
        client: &mut MonsaktiClient,
        circuit_breaker: &MySIMKARICircuitBreaker,
        storage: &SyncStatusStorage,
    ) -> Result<u64, MonsaktiError> {
        info!("Fetching all active pegawai data from MySIMKARI");

        let data = get_pegawai_aktif(client, circuit_breaker).await?;

        // Save to database
        let count = storage
            .save_mysimkari_data("pegawai", &data)
            .await?;

        Ok(count)
    }

    async fn sync_pegawai_mutations(
        client: &mut MonsaktiClient,
        circuit_breaker: &MySIMKARICircuitBreaker,
        storage: &SyncStatusStorage,
        start_date: &str,
        end_date: &str,
    ) -> Result<u64, MonsaktiError> {
        info!(
            "Fetching pegawai mutations from {} to {}",
            start_date, end_date
        );

        let data = get_pegawai_mutasi(client, start_date, end_date, circuit_breaker).await?;

        // Save to database (upsert based on NIP)
        let count = storage
            .save_mysimkari_data("pegawai_mutations", &data)
            .await?;

        Ok(count)
    }

    /// Get current sync status
    pub async fn get_status(&self) -> MySIMKARISyncStatus {
        self.status.lock().await.clone()
    }

    /// Get circuit breaker state
    pub async fn get_circuit_breaker_state(&self) -> String {
        format!("{:?}", self.circuit_breaker.get_state().await)
    }
}
