//! Secreton Kubernetes Operator
//!
//! This operator watches for SecretSync custom resources and synchronizes
//! secrets from Secreton to Kubernetes Secret objects.

use anyhow::Result;
use clap::Parser;
use kube::Client;
use secreton_k8s_operator::controller::{Context, run};
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Secreton server URL
    #[arg(long, env, default_value = "https://secreton.internal:8200")]
    secreton_url: String,

    /// Default authentication token (optional)
    #[arg(long, env)]
    token: Option<String>,

    /// Log level
    #[arg(long, env, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize tracing
    let level = match args.log_level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    let subscriber = FmtSubscriber::builder().with_max_level(level).finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting Secreton Kubernetes Operator");
    info!("Secreton URL: {}", args.secreton_url);

    // Create Kubernetes client
    let client = Client::try_default().await?;

    // Create context
    let context = Context {
        client,
        secreton_url: args.secreton_url,
        default_token: args.token,
    };

    // Run the controller
    run(context).await?;

    Ok(())
}
