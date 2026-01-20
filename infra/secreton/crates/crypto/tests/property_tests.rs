//! Property-Based Tests for Secreton Cryptographic Operations
//!
//! These tests use proptest to verify universal properties that should hold
//! across all valid inputs, providing stronger correctness guarantees than
//! example-based unit tests.

use proptest::prelude::*;
use secreton_crypto::shamir::{
    generate_shares_with_commitments, reconstruct_secret_verified, validate_shares, ShamirConfig,
};

// **Feature: secreton-comprehensive-enhancement, Property 1: Shamir Secret Sharing Round-Trip**
// **Validates: Requirements 1.1**
//
// Property: For any master key and valid Shamir configuration (threshold T, shares N where T ≤ N),
// generating N shares and reconstructing with any T shares SHALL produce the original master key.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn property_shamir_roundtrip(
        // Generate random master key (16-64 bytes)
        master_key in prop::collection::vec(any::<u8>(), 16..=64),
        // Generate valid threshold (2-10)
        threshold in 2usize..=10,
        // Generate valid total shares (threshold to threshold+10)
    ) {
        // Ensure total shares > threshold (required by ShamirConfig)
        let total_shares = threshold + 1 + (master_key.len() % 10); // Add 1-10 extra shares

        // Create Shamir configuration
        let config = ShamirConfig::new(threshold, total_shares)
            .expect("Valid Shamir configuration");

        // Generate shares with commitments
        let (mut shares, commitment) = generate_shares_with_commitments(&master_key, &config)
            .expect("Share generation should succeed");

        // Validate shares
        validate_shares(&mut shares).expect("Shares should be valid");

        // Test reconstruction with exactly threshold shares
        let threshold_shares: Vec<_> = shares.iter().take(threshold).cloned().collect();
        let reconstructed = reconstruct_secret_verified(&threshold_shares, &commitment)
            .expect("Reconstruction should succeed");

        // Property: Reconstructed secret equals original
        prop_assert_eq!(reconstructed, master_key.clone(),
            "Shamir round-trip failed: reconstructed secret doesn't match original");

        // Test reconstruction with more than threshold shares (should also work)
        if total_shares > threshold {
            let extra_shares: Vec<_> = shares.iter().take(threshold + 1).cloned().collect();
            let reconstructed_extra = reconstruct_secret_verified(&extra_shares, &commitment)
                .expect("Reconstruction with extra shares should succeed");

            prop_assert_eq!(reconstructed_extra, master_key.clone(),
                "Shamir round-trip with extra shares failed");
        }

        // Test that different combinations of threshold shares produce same result
        if total_shares > threshold + 1 {
            // Take last threshold shares instead of first
            let alt_shares: Vec<_> = shares.iter()
                .skip(total_shares - threshold)
                .cloned()
                .collect();
            let reconstructed_alt = reconstruct_secret_verified(&alt_shares, &commitment)
                .expect("Reconstruction with alternative shares should succeed");

            prop_assert_eq!(reconstructed_alt, master_key.clone(),
                "Different share combinations should produce same result");
        }
    }
}

#[cfg(test)]
mod edge_cases {
    use super::*;

    // Test minimum configuration (2-of-3)
    #[test]
    fn test_shamir_minimum_config() {
        let master_key = vec![0x42; 32];
        let config = ShamirConfig::new(2, 3).unwrap();

        let (mut shares, commitment) =
            generate_shares_with_commitments(&master_key, &config).unwrap();
        validate_shares(&mut shares).unwrap();

        let threshold_shares: Vec<_> = shares.iter().take(2).cloned().collect();
        let reconstructed = reconstruct_secret_verified(&threshold_shares, &commitment).unwrap();
        assert_eq!(reconstructed, master_key);
    }

    // Test maximum practical configuration (10-of-20)
    #[test]
    fn test_shamir_large_config() {
        let master_key = vec![0x42; 32];
        let config = ShamirConfig::new(10, 20).unwrap();

        let (mut shares, commitment) =
            generate_shares_with_commitments(&master_key, &config).unwrap();
        validate_shares(&mut shares).unwrap();

        let threshold_shares: Vec<_> = shares.iter().take(10).cloned().collect();
        let reconstructed = reconstruct_secret_verified(&threshold_shares, &commitment).unwrap();
        assert_eq!(reconstructed, master_key);
    }

    // Test that insufficient shares fail
    #[test]
    fn test_shamir_insufficient_shares() {
        let master_key = vec![0x42; 32];
        let config = ShamirConfig::new(3, 5).unwrap();

        let (mut shares, commitment) =
            generate_shares_with_commitments(&master_key, &config).unwrap();
        validate_shares(&mut shares).unwrap();

        // Try with only 2 shares (threshold is 3)
        let insufficient_shares: Vec<_> = shares.iter().take(2).cloned().collect();
        let result = reconstruct_secret_verified(&insufficient_shares, &commitment);

        // Should fail (though the specific error depends on implementation)
        assert!(result.is_err() || result.unwrap() != master_key);
    }
}

// **Feature: secreton-comprehensive-enhancement, Property 2: HKDF Key Derivation Determinism**
// **Validates: Requirements 1.2**
//
// Property: For any master key and context string, deriving a KEK with the same inputs SHALL
// always produce the same KEK, and different contexts SHALL produce different KEKs.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn property_hkdf_determinism(
        // Generate random master key (16-64 bytes)
        master_key in prop::collection::vec(any::<u8>(), 16..=64),
        // Generate random context strings
        context1 in "[a-z]{5,20}",
        context2 in "[a-z]{5,20}",
    ) {
        use secreton_crypto::key_derivation::stretch::derive_multiple_keys;

        // Property 1: Same inputs produce same output (determinism)
        let keys1 = derive_multiple_keys(&master_key, &[&context1], 32)
            .expect("Key derivation should succeed");
        let keys2 = derive_multiple_keys(&master_key, &[&context1], 32)
            .expect("Key derivation should succeed");

        prop_assert_eq!(&keys1[0], &keys2[0],
            "HKDF should be deterministic: same inputs must produce same output");

        // Property 2: Different contexts produce different outputs
        if context1 != context2 {
            let keys_ctx1 = derive_multiple_keys(&master_key, &[&context1], 32)
                .expect("Key derivation should succeed");
            let keys_ctx2 = derive_multiple_keys(&master_key, &[&context2], 32)
                .expect("Key derivation should succeed");

            prop_assert_ne!(&keys_ctx1[0], &keys_ctx2[0],
                "Different contexts must produce different keys");
        }

        // Property 3: Multiple keys from same master are all different
        let contexts = vec![&context1[..], &context2[..], "extra-context"];
        let multiple_keys = derive_multiple_keys(&master_key, &contexts, 32)
            .expect("Key derivation should succeed");

        prop_assert_eq!(multiple_keys.len(), 3, "Should generate 3 keys");

        // All keys should be different (if contexts are different)
        if context1 != context2 {
            prop_assert_ne!(&multiple_keys[0], &multiple_keys[1],
                "Keys with different contexts should differ");
            prop_assert_ne!(&multiple_keys[0], &multiple_keys[2],
                "Keys with different contexts should differ");
            prop_assert_ne!(&multiple_keys[1], &multiple_keys[2],
                "Keys with different contexts should differ");
        }
    }
}

#[cfg(test)]
mod hkdf_edge_cases {
    use super::*;
    use secreton_crypto::key_derivation::stretch::derive_multiple_keys;

    #[test]
    fn test_hkdf_empty_context() {
        let master_key = vec![0x42; 32];

        // Empty context should still work
        let keys = derive_multiple_keys(&master_key, &[""], 32).unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].len(), 32);
    }

    #[test]
    fn test_hkdf_long_context() {
        let master_key = vec![0x42; 32];
        let long_context = "a".repeat(1000);

        // Long context should work
        let keys = derive_multiple_keys(&master_key, &[&long_context], 32).unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].len(), 32);
    }

    #[test]
    fn test_hkdf_many_keys() {
        let master_key = vec![0x42; 32];
        let contexts: Vec<String> = (0..100).map(|i| format!("context-{}", i)).collect();
        let context_refs: Vec<&str> = contexts.iter().map(|s| s.as_str()).collect();

        let keys = derive_multiple_keys(&master_key, &context_refs, 32).unwrap();
        assert_eq!(keys.len(), 100);

        // All keys should be unique
        for i in 0..keys.len() {
            for j in (i + 1)..keys.len() {
                assert_ne!(keys[i], keys[j], "Keys {} and {} should be different", i, j);
            }
        }
    }

    #[test]
    fn test_hkdf_different_key_lengths() {
        let master_key = vec![0x42; 32];

        let keys_16 = derive_multiple_keys(&master_key, &["test"], 16).unwrap();
        let keys_32 = derive_multiple_keys(&master_key, &["test"], 32).unwrap();
        let keys_64 = derive_multiple_keys(&master_key, &["test"], 64).unwrap();

        assert_eq!(keys_16[0].len(), 16);
        assert_eq!(keys_32[0].len(), 32);
        assert_eq!(keys_64[0].len(), 64);
    }
}
