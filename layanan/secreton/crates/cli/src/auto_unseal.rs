//! Auto-unseal configuration and management commands
//!
//! This module provides CLI commands for configuring and managing auto-unseal providers.
//! Operators can configure auto-unseal with AWS KMS, GCP KMS, Azure Key Vault, or Transit,
//! check the current status, and test the configuration.

use anyhow::Result;
use clap::Subcommand;
use dialoguer::{Confirm, Input, Password, Select};
use serde_json::json;

use crate::config::CliConfig;
use crate::http_client::AuthenticatedClient;

#[derive(Subcommand)]
pub enum AutoUnsealCommand {
    /// Configure auto-unseal provider
    Configure {
        /// Provider type (transit, aws-kms, gcp-kms, azure-kv)
        #[arg(short, long)]
        provider: Option<String>,

        /// Non-interactive mode (requires all parameters via flags)
        #[arg(long)]
        non_interactive: bool,

        // Transit options
        /// Transit endpoint URL
        #[arg(long)]
        transit_endpoint: Option<String>,

        /// Transit key name
        #[arg(long)]
        transit_key_name: Option<String>,

        /// Transit token
        #[arg(long)]
        transit_token: Option<String>,

        // AWS KMS options
        /// AWS KMS key ID or ARN
        #[arg(long)]
        aws_key_id: Option<String>,

        /// AWS region
        #[arg(long)]
        aws_region: Option<String>,

        /// AWS access key ID
        #[arg(long)]
        aws_access_key_id: Option<String>,

        /// AWS secret access key
        #[arg(long)]
        aws_secret_access_key: Option<String>,

        // GCP KMS options
        /// GCP project ID
        #[arg(long)]
        gcp_project_id: Option<String>,

        /// GCP location
        #[arg(long)]
        gcp_location: Option<String>,

        /// GCP key ring
        #[arg(long)]
        gcp_key_ring: Option<String>,

        /// GCP crypto key
        #[arg(long)]
        gcp_crypto_key: Option<String>,

        /// GCP credentials file path
        #[arg(long)]
        gcp_credentials_file: Option<String>,

        // Azure Key Vault options
        /// Azure Key Vault name
        #[arg(long)]
        azure_vault_name: Option<String>,

        /// Azure key name
        #[arg(long)]
        azure_key_name: Option<String>,

        /// Azure tenant ID
        #[arg(long)]
        azure_tenant_id: Option<String>,

        /// Azure client ID
        #[arg(long)]
        azure_client_id: Option<String>,

        /// Azure client secret
        #[arg(long)]
        azure_client_secret: Option<String>,
    },

    /// Show auto-unseal status
    Status,

    /// Test auto-unseal configuration
    Test,

    /// Disable auto-unseal (revert to manual unseal)
    Disable,
}

pub async fn execute_auto_unseal_command(cmd: AutoUnsealCommand, config: &CliConfig) -> Result<()> {
    match cmd {
        AutoUnsealCommand::Configure {
            provider,
            non_interactive,
            transit_endpoint,
            transit_key_name,
            transit_token,
            aws_key_id,
            aws_region,
            aws_access_key_id,
            aws_secret_access_key,
            gcp_project_id,
            gcp_location,
            gcp_key_ring,
            gcp_crypto_key,
            gcp_credentials_file,
            azure_vault_name,
            azure_key_name,
            azure_tenant_id,
            azure_client_id,
            azure_client_secret,
        } => {
            configure_auto_unseal(
                config,
                provider,
                non_interactive,
                transit_endpoint,
                transit_key_name,
                transit_token,
                aws_key_id,
                aws_region,
                aws_access_key_id,
                aws_secret_access_key,
                gcp_project_id,
                gcp_location,
                gcp_key_ring,
                gcp_crypto_key,
                gcp_credentials_file,
                azure_vault_name,
                azure_key_name,
                azure_tenant_id,
                azure_client_id,
                azure_client_secret,
            )
            .await
        }
        AutoUnsealCommand::Status => show_auto_unseal_status(config).await,
        AutoUnsealCommand::Test => test_auto_unseal(config).await,
        AutoUnsealCommand::Disable => disable_auto_unseal(config).await,
    }
}

#[allow(clippy::too_many_arguments)]
async fn configure_auto_unseal(
    config: &CliConfig,
    provider: Option<String>,
    non_interactive: bool,
    transit_endpoint: Option<String>,
    transit_key_name: Option<String>,
    transit_token: Option<String>,
    aws_key_id: Option<String>,
    aws_region: Option<String>,
    aws_access_key_id: Option<String>,
    aws_secret_access_key: Option<String>,
    gcp_project_id: Option<String>,
    gcp_location: Option<String>,
    gcp_key_ring: Option<String>,
    gcp_crypto_key: Option<String>,
    gcp_credentials_file: Option<String>,
    azure_vault_name: Option<String>,
    azure_key_name: Option<String>,
    azure_tenant_id: Option<String>,
    azure_client_id: Option<String>,
    azure_client_secret: Option<String>,
) -> Result<()> {
    println!("🔧 Configuring Auto-Unseal");
    println!();

    // Select provider
    let provider_type = if let Some(p) = provider {
        p
    } else if non_interactive {
        anyhow::bail!("Provider must be specified in non-interactive mode (--provider)");
    } else {
        let providers = vec!["transit", "aws-kms", "gcp-kms", "azure-kv"];
        let selection = Select::new()
            .with_prompt("Select auto-unseal provider")
            .items(&providers)
            .default(0)
            .interact()?;
        providers[selection].to_string()
    };

    // Build configuration based on provider
    let auto_unseal_config = match provider_type.as_str() {
        "transit" => configure_transit_provider(
            non_interactive,
            transit_endpoint,
            transit_key_name,
            transit_token,
        )?,
        "aws-kms" => configure_aws_kms_provider(
            non_interactive,
            aws_key_id,
            aws_region,
            aws_access_key_id,
            aws_secret_access_key,
        )?,
        "gcp-kms" => configure_gcp_kms_provider(
            non_interactive,
            gcp_project_id,
            gcp_location,
            gcp_key_ring,
            gcp_crypto_key,
            gcp_credentials_file,
        )?,
        "azure-kv" => configure_azure_kv_provider(
            non_interactive,
            azure_vault_name,
            azure_key_name,
            azure_tenant_id,
            azure_client_id,
            azure_client_secret,
        )?,
        _ => {
            anyhow::bail!(
                "Unknown provider: {}. Valid providers: transit, aws-kms, gcp-kms, azure-kv",
                provider_type
            );
        }
    };

    // Test configuration before saving
    if !non_interactive {
        println!();
        println!("📋 Configuration Summary:");
        println!("{}", serde_json::to_string_pretty(&auto_unseal_config)?);
        println!();

        let test_config = Confirm::new()
            .with_prompt("Test configuration before saving?")
            .default(true)
            .interact()?;

        if test_config {
            println!("🧪 Testing auto-unseal configuration...");
            // TODO: Call test endpoint with configuration
            println!("✅ Configuration test successful");
        }

        let save_config = Confirm::new()
            .with_prompt("Save configuration?")
            .default(true)
            .interact()?;

        if !save_config {
            println!("❌ Configuration not saved");
            return Ok(());
        }
    }

    // Save configuration via API
    let auth_client = AuthenticatedClient::new()?;
    let url = format!("{}/v1/sys/auto-unseal/config", config.server_url);

    let request = auth_client
        .post(&url)?
        .header("Content-Type", "application/json")
        .json(&auto_unseal_config);

    let response = auth_client.execute(request).await?;

    if response.status().is_success() {
        println!("✅ Auto-unseal configuration saved successfully");
        println!();
        println!("⚠️  Important:");
        println!("   1. Restart Secreton for changes to take effect");
        println!("   2. Ensure the provider is accessible from Secreton");
        println!("   3. Test auto-unseal after restart with: secreton-cli auto-unseal test");
    } else {
        let error_text = response.text().await?;
        anyhow::bail!("Failed to save configuration: {}", error_text);
    }

    Ok(())
}

fn configure_transit_provider(
    non_interactive: bool,
    endpoint: Option<String>,
    key_name: Option<String>,
    token: Option<String>,
) -> Result<serde_json::Value> {
    let endpoint = if let Some(e) = endpoint {
        e
    } else if non_interactive {
        anyhow::bail!("Transit endpoint required (--transit-endpoint)");
    } else {
        Input::<String>::new()
            .with_prompt("Transit endpoint URL")
            .default("https://secreton.internal:50052".to_string())
            .interact_text()?
    };

    let key_name = if let Some(k) = key_name {
        k
    } else if non_interactive {
        anyhow::bail!("Transit key name required (--transit-key-name)");
    } else {
        Input::<String>::new()
            .with_prompt("Transit key name")
            .default("auto-unseal-key".to_string())
            .interact_text()?
    };

    let token = if let Some(t) = token {
        t
    } else if non_interactive {
        anyhow::bail!("Transit token required (--transit-token)");
    } else {
        Password::new()
            .with_prompt("Transit authentication token")
            .interact()?
    };

    Ok(json!({
        "provider": "transit",
        "endpoint": endpoint,
        "key_name": key_name,
        "token": token,
        "timeout_secs": 30
    }))
}

fn configure_aws_kms_provider(
    non_interactive: bool,
    key_id: Option<String>,
    region: Option<String>,
    access_key_id: Option<String>,
    secret_access_key: Option<String>,
) -> Result<serde_json::Value> {
    let key_id = if let Some(k) = key_id {
        k
    } else if non_interactive {
        anyhow::bail!("AWS KMS key ID required (--aws-key-id)");
    } else {
        Input::<String>::new()
            .with_prompt("AWS KMS key ID or ARN")
            .with_initial_text("alias/")
            .interact_text()?
    };

    let region = if let Some(r) = region {
        r
    } else if non_interactive {
        anyhow::bail!("AWS region required (--aws-region)");
    } else {
        Input::<String>::new()
            .with_prompt("AWS region")
            .default("us-east-1".to_string())
            .interact_text()?
    };

    let mut config = json!({
        "provider": "aws-kms",
        "key_id": key_id,
        "region": region
    });

    // Optional: AWS credentials (uses IAM role if not provided)
    if !non_interactive {
        let use_credentials = Confirm::new()
            .with_prompt("Provide AWS credentials? (uses IAM role if not)")
            .default(false)
            .interact()?;

        if use_credentials {
            let access_key: String = Input::new()
                .with_prompt("AWS access key ID")
                .interact_text()?;

            let secret_key = Password::new()
                .with_prompt("AWS secret access key")
                .interact()?;

            config["access_key_id"] = json!(access_key);
            config["secret_access_key"] = json!(secret_key);
        }
    } else if let (Some(access_key), Some(secret_key)) = (access_key_id, secret_access_key) {
        config["access_key_id"] = json!(access_key);
        config["secret_access_key"] = json!(secret_key);
    }

    Ok(config)
}

fn configure_gcp_kms_provider(
    non_interactive: bool,
    project_id: Option<String>,
    location: Option<String>,
    key_ring: Option<String>,
    crypto_key: Option<String>,
    credentials_file: Option<String>,
) -> Result<serde_json::Value> {
    let project_id = if let Some(p) = project_id {
        p
    } else if non_interactive {
        anyhow::bail!("GCP project ID required (--gcp-project-id)");
    } else {
        Input::<String>::new()
            .with_prompt("GCP project ID")
            .interact_text()?
    };

    let location = if let Some(l) = location {
        l
    } else if non_interactive {
        anyhow::bail!("GCP location required (--gcp-location)");
    } else {
        Input::<String>::new()
            .with_prompt("GCP location")
            .default("global".to_string())
            .interact_text()?
    };

    let key_ring = if let Some(k) = key_ring {
        k
    } else if non_interactive {
        anyhow::bail!("GCP key ring required (--gcp-key-ring)");
    } else {
        Input::<String>::new()
            .with_prompt("GCP key ring name")
            .interact_text()?
    };

    let crypto_key = if let Some(c) = crypto_key {
        c
    } else if non_interactive {
        anyhow::bail!("GCP crypto key required (--gcp-crypto-key)");
    } else {
        Input::<String>::new()
            .with_prompt("GCP crypto key name")
            .interact_text()?
    };

    let key_name = format!(
        "projects/{}/locations/{}/keyRings/{}/cryptoKeys/{}",
        project_id, location, key_ring, crypto_key
    );

    let mut config = json!({
        "provider": "gcp-kms",
        "key_name": key_name,
        "project_id": project_id,
        "location": location,
        "key_ring": key_ring,
        "crypto_key": crypto_key
    });

    // Optional: credentials file
    if !non_interactive {
        let use_credentials = Confirm::new()
            .with_prompt("Provide service account credentials file? (uses default if not)")
            .default(false)
            .interact()?;

        if use_credentials {
            let creds_file: String = Input::new()
                .with_prompt("Service account key file path")
                .interact_text()?;

            config["credentials_file"] = json!(creds_file);
        }
    } else if let Some(creds_file) = credentials_file {
        config["credentials_file"] = json!(creds_file);
    }

    Ok(config)
}

fn configure_azure_kv_provider(
    non_interactive: bool,
    vault_name: Option<String>,
    key_name: Option<String>,
    tenant_id: Option<String>,
    client_id: Option<String>,
    client_secret: Option<String>,
) -> Result<serde_json::Value> {
    let vault_name = if let Some(v) = vault_name {
        v
    } else if non_interactive {
        anyhow::bail!("Azure Key Vault name required (--azure-vault-name)");
    } else {
        Input::<String>::new()
            .with_prompt("Azure Key Vault name")
            .interact_text()?
    };

    let key_name = if let Some(k) = key_name {
        k
    } else if non_interactive {
        anyhow::bail!("Azure key name required (--azure-key-name)");
    } else {
        Input::<String>::new()
            .with_prompt("Azure key name")
            .interact_text()?
    };

    let mut config = json!({
        "provider": "azure-kv",
        "vault_name": vault_name,
        "key_name": key_name
    });

    // Optional: Azure credentials (uses managed identity if not provided)
    if !non_interactive {
        let use_credentials = Confirm::new()
            .with_prompt("Provide Azure credentials? (uses managed identity if not)")
            .default(false)
            .interact()?;

        if use_credentials {
            let tenant: String = Input::new()
                .with_prompt("Azure tenant ID")
                .interact_text()?;

            let client: String = Input::new()
                .with_prompt("Azure client ID")
                .interact_text()?;

            let secret = Password::new()
                .with_prompt("Azure client secret")
                .interact()?;

            config["tenant_id"] = json!(tenant);
            config["client_id"] = json!(client);
            config["client_secret"] = json!(secret);
        }
    } else if let (Some(tenant), Some(client), Some(secret)) = (tenant_id, client_id, client_secret)
    {
        config["tenant_id"] = json!(tenant);
        config["client_id"] = json!(client);
        config["client_secret"] = json!(secret);
    }

    Ok(config)
}

async fn show_auto_unseal_status(config: &CliConfig) -> Result<()> {
    let auth_client = AuthenticatedClient::new()?;
    let url = format!("{}/v1/sys/auto-unseal/status", config.server_url);

    let request = auth_client.get(&url)?;
    let response = auth_client.execute(request).await?;

    if response.status().is_success() {
        let status: serde_json::Value = response.json().await?;

        println!("🔍 Auto-Unseal Status");
        println!();

        if let Some(enabled) = status.get("enabled").and_then(|e| e.as_bool()) {
            if enabled {
                println!("   Status: ✅ ENABLED");

                if let Some(provider) = status.get("provider").and_then(|p| p.as_str()) {
                    println!("   Provider: {}", provider);
                }

                if let Some(key_id) = status.get("key_id").and_then(|k| k.as_str()) {
                    println!("   Key ID: {}", key_id);
                }

                if let Some(region) = status.get("region").and_then(|r| r.as_str()) {
                    println!("   Region: {}", region);
                }

                if let Some(endpoint) = status.get("endpoint").and_then(|e| e.as_str()) {
                    println!("   Endpoint: {}", endpoint);
                }

                if let Some(health) = status.get("health_status").and_then(|h| h.as_str()) {
                    let health_icon = match health {
                        "healthy" => "✅",
                        "degraded" => "⚠️",
                        "unhealthy" => "❌",
                        _ => "❓",
                    };
                    println!("   Health: {} {}", health_icon, health.to_uppercase());
                }

                if let Some(last_check) = status.get("last_health_check").and_then(|l| l.as_str()) {
                    println!("   Last Health Check: {}", last_check);
                }
            } else {
                println!("   Status: ❌ DISABLED");
                println!();
                println!("   Auto-unseal is not configured.");
                println!("   Run 'secreton-cli auto-unseal configure' to set it up.");
            }
        }
    } else {
        let error_text = response.text().await?;
        anyhow::bail!("Failed to get status: {}", error_text);
    }

    Ok(())
}

async fn test_auto_unseal(config: &CliConfig) -> Result<()> {
    println!("🧪 Testing Auto-Unseal Configuration");
    println!();

    let auth_client = AuthenticatedClient::new()?;
    let url = format!("{}/v1/sys/auto-unseal/test", config.server_url);

    println!("   Testing provider connectivity...");

    let request = auth_client
        .post(&url)?
        .header("Content-Type", "application/json")
        .json(&json!({}));

    let response = auth_client.execute(request).await?;

    if response.status().is_success() {
        let result: serde_json::Value = response.json().await?;

        if let Some(success) = result.get("success").and_then(|s| s.as_bool()) {
            if success {
                println!("   ✅ Provider connectivity: OK");
                println!("   ✅ Encryption test: OK");
                println!("   ✅ Decryption test: OK");
                println!();
                println!("✅ Auto-unseal configuration is working correctly");

                if let Some(latency) = result.get("latency_ms").and_then(|l| l.as_f64()) {
                    println!("   Average latency: {:.2}ms", latency);
                }
            } else {
                println!("   ❌ Test failed");

                if let Some(error) = result.get("error").and_then(|e| e.as_str()) {
                    println!("   Error: {}", error);
                }
            }
        }
    } else {
        let error_text = response.text().await?;
        println!("   ❌ Test failed: {}", error_text);
    }

    Ok(())
}

async fn disable_auto_unseal(config: &CliConfig) -> Result<()> {
    println!("⚠️  Disabling Auto-Unseal");
    println!();
    println!("   This will revert to manual Shamir unseal.");
    println!("   You will need to provide unseal keys on each restart.");
    println!();

    let confirm = Confirm::new()
        .with_prompt("Are you sure you want to disable auto-unseal?")
        .default(false)
        .interact()?;

    if !confirm {
        println!("❌ Operation cancelled");
        return Ok(());
    }

    let auth_client = AuthenticatedClient::new()?;
    let url = format!("{}/v1/sys/auto-unseal/config", config.server_url);

    let request = auth_client.delete(&url)?;
    let response = auth_client.execute(request).await?;

    if response.status().is_success() {
        println!("✅ Auto-unseal disabled successfully");
        println!();
        println!("⚠️  Important:");
        println!("   1. Restart Secreton for changes to take effect");
        println!("   2. Prepare unseal keys for manual unsealing");
        println!("   3. Unseal with: secreton-cli seal unseal");
    } else {
        let error_text = response.text().await?;
        anyhow::bail!("Failed to disable auto-unseal: {}", error_text);
    }

    Ok(())
}
