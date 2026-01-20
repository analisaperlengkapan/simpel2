//! Property-Based Tests for SSH Engine Enhancements
//!
//! These tests verify the correctness properties of the SSH secrets engine.

use proptest::prelude::*;
use secreton_core::services::secrets::ssh::{
    SshCertificateRequest, SshEngine, SshHostCertificateRequest, SshKeyType, SshRole,
};
use std::collections::HashMap;
use std::sync::Arc;

// Helper function to setup a test SSH engine with CA and role
async fn setup_test_ssh() -> Arc<SshEngine> {
    let engine = Arc::new(SshEngine::new());

    // Create SSH CA
    engine
        .create_ca("test-ca".to_string(), SshKeyType::Ed25519)
        .await
        .unwrap();

    // Create a test role with specific TTL bounds
    let mut role = SshRole::new(
        "test-role".to_string(),
        SshKeyType::Ed25519,
        "testuser".to_string(),
    );
    role.min_ttl = 60; // 1 minute
    role.max_ttl = 86400; // 24 hours
    role.default_ttl = 28800; // 8 hours
    role.allowed_users = vec!["user1".to_string(), "user2".to_string()];
    role.allow_host_certificates = true;

    // Add allowed extensions (Requirement 7.3)
    role.allowed_extensions.insert("permit-pty".to_string(), "".to_string());
    role.allowed_extensions.insert("permit-port-forwarding".to_string(), "".to_string());
    role.allowed_extensions.insert("permit-agent-forwarding".to_string(), "".to_string());

    engine.create_role(role).await.unwrap();

    engine
}

// Strategy to generate valid TTL within bounds (1 minute to 24 hours)
fn valid_ttl_strategy() -> impl Strategy<Value = i64> {
    60i64..=86400i64
}

// Strategy to generate TTL below minimum
fn below_min_ttl_strategy() -> impl Strategy<Value = i64> {
    1i64..60i64
}

// Strategy to generate TTL above maximum
fn above_max_ttl_strategy() -> impl Strategy<Value = i64> {
    86401i64..=172800i64
}

// Strategy to generate valid principals
fn principals_strategy() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(
        prop::string::string_regex("[a-z][a-z0-9]{2,10}").expect("valid regex"),
        1..=5,
    )
}

// Strategy to generate valid hostnames
fn hostnames_strategy() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(
        prop::string::string_regex("[a-z][a-z0-9-]{2,10}\\.(example|test)\\.com")
            .expect("valid regex"),
        1..=3,
    )
}

// **Feature: secreton-comprehensive-enhancement, Property 19: SSH Certificate Validity Bounds**
// **Validates: Requirements 7.1**
//
// Property: For any SSH certificate request with TTL, the issued certificate validity period
// SHALL be within bounds (min: 1 minute, max: 24 hours).
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test_ssh_certificate_valid_ttl_accepted(
        ttl in valid_ttl_strategy(),
        principals in principals_strategy()
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let engine = setup_test_ssh().await;

            // Generate a keypair to get a public key
            let keypair = engine
                .generate_keypair("test-role")
                .await
                .expect("Failed to generate keypair");

            // Request certificate with valid TTL
            let request = SshCertificateRequest {
                public_key: keypair.public_key,
                cert_type: "user".to_string(),
                valid_principals: principals.clone(),
                ttl: Some(ttl),
                extensions: None,
            };

            let result = engine
                .sign_certificate("test-ca", "test-role", request)
                .await;

            // Property: Valid TTL should be accepted
            prop_assert!(result.is_ok(), "Valid TTL {} should be accepted", ttl);

            let cert = result.unwrap();
            prop_assert!(!cert.serial_number.is_empty());
            prop_assert_eq!(cert.cert_type, "user");

            Ok(())
        })?;
    }

    #[test]
    fn test_ssh_certificate_below_min_ttl_rejected(
        ttl in below_min_ttl_strategy(),
        principals in principals_strategy()
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let engine = setup_test_ssh().await;

            // Generate a keypair to get a public key
            let keypair = engine
                .generate_keypair("test-role")
                .await
                .expect("Failed to generate keypair");

            // Request certificate with TTL below minimum
            let request = SshCertificateRequest {
                public_key: keypair.public_key,
                cert_type: "user".to_string(),
                valid_principals: principals.clone(),
                ttl: Some(ttl),
                extensions: None,
            };

            let result = engine
                .sign_certificate("test-ca", "test-role", request)
                .await;

            // Property: TTL below minimum should be rejected
            prop_assert!(result.is_err(), "TTL {} below minimum should be rejected", ttl);

            Ok(())
        })?;
    }

    #[test]
    fn test_ssh_certificate_above_max_ttl_rejected(
        ttl in above_max_ttl_strategy(),
        principals in principals_strategy()
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let engine = setup_test_ssh().await;

            // Generate a keypair to get a public key
            let keypair = engine
                .generate_keypair("test-role")
                .await
                .expect("Failed to generate keypair");

            // Request certificate with TTL above maximum
            let request = SshCertificateRequest {
                public_key: keypair.public_key,
                cert_type: "user".to_string(),
                valid_principals: principals.clone(),
                ttl: Some(ttl),
                extensions: None,
            };

            let result = engine
                .sign_certificate("test-ca", "test-role", request)
                .await;

            // Property: TTL above maximum should be rejected
            prop_assert!(result.is_err(), "TTL {} above maximum should be rejected", ttl);

            Ok(())
        })?;
    }
}

// **Feature: secreton-comprehensive-enhancement, Property 20: SSH Certificate Contains Requested Principals**
// **Validates: Requirements 7.2**
//
// Property: For any SSH certificate request with principals, the issued certificate
// SHALL contain exactly the requested principals.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test_ssh_certificate_contains_principals(
        principals in principals_strategy(),
        ttl in valid_ttl_strategy()
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let engine = setup_test_ssh().await;

            // Generate a keypair to get a public key
            let keypair = engine
                .generate_keypair("test-role")
                .await
                .expect("Failed to generate keypair");

            // Request certificate with specific principals
            let request = SshCertificateRequest {
                public_key: keypair.public_key,
                cert_type: "user".to_string(),
                valid_principals: principals.clone(),
                ttl: Some(ttl),
                extensions: None,
            };

            let cert = engine
                .sign_certificate("test-ca", "test-role", request)
                .await
                .expect("Failed to sign certificate");

            // Get certificate metadata
            let metadata = engine
                .get_certificate_metadata(&cert.serial_number)
                .await
                .expect("Certificate metadata should exist");

            // Property: Certificate should contain exactly the requested principals
            prop_assert_eq!(
                &metadata.principals,
                &principals,
                "Certificate principals should match requested principals"
            );

            // Verify principals are embedded in the signed key
            for principal in &metadata.principals {
                prop_assert!(
                    cert.signed_key.contains(principal),
                    "Signed key should contain principal: {}",
                    principal
                );
            }

            Ok(())
        })?;
    }

    #[test]
    fn test_ssh_host_certificate_contains_hostnames(
        hostnames in hostnames_strategy(),
        ttl in valid_ttl_strategy()
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let engine = setup_test_ssh().await;

            // Generate a keypair to get a public key
            let keypair = engine
                .generate_keypair("test-role")
                .await
                .expect("Failed to generate keypair");

            // Request host certificate with specific hostnames
            let request = SshHostCertificateRequest {
                public_key: keypair.public_key,
                hostnames: hostnames.clone(),
                ttl: Some(ttl),
            };

            let cert = engine
                .sign_host_certificate("test-ca", "test-role", request)
                .await
                .expect("Failed to sign host certificate");

            // Get certificate metadata
            let metadata = engine
                .get_certificate_metadata(&cert.serial_number)
                .await
                .expect("Certificate metadata should exist");

            // Property: Host certificate should contain exactly the requested hostnames as principals
            prop_assert_eq!(
                metadata.principals,
                hostnames,
                "Host certificate principals should match requested hostnames"
            );

            prop_assert_eq!(metadata.cert_type, "host");

            Ok(())
        })?;
    }
}

// Additional unit tests for SSH certificate extensions and audit
#[cfg(test)]
mod unit_tests {
    use super::*;

    #[tokio::test]
    async fn test_ssh_certificate_extensions() {
        let engine = setup_test_ssh().await;

        // Generate a keypair
        let keypair = engine
            .generate_keypair("test-role")
            .await
            .expect("Failed to generate keypair");

        // Request certificate with extensions
        let mut extensions = HashMap::new();
        extensions.insert("permit-pty".to_string(), "".to_string());
        extensions.insert("permit-port-forwarding".to_string(), "".to_string());

        let request = SshCertificateRequest {
            public_key: keypair.public_key,
            cert_type: "user".to_string(),
            valid_principals: vec!["testuser".to_string()],
            ttl: Some(3600),
            extensions: Some(extensions.clone()),
        };

        let cert = engine
            .sign_certificate("test-ca", "test-role", request)
            .await
            .expect("Failed to sign certificate");

        // Get certificate metadata
        let metadata = engine
            .get_certificate_metadata(&cert.serial_number)
            .await
            .expect("Certificate metadata should exist");

        // Verify extensions are included
        assert!(metadata.extensions.contains_key("permit-pty"));
        assert!(metadata.extensions.contains_key("permit-port-forwarding"));
    }

    #[tokio::test]
    async fn test_ssh_certificate_audit() {
        let engine = setup_test_ssh().await;

        // Generate and sign multiple certificates
        for i in 0..3 {
            let keypair = engine
                .generate_keypair("test-role")
                .await
                .expect("Failed to generate keypair");

            let request = SshCertificateRequest {
                public_key: keypair.public_key,
                cert_type: "user".to_string(),
                valid_principals: vec![format!("user{}", i)],
                ttl: Some(3600),
                extensions: None,
            };

            engine
                .sign_certificate("test-ca", "test-role", request)
                .await
                .expect("Failed to sign certificate");
        }

        // Get audit information
        let audit = engine.get_certificate_audit().await;

        // Should have 3 active certificates
        assert_eq!(audit.len(), 3);

        // Verify metadata fields
        for cert_meta in audit {
            assert!(!cert_meta.serial_number.is_empty());
            assert_eq!(cert_meta.cert_type, "user");
            assert_eq!(cert_meta.ca_name, "test-ca");
            assert_eq!(cert_meta.role_name, "test-role");
            assert!(cert_meta.valid_before > cert_meta.valid_after);
        }
    }

    #[tokio::test]
    async fn test_ssh_host_certificate_role_validation() {
        let engine = Arc::new(SshEngine::new());

        // Create CA
        engine
            .create_ca("test-ca".to_string(), SshKeyType::Ed25519)
            .await
            .unwrap();

        // Create role that does NOT allow host certificates
        let mut role = SshRole::new(
            "user-only-role".to_string(),
            SshKeyType::Ed25519,
            "testuser".to_string(),
        );
        role.allow_host_certificates = false;
        engine.create_role(role).await.unwrap();

        // Generate keypair
        let keypair = engine
            .generate_keypair("user-only-role")
            .await
            .expect("Failed to generate keypair");

        // Try to sign host certificate with role that doesn't allow it
        let request = SshHostCertificateRequest {
            public_key: keypair.public_key,
            hostnames: vec!["host.example.com".to_string()],
            ttl: Some(3600),
        };

        let result = engine
            .sign_host_certificate("test-ca", "user-only-role", request)
            .await;

        // Should fail
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_ssh_role_ttl_validation() {
        let role = SshRole::new(
            "test-role".to_string(),
            SshKeyType::Ed25519,
            "testuser".to_string(),
        );

        // Valid TTL
        assert!(role.validate_ttl(3600).is_ok());

        // Below minimum
        assert!(role.validate_ttl(30).is_err());

        // Above maximum
        assert!(role.validate_ttl(100000).is_err());

        // At boundaries
        assert!(role.validate_ttl(60).is_ok()); // min_ttl
        assert!(role.validate_ttl(86400).is_ok()); // max_ttl
    }
}
