//! Password hashing with Argon2id
//!
//! This module provides secure password hashing using the Argon2id algorithm,
//! which is the winner of the Password Hashing Competition and provides
//! resistance to both side-channel and GPU attacks.
//!
//! # Configuration
//!
//! - Algorithm: Argon2id (hybrid mode)
//! - Memory cost: 64 MB (65536 KiB)
//! - Time cost: 3 iterations
//! - Parallelism: 4 threads
//! - Salt: 16 bytes random
//!
//! # Example
//!
//! ```rust
//! use authenc_crypto::password::Argon2PasswordHasher;
//! use authenc_types::traits::PasswordHasher;
//!
//! let hasher = Argon2PasswordHasher::new();
//!
//! // Hash a password
//! let hash = hasher.hash("my-secure-password").unwrap();
//!
//! // Verify password
//! assert!(hasher.verify("my-secure-password", &hash).unwrap());
//! assert!(!hasher.verify("wrong-password", &hash).unwrap());
//! ```

use argon2::{
    Argon2, ParamsBuilder, Version,
    password_hash::{
        PasswordHash, PasswordHasher as Argon2Trait, PasswordVerifier, SaltString, rand_core::OsRng,
    },
};
use authenc_types::{AuthencError, result::Result, traits::PasswordHasher};

/// Argon2id password hasher
///
/// Implements the `PasswordHasher` trait using Argon2id algorithm with
/// secure default parameters.
#[derive(Debug, Clone)]
pub struct Argon2PasswordHasher {
    argon2: Argon2<'static>,
}

impl Argon2PasswordHasher {
    /// Create a new Argon2PasswordHasher with secure default parameters
    ///
    /// # Configuration
    ///
    /// - Memory cost: 64 MB (65536 KiB)
    /// - Time cost: 3 iterations
    /// - Parallelism: 4 threads
    /// - Version: Argon2 v0x13
    ///
    /// # Example
    ///
    /// ```rust
    /// use authenc_crypto::password::Argon2PasswordHasher;
    ///
    /// let hasher = Argon2PasswordHasher::new();
    /// ```
    pub fn new() -> Self {
        // Configure Argon2id parameters
        // Memory cost: 64 MB = 65536 KiB
        // Time cost: 3 iterations
        // Parallelism: 4 threads
        let params = ParamsBuilder::new()
            .m_cost(65536) // 64 MB
            .t_cost(3) // 3 iterations
            .p_cost(4) // 4 threads
            .build()
            .expect("Failed to build Argon2 parameters");

        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id, // Hybrid mode (side-channel + GPU resistant)
            Version::V0x13,              // Latest version
            params,
        );

        Self { argon2 }
    }

    /// Create a new Argon2PasswordHasher with custom parameters
    ///
    /// # Arguments
    ///
    /// * `m_cost` - Memory cost in KiB (e.g., 65536 for 64 MB)
    /// * `t_cost` - Time cost (number of iterations)
    /// * `p_cost` - Parallelism (number of threads)
    ///
    /// # Example
    ///
    /// ```rust
    /// use authenc_crypto::password::Argon2PasswordHasher;
    ///
    /// // Higher security (slower)
    /// let hasher = Argon2PasswordHasher::with_params(131072, 4, 8).unwrap();
    /// ```
    pub fn with_params(m_cost: u32, t_cost: u32, p_cost: u32) -> Result<Self> {
        let params = ParamsBuilder::new()
            .m_cost(m_cost)
            .t_cost(t_cost)
            .p_cost(p_cost)
            .build()
            .map_err(|e| AuthencError::CryptoError(format!("Invalid Argon2 parameters: {}", e)))?;

        let argon2 = Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, params);

        Ok(Self { argon2 })
    }
}

impl Default for Argon2PasswordHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl PasswordHasher for Argon2PasswordHasher {
    /// Hash a password using Argon2id
    ///
    /// # Arguments
    ///
    /// * `password` - The plaintext password to hash
    ///
    /// # Returns
    ///
    /// A PHC string format hash that includes:
    /// - Algorithm identifier (argon2id)
    /// - Version
    /// - Parameters (m, t, p)
    /// - Salt (base64 encoded)
    /// - Hash (base64 encoded)
    ///
    /// # Example
    ///
    /// ```rust
    /// use authenc_crypto::password::Argon2PasswordHasher;
    /// use authenc_types::traits::PasswordHasher;
    ///
    /// let hasher = Argon2PasswordHasher::new();
    /// let hash = hasher.hash("my-password").unwrap();
    /// // Returns: $argon2id$v=19$m=65536,t=3,p=4$...
    /// ```
    fn hash(&self, password: &str) -> Result<String> {
        // Generate a random salt (16 bytes)
        let salt = SaltString::generate(&mut OsRng);

        // Hash the password
        let password_hash = self
            .argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AuthencError::CryptoError(format!("Password hashing failed: {}", e)))?;

        // Return PHC string format
        Ok(password_hash.to_string())
    }

    /// Verify a password against a hash
    ///
    /// # Arguments
    ///
    /// * `password` - The plaintext password to verify
    /// * `hash` - The PHC string format hash to verify against
    ///
    /// # Returns
    ///
    /// `Ok(true)` if the password matches the hash, `Ok(false)` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use authenc_crypto::password::Argon2PasswordHasher;
    /// use authenc_types::traits::PasswordHasher;
    ///
    /// let hasher = Argon2PasswordHasher::new();
    /// let hash = hasher.hash("my-password").unwrap();
    ///
    /// assert!(hasher.verify("my-password", &hash).unwrap());
    /// assert!(!hasher.verify("wrong-password", &hash).unwrap());
    /// ```
    fn verify(&self, password: &str, hash: &str) -> Result<bool> {
        // Parse the PHC string
        let parsed_hash = PasswordHash::new(hash).map_err(|e| {
            AuthencError::CryptoError(format!("Invalid password hash format: {}", e))
        })?;

        // Verify the password
        match self
            .argon2
            .verify_password(password.as_bytes(), &parsed_hash)
        {
            Ok(()) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(AuthencError::CryptoError(format!(
                "Password verification failed: {}",
                e
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify() {
        let hasher = Argon2PasswordHasher::new();
        let password = "my-secure-password";

        // Hash the password
        let hash = hasher.hash(password).unwrap();

        // Verify correct password
        assert!(hasher.verify(password, &hash).unwrap());

        // Verify incorrect password
        assert!(!hasher.verify("wrong-password", &hash).unwrap());
    }

    #[test]
    fn test_hash_format() {
        let hasher = Argon2PasswordHasher::new();
        let hash = hasher.hash("test-password").unwrap();

        // Check PHC string format
        assert!(hash.starts_with("$argon2id$"));
        assert!(hash.contains("v=19"));
        assert!(hash.contains("m=65536"));
        assert!(hash.contains("t=3"));
        assert!(hash.contains("p=4"));
    }

    #[test]
    fn test_different_passwords_different_hashes() {
        let hasher = Argon2PasswordHasher::new();

        let hash1 = hasher.hash("password1").unwrap();
        let hash2 = hasher.hash("password2").unwrap();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_same_password_different_salts() {
        let hasher = Argon2PasswordHasher::new();

        let hash1 = hasher.hash("same-password").unwrap();
        let hash2 = hasher.hash("same-password").unwrap();

        // Different salts should produce different hashes
        assert_ne!(hash1, hash2);

        // But both should verify correctly
        assert!(hasher.verify("same-password", &hash1).unwrap());
        assert!(hasher.verify("same-password", &hash2).unwrap());
    }

    #[test]
    fn test_custom_params() {
        // Lower parameters for faster testing
        let hasher = Argon2PasswordHasher::with_params(4096, 2, 1).unwrap();

        let password = "test-password";
        let hash = hasher.hash(password).unwrap();

        assert!(hasher.verify(password, &hash).unwrap());
        assert!(hash.contains("m=4096"));
        assert!(hash.contains("t=2"));
        assert!(hash.contains("p=1"));
    }

    #[test]
    fn test_empty_password() {
        let hasher = Argon2PasswordHasher::new();

        let hash = hasher.hash("").unwrap();
        assert!(hasher.verify("", &hash).unwrap());
        assert!(!hasher.verify("not-empty", &hash).unwrap());
    }

    #[test]
    fn test_long_password() {
        let hasher = Argon2PasswordHasher::new();

        let long_password = "a".repeat(1000);
        let hash = hasher.hash(&long_password).unwrap();

        assert!(hasher.verify(&long_password, &hash).unwrap());
        assert!(!hasher.verify("short", &hash).unwrap());
    }

    #[test]
    fn test_unicode_password() {
        let hasher = Argon2PasswordHasher::new();

        let unicode_password = "パスワード🔐";
        let hash = hasher.hash(unicode_password).unwrap();

        assert!(hasher.verify(unicode_password, &hash).unwrap());
        assert!(!hasher.verify("password", &hash).unwrap());
    }

    #[test]
    fn test_invalid_hash_format() {
        let hasher = Argon2PasswordHasher::new();

        let result = hasher.verify("password", "invalid-hash");
        assert!(result.is_err());
    }
}
