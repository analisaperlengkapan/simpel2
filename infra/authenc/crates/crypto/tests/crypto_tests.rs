//! Comprehensive tests for authenc-crypto crate
//!
//! This test suite includes:
//! - Unit tests for JWT, password hashing, and encryption
//! - Property-based tests for cryptographic invariants
//! - Edge case testing
//! - Security property verification

use authenc_crypto::{
    jwt::{JwtService, TokenClaims},
    password::Argon2PasswordHasher,
    encryption::{EncryptionService, EncryptedData, key_derivation},
};
use authenc_types::traits::PasswordHasher;
use chrono::Duration;

// ============================================================================
// JWT Tests
// ============================================================================

mod jwt_tests {
    use super::*;

    fn create_test_jwt_service() -> JwtService {
        let key_bytes = JwtService::generate_signing_key();
        JwtService::new(
            &key_bytes,
            "https://test.authenc.local".to_string(),
            Duration::minutes(15),
            Duration::days(7),
        )
        .unwrap()
    }

    #[test]
    fn test_jwt_encode_decode_roundtrip() {
        let service = create_test_jwt_service();

        // Generate token
        let token = service
            .generate_access_token(
                "user-123",
                Some("test-realm".to_string()),
                Some("openid profile email".to_string()),
                Some("session-456".to_string()),
            )
            .unwrap();

        // Verify token
        let claims = service.verify_token(&token).unwrap();

        // Assertions
        assert_eq!(claims.sub, "user-123");
        assert_eq!(claims.iss, "https://test.authenc.local");
        assert_eq!(claims.realm, Some("test-realm".to_string()));
        assert_eq!(claims.scope, Some("openid profile email".to_string()));
        assert_eq!(claims.sid, Some("session-456".to_string()));
        assert!(!claims.is_expired());
        assert!(!claims.is_not_yet_valid());
    }

    #[test]
    fn test_jwt_refresh_token_roundtrip() {
        let service = create_test_jwt_service();

        let token = service
            .generate_refresh_token("user-789", "session-abc")
            .unwrap();

        let claims = service.verify_token(&token).unwrap();

        assert_eq!(claims.sub, "user-789");
        assert_eq!(claims.scope, Some("refresh_token".to_string()));
        assert_eq!(claims.sid, Some("session-abc".to_string()));
    }

    #[test]
    fn test_jwt_custom_claims_roundtrip() {
        let service = create_test_jwt_service();

        let mut claims = TokenClaims::new(
            "user-456".to_string(),
            service.issuer().to_string(),
            Duration::minutes(30),
        );
        claims = claims
            .with_custom_claim("role".to_string(), serde_json::json!("admin"))
            .with_custom_claim("department".to_string(), serde_json::json!("IT"))
            .with_custom_claim("permissions".to_string(), serde_json::json!(["read", "write", "delete"]));

        let token = service.generate_token(&claims).unwrap();
        let decoded = service.verify_token(&token).unwrap();

        assert_eq!(decoded.sub, "user-456");
        assert_eq!(decoded.custom.get("role"), Some(&serde_json::json!("admin")));
        assert_eq!(decoded.custom.get("department"), Some(&serde_json::json!("IT")));
        assert_eq!(
            decoded.custom.get("permissions"),
            Some(&serde_json::json!(["read", "write", "delete"]))
        );
    }

    #[test]
    fn test_jwt_signature_verification_fails_with_different_key() {
        let service1 = create_test_jwt_service();
        let service2 = create_test_jwt_service(); // Different key

        let token = service1
            .generate_access_token("user-123", None, None, None)
            .unwrap();

        let result = service2.verify_token(&token);
        assert!(result.is_err());
    }

    #[test]
    fn test_jwt_expired_token_rejected() {
        let key_bytes = JwtService::generate_signing_key();
        let service = JwtService::new(
            &key_bytes,
            "https://test.authenc.local".to_string(),
            Duration::seconds(-10), // Already expired
            Duration::days(7),
        )
        .unwrap();

        let token = service
            .generate_access_token("user-123", None, None, None)
            .unwrap();

        // Token should be immediately expired
        let result = service.verify_token(&token);
        assert!(result.is_err());
    }

    #[test]
    fn test_jwt_invalid_issuer_rejected() {
        let service1 = create_test_jwt_service();

        let key_bytes = JwtService::generate_signing_key();
        let service2 = JwtService::new(
            &key_bytes,
            "https://different-issuer.com".to_string(),
            Duration::minutes(15),
            Duration::days(7),
        )
        .unwrap();

        let token = service1
            .generate_access_token("user-123", None, None, None)
            .unwrap();

        let result = service2.verify_token(&token);
        assert!(result.is_err());
    }

    #[test]
    fn test_jwt_malformed_token_rejected() {
        let service = create_test_jwt_service();

        // Test various malformed tokens
        let malformed_tokens = vec![
            "not.a.jwt",
            "only.two.parts",
            "too.many.parts.here.invalid",
            "",
            "invalid-base64!@#$",
        ];

        for token in malformed_tokens {
            let result = service.verify_token(token);
            assert!(result.is_err(), "Token '{}' should be rejected", token);
        }
    }

    #[test]
    fn test_jwt_decode_unverified() {
        let service = create_test_jwt_service();

        let token = service
            .generate_access_token("user-123", None, None, None)
            .unwrap();

        let claims = service.decode_unverified(&token).unwrap();
        assert_eq!(claims.sub, "user-123");
    }

    #[test]
    fn test_jwt_public_key_export() {
        let service = create_test_jwt_service();
        let public_key = service.get_public_key_base64();

        assert!(!public_key.is_empty());
        assert!(public_key.len() > 32); // Base64 encoded should be longer
    }

    #[test]
    fn test_jwt_not_before_claim() {
        let service = create_test_jwt_service();

        let future_time = chrono::Utc::now() + Duration::hours(1);
        let claims = TokenClaims::new(
            "user-123".to_string(),
            service.issuer().to_string(),
            Duration::hours(2),
        )
        .with_not_before(future_time);

        let token = service.generate_token(&claims).unwrap();

        // Token should be rejected because it's not yet valid
        let result = service.verify_token(&token);
        assert!(result.is_err());
    }

    #[test]
    fn test_jwt_empty_audience() {
        let service = create_test_jwt_service();

        let claims = TokenClaims::new(
            "user-123".to_string(),
            service.issuer().to_string(),
            Duration::minutes(15),
        )
        .with_audience(vec![]);

        let token = service.generate_token(&claims).unwrap();
        let decoded = service.verify_token(&token).unwrap();

        assert!(decoded.aud.is_empty());
    }

    #[test]
    fn test_jwt_multiple_audiences() {
        let service = create_test_jwt_service();

        let audiences = vec![
            "simpelv2".to_string(),
            "portal".to_string(),
            "api".to_string(),
        ];

        let claims = TokenClaims::new(
            "user-123".to_string(),
            service.issuer().to_string(),
            Duration::minutes(15),
        )
        .with_audience(audiences.clone());

        let token = service.generate_token(&claims).unwrap();
        let decoded = service.verify_token(&token).unwrap();

        assert_eq!(decoded.aud, audiences);
    }
}

// ============================================================================
// Password Hashing Tests
// ============================================================================

mod password_tests {
    use super::*;

    #[test]
    fn test_password_hash_verify_roundtrip() {
        let hasher = Argon2PasswordHasher::new();
        let password = "my-secure-password-123";

        let hash = hasher.hash(password).unwrap();
        assert!(hasher.verify(password, &hash).unwrap());
    }

    #[test]
    fn test_password_wrong_password_rejected() {
        let hasher = Argon2PasswordHasher::new();
        let password = "correct-password";

        let hash = hasher.hash(password).unwrap();
        assert!(!hasher.verify("wrong-password", &hash).unwrap());
    }

    #[test]
    fn test_password_hash_format() {
        let hasher = Argon2PasswordHasher::new();
        let hash = hasher.hash("test-password").unwrap();

        // Verify PHC string format
        assert!(hash.starts_with("$argon2id$"));
        assert!(hash.contains("v=19"));
        assert!(hash.contains("m=65536")); // 64 MB
        assert!(hash.contains("t=3"));     // 3 iterations
        assert!(hash.contains("p=4"));     // 4 threads
    }

    #[test]
    fn test_password_different_passwords_different_hashes() {
        let hasher = Argon2PasswordHasher::new();

        let hash1 = hasher.hash("password1").unwrap();
        let hash2 = hasher.hash("password2").unwrap();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_password_same_password_different_salts() {
        let hasher = Argon2PasswordHasher::new();

        let hash1 = hasher.hash("same-password").unwrap();
        let hash2 = hasher.hash("same-password").unwrap();

        // Different salts produce different hashes
        assert_ne!(hash1, hash2);

        // But both verify correctly
        assert!(hasher.verify("same-password", &hash1).unwrap());
        assert!(hasher.verify("same-password", &hash2).unwrap());
    }

    #[test]
    fn test_password_empty_password() {
        let hasher = Argon2PasswordHasher::new();

        let hash = hasher.hash("").unwrap();
        assert!(hasher.verify("", &hash).unwrap());
        assert!(!hasher.verify("not-empty", &hash).unwrap());
    }

    #[test]
    fn test_password_long_password() {
        let hasher = Argon2PasswordHasher::new();

        let long_password = "a".repeat(1000);
        let hash = hasher.hash(&long_password).unwrap();

        assert!(hasher.verify(&long_password, &hash).unwrap());
        assert!(!hasher.verify("short", &hash).unwrap());
    }

    #[test]
    fn test_password_unicode_password() {
        let hasher = Argon2PasswordHasher::new();

        let unicode_password = "パスワード🔐🇮🇩";
        let hash = hasher.hash(unicode_password).unwrap();

        assert!(hasher.verify(unicode_password, &hash).unwrap());
        assert!(!hasher.verify("password", &hash).unwrap());
    }

    #[test]
    fn test_password_special_characters() {
        let hasher = Argon2PasswordHasher::new();

        let special_password = "p@$$w0rd!#%^&*()_+-=[]{}|;':\",./<>?";
        let hash = hasher.hash(special_password).unwrap();

        assert!(hasher.verify(special_password, &hash).unwrap());
    }

    #[test]
    fn test_password_whitespace_preserved() {
        let hasher = Argon2PasswordHasher::new();

        let password_with_spaces = "  password with spaces  ";
        let hash = hasher.hash(password_with_spaces).unwrap();

        assert!(hasher.verify(password_with_spaces, &hash).unwrap());
        assert!(!hasher.verify("password with spaces", &hash).unwrap());
    }

    #[test]
    fn test_password_case_sensitive() {
        let hasher = Argon2PasswordHasher::new();

        let hash = hasher.hash("Password").unwrap();

        assert!(hasher.verify("Password", &hash).unwrap());
        assert!(!hasher.verify("password", &hash).unwrap());
        assert!(!hasher.verify("PASSWORD", &hash).unwrap());
    }

    #[test]
    fn test_password_invalid_hash_format() {
        let hasher = Argon2PasswordHasher::new();

        let result = hasher.verify("password", "invalid-hash-format");
        assert!(result.is_err());
    }

    #[test]
    fn test_password_custom_params() {
        // Lower parameters for faster testing
        let hasher = Argon2PasswordHasher::with_params(4096, 2, 1).unwrap();

        let password = "test-password";
        let hash = hasher.hash(password).unwrap();

        assert!(hasher.verify(password, &hash).unwrap());
        assert!(hash.contains("m=4096"));
        assert!(hash.contains("t=2"));
        assert!(hash.contains("p=1"));
    }
}

// ============================================================================
// Encryption Tests
// ============================================================================

mod encryption_tests {
    use super::*;

    #[test]
    fn test_encryption_decrypt_roundtrip() {
        let service = EncryptionService::new_with_random_key();
        let plaintext = b"Hello, World! This is a secret message.";

        let encrypted = service.encrypt(plaintext).unwrap();
        let decrypted = service.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_encryption_empty_data() {
        let service = EncryptionService::new_with_random_key();
        let plaintext = b"";

        let encrypted = service.encrypt(plaintext).unwrap();
        let decrypted = service.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_encryption_large_data() {
        let service = EncryptionService::new_with_random_key();
        let plaintext = vec![0xAB; 1024 * 1024]; // 1 MB

        let encrypted = service.encrypt(&plaintext).unwrap();
        let decrypted = service.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_encryption_binary_data() {
        let service = EncryptionService::new_with_random_key();
        let plaintext: Vec<u8> = (0..=255).collect();

        let encrypted = service.encrypt(&plaintext).unwrap();
        let decrypted = service.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_encryption_base64_roundtrip() {
        let service = EncryptionService::new_with_random_key();
        let plaintext = b"Secret message for base64 encoding";

        let encoded = service.encrypt_to_base64(plaintext).unwrap();
        let decrypted = service.decrypt_from_base64(&encoded).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_encryption_different_nonces() {
        let service = EncryptionService::new_with_random_key();
        let plaintext = b"Same plaintext";

        let encrypted1 = service.encrypt(plaintext).unwrap();
        let encrypted2 = service.encrypt(plaintext).unwrap();

        // Different nonces should produce different ciphertexts
        assert_ne!(encrypted1.nonce, encrypted2.nonce);
        assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);

        // But both should decrypt to same plaintext
        let decrypted1 = service.decrypt(&encrypted1).unwrap();
        let decrypted2 = service.decrypt(&encrypted2).unwrap();
        assert_eq!(decrypted1, decrypted2);
        assert_eq!(plaintext, decrypted1.as_slice());
    }

    #[test]
    fn test_encryption_wrong_key_fails() {
        let service1 = EncryptionService::new_with_random_key();
        let service2 = EncryptionService::new_with_random_key();

        let plaintext = b"Secret";
        let encrypted = service1.encrypt(plaintext).unwrap();

        // Decryption with wrong key should fail
        let result = service2.decrypt(&encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_encryption_invalid_key_length() {
        let result = EncryptionService::new(&[0u8; 16]);
        assert!(result.is_err());

        let result = EncryptionService::new(&[0u8; 64]);
        assert!(result.is_err());
    }

    #[test]
    fn test_encryption_invalid_nonce_length() {
        let service = EncryptionService::new_with_random_key();
        let encrypted = EncryptedData {
            ciphertext: vec![0u8; 32],
            nonce: vec![0u8; 8], // Invalid nonce length
        };

        let result = service.decrypt(&encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_encryption_tampered_ciphertext_fails() {
        let service = EncryptionService::new_with_random_key();
        let plaintext = b"Original message";

        let mut encrypted = service.encrypt(plaintext).unwrap();

        // Tamper with ciphertext
        if !encrypted.ciphertext.is_empty() {
            encrypted.ciphertext[0] ^= 0xFF;
        }

        // Decryption should fail due to authentication tag mismatch
        let result = service.decrypt(&encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_encryption_tampered_nonce_fails() {
        let service = EncryptionService::new_with_random_key();
        let plaintext = b"Original message";

        let mut encrypted = service.encrypt(plaintext).unwrap();

        // Tamper with nonce
        encrypted.nonce[0] ^= 0xFF;

        // Decryption should fail
        let result = service.decrypt(&encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_key_derivation_same_password_same_salt() {
        let password = "my-secure-password";
        let salt = key_derivation::generate_salt();

        let key1 = key_derivation::derive_key_from_password(password, &salt).unwrap();
        let key2 = key_derivation::derive_key_from_password(password, &salt).unwrap();

        assert_eq!(key1, key2);
    }

    #[test]
    fn test_key_derivation_different_salts() {
        let password = "my-secure-password";
        let salt1 = key_derivation::generate_salt();
        let salt2 = key_derivation::generate_salt();

        let key1 = key_derivation::derive_key_from_password(password, &salt1).unwrap();
        let key2 = key_derivation::derive_key_from_password(password, &salt2).unwrap();

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_key_derivation_different_passwords() {
        let salt = key_derivation::generate_salt();

        let key1 = key_derivation::derive_key_from_password("password1", &salt).unwrap();
        let key2 = key_derivation::derive_key_from_password("password2", &salt).unwrap();

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_key_derivation_produces_32_bytes() {
        let password = "test-password";
        let salt = key_derivation::generate_salt();

        let key = key_derivation::derive_key_from_password(password, &salt).unwrap();

        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_encryption_with_derived_key() {
        let password = "my-password";
        let salt = key_derivation::generate_salt();

        let key = key_derivation::derive_key_from_password(password, &salt).unwrap();
        let service = EncryptionService::new(&key).unwrap();

        let plaintext = b"Secret data";
        let encrypted = service.encrypt(plaintext).unwrap();
        let decrypted = service.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }
}

// ============================================================================
// Edge Cases and Security Properties
// ============================================================================

mod edge_cases {
    use super::*;

    fn create_test_jwt_service() -> JwtService {
        let key_bytes = JwtService::generate_signing_key();
        JwtService::new(
            &key_bytes,
            "https://test.authenc.local".to_string(),
            Duration::minutes(15),
            Duration::days(7),
        )
        .unwrap()
    }

    #[test]
    fn test_jwt_very_long_subject() {
        let service = create_test_jwt_service();
        let long_subject = "a".repeat(10000);

        let token = service
            .generate_access_token(&long_subject, None, None, None)
            .unwrap();

        let claims = service.verify_token(&token).unwrap();
        assert_eq!(claims.sub, long_subject);
    }

    #[test]
    fn test_jwt_special_characters_in_claims() {
        let service = create_test_jwt_service();

        let special_chars = "user@example.com!#$%^&*()";
        let token = service
            .generate_access_token(special_chars, None, None, None)
            .unwrap();

        let claims = service.verify_token(&token).unwrap();
        assert_eq!(claims.sub, special_chars);
    }

    #[test]
    fn test_password_very_long_password() {
        let hasher = Argon2PasswordHasher::new();
        let very_long_password = "a".repeat(100000);

        let hash = hasher.hash(&very_long_password).unwrap();
        assert!(hasher.verify(&very_long_password, &hash).unwrap());
    }

    #[test]
    fn test_encryption_maximum_size_data() {
        let service = EncryptionService::new_with_random_key();
        let large_data = vec![0xFF; 10 * 1024 * 1024]; // 10 MB

        let encrypted = service.encrypt(&large_data).unwrap();
        let decrypted = service.decrypt(&encrypted).unwrap();

        assert_eq!(large_data, decrypted);
    }

    #[test]
    fn test_encryption_nonce_uniqueness() {
        let service = EncryptionService::new_with_random_key();
        let plaintext = b"Test";

        let mut nonces = std::collections::HashSet::new();

        // Generate 1000 encryptions and check nonce uniqueness
        for _ in 0..1000 {
            let encrypted = service.encrypt(plaintext).unwrap();
            assert!(
                nonces.insert(encrypted.nonce.clone()),
                "Duplicate nonce detected!"
            );
        }
    }
}
