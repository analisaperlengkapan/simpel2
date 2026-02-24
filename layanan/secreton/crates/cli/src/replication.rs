//! Replication management commands
//!
//! This module provides CLI commands for managing Secreton replication:
//! - Enable/disable replication
//! - View replication status and lag metrics
//! - Promote secondary to primary (manual failover)
//!
//! **Validates: Requirements 2.2.9**

use anyhow::Result;
use clap::Subcommand;
use serde::{Deserialize, Serialize};

use crate::config::CliConfig;
use crate::http_client::AuthenticatedClient;
use crate::middleware::SealChecker;

#[derive(Subcommand)]
pub enum ReplicationCommand {
    /// Enable replication on this node
    Enable {
        /// Replication mode (performance or dr)
        #[arg(short, long, default_value = "performance")]
        mode: String,

        /// Primary node endpoint (for secondary nodes)
        #[arg(short, long)]
        primary: Option<String>,

        /// Secondary node endpoints (for primary nodes)
        #[arg(short, long)]
        secondaries: Vec<String>,
    },

    /// Disable replication on this node
    Disable {
        /// Force disable even if replication is active
        #[arg(short, long)]
        force: bool,
    },

    /// Show replication status and metrics
    Status {
        /// Output format (table, json, yaml)
        #[arg(short, long, default_value = "table")]
        format: String,
    },

    /// Promote secondary to primary (manual failover)
    Promote {
        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// Add a secondary node to replication
    AddSecondary {
        /// Secondary node endpoint
        endpoint: String,
    },

    /// Remove a secondary node from replication
    RemoveSecondary {
        /// Secondary node ID
        node_id: String,
    },

    /// Show replication lag metrics
    Lag {
        /// Output format (table, json, yaml)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct ReplicationStatus {
    enabled: bool,
    mode: String,
    is_primary: bool,
    primary_endpoint: Option<String>,
    secondaries: Vec<SecondaryNodeInfo>,
    lag_ms: u64,
    lag_bytes: u64,
    primary_sequence: u64,
    secondary_sequence: u64,
    health: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SecondaryNodeInfo {
    node_id: String,
    endpoint: String,
    status: String,
    lag_ms: u64,
    lag_bytes: u64,
    last_heartbeat: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PromoteResponse {
    success: bool,
    message: String,
    new_role: String,
}

pub async fn execute_replication_command(
    cmd: ReplicationCommand,
    config: &CliConfig,
) -> Result<()> {
    // Check engine seal status before operations
    let mut seal_checker = SealChecker::new(config.server_url.clone(), None);
    if let Err(e) = seal_checker.require_unsealed().await {
        eprintln!("❌ {}", e);
        return Ok(());
    }

    // Create authenticated client
    let auth_client = AuthenticatedClient::new()?;

    match cmd {
        ReplicationCommand::Enable {
            mode,
            primary,
            secondaries,
        } => enable_replication(&auth_client, config, &mode, primary, secondaries).await,

        ReplicationCommand::Disable { force } => {
            disable_replication(&auth_client, config, force).await
        }

        ReplicationCommand::Status { format } => {
            show_replication_status(&auth_client, config, &format).await
        }

        ReplicationCommand::Promote { yes } => promote_to_primary(&auth_client, config, yes).await,

        ReplicationCommand::AddSecondary { endpoint } => {
            add_secondary(&auth_client, config, &endpoint).await
        }

        ReplicationCommand::RemoveSecondary { node_id } => {
            remove_secondary(&auth_client, config, &node_id).await
        }

        ReplicationCommand::Lag { format } => {
            show_replication_lag(&auth_client, config, &format).await
        }
    }
}

async fn enable_replication(
    client: &AuthenticatedClient,
    config: &CliConfig,
    mode: &str,
    primary: Option<String>,
    secondaries: Vec<String>,
) -> Result<()> {
    let url = format!("{}/v1/sys/replication/enable", config.server_url);

    let payload = serde_json::json!({
        "mode": mode,
        "primary_endpoint": primary,
        "secondary_endpoints": secondaries,
    });

    let response = client
        .execute(
            client
                .post(&url)?
                .header("Content-Type", "application/json")
                .json(&payload),
        )
        .await?;

    if response.status().is_success() {
        println!("✅ Replication enabled successfully");
        println!("   Mode: {}", mode);
        if let Some(primary_endpoint) = primary {
            println!("   Primary: {}", primary_endpoint);
        }
        if !secondaries.is_empty() {
            println!("   Secondaries:");
            for secondary in secondaries {
                println!("     • {}", secondary);
            }
        }
    } else {
        let error_text = response.text().await?;
        eprintln!("❌ Failed to enable replication: {}", error_text);
    }

    Ok(())
}

async fn disable_replication(
    client: &AuthenticatedClient,
    config: &CliConfig,
    force: bool,
) -> Result<()> {
    let url = format!("{}/v1/sys/replication/disable", config.server_url);

    let payload = serde_json::json!({
        "force": force,
    });

    let response = client
        .execute(
            client
                .post(&url)?
                .header("Content-Type", "application/json")
                .json(&payload),
        )
        .await?;

    if response.status().is_success() {
        println!("✅ Replication disabled successfully");
    } else {
        let error_text = response.text().await?;
        eprintln!("❌ Failed to disable replication: {}", error_text);
    }

    Ok(())
}

async fn show_replication_status(
    client: &AuthenticatedClient,
    config: &CliConfig,
    format: &str,
) -> Result<()> {
    let url = format!("{}/v1/sys/replication/status", config.server_url);

    let response = client.execute(client.get(&url)?).await?;

    if response.status().is_success() {
        let status: ReplicationStatus = response.json().await?;

        match format {
            "json" => {
                println!("{}", serde_json::to_string_pretty(&status)?);
            }
            "yaml" => {
                println!("{}", serde_yaml::to_string(&status)?);
            }
            "table" | _ => {
                print_replication_status_table(&status);
            }
        }
    } else {
        let error_text = response.text().await?;
        eprintln!("❌ Failed to get replication status: {}", error_text);
    }

    Ok(())
}

fn print_replication_status_table(status: &ReplicationStatus) {
    println!("📊 Replication Status");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!(
        "  Enabled:           {}",
        if status.enabled { "✅ Yes" } else { "❌ No" }
    );
    println!("  Mode:              {}", status.mode);
    println!(
        "  Role:              {}",
        if status.is_primary {
            "🔵 Primary"
        } else {
            "🟢 Secondary"
        }
    );
    println!("  Health:            {}", format_health(&status.health));

    if let Some(primary) = &status.primary_endpoint {
        println!("  Primary Endpoint:  {}", primary);
    }

    println!();
    println!("📈 Replication Metrics");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  Lag (time):        {} ms", status.lag_ms);
    println!("  Lag (bytes):       {} bytes", status.lag_bytes);
    println!("  Primary Sequence:  {}", status.primary_sequence);
    println!("  Secondary Seq:     {}", status.secondary_sequence);

    if !status.secondaries.is_empty() {
        println!();
        println!("🔗 Secondary Nodes");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        for secondary in &status.secondaries {
            println!("  Node ID:           {}", secondary.node_id);
            println!("  Endpoint:          {}", secondary.endpoint);
            println!("  Status:            {}", format_status(&secondary.status));
            println!("  Lag (time):        {} ms", secondary.lag_ms);
            println!("  Lag (bytes):       {} bytes", secondary.lag_bytes);
            println!("  Last Heartbeat:    {}", secondary.last_heartbeat);
            println!("  ────────────────────────────────────────────────────");
        }
    }
}

fn format_health(health: &str) -> String {
    match health {
        "healthy" => "✅ Healthy".to_string(),
        "degraded" => "⚠️  Degraded".to_string(),
        "unhealthy" => "❌ Unhealthy".to_string(),
        _ => health.to_string(),
    }
}

fn format_status(status: &str) -> String {
    match status {
        "active" => "✅ Active".to_string(),
        "syncing" => "🔄 Syncing".to_string(),
        "lagging" => "⚠️  Lagging".to_string(),
        "disconnected" => "❌ Disconnected".to_string(),
        _ => status.to_string(),
    }
}

async fn promote_to_primary(
    client: &AuthenticatedClient,
    config: &CliConfig,
    skip_confirmation: bool,
) -> Result<()> {
    if !skip_confirmation {
        println!("⚠️  WARNING: Promoting secondary to primary will:");
        println!("   • Stop replication from the current primary");
        println!("   • Enable write operations on this node");
        println!("   • Update cluster configuration");
        println!();
        print!("Are you sure you want to continue? (yes/no): ");
        use std::io::{self, Write};
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() != "yes" {
            println!("❌ Promotion cancelled");
            return Ok(());
        }
    }

    let url = format!("{}/v1/sys/replication/promote", config.server_url);

    let response = client
        .execute(
            client
                .post(&url)?
                .header("Content-Type", "application/json"),
        )
        .await?;

    if response.status().is_success() {
        let promote_response: PromoteResponse = response.json().await?;
        println!("✅ {}", promote_response.message);
        println!("   New Role: {}", promote_response.new_role);
    } else {
        let error_text = response.text().await?;
        eprintln!("❌ Failed to promote to primary: {}", error_text);
    }

    Ok(())
}

async fn add_secondary(
    client: &AuthenticatedClient,
    config: &CliConfig,
    endpoint: &str,
) -> Result<()> {
    let url = format!("{}/v1/sys/replication/secondary", config.server_url);

    let payload = serde_json::json!({
        "endpoint": endpoint,
    });

    let response = client
        .execute(
            client
                .post(&url)?
                .header("Content-Type", "application/json")
                .json(&payload),
        )
        .await?;

    if response.status().is_success() {
        let result: serde_json::Value = response.json().await?;
        if let Some(node_id) = result.get("node_id").and_then(|n| n.as_str()) {
            println!("✅ Secondary node added successfully");
            println!("   Node ID: {}", node_id);
            println!("   Endpoint: {}", endpoint);
        }
    } else {
        let error_text = response.text().await?;
        eprintln!("❌ Failed to add secondary: {}", error_text);
    }

    Ok(())
}

async fn remove_secondary(
    client: &AuthenticatedClient,
    config: &CliConfig,
    node_id: &str,
) -> Result<()> {
    let url = format!(
        "{}/v1/sys/replication/secondary/{}",
        config.server_url, node_id
    );

    let response = client.execute(client.delete(&url)?).await?;

    if response.status().is_success() {
        println!("✅ Secondary node removed successfully");
        println!("   Node ID: {}", node_id);
    } else {
        let error_text = response.text().await?;
        eprintln!("❌ Failed to remove secondary: {}", error_text);
    }

    Ok(())
}

async fn show_replication_lag(
    client: &AuthenticatedClient,
    config: &CliConfig,
    format: &str,
) -> Result<()> {
    let url = format!("{}/v1/sys/replication/lag", config.server_url);

    let response = client.execute(client.get(&url)?).await?;

    if response.status().is_success() {
        let lag: serde_json::Value = response.json().await?;

        match format {
            "json" => {
                println!("{}", serde_json::to_string_pretty(&lag)?);
            }
            "yaml" => {
                println!("{}", serde_yaml::to_string(&lag)?);
            }
            "table" | _ => {
                print_lag_table(&lag);
            }
        }
    } else {
        let error_text = response.text().await?;
        eprintln!("❌ Failed to get replication lag: {}", error_text);
    }

    Ok(())
}

fn print_lag_table(lag: &serde_json::Value) {
    println!("📊 Replication Lag Metrics");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    if let Some(lag_ms) = lag.get("lag_ms").and_then(|l| l.as_u64()) {
        println!("  Time Lag:          {} ms", lag_ms);

        // Color code based on lag severity
        if lag_ms < 100 {
            println!("  Status:            ✅ Excellent (< 100ms)");
        } else if lag_ms < 1000 {
            println!("  Status:            ⚠️  Warning (100-1000ms)");
        } else {
            println!("  Status:            ❌ Critical (> 1000ms)");
        }
    }

    if let Some(lag_bytes) = lag.get("lag_bytes").and_then(|l| l.as_u64()) {
        println!("  Byte Lag:          {} bytes", lag_bytes);
        println!("                     ({:.2} KB)", lag_bytes as f64 / 1024.0);
    }

    if let Some(primary_seq) = lag.get("primary_sequence").and_then(|s| s.as_u64()) {
        println!("  Primary Sequence:  {}", primary_seq);
    }

    if let Some(secondary_seq) = lag.get("secondary_sequence").and_then(|s| s.as_u64()) {
        println!("  Secondary Seq:     {}", secondary_seq);
    }

    if let Some(operations_behind) = lag.get("operations_behind").and_then(|o| o.as_u64()) {
        println!("  Operations Behind: {}", operations_behind);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_health() {
        assert_eq!(format_health("healthy"), "✅ Healthy");
        assert_eq!(format_health("degraded"), "⚠️  Degraded");
        assert_eq!(format_health("unhealthy"), "❌ Unhealthy");
        assert_eq!(format_health("unknown"), "unknown");
    }

    #[test]
    fn test_format_status() {
        assert_eq!(format_status("active"), "✅ Active");
        assert_eq!(format_status("syncing"), "🔄 Syncing");
        assert_eq!(format_status("lagging"), "⚠️  Lagging");
        assert_eq!(format_status("disconnected"), "❌ Disconnected");
        assert_eq!(format_status("unknown"), "unknown");
    }
}
