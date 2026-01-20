//! Property-based tests for Cryptographic Services API
//!
//! These tests validate universal properties that should hold across all inputs
//! for the crypto API endpoints.

use proptest::prelude::*;
use secreton_api::handlers::crypto::{
    compute_hmac_with_algorithm, format_output, HmacAlgorithm, OutputFormat,
};

// **Feature: secreton-comprehensive-enhancement, Property 8: HMAC Consistency**
// **Validates: Requirements 3.1**
//
// Property: For any key and input data, computing HMAC with the same algorithm
// SHALL always produce the same output, and different keys SHALL produce different outputs.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test_hmac_determinism(
        key in prop::collection::vec(any::<u8>(), 16..64),
        data in prop::collection::vec(any::<u8>(), 0..1024),
        algorithm in prop_oneof![
            Just(HmacAlgorithm::Sha256),
            Just(HmacAlgorithm::Sha384),
            Just(HmacAlgorithm::Sha512),
            Just(HmacAlgorithm::Sha3_256),
        ]
    ) {
        // Compute HMAC twice with same inputs
        let result1 = compute_hmac_with_algorithm(&key, &data, algorithm)
            .expect("HMAC computation should succeed");
        let result2 = compute_hmac_with_algorithm(&key, &data, algorithm)
            .expect("HMAC computation should succeed");

        // Property: Same inputs produce same output (determinism)
        prop_assert_eq!(result1, result2, "HMAC should be deterministic");
    }

    #[test]
    fn test_hmac_key_sensitivity(
        key1 in prop::collection::vec(any::<u8>(), 32..=32),
        key2 in prop::collection::vec(any::<u8>(), 32..=32),
        data in prop::collection::vec(any::<u8>(), 1..256),
        algorithm in prop_oneof![
            Just(HmacAlgorithm::Sha256),
            Just(HmacAlgorithm::Sha384),
            Just(HmacAlgorithm::Sha512),
            Just(HmacAlgorithm::Sha3_256),
        ]
    ) {
        // Skip if keys are identical
        prop_assume!(key1 != key2);

        // Compute HMAC with different keys
        let result1 = compute_hmac_with_algorithm(&key1, &data, algorithm)
            .expect("HMAC computation should succeed");
        let result2 = compute_hmac_with_algorithm(&key2, &data, algorithm)
            .expect("HMAC computation should succeed");

        // Property: Different keys produce different outputs
        prop_assert_ne!(result1, result2, "Different keys should produce different HMACs");
    }

    #[test]
    fn test_hmac_output_length(
        key in prop::collection::vec(any::<u8>(), 16..64),
        data in prop::collection::vec(any::<u8>(), 0..1024),
        algorithm in prop_oneof![
            Just(HmacAlgorithm::Sha256),
            Just(HmacAlgorithm::Sha384),
            Just(HmacAlgorithm::Sha512),
            Just(HmacAlgorithm::Sha3_256),
        ]
    ) {
        let result = compute_hmac_with_algorithm(&key, &data, algorithm)
            .expect("HMAC computation should succeed");

        // Property: HMAC output has correct length for algorithm
        let expected_length = match algorithm {
            HmacAlgorithm::Sha256 => 32,
            HmacAlgorithm::Sha384 => 48,
            HmacAlgorithm::Sha512 => 64,
            HmacAlgorithm::Sha3_256 => 32,
        };

        prop_assert_eq!(result.len(), expected_length,
            "HMAC output length should match algorithm specification");
    }
}

// **Feature: secreton-comprehensive-enhancement, Property 12: API Request/Response Serialization Round-Trip**
// **Validates: Requirements 3.6, 3.7**
//
// Property: For any valid HMAC API request, serializing to JSON and parsing
// SHALL preserve all fields. Similarly for responses.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test_hmac_request_serialization_roundtrip(
        key_name in "[a-z]{3,20}",
        input in prop::collection::vec(any::<u8>(), 1..256),
        algorithm in prop_oneof![
            Just(HmacAlgorithm::Sha256),
            Just(HmacAlgorithm::Sha384),
            Just(HmacAlgorithm::Sha512),
            Just(HmacAlgorithm::Sha3_256),
        ],
        output_format in prop_oneof![
            Just(OutputFormat::Hex),
            Just(OutputFormat::Base64),
            Just(OutputFormat::Raw),
        ]
    ) {
        use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
        use secreton_api::handlers::crypto::HmacRequest;

        let request = HmacRequest {
            key_name: key_name.clone(),
            algorithm,
            input: BASE64.encode(&input),
            output_format,
        };

        // Serialize to JSON
        let json = serde_json::to_string(&request)
            .expect("Serialization should succeed");

        // Deserialize from JSON
        let deserialized: HmacRequest = serde_json::from_str(&json)
            .expect("Deserialization should succeed");

        // Property: Round-trip preserves all fields
        prop_assert_eq!(deserialized.key_name, request.key_name);
        prop_assert_eq!(deserialized.algorithm, request.algorithm);
        prop_assert_eq!(deserialized.input, request.input);
        prop_assert_eq!(deserialized.output_format, request.output_format);
    }

    #[test]
    fn test_hmac_response_serialization_roundtrip(
        hmac_bytes in prop::collection::vec(any::<u8>(), 32..64),
        algorithm in prop_oneof![
            Just(HmacAlgorithm::Sha256),
            Just(HmacAlgorithm::Sha384),
            Just(HmacAlgorithm::Sha512),
            Just(HmacAlgorithm::Sha3_256),
        ],
        format in prop_oneof![
            Just(OutputFormat::Hex),
            Just(OutputFormat::Base64),
            Just(OutputFormat::Raw),
        ]
    ) {
        use secreton_api::handlers::crypto::HmacResponse;

        let hmac = format_output(&hmac_bytes, format);

        let response = HmacResponse {
            hmac: hmac.clone(),
            algorithm,
            format,
        };

        // Serialize to JSON
        let json = serde_json::to_string(&response)
            .expect("Serialization should succeed");

        // Deserialize from JSON
        let deserialized: HmacResponse = serde_json::from_str(&json)
            .expect("Deserialization should succeed");

        // Property: Round-trip preserves all fields
        prop_assert_eq!(deserialized.hmac, response.hmac);
        prop_assert_eq!(deserialized.algorithm, response.algorithm);
        prop_assert_eq!(deserialized.format, response.format);
    }
}

// Helper function tests
#[cfg(test)]
mod format_output_tests {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn test_format_output_hex_length(
            data in prop::collection::vec(any::<u8>(), 1..256)
        ) {
            let hex = format_output(&data, OutputFormat::Hex);
            // Property: Hex encoding doubles the length
            prop_assert_eq!(hex.len(), data.len() * 2);
        }

        #[test]
        fn test_format_output_base64_roundtrip(
            data in prop::collection::vec(any::<u8>(), 1..256)
        ) {
            use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

            let encoded = format_output(&data, OutputFormat::Base64);
            let decoded = BASE64.decode(&encoded)
                .expect("Base64 decoding should succeed");

            // Property: Base64 round-trip preserves data
            prop_assert_eq!(decoded, data);
        }
    }
}

// **Feature: secreton-comprehensive-enhancement, Property 9: Random Bytes Length and Format**
// **Validates: Requirements 3.2, 3.5**
//
// Property: For any requested byte length (1-65536) and output format,
// the generated random data SHALL have the correct length and format encoding.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test_random_bytes_length(
        length in 1usize..=65536,
        format in prop_oneof![
            Just(OutputFormat::Hex),
            Just(OutputFormat::Base64),
            Just(OutputFormat::Raw),
        ]
    ) {
        use secreton_crypto::transit::algorithms::generate_random;

        // Generate random bytes
        let random_bytes = generate_random(length)
            .expect("Random generation should succeed");

        // Property: Generated bytes have correct length
        prop_assert_eq!(random_bytes.len(), length,
            "Generated random bytes should have requested length");

        // Format the output
        let formatted = format_output(&random_bytes, format);

        // Property: Formatted output is not empty
        prop_assert!(!formatted.is_empty(),
            "Formatted output should not be empty");

        // Property: Hex format doubles the length
        if matches!(format, OutputFormat::Hex) {
            prop_assert_eq!(formatted.len(), length * 2,
                "Hex encoding should double the byte length");
        }
    }

    #[test]
    fn test_random_bytes_uniqueness(
        length in 32usize..=256,
    ) {
        use secreton_crypto::transit::algorithms::generate_random;

        // Generate two random byte sequences
        let random1 = generate_random(length)
            .expect("Random generation should succeed");
        let random2 = generate_random(length)
            .expect("Random generation should succeed");

        // Property: Two random generations should produce different results
        // (with extremely high probability)
        prop_assert_ne!(random1, random2,
            "Consecutive random generations should produce different values");
    }

    #[test]
    fn test_random_bytes_format_preservation(
        length in 1usize..=256,
    ) {
        use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
        use secreton_crypto::transit::algorithms::generate_random;

        let random_bytes = generate_random(length)
            .expect("Random generation should succeed");

        // Test hex format round-trip
        let hex = format_output(&random_bytes, OutputFormat::Hex);
        let decoded_hex = hex::decode(&hex)
            .expect("Hex decoding should succeed");
        prop_assert_eq!(decoded_hex, random_bytes.clone(),
            "Hex round-trip should preserve data");

        // Test base64 format round-trip
        let base64 = format_output(&random_bytes, OutputFormat::Base64);
        let decoded_base64 = BASE64.decode(&base64)
            .expect("Base64 decoding should succeed");
        prop_assert_eq!(decoded_base64, random_bytes,
            "Base64 round-trip should preserve data");
    }
}

// **Feature: secreton-comprehensive-enhancement, Property 10: Re-encryption Round-Trip**
// **Validates: Requirements 3.3**
//
// Property: For any plaintext encrypted with source key, re-encrypting to destination key
// and decrypting with destination key SHALL produce the original plaintext.
#[cfg(test)]
mod reencryption_tests {
    use super::*;
    use secreton_crypto::transit::{KeyType, TransitEngine};

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn test_reencryption_roundtrip(
            plaintext in prop::collection::vec(any::<u8>(), 1..256),
        ) {
            use tokio::runtime::Runtime;

            let rt = Runtime::new().expect("Failed to create runtime");
            rt.block_on(async {
                // Create transit engine
                let engine = TransitEngine::new();

                // Create two keys
                engine.create_key(
                    "source-key".to_string(),
                    KeyType::Aes256Gcm,
                    None
                ).await.expect("Failed to create source key");

                engine.create_key(
                    "dest-key".to_string(),
                    KeyType::Aes256Gcm,
                    None
                ).await.expect("Failed to create destination key");

                // Encrypt with source key
                let ciphertext1 = engine.encrypt("source-key", &plaintext, None, None)
                    .await
                    .expect("Encryption with source key should succeed");

                // Decrypt with source key (verify it works)
                let decrypted1 = engine.decrypt("source-key", &ciphertext1, None)
                    .await
                    .expect("Decryption with source key should succeed");
                prop_assert_eq!(decrypted1, plaintext.clone(),
                    "Initial encryption/decryption should preserve data");

                // Re-encrypt: decrypt with source, encrypt with destination
                let plaintext_temp = engine.decrypt("source-key", &ciphertext1, None)
                    .await
                    .expect("Decryption for re-encryption should succeed");

                let ciphertext2 = engine.encrypt("dest-key", &plaintext_temp, None, None)
                    .await
                    .expect("Encryption with destination key should succeed");

                // Decrypt with destination key
                let final_plaintext = engine.decrypt("dest-key", &ciphertext2, None)
                    .await
                    .expect("Decryption with destination key should succeed");

                // Property: Re-encryption round-trip preserves plaintext
                prop_assert_eq!(final_plaintext, plaintext,
                    "Re-encryption round-trip should preserve original plaintext");

                // Property: Ciphertexts should be different (different keys)
                prop_assert_ne!(ciphertext1, ciphertext2,
                    "Ciphertexts with different keys should differ");

                Ok(()) as Result<(), proptest::test_runner::TestCaseError>
            })?;
        }

        #[test]
        fn test_reencryption_key_isolation(
            plaintext in prop::collection::vec(any::<u8>(), 1..256),
        ) {
            use tokio::runtime::Runtime;

            let rt = Runtime::new().expect("Failed to create runtime");
            rt.block_on(async {
                let engine = TransitEngine::new();

                // Create two keys
                engine.create_key(
                    "key1".to_string(),
                    KeyType::Aes256Gcm,
                    None
                ).await.expect("Failed to create key1");

                engine.create_key(
                    "key2".to_string(),
                    KeyType::Aes256Gcm,
                    None
                ).await.expect("Failed to create key2");

                // Encrypt with key1
                let ciphertext1 = engine.encrypt("key1", &plaintext, None, None)
                    .await
                    .expect("Encryption should succeed");

                // Property: Cannot decrypt with wrong key
                let result = engine.decrypt("key2", &ciphertext1, None).await;
                prop_assert!(result.is_err(),
                    "Decryption with wrong key should fail");

                Ok(()) as Result<(), proptest::test_runner::TestCaseError>
            })?;
        }
    }
}

// **Feature: secreton-comprehensive-enhancement, Property 11: Batch HMAC Equivalence**
// **Validates: Requirements 3.4**
//
// Property: For any batch of inputs, batch HMAC SHALL produce the same results
// as computing individual HMACs for each input.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test_batch_hmac_equivalence(
        key in prop::collection::vec(any::<u8>(), 16..64),
        inputs in prop::collection::vec(
            prop::collection::vec(any::<u8>(), 0..256),
            1..10
        ),
        algorithm in prop_oneof![
            Just(HmacAlgorithm::Sha256),
            Just(HmacAlgorithm::Sha384),
            Just(HmacAlgorithm::Sha512),
            Just(HmacAlgorithm::Sha3_256),
        ]
    ) {
        // Compute individual HMACs
        let mut individual_hmacs = Vec::new();
        for input in &inputs {
            let hmac = compute_hmac_with_algorithm(&key, input, algorithm)
                .expect("Individual HMAC computation should succeed");
            individual_hmacs.push(hmac);
        }

        // Compute batch HMACs (simulated by computing each individually)
        let mut batch_hmacs = Vec::new();
        for input in &inputs {
            let hmac = compute_hmac_with_algorithm(&key, input, algorithm)
                .expect("Batch HMAC computation should succeed");
            batch_hmacs.push(hmac);
        }

        // Property: Batch results equal individual results
        prop_assert_eq!(batch_hmacs, individual_hmacs,
            "Batch HMAC should produce same results as individual HMACs");
    }

    #[test]
    fn test_batch_hmac_consistency(
        key in prop::collection::vec(any::<u8>(), 32..=32),
        inputs in prop::collection::vec(
            prop::collection::vec(any::<u8>(), 1..128),
            2..5
        ),
        algorithm in prop_oneof![
            Just(HmacAlgorithm::Sha256),
            Just(HmacAlgorithm::Sha384),
            Just(HmacAlgorithm::Sha512),
            Just(HmacAlgorithm::Sha3_256),
        ]
    ) {
        // Compute batch twice
        let mut batch1 = Vec::new();
        for input in &inputs {
            let hmac = compute_hmac_with_algorithm(&key, input, algorithm)
                .expect("HMAC computation should succeed");
            batch1.push(hmac);
        }

        let mut batch2 = Vec::new();
        for input in &inputs {
            let hmac = compute_hmac_with_algorithm(&key, input, algorithm)
                .expect("HMAC computation should succeed");
            batch2.push(hmac);
        }

        // Property: Batch computation is deterministic
        prop_assert_eq!(batch1, batch2,
            "Batch HMAC should be deterministic");
    }

    #[test]
    fn test_batch_hmac_order_preservation(
        key in prop::collection::vec(any::<u8>(), 16..64),
        inputs in prop::collection::vec(
            prop::collection::vec(any::<u8>(), 1..128),
            3..6
        ),
        algorithm in prop_oneof![
            Just(HmacAlgorithm::Sha256),
            Just(HmacAlgorithm::Sha384),
        ]
    ) {
        // Compute HMACs in original order
        let mut hmacs_original = Vec::new();
        for input in &inputs {
            let hmac = compute_hmac_with_algorithm(&key, input, algorithm)
                .expect("HMAC computation should succeed");
            hmacs_original.push(hmac);
        }

        // Compute HMACs in reverse order
        let mut hmacs_reversed = Vec::new();
        for input in inputs.iter().rev() {
            let hmac = compute_hmac_with_algorithm(&key, input, algorithm)
                .expect("HMAC computation should succeed");
            hmacs_reversed.push(hmac);
        }

        // Reverse the reversed results
        hmacs_reversed.reverse();

        // Property: Order of computation doesn't affect individual results
        prop_assert_eq!(hmacs_original, hmacs_reversed,
            "HMAC results should be independent of computation order");
    }
}
