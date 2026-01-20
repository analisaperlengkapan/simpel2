//! Token storage module for persistent authentication tokens
//!
//! This module provides secure storage for Secreton authentication tokens
//! in the user's home directory (~/.secreton/token).

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Metadata about a stored token
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TokenMetadata {
    pub expires_at: Option<DateTime<Utc>>,
    pub policies: Vec<String>,
    pub renewable: bool,
    pub display_name: Option<String>,
}

/// A stored authentication token with metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StoredToken {
    pub token: String,
    #[serde(flatten)]
    pub metadata: TokenMetadata,
}

/// Manages persistent storage of authentication tokens
pub struct TokenStore {
    pub(crate) config_dir: PathBuf,
    pub(crate) token_file: PathBuf,
}

impl TokenStore {
    /// Create a new TokenStore instance
    ///
    /// This will use ~/.secreton/ as the config directory
    pub fn new() -> Result<Self> {
        let home_dir = dirs::home_dir().context("Failed to determine home directory")?;

        let config_dir = home_dir.join(".secreton");
        let token_file = config_dir.join("token");

        Ok(Self {
            config_dir,
            token_file,
        })
    }

    /// Store a token with its metadata
    ///
    /// The token file will be created with 0600 permissions for security.
    /// Tokens must have the 'stn.' prefix.
    pub fn store_token(&self, token: &str, metadata: &TokenMetadata) -> Result<()> {
        // Validate token prefix
        if !token.starts_with("stn.") {
            anyhow::bail!("Invalid token format: must start with 'stn.' prefix");
        }

        // Ensure config directory exists
        if !self.config_dir.exists() {
            fs::create_dir_all(&self.config_dir).context("Failed to create config directory")?;
        }

        let stored_token = StoredToken {
            token: token.to_string(),
            metadata: metadata.clone(),
        };

        // Serialize to TOML
        let toml_content =
            toml::to_string_pretty(&stored_token).context("Failed to serialize token")?;

        // Write to file
        fs::write(&self.token_file, toml_content).context("Failed to write token file")?;

        // Set file permissions to 0600 (owner read/write only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = fs::Permissions::from_mode(0o600);
            fs::set_permissions(&self.token_file, permissions)
                .context("Failed to set token file permissions")?;
        }

        Ok(())
    }

    /// Load the stored token
    ///
    /// Returns None if no token is stored or if the file doesn't exist.
    pub fn load_token(&self) -> Result<Option<StoredToken>> {
        if !self.token_file.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&self.token_file).context("Failed to read token file")?;

        let stored_token: StoredToken =
            toml::from_str(&content).context("Failed to parse token file")?;

        Ok(Some(stored_token))
    }

    /// Clear the stored token
    ///
    /// This removes the token file from disk.
    pub fn clear_token(&self) -> Result<()> {
        if self.token_file.exists() {
            fs::remove_file(&self.token_file).context("Failed to remove token file")?;
        }
        Ok(())
    }

    /// Check if the stored token is expired
    ///
    /// Returns true if the token is expired or if no token is stored.
    /// Returns false if the token has no expiration or is still valid.
    #[allow(dead_code)]
    pub fn is_token_expired(&self) -> bool {
        match self.load_token() {
            Ok(Some(stored_token)) => {
                if let Some(expires_at) = stored_token.metadata.expires_at {
                    Utc::now() >= expires_at
                } else {
                    // No expiration means token doesn't expire
                    false
                }
            }
            _ => true, // No token or error loading = consider expired
        }
    }

    /// Get the path to the token file
    pub fn token_file_path(&self) -> &PathBuf {
        &self.token_file
    }
}

impl Default for TokenStore {
    fn default() -> Self {
        Self::new().expect("Failed to create TokenStore")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use tempfile::TempDir;

    fn create_test_token_store() -> (TokenStore, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().to_path_buf();
        let token_file = config_dir.join("token");

        let store = TokenStore {
            config_dir,
            token_file,
        };

        (store, temp_dir)
    }

    #[test]
    fn test_store_and_load_token() {
        let (store, _temp) = create_test_token_store();

        let token = "stn.test_token_12345";
        let metadata = TokenMetadata {
            expires_at: Some(Utc::now() + Duration::hours(1)),
            policies: vec!["default".to_string(), "admin".to_string()],
            renewable: true,
            display_name: Some("test@example.com".to_string()),
        };

        store.store_token(token, &metadata).unwrap();

        let loaded = store.load_token().unwrap().unwrap();
        assert_eq!(loaded.token, token);
        assert_eq!(loaded.metadata.policies, metadata.policies);
        assert_eq!(loaded.metadata.renewable, metadata.renewable);
        assert_eq!(loaded.metadata.display_name, metadata.display_name);
    }

    #[test]
    fn test_load_nonexistent_token() {
        let (store, _temp) = create_test_token_store();

        let loaded = store.load_token().unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn test_clear_token() {
        let (store, _temp) = create_test_token_store();

        let token = "stn.test_token_12345";
        let metadata = TokenMetadata {
            expires_at: None,
            policies: vec![],
            renewable: false,
            display_name: None,
        };

        store.store_token(token, &metadata).unwrap();
        assert!(store.token_file.exists());

        store.clear_token().unwrap();
        assert!(!store.token_file.exists());
    }

    #[test]
    fn test_is_token_expired() {
        let (store, _temp) = create_test_token_store();

        // No token stored - should be considered expired
        assert!(store.is_token_expired());

        // Store expired token
        let token = "stn.expired_token";
        let metadata = TokenMetadata {
            expires_at: Some(Utc::now() - Duration::hours(1)),
            policies: vec![],
            renewable: false,
            display_name: None,
        };
        store.store_token(token, &metadata).unwrap();
        assert!(store.is_token_expired());

        // Store valid token
        let token = "stn.valid_token";
        let metadata = TokenMetadata {
            expires_at: Some(Utc::now() + Duration::hours(1)),
            policies: vec![],
            renewable: false,
            display_name: None,
        };
        store.store_token(token, &metadata).unwrap();
        assert!(!store.is_token_expired());

        // Store token with no expiration
        let token = "stn.no_expiry_token";
        let metadata = TokenMetadata {
            expires_at: None,
            policies: vec![],
            renewable: false,
            display_name: None,
        };
        store.store_token(token, &metadata).unwrap();
        assert!(!store.is_token_expired());
    }

    #[test]
    fn test_invalid_token_prefix() {
        let (store, _temp) = create_test_token_store();

        let token = "invalid_prefix_token";
        let metadata = TokenMetadata {
            expires_at: None,
            policies: vec![],
            renewable: false,
            display_name: None,
        };

        let result = store.store_token(token, &metadata);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("stn."));
    }

    #[test]
    #[cfg(unix)]
    fn test_token_file_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let (store, _temp) = create_test_token_store();

        let token = "stn.test_token";
        let metadata = TokenMetadata {
            expires_at: None,
            policies: vec![],
            renewable: false,
            display_name: None,
        };

        store.store_token(token, &metadata).unwrap();

        let metadata = fs::metadata(&store.token_file).unwrap();
        let permissions = metadata.permissions();
        assert_eq!(permissions.mode() & 0o777, 0o600);
    }

    // Property-based tests
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        // Generator for valid token strings (with stn. prefix)
        fn arb_token() -> impl Strategy<Value = String> {
            "[a-zA-Z0-9_-]{20,64}".prop_map(|s| format!("stn.{}", s))
        }

        // Generator for token metadata
        fn arb_token_metadata() -> impl Strategy<Value = TokenMetadata> {
            (
                prop::option::of(
                    any::<i64>()
                        .prop_map(|offset| Utc::now() + Duration::seconds(offset.abs() % 86400)),
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

            /// **Feature: secreton-cli-workflow-integration, Property 3: Logout Token Removal**
            /// **Validates: Requirements 1.4**
            ///
            /// For any authenticated session (stored token), executing logout (clear_token)
            /// should remove the stored token, and subsequent operations (load_token) should
            /// require re-authentication (return None).
            #[test]
            fn prop_logout_removes_token(
                token in arb_token(),
                metadata in arb_token_metadata()
            ) {
                let (store, _temp) = create_test_token_store();

                // Store a token (simulating login)
                store.store_token(&token, &metadata).unwrap();

                // Verify token is stored
                let loaded = store.load_token().unwrap();
                prop_assert!(loaded.is_some());
                prop_assert_eq!(loaded.unwrap().token, token);

                // Execute logout (clear token)
                store.clear_token().unwrap();

                // Verify token is removed and subsequent load returns None
                let after_logout = store.load_token().unwrap();
                prop_assert!(after_logout.is_none());

                // Verify token file no longer exists
                prop_assert!(!store.token_file.exists());
            }

            /// Additional property: Token round-trip consistency
            ///
            /// For any valid token and metadata, storing and then loading should
            /// return the same token and metadata.
            #[test]
            fn prop_token_roundtrip(
                token in arb_token(),
                metadata in arb_token_metadata()
            ) {
                let (store, _temp) = create_test_token_store();

                store.store_token(&token, &metadata).unwrap();

                let loaded = store.load_token().unwrap().unwrap();
                prop_assert_eq!(loaded.token, token);
                prop_assert_eq!(loaded.metadata.policies, metadata.policies);
                prop_assert_eq!(loaded.metadata.renewable, metadata.renewable);
                prop_assert_eq!(loaded.metadata.display_name, metadata.display_name);

                // Check expiration time (allow small time difference due to serialization)
                if let (Some(expected), Some(actual)) = (metadata.expires_at, loaded.metadata.expires_at) {
                    let diff = (expected - actual).num_seconds().abs();
                    prop_assert!(diff < 2, "Expiration time difference too large: {} seconds", diff);
                }
            }
        }
    }
}
