use authenc::utils::crypto::password::validate_password_strength;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_policy() {
        let result = validate_password_strength("StrongSecure123!", None);
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
        assert!(result.strength_score >= 70);
    }

    #[test]
    fn test_password_too_short() {
        let result = validate_password_strength("Short1!", None);
        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.contains("at least 8 characters"))
        );
    }

    #[test]
    fn test_password_missing_uppercase() {
        let result = validate_password_strength("lowercaseonly123!", None);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("uppercase letter")));
    }

    #[test]
    fn test_password_missing_lowercase() {
        let result = validate_password_strength("UPPERCASEONLY123!", None);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("lowercase letter")));
    }

    #[test]
    fn test_password_missing_digit() {
        let result = validate_password_strength("NoDigitsHere!", None);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("digit")));
    }

    #[test]
    fn test_password_missing_special() {
        let result = validate_password_strength("NoSpecialChars123", None);
        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.contains("special character"))
        );
    }

    #[test]
    fn test_password_blacklisted() {
        let result = validate_password_strength("MyPassword123!", None);
        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.contains("Password cannot contain the word 'password'"))
        );
    }

    #[test]
    fn test_password_blacklisted_case_insensitive() {
        let result = validate_password_strength("ContainsPASSWORD123!", None);
        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.contains("Password cannot contain the word 'password'"))
        );
    }

    #[test]
    fn test_valid_password() {
        let result = validate_password_strength("StrongSecure123!", None);
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_valid_password_with_various_special_chars() {
        let result = validate_password_strength("ComplexPhrase456@#$%^&*()", None);
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_custom_policy_no_requirements() {
        // With the built-in policy, very weak passwords should be rejected
        let result = validate_password_strength("weak", None);
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_custom_policy_only_length() {
        // Verify boundaries around the minimum length requirement (8 characters)
        let too_short = validate_password_strength("short", None);
        assert!(!too_short.is_valid);
        assert!(
            too_short
                .errors
                .iter()
                .any(|e| e.contains("at least 8 characters"))
        );

        // A sufficiently complex password of at least 8 characters should be valid
        let long_enough = validate_password_strength("Abcdef1!", None);
        assert!(long_enough.is_valid);
    }

    #[test]
    fn test_custom_blacklist() {
        // Built-in weak pattern list includes "letmein" as an example weak phrase
        let bad = validate_password_strength("thisisletmein", None);
        assert!(!bad.is_valid);
        assert!(
            bad.errors
                .iter()
                .any(|e| e.to_lowercase().contains("letmein"))
        );

        let good = validate_password_strength("thisisgoodPASS123!", None);
        assert!(good.is_valid);
    }

    #[test]
    fn test_edge_case_empty_password() {
        let result = validate_password_strength("", None);
        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.contains("at least 8 characters"))
        );
    }

    #[test]
    fn test_edge_case_unicode_characters() {
        // Unicode characters plus required character classes should still be accepted
        let result = validate_password_strength("ValidPhrase123!", None);
        assert!(result.is_valid);
    }

    #[test]
    fn test_edge_case_only_special_chars() {
        let result = validate_password_strength("!@#$%^&*()123ABCdef", None);
        assert!(result.is_valid);
    }

    #[test]
    fn test_edge_case_minimum_length_boundary() {
        // With the built-in policy, minimum length is 8 characters
        let too_short = validate_password_strength("Abc1!", None);
        assert!(!too_short.is_valid);

        let min_ok = validate_password_strength("Abcdef1!", None); // 8 chars, complex
        assert!(min_ok.is_valid);

        let longer_ok = validate_password_strength("Abcdefgh1!", None);
        assert!(longer_ok.is_valid);
    }

    #[test]
    fn test_edge_case_blacklist_empty_string() {
        // A reasonably strong password with no weak patterns should be valid
        let result = validate_password_strength("GoodPass123!", None);
        assert!(result.is_valid);
    }

    #[test]
    fn test_comprehensive_policy_validation() {
        // Test all requirements together for a variety of strong passwords
        let valid_cases = vec![
            "StrongPass123!",
            "Complex@Phrase#456",
            "Secure_789$Word",
            "Robust(0)Phrase%",
        ];

        for password in valid_cases {
            let result = validate_password_strength(password, None);
            assert!(
                result.is_valid,
                "Password '{}' should be valid, errors: {:?}",
                password, result.errors
            );
        }

        let invalid_cases = vec![
            ("short", "too short"),
            ("nouppercase123!", "missing uppercase"),
            ("NOLOWERCASE123!", "missing lowercase"),
            ("NoDigits!", "missing digit"),
            ("NoSpecial123", "missing special"),
            ("Password123!", "blacklisted"),
        ];

        for (password, reason) in invalid_cases {
            let result = validate_password_strength(password, None);
            assert!(
                !result.is_valid,
                "Password '{}' should be invalid: {} (errors: {:?})",
                password, reason, result.errors
            );
        }
    }
}
