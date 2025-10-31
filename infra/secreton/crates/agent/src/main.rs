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
    let log_level = match cli.log_level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_target(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    // Load configuration
    let config = AgentConfig::load_from_file(&cli.config)?;

    // Create and start agent
    let mut agent = SecretonAgent::new(config)?;
    agent.start().await?;

    Ok(())
}
