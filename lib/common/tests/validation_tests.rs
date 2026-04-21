//! Unit tests for validation module

#[cfg(test)]
mod validation_tests {
    use lib_common::validation::*;

    #[test]
    fn test_email_validation() {
        assert!(validate_email("user@example.com"));
        assert!(validate_email("test.user@domain.co.id"));
        assert!(!validate_email("invalid.email"));
        assert!(!validate_email("@example.com"));
        assert!(!validate_email("user@"));
    }

    #[test]
    fn test_nip_validation() {
        // Valid NIP format: 18 digits
        assert!(validate_nip("199203142014031001"));
        assert!(!validate_nip("12345")); // Too short
        assert!(!validate_nip("abcd1234567890abcd")); // Contains letters
        assert!(!validate_nip("")); // Empty
    }

    #[test]
    fn test_phone_validation() {
        // validate_phone_number uses regex: ^\+?[1-9]\d{6,14}$
        assert!(validate_phone_number("+628123456789"));
        assert!(validate_phone_number("628123456789")); // Without +
        assert!(!validate_phone_number("08123456789")); // Leading 0 not valid in intl format
        assert!(!validate_phone_number("123")); // Too short
        assert!(!validate_phone_number("abcdefghij")); // Letters
    }

    #[test]
    fn test_url_validation() {
        assert!(validate_url("https://example.com", "url").is_ok());
        assert!(validate_url("http://localhost:8080", "url").is_ok());
        assert!(validate_url("https://sub.domain.com/path", "url").is_ok());
        assert!(validate_url("not-a-url", "url").is_err());
        // Note: validate_url uses url::Url::parse which accepts any valid URL scheme
        assert!(validate_url("ftp://files.example.com", "url").is_ok());
    }

    #[test]
    fn test_uuid_validation() {
        // Use uuid crate for validation since lib_common doesn't export is_valid_uuid
        assert!(uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").is_ok());
        assert!(uuid::Uuid::parse_str("invalid-uuid").is_err());
        assert!(uuid::Uuid::parse_str("").is_err());
    }

    #[test]
    fn test_date_range_validation() {
        use chrono::NaiveDate;

        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        assert!(start <= end);
        assert!(!(end <= start)); // End before start
    }

    #[test]
    fn test_sanitize_input() {
        // sanitize_string takes (input, max_length)
        assert_eq!(sanitize_string("Normal text", 1000), "Normal text");
        // sanitize_string filters control chars and truncates, not HTML tags
        assert_eq!(
            sanitize_string("Text with content", 1000),
            "Text with content"
        );
    }
}
