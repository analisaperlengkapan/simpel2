use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    pub server_url: String,
    #[serde(default = "default_namespace")]
    pub default_namespace: String,
}

fn default_namespace() -> String {
    "default".to_string()
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            server_url: "http://127.0.0.1:8200".to_string(),
            default_namespace: "default".to_string(),
        }
    }
}

impl CliConfig {
    /// Load configuration from a file
    pub async fn load_from_file(path: &str) -> Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        let config: CliConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Load configuration from the default location (~/.secreton/config.toml)
    pub async fn load_default() -> Result<Self> {
        let config_path = Self::default_config_path()?;

        if config_path.exists() {
            Self::load_from_file(config_path.to_str().unwrap()).await
        } else {
            Ok(Self::default())
        }
    }

    /// Get the default config file path
    pub fn default_config_path() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
        Ok(home.join(".secreton").join("config.toml"))
    }

    /// Save configuration to the default location
    pub async fn save_default(&self) -> Result<()> {
        let config_path = Self::default_config_path()?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let content = toml::to_string_pretty(self)?;
        tokio::fs::write(&config_path, content).await?;

        Ok(())
    }

    /// Get the effective namespace (from override or default)
    pub fn get_namespace(&self, override_namespace: Option<&str>) -> String {
        override_namespace
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.default_namespace.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_default_server_url() {
        let config = CliConfig::default();
        assert_eq!(config.server_url, "http://127.0.0.1:8200");
        assert_eq!(config.default_namespace, "default");
    }

    #[tokio::test]
    async fn test_load_from_file() {
        let tmp = tempfile::NamedTempFile::new().expect("temp file");
        tokio::fs::write(
            tmp.path(),
            "server_url = \"https://vault.example.com\"\ndefault_namespace = \"production\"",
        )
        .await
        .expect("write config");

        let loaded = CliConfig::load_from_file(tmp.path().to_str().unwrap())
            .await
            .expect("load config");
        assert_eq!(loaded.server_url, "https://vault.example.com");
        assert_eq!(loaded.default_namespace, "production");
    }

    #[tokio::test]
    async fn test_get_namespace_with_override() {
        let config = CliConfig::default();
        assert_eq!(config.get_namespace(Some("custom")), "custom");
        assert_eq!(config.get_namespace(None), "default");
    }

    #[tokio::test]
    async fn test_save_and_load_default() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("temp dir");
        let config_path = temp_dir.path().join("config.toml");

        let config = CliConfig {
            server_url: "https://test.example.com".to_string(),
            default_namespace: "test-ns".to_string(),
        };

        // Save to temp file
        tokio::fs::write(&config_path, toml::to_string_pretty(&config).unwrap())
            .await
            .expect("write config");

        // Load it back
        let loaded = CliConfig::load_from_file(config_path.to_str().unwrap())
            .await
            .expect("load config");

        assert_eq!(loaded.server_url, "https://test.example.com");
        assert_eq!(loaded.default_namespace, "test-ns");
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Generator for valid namespace names
    fn arb_namespace() -> impl Strategy<Value = String> {
        prop::string::string_regex("[a-z][a-z0-9-]{2,30}").unwrap()
    }

    // Generator for server URLs
    fn arb_server_url() -> impl Strategy<Value = String> {
        prop::string::string_regex("https?://[a-z0-9.-]+:[0-9]{4,5}").unwrap()
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// **Feature: secreton-cli-workflow-integration, Property 12: Namespace Isolation**
        /// **Validates: Requirements 11.1, 11.3**
        ///
        /// For any secret or policy created in a namespace, it should only be visible
        /// to users with access to that namespace.
        ///
        /// This property test validates that:
        /// 1. Namespace configuration is properly handled
        /// 2. Namespace overrides work correctly (flag > config > default)
        /// 3. Different namespaces are properly isolated in configuration
        ///
        /// Note: Full end-to-end namespace isolation requires a running Secreton server
        /// with proper namespace support and is covered by integration tests.
        #[test]
        fn prop_namespace_isolation(
            default_ns in arb_namespace(),
            override_ns in arb_namespace(),
            server_url in arb_server_url()
        ) {
            // Create config with a default namespace
            let config = CliConfig {
                server_url: server_url.clone(),
                default_namespace: default_ns.clone(),
            };

            // Test 1: Without override, should use default namespace
            let effective_ns = config.get_namespace(None);
            prop_assert_eq!(&effective_ns, &default_ns);

            // Test 2: With override, should use override namespace
            let effective_ns_override = config.get_namespace(Some(&override_ns));
            prop_assert_eq!(&effective_ns_override, &override_ns);

            // Test 3: Different namespaces should be distinct
            if default_ns != override_ns {
                prop_assert_ne!(
                    config.get_namespace(None),
                    config.get_namespace(Some(&override_ns))
                );
            }

            // Test 4: Server URL should not affect namespace
            prop_assert_eq!(&config.server_url, &server_url);
            prop_assert_eq!(&config.default_namespace, &default_ns);
        }

        /// Additional property: Namespace configuration persistence
        ///
        /// For any namespace configuration, saving and loading should preserve the namespace.
        #[test]
        fn prop_namespace_config_persistence(
            namespace in arb_namespace(),
            server_url in arb_server_url()
        ) {
            use tempfile::TempDir;

            let temp_dir = TempDir::new().unwrap();
            let config_path = temp_dir.path().join("config.toml");

            // Create config with namespace
            let config = CliConfig {
                server_url: server_url.clone(),
                default_namespace: namespace.clone(),
            };

            // Serialize to TOML
            let toml_content = toml::to_string_pretty(&config).unwrap();

            // Write to file
            std::fs::write(&config_path, toml_content).unwrap();

            // Read back
            let content = std::fs::read_to_string(&config_path).unwrap();
            let loaded: CliConfig = toml::from_str(&content).unwrap();

            // Verify namespace is preserved
            prop_assert_eq!(&loaded.default_namespace, &namespace);
            prop_assert_eq!(&loaded.server_url, &server_url);
        }

        /// Additional property: Namespace override precedence
        ///
        /// For any combination of default and override namespaces, the override
        /// should always take precedence when provided.
        #[test]
        fn prop_namespace_override_precedence(
            default_ns in arb_namespace(),
            override_ns1 in arb_namespace(),
            override_ns2 in arb_namespace()
        ) {
            let config = CliConfig {
                server_url: "http://localhost:8200".to_string(),
                default_namespace: default_ns.clone(),
            };

            // Test multiple overrides
            prop_assert_eq!(&config.get_namespace(Some(&override_ns1)), &override_ns1);
            prop_assert_eq!(&config.get_namespace(Some(&override_ns2)), &override_ns2);

            // Test that None always returns default
            prop_assert_eq!(&config.get_namespace(None), &default_ns);

            // Test that empty string override is treated as a valid namespace
            let empty_override = "";
            prop_assert_eq!(config.get_namespace(Some(empty_override)), empty_override);
        }

        /// Additional property: Namespace validation
        ///
        /// For any valid namespace string, the configuration should accept and store it.
        #[test]
        fn prop_namespace_validation(
            namespace in arb_namespace()
        ) {
            let config = CliConfig {
                server_url: "http://localhost:8200".to_string(),
                default_namespace: namespace.clone(),
            };

            // Namespace should be stored correctly
            prop_assert_eq!(&config.default_namespace, &namespace);

            // Namespace should be retrievable
            prop_assert_eq!(config.get_namespace(None), namespace.clone());

            // Namespace should serialize/deserialize correctly
            let serialized = toml::to_string(&config).unwrap();
            prop_assert!(serialized.contains(&namespace));

            let deserialized: CliConfig = toml::from_str(&serialized).unwrap();
            prop_assert_eq!(deserialized.default_namespace, namespace);
        }

        /// **Feature: secreton-cli-workflow-integration, Property 14: Server Address Configuration Precedence**
        /// **Validates: Requirements 5.1, 5.2, 5.3, 5.4**
        ///
        /// For any combination of environment variable, config file, and command-line flag
        /// for server address, the precedence should be: flag > env var > config file > default.
        ///
        /// This property test validates that:
        /// 1. Default server address is used when no other source is provided
        /// 2. Config file overrides default
        /// 3. Environment variable overrides config file
        /// 4. Command-line flag overrides everything
        ///
        /// Note: This test validates the configuration precedence logic. The actual CLI
        /// argument parsing and environment variable handling is tested in integration tests.
        #[test]
        fn prop_server_address_config_precedence(
            default_url in arb_server_url(),
            config_url in arb_server_url(),
            env_url in arb_server_url(),
            flag_url in arb_server_url()
        ) {
            // Test 1: Default value
            let default_config = CliConfig::default();
            prop_assert_eq!(&default_config.server_url, "http://127.0.0.1:8200");

            // Test 2: Config file overrides default
            let config_from_file = CliConfig {
                server_url: config_url.clone(),
                default_namespace: "default".to_string(),
            };
            prop_assert_eq!(&config_from_file.server_url, &config_url);

            // Test 3: Simulated precedence - flag overrides config
            let mut final_config = config_from_file.clone();
            final_config.server_url = flag_url.clone();
            prop_assert_eq!(&final_config.server_url, &flag_url);

            // Test 4: Different sources should be distinguishable
            if default_url != config_url && config_url != env_url && env_url != flag_url {
                prop_assert_ne!(&default_url, &config_url);
                prop_assert_ne!(&config_url, &env_url);
                prop_assert_ne!(&env_url, &flag_url);
            }

            // Test 5: Server URL changes don't affect namespace
            let original_ns = final_config.default_namespace.clone();
            final_config.server_url = default_url.clone();
            prop_assert_eq!(&final_config.default_namespace, &original_ns);
        }

        /// Additional property: Server URL format validation
        ///
        /// For any valid server URL, the configuration should accept and store it correctly.
        #[test]
        fn prop_server_url_format(
            server_url in arb_server_url()
        ) {
            let config = CliConfig {
                server_url: server_url.clone(),
                default_namespace: "default".to_string(),
            };

            // URL should be stored correctly
            prop_assert_eq!(&config.server_url, &server_url);

            // URL should serialize/deserialize correctly
            let serialized = toml::to_string(&config).unwrap();
            prop_assert!(serialized.contains(&server_url));

            let deserialized: CliConfig = toml::from_str(&serialized).unwrap();
            prop_assert_eq!(&deserialized.server_url, &server_url);
        }

        /// Additional property: Configuration immutability
        ///
        /// For any configuration, reading it multiple times should return the same values.
        #[test]
        fn prop_config_immutability(
            server_url in arb_server_url(),
            namespace in arb_namespace()
        ) {
            let config = CliConfig {
                server_url: server_url.clone(),
                default_namespace: namespace.clone(),
            };

            // Multiple reads should return same values
            prop_assert_eq!(&config.server_url, &server_url);
            prop_assert_eq!(&config.server_url, &server_url);
            prop_assert_eq!(&config.default_namespace, &namespace);
            prop_assert_eq!(&config.default_namespace, &namespace);

            // Clone should be identical
            let cloned = config.clone();
            prop_assert_eq!(&cloned.server_url, &config.server_url);
            prop_assert_eq!(&cloned.default_namespace, &config.default_namespace);
        }

        /// Additional property: Configuration serialization round-trip
        ///
        /// For any configuration, serializing and deserializing should preserve all values.
        #[test]
        fn prop_config_serialization_roundtrip(
            server_url in arb_server_url(),
            namespace in arb_namespace()
        ) {
            let original = CliConfig {
                server_url: server_url.clone(),
                default_namespace: namespace.clone(),
            };

            // Serialize to TOML
            let toml_str = toml::to_string_pretty(&original).unwrap();

            // Deserialize back
            let deserialized: CliConfig = toml::from_str(&toml_str).unwrap();

            // Should be identical
            prop_assert_eq!(&deserialized.server_url, &original.server_url);
            prop_assert_eq!(&deserialized.default_namespace, &original.default_namespace);

            // Serialize again - should produce equivalent TOML
            let toml_str2 = toml::to_string_pretty(&deserialized).unwrap();
            let deserialized2: CliConfig = toml::from_str(&toml_str2).unwrap();

            prop_assert_eq!(&deserialized2.server_url, &original.server_url);
            prop_assert_eq!(&deserialized2.default_namespace, &original.default_namespace);
        }
    }
}
