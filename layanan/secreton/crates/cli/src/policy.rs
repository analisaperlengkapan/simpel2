//! Policy management commands
//!
//! This module provides CLI commands for managing Secreton policies including
//! list, read, write, delete, format, validate, and test operations.

use anyhow::{Context, Result};
use clap::Subcommand;
use comfy_table::{Cell, Color, ContentArrangement, Table, presets::UTF8_FULL};
use std::path::{Path, PathBuf};

use crate::config::CliConfig;
use crate::middleware::SealChecker;
use crate::policy_parser::{
    Policy, PolicyParser,
    formatter::{OutputFormat, PolicyFormatter},
    toml_parser::TomlPolicyParser,
};
use crate::token_store::TokenStore;

/// Policy management subcommands
#[derive(Subcommand, Debug)]
pub enum PolicyCommand {
    /// List all policies
    List {
        /// Namespace to list policies from
        #[arg(long)]
        namespace: Option<String>,

        /// Output format (table, json, json-pretty)
        #[arg(long, default_value = "table")]
        format: String,
    },

    /// Read a policy
    Read {
        /// Policy name
        name: String,

        /// Output format (toml, json, json-pretty)
        #[arg(long, default_value = "toml")]
        format: String,
    },

    /// Write a policy from a file
    Write {
        /// Policy name
        name: String,

        /// Path to policy file
        file: PathBuf,
    },

    /// Delete a policy
    Delete {
        /// Policy name
        name: String,

        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },

    /// Format a policy file
    Fmt {
        /// Path to policy file
        file: PathBuf,

        /// Check if formatting is needed without modifying the file
        #[arg(long)]
        check: bool,

        /// Output format (toml, json, json-pretty)
        #[arg(long, default_value = "toml")]
        format: String,
    },

    /// Validate a policy file
    Validate {
        /// Path to policy file
        file: PathBuf,
    },

    /// Test a policy against a path and action
    Test {
        /// Policy name
        name: String,

        /// Path to test
        #[arg(long)]
        path: String,

        /// Action to test (create, read, update, delete, list)
        #[arg(long)]
        action: String,
    },
}

/// Execute a policy command
pub async fn execute_policy_command(
    cmd: PolicyCommand,
    config: &CliConfig,
    global_namespace: Option<&str>,
) -> Result<()> {
    match cmd {
        PolicyCommand::List { namespace, format } => {
            // Command-specific namespace takes precedence over global namespace
            let effective_namespace = namespace.or_else(|| global_namespace.map(|s| s.to_string()));
            list_policies(config, effective_namespace, &format).await
        }
        PolicyCommand::Read { name, format } => read_policy(config, &name, &format).await,
        PolicyCommand::Write { name, file } => write_policy(config, &name, &file).await,
        PolicyCommand::Delete { name, force } => delete_policy(config, &name, force).await,
        PolicyCommand::Fmt {
            file,
            check,
            format,
        } => format_policy_file(&file, check, &format),
        PolicyCommand::Validate { file } => validate_policy_file(&file),
        PolicyCommand::Test { name, path, action } => {
            test_policy(config, &name, &path, &action).await
        }
    }
}

/// List all policies
async fn list_policies(config: &CliConfig, namespace: Option<String>, format: &str) -> Result<()> {
    // Check engine seal status before operations
    let mut seal_checker = SealChecker::new(config.server_url.clone(), None);
    if let Err(e) = seal_checker.require_unsealed().await {
        eprintln!("❌ {}", e);
        return Ok(());
    }

    let client = reqwest::Client::new();
    let token_store = TokenStore::new()?;

    // Get authentication token
    let token = token_store
        .load_token()?
        .context("Not authenticated. Run 'secreton login' first.")?;

    // Build URL with optional namespace
    let url = if let Some(ns) = namespace {
        format!("{}/v1/sys/policies?namespace={}", config.server_url, ns)
    } else {
        format!("{}/v1/sys/policies", config.server_url)
    };

    // Make API request
    let response = client
        .get(&url)
        .header("X-Secreton-Token", &token.token)
        .send()
        .await
        .context("Failed to connect to Secreton server")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        anyhow::bail!("Failed to list policies: {} - {}", status, error_text);
    }

    let result: serde_json::Value = response.json().await.context("Failed to parse response")?;

    // Handle different output formats
    match format {
        "json" => {
            println!("{}", serde_json::to_string(&result)?);
        }
        "json-pretty" => {
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        _ => {
            // Extract policies array
            let policies = result
                .get("policies")
                .and_then(|p| p.as_array())
                .context("Invalid response format")?;

            if policies.is_empty() {
                println!("📋 No policies found");
                return Ok(());
            }

            // Create table
            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .set_content_arrangement(ContentArrangement::Dynamic)
                .set_header(vec![
                    Cell::new("Name").fg(Color::Cyan),
                    Cell::new("Namespace").fg(Color::Cyan),
                    Cell::new("Rules").fg(Color::Cyan),
                ]);

            for policy in policies {
                let name = policy
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("unknown");
                let ns = policy
                    .get("namespace")
                    .and_then(|n| n.as_str())
                    .unwrap_or("default");
                let rule_count = policy
                    .get("rules")
                    .and_then(|r| r.as_array())
                    .map(|r| r.len())
                    .unwrap_or(0);

                table.add_row(vec![name, ns, &rule_count.to_string()]);
            }

            println!("📋 Policies:");
            println!("{table}");
        }
    }

    Ok(())
}

/// Read a policy
async fn read_policy(config: &CliConfig, name: &str, format: &str) -> Result<()> {
    // Check engine seal status before operations
    let mut seal_checker = SealChecker::new(config.server_url.clone(), None);
    if let Err(e) = seal_checker.require_unsealed().await {
        eprintln!("❌ {}", e);
        return Ok(());
    }

    let client = reqwest::Client::new();
    let token_store = TokenStore::new()?;

    // Get authentication token
    let token = token_store
        .load_token()?
        .context("Not authenticated. Run 'secreton login' first.")?;

    // Build URL
    let url = format!("{}/v1/sys/policies/{}", config.server_url, name);

    // Make API request
    let response = client
        .get(&url)
        .header("X-Secreton-Token", &token.token)
        .send()
        .await
        .context("Failed to connect to Secreton server")?;

    if !response.status().is_success() {
        let status = response.status();
        if status == 404 {
            anyhow::bail!("Policy '{}' not found", name);
        }
        let error_text = response.text().await.unwrap_or_default();
        anyhow::bail!("Failed to read policy: {} - {}", status, error_text);
    }

    let policy: Policy = response
        .json()
        .await
        .context("Failed to parse policy response")?;

    // Format output
    let formatter = PolicyFormatter::new();
    let output_format = format.parse::<OutputFormat>().unwrap_or_else(|e| {
        eprintln!("Warning: {}. Using TOML format.", e);
        OutputFormat::Toml
    });

    let formatted = formatter
        .format_policy(&policy, output_format)
        .context("Failed to format policy")?;

    println!("{}", formatted);

    Ok(())
}

/// Write a policy from a file
async fn write_policy(config: &CliConfig, name: &str, file: &PathBuf) -> Result<()> {
    // Check engine seal status before operations
    let mut seal_checker = SealChecker::new(config.server_url.clone(), None);
    if let Err(e) = seal_checker.require_unsealed().await {
        eprintln!("❌ {}", e);
        return Ok(());
    }

    let client = reqwest::Client::new();
    let token_store = TokenStore::new()?;

    // Get authentication token
    let token = token_store
        .load_token()?
        .context("Not authenticated. Run 'secreton login' first.")?;

    // Read and parse policy file
    let content = std::fs::read_to_string(file)
        .with_context(|| format!("Failed to read policy file: {}", file.display()))?;

    let parser = TomlPolicyParser::new();
    let mut policy = parser
        .parse(&content)
        .context("Failed to parse policy file")?;

    // Override policy name with the one provided
    policy.name = name.to_string();

    // Build URL
    let url = format!("{}/v1/sys/policies/{}", config.server_url, name);

    // Make API request
    let response = client
        .post(&url)
        .header("X-Secreton-Token", &token.token)
        .header("Content-Type", "application/json")
        .json(&policy)
        .send()
        .await
        .context("Failed to connect to Secreton server")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        anyhow::bail!("Failed to write policy: {} - {}", status, error_text);
    }

    println!("✅ Policy '{}' written successfully", name);

    Ok(())
}

/// Delete a policy
async fn delete_policy(config: &CliConfig, name: &str, force: bool) -> Result<()> {
    // Check engine seal status before operations
    let mut seal_checker = SealChecker::new(config.server_url.clone(), None);
    if let Err(e) = seal_checker.require_unsealed().await {
        eprintln!("❌ {}", e);
        return Ok(());
    }

    let client = reqwest::Client::new();
    let token_store = TokenStore::new()?;

    // Get authentication token
    let token = token_store
        .load_token()?
        .context("Not authenticated. Run 'secreton login' first.")?;

    // Confirm deletion unless force flag is set
    if !force {
        print!("Are you sure you want to delete policy '{}'? (y/N): ", name);
        use std::io::{self, Write};
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Deletion cancelled");
            return Ok(());
        }
    }

    // Build URL
    let url = format!("{}/v1/sys/policies/{}", config.server_url, name);

    // Make API request
    let response = client
        .delete(&url)
        .header("X-Secreton-Token", &token.token)
        .send()
        .await
        .context("Failed to connect to Secreton server")?;

    if !response.status().is_success() {
        let status = response.status();
        if status == 404 {
            anyhow::bail!("Policy '{}' not found", name);
        }
        let error_text = response.text().await.unwrap_or_default();
        anyhow::bail!("Failed to delete policy: {} - {}", status, error_text);
    }

    println!("✅ Policy '{}' deleted successfully", name);

    Ok(())
}

/// Format a policy file
fn format_policy_file(file: &Path, check: bool, format: &str) -> Result<()> {
    let formatter = PolicyFormatter::new();
    let output_format = format.parse::<OutputFormat>().unwrap_or_else(|e| {
        eprintln!("Warning: {}. Using TOML format.", e);
        OutputFormat::Toml
    });

    let result = formatter
        .format_file(file, output_format, check)
        .with_context(|| format!("Failed to format policy file: {}", file.display()))?;

    if check {
        if result.needs_formatting {
            println!("❌ File needs formatting: {}", file.display());
            std::process::exit(1);
        } else {
            println!("✅ File is properly formatted: {}", file.display());
        }
    } else {
        println!("✅ Formatted policy file: {}", file.display());
    }

    Ok(())
}

/// Validate a policy file
fn validate_policy_file(file: &PathBuf) -> Result<()> {
    let content = std::fs::read_to_string(file)
        .with_context(|| format!("Failed to read policy file: {}", file.display()))?;

    let parser = TomlPolicyParser::new();
    let validation_result = parser.validate(&content);

    match validation_result {
        Ok(errors) => {
            if errors.is_empty() {
                println!("✅ Policy file is valid: {}", file.display());
                Ok(())
            } else {
                println!("❌ Policy file has validation errors:");
                for error in errors {
                    if let Some(line) = error.line {
                        println!("  Line {}: {} - {}", line, error.field, error.message);
                    } else {
                        println!("  {}: {}", error.field, error.message);
                    }
                }
                std::process::exit(1);
            }
        }
        Err(e) => {
            anyhow::bail!("Validation failed: {}", e);
        }
    }
}

/// Test a policy against a path and action
async fn test_policy(config: &CliConfig, name: &str, path: &str, action: &str) -> Result<()> {
    // Check engine seal status before operations
    let mut seal_checker = SealChecker::new(config.server_url.clone(), None);
    if let Err(e) = seal_checker.require_unsealed().await {
        eprintln!("❌ {}", e);
        return Ok(());
    }

    let client = reqwest::Client::new();
    let token_store = TokenStore::new()?;

    // Get authentication token
    let token = token_store
        .load_token()?
        .context("Not authenticated. Run 'secreton login' first.")?;

    // Build URL
    let url = format!("{}/v1/sys/policies/{}/test", config.server_url, name);

    // Build request payload
    let payload = serde_json::json!({
        "path": path,
        "action": action,
    });

    // Make API request
    let response = client
        .post(&url)
        .header("X-Secreton-Token", &token.token)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .context("Failed to connect to Secreton server")?;

    if !response.status().is_success() {
        let status = response.status();
        if status == 404 {
            anyhow::bail!("Policy '{}' not found", name);
        }
        let error_text = response.text().await.unwrap_or_default();
        anyhow::bail!("Failed to test policy: {} - {}", status, error_text);
    }

    let result: serde_json::Value = response.json().await.context("Failed to parse response")?;

    // Display test results
    println!("🧪 Policy Test Results:");
    println!("  Policy: {}", name);
    println!("  Path: {}", path);
    println!("  Action: {}", action);
    println!();

    let allowed = result
        .get("allowed")
        .and_then(|a| a.as_bool())
        .unwrap_or(false);

    if allowed {
        println!("✅ Access ALLOWED");
    } else {
        println!("❌ Access DENIED");
    }

    // Display matched rules if available
    if let Some(matched_rules) = result.get("matched_rules").and_then(|r| r.as_array())
        && !matched_rules.is_empty()
    {
        println!();
        println!("Matched Rules:");
        for (i, rule) in matched_rules.iter().enumerate() {
            println!("  {}. {}", i + 1, serde_json::to_string_pretty(rule)?);
        }
    }

    // Display reason if available
    if let Some(reason) = result.get("reason").and_then(|r| r.as_str()) {
        println!();
        println!("Reason: {}", reason);
    }

    Ok(())
}
