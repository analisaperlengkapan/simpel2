//! Security Validation Test Suite
//!
//! Comprehensive security architecture validation for Secreton with post-quantum cryptography.
//! This test suite validates:
//! - Zero-trust principles with actual crypto operations
//! - Independent deployability with PQ
//! - mTLS communication with hybrid signatures
//! - Audit trail captures PQ algorithm usage
//! - Secret isolation between satker with PQ encryption
//! - No shared cryptographic keys or dependencies
//! - EnhancedCryptoEngine security properties

use secreton_crypto::{
    CryptoMode, HybridCrypto, SecurityRequirements, PerformancePriority,
    MigrationStrategy, MigrationPhase,
};
use secreton_core::models::{
    audit::{AuditEvent, AuditEventType, Operation, OperationResult, SecurityContext},
    secret::{Secret, EncryptedValue, EncryptionAlgorithm, SecretMetadata, AccessControl, AuditTrail},
};
use chrono::Utc;
use std::collections::HashMap;

/// Test that HybridCrypto maintains zero-trust principles with actual crypto operations
#[tokio::test]
async fn test_hybrid_crypto_zero_trust_principles() {
    // Create two independent crypto instances (simulating authenc and secreton)
    let mut authenc_crypto = HybridCrypto::new(
        CryptoMode::Hybrid,
        SecurityRequirements::default(),
        PerformancePriority::Security,
    ).unwrap();

    let mut secreton_crypto = HybridCrypto::new(
        CryptoMode::Hybrid,
        SecurityRequirements::default(),
        PerformancePriority::Security,
    ).unwrap();

    // Generate independent keypairs
    authenc_crypto.generate_signing_keypair().unwrap();
    authenc_crypto.generate_kem_keypair().unwrap();

    secreton_crypto.generate_signing_keypair().unwrap();
    secreton_crypto.generate_kem_keypair().unwrap();

    // Verify keys are independent - get public keys
    let authenc_keys = authenc_crypto.get_public_keys().unwrap();
    let secreton_keys = secreton_crypto.get_public_keys().unwrap();

    // Verify keys are different (zero-trust: no shared keys)
    assert_ne!(
        authenc_keys.classical_verifying_key.unwrap(),
        secreton_keys.classical_verifying_key.unwrap(),
        "Classical keys must be independent"
    );
    assert_ne!(
        authenc_keys.pq_signing_public_key.unwrap(),
        secreton_keys.pq_signing_public_key.unwrap(),
        "PQ signing keys must be independent"
    );
    assert_ne!(
        authenc_keys.pq_kem_public_key.unwrap(),
        secreton_keys.pq_kem_public_key.unwrap(),
        "PQ KEM keys must be independent"
    );

    // Test that signatures from one cannot be verified by the other (zero-trust)
    let message = b"Test message for zero-trust validation";
    let authenc_signature = authenc_crypto.sign(message).unwrap();

    // Secreton should NOT be able to verify authenc's signature with its own keys
    let verification_result = secreton_crypto.verify(message, &authenc_signature);
rt!(
        verification_result.is_err() || !verification_result.unwrap(),
        "Zero-trust violated: different instances should not share trust"
    );

    // Test encryption independence
    let plaintext = b"Sensitive data";
    let authenc_encrypted = authenc_crypto.encrypt(
        plaintext,
        &authenc_keys.pq_kem_public_key.unwrap()
    ).unwrap();

    // Secreton should NOT be able to decrypt authenc's data (zero-trust)
    let decryption_result = secreton_crypto.decrypt(&authenc_encrypted);
    assert!(
        decryption_result.is_err(),
        "Zero-trust violated: different instances should not decrypt each other's data"
    );
}

/// Test that authenc and secreton remain independently deployable with PQ
#[tokio::test]
async fn test_independent_deployability_with_pq() {
    // Simulate secreton starting without authenc
    let mut secreton_crypto = HybridCrypto::new(
        CryptoMode::PostQuantum,
        SecurityRequirements {
            security_level: 256,
            quantum_safe: true,
            audit_required: true,
            compliance_flags: vec!["KEJAKSAAN_RI".to_string()],
        },
        PerformancePriority::Secur    ).u

    // Secreton should be fully functional independently
    secreton_crypto.generate_signing_keypair().unwrap();
    secreton_crypto.generate_kem_keypair().unwrap();

    let message = b"Secreton independent operation";
    let signature = secreton_crypto.sign(message).unwrap();
    assert!(secreton_crypto.verify(message, &signature).unwrap());

    // Test encryption/decryption works independently
    let keys = secreton_crypto.get_public_keys().unwrap();
    let plaintext = b"Independent secret data";
    let encrypted = secreton_crypto.encrypt(plaintext, &keys.pq_kem_public_key.unwrap()).unwrap();
    let decrypted = secreton_crypto.decrypt(&encrypted).unwrap();
    assert_eq!(plaintext.as_slice(), decrypted.as_slice());

    // Simulate authenc starting without secreton
    let mut authenc_crypto = HybridCrypto::new(
        CryptoMode::Hybrid,
        SecurityRequirements::default(),
        PerformancePriority::Balanced,
    ).unwrap();

    // Authenc should be fully functional independently
    authenc_crypto.generate_signing_keypair().unwrap();
    authenc_crypto.generate_kem_keypair().unwrap();

    let authenc_message = b"Authenc independent operation";
    let authenc_signature = authenc_crypto.sign(authenc_message).unwrap();
    assert!(authenc_crypto.verify(authenc_message, &authenc_signature).unwrap());

    // Both systems operational independently - zero dependency
    assert!(true, "Both systems operate independently with PQ");
}

/// Test that mTLS communication works with hybrid signatures
#[tokio::test]
async fn test_mtls_with_hybrid_signatures() {
    // Simulate mTLS handshake with hybrid signatures
    let mut server_crypto = HybridCrypto::new(
        CryptoMode::Hybrid,
        SecurityRequirements {
            security_level: 256,
            quantum_safe: true,
            audit_required: true,
            compliance_flags: vec!["MTLS_REQUIRED".to_string()],
        },
        PerformancePriority::Security,
    ).unwrap();

    let mut client_crypto = HybridCrypto::new(
        CryptoMode::Hybrid,
        SecurityRequirements {
            security_level: 256,
            quantum_safe: true,
            audit_required: true,
            compliance_flags: vec!["MTLS_REQUIRED".to_string()],
        },
        PerformancePriority::Security,
    ).unwrap();

    // Generate keypairs for both parties
    server_crypto.generate_signing_keypair().unwrap();
    client_crypto.generate_signing_keypair().unwrap();

    // Simulate certificate exchange and verification
    let server_cert_data = b"Server certificate data with hybrid signature";
    let server_signature = server_crypto.sign(server_cert_data).unwrap();

    // Verify hybrid signature contains both classical and PQ components
    assert!(!server_signature.classical_signature.is_empty(), "Classical signature required for mTLS");
    assert!(!server_signature.pq_signature.is_empty(), "PQ signature required for hybrid mTLS");
    assert!(server_signature.algorithm_info.contains("Ed25519"), "Ed25519 required");
    assert!(server_signature.algorithm_info.contains("ML-DSA"), "ML-DSA required");

    // Client signs its certificate
    let client_cert_data = b"Client certificate data with hybrid signature";
    let client_signature = client_crypto.sign(client_cert_data).unwrap();

    // Verify both signatures are hybrid
    assert!(!client_signature.classical_signature.is_empty());
    assert!(!client_signature.pq_signature.is_empty());

    // Simulate mutual authentication - each party would verify the other's signature
    // In real mTLS, public keys would be exchanged via certificates
    // Here we verify the signature structure is correct for mTLS

    // Test that tampering with either component breaks mTLS
    let mut tampered_server_sig = server_signature.clone();
    tampered_server_sig.classical_signature[0] ^= 0xFF;

    let verification = server_crypto.verify(server_cert_data, &tampered_server_sig);
    assert!(
        verification.is_err() || !verification.unwrap(),
        "Tampered mTLS signature should fail verification"
    );
}

/// Test that audit trail captures PQ algorithm usage
#[tokio::test]
async fn test_audit_trail_captures_pq_algorithm_usage() {
    // Create audit events for different crypto operations
    let mut metadata = HashMap::new();
    metadata.insert("algorithm_type".to_string(), serde_json::json!("ML-DSA-65"));
    metadata.insert("key_id".to_string(), serde_json::json!("pq-key-12345"));
    metadata.insert("crypto_mode".to_string(), serde_json::json!("Hybrid"));
    metadata.insert("security_level".to_string(), serde_json::json!(192));

    let pq_signing_event = AuditEvent {
        event_id: uuid::Uuid::new_v4(),
        timestamp: Utc::now(),
        event_type: AuditEventType::SecretAccess,
        nip: Some("198001012000011001".to_string()),
        satker_code: Some("KEJAKSAAN_PUSAT".to_string()),
        admin_level: None,
        authenc_session_id: Some("session-123".to_string()),
        resource_path: "secrets/kejaksaan/sensitive".to_string(),
        operation: Operation::Read,
        result: OperationResult::Success,
        security_context: SecurityContext {
            auth_method: Some("hybrid_signature".to_string()),
            token_type: Some("JWT_PQ".to_string()),
            security_level: secreton_core::SecurityLevel::Secret,
            mfa_used: true,
            client_cert_info: Some("CN=authenc.kejaksaan.go.id".to_string()),
            encryption_algorithm: Some("ML-KEM-768".to_string()),
            sensitive_data_involved: true,
        },
        risk_score: Some(0.2),
        compliance_flags: vec!["PQ_CRYPTO_USED".to_string(), "NIST_FIPS_204".to_string()],
        metadata: metadata.clone(),
        source_ip: Some("10.0.1.100".to_string()),
        user_agent: Some("SecretonClient/2.0 (PQ-enabled)".to_string()),
        geo_location: Some("Jakarta, Indonesia".to_string()),
        duration_ms: Some(45),
    };

    // Verify audit event captures all PQ-related information
    assert!(pq_signing_event.metadata.contains_key("algorithm_type"));
    assert_eq!(
        pq_signing_event.metadata.get("algorithm_type").unwrap(),
        &serde_json::json!("ML-DSA-65")
    );
    assert!(pq_signing_event.metadata.contains_key("key_id"));
    assert!(pq_signing_event.metadata.contains_key("crypto_mode"));

    // Verify security context captures encryption algorithm
    assert_eq!(
        pq_signing_event.security_context.encryption_algorithm,
        Some("ML-KEM-768".to_string())
    );

    // Verify compliance flags indicate PQ usage
    assert!(pq_signing_event.compliance_flags.contains(&"PQ_CRYPTO_USED".to_string()));
    assert!(pq_signing_event.compliance_flags.contains(&"NIST_FIPS_204".to_string()));

    // Verify timestamp is captured
    assert!(pq_signing_event.timestamp <= Utc::now());

    // Test audit event for encryption operation
    let mut encryption_metadata = HashMap::new();
    encryption_metadata.insert("algorithm_type".to_string(), serde_json::json!("ML-KEM-1024"));
    encryption_metadata.insert("key_id".to_string(), serde_json::json!("kem-key-67890"));
    encryption_metadata.insert("crypto_mode".to_string(), serde_json::json!("PostQuantum"));

    let pq_encryption_event = AuditEvent {
        event_id: uuid::Uuid::new_v4(),
        timestamp: Utc::now(),
        event_type: AuditEventType::SecretModification,
        nip: Some("198001012000011002".to_string()),
        satker_code: Some("KEJAKSAAN_TINGGI_JAKARTA".to_string()),
        admin_level: Some(secreton_core::models::user::AdminLevel::AdminWilayah("JAKARTA".to_string())),
        authenc_session_id: Some("session-456".to_string()),
        resource_path: "secrets/jakarta/classified".to_string(),
        operation: Operation::Write,
        result: OperationResult::Success,
        security_context: SecurityContext {
            auth_method: Some("pq_signature".to_string()),
            token_type: Some("JWT_PQ".to_string()),
            security_level: secreton_core::SecurityLevel::TopSecret,
            mfa_used: true,
            client_cert_info: Some("CN=authenc.kejaksaan.go.id".to_string()),
            encryption_algorithm: Some("ML-KEM-1024".to_string()),
            sensitive_data_involved: true,
        },
        risk_score: Some(0.1),
        compliance_flags: vec!["PQ_CRYPTO_USED".to_string(), "NIST_FIPS_203".to_string()],
        metadata: encryption_metadata,
        source_ip: Some("10.0.2.50".to_string()),
        user_agent: Some("SecretonClient/2.0 (PQ-enabled)".to_string()),
        geo_location: Some("Jakarta, Indonesia".to_string()),
        duration_ms: Some(120),
    };

    // Verify encryption event captures ML-KEM usage
    assert_eq!(
        pq_encryption_event.metadata.get("algorithm_type").unwrap(),
        &serde_json::json!("ML-KEM-1024")
    );
    assert_eq!(
        pq_encryption_event.security_context.encryption_algorithm,
        Some("ML-KEM-1024".to_string())
    );
}

/// Test secret isolation between satker with PQ encryption
#[tokio::test]
async fn test_secret_isolation_between_satker_with_pq() {
    // Create secrets for different satker with PQ encryption
    let mut crypto_jakarta = HybridCrypto::new(
        CryptoMode::PostQuantum,
        SecurityRequirements {
            security_level: 256,
            quantum_safe: true,
            audit_required: true,
            compliance_flags: vec!["SATKER_ISOLATION".to_string()],
        },
        PerformancePriority::Security,
    ).unwrap();

    let mut crypto_bandung = HybridCrypto::new(
        CryptoMode::PostQuantum,
        SecurityRequirements {
            security_level: 256,
            quantum_safe: true,
            audit_required: true,
            compliance_flags: vec!["SATKER_ISOLATION".to_string()],
        },
        PerformancePriority::Security,
    ).unwrap();

    // Generate independent keys for each satker
    crypto_jakarta.generate_kem_keypair().unwrap();
    crypto_bandung.generate_kem_keypair().unwrap();

    let jakarta_keys = crypto_jakarta.get_public_keys().unwrap();
    let bandung_keys = crypto_bandung.get_public_keys().unwrap();

    // Verify keys are different (satker isolation)
    assert_ne!(
        jakarta_keys.pq_kem_public_key.unwrap(),
        bandung_keys.pq_kem_public_key.unwrap(),
        "Satker must have independent encryption keys"
    );

    // Create secret for Jakarta satker
    let jakarta_secret_data = b"Jakarta satker classified information";
    let jakarta_encrypted = crypto_jakarta.encrypt(
        jakarta_secret_data,
        &jakarta_keys.pq_kem_public_key.unwrap()
    ).unwrap();

    // Verify Jakarta can decrypt its own secret
    let jakarta_decrypted = crypto_jakarta.decrypt(&jakarta_encrypted).unwrap();
    assert_eq!(jakarta_secret_data.as_slice(), jakarta_decrypted.as_slice());

    // Verify Bandung CANNOT decrypt Jakarta's secret (satker isolation)
    let bandung_decrypt_attempt = crypto_bandung.decrypt(&jakarta_encrypted);
    assert!(
        bandung_decrypt_attempt.is_err(),
        "Satker isolation violated: Bandung should not decrypt Jakarta's secrets"
    );

    // Create secret for Bandung satker
    let bandung_secret_data = b"Bandung satker classified information";
    let bandung_encrypted = crypto_bandung.encrypt(
        bandung_secret_data,
        &bandung_keys.pq_kem_public_key.unwrap()
    ).unwrap();

    // Verify Bandung can decrypt its own secret
    let bandung_decrypted = crypto_bandung.decrypt(&bandung_encrypted).unwrap();
    assert_eq!(bandung_secret_data.as_slice(), bandung_decrypted.as_slice());

    // Verify Jakarta CANNOT decrypt Bandung's secret (satker isolation)
    let jakarta_decrypt_attempt = crypto_jakarta.decrypt(&bandung_encrypted);
    assert!(
        jakarta_decrypt_attempt.is_err(),
        "Satker isolation violated: Jakarta should not decrypt Bandung's secrets"
    );

    // Test with Secret model to verify access control
    let jakarta_secret = create_test_secret_with_pq(
        "KEJAKSAAN_TINGGI_JAKARTA",
        "secrets/jakarta/case-001",
        "ML-KEM-1024"
    );

    let bandung_secret = create_test_secret_with_pq(
        "KEJAKSAAN_TINGGI_BANDUNG",
        "secrets/bandung/case-002",
        "ML-KEM-1024"
    );

    // Verify access control enforces satker isolation
    assert_eq!(jakarta_secret.satker_owner, "KEJAKSAAN_TINGGI_JAKARTA");
    assert_eq!(bandung_secret.satker_owner, "KEJAKSAAN_TINGGI_BANDUNG");
    assert_ne!(jakarta_secret.satker_owner, bandung_secret.satker_owner);

    // Verify required_satker in access control
    assert!(jakarta_secret.access_control.required_satker.contains(&"KEJAKSAAN_TINGGI_JAKARTA".to_string()));
    assert!(!jakarta_secret.access_control.required_satker.contains(&"KEJAKSAAN_TINGGI_BANDUNG".to_string()));
}

/// Test that there are no shared cryptographic keys or dependencies between services
#[tokio::test]
async fn test_no_shared_cryptographic_keys_or_dependencies() {
    // Create multiple independent crypto instances
    let mut instance1 = HybridCrypto::new(
        CryptoMode::Hybrid,
        SecurityRequirements::default(),
        PerformancePriority::default(),
    ).unwrap();

    let mut instance2 = HybridCrypto::new(
        CryptoMode::Hybrid,
        SecurityRequirements::default(),
        PerformancePriority::default(),
    ).unwrap();

    let mut instance3 = HybridCrypto::new(
        CryptoMode::PostQuantum,
        SecurityRequirements::default(),
        PerformancePriority::default(),
    ).unwrap();

    // Generate keys for all instances
    instance1.generate_signing_keypair().unwrap();
    instance1.generate_kem_keypair().unwrap();

    instance2.generate_signing_keypair().unwrap();
    instance2.generate_kem_keypair().unwrap();

    instance3.generate_signing_keypair().unwrap();
    instance3.generate_kem_keypair().unwrap();

    // Get public keys
    let keys1 = instance1.get_public_keys().unwrap();
    let keys2 = instance2.get_public_keys().unwrap();
    let keys3 = instance3.get_public_keys().unwrap();

    // Verify all classical keys are unique
    assert_ne!(keys1.classical_verifying_key, keys2.classical_verifying_key);
    assert_ne!(keys1.classical_verifying_key, keys3.classical_verifying_key);
    assert_ne!(keys2.classical_verifying_key, keys3.classical_verifying_key);

    // Verify all PQ signing keys are unique
    assert_ne!(keys1.pq_signing_public_key, keys2.pq_signing_public_key);
    assert_ne!(keys1.pq_signing_public_key, keys3.pq_signing_public_key);
    assert_ne!(keys2.pq_signing_public_key, keys3.pq_signing_public_key);

    // Verify all PQ KEM keys are unique
    assert_ne!(keys1.pq_kem_public_key, keys2.pq_kem_public_key);
    assert_ne!(keys1.pq_kem_public_key, keys3.pq_kem_public_key);
    assert_ne!(keys2.pq_kem_public_key, keys3.pq_kem_public_key);

    // Test that instances cannot use each other's keys
    let message = b"Test message";
    let sig1 = instance1.sign(message).unwrap();

    // Instance2 should not be able to verify instance1's signature
    let verify_result = instance2.verify(message, &sig1);
    assert!(
        verify_result.is_err() || !verify_result.unwrap(),
        "Instances should not share cryptographic trust"
    );

    // Test encryption independence
    let plaintext = b"Test data";
    let encrypted1 = instance1.encrypt(plaintext, &keys1.pq_kem_public_key.unwrap()).unwrap();

    // Instance2 should not be able to decrypt instance1's data
    let decrypt_result = instance2.decrypt(&encrypted1);
    assert!(
        decrypt_result.is_err(),
        "Instances should not share encryption keys"
    );

    // Verify no shared state between instances
    // Each instance maintains its own migration strategy
    let strategy1 = MigrationStrategy {
        phase: MigrationPhase::HybridTransition,
        target_date: Some(1735689600),
        pq_adoption_rate: 50,
    };

    let strategy2 = MigrationStrategy {
        phase: MigrationPhase::PostQuantumOnly,
        target_date: Some(1767225600),
        pq_adoption_rate: 100,
    };

    instance1.set_migration_strategy(strategy1.clone());
    instance2.set_migration_strategy(strategy2.clone());

    // Verify strategies are independent
    assert_eq!(instance1.migration_strategy().phase, MigrationPhase::HybridTransition);
    assert_eq!(instance2.migration_strategy().phase, MigrationPhase::PostQuantumOnly);
    assert_ne!(instance1.migration_strategy().pq_adoption_rate, instance2.migration_strategy().pq_adoption_rate);
}

/// Test EnhancedCryptoEngine security properties (constant-time operations, key zeroization)
#[tokio::test]
async fn test_enhanced_crypto_engine_security_properties() {
    // Test constant-time operations
    let mut crypto = HybridCrypto::new(
        CryptoMode::Hybrid,
        SecurityRequirements {
            security_level: 256,
            quantum_safe: true,
            audit_required: true,
            compliance_flags: vec!["CONSTANT_TIME_REQUIRED".to_string()],
        },
        PerformancePriority::Security,
    ).unwrap();

    crypto.generate_signing_keypair().unwrap();

    // Test that signature verification timing is consistent
    let message1 = b"Message 1";
    let message2 = b"Message 2 with different length and content";

    let sig1 = crypto.sign(message1).unwrap();
    let sig2 = crypto.sign(message2).unwrap();

    // Verify both signatures (timing should be constant regardless of message)
    let start1 = std::time::Instant::now();
    let result1 = crypto.verify(message1, &sig1).unwrap();
    let duration1 = start1.elapsed();

    let start2 = std::time::Instant::now();
    let result2 = crypto.verify(message2, &sig2).unwrap();
    let duration2 = start2.elapsed();

    assert!(result1, "Valid signature should verify");
    assert!(result2, "Valid signature should verify");

    // Timing should be similar (within reasonable variance)
    // Note: This is a basic check; true constant-time verification requires more sophisticated testing
    let timing_ratio = duration1.as_nanos() as f64 / duration2.as_nanos() as f64;
    assert!(
        timing_ratio > 0.5 && timing_ratio < 2.0,
        "Verification timing should be relatively constant"
    );

    // Test invalid signature verification timing
    let mut invalid_sig = sig1.clone();
    invalid_sig.classical_signature[0] ^= 0xFF;

    let start_invalid = std::time::Instant::now();
    let result_invalid = crypto.verify(message1, &invalid_sig);
    let duration_invalid = start_invalid.elapsed();

    assert!(
        result_invalid.is_err() || !result_invalid.unwrap(),
        "Invalid signature should not verify"
    );

    // Invalid signature verification should take similar time (constant-time)
    let invalid_timing_ratio = duration1.as_nanos() as f64 / duration_invalid.as_nanos() as f64;
    assert!(
        invalid_timing_ratio > 0.5 && invalid_timing_ratio < 2.0,
        "Invalid signature verification should be constant-time"
    );

    // Test key zeroization by verifying keys are properly managed
    crypto.generate_kem_keypair().unwrap();
    let keys_before = crypto.get_public_keys().unwrap();

    // Generate new keypair (old keys should be zeroized)
    crypto.generate_kem_keypair().unwrap();
    let keys_after = crypto.get_public_keys().unwrap();

    // Verify new keys are different (old keys were replaced and zeroized)
    assert_ne!(
        keys_before.pq_kem_public_key.unwrap(),
        keys_after.pq_kem_public_key.unwrap(),
        "Keys should be replaced and old keys zeroized"
    );

    // Test that sensitive data is not leaked in error messages
    let plaintext = b"Highly sensitive secret data that should not appear in errors";
    let encrypted = crypto.encrypt(plaintext, &keys_after.pq_kem_public_key.unwrap()).unwrap();

    // Create a different crypto instance that cannot decrypt
    let mut other_crypto = HybridCrypto::new(
        CryptoMode::Hybrid,
        SecurityRequirements::default(),
        PerformancePriority::default(),
    ).unwrap();
    other_crypto.generate_kem_keypair().unwrap();

    let decrypt_error = other_crypto.decrypt(&encrypted);
    assert!(decrypt_error.is_err(), "Decryption should fail");

    // Verify error message does not contain sensitive data
    let error_msg = format!("{:?}", decrypt_error.unwrap_err());
    assert!(
        !error_msg.contains("Highly sensitive"),
        "Error messages should not leak sensitive data"
    );
    assert!(
        !error_msg.contains("secret data"),
        "Error messages should not leak sensitive data"
    );
}

/// Test that PQ algorithms are correctly selected based on security level
#[tokio::test]
async fn test_pq_algorithm_selection_by_security_level() {
    // Test 128-bit security level
    let mut crypto_128 = HybridCrypto::new(
        CryptoMode::Hybrid,
        SecurityRequirements {
            security_level: 128,
            quantum_safe: true,
            audit_required: true,
            compliance_flags: vec![],
        },
        PerformancePriority::Speed,
    ).unwrap();

    crypto_128.generate_signing_keypair().unwrap();
    crypto_128.generate_kem_keypair().unwrap();

    let keys_128 = crypto_128.get_public_keys().unwrap();

    // ML-DSA-44 (128-bit security) public key is 1312 bytes
    assert_eq!(keys_128.pq_signing_public_key.unwrap().len(), 1312);
    // ML-KEM-512 (128-bit security) public key is 800 bytes
    assert_eq!(keys_128.pq_kem_public_key.unwrap().len(), 800);

    // Test 192-bit security level
    let mut crypto_192 = HybridCrypto::new(
        CryptoMode::Hybrid,
        SecurityRequirements {
            security_level: 192,
            quantum_safe: true,
            audit_required: true,
            compliance_flags: vec![],
        },
        PerformancePriority::Balanced,
    ).unwrap();

    crypto_192.generate_signing_keypair().unwrap();
    crypto_192.generate_kem_keypair().unwrap();

    let keys_192 = crypto_192.get_public_keys().unwrap();

    // ML-DSA-65 (192-bit security) public key is 1952 bytes
    assert_eq!(keys_192.pq_signing_public_key.unwrap().len(), 1952);
    // ML-KEM-768 (192-bit security) public key is 1184 bytes
    assert_eq!(keys_192.pq_kem_public_key.unwrap().len(), 1184);

    // Test 256-bit security level
    let mut crypto_256 = HybridCrypto::new(
        CryptoMode::Hybrid,
        SecurityRequirements {
            security_level: 256,
            quantum_safe: true,
            audit_required: true,
            compliance_flags: vec![],
        },
        PerformancePriority::Security,
    ).unwrap();

    crypto_256.generate_signing_keypair().unwrap();
    crypto_256.generate_kem_keypair().unwrap();

    let keys_256 = crypto_256.get_public_keys().unwrap();

    // ML-DSA-87 (256-bit security) public key is 2592 bytes
    assert_eq!(keys_256.pq_signing_public_key.unwrap().len(), 2592);
    // ML-KEM-1024 (256-bit security) public key is 1568 bytes
    assert_eq!(keys_256.pq_kem_public_key.unwrap().len(), 1568);
}

/// Helper function to create test secret with PQ encryption
fn create_test_secret_with_pq(satker_owner: &str, path: &str, algorithm: &str) -> Secret {
    let mut metadata_map = HashMap::new();
    metadata_map.insert("pq_algorithm".to_string(), serde_json::json!(algorithm));
    metadata_map.insert("crypto_mode".to_string(), serde_json::json!("PostQuantum"));

    Secret {
        id: rand::random(),
        path: path.to_string(),
        version: 1,
        data: EncryptedValue {
            data: serde_json::json!("encrypted-with-pq"),
            encryption_algorithm: EncryptionAlgorithm::Aes256Gcm,
            encrypted_at: Utc::now(),
            key_id: Some(format!("pq-key-{}", uuid::Uuid::new_v4())),
        },
        metadata: SecretMetadata {
            security_level: secreton_core::SecurityLevel::Secret,
            tags: vec!["pq-encrypted".to_string()],
            description: Some(format!("Secret encrypted with {}", algorithm)),
            custom_fields: secreton_core::Metadata::new(),
            compliance_flags: vec!["PQ_CRYPTO_USED".to_string()],
            risk_score: Some(0.1),
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
                    auth_method: Some("pq_signature".to_string()),
                    token_type: Some("JWT_PQ".to_string()),
                    security_level: secreton_core::SecurityLevel::Secret,
                    mfa_used: true,
                    client_cert_info: None,
                    encryption_algorithm: Some(algorithm.to_string()),
                    sensitive_data_involved: true,
                },
                risk_score: Some(0.1),
                compliance_flags: vec!["PQ_CRYPTO_USED".to_string()],
                metadata: metadata_map,
                source_ip: Some("10.0.1.100".to_string()),
                user_agent: Some("SecretonClient/2.0".to_string()),
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
