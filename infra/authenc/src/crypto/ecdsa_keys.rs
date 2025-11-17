use base64ct::{Base64UrlUnpadded, Encoding};
use once_cell::sync::Lazy;
use p256::{
    PublicKey, SecretKey,
    ecdsa::{Signature, SigningKey, VerifyingKey, signature::Signer, signature::Verifier},
    elliptic_curve::sec1::ToEncodedPoint,
    pkcs8::{EncodePrivateKey, EncodePublicKey},
};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::env;

/// ECDSA P-256 keypair for JWT signing - secure alternative to RSA
///
/// Production: Loads from ECDSA_P256_PRIVATE_KEY_BASE64 environment variable
/// Development: Generates ephemeral key with warning
pub static ECDSA_KEYPAIR: Lazy<SigningKey> = Lazy::new(|| {
    // Try to load from environment variable (production)
    if let Ok(key_base64) = env::var("ECDSA_P256_PRIVATE_KEY_BASE64") {
        match load_key_from_base64(&key_base64) {
            Ok(key) => {
                tracing::info!(
                    "✅ ECDSA P-256 signing key loaded from ECDSA_P256_PRIVATE_KEY_BASE64"
                );
                return key;
            }
            Err(e) => {
                tracing::error!("❌ Failed to load ECDSA P-256 key: {}", e);
                tracing::error!("⚠️  Falling back to ephemeral key generation");
            }
        }
    }

    // Fallback: Generate ephemeral key (ONLY for development/testing)
    tracing::warn!("⚠️  ECDSA_P256_PRIVATE_KEY_BASE64 not set - generating ephemeral key");
    tracing::warn!("⚠️  NOT SUITABLE FOR PRODUCTION!");

    SigningKey::random(&mut OsRng)
});

/// Load ECDSA P-256 signing key from base64-encoded bytes
fn load_key_from_base64(key_base64: &str) -> Result<SigningKey, String> {
    let key_bytes = base64ct::Base64::decode_vec(key_base64)
        .map_err(|e| format!("Base64 decode error: {}", e))?;

    SigningKey::from_slice(&key_bytes).map_err(|e| format!("Invalid ECDSA P-256 key: {}", e))
}

/// Generate a new ECDSA P-256 keypair and return base64-encoded private key
pub fn generate_new_p256_keypair() -> (SigningKey, String) {
    let signing_key = SigningKey::random(&mut OsRng);
    let private_key_bytes = signing_key.to_bytes();
    let private_key_base64 = base64ct::Base64::encode_string(&private_key_bytes);
    (signing_key, private_key_base64)
}

/// JSON Web Key Set containing ECDSA public keys
#[derive(Debug, Serialize, Deserialize)]
pub struct EcdsaJwkSet {
    /// Array of JSON Web Keys
    pub keys: Vec<EcdsaJwk>,
}

/// Individual ECDSA JSON Web Key
#[derive(Debug, Serialize, Deserialize)]
pub struct EcdsaJwk {
    /// Key type (always "EC" for ECDSA)
    pub kty: String,
    /// Elliptic curve (always "P-256" for ECDSA P-256)
    pub crv: String,
    /// Base64URL-encoded x coordinate of the public key
    pub x: String,
    /// Base64URL-encoded y coordinate of the public key
    pub y: String,
    /// Key ID for key identification
    pub kid: String,
    /// Intended use of the key ("sig" for signing)
    #[serde(rename = "use")]
    pub key_use: String,
    /// Algorithm identifier ("ES256" for ECDSA P-256)
    pub alg: String,
}

impl EcdsaJwk {
    /// Creates an ECDSA JWK from a verifying key and key ID.
    ///
    /// # Arguments
    /// * `verifying_key` - The ECDSA P-256 verifying key to convert
    /// * `kid` - The key ID to assign to this JWK
    ///
    /// # Returns
    /// A new `EcdsaJwk` instance with the public key coordinates and metadata.
    pub fn from_verifying_key(verifying_key: &VerifyingKey, kid: &str) -> Self {
        let public_key = PublicKey::from(verifying_key);
        let encoded_point = public_key.to_encoded_point(false);

        let x = Base64UrlUnpadded::encode_string(encoded_point.x().unwrap());
        let y = Base64UrlUnpadded::encode_string(encoded_point.y().unwrap());

        Self {
            kty: "EC".to_string(),
            crv: "P-256".to_string(),
            x,
            y,
            kid: kid.to_string(),
            key_use: "sig".to_string(),
            alg: "ES256".to_string(),
        }
    }
}

/// Get the ECDSA public key in JWK format
pub fn get_ecdsa_jwk() -> EcdsaJwk {
    let verifying_key = ECDSA_KEYPAIR.verifying_key();
    EcdsaJwk::from_verifying_key(verifying_key, "authence-ecdsa-key")
}

/// Get the ECDSA public key in PEM format
pub fn get_ecdsa_public_pem() -> Result<String, Box<dyn std::error::Error>> {
    let verifying_key = ECDSA_KEYPAIR.verifying_key();
    let public_key = PublicKey::from(verifying_key);
    Ok(public_key.to_public_key_pem(Default::default())?)
}

/// Get the ECDSA private key in PEM format (for testing)
pub fn get_ecdsa_private_pem() -> Result<String, Box<dyn std::error::Error>> {
    let secret_key = SecretKey::from(&*ECDSA_KEYPAIR);
    Ok(secret_key.to_pkcs8_pem(Default::default())?.to_string())
}

/// Sign data with ECDSA P-256 - secure replacement for RSA signing
pub fn sign_ecdsa(data: &[u8]) -> Result<Signature, ecdsa::Error> {
    ECDSA_KEYPAIR.try_sign(data)
}

/// Verify ECDSA signature
pub fn verify_ecdsa(data: &[u8], signature: &Signature) -> Result<(), ecdsa::Error> {
    let verifying_key = ECDSA_KEYPAIR.verifying_key();
    verifying_key.verify(data, signature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ecdsa_sign_verify() {
        let data = b"test message";
        let signature = sign_ecdsa(data).unwrap();
        assert!(verify_ecdsa(data, &signature).is_ok());
    }

    #[test]
    fn test_ecdsa_jwk_generation() {
        let jwk = get_ecdsa_jwk();
        assert_eq!(jwk.kty, "EC");
        assert_eq!(jwk.crv, "P-256");
        assert_eq!(jwk.alg, "ES256");
        assert!(!jwk.x.is_empty());
        assert!(!jwk.y.is_empty());
    }

    #[test]
    fn test_ecdsa_pem_generation() {
        let pem = get_ecdsa_public_pem().unwrap();
        assert!(pem.starts_with("-----BEGIN PUBLIC KEY-----"));
        assert!(pem.ends_with("-----END PUBLIC KEY-----\n"));
    }

    #[test]
    fn test_ecdsa_private_pem_generation() {
        let pem = get_ecdsa_private_pem().unwrap();
        assert!(pem.starts_with("-----BEGIN PRIVATE KEY-----"));
        assert!(pem.ends_with("-----END PRIVATE KEY-----\n"));
    }
}
