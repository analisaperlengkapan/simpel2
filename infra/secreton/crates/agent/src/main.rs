//! Secreton Agent binary

use anyhow::Result;
use clap::Parser;
use secreton_agent::{AgentConfig, SecretonAgent};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[derive(Parser)]
#[command(name = "secreton-agent")]
#[command(about = "Secreton Vault Agent - auto-auth, token renewal, template rendering")]
struct Cli {
    /// Configuration file path
    #[arg(short, long, default_value = "agent.yaml")]
    config: String,

    /// Log level
    #[arg(short, long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    lib_common::telemetry::init_subscriber(&cli.log_level);

    // Load configuration
    let config = AgentConfig::load_from_file(&cli.config)?;

    // Create and start agent
    let mut agent = SecretonAgent::new(config)?;
    agent.start().await?;

    Ok(())
}
