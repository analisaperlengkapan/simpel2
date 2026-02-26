//! Operator module for diagnostic and operational commands
//!
//! This module provides commands for engine operators to diagnose
//! connectivity, seal status, and authentication status.

use crate::config::CliConfig;
use crate::middleware::SealChecker;
use crate::token_store::TokenStore;
use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum OperatorCommand {
    /// Run comprehensive diagnostics on engine connectivity and status
    Diagnose,
}

/// Execute operator commands
pub async fn execute_operator_command(cmd: OperatorCommand, config: &CliConfig) -> Result<()> {
    match cmd {
        OperatorCommand::Diagnose => diagnose_command(config).await,
    }
}

/// Run comprehensive diagnostics
///
/// This command checks:
/// 1. Connectivity to the engine server
/// 2. Engine initialization status
/// 3. Engine seal status
/// 4. Authentication status (token validity)
async fn diagnose_command(config: &CliConfig) -> Result<()> {
    println!("🔍 Running Secreton Engine Diagnostics...\n");

    // 1. Check connectivity
    println!("1️⃣  Checking connectivity to engine server...");
    println!("   Server: {}", config.server_url);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?;

    let health_url = format!("{}/health", config.server_url);
    match client.get(&health_url).send().await {
        Ok(response) => {
            let status = response.status();
            if status.is_success() {
                println!("   ✅ Connectivity: OK");

                if let Ok(health) = response.json::<serde_json::Value>().await
                    && let Some(version) = health.get("version").and_then(|v| v.as_str())
                {
                    println!("   📦 Version: {}", version);
                }
            } else {
                println!(
                    "   ⚠️  Connectivity: Server responded with status {}",
                    status
                );
            }
        }
        Err(e) => {
            println!("   ❌ Connectivity: FAILED");
            println!("   Error: {}", e);
            println!("\n💡 Troubleshooting:");
            println!(
                "   - Verify the server URL is correct: {}",
                config.server_url
            );
            println!("   - Check if the Secreton server is running");
            println!("   - Check network connectivity and firewall rules");
            return Ok(());
        }
    }

    // 2. Check initialization and seal status
    println!("\n2️⃣  Checking engine initialization and seal status...");

    let mut seal_checker = SealChecker::new(config.server_url.clone(), None);

    match seal_checker.get_status().await {
        Ok(status) => {
            // Check initialization
            if status.initialized {
                println!("   ✅ Initialization: Engine is initialized");
            } else {
                println!("   ❌ Initialization: Engine is NOT initialized");
                println!("\n💡 Next steps:");
                println!("   Run: secreton seal init");
                return Ok(());
            }

            // Check seal status
            if status.state == "unsealed" {
                println!("   ✅ Seal Status: Unsealed (ready for operations)");
            } else {
                println!(
                    "   ⚠️  Seal Status: {} (progress: {}/{})",
                    status.state, status.progress, status.threshold
                );
                println!("\n💡 Next steps:");
                println!("   Run: secreton seal unseal");
                println!(
                    "   You need {} unseal key(s) to unseal the engine",
                    status.threshold
                );
                return Ok(());
            }

            println!("   📊 Seal Type: {}", status.seal_type);
            println!("   🔑 Total Shares: {}", status.total_shares);
            println!("   🎯 Threshold: {}", status.threshold);
        }
        Err(e) => {
            println!("   ❌ Status Check: FAILED");
            println!("   Error: {}", e);
            return Ok(());
        }
    }

    // 3. Check authentication status
    println!("\n3️⃣  Checking authentication status...");

    let token_store = TokenStore::new()?;

    match token_store.load_token() {
        Ok(Some(stored_token)) => {
            println!("   ✅ Authentication: Token found");

            // Display token info
            if let Some(display_name) = &stored_token.metadata.display_name {
                println!("   👤 User: {}", display_name);
            }

            if !stored_token.metadata.policies.is_empty() {
                println!(
                    "   🔐 Policies: {}",
                    stored_token.metadata.policies.join(", ")
                );
            }

            if let Some(expires_at) = stored_token.metadata.expires_at {
                let now = chrono::Utc::now();
                if now >= expires_at {
                    println!("   ⚠️  Token Status: EXPIRED");
                    println!(
                        "   Expired at: {}",
                        expires_at.format("%Y-%m-%d %H:%M:%S UTC")
                    );
                    println!("\n💡 Next steps:");
                    println!("   Run: secreton login");
                } else {
                    let duration = expires_at - now;
                    let hours = duration.num_hours();
                    let minutes = duration.num_minutes() % 60;

                    println!("   ✅ Token Status: Valid");
                    println!("   ⏰ Expires in: {}h {}m", hours, minutes);
                    println!(
                        "   Expires at: {}",
                        expires_at.format("%Y-%m-%d %H:%M:%S UTC")
                    );
                }
            } else {
                println!("   ✅ Token Status: Valid (no expiration)");
            }

            if stored_token.metadata.renewable {
                println!("   🔄 Renewable: Yes");
            }

            // Verify token with server
            println!("\n   Verifying token with server...");
            let lookup_url = format!("{}/v1/auth/token/lookup-self", config.server_url);
            match client
                .get(&lookup_url)
                .header("X-Secreton-Token", &stored_token.token)
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        println!("   ✅ Token Verification: Valid with server");
                    } else if response.status().as_u16() == 403 {
                        println!("   ❌ Token Verification: Token is invalid or revoked");
                        println!("\n💡 Next steps:");
                        println!("   Run: secreton login");
                    } else {
                        println!(
                            "   ⚠️  Token Verification: Server returned status {}",
                            response.status()
                        );
                    }
                }
                Err(e) => {
                    println!("   ⚠️  Token Verification: Failed to verify with server");
                    println!("   Error: {}", e);
                }
            }
        }
        Ok(None) => {
            println!("   ⚠️  Authentication: No token found");
            println!("\n💡 Next steps:");
            println!("   Run: secreton login");
        }
        Err(e) => {
            println!("   ❌ Authentication Check: FAILED");
            println!("   Error: {}", e);
        }
    }

    // Summary
    println!("\n{}", "=".repeat(60));
    println!("📋 Diagnostic Summary");
    println!("{}", "=".repeat(60));

    let connectivity_ok = true; // If we got here, connectivity is OK
    let engine_ready = seal_checker.check_ready().await.is_ok();
    let authenticated = token_store.load_token().is_ok() && token_store.load_token()?.is_some();

    if connectivity_ok && engine_ready && authenticated {
        println!("✅ All systems operational - engine is ready for use");
    } else {
        println!("⚠️  Some issues detected - see details above");

        if !engine_ready {
            println!("   • Engine needs to be initialized and/or unsealed");
        }
        if !authenticated {
            println!("   • Authentication required (run 'secreton login')");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operator_command_enum() {
        // Just verify the enum compiles and can be created
        let _cmd = OperatorCommand::Diagnose;
    }
}
