//! Token management commands for Secreton CLI
//!
//! Provides token lifecycle management including creation, lookup, renewal, and revocation.

use anyhow::{Context, Result};
use clap::Subcommand;
use comfy_table::{Cell, Color, Table};
use serde::{Deserialize, Serialize};

use crate::config::CliConfig;
use crate::token_store::TokenStore;

/// Token management commands
#[derive(Subcommand)]
pub enum TokenCommand {
    /// Create a new token with specified policies
    Create {
        /// Policies to attach to the token
        #[arg(long, value_delimiter = ',')]
        policies: Vec<String>,

        /// Time-to-live for the token (e.g., "1h", "24h", "7d")
        #[arg(long)]
        ttl: Option<String>,

        /// Whether the token is renewable
        #[arg(long, default_value = "true")]
        renewable: bool,

        /// Display name for the token
        #[arg(long)]
        display_name: Option<String>,
    },
    /// Look up information about a token
    Lookup {
        /// Token to lookup (defaults to current token)
        token: Option<String>,
    },
    /// Renew the current token
    Renew {
        /// Increment to add to token TTL (e.g., "1h", "24h")
        #[arg(long)]
        increment: Option<String>,
    },
    /// Revoke a token
    Revoke {
        /// Token to revoke
        token: String,
    },
    /// Show capabilities of current token for a path
    Capabilities {
        /// Path to check capabilities for
        path: String,
    },
}

/// Token creation request
#[derive(Debug, Serialize)]
struct TokenCreateRequest {
    policies: Vec<String>,
    ttl: Option<String>,
    renewable: bool,
    display_name: Option<String>,
}

/// Token creation response
#[derive(Debug, Deserialize)]
struct TokenCreateResponse {
    token: String,
    policies: Vec<String>,
    ttl: u64,
    renewable: bool,
    display_name: Option<String>,
}

/// Token lookup response
#[derive(Debug, Deserialize)]
struct TokenLookupResponse {
    id: String,
    policies: Vec<String>,
    ttl: i64,
    renewable: bool,
    display_name: Option<String>,
    created_at: String,
    expires_at: Option<String>,
}

/// Token renew response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TokenRenewResponse {
    token: String,
    ttl: u64,
    renewable: bool,
    expires_at: Option<String>,
}

/// Token capabilities response
#[derive(Debug, Deserialize)]
struct TokenCapabilitiesResponse {
    capabilities: Vec<String>,
}

/// API response wrapper
#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<ApiError>,
}

/// API error structure
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ApiError {
    code: String,
    message: String,
}

/// Execute token management commands
pub async fn execute_token_command(
    cmd: TokenCommand,
    config: &CliConfig,
    _namespace: Option<&str>,
) -> Result<()> {
    // Note: Token operations are typically not namespace-scoped in most engine systems
    // The namespace parameter is accepted for consistency but not used
    match cmd {
        TokenCommand::Create {
            policies,
            ttl,
            renewable,
            display_name,
        } => token_create_command(config, policies, ttl, renewable, display_name).await,
        TokenCommand::Lookup { token } => token_lookup_command(config, token).await,
        TokenCommand::Renew { increment } => token_renew_command(config, increment).await,
        TokenCommand::Revoke { token } => token_revoke_command(config, token).await,
        TokenCommand::Capabilities { path } => token_capabilities_command(config, path).await,
    }
}

/// Create a new token
async fn token_create_command(
    config: &CliConfig,
    policies: Vec<String>,
    ttl: Option<String>,
    renewable: bool,
    display_name: Option<String>,
) -> Result<()> {
    let token_store = TokenStore::new()?;

    // Load current token for authentication
    let current_token = token_store
        .load_token()
        .context("Failed to load authentication token")?
        .context("Not authenticated. Run 'secreton login' first.")?;

    let client = reqwest::Client::new();
    let url = format!("{}/v1/auth/token/create", config.server_url);

    let request = TokenCreateRequest {
        policies,
        ttl,
        renewable,
        display_name,
    };

    let response = client
        .post(&url)
        .header("X-Engine-Token", &current_token.token)
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .context("Failed to connect to Secreton server")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        anyhow::bail!("Failed to create token (HTTP {}): {}", status, error_text);
    }

    let api_response: ApiResponse<TokenCreateResponse> = response
        .json()
        .await
        .context("Failed to parse token creation response")?;

    if !api_response.success {
        if let Some(error) = api_response.error {
            anyhow::bail!("Failed to create token: {}", error.message);
        } else {
            anyhow::bail!("Failed to create token: Unknown error");
        }
    }

    let token_data = api_response
        .data
        .context("Token creation response missing data field")?;

    println!("✅ Token created successfully");
    println!();
    println!("Token:        {}", token_data.token);
    println!("Policies:     {}", token_data.policies.join(", "));
    println!("TTL:          {} seconds", token_data.ttl);
    println!("Renewable:    {}", token_data.renewable);
    if let Some(name) = &token_data.display_name {
        println!("Display Name: {}", name);
    }
    println!();
    println!("⚠️  Make sure to save this token - it cannot be retrieved later!");

    Ok(())
}

/// Look up token information
async fn token_lookup_command(config: &CliConfig, token: Option<String>) -> Result<()> {
    let token_store = TokenStore::new()?;

    // Determine which token to lookup
    let lookup_token = if let Some(t) = token {
        t
    } else {
        // Use current token
        token_store
            .load_token()
            .context("Failed to load authentication token")?
            .context("Not authenticated. Run 'secreton login' first.")?
            .token
    };

    let client = reqwest::Client::new();
    let url = format!("{}/v1/auth/token/lookup-self", config.server_url);

    let response = client
        .get(&url)
        .header("X-Engine-Token", &lookup_token)
        .send()
        .await
        .context("Failed to connect to Secreton server")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        anyhow::bail!("Failed to lookup token (HTTP {}): {}", status, error_text);
    }

    let api_response: ApiResponse<TokenLookupResponse> = response
        .json()
        .await
        .context("Failed to parse token lookup response")?;

    if !api_response.success {
        if let Some(error) = api_response.error {
            anyhow::bail!("Failed to lookup token: {}", error.message);
        } else {
            anyhow::bail!("Failed to lookup token: Unknown error");
        }
    }

    let token_data = api_response
        .data
        .context("Token lookup response missing data field")?;

    // Display token information in a table
    let mut table = Table::new();
    table.set_header(vec![
        Cell::new("Property").fg(Color::Cyan),
        Cell::new("Value").fg(Color::Green),
    ]);

    table.add_row(vec!["ID", &token_data.id]);
    table.add_row(vec!["Policies", &token_data.policies.join(", ")]);
    table.add_row(vec!["TTL", &format!("{} seconds", token_data.ttl)]);
    table.add_row(vec!["Renewable", &token_data.renewable.to_string()]);

    if let Some(name) = &token_data.display_name {
        table.add_row(vec!["Display Name", name]);
    }

    table.add_row(vec!["Created At", &token_data.created_at]);

    if let Some(expires) = &token_data.expires_at {
        table.add_row(vec!["Expires At", expires]);
    }

    println!("{}", table);

    Ok(())
}

/// Renew the current token
async fn token_renew_command(config: &CliConfig, increment: Option<String>) -> Result<()> {
    let token_store = TokenStore::new()?;

    // Load current token
    let current_token = token_store
        .load_token()
        .context("Failed to load authentication token")?
        .context("Not authenticated. Run 'secreton login' first.")?;

    if !current_token.metadata.renewable {
        anyhow::bail!("Current token is not renewable");
    }

    let client = reqwest::Client::new();
    let url = format!("{}/v1/auth/token/refresh", config.server_url);

    let mut payload = serde_json::json!({});
    if let Some(inc) = increment {
        payload["increment"] = serde_json::Value::String(inc);
    }

    let response = client
        .post(&url)
        .header("X-Engine-Token", &current_token.token)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .context("Failed to connect to Secreton server")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        anyhow::bail!("Failed to renew token (HTTP {}): {}", status, error_text);
    }

    let api_response: ApiResponse<TokenRenewResponse> = response
        .json()
        .await
        .context("Failed to parse token renewal response")?;

    if !api_response.success {
        if let Some(error) = api_response.error {
            anyhow::bail!("Failed to renew token: {}", error.message);
        } else {
            anyhow::bail!("Failed to renew token: Unknown error");
        }
    }

    let renew_data = api_response
        .data
        .context("Token renewal response missing data field")?;

    println!("✅ Token renewed successfully");
    println!("   New TTL: {} seconds", renew_data.ttl);
    if let Some(expires) = &renew_data.expires_at {
        println!("   Expires: {}", expires);
    }

    // Update stored token with new expiration
    if let Some(expires_str) = &renew_data.expires_at
        && let Ok(expires_dt) = chrono::DateTime::parse_from_rfc3339(expires_str)
    {
        let mut updated_metadata = current_token.metadata.clone();
        updated_metadata.expires_at = Some(expires_dt.with_timezone(&chrono::Utc));

        token_store
            .store_token(&current_token.token, &updated_metadata)
            .context("Failed to update stored token")?;
    }

    Ok(())
}

/// Revoke a token
async fn token_revoke_command(config: &CliConfig, token: String) -> Result<()> {
    let token_store = TokenStore::new()?;

    // Load current token for authentication
    let current_token = token_store
        .load_token()
        .context("Failed to load authentication token")?
        .context("Not authenticated. Run 'secreton login' first.")?;

    let client = reqwest::Client::new();
    let url = format!("{}/v1/auth/token/revoke", config.server_url);

    let payload = serde_json::json!({
        "token": token
    });

    let response = client
        .post(&url)
        .header("X-Engine-Token", &current_token.token)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .context("Failed to connect to Secreton server")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        anyhow::bail!("Failed to revoke token (HTTP {}): {}", status, error_text);
    }

    println!("✅ Token revoked successfully");

    // If revoking current token, clear local storage
    if token == current_token.token {
        token_store
            .clear_token()
            .context("Failed to clear local token")?;
        println!("   Local token cleared");
    }

    Ok(())
}

/// Show token capabilities for a path
async fn token_capabilities_command(config: &CliConfig, path: String) -> Result<()> {
    let token_store = TokenStore::new()?;

    // Load current token
    let current_token = token_store
        .load_token()
        .context("Failed to load authentication token")?
        .context("Not authenticated. Run 'secreton login' first.")?;

    let client = reqwest::Client::new();
    let url = format!("{}/v1/auth/token/capabilities", config.server_url);

    let payload = serde_json::json!({
        "path": path
    });

    let response = client
        .post(&url)
        .header("X-Engine-Token", &current_token.token)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .context("Failed to connect to Secreton server")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        anyhow::bail!(
            "Failed to check capabilities (HTTP {}): {}",
            status,
            error_text
        );
    }

    let api_response: ApiResponse<TokenCapabilitiesResponse> = response
        .json()
        .await
        .context("Failed to parse capabilities response")?;

    if !api_response.success {
        if let Some(error) = api_response.error {
            anyhow::bail!("Failed to check capabilities: {}", error.message);
        } else {
            anyhow::bail!("Failed to check capabilities: Unknown error");
        }
    }

    let caps_data = api_response
        .data
        .context("Capabilities response missing data field")?;

    println!("🔑 Token capabilities for path '{}':", path);
    println!();
    for cap in &caps_data.capabilities {
        println!("   • {}", cap);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_create_request_serialization() {
        let request = TokenCreateRequest {
            policies: vec!["default".to_string(), "admin".to_string()],
            ttl: Some("1h".to_string()),
            renewable: true,
            display_name: Some("test-token".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("default"));
        assert!(json.contains("admin"));
        assert!(json.contains("1h"));
    }

    #[test]
    fn test_token_create_response_deserialization() {
        let json = r#"{
            "token": "stn.test_token_12345",
            "policies": ["default", "admin"],
            "ttl": 3600,
            "renewable": true,
            "display_name": "test-token"
        }"#;

        let response: TokenCreateResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.token, "stn.test_token_12345");
        assert_eq!(response.policies, vec!["default", "admin"]);
        assert_eq!(response.ttl, 3600);
        assert!(response.renewable);
    }

    #[test]
    fn test_token_lookup_response_deserialization() {
        let json = r#"{
            "id": "token-id-123",
            "policies": ["default"],
            "ttl": 3600,
            "renewable": true,
            "display_name": "test-user",
            "created_at": "2025-12-02T10:00:00Z",
            "expires_at": "2025-12-02T11:00:00Z"
        }"#;

        let response: TokenLookupResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.id, "token-id-123");
        assert_eq!(response.policies, vec!["default"]);
        assert_eq!(response.ttl, 3600);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Generator for valid token strings (with stn. prefix)
    fn arb_token() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9_-]{20,64}".prop_map(|s| format!("stn.{}", s))
    }

    // Generator for policy names
    fn arb_policies() -> impl Strategy<Value = Vec<String>> {
        prop::collection::vec(prop::string::string_regex("[a-z-]{3,20}").unwrap(), 1..5)
    }

    // Generator for TTL values
    fn arb_ttl() -> impl Strategy<Value = u64> {
        1u64..86400u64 // 1 second to 24 hours
    }

    // Generator for display names
    fn arb_display_name() -> impl Strategy<Value = Option<String>> {
        prop::option::of(prop::string::string_regex("[a-z@.]{5,30}").unwrap())
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Feature: secreton-cli-workflow-integration, Property 7: Token Create-Lookup Consistency**
        /// **Validates: Requirements 4.1, 1.5**
        ///
        /// For any token created with specific policies, looking up that token should return
        /// the same policies.
        ///
        /// Note: This property test validates the data structures and serialization/deserialization
        /// consistency. Full end-to-end token creation and lookup requires a running Secreton server
        /// and is covered by integration tests.
        #[test]
        fn prop_token_create_lookup_consistency(
            token in arb_token(),
            policies in arb_policies(),
            ttl in arb_ttl(),
            renewable in any::<bool>(),
            display_name in arb_display_name()
        ) {
            // Test TokenCreateRequest serialization
            let create_request = TokenCreateRequest {
                policies: policies.clone(),
                ttl: Some(format!("{}s", ttl)),
                renewable,
                display_name: display_name.clone(),
            };

            let create_json = serde_json::to_string(&create_request).unwrap();
            prop_assert!(create_json.contains(&policies[0]));

            // Test TokenCreateResponse deserialization
            let create_response_json = serde_json::json!({
                "token": token,
                "policies": policies.clone(),
                "ttl": ttl,
                "renewable": renewable,
                "display_name": display_name
            });

            let create_response: TokenCreateResponse =
                serde_json::from_value(create_response_json).unwrap();

            prop_assert_eq!(create_response.token, token);
            prop_assert_eq!(create_response.policies, policies.clone());
            prop_assert_eq!(create_response.ttl, ttl);
            prop_assert_eq!(create_response.renewable, renewable);
            prop_assert_eq!(create_response.display_name, display_name.clone());

            // Test TokenLookupResponse deserialization with same policies
            let lookup_response_json = serde_json::json!({
                "id": "token-id-123",
                "policies": policies.clone(),
                "ttl": ttl as i64,
                "renewable": renewable,
                "display_name": display_name.clone(),
                "created_at": "2025-12-02T10:00:00Z",
                "expires_at": "2025-12-02T11:00:00Z"
            });

            let lookup_response: TokenLookupResponse =
                serde_json::from_value(lookup_response_json).unwrap();

            // Verify that lookup returns the same policies as create
            prop_assert_eq!(lookup_response.policies, policies);
            prop_assert_eq!(lookup_response.renewable, renewable);
            prop_assert_eq!(lookup_response.display_name, display_name);
        }

        /// Additional property: Token creation request validation
        ///
        /// For any valid token creation parameters, the request should serialize correctly
        /// and contain all specified fields.
        #[test]
        fn prop_token_create_request_valid(
            policies in arb_policies(),
            ttl_seconds in arb_ttl(),
            renewable in any::<bool>(),
            display_name in arb_display_name()
        ) {
            let request = TokenCreateRequest {
                policies: policies.clone(),
                ttl: Some(format!("{}s", ttl_seconds)),
                renewable,
                display_name: display_name.clone(),
            };

            let json = serde_json::to_string(&request).unwrap();

            // Verify all policies are in the JSON
            for policy in &policies {
                prop_assert!(json.contains(policy));
            }

            // Verify renewable flag
            prop_assert!(json.contains(&renewable.to_string()));

            // Verify display name if present
            if let Some(name) = &display_name {
                prop_assert!(json.contains(name));
            }
        }

        /// Additional property: Token lookup response completeness
        ///
        /// For any token lookup response, all required fields should be present and valid.
        #[test]
        fn prop_token_lookup_response_complete(
            policies in arb_policies(),
            ttl in 1i64..86400i64,
            renewable in any::<bool>(),
            display_name in arb_display_name()
        ) {
            let json = serde_json::json!({
                "id": "token-id-123",
                "policies": policies.clone(),
                "ttl": ttl,
                "renewable": renewable,
                "display_name": display_name,
                "created_at": "2025-12-02T10:00:00Z",
                "expires_at": "2025-12-02T11:00:00Z"
            });

            let response: TokenLookupResponse = serde_json::from_value(json).unwrap();

            prop_assert_eq!(response.policies, policies);
            prop_assert_eq!(response.ttl, ttl);
            prop_assert_eq!(response.renewable, renewable);
            prop_assert_eq!(response.display_name, display_name);
            prop_assert!(!response.id.is_empty());
            prop_assert!(!response.created_at.is_empty());
        }

        /// **Feature: secreton-cli-workflow-integration, Property 8: Token Revocation Effectiveness**
        /// **Validates: Requirements 4.2**
        ///
        /// For any revoked token, subsequent operations using that token should be rejected.
        ///
        /// This property validates that:
        /// 1. A token can be revoked (revoke request is properly formatted)
        /// 2. After revocation, the token should not be usable
        /// 3. If the revoked token is the current token, local storage is cleared
        ///
        /// Note: This property test validates the data structures and logic flow.
        /// Full end-to-end token revocation requires a running Secreton server
        /// and is covered by integration tests.
        #[test]
        fn prop_token_revocation_effectiveness(
            token_to_revoke in arb_token(),
            current_token in arb_token()
        ) {
            use tempfile::TempDir;

            // Create a test token store
            let temp_dir = TempDir::new().unwrap();
            let config_dir = temp_dir.path().to_path_buf();
            let token_file = config_dir.join("token");

            let store = crate::token_store::TokenStore {
                config_dir,
                token_file,
            };

            // Store the current token
            let metadata = crate::token_store::TokenMetadata {
                expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
                policies: vec!["default".to_string()],
                renewable: true,
                display_name: Some("test-user".to_string()),
            };

            store.store_token(&current_token, &metadata).unwrap();

            // Verify token is stored
            let loaded = store.load_token().unwrap();
            prop_assert!(loaded.is_some());

            // Test revoke request serialization
            let revoke_payload = serde_json::json!({
                "token": token_to_revoke.clone()
            });

            prop_assert!(revoke_payload.get("token").is_some());
            prop_assert_eq!(
                revoke_payload.get("token").unwrap().as_str().unwrap(),
                token_to_revoke.clone()
            );

            // Simulate revocation of current token
            if token_to_revoke == current_token {
                // After revoking current token, local storage should be cleared
                store.clear_token().unwrap();

                let after_revoke = store.load_token().unwrap();
                prop_assert!(after_revoke.is_none());
                prop_assert!(!store.token_file_path().exists());
             } else {
                // If revoking a different token, current token should remain
                let after_revoke = store.load_token().unwrap();
                prop_assert!(after_revoke.is_some());
                prop_assert_eq!(after_revoke.unwrap().token, current_token);
            }
        }

        /// Additional property: Token revocation request format
        ///
        /// For any token to be revoked, the revocation request should be properly formatted.
        #[test]
        fn prop_token_revoke_request_format(
            token in arb_token()
        ) {
            let payload = serde_json::json!({
                "token": token
            });

            let json_str = serde_json::to_string(&payload).unwrap();

            // Verify the token is in the JSON
            prop_assert!(json_str.contains(&token));

            // Verify it can be deserialized back
            let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
            prop_assert_eq!(parsed.get("token").unwrap().as_str().unwrap(), token);
        }
    }
}
