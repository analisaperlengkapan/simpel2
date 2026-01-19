//! Seal/Unseal CLI commands
//!
//! Provides CLI commands for vault seal operations including:
//! - init: Initialize vault and generate Shamir shares
//! - seal: Seal the vault
//! - unseal: Unseal the vault with shares
//! - status: Show seal status
//! - rekey: Rekey operation (change threshold/shares)

use anyhow::{Context, Result};
use clap::Subcommand;
use comfy_table::{Cell, Color, ContentArrangement, Table, presets::UTF8_FULL};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};

use crate::config::CliConfig;

#[derive(Subcommand)]
/// Mewakili pub `SealCommand`.
pub enum SealCommand {
    /// Initialize vault and generate Shamir shares
    Init {
        /// Number of key shares to generate (default: 5)
        #[arg(short = 'n', long, default_value = "5")]
        shares: usize,

        /// Number of shares required to unseal (default: 3)
        #[arg(short = 't', long, default_value = "3")]
        threshold: usize,

        /// Output shares to file instead of stdout
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Seal the vault
    Seal,

    /// Unseal the vault with a key share
    Unseal {
        /// Key share (base64 encoded). If not provided, will prompt securely
        #[arg(short, long)]
        key: Option<String>,

        /// Reset unseal progress
        #[arg(short, long)]
        reset: bool,
    },

    /// Show seal status
    Status,

    /// Rekey operation (change threshold/shares)
    Rekey {
        #[command(subcommand)]
        cmd: RekeyCommand,
    },
}

#[derive(Subcommand)]
/// Mewakili pub `RekeyCommand`.
pub enum RekeyCommand {
    /// Start rekey operation
    Init {
        /// New number of key shares
        #[arg(short = 'n', long)]
        shares: usize,

        /// New number of shares required to unseal
        #[arg(short = 't', long)]
        threshold: usize,
    },

    /// Provide a key share for rekey
    Update {
        /// Key share (base64 encoded). If not provided, will prompt securely
        #[arg(short, long)]
        key: Option<String>,
    },

    /// Cancel rekey operation
    Cancel,

    /// Show rekey progress
    Status,
}

/// Seal status response from API
#[derive(Debug, Deserialize, Serialize)]
struct SealStatus {
    state: String,
    seal_type: String,
    initialized: bool,
    total_shares: usize,
    threshold: usize,
    progress: usize,
    version: String,
}

/// Init response from API
#[derive(Debug, Deserialize, Serialize)]
struct InitResponse {
    keys: Vec<String>,
    keys_base64: Vec<String>,
    root_token: Option<String>,
}

/// Rekey operation response
#[derive(Debug, Deserialize, Serialize)]
struct RekeyOperation {
    new_shares: usize,
    new_threshold: usize,
    progress: usize,
    required: usize,
    nonce: String,
}

/// Execute seal command
pub async fn execute_seal_command(cmd: SealCommand, config: &CliConfig) -> Result<()> {
    match cmd {
        SealCommand::Init {
            shares,
            threshold,
            output,
        } => init_vault(config, shares, threshold, output).await,
        SealCommand::Seal => seal_vault(config).await,
        SealCommand::Unseal { key, reset } => unseal_vault(config, key, reset).await,
        SealCommand::Status => seal_status(config).await,
        SealCommand::Rekey { cmd } => execute_rekey_command(cmd, config).await,
    }
}

/// Initialize vault and generate Shamir shares
async fn init_vault(
    config: &CliConfig,
    shares: usize,
    threshold: usize,
    output: Option<String>,
) -> Result<()> {
    // Validate parameters
    if threshold > shares {
        anyhow::bail!(
            "Threshold ({}) cannot be greater than shares ({})",
            threshold,
            shares
        );
    }
    if threshold < 1 {
        anyhow::bail!("Threshold must be at least 1");
    }
    if shares < 1 {
        anyhow::bail!("Shares must be at least 1");
    }

    let client = reqwest::Client::new();
    let url = format!("{}/v1/sys/init", config.server_url);

    let payload = serde_json::json!({
        "secret_shares": shares,
        "secret_threshold": threshold,
    });

    println!("🔐 Initializing Secreton Vault...");
    println!("   Shares: {}", shares);
    println!("   Threshold: {}", threshold);
    println!();

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .context("Failed to connect to Secreton server")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        anyhow::bail!("Failed to initialize vault: {} - {}", status, error_text);
    }

    let init_response: InitResponse = response
        .json()
        .await
        .context("Failed to parse initialization response")?;

    // Display shares
    println!("✅ Vault initialized successfully!");
    println!();
    println!("⚠️  IMPORTANT: Save these unseal keys securely!");
    println!(
        "   You will need {} of {} keys to unseal the vault.",
        threshold, shares
    );
    println!();

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Key #").fg(Color::Cyan),
            Cell::new("Unseal Key (Base64)").fg(Color::Cyan),
        ]);

    for (i, key) in init_response.keys_base64.iter().enumerate() {
        table.add_row(vec![
            Cell::new(format!("{}", i + 1)),
            Cell::new(key).fg(Color::Yellow),
        ]);
    }

    println!("{}", table);
    println!();

    if let Some(root_token) = &init_response.root_token {
        println!("🔑 Root Token: {}", root_token);
        println!();
    }

    // Save to file if requested
    if let Some(output_path) = output {
        let output_data = serde_json::json!({
            "shares": shares,
            "threshold": threshold,
            "keys": init_response.keys_base64,
            "root_token": init_response.root_token,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        std::fs::write(&output_path, serde_json::to_string_pretty(&output_data)?)
            .context("Failed to write keys to file")?;

        println!("💾 Keys saved to: {}", output_path);
        println!();
    }

    println!("⚠️  WARNING:");
    println!("   • Store these keys in separate, secure locations");
    println!("   • Never store all keys together");
    println!("   • These keys cannot be recovered if lost");
    println!("   • The vault is now unsealed and ready to use");

    Ok(())
}

/// Seal the vault
async fn seal_vault(config: &CliConfig) -> Result<()> {
    let client = reqwest::Client::new();
    let url = format!("{}/v1/sys/seal", config.server_url);

    println!("🔒 Sealing vault...");

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .send()
        .await
        .context("Failed to connect to Secreton server")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        anyhow::bail!("Failed to seal vault: {} - {}", status, error_text);
    }

    println!("✅ Vault sealed successfully!");
    println!();
    println!("   All operations are now blocked until the vault is unsealed.");
    println!("   Use 'secreton unseal' to unseal the vault.");

    Ok(())
}

/// Unseal the vault with a key share
async fn unseal_vault(config: &CliConfig, key: Option<String>, reset: bool) -> Result<()> {
    let client = reqwest::Client::new();

    // Handle reset
    if reset {
        let url = format!("{}/v1/sys/unseal", config.server_url);
        let payload = serde_json::json!({ "reset": true });

        let response = client
            .put(&url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .context("Failed to connect to Secreton server")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to reset unseal: {} - {}", status, error_text);
        }

        println!("✅ Unseal progress reset");
        return Ok(());
    }

    // Get key from argument or prompt securely
    let unseal_key = if let Some(k) = key {
        k
    } else {
        // Prompt for key without echoing
        print!("Enter unseal key: ");
        io::stdout().flush()?;

        let key = rpassword::read_password().context("Failed to read unseal key")?;

        if key.trim().is_empty() {
            anyhow::bail!("Unseal key cannot be empty");
        }

        key.trim().to_string()
    };

    let url = format!("{}/v1/sys/unseal", config.server_url);
    let payload = serde_json::json!({
        "key": unseal_key,
    });

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .context("Failed to connect to Secreton server")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        anyhow::bail!("Failed to unseal vault: {} - {}", status, error_text);
    }

    let status: SealStatus = response
        .json()
        .await
        .context("Failed to parse unseal response")?;

    // Display progress
    if status.state == "unsealed" {
        println!("✅ Vault unsealed successfully!");
        println!();
        println!("   The vault is now operational.");
    } else {
        println!(
            "🔓 Unseal progress: {}/{}",
            status.progress, status.threshold
        );
        println!();
        println!(
            "   {} more key(s) required to unseal the vault.",
            status.threshold - status.progress
        );
    }

    Ok(())
}

/// Show seal status
async fn seal_status(config: &CliConfig) -> Result<()> {
    let client = reqwest::Client::new();
    let url = format!("{}/v1/sys/seal-status", config.server_url);

    let response = client
        .get(&url)
        .send()
        .await
        .context("Failed to connect to Secreton server")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        anyhow::bail!("Failed to get seal status: {} - {}", status, error_text);
    }

    let status: SealStatus = response
        .json()
        .await
        .context("Failed to parse seal status response")?;

    // Display status with colors
    let state_icon = match status.state.as_str() {
        "unsealed" => "🟢",
        "unsealing" => "🟡",
        "sealed" => "🔴",
        _ => "⚪",
    };

    let state_text = match status.state.as_str() {
        "unsealed" => "UNSEALED",
        "unsealing" => "UNSEALING",
        "sealed" => "SEALED",
        _ => "UNKNOWN",
    };

    println!("{} Vault Status: {}", state_icon, state_text);
    println!();

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);

    table.add_row(vec![
        "Initialized",
        if status.initialized { "Yes" } else { "No" },
    ]);
    table.add_row(vec!["Seal Type", &status.seal_type]);
    table.add_row(vec!["Total Shares", &status.total_shares.to_string()]);
    table.add_row(vec!["Threshold", &status.threshold.to_string()]);

    if status.state == "unsealing" {
        table.add_row(vec![
            "Progress",
            &format!("{}/{}", status.progress, status.threshold),
        ]);
        table.add_row(vec![
            "Remaining",
            &(status.threshold - status.progress).to_string(),
        ]);
    }

    table.add_row(vec!["Version", &status.version]);

    println!("{}", table);

    Ok(())
}

/// Execute rekey command
async fn execute_rekey_command(cmd: RekeyCommand, config: &CliConfig) -> Result<()> {
    match cmd {
        RekeyCommand::Init { shares, threshold } => rekey_init(config, shares, threshold).await,
        RekeyCommand::Update { key } => rekey_update(config, key).await,
        RekeyCommand::Cancel => rekey_cancel(config).await,
        RekeyCommand::Status => rekey_status(config).await,
    }
}

/// Start rekey operation
async fn rekey_init(config: &CliConfig, shares: usize, threshold: usize) -> Result<()> {
    // Validate parameters
    if threshold > shares {
        anyhow::bail!(
            "Threshold ({}) cannot be greater than shares ({})",
            threshold,
            shares
        );
    }
    if threshold < 1 {
        anyhow::bail!("Threshold must be at least 1");
    }
    if shares < 1 {
        anyhow::bail!("Shares must be at least 1");
    }

    let client = reqwest::Client::new();
    let url = format!("{}/v1/sys/rekey/init", config.server_url);

    let payload = serde_json::json!({
        "secret_shares": shares,
        "secret_threshold": threshold,
    });

    println!("🔄 Starting rekey operation...");
    println!("   New Shares: {}", shares);
    println!("   New Threshold: {}", threshold);
    println!();

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .context("Failed to connect to Secreton server")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        anyhow::bail!("Failed to start rekey: {} - {}", status, error_text);
    }

    let rekey: RekeyOperation = response
        .json()
        .await
        .context("Failed to parse rekey response")?;

    println!("✅ Rekey operation started");
    println!();
    println!("   Nonce: {}", rekey.nonce);
    println!("   Required keys: {}", rekey.required);
    println!();
    println!("Use 'secreton rekey update' to provide unseal keys.");

    Ok(())
}

/// Provide a key share for rekey
async fn rekey_update(config: &CliConfig, key: Option<String>) -> Result<()> {
    // Get key from argument or prompt securely
    let unseal_key = if let Some(k) = key {
        k
    } else {
        print!("Enter unseal key: ");
        io::stdout().flush()?;

        let key = rpassword::read_password().context("Failed to read unseal key")?;

        if key.trim().is_empty() {
            anyhow::bail!("Unseal key cannot be empty");
        }

        key.trim().to_string()
    };

    let client = reqwest::Client::new();
    let url = format!("{}/v1/sys/rekey/update", config.server_url);

    let payload = serde_json::json!({
        "key": unseal_key,
    });

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .context("Failed to connect to Secreton server")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        anyhow::bail!("Failed to update rekey: {} - {}", status, error_text);
    }

    let rekey: RekeyOperation = response
        .json()
        .await
        .context("Failed to parse rekey response")?;

    if rekey.progress >= rekey.required {
        println!("✅ Rekey operation completed!");
        println!();
        println!("   New configuration:");
        println!("   • Shares: {}", rekey.new_shares);
        println!("   • Threshold: {}", rekey.new_threshold);
        println!();
        println!("   New unseal keys have been generated.");
    } else {
        println!("🔄 Rekey progress: {}/{}", rekey.progress, rekey.required);
        println!();
        println!(
            "   {} more key(s) required.",
            rekey.required - rekey.progress
        );
    }

    Ok(())
}

/// Cancel rekey operation
async fn rekey_cancel(config: &CliConfig) -> Result<()> {
    let client = reqwest::Client::new();
    let url = format!("{}/v1/sys/rekey/init", config.server_url);

    let response = client
        .delete(&url)
        .send()
        .await
        .context("Failed to connect to Secreton server")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        anyhow::bail!("Failed to cancel rekey: {} - {}", status, error_text);
    }

    println!("✅ Rekey operation cancelled");

    Ok(())
}

/// Show rekey progress
async fn rekey_status(config: &CliConfig) -> Result<()> {
    let client = reqwest::Client::new();
    let url = format!("{}/v1/sys/rekey/init", config.server_url);

    let response = client
        .get(&url)
        .send()
        .await
        .context("Failed to connect to Secreton server")?;

    if response.status() == 404 {
        println!("ℹ️  No rekey operation in progress");
        return Ok(());
    }

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        anyhow::bail!("Failed to get rekey status: {} - {}", status, error_text);
    }

    let rekey: RekeyOperation = response
        .json()
        .await
        .context("Failed to parse rekey response")?;

    println!("🔄 Rekey Operation Status");
    println!();

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);

    table.add_row(vec!["Nonce", &rekey.nonce]);
    table.add_row(vec!["New Shares", &rekey.new_shares.to_string()]);
    table.add_row(vec!["New Threshold", &rekey.new_threshold.to_string()]);
    table.add_row(vec![
        "Progress",
        &format!("{}/{}", rekey.progress, rekey.required),
    ]);
    table.add_row(vec![
        "Remaining",
        &(rekey.required - rekey.progress).to_string(),
    ]);

    println!("{}", table);

    Ok(())
}
