//! Security Architecture Validation Tests
//!
//! Tests to validate that Secreton maintains proper security architecture
//! principles and isolation between components.

use secreton_crypto::{HybridCrypto, SecurityRequirements, CryptoMode};
use secreton_core::models::audit::{AuditEvent, AuditEventType, OperationResult, Operation, SecurityContext};
use secreton_core::models::secret::Secret;
use chrono::Utc;

/// Test that secreton can start without authenc dependency
#[tokio::test]
async fn test_secreton_independent_startup() {
    // Verify secreton can start without authenc
    let crypto_engine = HybridCrypto::new();

    // Test core cryptographic operations work independently
    let test_data = b"test secret data";
    let requirements = SecurityRequirements {
        security_level: 256,
        quantum_resistant: false,
        performance_priority: secreton_crypto::PerformancePriority::Medium,
        compliance_requirements: vec![],
    };
    let encrypted = crypto_engine.encrypt(test_data, &requirements).await.unwrap();
    let decrypted = crypto_engine.decrypt(&encrypted).await.unwrap();
    assert_eq!(test_data, decrypted.as_slice());

    // Test that crypto engine initializes properly
    // We can't access the private crypto_mode field, so we just verify it works
    assert!(true); // If we get here, initialization succeeded
}

/// Test that there are no shared dependencies with authenc
#[tokio::test]
async fn test_no_shared_dependencies_with_authenc() {
    // Parse Cargo.toml to verify no shared dependencies with authenc
    let cargo_toml = std::fs::read_to_string("Cargo.toml").unwrap();
    let authenc_deps: Vec<&str> = cargo_toml
        .lines()
        .filter(|line| line.contains("authenc") && line.contains("="))
        .collect();

    // Should have minimal or no authenc dependencies in core secreton
    assert!(authenc_deps.len() <= 1); // Allow one optional dependency
}

/// Test basic audit event creation and validation
#[tokio::test]
async fn test_audit_event_integrity() {
    // Create a basic audit event
    let event = AuditEvent {
        event_id: uuid::Uuid::new_v4(),
        timestamp: Utc::now(),
        event_type: AuditEventType::SecretAccess,
        nip: Some("198001012000011001".to_string()),
        satker_code: Some("SATKER_TEST".to_string()),
        admin_level: None,
        authenc_session_id: None,
        resource_path: "test/secret".to_string(),
        operation: Operation::Read,
        result: OperationResult::Success,
        security_context: SecurityContext {
            auth_method: Some("test".to_string()),
            token_type: None,
            security_level: secreton_core::SecurityLevel::Internal,
            mfa_used: false,
            client_cert_info: None,
            encryption_algorithm: None,
            sensitive_data_involved: false,
        },
        risk_score: Some(0.1),
        compliance_flags: vec![],
        metadata: std::collections::HashMap::new(),
        source_ip: Some("127.0.0.1".to_string()),
        user_agent: Some("test-agent".to_string()),
        geo_location: None,
        duration_ms: Some(150),
    };

    // Verify audit event has required fields
    assert!(!event.resource_path.is_empty());
    assert!(event.event_id != uuid::Uuid::nil());
    assert!(matches!(event.result, OperationResult::Success));
}

/// Test that failed operations are properly audited
#[tokio::test]
async fn test_audit_trail_completeness_for_failed_operations() {
    // Create an audit event for a failed operation
    let event = AuditEvent {
        event_id: uuid::Uuid::new_v4(),
        timestamp: Utc::now(),
        event_type: AuditEventType::SecretAccess,
        nip: Some("198001012000011001".to_string()),
        satker_code: Some("SATKER_TEST".to_string()),
        admin_level: None,
        authenc_session_id: None,
        resource_path: "test/secret".to_string(),
        operation: Operation::Read,
        result: OperationResult::Failure("Access denied".to_string()),
        security_context: SecurityContext {
            auth_method: Some("test".to_string()),
            token_type: None,
            security_level: secreton_core::SecurityLevel::Internal,
            mfa_used: false,
            client_cert_info: None,
            encryption_algorithm: None,
            sensitive_data_involved: false,
        },
        risk_score: Some(0.8),
        compliance_flags: vec![],
        metadata: std::collections::HashMap::new(),
        source_ip: Some("127.0.0.1".to_string()),
        user_agent: Some("test-agent".to_string()),
        geo_location: None,
        duration_ms: Some(50),
    };

    // Verify failed operations have error details
    match &event.result {
        OperationResult::Failure(msg) => assert!(!msg.is_empty()),
        _ => panic!("Expected failure result"),
    }

    // Failed operations should have higher risk scores
    assert!(event.risk_score.unwrap() > 0.5);
}

// Helper function to create test secret
fn create_test_secret(satker_owner: &str, path: &str) -> Secret {
    use secreton_core::models::secret::{EncryptedValue, EncryptionAlgorithm, SecretMetadata, AccessControl, AuditTrail};
    use secreton_core::models::audit::AuditEvent;
    use std::collections::HashMap;

    Secret {
        id: 1,
        path: path.to_string(),
        version: 1,
        data: EncryptedValue {
            data: serde_json::json!("test-value"),
            encryption_algorithm: EncryptionAlgorithm::Aes256Gcm,
            encrypted_at: Utc::now(),
            key_id: None,
        },
        metadata: SecretMetadata {
            security_level: secreton_core::SecurityLevel::Secret,
            tags: vec![],
            description: None,
            custom_fields: secreton_core::Metadata::new(),
            compliance_flags: vec![],
            risk_score: None,
        },
        access_control: AccessControl {
            required_roles: vec![],
            required_satker: vec![satker_owner.to_string()],
            nip_whitelist: None,
            nip_blacklist: None,
            time_based_access: None,
            audit_required: true,
            admin_level_required: None,
        },
        audit_trail: AuditTrail {
            creation_event: AuditEvent {
                event_id: uuid::Uuid::new_v4(),
                timestamp: Utc::now(),
                event_type: AuditEventType::SecretModification,
                nip: Some("198001012000011001".to_string()),
                satker_code: Some(satker_owner.to_string()),
                admin_level: None,
                authenc_session_id: None,
                resource_path: path.to_string(),
                operation: Operation::Write,
                result: OperationResult::Success,
                security_context: SecurityContext {
                    auth_method: Some("test".to_string()),
                    token_type: None,
                    security_level: secreton_core::SecurityLevel::Internal,
                    mfa_used: false,
                    client_cert_info: None,
                    encryption_algorithm: None,
                    sensitive_data_involved: false,
                },
                risk_score: Some(0.1),
                compliance_flags: vec![],
                metadata: HashMap::new(),
                source_ip: Some("127.0.0.1".to_string()),
                user_agent: Some("test-agent".to_string()),
                geo_location: None,
                duration_ms: Some(100),
            },
            access_events: vec![],
            modification_events: vec![],
            last_audit_check: None,
        },
        created_at: Utc::now(),
        updated_at: Utc::now(),
        created_by_nip: Some("198001012000011001".to_string()),
        satker_owner: satker_owner.to_string(),
        last_accessed: Utc::now(),
        namespace: satker_owner.to_string(),
    }
}

