//! Post-Quantum Cryptography (PQC) wrappers
//!
//! This module provides quantum-resistant cryptographic primitives. ML-DSA
//! (digital signatures, FIPS 204) and ML-KEM (key encapsulation, FIPS 203) come
//! from the RustCrypto `ml-dsa` and `ml-kem` crates; FALCON (compact signatures)
//! comes from `pqcrypto-falcon`, which has no RustCrypto counterpart.
//!
//! # Security Considerations
//! - These algorithms are designed to be resistant to quantum attacks
//! - ML-DSA and FALCON provide digital signature capabilities
//! - ML-KEM provides quantum-resistant key exchange
//! - All operations use cryptographically secure parameters
//! - Keys should be generated securely and stored safely
//! - Secret keys are zeroized on drop to prevent memory leaks

#[cfg(feature = "quantum")]
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
#[cfg(feature = "quantum")]
use ed25519_dalek::{self, Signer, SigningKey, Verifier, VerifyingKey};
#[cfg(feature = "quantum")]
use hkdf::Hkdf;
#[cfg(feature = "quantum")]
use pqcrypto_falcon::falcon512;
#[cfg(feature = "quantum")]
use rand::RngCore;
#[cfg(feature = "quantum")]
use sha2::Sha256;

/// Result type for PQC operations
pub type Result<T> = std::result::Result<T, PqcError>;

/// Errors that can occur during PQC operations
#[derive(Debug, thiserror::Error)]
pub enum PqcError {
    /// Invalid key format or corrupted key data
    #[error("Invalid key format or corrupted key data")]
    InvalidKey,

    /// Invalid signature format or corrupted signature
    #[error("Invalid signature format or corrupted signature")]
    InvalidSignature,

    /// Signature verification failed
    #[error("Signature verification failed")]
    VerificationFailed,

    /// Key generation failed
    #[error("Key generation failed")]
    KeyGenerationFailed,

    /// Encryption/decryption operation failed
    #[error("Encryption/decryption operation failed")]
    CryptoOperationFailed,

    /// Invalid input: {0}
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Feature not available: quantum cryptography requires 'quantum' feature flag
    #[error("Feature not available: quantum cryptography requires 'quantum' feature flag")]
    FeatureNotAvailable,
}

#[cfg(feature = "quantum")]
impl From<pqcrypto_traits::Error> for PqcError {
    fn from(err: pqcrypto_traits::Error) -> Self {
        match err {
            pqcrypto_traits::Error::BadLength { .. } => PqcError::InvalidKey,
            _ => PqcError::CryptoOperationFailed,
        }
    }
}

/// ML-DSA (Dilithium) digital signature wrapper, on the RustCrypto `ml-dsa` crate
///
/// The secret key is the 32-byte FIPS 204 seed, not the 2,560-byte expanded key:
/// `ml-dsa` loads the expanded form only through a deprecated call that does not
/// validate its input and panics on malformed bytes, which under `panic = "abort"`
/// ends the process. A seed is always valid.
#[cfg(feature = "quantum")]
pub mod mldsa {
    use super::*;
    use ml_dsa::{
        EncodedSignature, EncodedVerifyingKey, Generate, Keypair, MlDsa44, Seed, SigningKey,
        VerifyingKey, common::getrandom::SysRng,
    };

    /// Size of an ML-DSA-44 public key
    const PUBLIC_KEY_LEN: usize = 1312;
    /// Size of the secret key: the FIPS 204 seed
    const SECRET_KEY_LEN: usize = 32;
    /// Size of an ML-DSA-44 signature
    const SIGNATURE_LEN: usize = 2420;

    /// ML-DSA public key
    #[derive(Clone)]
    pub struct PublicKey {
        key: VerifyingKey<MlDsa44>,
        bytes: EncodedVerifyingKey<MlDsa44>,
    }

    /// ML-DSA secret key; the underlying key is wiped when it is dropped
    pub struct SecretKey {
        key: SigningKey<MlDsa44>,
    }

    /// ML-DSA signature, kept in its FIPS 204 encoding
    #[derive(Clone)]
    pub struct Signature(EncodedSignature<MlDsa44>);

    impl PublicKey {
        /// Create a public key from bytes
        pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
            if bytes.len() != PUBLIC_KEY_LEN {
                return Err(PqcError::InvalidInput(format!(
                    "Expected {} bytes for public key, got {}",
                    PUBLIC_KEY_LEN,
                    bytes.len()
                )));
            }
            let encoded = EncodedVerifyingKey::<MlDsa44>::try_from(bytes)
                .map_err(|_| PqcError::InvalidKey)?;
            Ok(Self::from_verifying_key(VerifyingKey::decode(&encoded)))
        }

        fn from_verifying_key(key: VerifyingKey<MlDsa44>) -> Self {
            let bytes = key.encode();
            PublicKey { key, bytes }
        }

        /// Export public key as bytes
        pub fn as_bytes(&self) -> &[u8] {
            self.bytes.as_slice()
        }

        /// Verify a signature
        pub fn verify(&self, message: &[u8], signature: &Signature) -> Result<()> {
            let signature = ml_dsa::Signature::<MlDsa44>::decode(&signature.0)
                .ok_or(PqcError::InvalidSignature)?;
            if self.key.verify_with_context(message, &[], &signature) {
                Ok(())
            } else {
                Err(PqcError::VerificationFailed)
            }
        }
    }

    impl SecretKey {
        /// Generate a new key pair
        pub fn new() -> Result<(PublicKey, SecretKey)> {
            let seed = Seed::try_generate().map_err(|_| PqcError::KeyGenerationFailed)?;
            let key = SigningKey::<MlDsa44>::from_seed(&seed);
            let public = PublicKey::from_verifying_key(key.verifying_key());
            Ok((public, SecretKey { key }))
        }

        /// Restore a secret key from its 32-byte seed, as exported by [`Self::as_bytes`].
        ///
        /// The public key is derived from the seed; `pk_bytes` must match it, which
        /// catches a seed and a public key that belong to different key pairs.
        pub fn from_bytes_with_public(sk_bytes: &[u8], pk_bytes: &[u8]) -> Result<Self> {
            if sk_bytes.len() != SECRET_KEY_LEN {
                return Err(PqcError::InvalidInput(format!(
                    "Expected {} bytes for secret key, got {}",
                    SECRET_KEY_LEN,
                    sk_bytes.len()
                )));
            }
            let claimed = PublicKey::from_bytes(pk_bytes)?;
            let seed = Seed::try_from(sk_bytes).map_err(|_| PqcError::InvalidKey)?;
            let key = SigningKey::<MlDsa44>::from_seed(&seed);
            if key.verifying_key().encode() != claimed.bytes {
                return Err(PqcError::InvalidKey);
            }
            Ok(SecretKey { key })
        }

        /// ⚠️ SECURITY WARNING: Export the secret key (the 32-byte seed)
        /// Only use for secure storage/transmission. Ensure zeroization after use.
        pub fn as_bytes(&self) -> Vec<u8> {
            self.key.to_seed().to_vec()
        }

        /// Sign a message (empty context string, hedged with fresh OS randomness)
        pub fn sign(&self, message: &[u8]) -> Result<Signature> {
            let signature = self
                .key
                .expanded_key()
                .sign_randomized(message, &[], &mut SysRng)
                .map_err(|_| PqcError::CryptoOperationFailed)?;
            Ok(Signature(signature.encode()))
        }

        /// Get the corresponding public key
        pub fn public_key(&self) -> PublicKey {
            PublicKey::from_verifying_key(self.key.verifying_key())
        }
    }

    impl Signature {
        /// Create a signature from bytes
        pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
            if bytes.len() != SIGNATURE_LEN {
                return Err(PqcError::InvalidInput(format!(
                    "Expected {} bytes for signature, got {}",
                    SIGNATURE_LEN,
                    bytes.len()
                )));
            }
            let encoded = EncodedSignature::<MlDsa44>::try_from(bytes)
                .map_err(|_| PqcError::InvalidSignature)?;
            // Refuse anything that is not a well-formed FIPS 204 signature up front.
            ml_dsa::Signature::<MlDsa44>::decode(&encoded).ok_or(PqcError::InvalidSignature)?;
            Ok(Signature(encoded))
        }

        /// Export signature as bytes
        pub fn as_bytes(&self) -> &[u8] {
            self.0.as_slice()
        }
    }

    /// Get the sizes of ML-DSA keys and signatures: (public key, secret key, signature)
    pub fn key_sizes() -> (usize, usize, usize) {
        (PUBLIC_KEY_LEN, SECRET_KEY_LEN, SIGNATURE_LEN)
    }
}

/// ML-KEM (Kyber) key encapsulation mechanism wrapper, on the RustCrypto `ml-kem` crate
///
/// The secret key is the 64-byte FIPS 203 seed (`d || z`), not the 2,400-byte
/// expanded decapsulation key, which RustCrypto deprecates.
#[cfg(feature = "quantum")]
pub mod mlkem {
    use super::*;
    use ml_kem::{
        DecapsulationKey, EncapsulationKey, Key, MlKem768, Seed,
        kem::{Ciphertext as KemCiphertext, Decapsulate, Encapsulate, Generate, KeyExport},
    };
    use zeroize::Zeroizing;

    /// Size of an ML-KEM-768 public key
    const PUBLIC_KEY_LEN: usize = 1184;
    /// Size of the secret key: the FIPS 203 seed
    const SECRET_KEY_LEN: usize = 64;
    /// Size of an ML-KEM-768 ciphertext
    const CIPHERTEXT_LEN: usize = 1088;
    /// Size of the shared secret
    const SHARED_SECRET_LEN: usize = 32;

    /// ML-KEM public key
    #[derive(Clone)]
    pub struct PublicKey {
        key: EncapsulationKey<MlKem768>,
        bytes: Key<EncapsulationKey<MlKem768>>,
    }

    /// ML-KEM secret key; the underlying key is wiped when it is dropped
    pub struct SecretKey {
        key: DecapsulationKey<MlKem768>,
    }

    /// ML-KEM ciphertext
    #[derive(Clone)]
    pub struct Ciphertext(KemCiphertext<MlKem768>);

    /// ML-KEM shared secret, wiped when it is dropped
    pub struct SharedSecret(Zeroizing<[u8; SHARED_SECRET_LEN]>);

    // Not derived: a derived `Debug` would print the secret into logs.
    impl std::fmt::Debug for SharedSecret {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("SharedSecret(..)")
        }
    }

    impl PublicKey {
        /// Create a public key from bytes
        pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
            if bytes.len() != PUBLIC_KEY_LEN {
                return Err(PqcError::InvalidInput(format!(
                    "Expected {} bytes for public key, got {}",
                    PUBLIC_KEY_LEN,
                    bytes.len()
                )));
            }
            let bytes = Key::<EncapsulationKey<MlKem768>>::try_from(bytes)
                .map_err(|_| PqcError::InvalidKey)?;
            // Runs the FIPS 203 encapsulation-key checks (coefficients reduced mod q).
            let key =
                EncapsulationKey::<MlKem768>::new(&bytes).map_err(|_| PqcError::InvalidKey)?;
            Ok(PublicKey { key, bytes })
        }

        fn from_encapsulation_key(key: &EncapsulationKey<MlKem768>) -> Self {
            PublicKey {
                bytes: key.to_bytes(),
                key: key.clone(),
            }
        }

        /// Export public key as bytes
        pub fn as_bytes(&self) -> &[u8] {
            self.bytes.as_slice()
        }

        /// Encapsulate a shared secret
        pub fn encapsulate(&self) -> Result<(Ciphertext, SharedSecret)> {
            let (ciphertext, shared) = self.key.encapsulate();
            Ok((Ciphertext(ciphertext), SharedSecret::from_array(&shared)))
        }
    }

    impl SecretKey {
        /// Generate a new key pair
        pub fn new() -> Result<(PublicKey, SecretKey)> {
            let seed = Seed::try_generate().map_err(|_| PqcError::KeyGenerationFailed)?;
            let key = DecapsulationKey::<MlKem768>::from_seed(seed);
            let public = PublicKey::from_encapsulation_key(key.encapsulation_key());
            Ok((public, SecretKey { key }))
        }

        /// Restore a secret key from its 64-byte seed, as exported by [`Self::as_bytes`].
        ///
        /// The public key is derived from the seed; `pk_bytes` must match it, which
        /// catches a seed and a public key that belong to different key pairs.
        pub fn from_bytes_with_public(sk_bytes: &[u8], pk_bytes: &[u8]) -> Result<Self> {
            if sk_bytes.len() != SECRET_KEY_LEN {
                return Err(PqcError::InvalidInput(format!(
                    "Expected {} bytes for secret key, got {}",
                    SECRET_KEY_LEN,
                    sk_bytes.len()
                )));
            }
            let claimed = PublicKey::from_bytes(pk_bytes)?;
            let seed = Seed::try_from(sk_bytes).map_err(|_| PqcError::InvalidKey)?;
            let key = DecapsulationKey::<MlKem768>::from_seed(seed);
            if key.encapsulation_key().to_bytes() != claimed.bytes {
                return Err(PqcError::InvalidKey);
            }
            Ok(SecretKey { key })
        }

        /// ⚠️ SECURITY WARNING: Export the secret key (the 64-byte seed)
        pub fn as_bytes(&self) -> Vec<u8> {
            // A key built by `new` or `from_bytes_with_public` always has its seed.
            self.key
                .to_seed()
                .map(|seed| seed.to_vec())
                .unwrap_or_default()
        }

        /// Decapsulate a shared secret from ciphertext
        pub fn decapsulate(&self, ciphertext: &Ciphertext) -> Result<SharedSecret> {
            let shared = self.key.decapsulate(&ciphertext.0);
            Ok(SharedSecret::from_array(&shared))
        }

        /// Get the corresponding public key
        pub fn public_key(&self) -> PublicKey {
            PublicKey::from_encapsulation_key(self.key.encapsulation_key())
        }
    }

    impl Ciphertext {
        /// Create a ciphertext from bytes
        pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
            if bytes.len() != CIPHERTEXT_LEN {
                return Err(PqcError::InvalidInput(format!(
                    "Expected {} bytes for ciphertext, got {}",
                    CIPHERTEXT_LEN,
                    bytes.len()
                )));
            }
            Ok(Ciphertext(
                KemCiphertext::<MlKem768>::try_from(bytes).map_err(|_| PqcError::InvalidKey)?,
            ))
        }

        /// Export ciphertext as bytes
        pub fn as_bytes(&self) -> &[u8] {
            self.0.as_slice()
        }
    }

    impl SharedSecret {
        fn from_array(bytes: &[u8]) -> Self {
            let mut secret = Zeroizing::new([0u8; SHARED_SECRET_LEN]);
            secret.copy_from_slice(bytes);
            SharedSecret(secret)
        }

        /// Create a shared secret from bytes
        pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
            if bytes.len() != SHARED_SECRET_LEN {
                return Err(PqcError::InvalidInput(format!(
                    "Expected {} bytes for shared secret, got {}",
                    SHARED_SECRET_LEN,
                    bytes.len()
                )));
            }
            Ok(Self::from_array(bytes))
        }

        /// Export shared secret as bytes (WARNING: Handle with care!)
        pub fn as_bytes(&self) -> &[u8] {
            &self.0[..]
        }
    }

    /// Constant-time comparison for SharedSecret
    impl PartialEq for SharedSecret {
        fn eq(&self, other: &Self) -> bool {
            use subtle::ConstantTimeEq;
            bool::from(self.as_bytes().ct_eq(other.as_bytes()))
        }
    }

    impl Eq for SharedSecret {}

    /// Get the sizes of ML-KEM keys and ciphertext:
    /// (public key, secret key, ciphertext, shared secret)
    pub fn key_sizes() -> (usize, usize, usize, usize) {
        (
            PUBLIC_KEY_LEN,
            SECRET_KEY_LEN,
            CIPHERTEXT_LEN,
            SHARED_SECRET_LEN,
        )
    }
}

/// FALCON compact signature wrapper
#[cfg(feature = "quantum")]
pub mod falcon {
    use super::*;

    /// FALCON public key
    #[derive(Clone)]
    pub struct PublicKey(falcon512::PublicKey);

    /// FALCON secret key with automatic zeroization
    pub struct SecretKey {
        secret: falcon512::SecretKey,
        public: falcon512::PublicKey,
    }

    impl Drop for SecretKey {
        fn drop(&mut self) {
            let bytes =
                <falcon512::SecretKey as pqcrypto_traits::sign::SecretKey>::as_bytes(&self.secret);

            // SAFETY: This unsafe block securely zeros out FALCON secret key material from memory
            // to prevent key leakage after the key is no longer needed.
            //
            // Safety invariants:
            // 1. bytes.as_ptr() points to valid memory owned by the FALCON SecretKey
            // 2. bytes.len() accurately represents the size of the memory region
            // 3. Zeroing memory is safe and does not create undefined behavior
            // 4. Exclusive access guaranteed via Drop's &mut self
            unsafe {
                std::ptr::write_bytes(bytes.as_ptr() as *mut u8, 0, bytes.len());
            }
        }
    }

    /// FALCON signature
    pub struct Signature(falcon512::DetachedSignature);

    impl PublicKey {
        /// Create a public key from bytes
        pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
            if bytes.len() != falcon512::public_key_bytes() {
                return Err(PqcError::InvalidInput(format!(
                    "Expected {} bytes for public key, got {}",
                    falcon512::public_key_bytes(),
                    bytes.len()
                )));
            }
            Ok(PublicKey(
                <falcon512::PublicKey as pqcrypto_traits::sign::PublicKey>::from_bytes(bytes)?,
            ))
        }

        /// Export public key as bytes
        pub fn as_bytes(&self) -> &[u8] {
            <falcon512::PublicKey as pqcrypto_traits::sign::PublicKey>::as_bytes(&self.0)
        }

        /// Verify a signature
        pub fn verify(&self, message: &[u8], signature: &Signature) -> Result<()> {
            falcon512::verify_detached_signature(&signature.0, message, &self.0)
                .map_err(|_| PqcError::VerificationFailed)
        }
    }

    impl SecretKey {
        /// Generate a new key pair with proper key storage
        pub fn new() -> Result<(PublicKey, SecretKey)> {
            let (pk, sk) = falcon512::keypair();
            let pk_clone = <falcon512::PublicKey as pqcrypto_traits::sign::PublicKey>::from_bytes(
                <falcon512::PublicKey as pqcrypto_traits::sign::PublicKey>::as_bytes(&pk),
            )?;
            Ok((
                PublicKey(pk_clone),
                SecretKey {
                    secret: sk,
                    public: pk,
                },
            ))
        }

        /// Create a secret key from bytes with public key
        pub fn from_bytes_with_public(sk_bytes: &[u8], pk_bytes: &[u8]) -> Result<Self> {
            if sk_bytes.len() != falcon512::secret_key_bytes() {
                return Err(PqcError::InvalidInput(format!(
                    "Expected {} bytes for secret key, got {}",
                    falcon512::secret_key_bytes(),
                    sk_bytes.len()
                )));
            }
            if pk_bytes.len() != falcon512::public_key_bytes() {
                return Err(PqcError::InvalidInput(format!(
                    "Expected {} bytes for public key, got {}",
                    falcon512::public_key_bytes(),
                    pk_bytes.len()
                )));
            }
            Ok(SecretKey {
                secret: <falcon512::SecretKey as pqcrypto_traits::sign::SecretKey>::from_bytes(
                    sk_bytes,
                )?,
                public: <falcon512::PublicKey as pqcrypto_traits::sign::PublicKey>::from_bytes(
                    pk_bytes,
                )?,
            })
        }

        /// ⚠️ SECURITY WARNING: Export secret key bytes
        pub fn as_bytes(&self) -> Vec<u8> {
            <falcon512::SecretKey as pqcrypto_traits::sign::SecretKey>::as_bytes(&self.secret)
                .to_vec()
        }

        /// Sign a message
        pub fn sign(&self, message: &[u8]) -> Result<Signature> {
            let sig = falcon512::detached_sign(message, &self.secret);
            Ok(Signature(sig))
        }

        /// Get the corresponding public key (properly stored)
        pub fn public_key(&self) -> PublicKey {
            PublicKey(
                <falcon512::PublicKey as pqcrypto_traits::sign::PublicKey>::from_bytes(
                    <falcon512::PublicKey as pqcrypto_traits::sign::PublicKey>::as_bytes(
                        &self.public,
                    ),
                )
                .expect("Public key bytes should always be valid"),
            )
        }
    }

    impl Signature {
        /// Create a signature from bytes
        pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
            if bytes.len() > falcon512::signature_bytes() {
                return Err(PqcError::InvalidInput(format!(
                    "Signature too long: got {} bytes, max {}",
                    bytes.len(),
                    falcon512::signature_bytes()
                )));
            }
            Ok(Signature(<falcon512::DetachedSignature as pqcrypto_traits::sign::DetachedSignature>::from_bytes(bytes)?))
        }

        /// Export signature as bytes
        pub fn as_bytes(&self) -> &[u8] {
            <falcon512::DetachedSignature as pqcrypto_traits::sign::DetachedSignature>::as_bytes(
                &self.0,
            )
        }
    }

    /// Get the sizes of FALCON keys and signatures
    pub fn key_sizes() -> (usize, usize, usize) {
        (
            falcon512::public_key_bytes(),
            falcon512::secret_key_bytes(),
            falcon512::signature_bytes(),
        )
    }
}

/// Hybrid cryptography combining classical and quantum algorithms
#[cfg(feature = "quantum")]
pub mod hybrid {
    use super::*;

    const AES_GCM_NONCE_SIZE: usize = 12;

    /// Hybrid key exchange using ML-KEM + HKDF for AES-GCM key derivation
    pub fn key_exchange(pk: &mlkem::PublicKey) -> Result<(mlkem::Ciphertext, Vec<u8>)> {
        let (ct, ss) = pk.encapsulate()?;

        // Use HKDF to derive AES key from shared secret
        let hk = Hkdf::<Sha256>::new(None, ss.as_bytes());
        let mut aes_key = [0u8; 32];
        hk.expand(b"AES-256-GCM-KEY-V1", &mut aes_key)
            .map_err(|_| PqcError::CryptoOperationFailed)?;

        Ok((ct, aes_key.to_vec()))
    }

    /// Hybrid encryption using ML-KEM + AES-GCM with automatic nonce generation
    /// Returns (ciphertext_kem, encrypted_data, nonce)
    /// The nonce MUST be transmitted alongside the encrypted data
    pub fn encrypt_hybrid(
        pk: &mlkem::PublicKey,
        plaintext: &[u8],
    ) -> Result<(mlkem::Ciphertext, Vec<u8>, [u8; AES_GCM_NONCE_SIZE])> {
        let (ct, aes_key_vec) = key_exchange(pk)?;
        let aes_key: [u8; 32] = aes_key_vec
            .try_into()
            .map_err(|_| PqcError::CryptoOperationFailed)?;

        // Generate cryptographically secure random nonce
        let mut nonce = [0u8; AES_GCM_NONCE_SIZE];
        use rand::rngs::OsRng;
        OsRng.fill_bytes(&mut nonce);

        let cipher = Aes256Gcm::new(&aes_key.into());
        let nonce_obj = Nonce::from_slice(&nonce);

        let ciphertext = cipher
            .encrypt(nonce_obj, plaintext)
            .map_err(|_| PqcError::CryptoOperationFailed)?;

        Ok((ct, ciphertext, nonce))
    }

    /// Hybrid decryption using ML-KEM + AES-GCM
    pub fn decrypt_hybrid(
        sk: &mlkem::SecretKey,
        ct: &mlkem::Ciphertext,
        ciphertext: &[u8],
        nonce: &[u8],
    ) -> Result<Vec<u8>> {
        if nonce.len() != AES_GCM_NONCE_SIZE {
            return Err(PqcError::InvalidInput(format!(
                "Invalid nonce size: expected {}, got {}",
                AES_GCM_NONCE_SIZE,
                nonce.len()
            )));
        }

        let ss = sk.decapsulate(ct)?;

        // Use HKDF to derive AES key
        let hk = Hkdf::<Sha256>::new(None, ss.as_bytes());
        let mut aes_key = [0u8; 32];
        hk.expand(b"AES-256-GCM-KEY-V1", &mut aes_key)
            .map_err(|_| PqcError::CryptoOperationFailed)?;

        let cipher = Aes256Gcm::new(&aes_key.into());
        let nonce_obj = Nonce::from_slice(nonce);

        cipher
            .decrypt(nonce_obj, ciphertext)
            .map_err(|_| PqcError::CryptoOperationFailed)
    }

    /// Hybrid signature using Ed25519 + ML-DSA with domain separation
    /// Applies different domain separators to prevent cross-protocol attacks
    pub fn sign_hybrid(
        message: &[u8],
        ed25519_sk: &SigningKey,
        mldsa_sk: &mldsa::SecretKey,
    ) -> Result<(ed25519_dalek::Signature, mldsa::Signature)> {
        // Domain separation: prepend protocol-specific tags
        let ed25519_msg = [b"HYBRID-ED25519-V1:", message].concat();
        let mldsa_msg = [b"HYBRID-MLDSA-V1:", message].concat();

        let ed25519_sig = ed25519_sk.sign(&ed25519_msg);
        let mldsa_sig = mldsa_sk.sign(&mldsa_msg)?;
        Ok((ed25519_sig, mldsa_sig))
    }

    /// Hybrid verification using Ed25519 + ML-DSA with domain separation
    pub fn verify_hybrid(
        message: &[u8],
        ed25519_sig: &ed25519_dalek::Signature,
        mldsa_sig: &mldsa::Signature,
        ed25519_pk: &VerifyingKey,
        mldsa_pk: &mldsa::PublicKey,
    ) -> Result<()> {
        // Apply same domain separation as in signing
        let ed25519_msg = [b"HYBRID-ED25519-V1:", message].concat();
        let mldsa_msg = [b"HYBRID-MLDSA-V1:", message].concat();

        // Verify Ed25519 signature
        ed25519_pk
            .verify(&ed25519_msg, ed25519_sig)
            .map_err(|_| PqcError::VerificationFailed)?;

        // Verify ML-DSA signature
        mldsa_pk.verify(&mldsa_msg, mldsa_sig)?;

        Ok(())
    }
}

// Non-quantum feature stubs for when quantum cryptography is not enabled
#[cfg(not(feature = "quantum"))]
/// Stub implementations of ML-DSA for non-quantum builds
pub mod mldsa {
    use super::*;

    /// Stub implementation of ML-DSA public key for non-quantum builds
    pub struct PublicKey;

    /// Stub implementation of ML-DSA secret key for non-quantum builds
    pub struct SecretKey;

    /// Stub implementation of ML-DSA signature for non-quantum builds
    pub struct Signature;

    impl PublicKey {
        /// Creates a public key from bytes (always fails in non-quantum builds)
        pub fn from_bytes(_bytes: &[u8]) -> Result<Self> {
            Err(PqcError::FeatureNotAvailable)
        }

        /// Returns the public key as bytes
        pub fn as_bytes(&self) -> &[u8] {
            &[]
        }

        /// Verifies a signature (always fails in non-quantum builds)
        pub fn verify(&self, _message: &[u8], _signature: &Signature) -> Result<()> {
            Err(PqcError::FeatureNotAvailable)
        }
    }

    impl SecretKey {
        /// Creates a new key pair (always fails in non-quantum builds)
        pub fn new() -> Result<(PublicKey, SecretKey)> {
            Err(PqcError::FeatureNotAvailable)
        }

        /// Creates a secret key from bytes (always fails in non-quantum builds)
        pub fn from_bytes(_bytes: &[u8]) -> Result<Self> {
            Err(PqcError::FeatureNotAvailable)
        }

        /// Returns the secret key as bytes
        pub fn as_bytes(&self) -> Vec<u8> {
            Vec::new()
        }

        /// Signs a message (always fails in non-quantum builds)
        pub fn sign(&self, _message: &[u8]) -> Result<Signature> {
            Err(PqcError::FeatureNotAvailable)
        }

        /// Returns the corresponding public key
        pub fn public_key(&self) -> PublicKey {
            PublicKey
        }
    }

    impl Signature {
        /// Creates a signature from bytes (always fails in non-quantum builds)
        pub fn from_bytes(_bytes: &[u8]) -> Result<Self> {
            Err(PqcError::FeatureNotAvailable)
        }

        /// Returns the signature as bytes
        pub fn as_bytes(&self) -> &[u8] {
            &[]
        }
    }

    /// Returns key sizes for Falcon (always returns zeros in non-quantum builds)
    pub fn key_sizes() -> (usize, usize, usize) {
        (0, 0, 0)
    }
}

#[cfg(not(feature = "quantum"))]
/// Stub implementations of ML-KEM for non-quantum builds
pub mod mlkem {
    use super::*;

    /// Stub implementation of ML-KEM public key for non-quantum builds
    pub struct PublicKey;

    /// Stub implementation of ML-KEM secret key for non-quantum builds
    pub struct SecretKey;

    /// Stub implementation of ML-KEM ciphertext for non-quantum builds
    pub struct Ciphertext;

    /// Stub implementation of ML-KEM shared secret for non-quantum builds
    pub struct SharedSecret;

    impl PublicKey {
        /// Creates a public key from bytes (always fails in non-quantum builds)
        pub fn from_bytes(_bytes: &[u8]) -> Result<Self> {
            Err(PqcError::FeatureNotAvailable)
        }

        /// Returns the public key as bytes
        pub fn as_bytes(&self) -> &[u8] {
            &[]
        }

        /// Encapsulates a shared secret (always fails in non-quantum builds)
        pub fn encapsulate(&self) -> Result<(Ciphertext, SharedSecret)> {
            Err(PqcError::FeatureNotAvailable)
        }
    }

    impl SecretKey {
        /// Creates a new key pair (always fails in non-quantum builds)
        pub fn new() -> Result<(PublicKey, SecretKey)> {
            Err(PqcError::FeatureNotAvailable)
        }

        /// Creates a secret key from bytes (always fails in non-quantum builds)
        pub fn from_bytes(_bytes: &[u8]) -> Result<Self> {
            Err(PqcError::FeatureNotAvailable)
        }

        /// Returns the secret key as bytes
        pub fn as_bytes(&self) -> Vec<u8> {
            Vec::new()
        }

        /// Decapsulates a shared secret (always fails in non-quantum builds)
        pub fn decapsulate(&self, _ciphertext: &Ciphertext) -> Result<SharedSecret> {
            Err(PqcError::FeatureNotAvailable)
        }

        /// Returns the corresponding public key
        pub fn public_key(&self) -> PublicKey {
            PublicKey
        }
    }

    impl Ciphertext {
        /// Creates a ciphertext from bytes (always fails in non-quantum builds)
        pub fn from_bytes(_bytes: &[u8]) -> Result<Self> {
            Err(PqcError::FeatureNotAvailable)
        }

        /// Returns the ciphertext as bytes
        pub fn as_bytes(&self) -> &[u8] {
            &[]
        }
    }

    impl SharedSecret {
        /// Creates a shared secret from bytes (always fails in non-quantum builds)
        pub fn from_bytes(_bytes: &[u8]) -> Result<Self> {
            Err(PqcError::FeatureNotAvailable)
        }

        /// Returns the shared secret as bytes
        pub fn as_bytes(&self) -> &[u8] {
            &[]
        }
    }

    impl PartialEq for SharedSecret {
        fn eq(&self, _other: &Self) -> bool {
            false
        }
    }

    impl Eq for SharedSecret {}

    /// Returns key sizes for ML-KEM (always returns zeros in non-quantum builds)
    pub fn key_sizes() -> (usize, usize, usize, usize) {
        (0, 0, 0, 0)
    }
}

#[cfg(not(feature = "quantum"))]
/// Stub implementations of Falcon for non-quantum builds
pub mod falcon {
    use super::*;

    /// Stub implementation of Falcon public key for non-quantum builds
    pub struct PublicKey;

    /// Stub implementation of Falcon secret key for non-quantum builds
    pub struct SecretKey;

    /// Stub implementation of Falcon signature for non-quantum builds
    pub struct Signature;

    impl PublicKey {
        /// Creates a public key from bytes (always fails in non-quantum builds)
        pub fn from_bytes(_bytes: &[u8]) -> Result<Self> {
            Err(PqcError::FeatureNotAvailable)
        }

        /// Returns the public key as bytes
        pub fn as_bytes(&self) -> &[u8] {
            &[]
        }

        /// Verifies a signature (always fails in non-quantum builds)
        pub fn verify(&self, _message: &[u8], _signature: &Signature) -> Result<()> {
            Err(PqcError::FeatureNotAvailable)
        }
    }

    impl SecretKey {
        /// Creates a new key pair (always fails in non-quantum builds)
        pub fn new() -> Result<(PublicKey, SecretKey)> {
            Err(PqcError::FeatureNotAvailable)
        }

        /// Creates a secret key from bytes (always fails in non-quantum builds)
        pub fn from_bytes(_bytes: &[u8]) -> Result<Self> {
            Err(PqcError::FeatureNotAvailable)
        }

        /// Returns the secret key as bytes
        pub fn as_bytes(&self) -> Vec<u8> {
            Vec::new()
        }

        /// Signs a message (always fails in non-quantum builds)
        pub fn sign(&self, _message: &[u8]) -> Result<Signature> {
            Err(PqcError::FeatureNotAvailable)
        }

        /// Returns the corresponding public key
        pub fn public_key(&self) -> PublicKey {
            PublicKey
        }
    }

    impl Signature {
        /// Creates a signature from bytes (always fails in non-quantum builds)
        pub fn from_bytes(_bytes: &[u8]) -> Result<Self> {
            Err(PqcError::FeatureNotAvailable)
        }

        /// Returns the signature as bytes
        pub fn as_bytes(&self) -> &[u8] {
            &[]
        }
    }

    /// Returns key sizes for Falcon (always returns zeros in non-quantum builds)
    pub fn key_sizes() -> (usize, usize, usize) {
        (0, 0, 0)
    }
}

#[cfg(not(feature = "quantum"))]
/// Stub implementations of hybrid cryptography for non-quantum builds
pub mod hybrid {
    use super::*;
    /// Performs key exchange (always fails in non-quantum builds)
    pub fn key_exchange(_pk: &mlkem::PublicKey) -> Result<(mlkem::Ciphertext, Vec<u8>)> {
        Err(PqcError::FeatureNotAvailable)
    }

    /// Encrypts data using hybrid cryptography (always fails in non-quantum builds)
    pub fn encrypt_hybrid(
        _pk: &mlkem::PublicKey,
        _plaintext: &[u8],
        _nonce: &[u8; 12],
    ) -> Result<(mlkem::Ciphertext, Vec<u8>)> {
        Err(PqcError::FeatureNotAvailable)
    }

    /// Decrypts data using hybrid cryptography (always fails in non-quantum builds)
    pub fn decrypt_hybrid(
        _sk: &mlkem::SecretKey,
        _ct: &mlkem::Ciphertext,
        _ciphertext: &[u8],
        _nonce: &[u8],
    ) -> Result<Vec<u8>> {
        Err(PqcError::FeatureNotAvailable)
    }
}

#[cfg(test)]
#[cfg(not(feature = "quantum"))]
mod tests_no_quantum {
    use super::*;

    #[test]
    fn test_feature_not_available() {
        assert!(matches!(
            mldsa::SecretKey::new(),
            Err(PqcError::FeatureNotAvailable)
        ));
        assert!(matches!(
            mlkem::SecretKey::new(),
            Err(PqcError::FeatureNotAvailable)
        ));
        assert!(matches!(
            falcon::SecretKey::new(),
            Err(PqcError::FeatureNotAvailable)
        ));
    }
}

#[cfg(test)]
#[cfg(feature = "quantum")]
mod tests_quantum {
    use super::*;

    #[test]
    fn test_mldsa_key_generation() {
        let (pk, sk) = mldsa::SecretKey::new().unwrap();
        assert!(!pk.as_bytes().is_empty());
        assert!(!sk.as_bytes().is_empty());
    }

    #[test]
    fn test_mldsa_sign_verify() {
        let (pk, sk) = mldsa::SecretKey::new().unwrap();
        let message = b"Hello, quantum world!";
        let signature = sk.sign(message).unwrap();
        assert!(!signature.as_bytes().is_empty());
        assert!(pk.verify(message, &signature).is_ok());
    }

    #[test]
    fn test_mldsa_sign_verify_wrong_message() {
        let (pk, sk) = mldsa::SecretKey::new().unwrap();
        let message = b"Hello, quantum world!";
        let wrong_message = b"Goodbye, quantum world!";
        let signature = sk.sign(message).unwrap();
        assert!(pk.verify(wrong_message, &signature).is_err());
    }

    #[test]
    fn test_mldsa_invalid_key_size() {
        let invalid_key = vec![0u8; 10];
        assert!(mldsa::PublicKey::from_bytes(&invalid_key).is_err());
    }

    #[test]
    fn test_mlkem_key_exchange() {
        let (pk, sk) = mlkem::SecretKey::new().unwrap();
        let (ciphertext, shared_secret) = pk.encapsulate().unwrap();
        let decrypted_secret = sk.decapsulate(&ciphertext).unwrap();
        // Compare bytes since SharedSecret doesn't implement Eq
        assert_eq!(shared_secret.as_bytes(), decrypted_secret.as_bytes());
    }

    #[test]
    fn test_mlkem_invalid_key_size() {
        let invalid_key = vec![0u8; 10];
        assert!(mlkem::PublicKey::from_bytes(&invalid_key).is_err());
    }

    #[test]
    fn test_falcon_key_generation() {
        let (pk, sk) = falcon::SecretKey::new().unwrap();
        assert!(!pk.as_bytes().is_empty());
        assert!(!sk.as_bytes().is_empty());
    }

    #[test]
    fn test_falcon_sign_verify() {
        let (pk, sk) = falcon::SecretKey::new().unwrap();
        let message = b"Falcon signature test";
        let signature = sk.sign(message).unwrap();
        assert!(!signature.as_bytes().is_empty());
        assert!(pk.verify(message, &signature).is_ok());
    }

    #[test]
    fn test_falcon_sign_verify_wrong_message() {
        let (pk, sk) = falcon::SecretKey::new().unwrap();
        let message = b"Falcon signature test";
        let wrong_message = b"Different message";
        let signature = sk.sign(message).unwrap();
        assert!(pk.verify(wrong_message, &signature).is_err());
    }

    #[test]
    fn test_hybrid_key_exchange() {
        let (pk, sk) = mlkem::SecretKey::new().unwrap();
        let (ciphertext, aes_key) = hybrid::key_exchange(&pk).unwrap();
        assert_eq!(aes_key.len(), 32); // AES-256 key size

        // Verify we can derive the same key
        let shared_secret = sk.decapsulate(&ciphertext).unwrap();
        use hkdf::Hkdf;
        use sha2::Sha256;
        let hk = Hkdf::<Sha256>::new(None, shared_secret.as_bytes());
        let mut derived_key = [0u8; 32];
        hk.expand(b"AES-256-GCM-KEY-V1", &mut derived_key).unwrap();
        assert_eq!(aes_key, derived_key.to_vec());
    }

    #[test]
    fn test_hybrid_encrypt_decrypt() {
        let (pk, sk) = mlkem::SecretKey::new().unwrap();
        let plaintext = b"Secret message for hybrid encryption";

        // Nonce is now auto-generated
        let (ciphertext, encrypted_data, nonce) = hybrid::encrypt_hybrid(&pk, plaintext).unwrap();
        let decrypted = hybrid::decrypt_hybrid(&sk, &ciphertext, &encrypted_data, &nonce).unwrap();

        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_hybrid_nonce_randomness() {
        let (pk, _sk) = mlkem::SecretKey::new().unwrap();
        let plaintext = b"Test";

        // Generate multiple encryptions, nonces should be different
        let (_, _, nonce1) = hybrid::encrypt_hybrid(&pk, plaintext).unwrap();
        let (_, _, nonce2) = hybrid::encrypt_hybrid(&pk, plaintext).unwrap();

        // Nonces should be different (extremely unlikely to be equal)
        assert_ne!(nonce1, nonce2);
    }

    #[test]
    fn test_hybrid_decrypt_invalid_nonce() {
        let (pk, sk) = mlkem::SecretKey::new().unwrap();
        let plaintext = b"Test";

        let (ciphertext, encrypted_data, _correct_nonce) =
            hybrid::encrypt_hybrid(&pk, plaintext).unwrap();

        // Try with invalid nonce size
        let invalid_nonce = [0u8; 8];
        assert!(hybrid::decrypt_hybrid(&sk, &ciphertext, &encrypted_data, &invalid_nonce).is_err());
    }

    #[test]
    fn test_hybrid_sign_verify() {
        use ed25519_dalek::{SigningKey, VerifyingKey};
        use rand::rngs::OsRng;

        let ed25519_sk = SigningKey::generate(&mut OsRng);
        let ed25519_pk = VerifyingKey::from(&ed25519_sk);
        let (mldsa_pk, mldsa_sk) = mldsa::SecretKey::new().unwrap();

        let message = b"Hybrid signature test";
        let (ed25519_sig, mldsa_sig) =
            hybrid::sign_hybrid(message, &ed25519_sk, &mldsa_sk).unwrap();

        assert!(
            hybrid::verify_hybrid(message, &ed25519_sig, &mldsa_sig, &ed25519_pk, &mldsa_pk)
                .is_ok()
        );
    }

    #[test]
    fn test_hybrid_verify_wrong_message() {
        use ed25519_dalek::{SigningKey, VerifyingKey};
        use rand::rngs::OsRng;

        let ed25519_sk = SigningKey::generate(&mut OsRng);
        let ed25519_pk = VerifyingKey::from(&ed25519_sk);
        let (mldsa_pk, mldsa_sk) = mldsa::SecretKey::new().unwrap();

        let message = b"Original message";
        let wrong_message = b"Tampered message";

        let (ed25519_sig, mldsa_sig) =
            hybrid::sign_hybrid(message, &ed25519_sk, &mldsa_sk).unwrap();

        assert!(
            hybrid::verify_hybrid(
                wrong_message,
                &ed25519_sig,
                &mldsa_sig,
                &ed25519_pk,
                &mldsa_pk
            )
            .is_err()
        );
    }

    #[test]
    fn test_key_sizes() {
        let (pk_size, sk_size, sig_size) = mldsa::key_sizes();
        assert!(pk_size > 0);
        assert!(sk_size > 0);
        assert!(sig_size > 0);

        let (pk_size, sk_size, ct_size, ss_size) = mlkem::key_sizes();
        assert!(pk_size > 0);
        assert!(sk_size > 0);
        assert!(ct_size > 0);
        assert!(ss_size > 0);

        let (pk_size, sk_size, sig_size) = falcon::key_sizes();
        assert!(pk_size > 0);
        assert!(sk_size > 0);
        assert!(sig_size > 0);
    }

    #[test]
    fn test_serialization_roundtrip() {
        // Test ML-DSA
        let (pk, sk) = mldsa::SecretKey::new().unwrap();
        let pk_bytes = pk.as_bytes();
        let sk_bytes = sk.as_bytes();
        let pk_restored = mldsa::PublicKey::from_bytes(pk_bytes).unwrap();
        let sk_restored = mldsa::SecretKey::from_bytes_with_public(&sk_bytes, pk_bytes).unwrap();

        let message = b"Serialization test";
        let sig = sk_restored.sign(message).unwrap();
        assert!(pk_restored.verify(message, &sig).is_ok());

        // Test ML-KEM
        let (pk, sk) = mlkem::SecretKey::new().unwrap();
        let pk_bytes = pk.as_bytes();
        let sk_bytes = sk.as_bytes();
        let pk_restored = mlkem::PublicKey::from_bytes(pk_bytes).unwrap();
        let sk_restored = mlkem::SecretKey::from_bytes_with_public(&sk_bytes, pk_bytes).unwrap();

        let (ct, ss1) = pk_restored.encapsulate().unwrap();
        let ss2 = sk_restored.decapsulate(&ct).unwrap();
        assert_eq!(ss1, ss2); // Now uses constant-time comparison

        // Test FALCON
        let (pk, sk) = falcon::SecretKey::new().unwrap();
        let pk_bytes = pk.as_bytes();
        let sk_bytes = sk.as_bytes();
        let pk_restored = falcon::PublicKey::from_bytes(pk_bytes).unwrap();
        let sk_restored = falcon::SecretKey::from_bytes_with_public(&sk_bytes, pk_bytes).unwrap();

        let sig = sk_restored.sign(message).unwrap();
        assert!(pk_restored.verify(message, &sig).is_ok());
    }

    #[test]
    fn test_public_key_derivation() {
        // Test that public_key() returns the correct key
        let (pk, sk) = mldsa::SecretKey::new().unwrap();
        let derived_pk = sk.public_key();

        // Sign with sk, verify with derived pk
        let message = b"Test public key derivation";
        let sig = sk.sign(message).unwrap();
        assert!(derived_pk.verify(message, &sig).is_ok());

        // Also verify with original pk
        assert!(pk.verify(message, &sig).is_ok());
    }

    #[test]
    fn test_constant_time_comparison() {
        // Test that SharedSecret uses constant-time comparison
        let (pk, sk) = mlkem::SecretKey::new().unwrap();
        let (ct, ss1) = pk.encapsulate().unwrap();
        let ss2 = sk.decapsulate(&ct).unwrap();

        // This should use constant-time comparison now
        assert_eq!(ss1, ss2);
        assert!(ss1 == ss2);
    }

    #[test]
    fn test_secret_keys_are_seeds_that_restore_the_key_pair() {
        let (pk, sk) = mldsa::SecretKey::new().unwrap();
        let seed = sk.as_bytes();
        assert_eq!(seed.len(), 32);
        let restored = mldsa::SecretKey::from_bytes_with_public(&seed, pk.as_bytes()).unwrap();
        assert_eq!(restored.public_key().as_bytes(), pk.as_bytes());
        let sig = restored.sign(b"from the seed").unwrap();
        assert!(pk.verify(b"from the seed", &sig).is_ok());

        let (pk, sk) = mlkem::SecretKey::new().unwrap();
        let seed = sk.as_bytes();
        assert_eq!(seed.len(), 64);
        let restored = mlkem::SecretKey::from_bytes_with_public(&seed, pk.as_bytes()).unwrap();
        let (ct, shared) = pk.encapsulate().unwrap();
        assert_eq!(restored.decapsulate(&ct).unwrap(), shared);
    }

    #[test]
    fn test_a_seed_and_a_public_key_of_different_pairs_are_refused() {
        let (_, sk_a) = mldsa::SecretKey::new().unwrap();
        let (pk_b, _) = mldsa::SecretKey::new().unwrap();
        assert!(
            mldsa::SecretKey::from_bytes_with_public(&sk_a.as_bytes(), pk_b.as_bytes()).is_err()
        );

        let (_, sk_a) = mlkem::SecretKey::new().unwrap();
        let (pk_b, _) = mlkem::SecretKey::new().unwrap();
        assert!(
            mlkem::SecretKey::from_bytes_with_public(&sk_a.as_bytes(), pk_b.as_bytes()).is_err()
        );
    }

    /// The expanded keys of the previous implementation (2,560 and 2,400 bytes) must be
    /// refused with an error, not loaded: `ml-dsa`'s loader for them can panic.
    #[test]
    fn test_old_expanded_secret_keys_are_refused() {
        let (pk, _) = mldsa::SecretKey::new().unwrap();
        assert!(mldsa::SecretKey::from_bytes_with_public(&[0xFF; 2560], pk.as_bytes()).is_err());
        let (pk, _) = mlkem::SecretKey::new().unwrap();
        assert!(mlkem::SecretKey::from_bytes_with_public(&[0xFF; 2400], pk.as_bytes()).is_err());
    }

    #[test]
    fn test_key_sizes_are_the_fips_sizes() {
        // Public key, signature, ciphertext and shared secret are the FIPS 203/204 wire
        // sizes; the secret key is the seed.
        assert_eq!(mldsa::key_sizes(), (1312, 32, 2420));
        assert_eq!(mlkem::key_sizes(), (1184, 64, 1088, 32));
        let (pk, sk) = mldsa::SecretKey::new().unwrap();
        let sig = sk.sign(b"m").unwrap();
        assert_eq!(pk.as_bytes().len(), 1312);
        assert_eq!(sig.as_bytes().len(), 2420);
        let (pk, _) = mlkem::SecretKey::new().unwrap();
        let (ct, shared) = pk.encapsulate().unwrap();
        assert_eq!(pk.as_bytes().len(), 1184);
        assert_eq!(ct.as_bytes().len(), 1088);
        assert_eq!(shared.as_bytes().len(), 32);
    }

    #[test]
    fn test_mldsa_signatures_are_hedged_and_tampering_is_caught() {
        let (pk, sk) = mldsa::SecretKey::new().unwrap();
        let first = sk.sign(b"same message").unwrap();
        let second = sk.sign(b"same message").unwrap();
        assert_ne!(first.as_bytes(), second.as_bytes());
        assert!(pk.verify(b"same message", &first).is_ok());
        assert!(pk.verify(b"same message", &second).is_ok());

        // The first bytes are the challenge hash, so the signature stays well formed
        // but no longer verifies.
        let mut tampered = first.as_bytes().to_vec();
        tampered[0] ^= 0x01;
        let tampered = mldsa::Signature::from_bytes(&tampered).unwrap();
        assert!(pk.verify(b"same message", &tampered).is_err());
    }

    #[test]
    fn test_mlkem_public_key_with_unreduced_coefficients_is_refused() {
        assert!(mlkem::PublicKey::from_bytes(&[0xFF; 1184]).is_err());
    }

    #[test]
    fn test_shared_secret_debug_does_not_print_the_secret() {
        let (pk, _) = mlkem::SecretKey::new().unwrap();
        let (_, shared) = pk.encapsulate().unwrap();
        let printed = format!("{shared:?}");
        assert_eq!(printed, "SharedSecret(..)");
    }
}
