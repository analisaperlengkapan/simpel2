//! Post-Quantum Cryptography Readiness Validation Tests
//!
//! Tests that exercise REAL post-quantum cryptographic operations using:
//! - ML-DSA-44 (digital signatures via pqcrypto-mldsa)
//! - ML-KEM-768 (key encapsulation via pqcrypto-mlkem)
//! - Falcon-512 (compact signatures via pqcrypto-falcon)
//! - Hybrid Classical + PQ (Ed25519 + ML-DSA, X25519 + ML-KEM)
//!
//! Run with: cargo test -p authenc --test post_quantum_readiness_validation --features quantum

#![cfg(feature = "quantum")]

use authenc::crypto::pqc::{PqcError, falcon, hybrid, mldsa, mlkem};

// ============================================================================
// ML-DSA-44 (CRYSTALS-Dilithium) Tests
// ============================================================================

#[test]
fn test_mldsa_keypair_generation() {
    let (pk, _sk) = mldsa::SecretKey::new().expect("ML-DSA keygen should succeed");
    let pk_bytes = pk.as_bytes();
    // ML-DSA-44 public key = 1312 bytes
    assert_eq!(pk_bytes.len(), 1312, "ML-DSA-44 public key should be 1312 bytes");
}

#[test]
fn test_mldsa_sign_verify_roundtrip() {
    let (pk, sk) = mldsa::SecretKey::new().expect("keygen");
    let message = b"SIMPelv2 Post-Quantum signature test";

    let signature = sk.sign(message).expect("signing should succeed");
    assert!(
        pk.verify(message, &signature).is_ok(),
        "ML-DSA signature verification should succeed"
    );
}

#[test]
fn test_mldsa_verify_wrong_message_fails() {
    let (pk, sk) = mldsa::SecretKey::new().expect("keygen");
    let message = b"Original message";
    let tampered = b"Tampered message";

    let signature = sk.sign(message).expect("signing");
    assert!(
        pk.verify(tampered, &signature).is_err(),
        "Verification with wrong message should fail"
    );
}

#[test]
fn test_mldsa_verify_wrong_key_fails() {
    let (_pk1, sk1) = mldsa::SecretKey::new().expect("keygen 1");
    let (pk2, _sk2) = mldsa::SecretKey::new().expect("keygen 2");
    let message = b"Key mismatch test";

    let signature = sk1.sign(message).expect("signing");
    assert!(
        pk2.verify(message, &signature).is_err(),
        "Verification with wrong public key should fail"
    );
}

#[test]
fn test_mldsa_key_sizes() {
    let (pk_size, sk_size, sig_size) = mldsa::key_sizes();
    assert_eq!(pk_size, 1312, "ML-DSA-44 public key size");
    assert_eq!(sk_size, 2560, "ML-DSA-44 secret key size");
    assert_eq!(sig_size, 2420, "ML-DSA-44 signature size");
}

#[test]
fn test_mldsa_key_serialization_roundtrip() {
    let (pk, sk) = mldsa::SecretKey::new().expect("keygen");
    let message = b"Serialization roundtrip test";

    // Serialize keys
    let pk_bytes = pk.as_bytes().to_vec();
    let sk_bytes = sk.as_bytes(); // Vec<u8>

    // Deserialize keys (SecretKey requires both sk + pk bytes)
    let pk_restored = mldsa::PublicKey::from_bytes(&pk_bytes)
        .expect("Public key deserialization should succeed");
    let sk_restored = mldsa::SecretKey::from_bytes_with_public(&sk_bytes, &pk_bytes)
        .expect("Secret key deserialization should succeed");

    // Sign with restored key, verify with restored key
    let signature = sk_restored.sign(message).expect("signing with restored key");
    assert!(
        pk_restored.verify(message, &signature).is_ok(),
        "Signature with deserialized keys should verify"
    );
}

#[test]
fn test_mldsa_signature_serialization() {
    let (pk, sk) = mldsa::SecretKey::new().expect("keygen");
    let message = b"Signature serialization test";

    let signature = sk.sign(message).expect("signing");
    let sig_bytes = signature.as_bytes().to_vec();

    // Verify expected signature size
    assert_eq!(sig_bytes.len(), 2420, "ML-DSA-44 signature should be 2420 bytes");

    // Deserialize and verify
    let sig_restored = mldsa::Signature::from_bytes(&sig_bytes)
        .expect("Signature deserialization should succeed");
    assert!(
        pk.verify(message, &sig_restored).is_ok(),
        "Deserialized signature should verify"
    );
}

// ============================================================================
// ML-KEM-768 (CRYSTALS-Kyber) Tests
// ============================================================================

#[test]
fn test_mlkem_keypair_generation() {
    let (pk, _sk) = mlkem::SecretKey::new().expect("ML-KEM keygen should succeed");
    let pk_bytes = pk.as_bytes();
    // ML-KEM-768 public key = 1184 bytes
    assert_eq!(pk_bytes.len(), 1184, "ML-KEM-768 public key should be 1184 bytes");
}

#[test]
fn test_mlkem_encapsulate_decapsulate_roundtrip() {
    let (pk, sk) = mlkem::SecretKey::new().expect("keygen");

    let (ct, ss_enc) = pk.encapsulate().expect("encapsulation should succeed");
    let ss_dec = sk.decapsulate(&ct).expect("decapsulation should succeed");

    assert_eq!(
        ss_enc.as_bytes(),
        ss_dec.as_bytes(),
        "Shared secrets from encapsulation and decapsulation must match"
    );
}

#[test]
fn test_mlkem_wrong_key_decapsulation() {
    let (pk, _sk1) = mlkem::SecretKey::new().expect("keygen 1");
    let (_pk2, sk2) = mlkem::SecretKey::new().expect("keygen 2");

    let (ct, ss_enc) = pk.encapsulate().expect("encapsulation");
    // ML-KEM decapsulation with wrong key produces different (implicit reject) shared secret
    let ss_dec = sk2.decapsulate(&ct).expect("decapsulation succeeds but with wrong secret");
    assert_ne!(
        ss_enc.as_bytes(),
        ss_dec.as_bytes(),
        "Different key should produce different shared secret (implicit reject)"
    );
}

#[test]
fn test_mlkem_key_sizes() {
    let (pk_size, sk_size, ct_size, ss_size) = mlkem::key_sizes();
    assert_eq!(pk_size, 1184, "ML-KEM-768 public key size");
    assert_eq!(sk_size, 2400, "ML-KEM-768 secret key size");
    assert_eq!(ct_size, 1088, "ML-KEM-768 ciphertext size");
    assert_eq!(ss_size, 32, "ML-KEM-768 shared secret size");
}

#[test]
fn test_mlkem_shared_secret_is_32_bytes() {
    let (pk, sk) = mlkem::SecretKey::new().expect("keygen");
    let (ct, ss_enc) = pk.encapsulate().expect("encapsulation");
    let ss_dec = sk.decapsulate(&ct).expect("decapsulation");

    assert_eq!(ss_enc.as_bytes().len(), 32, "Shared secret should be 32 bytes");
    assert_eq!(ss_dec.as_bytes().len(), 32, "Decapsulated shared secret should be 32 bytes");
}

#[test]
fn test_mlkem_key_serialization_roundtrip() {
    let (pk, sk) = mlkem::SecretKey::new().expect("keygen");

    let pk_bytes = pk.as_bytes().to_vec();
    let sk_bytes = sk.as_bytes(); // Vec<u8>

    let pk_restored = mlkem::PublicKey::from_bytes(&pk_bytes)
        .expect("Public key deserialization should succeed");
    let sk_restored = mlkem::SecretKey::from_bytes_with_public(&sk_bytes, &pk_bytes)
        .expect("Secret key deserialization should succeed");

    let (ct, ss_enc) = pk_restored.encapsulate().expect("encapsulation");
    let ss_dec = sk_restored.decapsulate(&ct).expect("decapsulation");

    assert_eq!(
        ss_enc.as_bytes(),
        ss_dec.as_bytes(),
        "Shared secrets with deserialized keys must match"
    );
}

#[test]
fn test_mlkem_ciphertext_serialization() {
    let (pk, sk) = mlkem::SecretKey::new().expect("keygen");

    let (ct, ss_enc) = pk.encapsulate().expect("encapsulation");
    let ct_bytes = ct.as_bytes().to_vec();

    // Verify expected ciphertext size
    assert_eq!(ct_bytes.len(), 1088, "ML-KEM-768 ciphertext should be 1088 bytes");

    // Deserialize ciphertext and decapsulate
    let ct_restored = mlkem::Ciphertext::from_bytes(&ct_bytes)
        .expect("Ciphertext deserialization should succeed");
    let ss_dec = sk.decapsulate(&ct_restored).expect("decapsulation with restored ciphertext");

    assert_eq!(
        ss_enc.as_bytes(),
        ss_dec.as_bytes(),
        "Decapsulation with deserialized ciphertext must produce same shared secret"
    );
}

// ============================================================================
// Falcon-512 Tests
// ============================================================================

#[test]
fn test_falcon_keypair_generation() {
    let (pk, _sk) = falcon::SecretKey::new().expect("Falcon keygen should succeed");
    let pk_bytes = pk.as_bytes();
    // Falcon-512 public key = 897 bytes
    assert_eq!(pk_bytes.len(), 897, "Falcon-512 public key should be 897 bytes");
}

#[test]
fn test_falcon_sign_verify_roundtrip() {
    let (pk, sk) = falcon::SecretKey::new().expect("keygen");
    let message = b"Falcon-512 signature verification";

    let signature = sk.sign(message).expect("signing should succeed");
    assert!(
        pk.verify(message, &signature).is_ok(),
        "Falcon signature verification should succeed"
    );
}

#[test]
fn test_falcon_verify_wrong_message_fails() {
    let (pk, sk) = falcon::SecretKey::new().expect("keygen");
    let message = b"Correct message";
    let tampered = b"Tampered message";

    let signature = sk.sign(message).expect("signing");
    assert!(
        pk.verify(tampered, &signature).is_err(),
        "Falcon verification with wrong message should fail"
    );
}

#[test]
fn test_falcon_key_sizes() {
    let (pk_size, sk_size, sig_size) = falcon::key_sizes();
    assert_eq!(pk_size, 897, "Falcon-512 public key size");
    assert!(sk_size > 0, "Falcon secret key should have non-zero size");
    assert!(sig_size > 0, "Falcon signature should have non-zero size");
}

// ============================================================================
// Hybrid Crypto Tests (Classical + PQ)
// ============================================================================

#[test]
fn test_hybrid_encrypt_decrypt_roundtrip() {
    let (pk, sk) = mlkem::SecretKey::new().expect("keygen");
    let plaintext = b"Hybrid encryption test for SIMPelv2 government BMN data";

    let (ct, encrypted, nonce) = hybrid::encrypt_hybrid(&pk, plaintext)
        .expect("Hybrid encryption should succeed");

    let decrypted = hybrid::decrypt_hybrid(&sk, &ct, &encrypted, &nonce)
        .expect("Hybrid decryption should succeed");

    assert_eq!(
        plaintext.as_slice(),
        decrypted.as_slice(),
        "Decrypted data must match original plaintext"
    );
}

#[test]
fn test_hybrid_encrypt_various_sizes() {
    let (pk, sk) = mlkem::SecretKey::new().expect("keygen");

    // Test with various payload sizes
    let test_cases: Vec<Vec<u8>> = vec![
        vec![0u8; 0],       // empty
        vec![42u8; 1],      // 1 byte
        vec![0xAB; 16],     // 16 bytes (AES block)
        vec![0xCD; 256],    // 256 bytes
        vec![0xEF; 4096],   // 4 KB
    ];

    for plaintext in &test_cases {
        let (ct, encrypted, nonce) = hybrid::encrypt_hybrid(&pk, plaintext)
            .expect("Encryption should succeed for all sizes");

        let decrypted = hybrid::decrypt_hybrid(&sk, &ct, &encrypted, &nonce)
            .expect("Decryption should succeed for all sizes");

        assert_eq!(
            plaintext.as_slice(),
            decrypted.as_slice(),
            "Roundtrip failed for payload size {}",
            plaintext.len()
        );
    }
}

#[test]
fn test_hybrid_sign_verify_roundtrip() {
    use ed25519_dalek::SigningKey;

    // Generate Ed25519 (classical) keypair
    let ed_sk = SigningKey::generate(&mut rand::rngs::OsRng);
    let ed_pk = ed25519_dalek::VerifyingKey::from(&ed_sk);

    // Generate ML-DSA (post-quantum) keypair
    let (ml_pk, ml_sk) = mldsa::SecretKey::new().expect("ML-DSA keygen");

    let message = b"Hybrid signature: Ed25519 + ML-DSA for government data integrity";

    let (ed_sig, ml_sig) = hybrid::sign_hybrid(message, &ed_sk, &ml_sk)
        .expect("Hybrid signing should succeed");

    assert!(
        hybrid::verify_hybrid(message, &ed_sig, &ml_sig, &ed_pk, &ml_pk).is_ok(),
        "Hybrid signature verification should succeed"
    );
}

#[test]
fn test_hybrid_sign_wrong_message_fails() {
    use ed25519_dalek::SigningKey;

    let ed_sk = SigningKey::generate(&mut rand::rngs::OsRng);
    let ed_pk = ed25519_dalek::VerifyingKey::from(&ed_sk);
    let (ml_pk, ml_sk) = mldsa::SecretKey::new().expect("keygen");

    let message = b"Original";
    let tampered = b"Tampered";

    let (ed_sig, ml_sig) = hybrid::sign_hybrid(message, &ed_sk, &ml_sk)
        .expect("Signing should succeed");

    assert!(
        hybrid::verify_hybrid(tampered, &ed_sig, &ml_sig, &ed_pk, &ml_pk).is_err(),
        "Hybrid verification with wrong message should fail"
    );
}

#[test]
fn test_hybrid_decrypt_with_wrong_key_fails() {
    let (pk1, _sk1) = mlkem::SecretKey::new().expect("keygen 1");
    let (_pk2, sk2) = mlkem::SecretKey::new().expect("keygen 2");
    let plaintext = b"Confidential BMN asset inventory";

    let (ct, encrypted, nonce) = hybrid::encrypt_hybrid(&pk1, plaintext)
        .expect("Encryption should succeed");

    // Decryption with wrong key should fail (AES-GCM tag mismatch)
    let result = hybrid::decrypt_hybrid(&sk2, &ct, &encrypted, &nonce);
    assert!(
        result.is_err(),
        "Hybrid decryption with wrong key should fail"
    );
}

// ============================================================================
// Performance Benchmarks
// ============================================================================

#[test]
fn test_mldsa_performance() {
    use std::time::Instant;

    let (pk, sk) = mldsa::SecretKey::new().expect("keygen");
    let message = b"Performance benchmark message for ML-DSA-44";

    let start = Instant::now();
    let iterations: u128 = 10;
    for _ in 0..iterations {
        let sig = sk.sign(message).expect("sign");
        let _ = pk.verify(message, &sig);
    }
    let elapsed = start.elapsed();
    let avg_ms = elapsed.as_millis() / iterations;

    // ML-DSA should complete sign+verify in under 100ms on average
    assert!(
        avg_ms < 100,
        "ML-DSA sign+verify should average under 100ms (got {}ms)",
        avg_ms
    );
}

#[test]
fn test_mlkem_performance() {
    use std::time::Instant;

    let iterations: u128 = 10;
    let start = Instant::now();
    for _ in 0..iterations {
        let (pk, sk) = mlkem::SecretKey::new().expect("keygen");
        let (ct, _ss_enc) = pk.encapsulate().expect("encapsulate");
        let _ss_dec = sk.decapsulate(&ct).expect("decapsulate");
    }
    let elapsed = start.elapsed();
    let avg_ms = elapsed.as_millis() / iterations;

    // ML-KEM should complete keygen+encap+decap in under 50ms on average
    assert!(
        avg_ms < 50,
        "ML-KEM keygen+encap+decap should average under 50ms (got {}ms)",
        avg_ms
    );
}

// ============================================================================
// NIST Parameter Validation
// ============================================================================

#[test]
fn test_nist_mldsa44_parameters() {
    // Validate ML-DSA-44 (formerly Dilithium2) NIST parameter sizes
    let (pk_size, sk_size, sig_size) = mldsa::key_sizes();
    assert_eq!(pk_size, 1312, "NIST ML-DSA-44 public key = 1312 bytes");
    assert_eq!(sk_size, 2560, "NIST ML-DSA-44 secret key = 2560 bytes");
    assert_eq!(sig_size, 2420, "NIST ML-DSA-44 signature = 2420 bytes");
}

#[test]
fn test_nist_mlkem768_parameters() {
    // Validate ML-KEM-768 (formerly Kyber768) NIST parameter sizes
    let (pk_size, sk_size, ct_size, ss_size) = mlkem::key_sizes();
    assert_eq!(pk_size, 1184, "NIST ML-KEM-768 public key = 1184 bytes");
    assert_eq!(sk_size, 2400, "NIST ML-KEM-768 secret key = 2400 bytes");
    assert_eq!(ct_size, 1088, "NIST ML-KEM-768 ciphertext = 1088 bytes");
    assert_eq!(ss_size, 32, "NIST ML-KEM-768 shared secret = 32 bytes");
}

#[test]
fn test_nist_falcon512_public_key_size() {
    // Validate Falcon-512 public key size
    let (pk_size, _sk_size, _sig_size) = falcon::key_sizes();
    assert_eq!(pk_size, 897, "NIST Falcon-512 public key = 897 bytes");
}

// ============================================================================
// Migration Readiness
// ============================================================================

#[test]
fn test_pq_migration_readiness() {
    // Verify all three PQ algorithm families are operational

    // 1. ML-DSA: Sign + Verify
    let (mldsa_pk, mldsa_sk) = mldsa::SecretKey::new().expect("ML-DSA keygen");
    let msg = b"Migration readiness";
    let sig = mldsa_sk.sign(msg).expect("ML-DSA sign");
    assert!(mldsa_pk.verify(msg, &sig).is_ok(), "ML-DSA must be operational");

    // 2. ML-KEM: Encapsulate + Decapsulate
    let (mlkem_pk, mlkem_sk) = mlkem::SecretKey::new().expect("ML-KEM keygen");
    let (ct, ss_enc) = mlkem_pk.encapsulate().expect("ML-KEM encapsulate");
    let ss_dec = mlkem_sk.decapsulate(&ct).expect("ML-KEM decapsulate");
    assert_eq!(ss_enc.as_bytes(), ss_dec.as_bytes(), "ML-KEM must be operational");

    // 3. Falcon: Sign + Verify
    let (falcon_pk, falcon_sk) = falcon::SecretKey::new().expect("Falcon keygen");
    let sig = falcon_sk.sign(msg).expect("Falcon sign");
    assert!(falcon_pk.verify(msg, &sig).is_ok(), "Falcon must be operational");

    // 4. Hybrid Encryption
    let (pk, sk) = mlkem::SecretKey::new().expect("keygen for hybrid");
    let data = b"Government BMN data";
    let (ct, enc, nonce) = hybrid::encrypt_hybrid(&pk, data).expect("Hybrid encrypt");
    let dec = hybrid::decrypt_hybrid(&sk, &ct, &enc, &nonce).expect("Hybrid decrypt");
    assert_eq!(data.as_slice(), dec.as_slice(), "Hybrid encryption must be operational");
}

#[test]
fn test_pqc_error_types() {
    // Verify error types are properly defined with correct Display messages
    let err = PqcError::InvalidKey;
    assert!(format!("{err}").contains("Invalid key"), "InvalidKey display: {err}");

    let err = PqcError::InvalidSignature;
    assert!(format!("{err}").contains("Invalid signature"), "InvalidSignature display: {err}");

    let err = PqcError::VerificationFailed;
    assert!(format!("{err}").contains("verification failed"), "VerificationFailed display: {err}");

    let err = PqcError::KeyGenerationFailed;
    assert!(format!("{err}").contains("Key generation"), "KeyGenerationFailed display: {err}");

    let err = PqcError::CryptoOperationFailed;
    assert!(format!("{err}").contains("Encryption/decryption"), "CryptoOperationFailed display: {err}");

    let err = PqcError::FeatureNotAvailable;
    assert!(format!("{err}").contains("quantum"), "FeatureNotAvailable display: {err}");
}

#[test]
fn test_mldsa_invalid_key_from_bytes() {
    // Verify that invalid byte lengths are rejected
    let result = mldsa::PublicKey::from_bytes(&[0u8; 10]);
    assert!(result.is_err(), "Invalid public key bytes should be rejected");

    let result = mldsa::SecretKey::from_bytes_with_public(&[0u8; 10], &[0u8; 10]);
    assert!(result.is_err(), "Invalid secret key bytes should be rejected");
}

#[test]
fn test_mlkem_invalid_key_from_bytes() {
    // Verify that invalid byte lengths are rejected
    let result = mlkem::PublicKey::from_bytes(&[0u8; 10]);
    assert!(result.is_err(), "Invalid public key bytes should be rejected");

    let result = mlkem::SecretKey::from_bytes_with_public(&[0u8; 10], &[0u8; 10]);
    assert!(result.is_err(), "Invalid secret key bytes should be rejected");

    let result = mlkem::Ciphertext::from_bytes(&[0u8; 10]);
    assert!(result.is_err(), "Invalid ciphertext bytes should be rejected");
}
