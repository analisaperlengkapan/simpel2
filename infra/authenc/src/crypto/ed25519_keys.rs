use base64ct::{Base64UrlUnpadded, Encoding};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use once_cell::sync::Lazy;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::env;

/// Ed25519 keypair for JWT signing - replaces vulnerable RSA
///
/// Production: Loads from ED25519_PRIVATE_KEY_BASE64 environment variable
/// Development: Generates ephemeral key with warning
pub static ED25519_KEYPAIR: Lazy<SigningKey> = Lazy::new(|| {
    // Try to load from environment variable (production)
    if let Ok(key_base64) = env::var("ED25519_PRIVATE_KEY_BASE64") {
        match load_key_from_base64(&key_base64) {
            Ok(key) => {
                tracing::info!("✅ Ed25519 signing key loaded from ED25519_PRIVATE_KEY_BASE64");
                return key;
            }
            Err(e) => {
                tracing::error!("❌ Failed to load Ed25519 key from environment: {}", e);
                tracing::error!(
                    "⚠️  Falling back to ephemeral key generation - THIS WILL BREAK PRODUCTION!"
                );
            }
        }
    }

    // Try to load from file path (alternative production method)
    if let Ok(key_path) = env::var("ED25519_PRIVATE_KEY_PATH") {
        match load_key_from_file(&key_path) {
            Ok(key) => {
                tracing::info!("✅ Ed25519 signing key loaded from file: {}", key_path);
                return key;
            }
            Err(e) => {
                tracing::error!(
                    "❌ Failed to load Ed25519 key from file {}: {}",
                    key_path,
                    e
                );
                tracing::error!(
                    "⚠️  Falling back to ephemeral key generation - THIS WILL BREAK PRODUCTION!"
                );
            }
        }
    }

    // Fallback: Generate ephemeral key (ONLY for development/testing)
    tracing::warn!("⚠️  ED25519_PRIVATE_KEY_BASE64 or ED25519_PRIVATE_KEY_PATH not set");
    tracing::warn!("⚠️  Generating EPHEMERAL Ed25519 signing key");
    tracing::warn!("⚠️  ALL JWT TOKENS WILL BE INVALIDATED ON RESTART");
    tracing::warn!("⚠️  THIS IS NOT SUITABLE FOR PRODUCTION!");

    SigningKey::generate(&mut OsRng)
});

/// Load Ed25519 signing key from base64-encoded bytes
fn load_key_from_base64(key_base64: &str) -> Result<SigningKey, String> {
    let key_bytes = base64ct::Base64::decode_vec(key_base64)
        .map_err(|e| format!("Base64 decode error: {}", e))?;

    if key_bytes.len() != 32 {
        return Err(format!(
            "Invalid key length: expected 32 bytes, got {}",
            key_bytes.len()
        ));
    }

    let key_array: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| "Failed to convert to 32-byte array".to_string())?;

    Ok(SigningKey::from_bytes(&key_array))
}

/// Load Ed25519 signing key from file
fn load_key_from_file(path: &str) -> Result<SigningKey, String> {
    let key_bytes = std::fs::read(path).map_err(|e| format!("Failed to read key file: {}", e))?;

    // Try base64 decode first (if file contains base64 string)
    if let Ok(key_str) = String::from_utf8(key_bytes.clone()) {
        if let Ok(decoded) = base64ct::Base64::decode_vec(key_str.trim()) {
            if decoded.len() == 32 {
                let key_array: [u8; 32] = decoded
                    .try_into()
                    .map_err(|_| "Failed to convert to 32-byte array".to_string())?;
                return Ok(SigningKey::from_bytes(&key_array));
            }
        }
    }

    // Otherwise treat as raw bytes
    if key_bytes.len() != 32 {
        return Err(format!(
            "Invalid key length: expected 32 bytes, got {}",
            key_bytes.len()
        ));
    }

    let key_array: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| "Failed to convert to 32-byte array".to_string())?;

    Ok(SigningKey::from_bytes(&key_array))
}

/// Generate a new Ed25519 keypair and return base64-encoded private key
/// This is a utility function for initial key generation
pub fn generate_new_keypair() -> (SigningKey, String) {
    let signing_key = SigningKey::generate(&mut OsRng);
    let private_key_base64 = base64ct::Base64::encode_string(signing_key.as_bytes());
    (signing_key, private_key_base64)
}

/// JSON Web Key Set containing Ed25519 public keys
#[derive(Debug, Serialize, Deserialize)]
pub struct Ed25519JwkSet {
    /// Array of JSON Web Keys
    pub keys: Vec<Ed25519Jwk>,
}

/// Individual Ed25519 JSON Web Key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ed25519Jwk {
    /// Key type (always "OKP" for Ed25519)
    pub kty: String,
    /// Elliptic curve (always "Ed25519")
    pub crv: String,
    /// Base64URL-encoded public key
    pub x: String,
    /// Key ID for key identification
    pub kid: String,
    /// Intended use of the key ("sig" for signing)
    #[serde(rename = "use")]
    pub key_use: String,
    /// Algorithm identifier ("EdDSA" for Ed25519)
    pub alg: String,
}

impl Ed25519Jwk {
    /// Creates an Ed25519 JWK from a verifying key and key ID.
    ///
    /// # Arguments
    /// * `verifying_key` - The Ed25519 verifying key to convert
    /// * `kid` - The key ID to assign to this JWK
    ///
    /// # Returns
    /// A new `Ed25519Jwk` instance with the public key and metadata.
    pub fn from_verifying_key(verifying_key: &VerifyingKey, kid: &str) -> Self {
        let x = Base64UrlUnpadded::encode_string(verifying_key.as_bytes());

        Self {
            kty: "OKP".to_string(),
            crv: "Ed25519".to_string(),
            x,
            kid: kid.to_string(),
            key_use: "sig".to_string(),
            alg: "EdDSA".to_string(),
        }
    }
}

/// Get the Ed25519 public key in JWK format
///
/// # Returns
/// An `Ed25519Jwk` containing the public key from the global keypair
pub fn get_ed25519_jwk() -> Ed25519Jwk {
    let verifying_key = ED25519_KEYPAIR.verifying_key();
    Ed25519Jwk::from_verifying_key(&verifying_key, "authence-ed25519-key")
}

/// Get the Ed25519 public key in PEM format
pub fn get_ed25519_public_pem() -> String {
    let verifying_key = ED25519_KEYPAIR.verifying_key();
    // Ed25519 public key in raw format (32 bytes)
    let raw_bytes = verifying_key.as_bytes();

    // Create PEM format manually since ed25519-dalek doesn't have built-in PEM support
    let b64_data = base64ct::Base64::encode_string(raw_bytes);
    format!(
        "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----",
        b64_data
    )
}

/// Sign data with Ed25519 - secure replacement for RSA signing
pub fn sign_ed25519(data: &[u8]) -> Signature {
    ED25519_KEYPAIR.sign(data)
}

/// Verify Ed25519 signature
pub fn verify_ed25519(
    data: &[u8],
    signature: &Signature,
) -> Result<(), ed25519_dalek::SignatureError> {
    let verifying_key = ED25519_KEYPAIR.verifying_key();
    verifying_key.verify(data, signature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ed25519_sign_verify() {
        let data = b"test message";
        let signature = sign_ed25519(data);
        assert!(verify_ed25519(data, &signature).is_ok());
    }

    #[test]
    fn test_ed25519_jwk_generation() {
        let jwk = get_ed25519_jwk();
        assert_eq!(jwk.kty, "OKP");
        assert_eq!(jwk.crv, "Ed25519");
        assert_eq!(jwk.alg, "EdDSA");
        assert!(!jwk.x.is_empty());
    }

    #[test]
    fn test_ed25519_pem_generation() {
        let pem = get_ed25519_public_pem();
        assert!(pem.starts_with("-----BEGIN PUBLIC KEY-----"));
        assert!(pem.ends_with("-----END PUBLIC KEY-----"));
    }
}
