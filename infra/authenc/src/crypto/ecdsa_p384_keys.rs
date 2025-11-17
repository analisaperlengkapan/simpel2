use base64ct::{Base64UrlUnpadded, Encoding};
use once_cell::sync::Lazy;
use p384::{
    PublicKey,
    ecdsa::{SigningKey, VerifyingKey, signature::Signer, signature::Verifier},
    elliptic_curve::sec1::ToEncodedPoint,
};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::env;

/// ECDSA P-384 keypair for JWT signing - enterprise-grade security
///
/// Production: Loads from ECDSA_P384_PRIVATE_KEY_BASE64 environment variable
/// Development: Generates ephemeral key with warning
pub static ECDSA_P384_KEYPAIR: Lazy<SigningKey> = Lazy::new(|| {
    // Try to load from environment variable (production)
    if let Ok(key_base64) = env::var("ECDSA_P384_PRIVATE_KEY_BASE64") {
        match load_key_from_base64(&key_base64) {
            Ok(key) => {
                tracing::info!(
                    "✅ ECDSA P-384 signing key loaded from ECDSA_P384_PRIVATE_KEY_BASE64"
                );
                return key;
            }
            Err(e) => {
                tracing::error!("❌ Failed to load ECDSA P-384 key: {}", e);
                tracing::error!("⚠️  Falling back to ephemeral key generation");
            }
        }
    }

    // Fallback: Generate ephemeral key (ONLY for development/testing)
    tracing::warn!("⚠️  ECDSA_P384_PRIVATE_KEY_BASE64 not set - generating ephemeral key");
    tracing::warn!("⚠️  NOT SUITABLE FOR PRODUCTION!");

    SigningKey::random(&mut OsRng)
});

/// JSON Web Key Set containing ECDSA P-384 public keys
#[derive(Debug, Serialize, Deserialize)]
pub struct EcdsaP384JwkSet {
    /// Array of JSON Web Keys
    pub keys: Vec<EcdsaP384Jwk>,
}

/// Individual ECDSA P-384 JSON Web Key
#[derive(Debug, Serialize, Deserialize)]
pub struct EcdsaP384Jwk {
    /// Key type (always "EC" for ECDSA)
    pub kty: String,
    /// Elliptic curve (always "P-384" for ECDSA P-384)
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
    /// Algorithm identifier ("ES384" for ECDSA P-384)
    pub alg: String,
}

impl EcdsaP384Jwk {
    /// Creates an ECDSA P-384 JWK from a verifying key and key ID.
    ///
    /// # Arguments
    /// * `verifying_key` - The ECDSA P-384 verifying key to convert
    /// * `kid` - The key ID to assign to this JWK
    ///
    /// # Returns
    /// A new `EcdsaP384Jwk` instance with the public key coordinates and metadata.
    pub fn from_verifying_key(verifying_key: &VerifyingKey, kid: &str) -> Self {
        let public_key = PublicKey::from(verifying_key);
        let encoded_point = public_key.to_encoded_point(false);

        let x = Base64UrlUnpadded::encode_string(encoded_point.x().unwrap());
        let y = Base64UrlUnpadded::encode_string(encoded_point.y().unwrap());

        Self {
            kty: "EC".to_string(),
            crv: "P-384".to_string(),
            x,
            y,
            kid: kid.to_string(),
            key_use: "sig".to_string(),
            alg: "ES384".to_string(),
        }
    }
}

/// Sign JWT with ECDSA P-384
pub fn sign_jwt_p384(header: &str, payload: &str) -> Result<String, String> {
    use base64ct::{Base64UrlUnpadded, Encoding};
    use p384::ecdsa::Signature;

    let header_b64 = Base64UrlUnpadded::encode_string(header.as_bytes());
    let payload_b64 = Base64UrlUnpadded::encode_string(payload.as_bytes());
    let message = format!("{}.{}", header_b64, payload_b64);

    let signature: Signature = ECDSA_P384_KEYPAIR.sign(message.as_bytes());
    let signature_b64 = Base64UrlUnpadded::encode_string(signature.to_bytes().as_slice());

    Ok(format!("{}.{}.{}", header_b64, payload_b64, signature_b64))
}

/// Verify JWT with ECDSA P-384
pub fn verify_jwt_p384(
    token: &str,
    verifying_key: &VerifyingKey,
) -> Result<(String, String), String> {
    use base64ct::Base64UrlUnpadded;
    use p384::ecdsa::Signature;

    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err("Invalid JWT format".to_string());
    }

    let header_b64 = parts[0];
    let payload_b64 = parts[1];
    let signature_b64 = parts[2];

    let message = format!("{}.{}", header_b64, payload_b64);

    let signature_bytes = Base64UrlUnpadded::decode_vec(signature_b64)
        .map_err(|e| format!("Invalid signature encoding: {}", e))?;

    let signature =
        Signature::from_slice(&signature_bytes).map_err(|e| format!("Invalid signature: {}", e))?;

    verifying_key
        .verify(message.as_bytes(), &signature)
        .map_err(|e| format!("Signature verification failed: {}", e))?;

    let header = String::from_utf8(
        Base64UrlUnpadded::decode_vec(header_b64)
            .map_err(|e| format!("Invalid header encoding: {}", e))?,
    )
    .map_err(|e| format!("Invalid header UTF-8: {}", e))?;

    let payload = String::from_utf8(
        Base64UrlUnpadded::decode_vec(payload_b64)
            .map_err(|e| format!("Invalid payload encoding: {}", e))?,
    )
    .map_err(|e| format!("Invalid payload UTF-8: {}", e))?;

    Ok((header, payload))
}

/// Get JWK Set for ECDSA P-384
pub fn get_p384_jwk_set() -> EcdsaP384JwkSet {
    let verifying_key = VerifyingKey::from(&*ECDSA_P384_KEYPAIR);
    let jwk = EcdsaP384Jwk::from_verifying_key(&verifying_key, "p384-key-1");

    EcdsaP384JwkSet { keys: vec![jwk] }
}

/// Load ECDSA P-384 signing key from base64-encoded bytes
fn load_key_from_base64(key_base64: &str) -> Result<SigningKey, String> {
    let key_bytes = base64ct::Base64::decode_vec(key_base64)
        .map_err(|e| format!("Base64 decode error: {}", e))?;

    SigningKey::from_slice(&key_bytes).map_err(|e| format!("Invalid ECDSA P-384 key: {}", e))
}

/// Generate a new ECDSA P-384 keypair and return base64-encoded private key
pub fn generate_new_p384_keypair() -> (SigningKey, String) {
    let signing_key = SigningKey::random(&mut OsRng);
    let private_key_bytes = signing_key.to_bytes();
    let private_key_base64 = base64ct::Base64::encode_string(&private_key_bytes);
    (signing_key, private_key_base64)
}
