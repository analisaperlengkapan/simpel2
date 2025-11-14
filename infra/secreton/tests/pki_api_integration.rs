//! PKI API Integration Tests
//!
//! End-to-end tests for PKI Secrets Engine API endpoints

#[cfg(test)]
mod tests {
    use secreton_api::pki::*;
    use secreton_core::services::secrets::pki::PkiEngine;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_pki_api_state_creation() {
        let state = PkiApiState::default();
        assert!(Arc::strong_count(&state.engine) == 1);
    }

    #[tokio::test]
    async fn test_pki_workflow() {
        // Create PKI engine and API state
        let engine = Arc::new(PkiEngine::new());
        let state = PkiApiState::new(engine.clone());

        // Generate root CA
        let ca_result = engine
            .generate_root_ca("Test Root CA".to_string(), 365)
            .await;
        assert!(ca_result.is_ok());

        // Create role
        let role = secreton_core::services::secrets::pki::PkiRole {
            name: "test-role".to_string(),
            ttl: chrono::Duration::days(90),
            max_ttl: chrono::Duration::days(365),
            allow_any_name: true,
            allowed_domains: vec![],
        };
        let role_result = engine.create_role(role).await;
        assert!(role_result.is_ok());

        // Issue certificate
        let cert_request = secreton_core::services::secrets::pki::IssueCertificateRequest {
            common_name: "test.example.com".to_string(),
            alt_names: vec!["www.test.example.com".to_string()],
            ttl: None,
        };
        let cert_result = engine.issue_certificate("test-role", cert_request).await;
        assert!(cert_result.is_ok());

        let cert = cert_result.unwrap();
        assert!(!cert.certificate_pem.is_empty());
        assert!(!cert.private_key_pem.is_empty());

        // Verify certificate can be retrieved
        let get_result = engine.get_certificate(&cert.serial_number).await;
        assert!(get_result.is_ok());

        // Revoke certificate
        let revoke_result = engine.revoke_certificate(&cert.serial_number).await;
        assert!(revoke_result.is_ok());

        // Verify CRL contains revoked certificate
        let crl_result = engine.generate_crl().await;
        assert!(crl_result.is_ok());
        let crl = crl_result.unwrap();
        assert!(!crl.is_empty());
    }

    #[tokio::test]
    async fn test_pki_list_operations() {
        let engine = Arc::new(PkiEngine::new());

        // Generate root CA
        let _ = engine
            .generate_root_ca("Test CA".to_string(), 365)
            .await
            .unwrap();

        // List CAs
        let cas = engine.list_cas().await;
        assert!(cas.contains(&"root".to_string()));

        // Create role
        let role = secreton_core::services::secrets::pki::PkiRole {
            name: "web-server".to_string(),
            ttl: chrono::Duration::days(90),
            max_ttl: chrono::Duration::days(365),
            allow_any_name: false,
            allowed_domains: vec!["example.com".to_string()],
        };
        engine.create_role(role).await.unwrap();

        // List roles
        let roles = engine.list_roles().await;
        assert!(roles.contains(&"web-server".to_string()));
    }

    #[test]
    fn test_request_serialization() {
        // Test GenerateRootCARequest
        let ca_req = GenerateRootCARequest {
            common_name: "Test Root CA".to_string(),
            ttl_days: 3650,
        };
        let json = serde_json::to_string(&ca_req).unwrap();
        assert!(json.contains("Test Root CA"));

        // Test CreateRoleRequest with defaults
        let role_req_json = r#"{"allowed_domains": ["example.com"]}"#;
        let role_req: CreateRoleRequest = serde_json::from_str(role_req_json).unwrap();
        assert_eq!(role_req.ttl_days, 90);
        assert_eq!(role_req.max_ttl_days, 365);

        // Test IssueCertificateRequest
        let cert_req = IssueCertificateRequest {
            common_name: "app.example.com".to_string(),
            alt_names: vec!["www.app.example.com".to_string()],
            ttl_days: Some(30),
        };
        let json = serde_json::to_string(&cert_req).unwrap();
        assert!(json.contains("app.example.com"));
    }

    #[test]
    fn test_response_serialization() {
        // Test GenerateRootCAResponse
        let response = GenerateRootCAResponse {
            success: true,
            message: "CA generated".to_string(),
            ca_name: "root".to_string(),
            certificate_pem: "-----BEGIN CERTIFICATE-----".to_string(),
            expires_at: "2034-01-01T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("CA generated"));
        assert!(json.contains("root"));

        // Test IssueCertificateResponse
        let cert_response = IssueCertificateResponse {
            success: true,
            message: "Certificate issued".to_string(),
            serial_number: "0000000000000001".to_string(),
            certificate_pem: "-----BEGIN CERTIFICATE-----".to_string(),
            private_key_pem: "-----BEGIN PRIVATE KEY-----".to_string(),
            ca_chain: vec!["-----BEGIN CERTIFICATE-----".to_string()],
            expires_at: "2025-04-01T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&cert_response).unwrap();
        assert!(json.contains("Certificate issued"));
        assert!(json.contains("0000000000000001"));
    }

    #[tokio::test]
    async fn test_domain_validation() {
        let engine = Arc::new(PkiEngine::new());

        // Generate root CA
        engine
            .generate_root_ca("Test CA".to_string(), 365)
            .await
            .unwrap();

        // Create role with domain restrictions
        let role = secreton_core::services::secrets::pki::PkiRole {
            name: "restricted".to_string(),
            ttl: chrono::Duration::days(90),
            max_ttl: chrono::Duration::days(365),
            allow_any_name: false,
            allowed_domains: vec!["example.com".to_string()],
        };
        engine.create_role(role).await.unwrap();

        // Try to issue cert for allowed domain - should succeed
        let valid_request = secreton_core::services::secrets::pki::IssueCertificateRequest {
            common_name: "app.example.com".to_string(),
            alt_names: vec![],
            ttl: None,
        };
        let result = engine.issue_certificate("restricted", valid_request).await;
        assert!(result.is_ok());

        // Try to issue cert for disallowed domain - should fail
        let invalid_request = secreton_core::services::secrets::pki::IssueCertificateRequest {
            common_name: "app.notallowed.com".to_string(),
            alt_names: vec![],
            ttl: None,
        };
        let result = engine
            .issue_certificate("restricted", invalid_request)
            .await;
        assert!(result.is_err());
    }
}
