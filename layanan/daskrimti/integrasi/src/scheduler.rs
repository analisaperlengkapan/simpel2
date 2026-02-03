//! Scheduler module for automated data fetching
//!
//! This module handles scheduled data fetching from external APIs:
//! - MonSAKTI: Scheduled daily at 02:00 WIB (configurable)
//! - MySIMKARI: Scheduled daily at 02:00 WIB (configurable)
//! - SIMAN: Scheduled every Sunday at 03:00 WIB (configurable)

use crate::client::MonsaktiClient;
use crate::config::Config;
use crate::mysimkari;
use crate::siman;
use anyhow::Result;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info, warn};

/// Scheduler for automated data fetching
pub struct IntegrationScheduler {
    config: Config,
    client: MonsaktiClient,
    scheduler: JobScheduler,
}

impl IntegrationScheduler {
    /// Create a new scheduler instance
    pub async fn new(config: Config) -> Result<Self> {
        let client = MonsaktiClient::new(config.clone()).await?;
        let scheduler = JobScheduler::new().await?;

        Ok(Self {
            config,
            client,
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

        let job = Job::new_async(schedule.as_str(), move |_uuid, _lock| {
            let config = config.clone();
            Box::pin(async move {
                info!("🚀 Starting MonSAKTI scheduled data fetch...");
                match fetch_monsakti_data(config).await {
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

        let job = Job::new_async(schedule.as_str(), move |_uuid, _lock| {
            let config = config.clone();
            Box::pin(async move {
                info!("🚀 Starting MySIMKARI scheduled data fetch...");
                match fetch_mysimkari_data(config).await {
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

        let job = Job::new_async(schedule.as_str(), move |_uuid, _lock| {
            let config = config.clone();
            Box::pin(async move {
                info!("🚀 Starting SIMAN scheduled data fetch...");
                match fetch_siman_data(config).await {
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
}

/// Fetch all MonSAKTI data
async fn fetch_monsakti_data(config: Config) -> Result<()> {
    let mut client = MonsaktiClient::new(config.clone()).await?;

    info!("📊 Fetching MonSAKTI data for all modules...");

    // Fetch ADM module data
    if config.tokens.contains_key("ADM") {
        info!("  → Fetching ADM module data...");
        let adm_data = crate::monsakti::adm::ref_admin(&mut client, "006", "").await?;

        // Convert Value to array
        if let Some(array) = adm_data.as_array() {
            info!("  ✓ ADM: {} records", array.len());
            // Save to database or file based on config
            save_data("monsakti_adm", array, &config).await?;
        } else {
            info!("  ✓ ADM: 1 record");
            save_data("monsakti_adm", &[adm_data], &config).await?;
        }
    }

    // Fetch other modules similarly...
    // ANG, AST, BEN, GLP, KOM, PEM, PER

    Ok(())
}

/// Fetch all MySIMKARI data
async fn fetch_mysimkari_data(config: Config) -> Result<()> {
    let mut client = MonsaktiClient::new(config.clone()).await?;

    info!("📊 Fetching MySIMKARI data...");

    // Fetch satker data
    info!("  → Fetching satker data...");
    let satker_response = mysimkari::api::get_satker(&mut client).await?;

    // Handle response - could be array or single object
    if let Some(satker_array) = satker_response.as_array() {
        info!("  ✓ Satker: {} records", satker_array.len());
        save_data("mysimkari_satker", satker_array, &config).await?;

        // Fetch pegawai data for first 5 satker as sample
        info!("  → Fetching pegawai data for sample satker...");
        for satker in satker_array.iter().take(5) {
            if let Some(id) = satker.get("id").and_then(|v| v.as_str()) {
                match mysimkari::api::pegawai_satker(&mut client, id).await {
                    Ok(pegawai_response) => {
                        if let Some(pegawai_array) = pegawai_response.as_array() {
                            info!("    ✓ Pegawai for satker {}: {} records", id, pegawai_array.len());
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
        save_data("mysimkari_satker", &[satker_response], &config).await?;
    }

    Ok(())
}

/// Fetch all SIMAN data
async fn fetch_siman_data(config: Config) -> Result<()> {
    let mut client = MonsaktiClient::new(config.clone()).await?;

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

    for category in categories {
        match siman::endpoints::get_row_count(&mut client, category).await {
            Ok(count) => {
                info!("    ✓ {:?}: {} records", category, count);
            }
            Err(e) => {
                warn!("    ⚠ Failed to fetch {:?}: {}", category, e);
            }
        }
    }

    Ok(())
}

/// Save data to database or file based on configuration
async fn save_data(
    table_name: &str,
    data: &[serde_json::Value],
    config: &Config,
) -> Result<()> {
    let storage_type = std::env::var("STORAGE_TYPE").unwrap_or_else(|_| "json".to_string());

    match storage_type.as_str() {
        "database" => {
            if let Some(ref _db_config) = config.db_config {
                info!("  💾 Saving {} records to database (table: {})", data.len(), table_name);
                // Database storage logic here
                // Use config.db_pool or create connection
                // For now, just log
                info!("  ✓ Database save complete (not yet implemented)");
            } else {
                warn!("  ⚠ Database storage requested but no DB config available");
            }
        }
        "json" | _ => {
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
        let config = Config::from_env().unwrap();
        let scheduler = IntegrationScheduler::new(config).await;
        assert!(scheduler.is_ok());
    }
}
