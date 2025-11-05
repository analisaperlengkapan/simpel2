use anyhow::Result;
use authenc::{app::ApplicationBuilder, config::AppConfig};

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration with hierarchy: default.toml → production.toml → env vars
    // This provides better security and auditability for IAM service
    let config = AppConfig::load()?;

    // Initialize logging
    authenc::app::initialize_logging(&config)?;

    tracing::info!("🚀 Starting Authenc Identity and Access Management System (by Cipherce)");
    tracing::info!("📖 Version: {}", env!("CARGO_PKG_VERSION"));

    // Create and run the application
    let app = ApplicationBuilder::new(config);
    app.run().await?;

    Ok(())
}
