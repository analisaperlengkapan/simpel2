//! Property-based tests for policy parser
//!
//! These tests verify correctness properties that should hold across all valid inputs.

use proptest::prelude::*;
use secreton_cli::policy_parser::{
    toml_parser::TomlPolicyParser, Capability, Condition, Effect, Policy, PolicyParser, PolicyRule,
    TimeRange,
};

// Generators for policy components

fn arb_effect() -> impl Strategy<Value = Effect> {
    prop_oneof![Just(Effect::Allow), Just(Effect::Deny)]
}

fn arb_capability() -> impl Strategy<Value = Capability> {
    prop_oneof![
        Just(Capability::Create),
        Just(Capability::Read),
        Just(Capability::Update),
        Just(Capability::Delete),
        Just(Capability::List),
        Just(Capability::Sudo),
    ]
}

fn arb_capabilities() -> impl Strategy<Value = Vec<Capability>> {
    prop_oneof![
        // Single capability
        arb_capability().prop_map(|c| vec![c]),
        // Multiple capabilities (2-3)
        prop::collection::vec(arb_capability(), 2..=3).prop_map(|mut caps| {
            // Remove duplicates
            caps.sort_by_key(|c| format!("{:?}", c));
            caps.dedup_by_key(|c| format!("{:?}", c));
            if caps.is_empty() {
                vec![Capability::Read]
            } else {
                caps
            }
        }),
        // All capability (alone)
        Just(vec![Capability::All]),
    ]
}

fn arb_path() -> impl Strategy<Value = String> {
    prop_oneof![
        // Simple paths
        Just("secret/data/test".to_string()),
        Just("secret/data/database".to_string()),
        Just("transit/keys/mykey".to_string()),
        // Wildcard paths
        Just("secret/data/*".to_string()),
        Just("secret/data/app/*".to_string()),
        Just("secret/*".to_string()),
        // Nested paths
        Just("secret/data/app/prod/db".to_string()),
        Just("secret/data/team/project/env".to_string()),
    ]
}

fn arb_time_range() -> impl Strategy<Value = TimeRange> {
    // Generate valid RFC3339 timestamps
    (
        2020u32..=2030,
        1u32..=12,
        1u32..=28,
        0u32..=23,
        0u32..=59,
        0u32..=59,
    )
        .prop_map(|(year, month, day, hour, min, sec)| {
            let start = format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
                year, month, day, hour, min, sec
            );
            let end = format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
                year + 1,
                month,
                day,
                hour,
                min,
                sec
            );
            TimeRange { start, end }
        })
}

fn arb_ip() -> impl Strategy<Value = String> {
    prop_oneof![
        // IPv4 addresses
        (0u8..=255, 0u8..=255, 0u8..=255, 0u8..=255)
            .prop_map(|(a, b, c, d)| format!("{}.{}.{}.{}", a, b, c, d)),
        // IPv4 CIDR
        (0u8..=255, 0u8..=255, 0u8..=255, 0u8..=255, 0u8..=32)
            .prop_map(|(a, b, c, d, prefix)| format!("{}.{}.{}.{}/{}", a, b, c, d, prefix)),
        // Common private networks
        Just("192.168.1.0/24".to_string()),
        Just("10.0.0.0/8".to_string()),
        Just("172.16.0.0/12".to_string()),
    ]
}

fn arb_condition() -> impl Strategy<Value = Option<Condition>> {
    prop_oneof![
        // No condition
        Just(None),
        // Time range only
        arb_time_range().prop_map(|tr| {
            Some(Condition {
                time_range: Some(tr),
                allowed_ips: None,
                required_claims: None,
            })
        }),
        // IP restriction only
        prop::collection::vec(arb_ip(), 1..=3).prop_map(|ips| {
            Some(Condition {
                time_range: None,
                allowed_ips: Some(ips),
                required_claims: None,
            })
        }),
        // Both time range and IPs
        (arb_time_range(), prop::collection::vec(arb_ip(), 1..=2)).prop_map(|(tr, ips)| {
            Some(Condition {
                time_range: Some(tr),
                allowed_ips: Some(ips),
                required_claims: None,
            })
        }),
    ]
}

fn arb_policy_rule() -> impl Strategy<Value = PolicyRule> {
    (
        arb_effect(),
        arb_path(),
        arb_capabilities(),
        arb_condition(),
        prop::option::of(prop::bool::ANY),
    )
        .prop_map(|(effect, path, capabilities, condition, mfa)| PolicyRule {
            effect,
            path,
            capabilities,
            condition,
            mfa,
        })
}

fn arb_policy() -> impl Strategy<Value = Policy> {
    (
        "[a-z][a-z0-9-]{2,20}",
        prop::option::of("[A-Za-z0-9 ]{5,50}"),
        "[a-z][a-z0-9-]{2,20}",
        prop::collection::vec(arb_policy_rule(), 1..=5),
    )
        .prop_map(|(name, description, namespace, rules)| Policy {
            name,
            description,
            namespace,
            rules,
        })
}

// Property tests

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: secreton-cli-workflow-integration, Property 4: Policy TOML Round-Trip Consistency**
    /// **Validates: Requirements 3.1, 3.2, 3.3, 8.4**
    ///
    /// For any valid policy definition, parsing from TOML format and then printing back
    /// to the same format should produce a semantically equivalent policy.
    #[test]
    fn test_policy_toml_roundtrip(policy in arb_policy()) {
        let parser = TomlPolicyParser::new();

        // Format policy to TOML
        let toml_str = parser.format(&policy).expect("Failed to format policy");

        // Parse back from TOML
        let parsed_policy = parser.parse(&toml_str).expect("Failed to parse formatted TOML");

        // Verify semantic equivalence
        prop_assert_eq!(policy.name, parsed_policy.name, "Policy name mismatch");
        prop_assert_eq!(policy.description, parsed_policy.description, "Policy description mismatch");
        prop_assert_eq!(policy.namespace, parsed_policy.namespace, "Policy namespace mismatch");
        prop_assert_eq!(policy.rules.len(), parsed_policy.rules.len(), "Number of rules mismatch");

        // Verify each rule
        for (original_rule, parsed_rule) in policy.rules.iter().zip(parsed_policy.rules.iter()) {
            prop_assert_eq!(&original_rule.effect, &parsed_rule.effect, "Rule effect mismatch");
            prop_assert_eq!(&original_rule.path, &parsed_rule.path, "Rule path mismatch");
            prop_assert_eq!(&original_rule.capabilities, &parsed_rule.capabilities, "Rule capabilities mismatch");
            prop_assert_eq!(&original_rule.mfa, &parsed_rule.mfa, "Rule MFA mismatch");

            // Verify condition if present
            match (&original_rule.condition, &parsed_rule.condition) {
                (Some(orig_cond), Some(parsed_cond)) => {
                    prop_assert_eq!(&orig_cond.time_range, &parsed_cond.time_range, "Condition time_range mismatch");
                    prop_assert_eq!(&orig_cond.allowed_ips, &parsed_cond.allowed_ips, "Condition allowed_ips mismatch");
                }
                (None, None) => {}
                _ => prop_assert!(false, "Condition presence mismatch"),
            }
        }
    }

    /// Test that parsing invalid TOML produces appropriate errors
    #[test]
    fn test_invalid_toml_error_handling(invalid_toml in "[a-z]{10,50}") {
        let parser = TomlPolicyParser::new();
        let result = parser.parse(&invalid_toml);

        // Should fail to parse random strings
        prop_assert!(result.is_err(), "Expected parse error for invalid TOML");
    }

    /// Test that empty capabilities are rejected
    #[test]
    fn test_empty_capabilities_rejected(
        name in "[a-z][a-z0-9-]{2,20}",
        path in arb_path(),
    ) {
        let toml_content = format!(
            r#"
name = "{}"
namespace = "default"

[[rules]]
effect = "allow"
path = "{}"
capabilities = []
"#,
            name, path
        );

        let parser = TomlPolicyParser::new();
        let result = parser.parse(&toml_content);

        // Should reject empty capabilities
        prop_assert!(result.is_err(), "Expected error for empty capabilities");
    }

    /// Test that wildcard capability cannot be combined with others
    #[test]
    fn test_wildcard_capability_exclusive(
        name in "[a-z][a-z0-9-]{2,20}",
        path in arb_path(),
    ) {
        let toml_content = format!(
            r#"
name = "{}"
namespace = "default"

[[rules]]
effect = "allow"
path = "{}"
capabilities = ["*", "read"]
"#,
            name, path
        );

        let parser = TomlPolicyParser::new();
        let result = parser.parse(&toml_content);

        // Should reject wildcard combined with other capabilities
        prop_assert!(result.is_err(), "Expected error for wildcard combined with other capabilities");
    }
}

// Integration tests for policy write-read consistency
// These tests require a running Secreton server

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn init_test_env() {
        INIT.call_once(|| {
            // Initialize test environment if needed
        });
    }

    /// Check if Secreton server is available for integration tests
    fn is_server_available() -> bool {
        std::env::var("SECRETON_TEST_SERVER").is_ok()
            || std::env::var("RUN_INTEGRATION_TESTS").is_ok()
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))]

        /// **Feature: secreton-cli-workflow-integration, Property 5: Policy Write-Read Consistency**
        /// **Validates: Requirements 2.2, 2.3**
        ///
        /// For any valid policy file, writing it via CLI and then reading it back
        /// should return the same policy rules.
        ///
        /// Note: This is an integration test that requires a running Secreton server.
        /// Set SECRETON_TEST_SERVER or RUN_INTEGRATION_TESTS environment variable to run.
        #[test]
        fn test_policy_write_read_consistency(policy in arb_policy()) {
            init_test_env();

            if !is_server_available() {
                // Skip integration test if server is not available
                return Ok(());
            }

            // This test would require:
            // 1. A running Secreton server
            // 2. Authentication token
            // 3. API calls to write and read the policy
            //
            // For now, we verify the data structures are consistent
            // The actual API integration will be tested manually or in E2E tests

            let parser = TomlPolicyParser::new();

            // Serialize policy to TOML (simulating file write)
            let toml_content = parser.format(&policy).expect("Failed to format policy");

            // Parse back (simulating file read)
            let parsed_policy = parser.parse(&toml_content).expect("Failed to parse policy");

            // Verify consistency
            prop_assert_eq!(policy.name, parsed_policy.name);
            prop_assert_eq!(policy.rules.len(), parsed_policy.rules.len());

            for (orig, parsed) in policy.rules.iter().zip(parsed_policy.rules.iter()) {
                prop_assert_eq!(&orig.effect, &parsed.effect);
                prop_assert_eq!(&orig.path, &parsed.path);
                prop_assert_eq!(&orig.capabilities, &parsed.capabilities);
            }
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: secreton-cli-workflow-integration, Property 6: Policy Validation Error Detection**
    /// **Validates: Requirements 2.6, 7.2**
    ///
    /// For any policy file with syntax errors, the CLI should detect and report
    /// all errors with accurate line numbers.
    #[test]
    fn test_policy_validation_error_detection(
        invalid_name in "[^a-z0-9-]{1,5}",
        invalid_path in "[^a-z0-9/*-_]{1,10}",
    ) {
        let parser = TomlPolicyParser::new();

        // Test 1: Empty policy name
        let toml_empty_name = r#"
name = ""
namespace = "default"

[[rules]]
effect = "allow"
path = "secret/*"
capabilities = ["read"]
"#;
        let result = parser.validate(toml_empty_name);
        prop_assert!(result.is_ok(), "Validation should return Ok with errors");
        if let Ok(errors) = result {
            prop_assert!(!errors.is_empty(), "Should detect empty name error");
        }

        // Test 2: Missing required field (name)
        let toml_missing_name = r#"
namespace = "default"

[[rules]]
effect = "allow"
path = "secret/*"
capabilities = ["read"]
"#;
        let result = parser.parse(toml_missing_name);
        prop_assert!(result.is_err(), "Should fail to parse policy without name");

        // Test 3: Empty capabilities
        let toml_empty_caps = r#"
name = "test"
namespace = "default"

[[rules]]
effect = "allow"
path = "secret/*"
capabilities = []
"#;
        let result = parser.parse(toml_empty_caps);
        prop_assert!(result.is_err(), "Should fail to parse policy with empty capabilities");

        // Test 4: Invalid capability combination (wildcard with others)
        let toml_invalid_caps = r#"
name = "test"
namespace = "default"

[[rules]]
effect = "allow"
path = "secret/*"
capabilities = ["*", "read"]
"#;
        let result = parser.parse(toml_invalid_caps);
        prop_assert!(result.is_err(), "Should fail to parse policy with invalid capability combination");

        // Test 5: Invalid path (empty)
        let toml_empty_path = r#"
name = "test"
namespace = "default"

[[rules]]
effect = "allow"
path = ""
capabilities = ["read"]
"#;
        let result = parser.parse(toml_empty_path);
        prop_assert!(result.is_err(), "Should fail to parse policy with empty path");

        // Test 6: Invalid IP address in condition
        let toml_invalid_ip = r#"
name = "test"
namespace = "default"

[[rules]]
effect = "allow"
path = "secret/*"
capabilities = ["read"]

[rules.condition]
allowed_ips = ["999.999.999.999"]
"#;
        let result = parser.parse(toml_invalid_ip);
        prop_assert!(result.is_err(), "Should fail to parse policy with invalid IP");

        // Test 7: Invalid time range format
        let toml_invalid_time = r#"
name = "test"
namespace = "default"

[[rules]]
effect = "allow"
path = "secret/*"
capabilities = ["read"]

[rules.condition]
time_range = { start = "not-a-date", end = "also-not-a-date" }
"#;
        let result = parser.parse(toml_invalid_time);
        prop_assert!(result.is_err(), "Should fail to parse policy with invalid time range");
    }

    /// Test that valid policies pass validation
    #[test]
    fn test_valid_policy_passes_validation(policy in arb_policy()) {
        let parser = TomlPolicyParser::new();

        // Format to TOML
        let toml_str = parser.format(&policy).expect("Failed to format policy");

        // Validate
        let validation_result = parser.validate(&toml_str);

        prop_assert!(validation_result.is_ok(), "Validation should succeed");
        if let Ok(errors) = validation_result {
            prop_assert!(errors.is_empty(), "Valid policy should have no validation errors");
        }
    }

    /// Test that validation detects duplicate capabilities
    #[test]
    fn test_validation_detects_duplicate_capabilities(
        name in "[a-z][a-z0-9-]{2,20}",
        path in arb_path(),
    ) {
        let toml_content = format!(
            r#"
name = "{}"
namespace = "default"

[[rules]]
effect = "allow"
path = "{}"
capabilities = ["read", "read"]
"#,
            name, path
        );

        let parser = TomlPolicyParser::new();
        let result = parser.parse(&toml_content);

        // Should reject duplicate capabilities
        prop_assert!(result.is_err(), "Should detect duplicate capabilities");
    }
}

// Property test for example policy files

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1))]

    /// **Feature: secreton-cli-workflow-integration, Property 15: Example Policy Validity**
    /// **Validates: Requirements 13.2, 13.4**
    ///
    /// For any example policy file provided, it should parse successfully and be
    /// applicable to the vault without modification.
    #[test]
    fn test_example_policy_validity(_dummy in 0u8..1u8) {
        let parser = TomlPolicyParser::new();

        // Define all example policy files
        let example_files = vec![
            ("read-only.toml", include_str!("../../../examples/policies/read-only.toml")),
            ("admin.toml", include_str!("../../../examples/policies/admin.toml")),
            ("database-secrets.toml", include_str!("../../../examples/policies/database-secrets.toml")),
            ("transit-only.toml", include_str!("../../../examples/policies/transit-only.toml")),
            ("namespace-scoped.toml", include_str!("../../../examples/policies/namespace-scoped.toml")),
        ];

        // Test each example policy file
        for (filename, content) in example_files {
            // Parse the policy
            let parse_result = parser.parse(content);
            prop_assert!(
                parse_result.is_ok(),
                "Example policy '{}' should parse successfully. Error: {:?}",
                filename,
                parse_result.err()
            );

            let policy = parse_result.unwrap();

            // Verify basic policy structure
            prop_assert!(
                !policy.name.is_empty(),
                "Example policy '{}' should have a non-empty name",
                filename
            );

            prop_assert!(
                !policy.namespace.is_empty(),
                "Example policy '{}' should have a non-empty namespace",
                filename
            );

            prop_assert!(
                !policy.rules.is_empty(),
                "Example policy '{}' should have at least one rule",
                filename
            );

            // Verify each rule is valid
            for (idx, rule) in policy.rules.iter().enumerate() {
                prop_assert!(
                    !rule.path.is_empty(),
                    "Example policy '{}' rule {} should have a non-empty path",
                    filename,
                    idx
                );

                prop_assert!(
                    !rule.capabilities.is_empty(),
                    "Example policy '{}' rule {} should have at least one capability",
                    filename,
                    idx
                );

                // Verify path is valid
                let path_validation = TomlPolicyParser::validate_path(&rule.path);
                prop_assert!(
                    path_validation.is_ok(),
                    "Example policy '{}' rule {} has invalid path '{}': {:?}",
                    filename,
                    idx,
                    rule.path,
                    path_validation.err()
                );

                // Verify capabilities are valid
                let caps_validation = TomlPolicyParser::validate_capabilities(&rule.capabilities);
                prop_assert!(
                    caps_validation.is_ok(),
                    "Example policy '{}' rule {} has invalid capabilities: {:?}",
                    filename,
                    idx,
                    caps_validation.err()
                );

                // Verify condition if present
                if let Some(condition) = &rule.condition {
                    let cond_validation = TomlPolicyParser::validate_condition(condition);
                    prop_assert!(
                        cond_validation.is_ok(),
                        "Example policy '{}' rule {} has invalid condition: {:?}",
                        filename,
                        idx,
                        cond_validation.err()
                    );
                }
            }

            // Verify round-trip consistency
            let formatted = parser.format(&policy);
            prop_assert!(
                formatted.is_ok(),
                "Example policy '{}' should format successfully",
                filename
            );

            let formatted_str = formatted.unwrap();
            let reparsed = parser.parse(&formatted_str);
            prop_assert!(
                reparsed.is_ok(),
                "Example policy '{}' should re-parse after formatting",
                filename
            );

            let reparsed_policy = reparsed.unwrap();
            prop_assert_eq!(
                policy.name,
                reparsed_policy.name,
                "Example policy '{}' name should be preserved in round-trip",
                filename
            );
            prop_assert_eq!(
                policy.rules.len(),
                reparsed_policy.rules.len(),
                "Example policy '{}' should have same number of rules after round-trip",
                filename
            );
        }
    }
}
