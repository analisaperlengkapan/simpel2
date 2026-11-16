//! Scheduled Data Fetcher
//!
//! This binary runs as a daemon service that automatically fetches data from:
//! - MonSAKTI: Daily at 02:00 WIB (configurable via MONSAKTI_SCHEDULE)
//! - MySIMKARI: Daily at 02:00 WIB (configurable via MYSIMKARI_SCHEDULE)
//! - SIMAN: Every Sunday at 03:00 WIB (configurable via SIMAN_SCHEDULE)
//!
//! # Usage
//!
//! ```bash
//! # Run the scheduler (reads .env for configuration)
//! cargo run --bin scheduler
//!
//! # Or with custom environment
//! SCHEDULER_ENABLED=true \
//! MONSAKTI_SCHEDULE="0 2 * * *" \
//! MYSIMKARI_SCHEDULE="0 2 * * *" \
//! SIMAN_SCHEDULE="0 3 * * 0" \
//! cargo run --bin scheduler
//! ```
//!
//! # Cron Expression Format
//!
//! The scheduler uses standard cron expressions with 5 fields:
//! - `minute hour day-of-month month day-of-week`
//!
//! Examples:
//! - `0 2 * * *` - Daily at 02:00
//! - `0 3 * * 0` - Sunday at 03:00
//! - `0 0 */6 * *` - Every 6 hours
//! - `0 0 12 * * MON-FRI` - Weekdays at 12:00

use anyhow::Result;
use clap::Parser;
use layanan_integrasi::{Config, scheduler::IntegrationScheduler};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Source to sync: 'all', 'monsakti', 'mysimkari', 'siman'
    #[arg(short, long)]
    source: Option<String>,

    /// Mode of sync: 'incremental' or 'complete'
    #[arg(short, long)]
    mode: Option<String>,

    /// Kode KL optional for MonSAKTI complete sync
    #[arg(long)]
    kode_kl: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    info!("🚀 SIMPEL - Integration Scheduler Starting...");

    // Load configuration
    let config = Config::from_env()?;

    // Parse CLI args first: whether the daemon flag applies depends on them.
    let cli = Cli::parse();

    // `SCHEDULER_ENABLED` switches the long-running DAEMON (no `--source`) on or off. It must not
    // gate `--source <p>`: that is an explicit request for one run, and it is what every
    // Kubernetes CronJob executes. With the flag off by default (the server must not schedule
    // work on its own), gating `--source` here would make every CronJob log an error and exit 0
    // having synced nothing — the silent success this binary's exit status exists to prevent.
    if cli.source.is_none() && !config.scheduler_enabled {
        error!("❌ Scheduler daemon is disabled (SCHEDULER_ENABLED=false)");
        info!("Set SCHEDULER_ENABLED=true to run the daemon, or pass --source <p> for one run");
        return Ok(());
    }

    // Print configuration
    info!("📋 Scheduler Configuration:");
    info!("  • Timezone: {}", config.scheduler_timezone);
    info!("  • MonSAKTI Schedule: {}", config.monsakti_schedule);
    info!("  • MySIMKARI Schedule: {}", config.mysimkari_schedule);
    info!("  • SIMAN Schedule: {}", config.siman_schedule);
    info!(
        "  • Storage Type: {}",
        std::env::var("STORAGE_TYPE").unwrap_or_else(|_| "json".to_string())
    );
    info!("  • Output Directory: {}", config.output_dir);

    // Create scheduler
    let mut scheduler = IntegrationScheduler::new(config).await?;

    if let Some(source) = cli.source {
        info!("🔧 Manual execution requested for source: {}", source);

        let source_enum = match source.to_lowercase().as_str() {
            "all" => layanan_integrasi::scheduler::DataSource::All,
            "monsakti" => layanan_integrasi::scheduler::DataSource::Monsakti,
            "mysimkari" => layanan_integrasi::scheduler::DataSource::Mysimkari,
            "siman" => layanan_integrasi::scheduler::DataSource::Siman,
            _ => {
                // A typo in a CronJob's `--source` is a broken deployment, not a run to shrug at.
                anyhow::bail!(
                    "Invalid source: {}. Use 'all', 'monsakti', 'mysimkari', or 'siman'.",
                    source
                );
            }
        };

        // The exit status is the ONLY thing a Kubernetes CronJob sees. A failed sync used to be
        // logged and followed by `return Ok(())`, so every run ended `Complete` — with
        // `lastSuccessfulTime` advancing — while not one row had been fetched (found 2026-11-09
        // in the production rehearsal: `ERROR ❌ Manual sync failed` and `Succeeded` together).
        scheduler.run_manual_sync(source_enum).await.map_err(|e| {
            error!("❌ Manual sync failed: {}", e);
            e
        })?;
        info!("✅ Manual sync completed successfully");
        return Ok(());
    }

    scheduler.start().await?;

    // Keep the process running
    info!("⏰ Scheduler is now running. Press Ctrl+C to stop.");

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;

    info!("🛑 Shutdown signal received...");
    scheduler.shutdown().await?;

    info!("👋 Scheduler stopped successfully");
    Ok(())
}
