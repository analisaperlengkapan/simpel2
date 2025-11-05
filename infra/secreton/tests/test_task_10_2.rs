//! Simple test to verify Task 10.2 implementation
//! Tests the new integration test components

use chrono::Utc;
use secreton_core::auth::{AuthencAuthProvider, PqSignature};
use secreton_core::models::secret::{
    AccessControl, EncryptedValue, EncryptionAlgorithm, Secret, SecretMetadata,
};
use secreton_core::SecurityLevel;
use secreton_crypto::{CryptoMode, HybridCrypto, PerformancePriority, SecurityRequirements};

#[test]
fn test_authenc_provider_can_be_created() {
    // Test that AuthencAuthProvider can be instantiated
    let provider =
        AuthencAuthProvider::new("https://authenc.test.kejaksaan.go.id".to_string(), None);

    // Verify it was created
    assert!(format!("{:?}", provider).contains("AuthencAuthProvider"));
    println!("✓ AuthencAuthProvider created successfully");
}

#[test]
fn test_hybrid_crypto_can_encrypt_decrypt() {
    // Test that HybridCrypto works for secret encryption
    let security_reqs = SecurityRequirements {
        security_level: 256,
        quantum_safe: true,
        audit_required: true,
        compliance_flags: vec!["KEJAKSAAN_SECURITY".to_string()],
    };

    let perf_priority = PerformancePriority::Balanced;
    let hybrid_crypto = HybridCrypto::new(CryptoMode::Hybrid, security_reqs, perf_priority)
        .expect("Failed to create HybridCrypto");

    let secret_data = b"test secret data";
    let associated_data = b"satker:KEJATI_DKI_JAKPUS";

    // Encrypt
    let encrypted = hybrid_crypto
        .encrypt(secret_data, associated_data)
        .expect("Encryption should succeed");

    // Verify metadata exists
    assert!(encrypted.metadata.security_level > 0);

    // Decrypt
    let decrypted = hybrid_crypto
        .decrypt(&encrypted)
        .expect("Decryption should succeed");

    assert_eq!(decrypted, secret_data);
    println!("✓ Hybrid encryption/decryption works");
}

#[test]
fn test_secret_struct_can_be_created() {
    // Test that Secret struct can be instantiated with all required fields
    let secret = Secret {
        id: 1,
        path: "secrets/KEJATI_DKI_JAKPUS/test".to_string(),
        version: 1,
        data: EncryptedValue {
            data: serde_json::json!({"value": "encrypted_data"}),
            encryption_algorithm: EncryptionAlgorithm::Aes256Gcm,
            encrypted_at: Utc::now(),
            key_id: Some("test_key".to_string()),
        },
        metadata: SecretMetadata {
            security_level: SecurityLevel::Secret,
            tags: vec!["test".to_string()],
            description: Some("Test secret".to_string()),
            custom_fields: Default::default(),
            compliance_flags: vec!["KEJAKSAAN_SECURITY".to_string()],
            risk_score: Some(0.5),
        },
        access_control: AccessControl {
            required_roles: vec!["SecretonUser".to_string()],
            required_satker: vec!["KEJATI_DKI_JAKPUS".to_string()],
            nip_whitelist: None,
            nip_blacklist: None,
            time_based_access: None,
            audit_required: true,
            admin_level_required: None,
        },
        audit_trail: Default::default(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        created_by_nip: Some("198001012000011001".to_string()),
        satker_owner: "KEJATI_DKI_JAKPUS".to_string(),
        last_accessed: Utc::now(),
        namespace: "test".to_string(),
    };

    assert_eq!(secret.satker_owner, "KEJATI_DKI_JAKPUS");
    assert_eq!(secret.path, "secrets/KEJATI_DKI_JAKPUS/test");
    println!("✓ Secret struct created successfully");
}

#[test]
fn test_pq_signature_types_exist() {
    // Test that PQ signature types can be created
    let ml_dsa_sig = PqSignature::MlDsa {
        signature: vec![1, 2, 3, 4],
        public_key: vec![5, 6, 7, 8],
    };

    assert!(matches!(ml_dsa_sig, PqSignature::MlDsa { .. }));
    println!("✓ PQ signature types work");
}
