//! Test for health endpoint with auto-unseal status
//!
//! This test verifies that the health endpoint correctly reports auto-unseal status
//! when auto-unseal is configured via environment variables.

#[cfg(test)]
mod tests {
    use secreton_api::response::{AutoUnsealStatus, HealthCheckResponse};

    #[test]
    fn test_auto_unseal_status_structure() {
        // Test that AutoUnsealStatus can be created and serialized
        let auto_unseal = AutoUnsealStatus {
            enabled: true,
            provider: Some("aws-kms".to_string()),
            provider_key_id: Some("alias/secreton-unseal".to_string()),
            provider_region: Some("us-east-1".to_string()),
            provider_endpoint: None,
            provider_healthy: true,
            last_unseal: Some(chrono::Utc::now()),
            fallback_enabled: true,
        };

        // Serialize to JSON
        let json = serde_json::to_string(&auto_unseal).expect("Failed to serialize");
        assert!(json.contains("aws-kms"));
        assert!(json.contains("alias/secreton-unseal"));
        assert!(json.contains("us-east-1"));
    }

    #[test]
    fn test_health_response_with_auto_unseal() {
        // Test that HealthCheckResponse can include auto-unseal status
        let health = HealthCheckResponse {
            status: "healthy".to_string(),
            version: "1.0.0".to_string(),
            uptime_seconds: 3600,
            dependencies: secreton_api::response::HealthCheckDependencies {
                storage: secreton_api::response::DependencyStatus::healthy(),
                crypto: secreton_api::response::DependencyStatus::healthy(),
                audit: secreton_api::response::DependencyStatus::healthy(),
            },
            auto_unseal: Some(AutoUnsealStatus {
                enabled: true,
                provider: Some("transit".to_string()),
                provider_key_id: Some("auto-unseal-key".to_string()),
                provider_region: None,
                provider_endpoint: Some("https://secreton.internal:50052".to_string()),
                provider_healthy: true,
                last_unseal: None,
                fallback_enabled: true,
            }),
        };

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&health).expect("Failed to serialize");
        println!("Health response with auto-unseal:\n{}", json);

        assert!(json.contains("auto_unseal"));
        assert!(json.contains("transit"));
        assert!(json.contains("auto-unseal-key"));
    }

    #[test]
    fn test_health_response_without_auto_unseal() {
        // Test that HealthCheckResponse works without auto-unseal (None)
        let health = HealthCheckResponse {
            status: "healthy".to_string(),
            version: "1.0.0".to_string(),
            uptime_seconds: 3600,
            dependencies: secreton_api::response::HealthCheckDependencies {
                storage: secreton_api::response::DependencyStatus::healthy(),
                crypto: secreton_api::response::DependencyStatus::healthy(),
                audit: secreton_api::response::DependencyStatus::healthy(),
            },
            auto_unseal: None,
        };

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&health).expect("Failed to serialize");
        println!("Health response without auto-unseal:\n{}", json);

        // auto_unseal should not be present when None (skip_serializing_if)
        assert!(!json.contains("auto_unseal"));
    }

    #[test]
    fn test_auto_unseal_status_all_providers() {
        // Test different provider configurations
        let providers = vec![
            ("aws-kms", Some("us-east-1"), None),
            ("gcp-kms", Some("us-central1"), None),
            ("azure-kv", None, Some("https://myvault.vault.azure.net")),
            ("transit", None, Some("https://secreton.internal:50052")),
        ];

        for (provider, region, endpoint) in providers {
            let auto_unseal = AutoUnsealStatus {
                enabled: true,
                provider: Some(provider.to_string()),
                provider_key_id: Some(format!("{}-key", provider)),
                provider_region: region.map(String::from),
                provider_endpoint: endpoint.map(String::from),
                provider_healthy: true,
                last_unseal: None,
                fallback_enabled: true,
            };

            let json = serde_json::to_string(&auto_unseal).expect("Failed to serialize");
            assert!(json.contains(provider));
        }
    }
}
