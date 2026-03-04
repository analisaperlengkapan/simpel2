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

    if !config.scheduler_enabled {
        error!("❌ Scheduler is disabled (SCHEDULER_ENABLED=false)");
        info!("Set SCHEDULER_ENABLED=true in .env to enable scheduling");
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

    // Parse CLI args
    let cli = Cli::parse();

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
                error!(
                    "Invalid source: {}. Use 'all', 'monsakti', 'mysimkari', or 'siman'.",
                    source
                );
                return Ok(());
            }
        };

        if let Err(e) = scheduler.run_manual_sync(source_enum).await {
            error!("❌ Manual sync failed: {}", e);
        } else {
            info!("✅ Manual sync completed successfully");
        }
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
