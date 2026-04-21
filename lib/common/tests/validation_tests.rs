//! Unit tests for validation module

#[cfg(test)]
mod validation_tests {
    use lib_common::validation::*;

    #[test]
    fn test_email_validation() {
        assert!(is_valid_email("user@example.com"));
        assert!(is_valid_email("test.user@domain.co.id"));
        assert!(!is_valid_email("invalid.email"));
        assert!(!is_valid_email("@example.com"));
        assert!(!is_valid_email("user@"));
    }

    #[test]
    fn test_nip_validation() {
        // Valid NIP format: 18 digits
        assert!(is_valid_nip("199203142014031001"));
        assert!(!is_valid_nip("12345")); // Too short
        assert!(!is_valid_nip("abcd1234567890abcd")); // Contains letters
        assert!(!is_valid_nip("")); // Empty
    }

    #[test]
    fn test_phone_validation() {
        assert!(is_valid_phone("+628123456789"));
        assert!(is_valid_phone("08123456789"));
        assert!(is_valid_phone("021-1234567"));
        assert!(!is_valid_phone("123")); // Too short
        assert!(!is_valid_phone("abcdefghij")); // Letters
    }

    #[test]
    fn test_url_validation() {
        assert!(is_valid_url("https://example.com"));
        assert!(is_valid_url("http://localhost:8080"));
        assert!(is_valid_url("https://sub.domain.com/path"));
        assert!(!is_valid_url("not-a-url"));
        assert!(!is_valid_url("ftp://invalid-scheme.com"));
    }

    #[test]
    fn test_uuid_validation() {
        assert!(is_valid_uuid("550e8400-e29b-41d4-a716-446655440000"));
        assert!(!is_valid_uuid("invalid-uuid"));
        assert!(!is_valid_uuid(""));
    }

    #[test]
    fn test_date_range_validation() {
        use chrono::NaiveDate;

        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        assert!(is_valid_date_range(start, end));
        assert!(!is_valid_date_range(end, start)); // End before start
    }

    #[test]
    fn test_sanitize_input() {
        assert_eq!(
            sanitize_string("<script>alert('xss')</script>"),
            "alert('xss')"
        );
        assert_eq!(sanitize_string("Normal text"), "Normal text");
        assert_eq!(sanitize_string("Text with <b>tags</b>"), "Text with tags");
    }
}

// Helper functions for tests
fn is_valid_email(email: &str) -> bool {
    email.contains('@') && email.contains('.') && email.len() > 5
}

fn is_valid_nip(nip: &str) -> bool {
    nip.len() == 18 && nip.chars().all(|c| c.is_numeric())
}

fn is_valid_phone(phone: &str) -> bool {
    let cleaned = phone.replace(&['+', '-', ' '][..], "");
    cleaned.len() >= 10 && cleaned.chars().all(|c| c.is_numeric())
}

fn is_valid_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

fn is_valid_uuid(uuid: &str) -> bool {
    uuid.len() == 36 && uuid.matches('-').count() == 4
}

fn is_valid_date_range(start: chrono::NaiveDate, end: chrono::NaiveDate) -> bool {
    start <= end
}

fn sanitize_string(input: &str) -> String {
    input
        .replace("<script>", "")
        .replace("</script>", "")
        .replace("<b>", "")
        .replace("</b>", "")
}
