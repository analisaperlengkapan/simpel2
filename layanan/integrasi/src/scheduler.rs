//! Scheduler module for automated data fetching
//!
//! This module handles scheduled data fetching from external APIs:
//! - MonSAKTI: Scheduled daily at 02:00 WIB (configurable)
//! - MySIMKARI: Scheduled daily at 02:00 WIB (configurable)
//! - SIMAN: Scheduled every Sunday at 03:00 WIB (configurable)

use crate::client::MonsaktiClient;
use crate::config::Config;
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
                self.recorded(source, fetch_monsakti_data(config, self.db_client.clone()))
                    .await?;
            }
            DataSource::Mysimkari => {
                self.recorded(source, fetch_mysimkari_data(config, self.db_client.clone()))
                    .await?;
            }
            DataSource::Siman => {
                self.recorded(source, fetch_siman_data(config, self.db_client.clone()))
                    .await?;
            }
            DataSource::All => {
                info!("Executing full synchronization across all modules...");

                let c1 = config.clone();
                let c2 = config.clone();
                let c3 = config.clone();

                // Run sequentially to avoid overwhelming the database or network. One source
                // failing must not stop the others, but it MUST still fail the run: see
                // `fold_source_results`.
                let results = vec![
                    (
                        "MonSAKTI",
                        self.recorded(
                            DataSource::Monsakti,
                            fetch_monsakti_data(c1, self.db_client.clone()),
                        )
                        .await,
                    ),
                    (
                        "MySIMKARI",
                        self.recorded(
                            DataSource::Mysimkari,
                            fetch_mysimkari_data(c2, self.db_client.clone()),
                        )
                        .await,
                    ),
                    (
                        "SIMAN",
                        self.recorded(
                            DataSource::Siman,
                            fetch_siman_data(c3, self.db_client.clone()),
                        )
                        .await,
                    ),
                ];
                for (name, result) in &results {
                    if let Err(e) = result {
                        error!("❌ {} manual fetch failed: {}", name, e);
                    }
                }
                fold_source_results(results)?;
            }
        }

        info!("✅ Manual data fetch completed for {:?}", source);
        Ok(())
    }

    /// Runs one source's sync and records how it went in `integrasi.sync_status`
    /// (see `crate::sync_run`).
    ///
    /// Recording never changes the run's own result: a sync that succeeded is not failed because
    /// its bookkeeping could not be written, and vice versa. A recording that could not be written
    /// is logged at ERROR, and the row is left as it was, so the next expected run time passes
    /// without a newer start. That is visible to whoever reads the row.
    async fn recorded<F>(&self, source: DataSource, run: F) -> Result<()>
    where
        F: std::future::Future<Output = Result<()>>,
    {
        let module = crate::sync_run::module_of(source);
        let db = self.db_client.as_deref();
        if let (Some(db), Some(module)) = (db, module) {
            let schedule = crate::sync_run::Schedule::from_env();
            match crate::sync_run::mark_started(db, module, schedule.as_ref()).await {
                Ok(0) => error!("sync_status has no row for {module}: this run is not recorded"),
                Ok(_) => {}
                Err(e) => error!("could not record the start of the {module} run: {e}"),
            }
        }

        let result = run.await;

        if let (Some(db), Some(module)) = (db, module) {
            // `{:#}`: the whole chain of the error, on one line.
            let outcome = result.as_ref().map(|_| ()).map_err(|e| format!("{e:#}"));
            if let Err(e) = crate::sync_run::mark_finished(db, module, outcome).await {
                error!("could not record the end of the {module} run: {e}");
            }
        }
        result
    }
}

/// Folds the per-source results of an `All` run into ONE result: the run succeeded only if
/// EVERY source did.
///
/// This used to log each failure and return `Ok(())`. The Kubernetes CronJobs run this as a
/// process, and the process exit status is the ONLY thing the Job controller sees: a run in which
/// every source failed (expired token, API down, wrong base URL) still ended `Complete`, so
/// `lastSuccessfulTime` advanced and nothing anywhere turned red while no data arrived.
fn fold_source_results(results: Vec<(&'static str, Result<()>)>) -> Result<()> {
    let failed: Vec<String> = results
        .into_iter()
        .filter_map(|(name, r)| r.err().map(|e| format!("{name}: {e}")))
        .collect();
    if failed.is_empty() {
        Ok(())
    } else {
        anyhow::bail!("{} sumber gagal — {}", failed.len(), failed.join("; "))
    }
}

/// Ends a loop that keeps going after each failure.
///
/// The loops below carry on so that one bad item does not hide the others, but the RUN must still
/// fail if any item did: logging `Failed to fetch …` and returning `Ok(())` is how the rehearsal's
/// `siman` Job ended `Complete` with 10 of its 10 categories failed.
fn fail_if_any(what: &str, failures: Vec<String>) -> Result<()> {
    if failures.is_empty() {
        Ok(())
    } else {
        anyhow::bail!(
            "{} {} gagal — {}",
            failures.len(),
            what,
            failures.join("; ")
        )
    }
}

/// How a SIMAN category went wrong, or `None` when it was taken in whole.
///
/// `fetch_all_assets_with_pagination` reports two different kinds of trouble. `Err` means the
/// category could not be asked for at all (expired client secret, API down, wrong URL).
/// `Ok((_, failed))` with `failed > 0` means some 1000-row chunks were not fetched or not stored.
/// Both leave the local copy of that category incomplete, so neither may end in a green Job.
fn siman_category_failure<E: std::fmt::Display>(
    category: &str,
    outcome: &std::result::Result<(usize, usize), E>,
) -> Option<String> {
    match outcome {
        Ok((_, 0)) => None,
        Ok((ok, failed)) => Some(format!(
            "{category}: {failed} potongan data gagal ({ok} berhasil)"
        )),
        Err(e) => Some(format!("{category}: {e}")),
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

/// Fetch all MySIMKARI data: every satker and every satker's employees, into
/// `integrasi.mysimkari_satker` / `integrasi.mysimkari_pegawai`.
///
/// This is what `./scheduler --source mysimkari` (the CronJob) runs. It used to save into
/// `mysimkari_satker_satker` and `mysimkari_pegawai_pegawai`, tables `db::save_to_database`
/// derived from a module/endpoint pair and created on the fly that nothing reads, and it fetched
/// the employees of the first five satker only ("as sample"). The complete implementation,
/// `batch::fetchers::fetch_mysimkari`, was reachable only from a CLI the image does not ship.
/// There is one implementation now (see `tests/sync_lands_in_source_tables.rs`).
async fn fetch_mysimkari_data(config: Config, db_client: Option<Arc<Client>>) -> Result<()> {
    let mut client = if let Some(db) = db_client {
        MonsaktiClient::new_with_db(config.clone(), db).await?
    } else {
        MonsaktiClient::new(config.clone()).await?
    };

    info!("📊 Fetching MySIMKARI data...");
    crate::batch::fetchers::fetch_mysimkari(&mut client, &storage_from_env()).await?;
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

    // Every category is attempted even after one fails, so a single bad category does not hide
    // the rest — but the run as a whole fails if any category did (see `fail_if_any`).
    let mut failures: Vec<String> = Vec::new();
    for category in categories {
        info!("  → Process category: {}", category.description());
        let outcome = fetch_all_assets_with_pagination(&mut client, &storage, category).await;
        match &outcome {
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
        failures.extend(siman_category_failure(category.description(), &outcome));
    }

    fail_if_any("kategori SIMAN", failures)
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

    #[test]
    fn a_run_succeeds_only_when_every_source_did() {
        let ok = vec![
            ("MonSAKTI", Ok(())),
            ("MySIMKARI", Ok(())),
            ("SIMAN", Ok(())),
        ];
        assert!(fold_source_results(ok).is_ok());
    }

    #[test]
    fn one_failed_source_fails_the_run_and_names_it() {
        let results = vec![
            ("MonSAKTI", Ok(())),
            ("MySIMKARI", Err(anyhow::anyhow!("token kedaluwarsa"))),
            ("SIMAN", Ok(())),
        ];
        let err = fold_source_results(results).unwrap_err().to_string();
        assert!(err.contains("1 sumber gagal"), "{err}");
        assert!(err.contains("MySIMKARI: token kedaluwarsa"), "{err}");
        assert!(
            !err.contains("MonSAKTI"),
            "a source that worked must not be blamed: {err}"
        );
    }

    #[test]
    fn every_failed_source_is_reported_not_just_the_first() {
        let results = vec![
            ("MonSAKTI", Err(anyhow::anyhow!("a"))),
            ("MySIMKARI", Err(anyhow::anyhow!("b"))),
            ("SIMAN", Err(anyhow::anyhow!("c"))),
        ];
        let err = fold_source_results(results).unwrap_err().to_string();
        assert!(err.contains("3 sumber gagal"), "{err}");
        for n in ["MonSAKTI: a", "MySIMKARI: b", "SIMAN: c"] {
            assert!(err.contains(n), "{err}");
        }
    }

    #[test]
    fn a_category_taken_in_whole_is_not_a_failure() {
        let ok: std::result::Result<(usize, usize), String> = Ok((12_000, 0));
        assert_eq!(siman_category_failure("Tanah", &ok), None);
        // Nothing to fetch is not a failure either: the API answered, the category is empty.
        let empty: std::result::Result<(usize, usize), String> = Ok((0, 0));
        assert_eq!(siman_category_failure("KDP", &empty), None);
    }

    #[test]
    fn a_category_that_could_not_be_asked_for_is_a_failure_naming_why() {
        let err: std::result::Result<(usize, usize), String> =
            Err("Request HTTP gagal: token".to_string());
        let msg = siman_category_failure("Tanah", &err).expect("an Err is a failure");
        assert!(
            msg.contains("Tanah") && msg.contains("Request HTTP gagal"),
            "{msg}"
        );
    }

    #[test]
    fn a_category_with_failed_chunks_is_a_failure_even_though_the_call_returned_ok() {
        // `Ok((3, 2))`: two of five 1000-row chunks never arrived. The local copy is incomplete.
        let partial: std::result::Result<(usize, usize), String> = Ok((3, 2));
        let msg = siman_category_failure("Gedung dan Bangunan", &partial)
            .expect("failed chunks are a failure");
        assert!(msg.contains("Gedung dan Bangunan"), "{msg}");
        assert!(msg.contains("2 potongan data gagal"), "{msg}");
    }

    #[test]
    fn a_loop_that_carried_on_still_fails_the_run_and_names_every_item() {
        assert!(fail_if_any("kategori SIMAN", vec![]).is_ok());
        let err = fail_if_any("kategori SIMAN", vec!["Tanah: a".into(), "KDP: b".into()])
            .unwrap_err()
            .to_string();
        assert!(err.contains("2 kategori SIMAN gagal"), "{err}");
        assert!(err.contains("Tanah: a") && err.contains("KDP: b"), "{err}");
    }
}
