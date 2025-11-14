//! Hybrid Cryptography Tests for SIMKARI Secreton
//!
//! This test suite covers:
//! - Hybrid cryptographic system functionality with actual implementations
//! - Post-quantum key management with real ML-KEM and ML-DSA algorithms
//! - Migration strategy validation with actual key transitions
//! - Algorithm selection based on security requirements
//! - Real hybrid encryption and decryption operations
//! - Real hybrid signature generation and verification with Ed25519 + ML-DSA
//!
//! All tests use actual cryptographic implementations - no mocks.

use secreton_crypto::{
    CryptoMode, HybridCrypto, HybridSignatureData, MigrationPhase, MigrationStrategy,
    PerformancePriority, SecurityRequirements,
};

#[test]
fn test_crypto_mode_transitions() {
    // Test valid mode transitions
    let classical = CryptoMode::Classical;
    let hybrid = CryptoMode::Hybrid;
    let post_quantum = CryptoMode::PostQuantum;

    // Test mode properties
    assert_eq!(classical, CryptoMode::Classical);
    assert_eq!(hybrid, CryptoMode::Hybrid);
    assert_eq!(post_quantum, CryptoMode::PostQuantum);

    // Test serialization
    let modes = vec![classical, hybrid, post_quantum];
    for mode in modes {
        let json = serde_json::to_string(&mode).unwrap();
        let deserialized: CryptoMode = serde_json::from_str(&json).unwrap();
        assert_eq!(mode, deserialized);
    }
}

#[test]
fn test_security_requirements_validation() {
    let high_security = SecurityRequirements {
        security_level: 256,
        quantum_safe: true,
        audit_required: true,
        compliance_flags: vec![
            "NIST_PQC".to_string(),
            "FIPS_140_3".to_string(),
            "KEJAKSAAN_SECURITY".to_string(),
        ],
    };

    let balanced_security = SecurityRequirements {
        security_level: 128,
        quantum_safe: false,
        audit_required: false,
        compliance_flags: vec!["FIPS_140_2".to_string()],
    };

    // Test security level validation
    assert!(high_security.security_level >= 256);
    assert!(balanced_security.security_level >= 128);

    // Test quantum resistance
    assert!(high_security.quantum_safe);
    assert!(!balanced_security.quantum_safe);

    // Test compliance requirements
    assert!(
        high_security
            .compliance_flags
            .contains(&"NIST_PQC".to_string())
    );
    assert!(
        high_security
            .compliance_flags
            .contains(&"KEJAKSAAN_SECURITY".to_string())
    );
    assert!(
        !balanced_security
            .compliance_flags
            .contains(&"NIST_PQC".to_string())
    );
}

#[test]
fn test_migration_strategy_phases() {
    let phase1 = MigrationStrategy {
        phase: MigrationPhase::ClassicalOnly,
        target_date: Some(chrono::Utc::now().timestamp() + 90 * 24 * 3600),
        pq_adoption_rate: 0,
    };

    let phase2 = MigrationStrategy {
        phase: MigrationPhase::HybridTransition,
        target_date: Some(chrono::Utc::now().timestamp() + 180 * 24 * 3600),
        pq_adoption_rate: 50,
    };

    let phase3 = MigrationStrategy {
        phase: MigrationPhase::PostQuantumOnly,
        target_date: Some(chrono::Utc::now().timestamp() + 365 * 24 * 3600),
        pq_adoption_rate: 100,
    };

    // Test phase progression
    assert_eq!(phase1.pq_adoption_rate, 0);
    assert_eq!(phase2.pq_adoption_rate, 50);
    assert_eq!(phase3.pq_adoption_rate, 100);

    // Test phase types
    assert_eq!(phase1.phase, MigrationPhase::ClassicalOnly);
    assert_eq!(phase2.phase, MigrationPhase::HybridTransition);
    assert_eq!(phase3.phase, MigrationPhase::PostQuantumOnly);
}

#[test]
fn test_hybrid_crypto_initialization() {
    // Test default initialization
    let crypto = HybridCrypto::new_default().unwrap();
    assert_eq!(crypto.mode(), CryptoMode::Hybrid);
    assert_eq!(crypto.security_requirements().security_level, 192);

    // Test custom initialization
    let custom_crypto = HybridCrypto::new(
        CryptoMode::PostQuantum,
        SecurityRequirements {
            security_level: 256,
            quantum_safe: true,
            audit_required: true,
            compliance_flags: vec!["NIST_PQC".to_string()],
        },
        PerformancePriority::Security,
    )
    .unwrap();

    assert_eq!(custom_crypto.mode(), CryptoMode::PostQuantum);
    assert_eq!(custom_crypto.security_requirements().security_level, 256);
}

#[test]
fn test_classical_mode_signing_with_real_crypto() {
    let mut crypto = HybridCrypto::new(
        CryptoMode::Classical,
        SecurityRequirements::default(),
        PerformancePriority::default(),
    )
    .unwrap();

    crypto.generate_signing_keypair().unwrap();

    let message = b"Test message for Attorney General's Office";
    let signature = crypto.sign(message).unwrap();

    // Verify signature structure
    assert!(!signature.classical_signature.is_empty());
    assert!(signature.pq_signature.is_empty());
    assert_eq!(signature.algorithm_info, "Ed25519");

    // Verify signature is valid
    let is_valid = crypto.verify(message, &signature).unwrap();
    assert!(is_valid);

    // Verify tampered message fails
    let tampered_message = b"Tampered message";
    let is_invalid = crypto.verify(tampered_message, &signature).unwrap();
    assert!(!is_invalid);
}

#[test]
fn test_hybrid_mode_signing_with_real_crypto() {
    let mut crypto = HybridCrypto::new_default().unwrap();
    crypto.generate_signing_keypair().unwrap();

    let message = b"Hybrid signature test for SIMKARI";
    let signature = crypto.sign(message).unwrap();

    // Verify both signatures are present
    assert!(!signature.classical_signature.is_empty());
    assert!(!signature.pq_signature.is_empty());
    assert!(signature.algorithm_info.contains("Ed25519"));
    assert!(signature.algorithm_info.contains("ML-DSA"));

    // Verify signature is valid
    let is_valid = crypto.verify(message, &signature).unwrap();
    assert!(is_valid);

    // Test that tampering with either signature fails verification
    let mut tampered_sig = signature.clone();
    tampered_sig.classical_signature[0] ^= 0xFF;
    let is_invalid = crypto.verify(message, &tampered_sig).unwrap();
    assert!(!is_invalid, "Tampered classical signature should fail");

    let mut tampered_pq = signature.clone();
    tampered_pq.pq_signature[0] ^= 0xFF;
    let is_invalid_pq = crypto.verify(message, &tampered_pq).unwrap();
    assert!(!is_invalid_pq, "Tampered PQ signature should fail");
}

#[test]
fn test_post_quantum_mode_signing_with_real_crypto() {
    let mut crypto = HybridCrypto::new(
        CryptoMode::PostQuantum,
        SecurityRequirements::default(),
        PerformancePriority::default(),
    )
    .unwrap();

    crypto.generate_signing_keypair().unwrap();

    let message = b"Pure PQ signature test";
    let signature = crypto.sign(message).unwrap();

    // Verify only PQ signature is present
    assert!(signature.classical_signature.is_empty());
    assert!(!signature.pq_signature.is_empty());
    assert!(signature.algorithm_info.contains("ML-DSA"));

    // Verify signature is valid
    let is_valid = crypto.verify(message, &signature).unwrap();
    assert!(is_valid);
}

#[test]
fn test_security_level_variant_selection() {
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
    )
    .unwrap();

    crypto_128.generate_signing_keypair().unwrap();
    let message = b"Test 128-bit security";
    let signature = crypto_128.sign(message).unwrap();
    assert!(crypto_128.verify(message, &signature).unwrap());

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
    )
    .unwrap();

    crypto_256.generate_signing_keypair().unwrap();
    let message = b"Test 256-bit security";
    let signature = crypto_256.sign(message).unwrap();
    assert!(crypto_256.verify(message, &signature).unwrap());
}

#[test]
fn test_migration_strategy_configuration() {
    let mut crypto = HybridCrypto::new_default().unwrap();

    let strategy = MigrationStrategy {
        phase: MigrationPhase::HybridTransition,
        target_date: Some(1735689600), // 2025-01-01
        pq_adoption_rate: 75,
    };

    crypto.set_migration_strategy(strategy.clone());
    assert_eq!(
        crypto.migration_strategy().phase,
        MigrationPhase::HybridTransition
    );
    assert_eq!(crypto.migration_strategy().pq_adoption_rate, 75);
    assert_eq!(crypto.migration_strategy().target_date, Some(1735689600));
}

#[test]
fn test_public_keys_export() {
    let mut crypto = HybridCrypto::new_default().unwrap();
    crypto.generate_signing_keypair().unwrap();
    crypto.generate_kem_keypair().unwrap();

    let public_keys = crypto.get_public_keys().unwrap();

    // Verify all keys are present for hybrid mode
    assert!(public_keys.classical_verifying_key.is_some());
    assert!(public_keys.pq_signing_public_key.is_some());
    assert!(public_keys.pq_kem_public_key.is_some());

    // Verify key sizes match expected values
    assert_eq!(public_keys.classical_verifying_key.unwrap().len(), 32); // Ed25519
    assert_eq!(public_keys.pq_signing_public_key.unwrap().len(), 1952); // ML-DSA-65
    assert_eq!(public_keys.pq_kem_public_key.unwrap().len(), 1184); // ML-KEM-768
}

#[test]
fn test_kem_keypair_generation() {
    let mut crypto = HybridCrypto::new_default().unwrap();
    crypto.generate_kem_keypair().unwrap();

    let public_keys = crypto.get_public_keys().unwrap();
    assert!(public_keys.pq_kem_public_key.is_some());

    // Verify ML-KEM-768 key size (192-bit security)
    assert_eq!(public_keys.pq_kem_public_key.unwrap().len(), 1184);
}

#[test]
fn test_different_crypto_modes() {
    for mode in [
        CryptoMode::Classical,
        CryptoMode::Hybrid,
        CryptoMode::PostQuantum,
    ] {
        let mut crypto = HybridCrypto::new(
            mode,
            SecurityRequirements::default(),
            PerformancePriority::default(),
        )
        .unwrap();

        crypto.generate_signing_keypair().unwrap();
        let message = b"Test message for all modes";
        let signature = crypto.sign(message).unwrap();
        let is_valid = crypto.verify(message, &signature).unwrap();

        assert!(
            is_valid,
            "Signature verification failed for mode {:?}",
            mode
        );
    }
}

#[test]
fn test_performance_priorities() {
    for priority in [
        PerformancePriority::Speed,
        PerformancePriority::Balanced,
        PerformancePriority::Security,
    ] {
        let mut crypto = HybridCrypto::new(
            CryptoMode::Hybrid,
            SecurityRequirements {
                security_level: 192,
                quantum_safe: true,
                audit_required: true,
                compliance_flags: vec![],
            },
            priority,
        )
        .unwrap();

        crypto.generate_signing_keypair().unwrap();
        let message = b"Performance priority test";
        let signature = crypto.sign(message).unwrap();
        assert!(crypto.verify(message, &signature).unwrap());
    }
}

#[test]
fn test_hybrid_signature_both_components_required() {
    let mut crypto = HybridCrypto::new_default().unwrap();
    crypto.generate_signing_keypair().unwrap();

    let message = b"Test message";
    let signature = crypto.sign(message).unwrap();

    // Both signatures must be valid for hybrid mode
    assert!(!signature.classical_signature.is_empty());
    assert!(!signature.pq_signature.is_empty());

    // Valid signature should verify
    assert!(crypto.verify(message, &signature).unwrap());

    // Create invalid signature with empty classical component
    let invalid_sig = HybridSignatureData {
        classical_signature: Vec::new(),
        pq_signature: signature.pq_signature.clone(),
        algorithm_info: signature.algorithm_info.clone(),
    };
    assert!(!crypto.verify(message, &invalid_sig).unwrap());

    // Create invalid signature with empty PQ component
    let invalid_sig2 = HybridSignatureData {
        classical_signature: signature.classical_signature.clone(),
        pq_signature: Vec::new(),
        algorithm_info: signature.algorithm_info.clone(),
    };
    assert!(!crypto.verify(message, &invalid_sig2).unwrap());
}

#[test]
fn test_compliance_flags() {
    let crypto = HybridCrypto::new(
        CryptoMode::Hybrid,
        SecurityRequirements {
            security_level: 256,
            quantum_safe: true,
            audit_required: true,
            compliance_flags: vec![
                "KEJAKSAAN_RI".to_string(),
                "NIST_PQC".to_string(),
                "FIPS_203".to_string(),
                "FIPS_204".to_string(),
            ],
        },
        PerformancePriority::Security,
    )
    .unwrap();

    let requirements = crypto.security_requirements();
    assert!(
        requirements
            .compliance_flags
            .contains(&"KEJAKSAAN_RI".to_string())
    );
    assert!(
        requirements
            .compliance_flags
            .contains(&"NIST_PQC".to_string())
    );
    assert!(
        requirements
            .compliance_flags
            .contains(&"FIPS_203".to_string())
    );
    assert!(
        requirements
            .compliance_flags
            .contains(&"FIPS_204".to_string())
    );
}

#[test]
fn test_audit_requirements() {
    let crypto_with_audit = HybridCrypto::new(
        CryptoMode::Hybrid,
        SecurityRequirements {
            security_level: 192,
            quantum_safe: true,
            audit_required: true,
            compliance_flags: vec!["KEJAKSAAN_RI".to_string()],
        },
        PerformancePriority::Balanced,
    )
    .unwrap();

    assert!(crypto_with_audit.security_requirements().audit_required);

    let crypto_without_audit = HybridCrypto::new(
        CryptoMode::Classical,
        SecurityRequirements {
            security_level: 128,
            quantum_safe: false,
            audit_required: false,
            compliance_flags: vec![],
        },
        PerformancePriority::Speed,
    )
    .unwrap();

    assert!(!crypto_without_audit.security_requirements().audit_required);
}

#[test]
fn test_hybrid_encryption_with_real_algorithms() {
    let mut sender = HybridCrypto::new_default().unwrap();
    sender.generate_kem_keypair().unwrap();

    let mut receiver = HybridCrypto::new_default().unwrap();
    receiver.generate_kem_keypair().unwrap();

    // Get receiver's public key
    let receiver_keys = receiver.get_public_keys().unwrap();
    let receiver_public_key = receiver_keys.pq_kem_public_key.unwrap();

    // Encrypt data with hybrid encryption
    let plaintext = b"Sensitive data for Attorney General's Office - SIMKARI";
    let encrypted = sender.encrypt(plaintext, &receiver_public_key).unwrap();

    // Verify encrypted data structure
    assert!(!encrypted.classical_data.ciphertext.is_empty());
    assert!(!encrypted.pq_ciphertext.is_empty());
    assert_eq!(encrypted.metadata.mlkem_variant, "ML-KEM-768");
    assert_eq!(encrypted.metadata.security_level, 192);

    // Decrypt with receiver's private key
    let decrypted = receiver.decrypt(&encrypted).unwrap();
    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_classical_encryption_with_real_aes_gcm() {
    let sender = HybridCrypto::new(
        CryptoMode::Classical,
        SecurityRequirements::default(),
        PerformancePriority::default(),
    )
    .unwrap();

    let plaintext = b"Classical encryption test data";
    let dummy_public_key = vec![0u8; 32]; // Not used in classical mode

    let encrypted = sender.encrypt(plaintext, &dummy_public_key).unwrap();

    // Verify classical encryption structure
    assert!(!encrypted.classical_data.ciphertext.is_empty());
    assert!(encrypted.pq_ciphertext.is_empty());
    assert_eq!(encrypted.metadata.mlkem_variant, "None");
    assert_eq!(encrypted.metadata.security_level, 256);
}

#[test]
fn test_post_quantum_encryption_with_real_mlkem() {
    let mut sender = HybridCrypto::new(
        CryptoMode::PostQuantum,
        SecurityRequirements::default(),
        PerformancePriority::default(),
    )
    .unwrap();
    sender.generate_kem_keypair().unwrap();

    let mut receiver = HybridCrypto::new(
        CryptoMode::PostQuantum,
        SecurityRequirements::default(),
        PerformancePriority::default(),
    )
    .unwrap();
    receiver.generate_kem_keypair().unwrap();

    let receiver_keys = receiver.get_public_keys().unwrap();
    let receiver_public_key = receiver_keys.pq_kem_public_key.unwrap();

    let plaintext = b"Pure post-quantum encrypted data";
    let encrypted = sender.encrypt(plaintext, &receiver_public_key).unwrap();

    // Verify PQ encryption structure
    assert!(!encrypted.classical_data.ciphertext.is_empty());
    assert!(!encrypted.pq_ciphertext.is_empty());
    assert!(encrypted.metadata.mlkem_variant.contains("ML-KEM"));

    // Decrypt and verify
    let decrypted = receiver.decrypt(&encrypted).unwrap();
    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_migration_strategy_execution_phase_transitions() {
    // Phase 1: Classical Only
    let mut crypto_phase1 = HybridCrypto::new(
        CryptoMode::Classical,
        SecurityRequirements::default(),
        PerformancePriority::default(),
    )
    .unwrap();

    let strategy_phase1 = MigrationStrategy {
        phase: MigrationPhase::ClassicalOnly,
        target_date: Some(chrono::Utc::now().timestamp() + 90 * 24 * 3600),
        pq_adoption_rate: 0,
    };
    crypto_phase1.set_migration_strategy(strategy_phase1);

    crypto_phase1.generate_signing_keypair().unwrap();
    let message = b"Phase 1 message";
    let sig_phase1 = crypto_phase1.sign(message).unwrap();

    // Verify classical-only signature
    assert!(!sig_phase1.classical_signature.is_empty());
    assert!(sig_phase1.pq_signature.is_empty());
    assert!(crypto_phase1.verify(message, &sig_phase1).unwrap());

    // Phase 2: Hybrid Transition
    let mut crypto_phase2 = HybridCrypto::new(
        CryptoMode::Hybrid,
        SecurityRequirements::default(),
        PerformancePriority::default(),
    )
    .unwrap();

    let strategy_phase2 = MigrationStrategy {
        phase: MigrationPhase::HybridTransition,
        target_date: Some(chrono::Utc::now().timestamp() + 180 * 24 * 3600),
        pq_adoption_rate: 50,
    };
    crypto_phase2.set_migration_strategy(strategy_phase2);

    crypto_phase2.generate_signing_keypair().unwrap();
    let sig_phase2 = crypto_phase2.sign(message).unwrap();

    // Verify hybrid signature with both components
    assert!(!sig_phase2.classical_signature.is_empty());
    assert!(!sig_phase2.pq_signature.is_empty());
    assert!(crypto_phase2.verify(message, &sig_phase2).unwrap());

    // Phase 3: Post-Quantum Only
    let mut crypto_phase3 = HybridCrypto::new(
        CryptoMode::PostQuantum,
        SecurityRequirements::default(),
        PerformancePriority::default(),
    )
    .unwrap();

    let strategy_phase3 = MigrationStrategy {
        phase: MigrationPhase::PostQuantumOnly,
        target_date: Some(chrono::Utc::now().timestamp() + 365 * 24 * 3600),
        pq_adoption_rate: 100,
    };
    crypto_phase3.set_migration_strategy(strategy_phase3);

    crypto_phase3.generate_signing_keypair().unwrap();
    let sig_phase3 = crypto_phase3.sign(message).unwrap();

    // Verify PQ-only signature
    assert!(sig_phase3.classical_signature.is_empty());
    assert!(!sig_phase3.pq_signature.is_empty());
    assert!(crypto_phase3.verify(message, &sig_phase3).unwrap());
}

#[test]
fn test_algorithm_selection_based_on_security_requirements() {
    // Test ML-DSA-44 selection for 128-bit security
    let mut crypto_128 = HybridCrypto::new(
        CryptoMode::Hybrid,
        SecurityRequirements {
            security_level: 128,
            quantum_safe: true,
            audit_required: true,
            compliance_flags: vec![],
        },
        PerformancePriority::Speed,
    )
    .unwrap();
    crypto_128.generate_signing_keypair().unwrap();
    crypto_128.generate_kem_keypair().unwrap();

    let keys_128 = crypto_128.get_public_keys().unwrap();
    // ML-DSA-44 public key size is 1312 bytes
    assert_eq!(keys_128.pq_signing_public_key.unwrap().len(), 1312);
    // ML-KEM-512 public key size is 800 bytes
    assert_eq!(keys_128.pq_kem_public_key.unwrap().len(), 800);

    // Test ML-DSA-65 selection for 192-bit security (default)
    let mut crypto_192 = HybridCrypto::new(
        CryptoMode::Hybrid,
        SecurityRequirements {
            security_level: 192,
            quantum_safe: true,
            audit_required: true,
            compliance_flags: vec![],
        },
        PerformancePriority::Balanced,
    )
    .unwrap();
    crypto_192.generate_signing_keypair().unwrap();
    crypto_192.generate_kem_keypair().unwrap();

    let keys_192 = crypto_192.get_public_keys().unwrap();
    // ML-DSA-65 public key size is 1952 bytes
    assert_eq!(keys_192.pq_signing_public_key.unwrap().len(), 1952);
    // ML-KEM-768 public key size is 1184 bytes
    assert_eq!(keys_192.pq_kem_public_key.unwrap().len(), 1184);

    // Test ML-DSA-87 selection for 256-bit security
    let mut crypto_256 = HybridCrypto::new(
        CryptoMode::Hybrid,
        SecurityRequirements {
            security_level: 256,
            quantum_safe: true,
            audit_required: true,
            compliance_flags: vec![],
        },
        PerformancePriority::Security,
    )
    .unwrap();
    crypto_256.generate_signing_keypair().unwrap();
    crypto_256.generate_kem_keypair().unwrap();

    let keys_256 = crypto_256.get_public_keys().unwrap();
    // ML-DSA-87 public key size is 2592 bytes
    assert_eq!(keys_256.pq_signing_public_key.unwrap().len(), 2592);
    // ML-KEM-1024 public key size is 1568 bytes
    assert_eq!(keys_256.pq_kem_public_key.unwrap().len(), 1568);
}

#[test]
fn test_real_key_transitions_during_migration() {
    // Start with classical keys
    let mut crypto = HybridCrypto::new(
        CryptoMode::Classical,
        SecurityRequirements::default(),
        PerformancePriority::default(),
    )
    .unwrap();
    crypto.generate_signing_keypair().unwrap();

    let message = b"Migration test message";
    let classical_sig = crypto.sign(message).unwrap();
    assert!(crypto.verify(message, &classical_sig).unwrap());

    // Transition to hybrid mode - generate PQ keys
    let mut crypto_hybrid = HybridCrypto::new(
        CryptoMode::Hybrid,
        SecurityRequirements::default(),
        PerformancePriority::default(),
    )
    .unwrap();
    crypto_hybrid.generate_signing_keypair().unwrap();

    let hybrid_sig = crypto_hybrid.sign(message).unwrap();
    assert!(crypto_hybrid.verify(message, &hybrid_sig).unwrap());

    // Verify hybrid signature has both components
    assert!(!hybrid_sig.classical_signature.is_empty());
    assert!(!hybrid_sig.pq_signature.is_empty());

    // Transition to pure PQ mode
    let mut crypto_pq = HybridCrypto::new(
        CryptoMode::PostQuantum,
        SecurityRequirements::default(),
        PerformancePriority::default(),
    )
    .unwrap();
    crypto_pq.generate_signing_keypair().unwrap();

    let pq_sig = crypto_pq.sign(message).unwrap();
    assert!(crypto_pq.verify(message, &pq_sig).unwrap());

    // Verify PQ signature has only PQ component
    assert!(pq_sig.classical_signature.is_empty());
    assert!(!pq_sig.pq_signature.is_empty());
}

#[test]
fn test_hybrid_signature_verification_with_ed25519_and_mldsa() {
    let mut crypto = HybridCrypto::new_default().unwrap();
    crypto.generate_signing_keypair().unwrap();

    let message = b"Test Ed25519 + ML-DSA hybrid signature";
    let signature = crypto.sign(message).unwrap();

    // Verify signature structure
    assert_eq!(signature.classical_signature.len(), 64); // Ed25519 signature is 64 bytes
    assert!(!signature.pq_signature.is_empty()); // ML-DSA signature varies by variant
    assert!(signature.algorithm_info.contains("Ed25519"));
    assert!(signature.algorithm_info.contains("ML-DSA"));

    // Verify signature is valid
    assert!(crypto.verify(message, &signature).unwrap());

    // Verify tampering detection on Ed25519 component
    let mut tampered_ed25519 = signature.clone();
    tampered_ed25519.classical_signature[0] ^= 0xFF;
    assert!(!crypto.verify(message, &tampered_ed25519).unwrap());

    // Verify tampering detection on ML-DSA component
    let mut tampered_mldsa = signature.clone();
    tampered_mldsa.pq_signature[0] ^= 0xFF;
    assert!(!crypto.verify(message, &tampered_mldsa).unwrap());

    // Verify wrong message detection
    let wrong_message = b"Different message";
    assert!(!crypto.verify(wrong_message, &signature).unwrap());
}

#[test]
fn test_encryption_metadata_accuracy() {
    let mut crypto = HybridCrypto::new(
        CryptoMode::Hybrid,
        SecurityRequirements {
            security_level: 256,
            quantum_safe: true,
            audit_required: true,
            compliance_flags: vec!["KEJAKSAAN_RI".to_string()],
        },
        PerformancePriority::Security,
    )
    .unwrap();
    crypto.generate_kem_keypair().unwrap();

    let receiver_keys = crypto.get_public_keys().unwrap();
    let receiver_public_key = receiver_keys.pq_kem_public_key.unwrap();

    let plaintext = b"Test metadata";
    let encrypted = crypto.encrypt(plaintext, &receiver_public_key).unwrap();

    // Verify metadata
    assert_eq!(encrypted.metadata.mlkem_variant, "ML-KEM-1024");
    assert_eq!(encrypted.metadata.security_level, 256);
    assert!(encrypted.metadata.timestamp > 0);
    assert!(encrypted.metadata.timestamp <= chrono::Utc::now().timestamp());
}

#[test]
fn test_performance_priority_affects_algorithm_selection() {
    // Speed priority should select faster variants
    let mut crypto_speed = HybridCrypto::new(
        CryptoMode::Hybrid,
        SecurityRequirements {
            security_level: 128,
            quantum_safe: true,
            audit_required: false,
            compliance_flags: vec![],
        },
        PerformancePriority::Speed,
    )
    .unwrap();
    crypto_speed.generate_signing_keypair().unwrap();
    crypto_speed.generate_kem_keypair().unwrap();

    let keys_speed = crypto_speed.get_public_keys().unwrap();
    // Should use ML-DSA-44 and ML-KEM-512 for speed
    assert_eq!(keys_speed.pq_signing_public_key.unwrap().len(), 1312);
    assert_eq!(keys_speed.pq_kem_public_key.unwrap().len(), 800);

    // Security priority should select stronger variants
    let mut crypto_security = HybridCrypto::new(
        CryptoMode::Hybrid,
        SecurityRequirements {
            security_level: 256,
            quantum_safe: true,
            audit_required: true,
            compliance_flags: vec![],
        },
        PerformancePriority::Security,
    )
    .unwrap();
    crypto_security.generate_signing_keypair().unwrap();
    crypto_security.generate_kem_keypair().unwrap();

    let keys_security = crypto_security.get_public_keys().unwrap();
    // Should use ML-DSA-87 and ML-KEM-1024 for security
    assert_eq!(keys_security.pq_signing_public_key.unwrap().len(), 2592);
    assert_eq!(keys_security.pq_kem_public_key.unwrap().len(), 1568);
}

#[test]
fn test_cross_mode_signature_compatibility() {
    // Generate signatures in different modes
    let mut classical = HybridCrypto::new(
        CryptoMode::Classical,
        SecurityRequirements::default(),
        PerformancePriority::default(),
    )
    .unwrap();
    classical.generate_signing_keypair().unwrap();

    let mut hybrid = HybridCrypto::new(
        CryptoMode::Hybrid,
        SecurityRequirements::default(),
        PerformancePriority::default(),
    )
    .unwrap();
    hybrid.generate_signing_keypair().unwrap();

    let mut pq = HybridCrypto::new(
        CryptoMode::PostQuantum,
        SecurityRequirements::default(),
        PerformancePriority::default(),
    )
    .unwrap();
    pq.generate_signing_keypair().unwrap();

    let message = b"Cross-mode test";

    // Each mode should produce valid signatures
    let classical_sig = classical.sign(message).unwrap();
    assert!(classical.verify(message, &classical_sig).unwrap());

    let hybrid_sig = hybrid.sign(message).unwrap();
    assert!(hybrid.verify(message, &hybrid_sig).unwrap());

    let pq_sig = pq.sign(message).unwrap();
    assert!(pq.verify(message, &pq_sig).unwrap());

    // Verify signature structures are mode-appropriate
    assert!(!classical_sig.classical_signature.is_empty());
    assert!(classical_sig.pq_signature.is_empty());

    assert!(!hybrid_sig.classical_signature.is_empty());
    assert!(!hybrid_sig.pq_signature.is_empty());

    assert!(pq_sig.classical_signature.is_empty());
    assert!(!pq_sig.pq_signature.is_empty());
}
