//! Audit module for viewing and filtering audit logs
//!
//! This module provides commands for viewing audit logs with filtering
//! capabilities by user, operation, path, and time range.

use crate::config::CliConfig;
use crate::token_store::TokenStore;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::Subcommand;
use comfy_table::{presets::UTF8_FULL, Cell, Color, ContentArrangement, Table};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Subcommand)]
pub enum AuditCommand {
    /// List audit log entries with optional filtering
    List {
        /// Filter by user/actor
        #[arg(short, long)]
        user: Option<String>,

        /// Filter by operation/action
        #[arg(short, long)]
        operation: Option<String>,

        /// Filter by resource path
        #[arg(short, long)]
        path: Option<String>,

        /// Filter by start time (RFC3339 format, e.g., 2025-12-01T00:00:00Z)
        #[arg(long)]
        start_time: Option<String>,

        /// Filter by end time (RFC3339 format, e.g., 2025-12-31T23:59:59Z)
        #[arg(long)]
        end_time: Option<String>,

        /// Number of entries to display (default: 50)
        #[arg(short, long, default_value = "50")]
        limit: usize,

        /// Output format (table, json, csv)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
}

/// Audit log entry structure matching the API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub action: String,
    pub actor: Option<String>,
    pub resource_type: String,
    pub resource_id: String,
    pub status: AuditStatus,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub namespace: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Status of an audited action
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AuditStatus {
    Success,
    Failure,
    Denied,
}

impl std::fmt::Display for AuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditStatus::Success => write!(f, "Success"),
            AuditStatus::Failure => write!(f, "Failure"),
            AuditStatus::Denied => write!(f, "Denied"),
        }
    }
}

/// Execute audit commands
pub async fn execute_audit_command(cmd: AuditCommand, config: &CliConfig) -> Result<()> {
    match cmd {
        AuditCommand::List {
            user,
            operation,
            path,
            start_time,
            end_time,
            limit,
            format,
        } => {
            list_audit_logs(
                config, user, operation, path, start_time, end_time, limit, &format,
            )
            .await
        }
    }
}

/// List audit logs with filtering
async fn list_audit_logs(
    config: &CliConfig,
    user: Option<String>,
    operation: Option<String>,
    path: Option<String>,
    start_time: Option<String>,
    end_time: Option<String>,
    limit: usize,
    format: &str,
) -> Result<()> {
    // Load token for authentication
    let token_store = TokenStore::new()?;
    let stored_token = token_store
        .load_token()?
        .context("No authentication token found. Run 'secreton login' first.")?;

    // Build query parameters
    let mut query_params = vec![format!("limit={}", limit)];

    if let Some(u) = user {
        query_params.push(format!("user_id={}", urlencoding::encode(&u)));
    }

    if let Some(op) = operation {
        query_params.push(format!("action={}", urlencoding::encode(&op)));
    }

    if let Some(p) = path {
        query_params.push(format!("resource_id={}", urlencoding::encode(&p)));
    }

    if let Some(st) = start_time {
        // Validate and parse the time
        let _parsed: DateTime<Utc> = st
            .parse()
            .context("Invalid start_time format. Use RFC3339 format (e.g., 2025-12-01T00:00:00Z)")?;
        query_params.push(format!("start_time={}", urlencoding::encode(&st)));
    }

    if let Some(et) = end_time {
        // Validate and parse the time
        let _parsed: DateTime<Utc> = et
            .parse()
            .context("Invalid end_time format. Use RFC3339 format (e.g., 2025-12-31T23:59:59Z)")?;
        query_params.push(format!("end_time={}", urlencoding::encode(&et)));
    }

    let query_string = if query_params.is_empty() {
        String::new()
    } else {
        format!("?{}", query_params.join("&"))
    };

    // Make API request
    let client = reqwest::Client::new();
    let url = format!("{}/v1/audit/logs{}", config.server_url, query_string);

    let response = client
        .get(&url)
        .header("X-Secreton-Token", &stored_token.token)
        .send()
        .await
        .context("Failed to connect to Secreton server")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();

        if status.as_u16() == 403 {
            anyhow::bail!("Permission denied. Your token may not have audit read permissions.");
        } else if status.as_u16() == 401 {
            anyhow::bail!("Authentication failed. Run 'secreton login' to re-authenticate.");
        } else {
            anyhow::bail!("API error: {} - {}", status, error_text);
        }
    }

    // Parse response
    let api_response: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse API response")?;

    let logs: Vec<AuditLog> = if let Some(data) = api_response.get("data") {
        serde_json::from_value(data.clone()).context("Failed to parse audit logs")?
    } else {
        vec![]
    };

    // Display results based on format
    match format {
        "json" => display_json(&logs)?,
        "csv" => display_csv(&logs)?,
        _ => display_table(&logs)?,
    }

    Ok(())
}

/// Display audit logs as a formatted table
fn display_table(logs: &[AuditLog]) -> Result<()> {
    if logs.is_empty() {
        println!("📋 No audit logs found matching the criteria.");
        return Ok(());
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);

    // Add header
    table.set_header(vec![
        Cell::new("Timestamp").fg(Color::Cyan),
        Cell::new("User").fg(Color::Cyan),
        Cell::new("Action").fg(Color::Cyan),
        Cell::new("Resource").fg(Color::Cyan),
        Cell::new("Status").fg(Color::Cyan),
        Cell::new("IP").fg(Color::Cyan),
    ]);

    // Add rows
    for log in logs {
        let status_cell = match log.status {
            AuditStatus::Success => Cell::new(log.status.to_string()).fg(Color::Green),
            AuditStatus::Failure => Cell::new(log.status.to_string()).fg(Color::Red),
            AuditStatus::Denied => Cell::new(log.status.to_string()).fg(Color::Yellow),
        };

        table.add_row(vec![
            Cell::new(log.timestamp.format("%Y-%m-%d %H:%M:%S")),
            Cell::new(log.actor.as_deref().unwrap_or("system")),
            Cell::new(&log.action),
            Cell::new(format!("{}/{}", log.resource_type, log.resource_id)),
            status_cell,
            Cell::new(log.ip.as_deref().unwrap_or("-")),
        ]);
    }

    println!("\n📋 Audit Logs ({} entries)\n", logs.len());
    println!("{}", table);

    Ok(())
}

/// Display audit logs as JSON
fn display_json(logs: &[AuditLog]) -> Result<()> {
    let json = serde_json::to_string_pretty(logs)?;
    println!("{}", json);
    Ok(())
}

/// Display audit logs as CSV
fn display_csv(logs: &[AuditLog]) -> Result<()> {
    // Print header
    println!("timestamp,user,action,resource_type,resource_id,status,ip,namespace");

    // Print rows
    for log in logs {
        println!(
            "{},{},{},{},{},{},{},{}",
            log.timestamp.to_rfc3339(),
            log.actor.as_deref().unwrap_or("system"),
            log.action,
            log.resource_type,
            log.resource_id,
            log.status,
            log.ip.as_deref().unwrap_or(""),
            log.namespace.as_deref().unwrap_or("")
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_status_display() {
        assert_eq!(AuditStatus::Success.to_string(), "Success");
        assert_eq!(AuditStatus::Failure.to_string(), "Failure");
        assert_eq!(AuditStatus::Denied.to_string(), "Denied");
    }

    #[test]
    fn test_audit_command_enum() {
        // Just verify the enum compiles and can be created
        // This is a smoke test for the command structure
    }

    #[test]
    fn test_display_empty_logs() {
        let logs: Vec<AuditLog> = vec![];
        let result = display_table(&logs);
        assert!(result.is_ok());
    }

    #[test]
    fn test_display_json_empty() {
        let logs: Vec<AuditLog> = vec![];
        let result = display_json(&logs);
        assert!(result.is_ok());
    }

    #[test]
    fn test_display_csv_empty() {
        let logs: Vec<AuditLog> = vec![];
        let result = display_csv(&logs);
        assert!(result.is_ok());
    }
}
