//! Property-Based Tests for PKI Engine
//!
//! These tests verify the correctness properties of the PKI (Public Key Infrastructure) system.

use proptest::prelude::*;
use secreton_core::services::secrets::pki::{
    IssueCertificateRequest, OcspStatus, PkiEngine, PkiRole,
};
use std::sync::Arc;

// Helper function to setup a test PKI engine with root CA and role
async fn setup_test_pki() -> Arc<PkiEngine> {
    let engine = Arc::new(PkiEngine::new());

    // Generate root CA
    engine
        .generate_root_ca("Test CA".to_string(), 365)
        .await
        .unwrap();

    // Create a test role
    let role = PkiRole {
        name: "test-role".to_string(),
        allow_any_name: true,
        ..Default::default()
    };
    engine.create_role(role).await.unwrap();

    engine
}

// Strategy to generate valid common names
fn common_name_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z][a-z0-9-]{0,10}\\.(example|test)\\.com")
        .expect("valid regex")
}

// **Feature: secreton-comprehensive-enhancement, Property 16: OCSP Status Consistency**
// **Validates: Requirements 6.1**
//
// Property: For any certificate, OCSP response status SHALL match the certificate's actual status
// (good if valid, revoked if revoked).
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test_ocsp_good_status_consistency(common_name in common_name_strategy()) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let engine = setup_test_pki().await;

            // Issue a certificate
            let request = IssueCertificateRequest {
                common_name: common_name.clone(),
                alt_names: vec![],
                ttl: None,
            };

            let cert = engine
                .issue_certificate("test-role", request)
                .await
                .expect("Failed to issue certificate");

            // Check OCSP status - should be Good
            let response = engine
                .get_ocsp_status(&cert.serial_number)
                .await
                .expect("Failed to get OCSP status");

            // Property: Issued certificate should have Good status
            prop_assert_eq!(response.status, OcspStatus::Good);
            prop_assert_eq!(response.serial_number, cert.serial_number);
            prop_assert!(response.revocation_time.is_none());

            Ok(())
        })?;
    }

    #[test]
    fn test_ocsp_revoked_status_consistency(common_name in common_name_strategy()) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let engine = setup_test_pki().await;

            // Issue a certificate
            let request = IssueCertificateRequest {
                common_name: common_name.clone(),
                alt_names: vec![],
                ttl: None,
            };

            let cert = engine
                .issue_certificate("test-role", request)
                .await
                .expect("Failed to issue certificate");

            // Revoke the certificate
            engine
                .revoke_certificate(&cert.serial_number)
                .await
                .expect("Failed to revoke certificate");

            // Check OCSP status - should be Revoked
            let response = engine
                .get_ocsp_status(&cert.serial_number)
                .await
                .expect("Failed to get OCSP status");

            // Property: Revoked certificate should have Revoked status
            prop_assert_eq!(response.status, OcspStatus::Revoked);
            prop_assert_eq!(response.serial_number, cert.serial_number);
            prop_assert!(response.revocation_time.is_some());

            Ok(())
        })?;
    }

    #[test]
    fn test_ocsp_unknown_status_consistency(serial in "[a-f0-9]{16}") {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let engine = setup_test_pki().await;

            // Check status for non-existent certificate
            let response = engine
                .get_ocsp_status(&serial)
                .await
                .expect("Failed to get OCSP status");

            // Property: Non-existent certificate should have Unknown status
            prop_assert_eq!(response.status, OcspStatus::Unknown);
            prop_assert_eq!(response.serial_number, serial);
            prop_assert!(response.revocation_time.is_none());

            Ok(())
        })?;
    }

    #[test]
    fn test_ocsp_status_transition(common_name in common_name_strategy()) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let engine = setup_test_pki().await;

            // Issue a certificate
            let request = IssueCertificateRequest {
                common_name: common_name.clone(),
                alt_names: vec![],
                ttl: None,
            };

            let cert = engine
                .issue_certificate("test-role", request)
                .await
                .expect("Failed to issue certificate");

            // Property: Status should transition from Good to Revoked
            let response1 = engine
                .get_ocsp_status(&cert.serial_number)
                .await
                .expect("Failed to get OCSP status");
            prop_assert_eq!(response1.status, OcspStatus::Good);

            // Revoke the certificate
            engine
                .revoke_certificate(&cert.serial_number)
                .await
                .expect("Failed to revoke certificate");

            let response2 = engine
                .get_ocsp_status(&cert.serial_number)
                .await
                .expect("Failed to get OCSP status");
            prop_assert_eq!(response2.status, OcspStatus::Revoked);

            // Property: Once revoked, status should remain Revoked
            let response3 = engine
                .get_ocsp_status(&cert.serial_number)
                .await
                .expect("Failed to get OCSP status");
            prop_assert_eq!(response3.status, OcspStatus::Revoked);

            Ok(())
        })?;
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[tokio::test]
    async fn test_ocsp_response_fields() {
        let engine = setup_test_pki().await;

        // Issue a certificate
        let request = IssueCertificateRequest {
            common_name: "test.example.com".to_string(),
            alt_names: vec![],
            ttl: None,
        };

        let cert = engine
            .issue_certificate("test-role", request)
            .await
            .unwrap();

        // Get OCSP response
        let response = engine.get_ocsp_status(&cert.serial_number).await.unwrap();

        // Verify response fields
        assert_eq!(response.serial_number, cert.serial_number);
        assert_eq!(response.status, OcspStatus::Good);
        assert!(response.this_update <= chrono::Utc::now());
        assert!(response.next_update > response.this_update);
        assert!(response.revocation_time.is_none());
    }

    #[tokio::test]
    async fn test_ocsp_revocation_time() {
        let engine = setup_test_pki().await;

        // Issue and revoke a certificate
        let request = IssueCertificateRequest {
            common_name: "test.example.com".to_string(),
            alt_names: vec![],
            ttl: None,
        };

        let cert = engine
            .issue_certificate("test-role", request)
            .await
            .unwrap();

        let before_revocation = chrono::Utc::now();
        engine.revoke_certificate(&cert.serial_number).await.unwrap();
        let after_revocation = chrono::Utc::now();

        // Get OCSP response
        let response = engine.get_ocsp_status(&cert.serial_number).await.unwrap();

        // Verify revocation time is set and within expected range
        assert_eq!(response.status, OcspStatus::Revoked);
        assert!(response.revocation_time.is_some());
        let revocation_time = response.revocation_time.unwrap();
        assert!(revocation_time >= before_revocation);
        assert!(revocation_time <= after_revocation);
    }

    #[tokio::test]
    async fn test_template_domain_enforcement() {
        let engine = setup_test_pki().await;

        // Create a restrictive template
        let template = secreton_core::services::secrets::pki::CertificateTemplate {
            name: "domain-restricted".to_string(),
            allow_any_name: false,
            allowed_domains: vec!["example.com".to_string(), "test.com".to_string()],
            ..Default::default()
        };
        engine.create_role(template).await.unwrap();

        // Valid domain should succeed
        let request = IssueCertificateRequest {
            common_name: "api.example.com".to_string(),
            alt_names: vec![],
            ttl: None,
        };
        assert!(engine
            .issue_certificate("domain-restricted", request)
            .await
            .is_ok());

        // Invalid domain should fail
        let request = IssueCertificateRequest {
            common_name: "api.invalid.com".to_string(),
            alt_names: vec![],
            ttl: None,
        };
        assert!(engine
            .issue_certificate("domain-restricted", request)
            .await
            .is_err());
    }
}

// **Feature: secreton-comprehensive-enhancement, Property 17: Certificate Template Enforcement**
// **Validates: Requirements 6.3**
//
// Property: For any certificate issued under a template, the certificate SHALL conform to all
// template constraints (key usage, validity period, etc.).
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn test_template_ttl_enforcement(
        ttl_days in 1i64..100i64,
        max_ttl_days in 100i64..200i64,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let engine = setup_test_pki().await;

            // Create template with specific TTL constraints
            let template = secreton_core::services::secrets::pki::CertificateTemplate {
                name: "ttl-test".to_string(),
                allow_any_name: true,
                ttl: chrono::Duration::days(ttl_days),
                max_ttl: chrono::Duration::days(max_ttl_days),
                ..Default::default()
            };
            engine.create_role(template).await.unwrap();

            // Property: Certificate with TTL within max_ttl should succeed
            let request = IssueCertificateRequest {
                common_name: "test.example.com".to_string(),
                alt_names: vec![],
                ttl: Some(chrono::Duration::days(ttl_days)),
            };
            let result = engine.issue_certificate("ttl-test", request).await;
            prop_assert!(result.is_ok());

            // Property: Certificate with TTL exceeding max_ttl should fail
            let request = IssueCertificateRequest {
                common_name: "test2.example.com".to_string(),
                alt_names: vec![],
                ttl: Some(chrono::Duration::days(max_ttl_days + 1)),
            };
            let result = engine.issue_certificate("ttl-test", request).await;
            prop_assert!(result.is_err());

            Ok(())
        })?;
    }

    #[test]
    fn test_template_domain_constraint(
        subdomain in "[a-z]{3,8}",
        domain in "(example|test|demo)\\.com",
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let engine = setup_test_pki().await;

            // Create template with domain constraints
            let template = secreton_core::services::secrets::pki::CertificateTemplate {
                name: "domain-test".to_string(),
                allow_any_name: false,
                allowed_domains: vec![domain.clone()],
                ..Default::default()
            };
            engine.create_role(template).await.unwrap();

            // Property: Certificate with allowed domain should succeed
            let common_name = format!("{}.{}", subdomain, domain);
            let request = IssueCertificateRequest {
                common_name: common_name.clone(),
                alt_names: vec![],
                ttl: None,
            };
            let result = engine.issue_certificate("domain-test", request).await;
            prop_assert!(result.is_ok(), "Failed to issue cert for allowed domain: {}", common_name);

            // Property: Certificate with disallowed domain should fail
            let invalid_domain = "invalid.org";
            let request = IssueCertificateRequest {
                common_name: format!("{}.{}", subdomain, invalid_domain),
                alt_names: vec![],
                ttl: None,
            };
            let result = engine.issue_certificate("domain-test", request).await;
            prop_assert!(result.is_err());

            Ok(())
        })?;
    }

    #[test]
    fn test_template_localhost_constraint(allow_localhost: bool) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let engine = setup_test_pki().await;

            // Create template with localhost constraint
            let template = secreton_core::services::secrets::pki::CertificateTemplate {
                name: "localhost-test".to_string(),
                allow_any_name: true,
                allow_localhost,
                ..Default::default()
            };
            engine.create_role(template).await.unwrap();

            // Property: Localhost certificate should succeed/fail based on template
            let request = IssueCertificateRequest {
                common_name: "localhost".to_string(),
                alt_names: vec![],
                ttl: None,
            };
            let result = engine.issue_certificate("localhost-test", request).await;

            if allow_localhost {
                prop_assert!(result.is_ok(), "Should allow localhost when template permits");
            } else {
                prop_assert!(result.is_err(), "Should reject localhost when template forbids");
            }

            Ok(())
        })?;
    }

    #[test]
    fn test_template_cn_requirement(require_cn: bool, cn in "[a-z]{0,10}") {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let engine = setup_test_pki().await;

            // Create template with CN requirement
            let template = secreton_core::services::secrets::pki::CertificateTemplate {
                name: "cn-test".to_string(),
                allow_any_name: true,
                require_cn,
                ..Default::default()
            };
            engine.create_role(template).await.unwrap();

            // Property: Empty CN should succeed/fail based on template
            let request = IssueCertificateRequest {
                common_name: cn.clone(),
                alt_names: vec![],
                ttl: None,
            };
            let result = engine.issue_certificate("cn-test", request).await;

            if require_cn && cn.is_empty() {
                prop_assert!(result.is_err(), "Should reject empty CN when required");
            } else if !cn.is_empty() {
                prop_assert!(result.is_ok(), "Should accept non-empty CN");
            }

            Ok(())
        })?;
    }
}

// **Feature: secreton-comprehensive-enhancement, Property 18: CRL Contains All Revoked Certificates**
// **Validates: Requirements 6.5**
//
// Property: For any CRL generation, the CRL SHALL contain serial numbers of all revoked certificates
// and none of the valid certificates.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn test_crl_completeness(
        num_valid_certs in 1usize..10usize,
        num_revoked_certs in 1usize..10usize,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let engine = setup_test_pki().await;

            let mut valid_serials = Vec::new();
            let mut revoked_serials = Vec::new();

            // Issue valid certificates
            for i in 0..num_valid_certs {
                let request = IssueCertificateRequest {
                    common_name: format!("valid-{}.example.com", i),
                    alt_names: vec![],
                    ttl: None,
                };
                let cert = engine
                    .issue_certificate("test-role", request)
                    .await
                    .unwrap();
                valid_serials.push(cert.serial_number);
            }

            // Issue and revoke certificates
            for i in 0..num_revoked_certs {
                let request = IssueCertificateRequest {
                    common_name: format!("revoked-{}.example.com", i),
                    alt_names: vec![],
                    ttl: None,
                };
                let cert = engine
                    .issue_certificate("test-role", request)
                    .await
                    .unwrap();

                engine.revoke_certificate(&cert.serial_number).await.unwrap();
                revoked_serials.push(cert.serial_number);
            }

            // Generate CRL
            let crl = engine.generate_crl().await.unwrap();
            let crl_serials: Vec<String> = crl.iter().map(|r| r.serial_number.clone()).collect();

            // Property 1: CRL contains all revoked certificates
            for revoked_serial in &revoked_serials {
                prop_assert!(
                    crl_serials.contains(revoked_serial),
                    "CRL missing revoked certificate: {}",
                    revoked_serial
                );
            }

            // Property 2: CRL does not contain valid certificates
            for valid_serial in &valid_serials {
                prop_assert!(
                    !crl_serials.contains(valid_serial),
                    "CRL incorrectly contains valid certificate: {}",
         valid_serial
                );
            }

            // Property 3: CRL size matches number of revoked certificates
            prop_assert_eq!(
                crl.len(),
                num_revoked_certs,
                "CRL size mismatch: expected {}, got {}",
                num_revoked_certs,
                crl.len()
            );

            Ok(())
        })?;
    }

    #[test]
    fn test_crl_revocation_time(num_certs in 1usize..5usize) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let engine = setup_test_pki().await;

            let before_revocation = chrono::Utc::now();

            // Issue and revoke certificates
            for i in 0..num_certs {
                let request = IssueCertificateRequest {
                    common_name: format!("test-{}.example.com", i),
                    alt_names: vec![],
                    ttl: None,
                };
                let cert = engine
                    .issue_certificate("test-role", request)
                    .await
                    .unwrap();

                engine.revoke_certificate(&cert.serial_number).await.unwrap();
            }

            let after_revocation = chrono::Utc::now();

            // Generate CRL
            let crl = engine.generate_crl().await.unwrap();

            // Property: All revocation times should be within the test window
            for entry in crl {
                prop_assert!(
                    entry.revoked_at >= before_revocation,
                    "Revocation time too early"
                );
                prop_assert!(
                    entry.revoked_at <= after_revocation,
                    "Revocation time too late"
                );
            }

            Ok(())
        })?;
    }

    #[test]
    fn test_crl_idempotence(num_certs in 1usize..5usize) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let engine = setup_test_pki().await;

            // Issue and revoke certificates
            for i in 0..num_certs {
                let request = IssueCertificateRequest {
                    common_name: format!("test-{}.example.com", i),
                    alt_names: vec![],
                    ttl: None,
                };
                let cert = engine
                    .issue_certificate("test-role", request)
                    .await
                    .unwrap();

                engine.revoke_certificate(&cert.serial_number).await.unwrap();
            }

            // Generate CRL multiple times
            let crl1 = engine.generate_crl().await.unwrap();
            let crl2 = engine.generate_crl().await.unwrap();

            // Property: Multiple CRL generations should produce identical results
            prop_assert_eq!(crl1.len(), crl2.len(), "CRL size changed between generations");

            let serials1: Vec<String> = crl1.iter().map(|r| r.serial_number.clone()).collect();
            let serials2: Vec<String> = crl2.iter().map(|r| r.serial_number.clone()).collect();

            for serial in &serials1 {
                prop_assert!(
                    serials2.contains(serial),
                    "CRL inconsistency: serial {} missing in second generation",
                    serial
                );
            }

            Ok(())
        })?;
    }
}
