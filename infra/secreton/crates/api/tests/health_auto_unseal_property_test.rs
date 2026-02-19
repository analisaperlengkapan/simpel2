//! Property-based tests for health endpoint auto-unseal status
//!
//! **Property 5: Health endpoint auto-unseal status**
//! *For any* health check request, the response should include auto-unseal provider
//! information and status when auto-unseal is enabled.
//!
//! **Validates: Requirements 2.1.9**
//!
//! This test verifies that:
//! 1. When auto-unseal is enabled, the health response includes auto_unseal field
//! 2. When auto-unseal is disabled, the auto_unseal field is None
//! 3. Provider information is correctly included in the response
//! 4. All provider types are handled correctly (AWS KMS, GCP KMS, Azure KV, Transit)
//! 5. The response format is consistent and serializable
//! 6. Sensitive data is not exposed in health responses

use proptest::prelude::*;
use secreton_api::response::{AutoUnsealStatus, HealthCheckResponse, HealthCheckDependencies, DependencyStatus};

/// Provider type for auto-unseal
#[derive(Debug, Clone, PartialEq)]
enum ProviderType {
    AwsKms,
    GcpKms,
    AzureKv,
    Transit,
}

impl ProviderType {
    fn as_str(&self) -> &'static str {
        match self {
            ProviderType::AwsKms => "aws-kms",
            ProviderType::GcpKms => "gcp-kms",
            ProviderType::AzureKv => "azure-kv",
            ProviderType::Transit => "transit",
        }
    }

    fn requires_region(&self) -> bool {
        matches!(self, ProviderType::AwsKms | ProviderType::GcpKms)
    }

    fn requires_endpoint(&self) -> bool {
        matches!(self, ProviderType::AzureKv | ProviderType::Transit)
    }
}

/// Strategy for generating provider types
fn provider_type_strategy() -> impl Strategy<Value = ProviderType> {
    prop_oneof![
        Just(ProviderType::AwsKms),
        Just(ProviderType::GcpKms),
        Just(ProviderType::AzureKv),
        Just(ProviderType::Transit),
    ]
}

/// Strategy for generating key IDs
fn key_id_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // AWS KMS ARN format
        "arn:aws:kms:[a-z]{2}-[a-z]+-[0-9]:[0-9]{12}:key/[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}",
        // AWS KMS alias format
        "alias/[a-z0-9-]{3,20}",
        // GCP KMS resource name format
        "projects/[a-z0-9-]{3,20}/locations/[a-z0-9-]{3,20}/keyRings/[a-z0-9-]{3,20}/cryptoKeys/[a-z0-9-]{3,20}",
        // Azure Key Vault key name
        "[a-z0-9-]{3,20}",
        // Transit key name
        "[a-z0-9-]{3,20}",
    ]
}

/// Strategy for generating AWS regions
fn aws_region_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("us-east-1".to_string()),
        Just("us-west-2".to_string()),
        Just("eu-west-1".to_string()),
        Just("ap-southeast-1".to_string()),
        Just("ap-northeast-1".to_string()),
    ]
}

/// Strategy for generating GCP locations
fn gcp_location_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("us-central1".to_string()),
        Just("us-east1".to_string()),
        Just("europe-west1".to_string()),
        Just("asia-southeast1".to_string()),
    ]
}

/// Strategy for generating Azure vault URLs
fn azure_vault_url_strategy() -> impl Strategy<Value = String> {
    "[a-z0-9-]{3,20}".prop_map(|name| format!("https://{}.vault.azure.net", name))
}

/// Strategy for generating Transit endpoints
fn transit_endpoint_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("https://secreton.internal:50052".to_string()),
        Just("https://vault.example.com:8200".to_string()),
        "[a-z0-9-]{3,20}".prop_map(|host| format!("https://{}:50052", host)),
    ]
}

/// Strategy for generating AutoUnsealStatus
fn auto_unseal_status_strategy() -> impl Strategy<Value = Option<AutoUnsealStatus>> {
    prop_oneof![
        // Case 1: Auto-unseal disabled (None)
        Just(None),
        // Case 2: Auto-unseal enabled with various configurations
        (
            provider_type_strategy(),
            key_id_strategy(),
            any::<bool>(), // provider_healthy
            any::<bool>(), // fallback_enabled
            proptest::option::of(any::<i64>().prop_map(|ts| {
                chrono::DateTime::from_timestamp(ts.abs() % 2_000_000_000, 0)
                    .unwrap_or_else(|| chrono::Utc::now())
            })),
        )
            .prop_flat_map(|(provider_type, key_id, provider_healthy, fallback_enabled, last_unseal)| {
                let provider_str = provider_type.as_str().to_string();

                // Generate region or endpoint based on provider type
                let region_strategy = if provider_type.requires_region() {
                    match provider_type {
                        ProviderType::AwsKms => aws_region_strategy().boxed(),
                        ProviderType::GcpKms => gcp_location_strategy().boxed(),
                        _ => Just(String::new()).boxed(),
                    }
                } else {
                    Just(String::new()).boxed()
                };

                let endpoint_strategy = if provider_type.requires_endpoint() {
                    match provider_type {
                        ProviderType::AzureKv => azure_vault_url_strategy().boxed(),
                        ProviderType::Transit => transit_endpoint_strategy().boxed(),
                        _ => Just(String::new()).boxed(),
                    }
                } else {
                    Just(String::new()).boxed()
                };

                (Just(provider_str), Just(key_id), region_strategy, endpoint_strategy, Just(provider_healthy), Just(fallback_enabled), Just(last_unseal))
            })
            .prop_map(|(provider, key_id, region, endpoint, provider_healthy, fallback_enabled, last_unseal)| {
                Some(AutoUnsealStatus {
                    enabled: true,
                    provider: Some(provider),
                    provider_key_id: Some(key_id),
                    provider_region: if !region.is_empty() { Some(region) } else { None },
                    provider_endpoint: if !endpoint.is_empty() { Some(endpoint) } else { None },
                    provider_healthy,
                    last_unseal,
                    fallback_enabled,
                })
            }),
    ]
}

/// Strategy for generating HealthCheckResponse
fn health_check_response_strategy() -> impl Strategy<Value = HealthCheckResponse> {
    (
        prop_oneof![
            Just("healthy".to_string()),
            Just("degraded".to_string()),
            Just("unhealthy".to_string()),
        ],
        "[0-9]{1,2}\\.[0-9]{1,2}\\.[0-9]{1,2}",
        0u64..86400u64, // uptime in seconds (0-24 hours)
        any::<bool>(),  // storage healthy
        any::<bool>(),  // crypto healthy
        any::<bool>(),  // audit healthy
        auto_unseal_status_strategy(),
    )
        .prop_map(
            |(status, version, uptime, storage_healthy, crypto_healthy, audit_healthy, auto_unseal)| {
                HealthCheckResponse {
                    status,
                    version,
                    uptime_seconds: uptime,
                    dependencies: HealthCheckDependencies {
                        storage: DependencyStatus {
                            healthy: storage_healthy,
                            message: None,
                            response_time_ms: Some(10),
                        },
                        crypto: DependencyStatus {
                            healthy: crypto_healthy,
                            message: None,
                            response_time_ms: Some(5),
                        },
                        audit: DependencyStatus {
                            healthy: audit_healthy,
                            message: None,
                            response_time_ms: Some(8),
                        },
                    },
                    auto_unseal,
                }
            },
        )
}

proptest! {
    /// **Property 5.1: Auto-unseal field presence**
    ///
    /// When auto-unseal is enabled, the health response MUST include the auto_unseal field.
    /// When auto-unseal is disabled, the auto_unseal field MUST be None.
    #[test]
    fn prop_auto_unseal_field_presence(health in health_check_response_strategy()) {
        // Serialize to JSON
        let json = serde_json::to_value(&health).expect("Failed to serialize health response");

        if health.auto_unseal.is_some() {
            // Auto-unseal enabled: field must be present in JSON
            prop_assert!(json.get("auto_unseal").is_some(), "auto_unseal field missing when enabled");

            let auto_unseal = health.auto_unseal.as_ref().unwrap();
            prop_assert!(auto_unseal.enabled, "auto_unseal.enabled must be true when present");
        } else {
            // Auto-unseal disabled: field should not be present (skip_serializing_if)
            prop_assert!(json.get("auto_unseal").is_none(), "auto_unseal field present when disabled");
        }
    }

    /// **Property 5.2: Provider information completeness**
    ///
    /// When auto-unseal is enabled, provider information MUST be complete and consistent
    /// with the provider type.
    #[test]
    fn prop_provider_information_completeness(health in health_check_response_strategy()) {
        if let Some(auto_unseal) = &health.auto_unseal {
            // Provider type must be present
            prop_assert!(auto_unseal.provider.is_some(), "provider type missing");

            let provider = auto_unseal.provider.as_ref().unwrap();

            // Provider key ID must be present
            prop_assert!(auto_unseal.provider_key_id.is_some(), "provider_key_id missing");

            // Check provider-specific requirements
            match provider.as_str() {
                "aws-kms" | "gcp-kms" => {
                    // Cloud KMS providers should have region
                    prop_assert!(
                        auto_unseal.provider_region.is_some(),
                        "region missing for cloud KMS provider: {}", provider
                    );
                }
                "azure-kv" | "transit" => {
                    // Azure and Transit should have endpoint
                    prop_assert!(
                        auto_unseal.provider_endpoint.is_some(),
                        "endpoint missing for provider: {}", provider
                    );
                }
                _ => {
                    // Unknown provider - should not happen in valid configurations
                    prop_assert!(false, "unknown provider type: {}", provider);
                }
            }
        }
    }

    /// **Property 5.3: JSON serialization consistency**
    ///
    /// Health responses with auto-unseal status MUST serialize to valid JSON
    /// and deserialize back to equivalent structures.
    #[test]
    fn prop_json_serialization_consistency(health in health_check_response_strategy()) {
        // Serialize to JSON
        let json_str = serde_json::to_string(&health).expect("Failed to serialize");

        // Deserialize back
        let deserialized: HealthCheckResponse = serde_json::from_str(&json_str)
            .expect("Failed to deserialize");

        // Check key fields match
        prop_assert_eq!(health.status, deserialized.status);
        prop_assert_eq!(health.version, deserialized.version);
        prop_assert_eq!(health.uptime_seconds, deserialized.uptime_seconds);

        // Check auto_unseal field
        match (&health.auto_unseal, &deserialized.auto_unseal) {
            (None, None) => {}, // Both None - OK
            (Some(orig), Some(deser)) => {
                prop_assert_eq!(orig.enabled, deser.enabled);
                prop_assert_eq!(orig.provider, deser.provider);
                prop_assert_eq!(orig.provider_key_id, deser.provider_key_id);
                prop_assert_eq!(orig.provider_region, deser.provider_region);
                prop_assert_eq!(orig.provider_endpoint, deser.provider_endpoint);
                prop_assert_eq!(orig.provider_healthy, deser.provider_healthy);
                prop_assert_eq!(orig.fallback_enabled, deser.fallback_enabled);
            }
            _ => prop_assert!(false, "auto_unseal mismatch after round-trip"),
        }
    }

    /// **Property 5.4: All provider types handled**
    ///
    /// The health endpoint MUST correctly handle all four provider types:
    /// AWS KMS, GCP KMS, Azure Key Vault, and Transit.
    #[test]
    fn prop_all_provider_types_handled(provider_type in provider_type_strategy()) {
        let provider_str = provider_type.as_str();

        // Create auto-unseal status for this provider
        let auto_unseal = AutoUnsealStatus {
            enabled: true,
            provider: Some(provider_str.to_string()),
            provider_key_id: Some("test-key".to_string()),
            provider_region: if provider_type.requires_region() {
                Some("us-east-1".to_string())
            } else {
                None
            },
            provider_endpoint: if provider_type.requires_endpoint() {
                Some("https://example.com".to_string())
            } else {
                None
            },
            provider_healthy: true,
            last_unseal: None,
            fallback_enabled: true,
        };

        // Create health response
        let health = HealthCheckResponse {
            status: "healthy".to_string(),
            version: "1.0.0".to_string(),
            uptime_seconds: 3600,
            dependencies: HealthCheckDependencies {
                storage: DependencyStatus::healthy(),
                crypto: DependencyStatus::healthy(),
                audit: DependencyStatus::healthy(),
            },
            auto_unseal: Some(auto_unseal),
        };

        // Serialize to JSON
        let json = serde_json::to_value(&health).expect("Failed to serialize");

        // Verify provider type is in JSON
        let auto_unseal_json = json.get("auto_unseal").expect("auto_unseal missing");
        let provider_json = auto_unseal_json.get("provider").expect("provider missing");

        prop_assert_eq!(
            provider_json.as_str().unwrap(),
            provider_str,
            "provider type mismatch"
        );
    }

    /// **Property 5.5: Sensitive data not exposed**
    ///
    /// The health endpoint MUST NOT expose sensitive data such as:
    /// - Encryption keys
    /// - Credentials
    /// - Full ARNs with account IDs (for AWS)
    ///
    /// Only metadata like provider type, key ID/alias, and region should be included.
    #[test]
    fn prop_no_sensitive_data_exposed(health in health_check_response_strategy()) {
        // Serialize to JSON string
        let json_str = serde_json::to_string(&health).expect("Failed to serialize");

        // Check for common sensitive patterns
        prop_assert!(
            !json_str.contains("secret"),
            "JSON contains 'secret' - possible sensitive data leak"
        );
        prop_assert!(
            !json_str.contains("password"),
            "JSON contains 'password' - possible sensitive data leak"
        );
        prop_assert!(
            !json_str.contains("credential"),
            "JSON contains 'credential' - possible sensitive data leak"
        );
        prop_assert!(
            !json_str.contains("private_key"),
            "JSON contains 'private_key' - possible sensitive data leak"
        );

        // If auto-unseal is present, verify only metadata is included
        if let Some(auto_unseal) = &health.auto_unseal {
            let json = serde_json::to_value(auto_unseal).expect("Failed to serialize auto_unseal");

            // Should only have these fields (all metadata, no secrets)
            let allowed_fields = vec![
                "enabled",
                "provider",
                "provider_key_id",
                "provider_region",
                "provider_endpoint",
                "provider_healthy",
                "last_unseal",
                "fallback_enabled",
            ];

            if let Some(obj) = json.as_object() {
                for key in obj.keys() {
                    prop_assert!(
                        allowed_fields.contains(&key.as_str()),
                        "unexpected field in auto_unseal: {}",
                        key
                    );
                }
            }
        }
    }

    /// **Property 5.6: Provider health status consistency**
    ///
    /// The provider_healthy field MUST be a boolean and MUST be present
    /// when auto-unseal is enabled.
    #[test]
    fn prop_provider_health_status_consistency(health in health_check_response_strategy()) {
        if let Some(auto_unseal) = &health.auto_unseal {
            // provider_healthy must be a boolean (always present, not Option)
            let json = serde_json::to_value(auto_unseal).expect("Failed to serialize");
            let provider_healthy = json.get("provider_healthy").expect("provider_healthy missing");

            prop_assert!(
                provider_healthy.is_boolean(),
                "provider_healthy must be boolean, got: {:?}",
                provider_healthy
            );
        }
    }

    /// **Property 5.7: Fallback configuration included**
    ///
    /// The fallback_enabled field MUST be present when auto-unseal is enabled,
    /// indicating whether manual unseal fallback is available.
    #[test]
    fn prop_fallback_configuration_included(health in health_check_response_strategy()) {
        if let Some(auto_unseal) = &health.auto_unseal {
            // fallback_enabled must be present
            let json = serde_json::to_value(auto_unseal).expect("Failed to serialize");
            let fallback_enabled = json.get("fallback_enabled").expect("fallback_enabled missing");

            prop_assert!(
                fallback_enabled.is_boolean(),
                "fallback_enabled must be boolean, got: {:?}",
                fallback_enabled
            );
        }
    }

    /// **Property 5.8: Last unseal timestamp format**
    ///
    /// When last_unseal is present, it MUST be a valid ISO 8601 timestamp.
    #[test]
    fn prop_last_unseal_timestamp_format(health in health_check_response_strategy()) {
        if let Some(auto_unseal) = &health.auto_unseal {
            if let Some(last_unseal) = auto_unseal.last_unseal {
                // Serialize to JSON
                let json = serde_json::to_value(auto_unseal).expect("Failed to serialize");
                let last_unseal_json = json.get("last_unseal").expect("last_unseal missing");

                // Should be a string in ISO 8601 format
                prop_assert!(
                    last_unseal_json.is_string(),
                    "last_unseal must be string, got: {:?}",
                    last_unseal_json
                );

                let timestamp_str = last_unseal_json.as_str().unwrap();

                // Verify it can be parsed back
                let parsed = chrono::DateTime::parse_from_rfc3339(timestamp_str);
                prop_assert!(
                    parsed.is_ok(),
                    "last_unseal timestamp not valid ISO 8601: {}",
                    timestamp_str
                );

                // Verify it matches the original
                let parsed_utc = parsed.unwrap().with_timezone(&chrono::Utc);
                prop_assert_eq!(
                    last_unseal.timestamp(),
                    parsed_utc.timestamp(),
                    "timestamp mismatch after round-trip"
                );
            }
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_provider_type_requirements() {
        assert!(ProviderType::AwsKms.requires_region());
        assert!(ProviderType::GcpKms.requires_region());
        assert!(!ProviderType::AzureKv.requires_region());
        assert!(!ProviderType::Transit.requires_region());

        assert!(!ProviderType::AwsKms.requires_endpoint());
        assert!(!ProviderType::GcpKms.requires_endpoint());
        assert!(ProviderType::AzureKv.requires_endpoint());
        assert!(ProviderType::Transit.requires_endpoint());
    }

    #[test]
    fn test_provider_type_strings() {
        assert_eq!(ProviderType::AwsKms.as_str(), "aws-kms");
        assert_eq!(ProviderType::GcpKms.as_str(), "gcp-kms");
        assert_eq!(ProviderType::AzureKv.as_str(), "azure-kv");
        assert_eq!(ProviderType::Transit.as_str(), "transit");
    }
}
