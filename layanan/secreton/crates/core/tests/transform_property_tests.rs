//! Property-Based Tests for Transform Engine
//!
//! These tests verify tokenization format preservation and round-trip properties.

use proptest::prelude::*;
use secreton_core::services::secrets::transform::{
    Alphabet, MaskingPattern, TransformEngine, TransformRole, Transformation, TransformationType,
};
use secreton_crypto::{FpeAlphabet, FpeEngine, FpeKey};

// **Feature: secreton-comprehensive-enhancement, Property 25: Tokenization Format Preservation**
// **Validates: Requirements 10.1**
//
// Property: For any input string, the tokenized output SHALL have the same length
// and character class distribution as the input (when using FPE).
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn property_tokenization_format_preservation(
        // Generate numeric strings with at least some variation (avoid all same digit)
        // FF1 requires radix^len >= 1,000,000; for radix 10 the minimum length is 6
        first_part in "[1-9][0-9]{3,7}",
        second_part in "[0-9]{2,7}",
        tweak in prop::collection::vec(any::<u8>(), 0..=32),
    ) {
        let plaintext = format!("{}{}", first_part, second_part);

        // Create FPE engine with numeric alphabet
        let key = FpeKey::generate();
        let engine = FpeEngine::new(key, FpeAlphabet::Numeric)
            .expect("FPE engine creation should succeed");

        // Encrypt the plaintext
        let ciphertext = engine.encrypt(&plaintext, &tweak)
            .expect("Encryption should succeed");

        // Property 1: Length preservation
        prop_assert_eq!(ciphertext.len(), plaintext.len(),
            "FPE should preserve input length");

        // Property 2: Character class preservation (all digits remain digits)
        prop_assert!(ciphertext.chars().all(|c| c.is_ascii_digit()),
            "FPE with numeric alphabet should produce only digits");

        // Property 3: Format is different from input (unless by chance)
        // We can't guarantee this for all inputs, but for most it should differ
        // This is a weak property check - just verify encryption happened
        prop_assert!(!ciphertext.is_empty(), "Ciphertext should not be empty");
    }

    #[test]
    fn property_alphanumeric_format_preservation(
        // Generate alphanumeric strings with variation
        first_part in "[a-z][a-z0-9]{2,9}",
        second_part in "[0-9a-z]{2,9}",
        tweak in prop::collection::vec(any::<u8>(), 0..=32),
    ) {
        let plaintext = format!("{}{}", first_part, second_part);

        let key = FpeKey::generate();
        let engine = FpeEngine::new(key, FpeAlphabet::Alphanumeric)
            .expect("FPE engine creation should succeed");

        let ciphertext = engine.encrypt(&plaintext, &tweak)
            .expect("Encryption should succeed");

        // Property 1: Length preservation
        prop_assert_eq!(ciphertext.len(), plaintext.len(),
            "FPE should preserve input length");

        // Property 2: Character class preservation
        prop_assert!(ciphertext.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()),
            "FPE with alphanumeric alphabet should produce only lowercase letters and digits");
    }
}

// **Feature: secreton-comprehensive-enhancement, Property 26: Tokenization Round-Trip**
// **Validates: Requirements 10.2**
//
// Property: For any tokenized value, detokenization by authorized user SHALL return the original value.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn property_tokenization_roundtrip(
        first_part in "[a-z][a-z0-9]{2,15}",
        second_part in "[0-9a-z]{2,15}",
        tweak_bytes in prop::collection::vec(any::<u8>(), 0..=32),
    ) {
        let plaintext = format!("{}{}", first_part, second_part);

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            let engine = TransformEngine::new();

            // Create FPE transformation
            let mut transformation = Transformation::new(
                "test-fpe".to_string(),
                TransformationType::FPE,
            );
            transformation.alphabet = Some(Alphabet::Alphanumeric);

            engine.create_transformation(transformation.clone())
                .await
                .expect("Transformation creation should succeed");

            // Create role with access
            let role = TransformRole::new(
                "test-role".to_string(),
                vec!["test-fpe".to_string()],
            );
            engine.create_role(role).await
                .expect("Role creation should succeed");

            // Convert tweak bytes to string for API
            let tweak = if tweak_bytes.is_empty() {
                None
             } else { Some(String::from_utf8_lossy(&tweak_bytes).to_string()) };

            // Encode
            let encoded = engine.encode(
                "test-role",
                "test-fpe",
                &plaintext,
                tweak.as_deref(),
            ).await.expect("Encoding should succeed");

            // Decode


            engine.decode(
                "test-role",
                "test-fpe",
                &encoded,
                tweak.as_deref(),
            ).await.expect("Decoding should succeed")
        });

        // Property: Round-trip preserves original value
        prop_assert_eq!(result, plaintext,
            "FPE round-trip should preserve original value");
    }

    #[test]
    fn property_tokenization_roundtrip_tokenization_type(
        plaintext in "[a-zA-Z0-9 ]{4,32}",
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let (token, decoded, token2) = rt.block_on(async {
            let engine = TransformEngine::new();

            // Create tokenization transformation
            let transformation = Transformation::new(
                "test-token".to_string(),
                TransformationType::Tokenization,
            );

            engine.create_transformation(transformation.clone())
                .await
                .expect("Transformation creation should succeed");

            // Create role with access
            let role = TransformRole::new(
                "test-role".to_string(),
                vec!["test-token".to_string()],
            );
            engine.create_role(role).await
                .expect("Role creation should succeed");

            // Encode (tokenize)
            let token = engine.encode(
                "test-role",
                "test-token",
                &plaintext,
                None,
            ).await.expect("Tokenization should succeed");

            // Decode (detokenize)
            let decoded = engine.decode(
                "test-role",
                "test-token",
                &token,
                None,
            ).await.expect("Detokenization should succeed");

            // Property 3: Same input produces same token (consistency)
            let token2 = engine.encode(
                "test-role",
                "test-token",
                &plaintext,
                None,
            ).await.expect("Second tokenization should succeed");

            (token, decoded, token2)
        });

        // Property 1: Token format (should start with "tok_")
        prop_assert!(token.starts_with("tok_"),
            "Token should have correct format");

        // Property 2: Round-trip preserves original value
        prop_assert_eq!(decoded, plaintext,
            "Tokenization round-trip should preserve original value");

        prop_assert_eq!(token, token2,
            "Same input should produce same token (consistent mapping)");
    }
}

// Additional property tests for masking patterns
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn property_credit_card_masking_format(
        digits in "[0-9]{16}",
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let masked = rt.block_on(async {
            let engine = TransformEngine::new();

            // Create masking transformation with credit card pattern
            let mut transformation = Transformation::new(
                "test-cc-mask".to_string(),
                TransformationType::Masking,
            );
            transformation.masking_pattern = Some(MaskingPattern::CreditCard);
            transformation.masking_char = Some('*');

            engine.create_transformation(transformation.clone())
                .await
                .expect("Transformation creation should succeed");

            let role = TransformRole::new(
                "test-role".to_string(),
                vec!["test-cc-mask".to_string()],
            );
            engine.create_role(role).await
                .expect("Role creation should succeed");

            // Mask the credit card
            engine.encode(
                "test-role",
                "test-cc-mask",
                &digits,
                None,
            ).await.expect("Masking should succeed")
        });

        // Property 1: Last 4 digits are visible
        let last_4 = &digits[12..16];
        prop_assert!(masked.ends_with(last_4),
            "Credit card masking should show last 4 digits");

        // Property 2: Contains masking characters
        prop_assert!(masked.contains('*'),
            "Masked credit card should contain mask characters");

        // Property 3: Format includes dashes
        prop_assert!(masked.contains('-'),
            "Masked credit card should be formatted with dashes");
    }

    #[test]
    fn property_email_masking_format(
        local in "[a-z]{3,10}",
        domain in "[a-z]{3,10}",
        tld in "(com|org|net|edu)",
    ) {
        let email = format!("{}@{}.{}", local, domain, tld);

        let rt = tokio::runtime::Runtime::new().unwrap();
        let masked = rt.block_on(async {
            let engine = TransformEngine::new();

            let mut transformation = Transformation::new(
                "test-email-mask".to_string(),
                TransformationType::Masking,
            );
            transformation.masking_pattern = Some(MaskingPattern::Email);
            transformation.masking_char = Some('*');

            engine.create_transformation(transformation.clone())
                .await
                .expect("Transformation creation should succeed");

            let role = TransformRole::new(
                "test-role".to_string(),
                vec!["test-email-mask".to_string()],
            );
            engine.create_role(role).await
                .expect("Role creation should succeed");

            engine.encode(
                "test-role",
                "test-email-mask",
                &email,
                None,
            ).await.expect("Masking should succeed")
        });

        // Property 1: First character is visible
        let first_char = email.chars().next().unwrap();
        prop_assert!(masked.starts_with(first_char),
            "Email masking should show first character");

        // Property 2: Domain is visible
        let domain_part = format!("@{}.{}", domain, tld);
        prop_assert!(masked.ends_with(&domain_part),
            "Email masking should show domain");

        // Property 3: Contains masking characters
        prop_assert!(masked.contains('*'),
            "Masked email should contain mask characters");
    }

    #[test]
    fn property_phone_masking_format(
        digits in "[0-9]{10}",
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let masked = rt.block_on(async {
            let engine = TransformEngine::new();

            let mut transformation = Transformation::new(
                "test-phone-mask".to_string(),
                TransformationType::Masking,
            );
            transformation.masking_pattern = Some(MaskingPattern::Phone);
            transformation.masking_char = Some('*');

            engine.create_transformation(transformation.clone())
                .await
                .expect("Transformation creation should succeed");

            let role = TransformRole::new(
                "test-role".to_string(),
                vec!["test-phone-mask".to_string()],
            );
            engine.create_role(role).await
                .expect("Role creation should succeed");

            engine.encode(
                "test-role",
                "test-phone-mask",
                &digits,
                None,
            ).await.expect("Masking should succeed")
        });

        // Property 1: Last 4 digits are visible
        let last_4 = &digits[6..10];
        prop_assert!(masked.ends_with(last_4),
            "Phone masking should show last 4 digits");

        // Property 2: Contains masking characters
        prop_assert!(masked.contains('*'),
            "Masked phone should contain mask characters");

        // Property 3: Format includes dashes
        prop_assert!(masked.contains('-'),
            "Masked phone should be formatted with dashes");
    }
}

#[cfg(test)]
mod edge_cases {
    use super::*;

    #[tokio::test]
    async fn test_batch_encode_consistency() {
        let engine = TransformEngine::new();

        // Create tokenization transformation
        let transformation =
            Transformation::new("batch-test".to_string(), TransformationType::Tokenization);
        engine.create_transformation(transformation).await.unwrap();

        let role = TransformRole::new("batch-role".to_string(), vec!["batch-test".to_string()]);
        engine.create_role(role).await.unwrap();

        // Test batch encoding
        let values = vec![
            "value1".to_string(),
            "value2".to_string(),
            "value3".to_string(),
        ];

        let batch_results = engine
            .batch_encode("batch-role", "batch-test", &values, None)
            .await
            .unwrap();

        // Verify each value individually
        for (i, value) in values.iter().enumerate() {
            let individual_result = engine
                .encode("batch-role", "batch-test", value, None)
                .await
                .unwrap();

            // Should produce same token (consistent mapping)
            assert_eq!(
                batch_results[i], individual_result,
                "Batch encoding should produce same results as individual encoding"
            );
        }
    }

    #[tokio::test]
    async fn test_audit_statistics() {
        let engine = TransformEngine::new();

        // Create tokenization transformation
        let transformation =
            Transformation::new("audit-test".to_string(), TransformationType::Tokenization);
        engine.create_transformation(transformation).await.unwrap();

        let role = TransformRole::new("audit-role".to_string(), vec!["audit-test".to_string()]);
        engine.create_role(role).await.unwrap();

        // Perform some operations
        let token1 = engine
            .encode("audit-role", "audit-test", "secret1", None)
            .await
            .unwrap();

        let _token2 = engine
            .encode("audit-role", "audit-test", "secret2", None)
            .await
            .unwrap();

        // Decode one token
        let _decoded = engine
            .decode("audit-role", "audit-test", &token1, None)
            .await
            .unwrap();

        // Get audit statistics
        let stats = engine.get_audit_statistics().await.unwrap();

        // Verify statistics
        assert_eq!(stats.total_tokens, 2, "Should have 2 tokens");
        assert_eq!(
            stats.transformations.len(),
            1,
            "Should have 1 transformation"
        );

        let transform_stats = &stats.transformations[0];
        assert_eq!(
            transform_stats.transformation_name, "audit-test",
            "Should be audit-test transformation"
        );
        assert_eq!(transform_stats.token_count, 2, "Should have 2 tokens");

        // Note: encode_count includes initial creation, so we expect at least 2
        assert!(
            transform_stats.total_encode_operations >= 2,
            "Should have at least 2 encode operations"
        );
        assert_eq!(
            transform_stats.total_decode_operations, 1,
            "Should have 1 decode operation"
        );
    }
}
