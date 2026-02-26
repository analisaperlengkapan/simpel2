//! Property-based tests for cryptographic invariants
//!
//! These tests use proptest to verify that cryptographic operations
//! maintain their invariants across arbitrary inputs.
//!
//! # Properties Tested
//!
//! ## JWT Properties
//! 1. **Encode/Decode Roundtrip**: Any valid token can be decoded back to original claims
//! 2. **Signature Verification**: Tokens signed with one key cannot be verified with another
//! 3. **Expiration Monotonicity**: Expired tokens remain expired
//!
//! ## Password Hashing Properties
//! 1. **Hash/Verify Roundtrip**: Any password can be hashed and verified
//! 2. **Hash Uniqueness**: Same password with different salts produces different hashes
//! 3. **Verification Determinism**: Verification result is consistent
//!
//! ## Encryption Properties
//! 1. **Encrypt/Decrypt Roundtrip**: Any data can be encrypted and decrypted
//! 2. **Ciphertext Uniqueness**: Same plaintext produces different ciphertexts (due to nonce)
//! 3. **Authentication**: Tampered ciphertext fails decryption

use authenc_crypto::{
    encryption::{EncryptionService, key_derivation},
    jwt::{JwtService, TokenClaims},
    password::Argon2PasswordHasher,
};
use authenc_types::traits::PasswordHasher;
use chrono::Duration;
use proptest::prelude::*;

// ============================================================================
// JWT Property Tests
// ============================================================================

/// Property: JWT encode/decode roundtrip
///
/// For any valid user_id and realm, encoding and then decoding should
/// return the same claims.
#[test]
fn prop_jwt_encode_decode_roundtrip() {
    proptest!(|(
        user_id in "[a-zA-Z0-9-]{1,100}",
        realm in "[a-zA-Z0-9-]{1,50}",
        scope in "[a-zA-Z0-9 ]{0,100}",
    )| {
        let key_bytes = JwtService::generate_signing_key();
        let service = JwtService::new(
            &key_bytes,
            "https://test.authenc.local".to_string(),
            Duration::minutes(15),
            Duration::days(7),
        )
        .unwrap();

        let token = service
            .generate_access_token(
                &user_id,
                Some(realm.clone()),
                if scope.is_empty() { None } else { Some(scope.clone()) },
                None,
            )
            .unwrap();

        let claims = service.verify_token(&token).unwrap();

        prop_assert_eq!(&claims.sub, &user_id);
        prop_assert_eq!(claims.realm.as_ref(), Some(&realm));
        if !scope.is_empty() {
            prop_assert_eq!(claims.scope.as_ref(), Some(&scope));
        }
    });
}

/// Property: JWT signature verification with different keys
///
/// A token signed with one key should not verify with a different key.
#[test]
fn prop_jwt_signature_verification_different_keys() {
    proptest!(|(user_id in "[a-zA-Z0-9-]{1,100}")| {
        let key1 = JwtService::generate_signing_key();
        let key2 = JwtService::generate_signing_key();

        let service1 = JwtService::new(
            &key1,
            "https://test.authenc.local".to_string(),
            Duration::minutes(15),
            Duration::days(7),
        )
        .unwrap();

        let service2 = JwtService::new(
            &key2,
            "https://test.authenc.local".to_string(),
            Duration::minutes(15),
            Duration::days(7),
        )
        .unwrap();

        let token = service1
            .generate_access_token(&user_id, None, None, None)
            .unwrap();

        // Verification with different key should fail
        let result = service2.verify_token(&token);
        prop_assert!(result.is_err());
    });
}

/// Property: JWT expiration monotonicity
///
/// Once a token is expired, it remains expired.
#[test]
fn prop_jwt_expiration_monotonicity() {
    proptest!(|(user_id in "[a-zA-Z0-9-]{1,100}")| {
        let key_bytes = JwtService::generate_signing_key();
        let service = JwtService::new(
            &key_bytes,
            "https://test.authenc.local".to_string(),
            Duration::seconds(-10), // Already expired
            Duration::days(7),
        )
        .unwrap();

        let token = service
            .generate_access_token(&user_id, None, None, None)
            .unwrap();

        // First verification should fail (expired)
        let result1 = service.verify_token(&token);
        prop_assert!(result1.is_err());

        // Second verification should also fail (still expired)
        let result2 = service.verify_token(&token);
        prop_assert!(result2.is_err());
    });
}

/// Property: JWT custom claims preservation
///
/// Custom claims should be preserved through encode/decode cycle.
#[test]
fn prop_jwt_custom_claims_preservation() {
    proptest!(|(
        user_id in "[a-zA-Z0-9-]{1,100}",
        role in "[a-zA-Z]{1,20}",
        department in "[a-zA-Z]{1,20}",
    )| {
        let key_bytes = JwtService::generate_signing_key();
        let service = JwtService::new(
            &key_bytes,
            "https://test.authenc.local".to_string(),
            Duration::minutes(15),
            Duration::days(7),
        )
        .unwrap();

        let claims = TokenClaims::new(
            user_id.clone(),
            service.issuer().to_string(),
            Duration::minutes(15),
        )
        .with_custom_claim("role".to_string(), serde_json::json!(role.clone()))
        .with_custom_claim("department".to_string(), serde_json::json!(department.clone()));

        let token = service.generate_token(&claims).unwrap();
        let decoded = service.verify_token(&token).unwrap();

        prop_assert_eq!(&decoded.sub, &user_id);
        prop_assert_eq!(decoded.custom.get("role"), Some(&serde_json::json!(role)));
        prop_assert_eq!(decoded.custom.get("department"), Some(&serde_json::json!(department)));
    });
}

// ============================================================================
// Password Hashing Property Tests
// ============================================================================

/// Property: Password hash/verify roundtrip
///
/// For any password, hashing and then verifying should succeed.
#[test]
fn prop_password_hash_verify_roundtrip() {
    proptest!(|(password in "\\PC{1,100}")| {
        let hasher = Argon2PasswordHasher::new();

        let hash = hasher.hash(&password).unwrap();
        let verified = hasher.verify(&password, &hash).unwrap();

        prop_assert!(verified);
    });
}

/// Property: Password hash uniqueness with different salts
///
/// Hashing the same password twice should produce different hashes
/// (due to random salt generation).
#[test]
fn prop_password_hash_uniqueness() {
    proptest!(|(password in "\\PC{1,100}")| {
        let hasher = Argon2PasswordHasher::new();

        let hash1 = hasher.hash(&password).unwrap();
        let hash2 = hasher.hash(&password).unwrap();

        // Different hashes due to different salts
        prop_assert_ne!(&hash1, &hash2);

        // But both should verify correctly
        prop_assert!(hasher.verify(&password, &hash1).unwrap());
        prop_assert!(hasher.verify(&password, &hash2).unwrap());
    });
}

/// Property: Password verification determinism
///
/// Verifying the same password against the same hash should always
/// produce the same result.
#[test]
fn prop_password_verification_determinism() {
    proptest!(|(password in "\\PC{1,100}")| {
        let hasher = Argon2PasswordHasher::new();

        let hash = hasher.hash(&password).unwrap();

        // Multiple verifications should produce same result
        let result1 = hasher.verify(&password, &hash).unwrap();
        let result2 = hasher.verify(&password, &hash).unwrap();
        let result3 = hasher.verify(&password, &hash).unwrap();

        prop_assert_eq!(result1, result2);
        prop_assert_eq!(result2, result3);
        prop_assert!(result1);
    });
}

/// Property: Wrong password always fails verification
///
/// Verifying a different password should always fail.
#[test]
fn prop_password_wrong_password_fails() {
    proptest!(|(
        password1 in "\\PC{1,100}",
        password2 in "\\PC{1,100}",
    )| {
        prop_assume!(password1 != password2);

        let hasher = Argon2PasswordHasher::new();

        let hash = hasher.hash(&password1).unwrap();
        let verified = hasher.verify(&password2, &hash).unwrap();

        prop_assert!(!verified);
    });
}

/// Property: Password hash format consistency
///
/// All hashes should follow the PHC string format.
#[test]
fn prop_password_hash_format() {
    proptest!(|(password in "\\PC{1,100}")| {
        let hasher = Argon2PasswordHasher::new();

        let hash = hasher.hash(&password).unwrap();

        // Check PHC string format
        prop_assert!(hash.starts_with("$argon2id$"));
        prop_assert!(hash.contains("v=19"));
        prop_assert!(hash.contains("m=65536"));
        prop_assert!(hash.contains("t=3"));
        prop_assert!(hash.contains("p=4"));
    });
}

// ============================================================================
// Encryption Property Tests
// ============================================================================

/// Property: Encryption/decryption roundtrip
///
/// For any plaintext, encrypting and then decrypting should return
/// the original plaintext.
#[test]
fn prop_encryption_decrypt_roundtrip() {
    proptest!(|(plaintext in prop::collection::vec(any::<u8>(), 0..10000))| {
        let service = EncryptionService::new_with_random_key();

        let encrypted = service.encrypt(&plaintext).unwrap();
        let decrypted = service.decrypt(&encrypted).unwrap();

        prop_assert_eq!(plaintext, decrypted);
    });
}

/// Property: Ciphertext uniqueness (nonce randomness)
///
/// Encrypting the same plaintext twice should produce different
/// ciphertexts due to random nonce generation.
#[test]
fn prop_encryption_ciphertext_uniqueness() {
    proptest!(|(plaintext in prop::collection::vec(any::<u8>(), 1..1000))| {
        let service = EncryptionService::new_with_random_key();

        let encrypted1 = service.encrypt(&plaintext).unwrap();
        let encrypted2 = service.encrypt(&plaintext).unwrap();

        // Different nonces
        prop_assert_ne!(&encrypted1.nonce, &encrypted2.nonce);

        // Different ciphertexts
        prop_assert_ne!(&encrypted1.ciphertext, &encrypted2.ciphertext);

        // But both decrypt to same plaintext
        let decrypted1 = service.decrypt(&encrypted1).unwrap();
        let decrypted2 = service.decrypt(&encrypted2).unwrap();
        prop_assert_eq!(&decrypted1, &decrypted2);
        prop_assert_eq!(&plaintext, &decrypted1);
    });
}

/// Property: Encryption with wrong key fails
///
/// Decrypting with a different key should always fail.
#[test]
fn prop_encryption_wrong_key_fails() {
    proptest!(|(plaintext in prop::collection::vec(any::<u8>(), 1..1000))| {
        let service1 = EncryptionService::new_with_random_key();
        let service2 = EncryptionService::new_with_random_key();

        let encrypted = service1.encrypt(&plaintext).unwrap();

        // Decryption with wrong key should fail
        let result = service2.decrypt(&encrypted);
        prop_assert!(result.is_err());
    });
}

/// Property: Tampered ciphertext fails authentication
///
/// Modifying the ciphertext should cause decryption to fail.
#[test]
fn prop_encryption_tampered_ciphertext_fails() {
    proptest!(|(
        plaintext in prop::collection::vec(any::<u8>(), 1..1000),
        tamper_index in 0usize..100,
    )| {
        let service = EncryptionService::new_with_random_key();

        let mut encrypted = service.encrypt(&plaintext).unwrap();

        // Tamper with ciphertext if possible
        if !encrypted.ciphertext.is_empty() {
            let idx = tamper_index % encrypted.ciphertext.len();
            encrypted.ciphertext[idx] ^= 0xFF;

            // Decryption should fail
            let result = service.decrypt(&encrypted);
            prop_assert!(result.is_err());
        }
    });
}

/// Property: Base64 encoding roundtrip
///
/// Encrypting to base64 and decrypting should preserve plaintext.
#[test]
fn prop_encryption_base64_roundtrip() {
    proptest!(|(plaintext in prop::collection::vec(any::<u8>(), 0..1000))| {
        let service = EncryptionService::new_with_random_key();

        let encoded = service.encrypt_to_base64(&plaintext).unwrap();
        let decrypted = service.decrypt_from_base64(&encoded).unwrap();

        prop_assert_eq!(plaintext, decrypted);
    });
}

/// Property: Nonce length is always 12 bytes
///
/// ChaCha20-Poly1305 requires 12-byte nonces.
#[test]
fn prop_encryption_nonce_length() {
    proptest!(|(plaintext in prop::collection::vec(any::<u8>(), 0..1000))| {
        let service = EncryptionService::new_with_random_key();

        let encrypted = service.encrypt(&plaintext).unwrap();

        prop_assert_eq!(encrypted.nonce.len(), 12);
    });
}

// ============================================================================
// Key Derivation Property Tests
// ============================================================================

/// Property: Key derivation determinism
///
/// Deriving a key from the same password and salt should always
/// produce the same key.
#[test]
fn prop_key_derivation_determinism() {
    proptest!(|(
        password in "\\PC{1,100}",
        salt in prop::collection::vec(any::<u8>(), 16..32),
    )| {
        let key1 = key_derivation::derive_key_from_password(&password, &salt).unwrap();
        let key2 = key_derivation::derive_key_from_password(&password, &salt).unwrap();

        prop_assert_eq!(key1, key2);
    });
}

/// Property: Key derivation produces 32-byte keys
///
/// All derived keys should be exactly 32 bytes.
#[test]
fn prop_key_derivation_length() {
    proptest!(|(
        password in "\\PC{1,100}",
        salt in prop::collection::vec(any::<u8>(), 16..32),
    )| {
        let key = key_derivation::derive_key_from_password(&password, &salt).unwrap();

        prop_assert_eq!(key.len(), 32);
    });
}

/// Property: Different passwords produce different keys
///
/// Deriving keys from different passwords should produce different keys.
#[test]
fn prop_key_derivation_different_passwords() {
    proptest!(|(
        password1 in "\\PC{1,100}",
        password2 in "\\PC{1,100}",
        salt in prop::collection::vec(any::<u8>(), 16..32),
    )| {
        prop_assume!(password1 != password2);

        let key1 = key_derivation::derive_key_from_password(&password1, &salt).unwrap();
        let key2 = key_derivation::derive_key_from_password(&password2, &salt).unwrap();

        prop_assert_ne!(key1, key2);
    });
}

/// Property: Different salts produce different keys
///
/// Deriving keys with different salts should produce different keys.
#[test]
fn prop_key_derivation_different_salts() {
    proptest!(|(
        password in "\\PC{1,100}",
        salt1 in prop::collection::vec(any::<u8>(), 16..32),
        salt2 in prop::collection::vec(any::<u8>(), 16..32),
    )| {
        prop_assume!(salt1 != salt2);

        let key1 = key_derivation::derive_key_from_password(&password, &salt1).unwrap();
        let key2 = key_derivation::derive_key_from_password(&password, &salt2).unwrap();

        prop_assert_ne!(key1, key2);
    });
}

// ============================================================================
// Cross-Component Property Tests
// ============================================================================

/// Property: Encryption with derived key roundtrip
///
/// Deriving a key from a password and using it for encryption should work.
#[test]
fn prop_encryption_with_derived_key() {
    proptest!(|(
        password in "\\PC{1,100}",
        salt in prop::collection::vec(any::<u8>(), 16..32),
        plaintext in prop::collection::vec(any::<u8>(), 0..1000),
    )| {
        let key = key_derivation::derive_key_from_password(&password, &salt).unwrap();
        let service = EncryptionService::new(&key).unwrap();

        let encrypted = service.encrypt(&plaintext).unwrap();
        let decrypted = service.decrypt(&encrypted).unwrap();

        prop_assert_eq!(plaintext, decrypted);
    });
}

/// Property: Multiple encryptions with same derived key
///
/// Using the same derived key for multiple encryptions should work.
#[test]
fn prop_multiple_encryptions_same_derived_key() {
    proptest!(|(
        password in "\\PC{1,100}",
        salt in prop::collection::vec(any::<u8>(), 16..32),
        plaintext1 in prop::collection::vec(any::<u8>(), 1..500),
        plaintext2 in prop::collection::vec(any::<u8>(), 1..500),
    )| {
        let key = key_derivation::derive_key_from_password(&password, &salt).unwrap();
        let service = EncryptionService::new(&key).unwrap();

        let encrypted1 = service.encrypt(&plaintext1).unwrap();
        let encrypted2 = service.encrypt(&plaintext2).unwrap();

        let decrypted1 = service.decrypt(&encrypted1).unwrap();
        let decrypted2 = service.decrypt(&encrypted2).unwrap();

        prop_assert_eq!(plaintext1, decrypted1);
        prop_assert_eq!(plaintext2, decrypted2);
    });
}
