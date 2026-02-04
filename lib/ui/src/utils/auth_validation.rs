//! Validasi khusus untuk authentication dan user management
//!
//! File ini HARUS SINKRON dengan backend validation di:
//! `infra/authenc/src/utils/validation.rs`
//!
//! Prinsip: Frontend validation untuk UX, Backend validation untuk security

use crate::core::types::*;
// usage of regex::Regex removed as we delegate to lib_common

// ============================================================================
// CONSTANTS (Sinkron dengan backend)
// ============================================================================

/// Username length constraints
pub const USERNAME_MIN_LENGTH: usize = 3;
pub const USERNAME_MAX_LENGTH: usize = 50;

/// Password length constraints
pub const PASSWORD_MIN_LENGTH: usize = 8;
pub const PASSWORD_MAX_LENGTH: usize = 128;

/// Satker code length constraints
pub const SATKER_CODE_MIN_LENGTH: usize = 2;
pub const SATKER_CODE_MAX_LENGTH: usize = 20;

// ============================================================================
// VALIDATION FUNCTIONS
// ============================================================================

/// Validate username format
/// Rules:
/// - 3-50 characters
/// - Alphanumeric with underscore (_) or hyphen (-)
/// - No spaces or special characters
///
/// Sinkron dengan: `infra/authenc/src/utils/validation.rs::validate_username`
pub fn validate_username(username: &str) -> ValidationResult {
    if username.is_empty() {
        return ValidationResult::invalid(vec![ValidationError::new(
            "username",
            "Username tidak boleh kosong",
        )]);
    }

    if username.len() < USERNAME_MIN_LENGTH {
        return ValidationResult::invalid(vec![ValidationError::new(
            "username",
            format!("Username minimal {} karakter", USERNAME_MIN_LENGTH),
        )]);
    }

    if username.len() > USERNAME_MAX_LENGTH {
        return ValidationResult::invalid(vec![ValidationError::new(
            "username",
            format!("Username maksimal {} karakter", USERNAME_MAX_LENGTH),
        )]);
    }

    if !lib_common::validation::validate_username(username) {
        return ValidationResult::invalid(vec![ValidationError::new(
            "username",
            "Username hanya boleh mengandung huruf, angka, underscore (_), atau hyphen (-)",
        )]);
    }

    ValidationResult::valid()
}

/// Validate password complexity
/// Rules:
/// - Minimum 8 characters
/// - At least one uppercase letter
/// - At least one lowercase letter
/// - At least one digit
///
/// Sinkron dengan: `infra/authenc/src/utils/validation.rs::validate_password_complexity`
pub fn validate_password(password: &str) -> ValidationResult {
    let mut errors = Vec::new();

    if password.is_empty() {
        return ValidationResult::invalid(vec![ValidationError::new(
            "password",
            "Password tidak boleh kosong",
        )]);
    }

    if password.len() < PASSWORD_MIN_LENGTH {
        errors.push(ValidationError::new(
            "password",
            format!("Password minimal {} karakter", PASSWORD_MIN_LENGTH),
        ));
    }

    if password.len() > PASSWORD_MAX_LENGTH {
        errors.push(ValidationError::new(
            "password",
            format!("Password maksimal {} karakter", PASSWORD_MAX_LENGTH),
        ));
    }

    if !password.chars().any(|c| c.is_uppercase()) {
        errors.push(ValidationError::new(
            "password",
            "Password harus mengandung minimal satu huruf besar (A-Z)",
        ));
    }

    if !password.chars().any(|c| c.is_lowercase()) {
        errors.push(ValidationError::new(
            "password",
            "Password harus mengandung minimal satu huruf kecil (a-z)",
        ));
    }

    if !password.chars().any(|c| c.is_numeric()) {
        errors.push(ValidationError::new(
            "password",
            "Password harus mengandung minimal satu angka (0-9)",
        ));
    }

    if errors.is_empty() {
        ValidationResult::valid()
    } else {
        ValidationResult::invalid(errors)
    }
}

/// Validate satker code format
/// Rules:
/// - 2-20 characters
/// - Uppercase letters and numbers only
/// - No spaces or special characters
///
/// Sinkron dengan: `infra/authenc/src/utils/validation.rs::validate_satker_code`
pub fn validate_satker_code(code: &str) -> ValidationResult {
    if code.is_empty() {
        return ValidationResult::invalid(vec![ValidationError::new(
            "satker_code",
            "Kode satker tidak boleh kosong",
        )]);
    }

    if code.len() < SATKER_CODE_MIN_LENGTH {
        return ValidationResult::invalid(vec![ValidationError::new(
            "satker_code",
            format!("Kode satker minimal {} karakter", SATKER_CODE_MIN_LENGTH),
        )]);
    }

    if code.len() > SATKER_CODE_MAX_LENGTH {
        return ValidationResult::invalid(vec![ValidationError::new(
            "satker_code",
            format!("Kode satker maksimal {} karakter", SATKER_CODE_MAX_LENGTH),
        )]);
    }

    if !lib_common::validation::validate_satker_code(code) {
        return ValidationResult::invalid(vec![ValidationError::new(
            "satker_code",
            "Kode satker harus huruf besar dan angka saja (contoh: KEJARI, KEJATI01)",
        )]);
    }

    ValidationResult::valid()
}

/// Validate MFA code (6 digits)
/// Rules:
/// - Exactly 6 digits
/// - Numbers only
///
/// Sinkron dengan: Backend MFA verification
pub fn validate_mfa_code(code: &str) -> ValidationResult {
    if code.is_empty() {
        return ValidationResult::invalid(vec![ValidationError::new(
            "mfa_code",
            "Kode MFA tidak boleh kosong",
        )]);
    }

    if !lib_common::validation::validate_mfa_code(code) {
        return ValidationResult::invalid(vec![ValidationError::new(
            "mfa_code",
            "Kode MFA harus 6 digit angka",
        )]);
    }

    ValidationResult::valid()
}

/// Validate realm name
/// Rules:
/// - 1-100 characters
/// - Not empty
///
/// Sinkron dengan: Backend realm validation
pub fn validate_realm(realm: &str) -> ValidationResult {
    if realm.is_empty() {
        return ValidationResult::invalid(vec![ValidationError::new(
            "realm",
            "Realm tidak boleh kosong",
        )]);
    }

    if realm.len() > 100 {
        return ValidationResult::invalid(vec![ValidationError::new(
            "realm",
            "Realm maksimal 100 karakter",
        )]);
    }

    ValidationResult::valid()
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Get password strength indicator
/// Returns: (strength_level, strength_text, color_class)
/// - strength_level: 0-4 (weak to very strong)
/// - strength_text: Human-readable description
/// - color_class: CSS class for visual indicator
pub fn get_password_strength(password: &str) -> (u8, &'static str, &'static str) {
    let mut strength = 0u8;

    // Length check
    if password.len() >= 8 {
        strength += 1;
    }
    if password.len() >= 12 {
        strength += 1;
    }

    // Character variety
    if password.chars().any(|c| c.is_uppercase()) {
        strength += 1;
    }
    if password.chars().any(|c| c.is_lowercase()) {
        strength += 1;
    }
    if password.chars().any(|c| c.is_numeric()) {
        strength += 1;
    }
    if password.chars().any(|c| !c.is_alphanumeric()) {
        strength += 1;
    }

    // Cap at 4
    strength = strength.min(4);

    match strength {
        0..=1 => (strength, "Sangat Lemah", "text-red-600"),
        2 => (strength, "Lemah", "text-orange-600"),
        3 => (strength, "Sedang", "text-yellow-600"),
        4 => (strength, "Kuat", "text-green-600"),
        _ => (strength, "Sangat Kuat", "text-green-700"),
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Username tests
    #[test]
    fn test_username_valid() {
        assert!(validate_username("user123").valid);
        assert!(validate_username("test_user").valid);
        assert!(validate_username("user-name").valid);
        assert!(validate_username("abc").valid); // min 3 chars
    }

    #[test]
    fn test_username_invalid() {
        assert!(!validate_username("").valid); // empty
        assert!(!validate_username("ab").valid); // too short
        assert!(!validate_username("a".repeat(51).as_str()).valid); // too long
        assert!(!validate_username("user@name").valid); // invalid char
        assert!(!validate_username("user name").valid); // space
    }

    // Password tests
    #[test]
    fn test_password_valid() {
        assert!(validate_password("Password123").valid);
        assert!(validate_password("Secure1Pass").valid);
        assert!(validate_password("MyP@ssw0rd").valid);
    }

    #[test]
    fn test_password_invalid() {
        assert!(!validate_password("").valid); // empty
        assert!(!validate_password("pass").valid); // too short
        assert!(!validate_password("password").valid); // no uppercase
        assert!(!validate_password("PASSWORD").valid); // no lowercase
        assert!(!validate_password("Password").valid); // no digit
        assert!(!validate_password("Pass1").valid); // too short
    }

    // Satker code tests
    #[test]
    fn test_satker_code_valid() {
        assert!(validate_satker_code("KEJARI").valid);
        assert!(validate_satker_code("KEJATI01").valid);
        assert!(validate_satker_code("AB").valid); // min 2 chars
    }

    #[test]
    fn test_satker_code_invalid() {
        assert!(!validate_satker_code("").valid); // empty
        assert!(!validate_satker_code("A").valid); // too short
        assert!(!validate_satker_code("A".repeat(21).as_str()).valid); // too long
        assert!(!validate_satker_code("kejari").valid); // lowercase
        assert!(!validate_satker_code("KEJARI-01").valid); // hyphen
    }

    // MFA code tests
    #[test]
    fn test_mfa_code_valid() {
        assert!(validate_mfa_code("123456").valid);
        assert!(validate_mfa_code("000000").valid);
        assert!(validate_mfa_code("999999").valid);
    }

    #[test]
    fn test_mfa_code_invalid() {
        assert!(!validate_mfa_code("").valid); // empty
        assert!(!validate_mfa_code("12345").valid); // too short
        assert!(!validate_mfa_code("1234567").valid); // too long
        assert!(!validate_mfa_code("12345a").valid); // non-digit
    }

    // Password strength tests
    #[test]
    fn test_password_strength() {
        let (level, _, _) = get_password_strength("pass");
        assert!(level <= 2); // weak

        let (level, _, _) = get_password_strength("Password123");
        assert!(level >= 3); // strong

        let (level, _, _) = get_password_strength("P@ssw0rd123!");
        assert_eq!(level, 4); // very strong
    }
}
