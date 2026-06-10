//! Scheduler module for automated data fetching
//!
//! This module handles scheduled data fetching from external APIs:
//! - MonSAKTI: Scheduled daily at 02:00 WIB (configurable)
//! - MySIMKARI: Scheduled daily at 02:00 WIB (configurable)
//! - SIMAN: Scheduled every Sunday at 03:00 WIB (configurable)

use crate::client::MonsaktiClient;
use crate::config::Config;
use crate::mysimkari;
// use crate::siman;
use crate::siman::endpoints::fetch_all_assets_with_pagination;
use crate::storage_from_env;
use anyhow::Result;
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};
use tokio_postgres::Client;
use tracing::{error, info, warn};

/// Source of data for manual synchronization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataSource {
    All,
    Monsakti,
    Mysimkari,
    Siman,
}

/// Scheduler for automated data fetching
pub struct IntegrationScheduler {
    config: Config,
    db_client: Option<Arc<Client>>,
    scheduler: JobScheduler,
}

impl IntegrationScheduler {
    /// Create a new scheduler instance
    pub async fn new(config: Config) -> Result<Self> {
        let client = MonsaktiClient::new(config.clone()).await?;
        let db_client = client.get_db_client_arc();
        let scheduler = JobScheduler::new().await?;

        Ok(Self {
            config,
            db_client,
            scheduler,
        })
    }

    /// Initialize and start all scheduled jobs
    pub async fn start(&mut self) -> Result<()> {
        if !self.config.scheduler_enabled {
            warn!("Scheduler is disabled in configuration");
            return Ok(());
        }

        info!("Starting Integration Scheduler...");
        info!("Timezone: {}", self.config.scheduler_timezone);

        // Add MonSAKTI job
        self.add_monsakti_job().await?;

        // Add MySIMKARI job
        self.add_mysimkari_job().await?;

        // Add SIMAN job
        self.add_siman_job().await?;

        // Start the scheduler
        self.scheduler.start().await?;

        info!("✅ Scheduler started successfully");
        Ok(())
    }

    /// Add MonSAKTI scheduled job
    async fn add_monsakti_job(&mut self) -> Result<()> {
        let schedule = self.config.monsakti_schedule.clone();
        info!("📅 Scheduling MonSAKTI data fetch: {}", schedule);

        let config = self.config.clone();

        let db_client = self.db_client.clone();
        let job = Job::new_async(schedule.as_str(), move |_uuid, _lock| {
            let config = config.clone();
            let db_client = db_client.clone();
            Box::pin(async move {
                info!("🚀 Starting MonSAKTI scheduled data fetch...");
                match fetch_monsakti_data(config, db_client).await {
                    Ok(_) => info!("✅ MonSAKTI data fetch completed successfully"),
                    Err(e) => error!("❌ MonSAKTI data fetch failed: {}", e),
                }
            })
        })?;

        self.scheduler.add(job).await?;
        Ok(())
    }

    /// Add MySIMKARI scheduled job
    async fn add_mysimkari_job(&mut self) -> Result<()> {
        let schedule = self.config.mysimkari_schedule.clone();
        info!("📅 Scheduling MySIMKARI data fetch: {}", schedule);

        let config = self.config.clone();

        let db_client = self.db_client.clone();
        let job = Job::new_async(schedule.as_str(), move |_uuid, _lock| {
            let config = config.clone();
            let db_client = db_client.clone();
            Box::pin(async move {
                info!("🚀 Starting MySIMKARI scheduled data fetch...");
                match fetch_mysimkari_data(config, db_client).await {
                    Ok(_) => info!("✅ MySIMKARI data fetch completed successfully"),
                    Err(e) => error!("❌ MySIMKARI data fetch failed: {}", e),
                }
            })
        })?;

        self.scheduler.add(job).await?;
        Ok(())
    }

    /// Add SIMAN scheduled job
    async fn add_siman_job(&mut self) -> Result<()> {
        let schedule = self.config.siman_schedule.clone();
        info!("📅 Scheduling SIMAN data fetch: {}", schedule);

        let config = self.config.clone();

        let db_client = self.db_client.clone();
        let job = Job::new_async(schedule.as_str(), move |_uuid, _lock| {
            let config = config.clone();
            let db_client = db_client.clone();
            Box::pin(async move {
                info!("🚀 Starting SIMAN scheduled data fetch...");
                match fetch_siman_data(config, db_client).await {
                    Ok(_) => info!("✅ SIMAN data fetch completed successfully"),
                    Err(e) => error!("❌ SIMAN data fetch failed: {}", e),
                }
            })
        })?;

        self.scheduler.add(job).await?;
        Ok(())
    }

    /// Shutdown the scheduler gracefully
    pub async fn shutdown(mut self) -> Result<()> {
        info!("Shutting down scheduler...");
        self.scheduler.shutdown().await?;
        info!("✅ Scheduler shutdown complete");
        Ok(())
    }

    /// Run a manual synchronization for a specific data source
    pub async fn run_manual_sync(&self, source: DataSource) -> Result<()> {
        info!("🚀 Starting manual data fetch for {:?}", source);
        let config = self.config.clone();

        match source {
            DataSource::Monsakti => {
                fetch_monsakti_data(config, self.db_client.clone()).await?;
            }
            DataSource::Mysimkari => {
                fetch_mysimkari_data(config, self.db_client.clone()).await?;
            }
            DataSource::Siman => {
                fetch_siman_data(config, self.db_client.clone()).await?;
            }
            DataSource::All => {
                info!("Executing full synchronization across all modules...");

                let c1 = config.clone();
                let c2 = config.clone();
                let c3 = config.clone();

                // Run sequentially to avoid overwhelming the database or network
                if let Err(e) = fetch_monsakti_data(c1, self.db_client.clone()).await {
                    error!("❌ MonSAKTI manual fetch failed: {}", e);
                }
                if let Err(e) = fetch_mysimkari_data(c2, self.db_client.clone()).await {
                    error!("❌ MySIMKARI manual fetch failed: {}", e);
                }
                if let Err(e) = fetch_siman_data(c3, self.db_client.clone()).await {
                    error!("❌ SIMAN manual fetch failed: {}", e);
                }
            }
        }

        info!("✅ Manual data fetch completed for {:?}", source);
        Ok(())
    }
}

/// Fetch all MonSAKTI data
async fn fetch_monsakti_data(config: Config, db_client: Option<Arc<Client>>) -> Result<()> {
    let mut client = if let Some(db) = db_client {
        MonsaktiClient::new_with_db(config.clone(), db).await?
    } else {
        MonsaktiClient::new(config.clone()).await?
    };

    info!("📊 Fetching MonSAKTI data for all modules...");

    // Fetch ADM module data
    if config.tokens.contains_key("ADM") {
        info!("  → Fetching ADM module data...");
        let adm_data = crate::monsakti::adm::ref_admin(&mut client, "006", "").await?;

        // Convert Value to array
        if let Some(array) = adm_data.as_array() {
            info!("  ✓ ADM: {} records", array.len());
            // Save to database
            if let Some(db) = client.get_db_client_arc() {
                crate::db::save_to_database(&db, "monsakti_adm", "adm/refAdmin", &adm_data).await?;
            }
        } else {
            info!("  ✓ ADM: 1 record");
            if let Some(db) = client.get_db_client_arc() {
                crate::db::save_to_database(&db, "monsakti_adm", "adm/refAdmin", &adm_data).await?;
            }
        }
    }

    // Fetch other modules similarly...
    // ANG, AST, BEN, GLP, KOM, PEM, PER

    Ok(())
}

/// Fetch all MySIMKARI data
async fn fetch_mysimkari_data(config: Config, db_client: Option<Arc<Client>>) -> Result<()> {
    let mut client = if let Some(db) = db_client {
        MonsaktiClient::new_with_db(config.clone(), db).await?
    } else {
        MonsaktiClient::new(config.clone()).await?
    };
    let circuit_breaker = mysimkari::api::MySIMKARICircuitBreaker::new();

    info!("📊 Fetching MySIMKARI data...");

    // Fetch satker data
    info!("  → Fetching satker data...");
    let satker_response = mysimkari::api::get_satker(&mut client, &circuit_breaker).await?;

    // Handle response - could be array or single object
    if let Some(satker_array) = satker_response.as_array() {
        info!("  ✓ Satker: {} records", satker_array.len());
        if let Some(db) = client.get_db_client_arc() {
            crate::db::save_to_database(&db, "mysimkari_satker", "satker", &satker_response)
                .await?;
        }

        // Fetch pegawai data for first 5 satker as sample
        info!("  → Fetching pegawai data for sample satker...");
        for satker in satker_array.iter().take(5) {
            if let Some(id) = satker.get("id").and_then(|v| v.as_str()) {
                match mysimkari::api::pegawai_satker(&mut client, id, &circuit_breaker).await {
                    Ok(pegawai_response) => {
                        if let Some(pegawai_array) = pegawai_response.as_array() {
                            info!(
                                "    ✓ Pegawai for satker {}: {} records",
                                id,
                                pegawai_array.len()
                            );
                            if let Some(db) = client.get_db_client_arc() {
                                crate::db::save_to_database(
                                    &db,
                                    "mysimkari_pegawai",
                                    "pegawai",
                                    &pegawai_response,
                                )
                                .await?;
                            }
                        }
                    }
                    Err(e) => {
                        warn!("    ⚠ Failed to fetch pegawai for satker {}: {}", id, e);
                    }
                }
            }
        }
    } else {
        info!("  ✓ Satker: 1 record");
        if let Some(db) = client.get_db_client_arc() {
            crate::db::save_to_database(&db, "mysimkari_satker", "satker", &satker_response)
                .await?;
        }
    }

    Ok(())
}

/// Fetch all SIMAN data
async fn fetch_siman_data(config: Config, db_client: Option<Arc<Client>>) -> Result<()> {
    let mut client = if let Some(db) = db_client {
        MonsaktiClient::new_with_db(config.clone(), db).await?
    } else {
        MonsaktiClient::new(config.clone()).await?
    };

    info!("📊 Fetching SIMAN data...");

    // Note: OAuth2 token is handled internally by the client
    // Fetch data from all SIMAN categories using the enum
    use crate::siman::models::SimanAssetCategory;

    let categories = [
        SimanAssetCategory::Tanah,
        SimanAssetCategory::GedungBangunan,
        SimanAssetCategory::JalandanJembatan,
        SimanAssetCategory::InstalasiJaringan,
        SimanAssetCategory::TetapLainnya,
        SimanAssetCategory::KDP,
        SimanAssetCategory::AlatBesar,
        SimanAssetCategory::AngkutanBermotor,
        SimanAssetCategory::AlatPersenjataan,
        SimanAssetCategory::NonTIK,
    ];

    info!("  → Fetching data from {} categories...", categories.len());
    let storage = storage_from_env();

    for category in categories {
        info!("  → Process category: {}", category.description());
        match fetch_all_assets_with_pagination(&mut client, &storage, category).await {
            Ok((success, failed)) => {
                info!(
                    "    ✓ {}: {} success, {} failed",
                    category.description(),
                    success,
                    failed
                );
            }
            Err(e) => {
                warn!("    ⚠ Failed to fetch {:?}: {}", category, e);
            }
        }
    }

    Ok(())
}

/// Save data to database or file based on configuration
#[allow(dead_code)]
async fn save_data(table_name: &str, data: &[serde_json::Value], config: &Config) -> Result<()> {
    let storage_type = std::env::var("STORAGE_TYPE").unwrap_or_else(|_| "json".to_string());

    match storage_type.as_str() {
        "database" => {
            if let Some(ref _db_config) = config.db_config {
                info!(
                    "  💾 Saving {} records to database (table: {})",
                    data.len(),
                    table_name
                );
                // Database storage logic here
                // Use config.db_pool or create connection
                // For now, just log
                info!("  ✓ Database save complete (not yet implemented)");
            } else {
                warn!("  ⚠ Database storage requested but no DB config available");
            }
        }
        // Default output format: JSON.
        _ => {
            let output_dir = &config.output_dir;
            let filename = format!("{}/{}.json", output_dir, table_name);

            // Create directory if it doesn't exist
            tokio::fs::create_dir_all(output_dir).await?;

            // Write to file
            let json_string = serde_json::to_string_pretty(data)?;
            tokio::fs::write(&filename, json_string).await?;

            info!("  💾 Saved {} records to {}", data.len(), filename);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scheduler_creation() {
        let config = Config::default();
        let scheduler = IntegrationScheduler::new(config).await;
        assert!(scheduler.is_ok());
    }
}
