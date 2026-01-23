use anyhow::Result;
use base64::prelude::*;
use clap::{Parser, Subcommand};
use tracing::info;

mod audit;
mod auth;
mod backup;
mod config;
mod http_client;
mod middleware;
mod operator;
mod policy;
mod policy_parser;
mod seal;
mod token;
mod token_store;

use audit::{AuditCommand, execute_audit_command};
use auth::{login_command, logout_command};
use backup::{BackupCommand, execute_backup_command};
use config::CliConfig;
use http_client::AuthenticatedClient;
use middleware::SealChecker;
use operator::{OperatorCommand, execute_operator_command};
use policy::{PolicyCommand, execute_policy_command};
use seal::{SealCommand, execute_seal_command};
use token::{TokenCommand, execute_token_command};

#[derive(Parser)]
#[command(
    name = "secreton-cli",
    about = "Command line interface for Secreton engine system",
    version = "1.0.0",
    author = "Secreton Team"
)]
struct Cli {
    #[arg(short, long, global = true)]
    verbose: bool,

    #[arg(short, long, global = true)]
    config: Option<String>,

    #[arg(long, global = true)]
    server: Option<String>,

    #[arg(long, global = true)]
    namespace: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// System health and status commands
    Status,
    /// Login to Secreton engine
    Login {
        /// Authentication method (userpass or token)
        #[arg(short, long, default_value = "userpass")]
        method: String,

        /// Username for userpass authentication
        #[arg(short, long)]
        username: Option<String>,

        /// Token for token authentication
        #[arg(short, long)]
        token: Option<String>,
    },
    /// Logout from Secreton engine
    Logout,
    /// Configuration management
    #[command(subcommand)]
    Config(ConfigCommand),
    /// Policy management operations
    #[command(subcommand)]
    Policy(PolicyCommand),
    /// Token management operations
    #[command(subcommand)]
    Token(TokenCommand),
    /// Seal/unseal operations
    #[command(subcommand)]
    Seal(SealCommand),
    /// Backup and restore operations
    #[command(subcommand)]
    Backup(BackupCommand),
    /// Operator diagnostic commands
    #[command(subcommand)]
    Operator(OperatorCommand),
    /// Audit log commands
    #[command(subcommand)]
    Audit(AuditCommand),
    /// Transit engine operations (encryption/decryption)
    Transit {
        #[command(subcommand)]
        cmd: TransitCommand,
    },
    /// KV secrets engine operations
    Secret {
        #[command(subcommand)]
        cmd: SecretCommand,
    },
}

#[derive(Subcommand)]
enum ConfigCommand {
    /// Set a configuration value
    Set {
        /// Configuration key (e.g., "server", "namespace")
        key: String,
        /// Configuration value
        value: String,
    },
    /// Get a configuration value
    Get {
        /// Configuration key (e.g., "server", "namespace")
        key: String,
    },
    /// Show all configuration values
    Show,
}

#[derive(Subcommand)]
enum TransitCommand {
    /// Create a new encryption key
    CreateKey { name: String },
    /// List all encryption keys
    ListKeys,
    /// Encrypt data with a key
    Encrypt {
        key: String,
        #[arg(short, long)]
        data: Option<String>,
    },
    /// Decrypt data with a key
    Decrypt {
        key: String,
        #[arg(short, long)]
        data: Option<String>,
    },
}

#[derive(Subcommand)]
enum SecretCommand {
    /// Store a secret
    Put {
        path: String,
        #[arg(short, long)]
        data: Vec<String>, // key=value format
    },
    /// Retrieve a secret
    Get { path: String },
    /// List all secret paths
    List,
    /// Delete a secret
    Delete { path: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let level = if cli.verbose { "debug" } else { "info" };
    lib_common::telemetry::init_subscriber(level);

    // Load configuration with precedence: flag > env > config file > default
    let mut config = if let Some(config_path) = &cli.config {
        CliConfig::load_from_file(config_path).await?
    } else {
        CliConfig::load_default().await.unwrap_or_default()
    };

    // Override server URL with precedence: flag > env > config
    if let Some(server_url) = &cli.server {
        config.server_url = server_url.clone();
    } else if let Ok(env_addr) = std::env::var("SECRETON_ADDR") {
        config.server_url = env_addr;
    }

    // Override namespace if provided via command line
    if let Some(namespace) = &cli.namespace {
        config.default_namespace = namespace.clone();
    }

    info!("Using server: {}", config.server_url);
    info!("Using namespace: {}", config.default_namespace);

    // Execute commands
    match cli.command {
        Commands::Status => status_command(&config).await,
        Commands::Login {
            method,
            username,
            token,
        } => login_command(&config, &method, username, token).await,
        Commands::Logout => logout_command(&config).await,
        Commands::Config(cmd) => config_command(cmd, &config).await,
        Commands::Policy(cmd) => {
            execute_policy_command(cmd, &config, cli.namespace.as_deref()).await
        }
        Commands::Token(cmd) => execute_token_command(cmd, &config, cli.namespace.as_deref()).await,
        Commands::Seal(cmd) => execute_seal_command(cmd, &config).await,
        Commands::Backup(cmd) => execute_backup_command(cmd, &config).await,
        Commands::Operator(cmd) => execute_operator_command(cmd, &config).await,
        Commands::Audit(cmd) => execute_audit_command(cmd, &config).await,
        Commands::Transit { cmd } => transit_command(cmd, &config, cli.namespace.as_deref()).await,
        Commands::Secret { cmd } => secret_command(cmd, &config, cli.namespace.as_deref()).await,
    }
}

async fn config_command(cmd: ConfigCommand, config: &CliConfig) -> Result<()> {
    match cmd {
        ConfigCommand::Set { key, value } => {
            let mut new_config = config.clone();

            match key.as_str() {
                "server" => {
                    new_config.server_url = value.clone();
                    println!("✅ Server URL set to: {}", value);
                }
                "namespace" => {
                    new_config.default_namespace = value.clone();
                    println!("✅ Default namespace set to: {}", value);
                }
                _ => {
                    anyhow::bail!(
                        "Unknown configuration key: {}. Valid keys: server, namespace",
                        key
                    );
                }
            }

            new_config.save_default().await?;
            println!(
                "   Configuration saved to: {}",
                CliConfig::default_config_path()?.display()
            );

            Ok(())
        }
        ConfigCommand::Get { key } => {
            match key.as_str() {
                "server" => {
                    println!("{}", config.server_url);
                }
                "namespace" => {
                    println!("{}", config.default_namespace);
                }
                _ => {
                    anyhow::bail!(
                        "Unknown configuration key: {}. Valid keys: server, namespace",
                        key
                    );
                }
            }
            Ok(())
        }
        ConfigCommand::Show => {
            println!("📋 Current Configuration:");
            println!("   Server URL:        {}", config.server_url);
            println!("   Default Namespace: {}", config.default_namespace);
            println!();
            println!(
                "   Config file: {}",
                CliConfig::default_config_path()?.display()
            );
            Ok(())
        }
    }
}

async fn status_command(config: &CliConfig) -> Result<()> {
    let client = reqwest::Client::new();

    // Check health
    let health_url = format!("{}/health", config.server_url);
    let health_response = client.get(&health_url).send().await?;

    if health_response.status().is_success() {
        let health: serde_json::Value = health_response.json().await?;
        println!("🟢 Secreton Engine Status: HEALTHY");
        println!("   Server: {}", config.server_url);
        println!(
            "   Version: {}",
            health
                .get("version")
                .unwrap_or(&serde_json::Value::String("unknown".to_string()))
        );
        println!(
            "   Timestamp: {}",
            health
                .get("timestamp")
                .unwrap_or(&serde_json::Value::String("unknown".to_string()))
        );
    } else {
        println!("🔴 Secreton Engine Status: UNHEALTHY");
        println!("   HTTP Status: {}", health_response.status());
    }

    Ok(())
}

async fn transit_command(
    cmd: TransitCommand,
    config: &CliConfig,
    namespace: Option<&str>,
) -> Result<()> {
    // Check engine seal status before operations
    let mut seal_checker = SealChecker::new(config.server_url.clone(), None);
    if let Err(e) = seal_checker.require_unsealed().await {
        eprintln!("❌ {}", e);
        return Ok(());
    }

    // Create authenticated client
    let auth_client = AuthenticatedClient::new()?;
    let _effective_namespace = config.get_namespace(namespace);

    match cmd {
        TransitCommand::CreateKey { name } => {
            let url = format!("{}/v1/transit/keys/{}", config.server_url, name);
            let response = auth_client
                .execute(
                    auth_client
                        .post(&url)?
                        .header("Content-Type", "application/json")
                        .json(&serde_json::json!({})),
                )
                .await?;

            if response.status().is_success() {
                println!("✅ Created encryption key: {}", name);
            } else {
                println!("❌ Failed to create key: {}", response.status());
            }
        }
        TransitCommand::ListKeys => {
            let url = format!("{}/v1/transit/keys", config.server_url);
            let response = auth_client.execute(auth_client.get(&url)?).await?;

            if response.status().is_success() {
                let keys: serde_json::Value = response.json().await?;
                println!("🔑 Available encryption keys:");
                if let Some(key_list) = keys.get("keys").and_then(|k| k.as_array()) {
                    for key in key_list {
                        if let Some(key_str) = key.as_str() {
                            println!("   • {}", key_str);
                        }
                    }
                } else {
                    println!("   No keys found");
                }
            } else {
                println!("❌ Failed to list keys: {}", response.status());
            }
        }
        TransitCommand::Encrypt { key, data } => {
            let plaintext = if let Some(d) = data {
                d
            } else {
                // Read from stdin if no data provided
                use std::io::Read;
                let mut buffer = String::new();
                std::io::stdin().read_to_string(&mut buffer)?;
                buffer.trim().to_string()
            };

            // Base64 encode the plaintext
            let encoded_data = BASE64_STANDARD.encode(plaintext.as_bytes());

            let url = format!("{}/v1/transit/encrypt/{}", config.server_url, key);
            let payload = serde_json::json!({
                "plaintext": encoded_data
            });

            let response = auth_client
                .execute(
                    auth_client
                        .post(&url)?
                        .header("Content-Type", "application/json")
                        .json(&payload),
                )
                .await?;

            if response.status().is_success() {
                let result: serde_json::Value = response.json().await?;
                if let Some(ciphertext) = result.get("ciphertext").and_then(|c| c.as_str()) {
                    println!("🔐 Encrypted data:");
                    println!("{}", ciphertext);
                }
            } else {
                println!("❌ Failed to encrypt: {}", response.status());
            }
        }
        TransitCommand::Decrypt { key, data } => {
            let ciphertext = if let Some(d) = data {
                d
            } else {
                use std::io::Read;
                let mut buffer = String::new();
                std::io::stdin().read_to_string(&mut buffer)?;
                buffer.trim().to_string()
            };

            let url = format!("{}/v1/transit/decrypt/{}", config.server_url, key);
            let payload = serde_json::json!({
                "ciphertext": ciphertext
            });

            let response = auth_client
                .execute(
                    auth_client
                        .post(&url)?
                        .header("Content-Type", "application/json")
                        .json(&payload),
                )
                .await?;

            if response.status().is_success() {
                let result: serde_json::Value = response.json().await?;
                if let Some(plaintext_b64) = result.get("plaintext").and_then(|p| p.as_str()) {
                    // Base64 decode the result
                    let decoded = BASE64_STANDARD.decode(plaintext_b64)?;
                    let plaintext = String::from_utf8(decoded)?;
                    println!("🔓 Decrypted data:");
                    println!("{}", plaintext);
                }
            } else {
                println!("❌ Failed to decrypt: {}", response.status());
            }
        }
    }

    Ok(())
}

async fn secret_command(
    cmd: SecretCommand,
    config: &CliConfig,
    namespace: Option<&str>,
) -> Result<()> {
    // Check engine seal status before operations
    let mut seal_checker = SealChecker::new(config.server_url.clone(), None);
    if let Err(e) = seal_checker.require_unsealed().await {
        eprintln!("❌ {}", e);
        return Ok(());
    }

    // Create authenticated client
    let auth_client = AuthenticatedClient::new()?;
    let _effective_namespace = config.get_namespace(namespace);

    match cmd {
        SecretCommand::Put { path, data } => {
            // Parse key=value pairs
            let mut secret_data = serde_json::Map::new();
            for pair in data {
                if let Some((key, value)) = pair.split_once('=') {
                    secret_data.insert(
                        key.to_string(),
                        serde_json::Value::String(value.to_string()),
                    );
                } else {
                    println!("❌ Invalid format '{}'. Use key=value format.", pair);
                    return Ok(());
                }
            }

            let url = format!("{}/v1/secret/data/{}", config.server_url, path);
            let payload = serde_json::json!({
                "data": secret_data
            });

            let response = auth_client
                .execute(
                    auth_client
                        .post(&url)?
                        .header("Content-Type", "application/json")
                        .json(&payload),
                )
                .await?;

            if response.status().is_success() {
                let result: serde_json::Value = response.json().await?;
                if let Some(version) = result.get("version").and_then(|v| v.as_u64()) {
                    println!("✅ Secret stored at path '{}' (version {})", path, version);
                } else {
                    println!("✅ Secret stored at path '{}'", path);
                }
            } else {
                println!("❌ Failed to store secret: {}", response.status());
            }
        }
        SecretCommand::Get { path } => {
            let url = format!("{}/v1/secret/data/{}", config.server_url, path);
            let response = auth_client.execute(auth_client.get(&url)?).await?;

            if response.status().is_success() {
                let result: serde_json::Value = response.json().await?;
                if let Some(data) = result.get("data") {
                    println!("🔍 Secret at path '{}':", path);
                    println!("{}", serde_json::to_string_pretty(data)?);
                }
                if let Some(version) = result.get("version").and_then(|v| v.as_u64()) {
                    println!("Version: {}", version);
                }
            } else if response.status() == 404 {
                println!("❌ Secret not found at path '{}'", path);
            } else {
                println!("❌ Failed to retrieve secret: {}", response.status());
            }
        }
        SecretCommand::List => {
            let url = format!("{}/v1/secrets", config.server_url);
            let response = auth_client.execute(auth_client.get(&url)?).await?;

            if response.status().is_success() {
                let result: serde_json::Value = response.json().await?;
                if let Some(secrets) = result.get("keys").and_then(|s| s.as_array()) {
                    println!("📋 Available secrets:");
                    for secret in secrets {
                        if let Some(path) = secret.as_str() {
                            println!("   • {}", path);
                        }
                    }
                } else {
                    println!("📋 No secrets found");
                }
            } else {
                println!("❌ Failed to list secrets: {}", response.status());
            }
        }
        SecretCommand::Delete { path } => {
            let url = format!("{}/v1/secret/data/{}", config.server_url, path);
            let response = auth_client.execute(auth_client.delete(&url)?).await?;

            if response.status().is_success() {
                println!("✅ Secret deleted at path '{}'", path);
            } else if response.status() == 404 {
                println!("❌ Secret not found at path '{}'", path);
            } else {
                println!("❌ Failed to delete secret: {}", response.status());
            }
        }
    }

    Ok(())
}
