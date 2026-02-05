use base64ct::{Base64UrlUnpadded, Encoding};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

// Import Ed25519 functions
use crate::crypto::ed25519_keys::{ED25519_KEYPAIR, sign_ed25519};
use ed25519_dalek::{Signature, Verifier};

/// JWT claims structure for token payload
/// This struct represents the standard JWT claims used in authentication tokens.
/// It contains the essential claims for user identification and token expiration.
/// # Fields
/// * `sub` - Subject identifier (typically user ID)
/// * `exp` - Expiration timestamp (Unix timestamp)
/// # Security Considerations
/// - The `exp` claim should always be validated to prevent token reuse
/// - The `sub` claim should be validated against authenticated user identity
/// - Additional claims may be needed for more complex authorization scenarios
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject identifier (typically the user ID or username)
    pub sub: String,
    /// Token expiration timestamp as Unix timestamp
    pub exp: usize,
    /// Token purpose (access, mfa_verification, etc.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,
    /// Email address of the user
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// List of roles assigned to the user
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>,
}

/// Refresh token claims with JWT ID for token rotation
///
/// This struct extends basic claims with a unique JWT ID (`jti`) to ensure
/// each refresh token is unique, even when generated in rapid succession.
/// This is critical for proper token rotation security.
#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenClaims {
    /// Subject identifier (typically the user ID or username)
    pub sub: String,
    /// Token expiration timestamp as Unix timestamp
    pub exp: usize,
    /// JWT ID - unique identifier for this token (enables token rotation)
    pub jti: String,
    /// Token purpose
    #[serde(default = "default_refresh_purpose")]
    pub purpose: String,
}

fn default_refresh_purpose() -> String {
    "refresh".to_string()
}

// SECURITY NOTE: Previously had hardcoded secret here. Now removed for security.
// JWT signing now uses Ed25519 keypair (see crypto/ed25519_keys.rs)
// If symmetric signing is needed, use get_jwt_secret() function below.

/// Get JWT secret from environment variable or secure configuration
/// This function retrieves the JWT secret from the environment variable `JWT_SECRET`.
/// If not found, it falls back to a test-only value (NOT for production).
/// # Security Considerations
/// - ALWAYS set JWT_SECRET environment variable in production
/// - Never use hardcoded secrets
/// - Rotate secrets regularly
/// - Use strong random secrets (at least 32 bytes)
/// # Returns
/// A `Result` containing the secret bytes on success, or an error string on failure
fn get_jwt_secret() -> Result<Vec<u8>, String> {
    // Try to get from environment variable
    if let Ok(secret) = std::env::var("JWT_SECRET") {
        if secret.is_empty() {
            return Err("JWT_SECRET environment variable is empty".to_string());
        }
        return Ok(secret.into_bytes());
    }

    // For development/testing only - should NEVER reach here in production
    #[cfg(debug_assertions)]
    {
        log::warn!("JWT_SECRET not set! Using insecure default. DO NOT USE IN PRODUCTION!");
        return Ok(b"test-secret-for-development-only-change-in-production".to_vec());
    }

    #[cfg(not(debug_assertions))]
    {
        Err("JWT_SECRET environment variable must be set in production".to_string())
    }
}

/// Generate a temporary JWT token for MFA verification using Ed25519
/// Creates a short-lived JWT token for users who need to complete MFA verification.
/// These tokens have limited validity and should only be used for MFA endpoints.
/// # Arguments
/// * `user_id` - The user identifier to include in the token's subject claim
/// # Returns
/// A `Result` containing the temporary JWT token string on success, or an error string on failure
/// # Security Considerations
/// - Tokens expire after 10 minutes (shorter than regular tokens)
/// - Should only be accepted by MFA verification endpoints
/// - Uses Ed25519 for cryptographic signing
/// # Example
/// ```rust
/// use authenc::utils::crypto::jwt::generate_temp_jwt;
/// let temp_token = generate_temp_jwt("user123").expect("Failed to generate temp token");
/// ```
pub fn generate_temp_jwt(user_id: &str) -> Result<String, String> {
    let expiration = SystemTime::now()
        .checked_add(Duration::from_secs(10 * 60)) // 10 minutes for MFA verification
        .unwrap()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;

    let claims = Claims {
        sub: user_id.to_owned(),
        exp: expiration,
        purpose: Some("mfa_verification".to_string()),
        email: None,
        roles: None,
    };

    // Create JWT header
    let header = r#"{"alg":"EdDSA","typ":"JWT"}"#;

    // Encode header and payload
    let header_b64 = Base64UrlUnpadded::encode_string(header.as_bytes());
    let payload_json =
        serde_json::to_string(&claims).map_err(|e| format!("Failed to serialize claims: {}", e))?;
    let payload_b64 = Base64UrlUnpadded::encode_string(payload_json.as_bytes());

    // Create message to sign
    let message = format!("{}.{}", header_b64, payload_b64);

    // Sign with Ed25519
    let signature = sign_ed25519(message.as_bytes());
    let signature_b64 = Base64UrlUnpadded::encode_string(&signature.to_bytes());

    // Combine into JWT
    Ok(format!("{}.{}.{}", header_b64, payload_b64, signature_b64))
}

/// Generate a JWT token for user authentication using Ed25519
/// Creates a signed JWT token with standard claims for the specified user.
/// The token includes subject identifier and expiration time, and is signed
/// using Ed25519 digital signatures - secure replacement for RSA.
/// # Arguments
/// * `user_id` - The user identifier to include in the token's subject claim
/// # Returns
/// A `Result` containing the JWT token string on success, or an error string on failure
/// # Security Considerations
/// - Tokens expire after 1 hour by default
/// - Uses Ed25519 for cryptographic signing (secure replacement for vulnerable RSA)
/// - Ed25519 provides better security than RSA and is resistant to timing attacks
/// - Tokens should be validated on every request
/// # Example
/// ```rust
/// use authenc::utils::crypto::jwt::generate_jwt;
/// let token = generate_jwt("user123").expect("Failed to generate token");
/// ```
pub fn generate_jwt(
    user_id: &str,
    email: Option<String>,
    roles: Option<Vec<String>>,
) -> Result<String, String> {
    let expiration = SystemTime::now()
        .checked_add(Duration::from_secs(60 * 60))
        .unwrap()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;

    let claims = Claims {
        sub: user_id.to_owned(),
        exp: expiration,
        purpose: Some("access".to_string()),
        email,
        roles,
    };

    // Create JWT header
    let header = r#"{"alg":"EdDSA","typ":"JWT"}"#;

    // Encode header and payload
    let header_b64 = Base64UrlUnpadded::encode_string(header.as_bytes());
    let payload_json =
        serde_json::to_string(&claims).map_err(|e| format!("Failed to serialize claims: {}", e))?;
    let payload_b64 = Base64UrlUnpadded::encode_string(payload_json.as_bytes());

    // Create message to sign
    let message = format!("{}.{}", header_b64, payload_b64);

    // Sign with Ed25519
    let signature = sign_ed25519(message.as_bytes());
    let signature_b64 = Base64UrlUnpadded::encode_string(&signature.to_bytes());

    // Combine into JWT
    Ok(format!("{}.{}.{}", header_b64, payload_b64, signature_b64))
}

/// Verify and decode a JWT token using Ed25519
/// Validates the signature and expiration of a JWT token, then extracts the claims.
/// This function performs all standard JWT validation including signature verification,
/// expiration checking, and claim extraction using Ed25519.
/// # Arguments
/// * `token` - The JWT token string to verify and decode
/// # Returns
/// A `Result` containing the decoded `Claims` on success, or an error string on failure
/// # Security Considerations
/// - Always verify tokens before trusting their claims
/// - Check token expiration to prevent replay attacks
/// - Validate Ed25519 signature to ensure token integrity
/// - Handle verification failures gracefully without exposing sensitive information
/// # Example
/// ```rust
/// use authenc::utils::crypto::jwt::{generate_jwt, verify_jwt};
/// let token = generate_jwt("user123").unwrap();
/// let claims = verify_jwt(&token).expect("Token verification failed");
/// assert_eq!(claims.sub, "user123");
/// ```
pub fn verify_jwt(token: &str) -> Result<Claims, String> {
    verify_jwt_internal(token)
}

/// Internal JWT verification function
/// This is the core verification logic.
/// # Arguments
/// * `token` - The JWT token string to verify and decode
/// # Returns
/// A `Result` containing the decoded `Claims` on success, or an error string on failure
fn verify_jwt_internal(token: &str) -> Result<Claims, String> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err("Invalid JWT format".to_string());
    }

    let header_b64 = parts[0];
    let payload_b64 = parts[1];
    let signature_b64 = parts[2];

    // Decode signature
    let signature_bytes = Base64UrlUnpadded::decode_vec(signature_b64)
        .map_err(|e| format!("Invalid signature encoding: {}", e))?;
    let signature =
        Signature::from_slice(&signature_bytes).map_err(|e| format!("Invalid signature: {}", e))?;

    // Create message for verification
    let message = format!("{}.{}", header_b64, payload_b64);

    // Verify signature with Ed25519
    let verifying_key = ED25519_KEYPAIR.verifying_key();
    verifying_key
        .verify(message.as_bytes(), &signature)
        .map_err(|e| format!("Signature verification failed: {}", e))?;

    // Decode payload
    let payload_json = String::from_utf8(
        Base64UrlUnpadded::decode_vec(payload_b64)
            .map_err(|e| format!("Invalid payload encoding: {}", e))?,
    )
    .map_err(|e| format!("Invalid payload UTF-8: {}", e))?;

    let claims: Claims =
        serde_json::from_str(&payload_json).map_err(|e| format!("Invalid claims JSON: {}", e))?;

    // Check expiration
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;

    if claims.exp < now {
        return Err("Token has expired".to_string());
    }

    Ok(claims)
}

/// Compute SHA-256 hash of a token for cache key generation
/// Creates a deterministic hash of the token that can be used as a cache key.
/// This allows for efficient cache lookups without storing the full token.
/// # Arguments
/// * `token` - The JWT token string to hash
/// # Returns
/// A hex-encoded SHA-256 hash of the token
pub fn hash_token(token: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Generate a refresh token with longer expiration (30 days)
/// Creates a JWT refresh token that can be used to obtain new access tokens.
/// Refresh tokens have a longer lifetime than access tokens.
/// Each token has a unique JWT ID (jti) for proper token rotation.
///
/// # Arguments
/// * `user_id` - The user identifier to include in the token's subject claim
/// # Returns
/// A `Result` containing the refresh token string on success, or an error string on failure
/// # Security Considerations
/// - Refresh tokens expire after 30 days
/// - Should be stored securely (httpOnly cookies recommended)
/// - Should be rotated on each use (enabled by unique jti)
/// - Uses Ed25519 for cryptographic signing
pub fn generate_refresh_token(user_id: &str) -> Result<String, String> {
    let expiration = SystemTime::now()
        .checked_add(Duration::from_secs(30 * 24 * 60 * 60)) // 30 days
        .unwrap()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;

    // Generate unique JWT ID for token rotation
    let jti = Uuid::new_v4().to_string();

    let claims = RefreshTokenClaims {
        sub: user_id.to_owned(),
        exp: expiration,
        jti,
        purpose: "refresh".to_string(),
    };

    // Create JWT header
    let header = r#"{"alg":"EdDSA","typ":"JWT"}"#;

    // Encode header and payload
    let header_b64 = Base64UrlUnpadded::encode_string(header.as_bytes());
    let payload_json =
        serde_json::to_string(&claims).map_err(|e| format!("Failed to serialize claims: {}", e))?;
    let payload_b64 = Base64UrlUnpadded::encode_string(payload_json.as_bytes());

    // Create message to sign
    let message = format!("{}.{}", header_b64, payload_b64);

    // Sign with Ed25519
    let signature = sign_ed25519(message.as_bytes());
    let signature_b64 = Base64UrlUnpadded::encode_string(&signature.to_bytes());

    // Combine into JWT
    Ok(format!("{}.{}.{}", header_b64, payload_b64, signature_b64))
}

/// Verify and decode a refresh token
/// Validates the signature and expiration of a refresh token.
/// This is identical to verify_jwt but semantically distinct for refresh tokens.
/// # Arguments
/// * `token` - The refresh token string to verify and decode
/// # Returns
/// A `Result` containing the decoded `Claims` on success, or an error string on failure
pub fn verify_refresh_token(token: &str) -> Result<Claims, String> {
    // Refresh tokens use the same verification as access tokens
    verify_jwt(token)
}

/// Extended JWT claims with additional OAuth2/OIDC fields
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExtendedClaims {
    /// Issuer (iss)
    pub iss: String,
    /// Subject (sub) - user identifier
    pub sub: String,
    /// Audience (aud) - client identifier
    pub aud: String,
    /// Expiration time (exp)
    pub exp: usize,
    /// Issued at (iat)
    pub iat: Option<usize>,
    /// Not before (nbf)
    pub nbf: Option<usize>,
    /// JWT ID (jti)
    pub jti: Option<String>,
    /// Scope
    pub scope: Option<String>,
    /// Client ID
    pub client_id: Option<String>,
    /// Additional claims
    #[serde(flatten)]
    pub additional: std::collections::HashMap<String, serde_json::Value>,
}

/// Verify JWT with extended validation and return extended claims
/// This function performs comprehensive JWT validation including:
/// - Signature verification using Ed25519
/// - Expiration checking
/// - Claims extraction with all standard and custom fields
/// # Arguments
/// * `token` - The JWT token string to verify
/// # Returns
/// * `Ok(ExtendedClaims)` - Validated claims with all fields
/// * `Err(crate::error::Result)` - Validation error
/// # Usage
/// Used by token exchange and advanced OAuth2 flows that need access to
/// all token claims (not just sub/exp).
pub fn verify_jwt_with_validation(token: &str) -> crate::error::Result<ExtendedClaims> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(crate::error::AuthencError::validation("Invalid JWT format"));
    }

    let header_b64 = parts[0];
    let payload_b64 = parts[1];
    let signature_b64 = parts[2];

    // Decode signature
    let signature_bytes = Base64UrlUnpadded::decode_vec(signature_b64).map_err(|e| {
        crate::error::AuthencError::validation(format!("Invalid signature encoding: {}", e))
    })?;
    let signature = Signature::from_slice(&signature_bytes)
        .map_err(|e| crate::error::AuthencError::validation(format!("Invalid signature: {}", e)))?;

    // Create message for verification
    let message = format!("{}.{}", header_b64, payload_b64);

    // Verify signature with Ed25519
    let verifying_key = ED25519_KEYPAIR.verifying_key();
    verifying_key
        .verify(message.as_bytes(), &signature)
        .map_err(|e| {
            crate::error::AuthencError::unauthorized(format!(
                "Signature verification failed: {}",
                e
            ))
        })?;

    // Decode payload
    let payload_json =
        String::from_utf8(Base64UrlUnpadded::decode_vec(payload_b64).map_err(|e| {
            crate::error::AuthencError::validation(format!("Invalid payload encoding: {}", e))
        })?)
        .map_err(|e| {
            crate::error::AuthencError::validation(format!("Invalid payload UTF-8: {}", e))
        })?;

    let claims: ExtendedClaims = serde_json::from_str(&payload_json).map_err(|e| {
        crate::error::AuthencError::validation(format!("Invalid claims JSON: {}", e))
    })?;

    // Check expiration
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;

    if claims.exp < now {
        return Err(crate::error::AuthencError::unauthorized(
            "Token has expired",
        ));
    }

    // Check not before if present
    if let Some(nbf) = claims.nbf {
        if nbf > now {
            return Err(crate::error::AuthencError::unauthorized(
                "Token not yet valid",
            ));
        }
    }

    Ok(claims)
}

#[cfg(test)]
mod tests {
    use super::{generate_jwt, verify_jwt};

    #[test]
    fn test_jwt_generation_and_verification() {
        let user_id = "test_user_123";
        let email = Some("test@example.com".to_string());
        let roles = Some(vec!["user".to_string(), "admin".to_string()]);

        // Generate a JWT
        let token =
            generate_jwt(user_id, email.clone(), roles.clone()).expect("Failed to generate JWT");

        // Verify the JWT
        let claims = verify_jwt(&token).expect("Failed to verify JWT");

        // Check that the claims are correct
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.email, email);
        assert_eq!(claims.roles, roles);
        assert!(claims.exp > 0); // Expiration should be set
    }

    #[test]
    fn test_jwt_verification_fails_with_invalid_token() {
        let invalid_token = "invalid.jwt.token";

        // This should fail
        let result = verify_jwt(invalid_token);
        assert!(result.is_err());
    }

    #[test]
    fn test_jwt_verification_fails_with_tampered_token() {
        let user_id = "test_user_123";

        // Generate a valid JWT
        let token = generate_jwt(user_id, None, None).expect("Failed to generate JWT");

        // Tamper with the token (change a character in the payload)
        let mut token_bytes = token.into_bytes();
        if token_bytes.len() > 20 {
            token_bytes[20] = b'x'; // Change a character
        }
        let tampered_token = String::from_utf8(token_bytes).expect("Invalid UTF-8");

        // This should fail
        let result = verify_jwt(&tampered_token);
        assert!(result.is_err());
    }
}
