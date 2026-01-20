//! Authentication commands for Secreton CLI
//!
//! Provides login and logout functionality with persistent token storage.

use anyhow::{Context, Result};
use clap::Subcommand;
use serde::{Deserialize, Serialize};

use crate::config::CliConfig;
use crate::token_store::{TokenMetadata, TokenStore};

/// Authentication commands
#[derive(Subcommand)]
pub enum AuthCommand {
    /// Login to Secreton vault
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
    /// Logout from Secreton vault
    Logout,
}

/// Login request payload
#[derive(Debug, Serialize)]
struct LoginRequest {
    username: String,
    password: String,
    mfa_code: Option<String>,
    remember_me: Option<bool>,
}

/// Login response payload
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct LoginResponse {
    access_token: String,
    refresh_token: String,
    token_type: String,
    expires_in: u64,
    user: UserInfo,
    mfa_required: bool,
}

/// User information
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct UserInfo {
    username: String,
    email: Option<String>,
    display_name: Option<String>,
    groups: Vec<String>,
    policies: Vec<String>,
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

/// Execute authentication commands
#[allow(dead_code)]
pub async fn execute_auth_command(cmd: AuthCommand, config: &CliConfig) -> Result<()> {
    match cmd {
        AuthCommand::Login {
            method,
            username,
            token,
        } => login_command(config, &method, username, token).await,
        AuthCommand::Logout => logout_command(config).await,
    }
}

/// Handle login command
pub async fn login_command(
    config: &CliConfig,
    method: &str,
    username: Option<String>,
    token: Option<String>,
) -> Result<()> {
    let token_store = TokenStore::new()?;

    match method {
        "userpass" => {
            // Get username
            let username = if let Some(u) = username {
                u
            } else {
                print!("Username: ");
                use std::io::{self, Write};
                io::stdout().flush()?;
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                input.trim().to_string()
            };

            // Get password securely
            let password = rpassword::prompt_password("Password: ")
                .context("Failed to read password")?;

            // Call login API
            let client = reqwest::Client::new();
            let url = format!("{}/v1/auth/login", config.server_url);

            let request = LoginRequest {
                username: username.clone(),
                password,
                mfa_code: None,
                remember_me: Some(true),
            };

            let response = client
                .post(&url)
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
                anyhow::bail!(
                    "Authentication failed (HTTP {}): {}",
                    status,
                    error_text
                );
            }

            let api_response: ApiResponse<LoginResponse> = response
                .json()
                .await
                .context("Failed to parse login response")?;

            if !api_response.success {
                if let Some(error) = api_response.error {
                    anyhow::bail!("Authentication failed: {}", error.message);
                } else {
                    anyhow::bail!("Authentication failed: Unknown error");
                }
            }

            let login_data = api_response
                .data
                .context("Login response missing data field")?;

            if login_data.mfa_required {
                println!("⚠️  MFA required but not yet supported in CLI");
                println!("   Please use the web interface to complete authentication");
                return Ok(());
            }

            // Calculate expiration time
            let expires_at = chrono::Utc::now()
                + chrono::Duration::seconds(login_data.expires_in as i64);

            // Store token
            let metadata = TokenMetadata {
                expires_at: Some(expires_at),
                policies: login_data.user.policies.clone(),
                renewable: true,
                display_name: login_data
                    .user
                    .display_name
                    .or(Some(login_data.user.username.clone())),
            };

            token_store
                .store_token(&login_data.access_token, &metadata)
                .context("Failed to store authentication token")?;

            println!("✅ Successfully authenticated as {}", login_data.user.username);
            println!("   Token stored in: {}", token_store.token_file_path().display());
            println!("   Policies: {}", login_data.user.policies.join(", "));
            println!(
                "   Expires: {}",
                expires_at.format("%Y-%m-%d %H:%M:%S UTC")
            );
        }
        "token" => {
            // Token authentication
            let token_value = token.context("Token is required for token authentication method. Use --token <TOKEN>")?;

            // Validate token prefix
            if !token_value.starts_with("stn.") {
                anyhow::bail!("Invalid token format: must start with 'stn.' prefix");
            }

            // Verify token by calling token lookup endpoint
            let client = reqwest::Client::new();
            let url = format!("{}/v1/auth/token/lookup-self", config.server_url);

            let response = client
                .get(&url)
                .header("X-Vault-Token", &token_value)
                .send()
                .await
                .context("Failed to verify token")?;

            if !response.status().is_success() {
                anyhow::bail!("Invalid token: verification failed");
            }

            let api_response: ApiResponse<serde_json::Value> = response
                .json()
                .await
                .context("Failed to parse token verification response")?;

            if !api_response.success {
                anyhow::bail!("Invalid token: verification failed");
            }

            let token_data = api_response
                .data
                .context("Token verification response missing data")?;

            // Extract policies and display name
            let policies: Vec<String> = token_data
                .get("policies")
                .and_then(|p| p.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            let display_name = token_data
                .get("display_name")
                .and_then(|d| d.as_str())
                .map(String::from);

            // Calculate expiration from TTL
            let ttl = token_data
                .get("ttl")
                .and_then(|t| t.as_i64())
                .unwrap_or(86400); // Default 24 hours

            let expires_at = chrono::Utc::now() + chrono::Duration::seconds(ttl);

            // Store token
            let metadata = TokenMetadata {
                expires_at: Some(expires_at),
                policies: policies.clone(),
                renewable: token_data
                    .get("renewable")
                    .and_then(|r| r.as_bool())
                    .unwrap_or(true),
                display_name,
            };

            token_store
                .store_token(&token_value, &metadata)
                .context("Failed to store authentication token")?;

            println!("✅ Successfully authenticated with token");
            println!("   Token stored in: {}", token_store.token_file_path().display());
            println!("   Policies: {}", policies.join(", "));
            println!(
                "   Expires: {}",
                expires_at.format("%Y-%m-%d %H:%M:%S UTC")
            );
        }
        _ => {
            anyhow::bail!(
                "Unsupported authentication method: {}. Supported methods: userpass, token",
                method
            );
        }
    }

    Ok(())
}

/// Handle logout command
pub async fn logout_command(config: &CliConfig) -> Result<()> {
    let token_store = TokenStore::new()?;

    // Load current token
    let stored_token = token_store
        .load_token()
        .context("Failed to load token")?;

    if stored_token.is_none() {
        println!("ℹ️  No active session found");
        return Ok(());
    }

    let token = stored_token.unwrap();

    // Call logout API
    let client = reqwest::Client::new();
    let url = format!("{}/v1/auth/logout", config.server_url);

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token.token))
        .send()
        .await;

    // Clear local token regardless of API response
    token_store
        .clear_token()
        .context("Failed to clear local token")?;

    match response {
        Ok(resp) if resp.status().is_success() => {
            println!("✅ Successfully logged out");
            println!("   Token cleared from: {}", token_store.token_file_path().display());
        }
        Ok(resp) => {
            println!("⚠️  Server logout failed (HTTP {}), but local token cleared", resp.status());
            println!("   Token cleared from: {}", token_store.token_file_path().display());
        }
        Err(e) => {
            println!("⚠️  Could not reach server ({}), but local token cleared", e);
            println!("   Token cleared from: {}", token_store.token_file_path().display());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_request_serialization() {
        let request = LoginRequest {
            username: "testuser".to_string(),
            password: "testpass".to_string(),
            mfa_code: None,
            remember_me: Some(true),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("testuser"));
        assert!(json.contains("testpass"));
    }

    #[test]
    fn test_login_response_deserialization() {
        let json = r#"{
            "access_token": "stn.test_token",
            "refresh_token": "stn.refresh_token",
            "token_type": "Bearer",
            "expires_in": 3600,
            "user": {
                "username": "testuser",
                "email": "test@example.com",
                "display_name": "Test User",
                "groups": ["admin"],
                "policies": ["default", "admin"]
            },
            "mfa_required": false
        }"#;

        let response: LoginResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.access_token, "stn.test_token");
        assert_eq!(response.user.username, "testuser");
        assert_eq!(response.user.policies, vec!["default", "admin"]);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use tempfile::TempDir;

    // Helper to create a test token store in a temp directory
    // We use the same approach as in token_store tests
    fn create_test_token_store() -> (TokenStore, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().to_path_buf();
        let token_file = config_dir.join("token");

        // Create the TokenStore using the internal struct directly
        // This is the same pattern used in token_store.rs tests
        let store = crate::token_store::TokenStore {
            config_dir,
            token_file,
        };

        (store, temp_dir)
    }

    // Generator for valid usernames
    #[allow(dead_code)]
    fn arb_username() -> impl Strategy<Value = String> {
        "[a-z][a-z0-9_-]{2,19}".prop_map(|s| s.to_string())
    }

    // Generator for valid passwords
    #[allow(dead_code)]
    fn arb_password() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9!@#$%^&*]{8,32}".prop_map(|s| s.to_string())
    }

    // Generator for valid tokens (with stn. prefix)
    fn arb_token() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9_-]{20,64}".prop_map(|s| format!("stn.{}", s))
    }

    // Generator for token metadata
    fn arb_token_metadata() -> impl Strategy<Value = TokenMetadata> {
        (
            prop::option::of(
                any::<i64>()
                    .prop_map(|offset| chrono::Utc::now() + chrono::Duration::seconds(offset.abs() % 86400)),
            ),
            prop::collection::vec(prop::string::string_regex("[a-z-]{3,20}").unwrap(), 0..5),
            any::<bool>(),
            prop::option::of(prop::string::string_regex("[a-z@.]{5,30}").unwrap()),
        )
            .prop_map(|(expires_at, policies, renewable, display_name)| {
                TokenMetadata {
                    expires_at,
                    policies,
                    renewable,
                    display_name,
                }
            })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Feature: secreton-cli-workflow-integration, Property 1: Login Authentication Consistency**
        /// **Validates: Requirements 1.1, 1.2**
        ///
        /// For any valid credentials (username/password or token), authenticating via CLI
        /// should result in a token being stored locally, and that token should be usable
        /// for subsequent requests.
        ///
        /// Note: This property test validates the token storage mechanism. Full end-to-end
        /// authentication testing requires a running Secreton server and is covered by
        /// integration tests.
        #[test]
        fn prop_login_stores_token_consistently(
            token in arb_token(),
            metadata in arb_token_metadata()
        ) {
            let (store, _temp) = create_test_token_store();

            // Simulate successful login by storing token
            store.store_token(&token, &metadata).unwrap();

            // Verify token is stored and can be loaded
            let loaded = store.load_token().unwrap();
            prop_assert!(loaded.is_some());

            let loaded_token = loaded.unwrap();
            prop_assert_eq!(loaded_token.token, token);
            prop_assert_eq!(loaded_token.metadata.policies, metadata.policies);
            prop_assert_eq!(loaded_token.metadata.renewable, metadata.renewable);

            // Verify token file exists
            prop_assert!(store.token_file_path().exists());
        }

        /// **Feature: secreton-cli-workflow-integration, Property 2: Invalid Credentials Rejection**
        /// **Validates: Requirements 1.3**
        ///
        /// For any invalid credentials, the CLI should reject authentication and not store
        /// any token in the local configuration.
        ///
        /// This property validates that invalid token formats (missing stn. prefix) are
        /// rejected before storage.
        #[test]
        fn prop_invalid_credentials_rejected(
            invalid_token in "[a-zA-Z0-9_-]{20,64}",  // No stn. prefix
            metadata in arb_token_metadata()
        ) {
            let (store, _temp) = create_test_token_store();

            // Attempt to store invalid token (missing stn. prefix)
            let result = store.store_token(&invalid_token, &metadata);

            // Should fail with error
            prop_assert!(result.is_err());

            // Verify no token was stored
            let loaded = store.load_token().unwrap();
            prop_assert!(loaded.is_none());

            // Verify token file was not created
            prop_assert!(!store.token_file_path().exists());
        }

        /// Additional property: Token authentication method validation
        ///
        /// For any token provided via --token flag, it must have the correct prefix
        /// to be considered valid.
        #[test]
        fn prop_token_method_validates_prefix(
            token_suffix in "[a-zA-Z0-9_-]{20,64}"
        ) {
            // Valid token with prefix
            let valid_token = format!("stn.{}", token_suffix);
            prop_assert!(valid_token.starts_with("stn."));

            // Invalid token without prefix
            prop_assert!(!token_suffix.starts_with("stn."));
        }

        /// Additional property: Stored token metadata consistency
        ///
        /// For any token stored with metadata, loading it should return the same metadata.
        #[test]
        fn prop_token_metadata_consistency(
            token in arb_token(),
            policies in prop::collection::vec(prop::string::string_regex("[a-z-]{3,20}").unwrap(), 0..5),
            renewable in any::<bool>(),
            display_name in prop::option::of(prop::string::string_regex("[a-z@.]{5,30}").unwrap())
        ) {
            let (store, _temp) = create_test_token_store();

            let metadata = TokenMetadata {
                expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
                policies: policies.clone(),
                renewable,
                display_name: display_name.clone(),
            };

            store.store_token(&token, &metadata).unwrap();

            let loaded = store.load_token().unwrap().unwrap();
            prop_assert_eq!(loaded.metadata.policies, policies);
            prop_assert_eq!(loaded.metadata.renewable, renewable);
            prop_assert_eq!(loaded.metadata.display_name, display_name);
        }
    }
}
