use argon2::{
    Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version,
    password_hash::{SaltString, rand_core::OsRng},
};

/// Hash a password using Argon2 for secure storage with enhanced security parameters
/// This function generates a cryptographically secure password hash using the
/// Argon2id algorithm with hardened parameters. The resulting hash can be safely
/// stored in a database and later used for password verification.
/// # Security Parameters
/// - Algorithm: Argon2id (hybrid mode, resistant to both side-channel and GPU attacks)
/// - Memory cost: 64 MB (65536 KiB) - prevents parallel attacks
/// - Time cost: 10 iterations - balances security and performance
/// - Parallelism: 4 threads - utilizes modern CPU capabilities
/// - Output length: 32 bytes (256 bits)
/// # Arguments
/// * `password` - The plaintext password to hash
/// # Returns
/// A `Result` containing the password hash string in PHC format on success,
/// or an Argon2 error on failure
/// # Security Considerations
/// - Uses Argon2id with parameters exceeding OWASP recommendations
/// - Generates cryptographically secure random salt for each password
/// - Hash format includes algorithm parameters for future verification
/// - Computationally expensive to prevent brute force attacks (>100ms per hash)
/// - Memory-hard to prevent GPU-based attacks
/// - Should be used for all password storage operations
/// # Example
/// ```rust
/// use authenc::utils::crypto::password::hash_password;
/// let hash = hash_password("my_secure_password").expect("Failed to hash password");
/// // Store the hash in database
/// ```
pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);

    // Configure Argon2id with enhanced security parameters
    // Memory: 64 MB (65536 KiB), Iterations: 10, Parallelism: 4
    let params = Params::new(
        65536,    // m_cost: 64 MB memory
        10,       // t_cost: 10 iterations
        4,        // p_cost: 4 parallel threads
        Some(32), // output length: 32 bytes
    )?;

    let argon2 = Argon2::new(
        Algorithm::Argon2id, // Hybrid mode for maximum security
        Version::V0x13,      // Latest version
        params,
    );

    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(password_hash.to_string())
}

/// Verify a password against its hash
/// This function verifies a plaintext password against a previously computed
/// Argon2 password hash. It performs a constant-time comparison to prevent
/// timing attacks and returns whether the password matches the hash.
/// # Arguments
/// * `hash` - The password hash string in PHC format (from `hash_password`)
/// * `password` - The plaintext password to verify
/// # Returns
/// A `Result` containing `true` if the password matches, `false` otherwise,
/// or an Argon2 error if the hash format is invalid
/// # Security Considerations
/// - Uses constant-time comparison to prevent timing attacks
/// - Validates hash format before verification
/// - Returns boolean result to avoid information leakage
/// - Should be used for all password verification operations
/// - Failed verifications should be logged for security monitoring
/// # Example
/// ```rust
/// use authenc::utils::crypto::password::{hash_password, verify_password};
/// let hash = hash_password("my_password").unwrap();
/// let is_valid = verify_password(&hash, "my_password").unwrap();
/// assert!(is_valid);
/// let is_invalid = verify_password(&hash, "wrong_password").unwrap();
/// assert!(!is_invalid);
/// ```
pub fn verify_password(hash: &str, password: &str) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hash)?;
    let argon2 = Argon2::default();
    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Password strength validation result
#[derive(Debug, Clone)]
pub struct PasswordStrengthResult {
    /// Whether the password meets all requirements
    pub is_valid: bool,
    /// List of validation errors
    pub errors: Vec<String>,
    /// Password strength score (0-100)
    pub strength_score: u8,
}

impl PasswordStrengthResult {
    /// Creates a new password strength result
    pub fn new(is_valid: bool, errors: Vec<String>, strength_score: u8) -> Self {
        Self {
            is_valid,
            errors,
            strength_score,
        }
    }

    /// Creates a valid result with no errors
    pub fn valid(strength_score: u8) -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            strength_score,
        }
    }

    /// Creates an invalid rerrors
    pub fn invalid(errors: Vec<String>) -> Self {
        Self {
            is_valid: false,
            errors,
            strength_score: 0,
        }
    }
}

/// Validate password strength according to security requirements
/// This function validates a password against comprehensive security requirements:
/// - Minimum 8 characters length
/// - At least one uppercase letter
/// - At least one lowercase letter
/// - At least one digit
/// - At least one special character
/// - No common weak patterns
/// - No username inclusion
/// # Arguments
/// * `password` - The plaintext password to validate
/// * `username` - Optional username to check for inclusion
/// # Returns
/// A `PasswordStrengthResult` containing validation status, errors, and strength score
/// # Example
/// ```rust
/// use authenc::utils::crypto::password::validate_password_strength;
/// let result = validate_password_strength("MyP@ssw0rd", Some("user123"));
/// assert!(result.is_valid);
/// assert!(result.strength_score >= 70);
/// ```
pub fn validate_password_strength(
    password: &str,
    username: Option<&str>,
) -> PasswordStrengthResult {
    let mut errors = Vec::new();
    let mut strength_score = 0u8;

    // Check minimum length (8 characters)
    if password.len() < 8 {
        errors.push("Password must be at least 8 characters long".to_string());
    } else {
        strength_score += 20;
        // Bonus for longer passwords
        if password.len() >= 12 {
            strength_score += 10;
        }
        if password.len() >= 16 {
            strength_score += 10;
        }
    }

    // Check for uppercase letter
    if !password.chars().any(|c| c.is_uppercase()) {
        errors.push("Password must contain at least one uppercase letter".to_string());
    } else {
        strength_score += 15;
    }

    // Check for lowercase letter
    if !password.chars().any(|c| c.is_lowercase()) {
        errors.push("Password must contain at least one lowercase letter".to_string());
    } else {
        strength_score += 15;
    }

    // Check for digit
    if !password.chars().any(|c| c.is_ascii_digit()) {
        errors.push("Password must contain at least one digit".to_string());
    } else {
        strength_score += 15;
    }

    // Check for special character
    let special_chars = "!@#$%^&*()_+-=[]{}|;:,.<>?/~`";
    if !password.chars().any(|c| special_chars.contains(c)) {
        errors.push(
            "Password must contain at least one special character (!@#$%^&*()_+-=[]{}|;:,.<>?/~`)"
                .to_string(),
        );
    } else {
        strength_score += 15;
    }

    // Check for username inclusion
    if let Some(user) = username {
        let lower_password = password.to_lowercase();
        let lower_username = user.to_lowercase();
        if lower_password.contains(&lower_username) {
            errors.push("Password cannot contain your username".to_string());
            strength_score = strength_score.saturating_sub(20);
        }
    }

    // Check for common weak patterns
    let lower_password = password.to_lowercase();
    let weak_patterns = [
        ("password", "Password cannot contain the word 'password'"),
        (
            "123456",
            "Password cannot contain sequential numbers like '123456'",
        ),
        (
            "qwerty",
            "Password cannot contain keyboard patterns like 'qwerty'",
        ),
        ("admin", "Password cannot contain the word 'admin'"),
        (
            "letmein",
            "Password cannot contain common phrases like 'letmein'",
        ),
    ];

    for (pattern, message) in &weak_patterns {
        if lower_password.contains(pattern) {
            errors.push(message.to_string());
            strength_score = strength_score.saturating_sub(15);
        }
    }

    // Check for repeated characters (3 or more in a row)
    let chars: Vec<char> = password.chars().collect();
    for window in chars.windows(3) {
        if window[0] == window[1] && window[1] == window[2] {
            errors
                .push("Password cannot contain 3 or more repeated characters in a row".to_string());
            strength_score = strength_score.saturating_sub(10);
            break;
        }
    }

    // Check for sequential characters
    let sequential_patterns = [
        "123", "234", "345", "456", "567", "678", "789", "abc", "bcd", "cde", "def", "efg", "fgh",
        "qwe", "wer", "ert", "rty", "tyu", "yui", "asd", "sdf", "dfg", "fgh", "ghj", "hjk", "zxc",
        "xcv", "cvb", "vbn", "bnm",
    ];

    for pattern in &sequential_patterns {
        if lower_password.contains(pattern) {
            errors.push("Password cannot contain sequential characters".to_string());
            strength_score = strength_score.saturating_sub(10);
            break;
        }
    }

    // Bonus for character diversity
    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| special_chars.contains(c));

    let diversity_count = [has_upper, has_lower, has_digit, has_special]
        .iter()
        .filter(|&&x| x)
        .count();

    if diversity_count == 4 {
        strength_score = strength_score.saturating_add(10);
    }

    let is_valid = errors.is_empty();
    PasswordStrengthResult::new(is_valid, errors, strength_score.min(100))
}

/// Check if a password matches any in the password history
/// This function verifies if a new password has been used before by comparing
/// it against a list of previous password hashes. This prevents password reuse
/// and enforces password history policies.
/// # Arguments
/// * `password` - The new plaintext password to check
/// * `password_history` - List of previous password hashes (most recent first)
/// * `history_limit` - Maximum number of previous passwords to check (default: 5)
/// # Returns
/// `true` if the password was found in history, `false` otherwise
/// # Security Considerations
/// - Uses constant-time comparison for each hash check
/// - Limits history check to prevent performance issues
/// - Should be called before accepting a new password
/// - History should be stored securely in the database
/// # Example
/// ```rust
/// use authenc::utils::crypto::password::{hash_password, check_password_history};
/// let old_hash1 = hash_password("OldPassword1!").unwrap();
/// let old_hash2 = hash_password("OldPassword2!").unwrap();
/// let history = vec![old_hash1, old_hash2];
/// let is_reused = check_password_history("OldPassword1!", &history, 5);
/// assert!(is_reused);
/// let is_new = check_password_history("NewPassword3!", &history, 5);
/// assert!(!is_new);
/// ```
pub fn check_password_history(
    password: &str,
    password_history: &[String],
    history_limit: usize,
) -> bool {
    // Limit the number of history entries to check
    let check_count = password_history.len().min(history_limit);

    // Check if password matches any in history
    for hash in password_history.iter().take(check_count) {
        if let Ok(matches) = verify_password(hash, password)
            && matches
        {
            return true;
        }
    }

    false
}

/// Calculate password expiration date based on policy
/// This function calculates when a password should expire based on the
/// password change date and the configured expiration policy.
/// # Arguments
/// * `password_changed_at` - Timestamp when the password was last changed
/// * `expiration_days` - Number of days until password expires (0 = never expires)
/// # Returns
/// Optional timestamp when the password will expire, or None if no expiration
/// # Example
/// ```rust
/// use authenc::utils::crypto::password::calculate_password_expiration;
/// use chrono::Utc;
/// let changed_at = Utc::now();
/// let expires_at = calculate_password_expiration(changed_at, 90);
/// assert!(expires_at.is_some());
/// ```
pub fn calculate_password_expiration(
    password_changed_at: chrono::DateTime<chrono::Utc>,
    expiration_days: u32,
) -> Option<chrono::DateTime<chrono::Utc>> {
    if expiration_days == 0 {
        return None;
    }

    Some(password_changed_at + chrono::Duration::days(expiration_days as i64))
}

/// Check if a password has expired
/// This function checks if a password has expired based on the expiration
/// timestamp. Returns true if the password is expired or will expire soon.
/// # Arguments
/// * `password_expires_at` - Optional timestamp when the password expires
/// * `grace_period_days` - Number of days before expiration to warn (default: 7)
/// # Returns
/// Tuple of (is_expired, days_until_expiration)
/// # Example
/// ```rust
/// use authenc::utils::crypto::password::check_password_expiration;
/// use chrono::Utc;
/// let expires_at = Utc::now() + chrono::Duration::days(5);
/// let (is_expired, days_left) = check_password_expiration(Some(expires_at), 7);
/// assert!(!is_expired);
/// assert!(days_left.unwrap() <= 5);
/// ```
pub fn check_password_expiration(
    password_expires_at: Option<chrono::DateTime<chrono::Utc>>,
    grace_period_days: u32,
) -> (bool, Option<i64>) {
    if let Some(expires_at) = password_expires_at {
        let now = chrono::Utc::now();
        let duration = expires_at.signed_duration_since(now);
        let days_left = duration.num_days();

        // Check if expired or within grace period
        let is_expired = days_left <= grace_period_days as i64;

        (is_expired, Some(days_left))
    } else {
        // No expiration set
        (false, None)
    }
}
