//! ML-DSA (Module-Lattice Digital Signature Algorithm) Implementation
//!
//! NIST PQC Standard for Post-Quantum Digital Signatures (FIPS 204), on the
//! RustCrypto `ml-dsa` crate.
//!
//! # Private keys are seeds
//!
//! `MLDsaKeypair::private_key` holds the 32-byte FIPS 204 seed (`xi`), the same
//! length for every variant, not the 2,560 / 4,032 / 4,896-byte expanded signing
//! key. The signing key is re-derived from the seed when it is needed. This is a
//! safety decision as much as a size one: `ml-dsa` can load the expanded form only
//! through a deprecated call that does not validate its input and panics on
//! malformed bytes (every one of 200 random 2,560-byte blobs did), and this
//! workspace builds with `panic = "abort"`, so a corrupted key would take the whole
//! process down. A seed is always valid. The public key and the signature are the
//! FIPS 204 byte strings and interoperate with any other FIPS 204 implementation.
//!
//! Because the seed has the same length in every variant, a private key no longer
//! tells you which variant it belongs to by its length alone: keep the `variant`
//! next to it, as `MLDsaKeypair` does.
//!
//! # Signatures are hedged
//!
//! Every signature mixes 32 fresh random bytes from the operating system into the
//! signing process (FIPS 204's default, "hedged" variant), so signing the same
//! message twice gives two different, equally valid signatures.

use crate::error::{CryptoError, CryptoResult};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Length of the FIPS 204 seed, identical for all three parameter sets.
const SEED_LEN: usize = 32;

/// One entry point per operation over the three RustCrypto parameter sets.
mod raw {
    use super::{CryptoError, CryptoResult, MLDsaVariant, SEED_LEN};
    use ml_dsa::{
        EncodedVerifyingKey, Generate, Keypair, MlDsaParams, Seed, Signature, SigningKey,
        VerifyingKey, common::getrandom::SysRng,
    };
    use zeroize::Zeroizing;

    /// Run `$body` with `$p` bound to the `ml_dsa` parameter set of the variant.
    macro_rules! per_variant {
        ($variant:expr, $p:ident => $body:block) => {
            match $variant {
                MLDsaVariant::MLDsa44 => {
                    type $p = ml_dsa::MlDsa44;
                    $body
                }
                MLDsaVariant::MLDsa65 => {
                    type $p = ml_dsa::MlDsa65;
                    $body
                }
                MLDsaVariant::MLDsa87 => {
                    type $p = ml_dsa::MlDsa87;
                    $body
                }
            }
        };
    }

    /// A fresh seed from the operating system's random number generator.
    pub(super) fn random_seed() -> CryptoResult<Zeroizing<[u8; SEED_LEN]>> {
        <[u8; SEED_LEN]>::try_generate()
            .map(Zeroizing::new)
            .map_err(|_| CryptoError::RandomGenerationFailed)
    }

    /// Seed bytes -> signing key. Always succeeds for a 32-byte seed.
    fn signing_key<P: MlDsaParams>(seed: &[u8]) -> CryptoResult<SigningKey<P>> {
        let seed = Seed::try_from(seed).map_err(|_| {
            CryptoError::InvalidKey(format!(
                "Invalid private key size: expected {SEED_LEN} bytes, got {} bytes",
                seed.len()
            ))
        })?;
        Ok(SigningKey::<P>::from_seed(&seed))
    }

    /// The public key the seed belongs to.
    pub(super) fn public_key_of(variant: MLDsaVariant, seed: &[u8]) -> CryptoResult<Vec<u8>> {
        per_variant!(variant, P => {
            Ok(signing_key::<P>(seed)?.verifying_key().encode().to_vec())
        })
    }

    /// A detached signature over every message, with the key derived once.
    pub(super) fn sign_all(
        variant: MLDsaVariant,
        seed: &[u8],
        messages: &[&[u8]],
    ) -> CryptoResult<Vec<Vec<u8>>> {
        per_variant!(variant, P => {
            let key = signing_key::<P>(seed)?;
            let key = key.expanded_key();
            messages
                .iter()
                .map(|message| {
                    // Empty context string; hedged with fresh OS randomness.
                    key.sign_randomized(message, &[], &mut SysRng)
                        .map(|signature| signature.encode().to_vec())
                        .map_err(|_| CryptoError::SigningFailed("Signing failed".to_string()))
                })
                .collect()
        })
    }

    /// Whether each `(message, signature)` pair verifies under the public key.
    ///
    /// A signature that is not even well formed is an error, not `false`.
    pub(super) fn verify_all(
        variant: MLDsaVariant,
        public_key: &[u8],
        pairs: &[(&[u8], &[u8])],
    ) -> CryptoResult<Vec<bool>> {
        per_variant!(variant, P => {
            let encoded = EncodedVerifyingKey::<P>::try_from(public_key)
                .map_err(|_| CryptoError::InvalidKey("Key validation failed".to_string()))?;
            let key = VerifyingKey::<P>::decode(&encoded);
            pairs
                .iter()
                .map(|(message, signature)| {
                    let signature = Signature::<P>::try_from(*signature).map_err(|_| {
                        CryptoError::InvalidSignature("Signature format invalid".to_string())
                    })?;
                    Ok(key.verify_with_context(message, &[], &signature))
                })
                .collect()
        })
    }
}

/// ML-DSA variant configurations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MLDsaVariant {
    /// ML-DSA-44: 128-bit security, 1,312-byte public key, 2,420-byte signature
    MLDsa44,
    /// ML-DSA-65: 192-bit security, 1,952-byte public key, 3,300-byte signature
    MLDsa65,
    /// ML-DSA-87: 256-bit security, 2,592-byte public key, 4,611-byte signature
    MLDsa87,
}

impl MLDsaVariant {
    /// Get the public key size in bytes for this variant
    pub fn public_key_size(&self) -> usize {
        match self {
            MLDsaVariant::MLDsa44 => 1312,
            MLDsaVariant::MLDsa65 => 1952,
            MLDsaVariant::MLDsa87 => 2592,
        }
    }

    /// Get the private key size in bytes for this variant
    ///
    /// The private key is the 32-byte FIPS 204 seed, the same for every variant
    /// (see the module documentation).
    pub fn private_key_size(&self) -> usize {
        SEED_LEN
    }

    /// Get the detached signature size in bytes for this variant
    ///
    /// These are the fixed sizes from the FIPS 204 specification.
    pub fn signature_size(&self) -> usize {
        match self {
            MLDsaVariant::MLDsa44 => 2420,
            MLDsaVariant::MLDsa65 => 3309,
            MLDsaVariant::MLDsa87 => 4627,
        }
    }

    /// Get the maximum signature size (upper bound for allocation)
    pub fn max_signature_size(&self) -> usize {
        self.signature_size() + 50 // Add 50 bytes buffer for encoding overhead
    }

    /// Get the security level name
    pub fn security_level(&self) -> &'static str {
        match self {
            MLDsaVariant::MLDsa44 => "128-bit",
            MLDsaVariant::MLDsa65 => "192-bit",
            MLDsaVariant::MLDsa87 => "256-bit",
        }
    }
}

impl fmt::Display for MLDsaVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MLDsaVariant::MLDsa44 => write!(f, "ML-DSA-44"),
            MLDsaVariant::MLDsa65 => write!(f, "ML-DSA-65"),
            MLDsaVariant::MLDsa87 => write!(f, "ML-DSA-87"),
        }
    }
}

/// ML-DSA keypair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLDsaKeypair {
    /// Public key
    pub public_key: Vec<u8>,
    /// Private key: the 32-byte FIPS 204 seed (see the module docs)
    pub private_key: Vec<u8>,
    /// Algorithm variant
    pub variant: MLDsaVariant,
}

impl MLDsaKeypair {
    /// Generate a new ML-DSA keypair
    pub fn generate(variant: MLDsaVariant) -> CryptoResult<Self> {
        let seed = raw::random_seed()?;
        Self::from_seed(variant, &seed[..])
    }

    /// Rebuild a keypair from its 32-byte seed; the public key is derived from it.
    ///
    /// The seed is the `private_key` of a keypair, so this is how a keypair is
    /// recovered from the private key alone.
    pub fn from_seed(variant: MLDsaVariant, seed: &[u8]) -> CryptoResult<Self> {
        let public_key = raw::public_key_of(variant, seed)?;
        Ok(Self {
            public_key,
            private_key: seed.to_vec(),
            variant,
        })
    }

    /// Sign a message and return detached signature
    ///
    /// # Arguments
    /// * `message` - The message to sign (max 100 MB recommended)
    ///
    /// # Security
    /// - Uses cryptographically secure randomness internally
    /// - Returns DETACHED signature (signature only, not message)
    /// - Constant-time implementation resistant to timing attacks
    pub fn sign(&self, message: &[u8]) -> CryptoResult<Vec<u8>> {
        // Input validation: Reasonable message size limit (100 MB)
        const MAX_MESSAGE_SIZE: usize = 100 * 1024 * 1024;
        if message.len() > MAX_MESSAGE_SIZE {
            return Err(CryptoError::InvalidInput(format!(
                "Message too large: {} bytes (max: {} bytes)",
                message.len(),
                MAX_MESSAGE_SIZE
            )));
        }

        // Validate private key size
        if self.private_key.len() != self.variant.private_key_size() {
            return Err(CryptoError::InvalidKey(format!(
                "Invalid private key size: expected {} bytes, got {} bytes",
                self.variant.private_key_size(),
                self.private_key.len()
            )));
        }

        let mut signatures = raw::sign_all(self.variant, &self.private_key, &[message])?;
        signatures
            .pop()
            .ok_or_else(|| CryptoError::Internal("Signing produced no output".to_string()))
    }

    /// Verify a detached signature
    ///
    /// # Arguments
    /// * `message` - The original message
    /// * `signature` - The detached signature to verify
    ///
    /// # Returns
    /// * `Ok(true)` if signature is valid
    /// * `Ok(false)` if signature is invalid
    /// * `Err(...)` if inputs are malformed
    ///
    /// # Security
    /// - Constant-time verification
    /// - Returns false for invalid signatures (doesn't leak why)
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> CryptoResult<bool> {
        // Input validation: Check message size
        const MAX_MESSAGE_SIZE: usize = 100 * 1024 * 1024;
        if message.len() > MAX_MESSAGE_SIZE {
            return Err(CryptoError::InvalidInput("Message too large".to_string()));
        }

        // Validate public key size
        if self.public_key.len() != self.variant.public_key_size() {
            return Err(CryptoError::InvalidKey(
                "Invalid public key size".to_string(),
            ));
        }

        // Validate signature size (approximate check - some variance allowed)
        let expected_sig_size = self.variant.signature_size();
        if signature.len() < expected_sig_size - 50 || signature.len() > expected_sig_size + 50 {
            return Err(CryptoError::InvalidSignature(format!(
                "Signature size out of range: {} bytes (expected ~{})",
                signature.len(),
                expected_sig_size
            )));
        }

        let mut results = raw::verify_all(self.variant, &self.public_key, &[(message, signature)])?;
        results
            .pop()
            .ok_or_else(|| CryptoError::Internal("Verification produced no output".to_string()))
    }

    /// Get the public key as bytes
    pub fn public_key_bytes(&self) -> &[u8] {
        &self.public_key
    }

    /// Get the private key as bytes
    pub fn private_key_bytes(&self) -> &[u8] {
        &self.private_key
    }

    /// Get the variant
    pub fn variant(&self) -> MLDsaVariant {
        self.variant
    }
}

/// ML-DSA signature provider implementing the PostQuantumSignatures trait
pub struct MLDsaProvider {
    variant: MLDsaVariant,
}

impl MLDsaProvider {
    /// Create a new ML-DSA provider with the specified variant
    pub fn new(variant: MLDsaVariant) -> Self {
        Self { variant }
    }

    /// Generate a new keypair
    pub fn generate_keypair(&self) -> CryptoResult<MLDsaKeypair> {
        MLDsaKeypair::generate(self.variant)
    }

    /// Create provider from existing keypair
    pub fn from_keypair(keypair: MLDsaKeypair) -> Self {
        Self {
            variant: keypair.variant,
        }
    }
}

impl crate::pqc::PostQuantumSignatures for MLDsaProvider {
    fn algorithm_id(&self) -> &'static str {
        match self.variant {
            MLDsaVariant::MLDsa44 => "ML-DSA-44",
            MLDsaVariant::MLDsa65 => "ML-DSA-65",
            MLDsaVariant::MLDsa87 => "ML-DSA-87",
        }
    }

    fn keypair_generate(&self) -> crate::pqc::PQCResult<(Vec<u8>, Vec<u8>)> {
        let keypair = MLDsaKeypair::generate(self.variant)?;
        Ok((keypair.public_key, keypair.private_key))
    }

    fn public_key_size(&self) -> usize {
        self.variant.public_key_size()
    }

    fn private_key_size(&self) -> usize {
        self.variant.private_key_size()
    }

    fn sign(&self, message: &[u8], private_key: &[u8]) -> crate::pqc::PQCResult<Vec<u8>> {
        let keypair = MLDsaKeypair {
            public_key: Vec::new(), // Not needed for signing
            private_key: private_key.to_vec(),
            variant: self.variant,
        };
        keypair
            .sign(message)
            .map_err(|e| crate::pqc::PQCError::SignatureError(e.to_string()))
    }

    fn verify(
        &self,
        message: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> crate::pqc::PQCResult<bool> {
        let keypair = MLDsaKeypair {
            public_key: public_key.to_vec(),
            private_key: Vec::new(), // Not needed for verification
            variant: self.variant,
        };
        keypair
            .verify(message, signature)
            .map_err(|e| crate::pqc::PQCError::VerificationError(e.to_string()))
    }

    fn signature_size(&self) -> usize {
        self.variant.signature_size()
    }
}

/// ML-DSA signature batch operations for performance
pub struct MLDsaBatchSigner {
    private_key: Vec<u8>,
    variant: MLDsaVariant,
}

impl MLDsaBatchSigner {
    /// Create a new batch signer
    ///
    /// # Security Warning
    /// Batch operations reuse the same private key. Each signature still draws
    /// fresh randomness from the operating system (hedged signing).
    pub fn new(private_key: Vec<u8>, variant: MLDsaVariant) -> CryptoResult<Self> {
        // Validate private key size at construction
        if private_key.len() != variant.private_key_size() {
            return Err(CryptoError::InvalidKey(format!(
                "Invalid private key size: expected {} bytes, got {} bytes",
                variant.private_key_size(),
                private_key.len()
            )));
        }

        Ok(Self {
            private_key,
            variant,
        })
    }

    /// Sign multiple messages efficiently (returns detached signatures)
    ///
    /// # Arguments
    /// * `messages` - Slice of messages to sign
    ///
    /// # Performance
    /// Pre-allocates result vector for better performance.
    /// Consider using parallel processing for large batches (requires rayon).
    ///
    /// # Security
    /// - Each signature uses fresh randomness (hedged ML-DSA, drawn from the OS)
    /// - Recommended batch size: ≤ 1000 messages
    /// - Rate limiting recommended for production use
    pub fn batch_sign(&self, messages: &[&[u8]]) -> CryptoResult<Vec<Vec<u8>>> {
        // Recommended batch size limit
        const MAX_BATCH_SIZE: usize = 1000;
        if messages.len() > MAX_BATCH_SIZE {
            return Err(CryptoError::InvalidInput(format!(
                "Batch size too large: {} (max: {})",
                messages.len(),
                MAX_BATCH_SIZE
            )));
        }

        for message in messages {
            // Validate message size
            const MAX_MESSAGE_SIZE: usize = 100 * 1024 * 1024;
            if message.len() > MAX_MESSAGE_SIZE {
                return Err(CryptoError::InvalidInput(
                    "Message too large in batch".to_string(),
                ));
            }
        }

        // The signing key is derived once for the whole batch.
        raw::sign_all(self.variant, &self.private_key, messages)
    }
}

/// ML-DSA signature batch verifier for performance
pub struct MLDsaBatchVerifier {
    public_key: Vec<u8>,
    variant: MLDsaVariant,
}

impl MLDsaBatchVerifier {
    /// Create a new batch verifier
    pub fn new(public_key: Vec<u8>, variant: MLDsaVariant) -> Self {
        Self {
            public_key,
            variant,
        }
    }

    /// Verify multiple signatures efficiently
    pub fn batch_verify(
        &self,
        messages: &[&[u8]],
        signatures: &[&[u8]],
    ) -> CryptoResult<Vec<bool>> {
        if messages.len() != signatures.len() {
            return Err(CryptoError::InvalidInput(
                "Messages and signatures count mismatch".to_string(),
            ));
        }

        let pairs: Vec<(&[u8], &[u8])> = messages
            .iter()
            .copied()
            .zip(signatures.iter().copied())
            .collect();
        raw::verify_all(self.variant, &self.public_key, &pairs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pqc::PostQuantumSignatures;

    #[test]
    fn test_mldsa_variant_properties() {
        // Test ML-DSA-44
        assert_eq!(MLDsaVariant::MLDsa44.public_key_size(), 1312);
        assert_eq!(MLDsaVariant::MLDsa44.private_key_size(), 32);
        assert_eq!(MLDsaVariant::MLDsa44.signature_size(), 2420);
        assert_eq!(MLDsaVariant::MLDsa44.security_level(), "128-bit");

        // Test ML-DSA-65
        assert_eq!(MLDsaVariant::MLDsa65.public_key_size(), 1952);
        assert_eq!(MLDsaVariant::MLDsa65.private_key_size(), 32);
        assert_eq!(MLDsaVariant::MLDsa65.signature_size(), 3309);
        assert_eq!(MLDsaVariant::MLDsa65.security_level(), "192-bit");

        // Test ML-DSA-87
        assert_eq!(MLDsaVariant::MLDsa87.public_key_size(), 2592);
        assert_eq!(MLDsaVariant::MLDsa87.private_key_size(), 32);
        assert_eq!(MLDsaVariant::MLDsa87.signature_size(), 4627);
        assert_eq!(MLDsaVariant::MLDsa87.security_level(), "256-bit");

        // Test max_signature_size
        assert!(
            MLDsaVariant::MLDsa44.max_signature_size() > MLDsaVariant::MLDsa44.signature_size()
        );
    }

    #[test]
    fn test_mldsa_keypair_generation() {
        let keypair = MLDsaKeypair::generate(MLDsaVariant::MLDsa44).unwrap();

        assert_eq!(keypair.public_key.len(), 1312);
        assert_eq!(keypair.private_key.len(), 32);
        assert_eq!(keypair.variant, MLDsaVariant::MLDsa44);
    }

    #[test]
    fn test_mldsa_sign_verify() {
        let keypair = MLDsaKeypair::generate(MLDsaVariant::MLDsa44).unwrap();
        let message = b"Hello, Post-Quantum World!";

        let signature = keypair.sign(message).unwrap();

        // Detached signature should be approximately the expected size
        // Allow some variance due to encoding
        let expected_size = MLDsaVariant::MLDsa44.signature_size();
        assert!(
            signature.len() >= expected_size - 50 && signature.len() <= expected_size + 50,
            "Signature size {} not in range [{}, {}]",
            signature.len(),
            expected_size - 50,
            expected_size + 50
        );

        // Verify correct signature
        let is_valid = keypair.verify(message, &signature).unwrap();
        assert!(is_valid, "Valid signature should verify successfully");

        // Test with wrong message
        let wrong_message = b"Hello, Wrong World!";
        let is_invalid = keypair.verify(wrong_message, &signature).unwrap();
        assert!(
            !is_invalid,
            "Signature should not verify with different message"
        );
    }

    #[test]
    fn test_mldsa_provider_trait() {
        let provider = MLDsaProvider::new(MLDsaVariant::MLDsa65);

        assert_eq!(provider.algorithm_id(), "ML-DSA-65");
        assert_eq!(provider.public_key_size(), 1952);
        assert_eq!(provider.private_key_size(), 32);
        assert_eq!(provider.signature_size(), 3309);
    }

    #[test]
    fn test_batch_operations() {
        let keypair = MLDsaKeypair::generate(MLDsaVariant::MLDsa44).unwrap();
        let messages_raw = [b"Message 1", b"Message 2", b"Message 3"];
        let messages: Vec<&[u8]> = messages_raw.iter().map(|m| m.as_ref()).collect();

        let batch_signer =
            MLDsaBatchSigner::new(keypair.private_key.clone(), MLDsaVariant::MLDsa44).unwrap();
        let signatures = batch_signer.batch_sign(&messages).unwrap();

        assert_eq!(signatures.len(), 3, "Should produce 3 signatures");

        // All signatures should be approximately the expected size
        let expected_size = MLDsaVariant::MLDsa44.signature_size();
        for sig in &signatures {
            assert!(
                sig.len() >= expected_size - 50 && sig.len() <= expected_size + 50,
                "Signature size {} not in valid range",
                sig.len()
            );
        }

        let batch_verifier = MLDsaBatchVerifier::new(keypair.public_key, MLDsaVariant::MLDsa44);
        let sig_slices: Vec<&[u8]> = signatures.iter().map(|s| s.as_slice()).collect();
        let results = batch_verifier.batch_verify(&messages, &sig_slices).unwrap();

        assert_eq!(results.len(), 3);
        assert!(
            results.iter().all(|&valid| valid),
            "All signatures should be valid"
        );
    }

    #[test]
    fn test_different_variants() {
        for variant in [
            MLDsaVariant::MLDsa44,
            MLDsaVariant::MLDsa65,
            MLDsaVariant::MLDsa87,
        ] {
            let keypair = MLDsaKeypair::generate(variant).unwrap();
            let message = b"Test message";

            let signature = keypair.sign(message).unwrap();

            // Check signature size is in acceptable range
            let expected = variant.signature_size();
            assert!(
                signature.len() >= expected - 50 && signature.len() <= expected + 50,
                "Variant {:?}: signature size {} not in range [{}, {}]",
                variant,
                signature.len(),
                expected - 50,
                expected + 50
            );

            let is_valid = keypair.verify(message, &signature).unwrap();
            assert!(
                is_valid,
                "Signature for variant {:?} should be valid",
                variant
            );
        }
    }

    #[test]
    fn test_input_validation() {
        let keypair = MLDsaKeypair::generate(MLDsaVariant::MLDsa44).unwrap();

        // Test invalid key size
        let invalid_sk = vec![0u8; 100]; // Wrong size
        let batch_signer = MLDsaBatchSigner::new(invalid_sk, MLDsaVariant::MLDsa44);
        assert!(batch_signer.is_err(), "Should reject invalid key size");

        // Test message too large
        let huge_message = vec![0u8; 101 * 1024 * 1024]; // 101 MB
        let result = keypair.sign(&huge_message);
        assert!(result.is_err(), "Should reject oversized message");

        // Test batch size limit
        let batch_signer =
            MLDsaBatchSigner::new(keypair.private_key.clone(), MLDsaVariant::MLDsa44).unwrap();
        let too_many: Vec<&[u8]> = vec![b"msg"; 1001];
        let result = batch_signer.batch_sign(&too_many);
        assert!(result.is_err(), "Should reject oversized batch");
    }

    #[test]
    fn test_cross_variant_rejection() {
        // Generate keypairs for different variants
        let keypair_44 = MLDsaKeypair::generate(MLDsaVariant::MLDsa44).unwrap();
        let keypair_65 = MLDsaKeypair::generate(MLDsaVariant::MLDsa65).unwrap();

        let message = b"Test message";
        let signature_44 = keypair_44.sign(message).unwrap();

        // Try to verify ML-DSA-44 signature with ML-DSA-65 key (should fail)
        let result = keypair_65.verify(message, &signature_44);
        // This should either error or return false
        assert!(
            result.is_err() || !result.unwrap(),
            "Cross-variant verification should fail"
        );
    }

    #[test]
    fn test_private_key_is_a_seed_that_rebuilds_the_keypair() {
        for variant in [
            MLDsaVariant::MLDsa44,
            MLDsaVariant::MLDsa65,
            MLDsaVariant::MLDsa87,
        ] {
            let keypair = MLDsaKeypair::generate(variant).unwrap();
            assert_eq!(keypair.private_key.len(), 32);

            let rebuilt = MLDsaKeypair::from_seed(variant, &keypair.private_key).unwrap();
            assert_eq!(rebuilt.public_key, keypair.public_key);

            let signature = rebuilt.sign(b"signed from the seed alone").unwrap();
            assert!(
                keypair
                    .verify(b"signed from the seed alone", &signature)
                    .unwrap()
            );
        }
    }

    /// The expanded keys of the previous implementation were 2,560 / 4,032 / 4,896
    /// bytes. Handing one over must be a clean error, never the abort that
    /// `ml-dsa`'s unchecked expanded-key loader would cause under `panic = "abort"`.
    #[test]
    fn test_keys_of_the_wrong_length_are_refused_without_panicking() {
        for len in [0usize, 31, 33, 2560, 4032, 4896] {
            let keypair = MLDsaKeypair {
                public_key: Vec::new(),
                private_key: vec![0xFF; len],
                variant: MLDsaVariant::MLDsa44,
            };
            assert!(
                keypair.sign(b"message").is_err(),
                "a {len}-byte private key must be refused"
            );
            assert!(
                MLDsaKeypair::from_seed(MLDsaVariant::MLDsa44, &vec![0xFF; len]).is_err(),
                "a {len}-byte seed must be refused"
            );
        }
    }

    /// Pins the byte format: a given seed must keep producing the same FIPS 204 public
    /// key. If a dependency upgrade ever changes this, keys already handed out stop
    /// matching, and this test is where that shows up.
    #[test]
    fn test_seed_to_public_key_is_stable() {
        let seed: Vec<u8> = (0u8..32).collect();
        let keypair = MLDsaKeypair::from_seed(MLDsaVariant::MLDsa44, &seed).unwrap();
        assert_eq!(keypair.public_key.len(), 1312);
        assert_eq!(
            hex::encode(&keypair.public_key[..32]),
            "d7b2b47254aae0db45e7930d4a98d2c97d8f1397d1789dafa17024b316e9bec9"
        );
    }

    #[test]
    fn test_signatures_are_hedged() {
        let keypair = MLDsaKeypair::generate(MLDsaVariant::MLDsa44).unwrap();
        let first = keypair.sign(b"same message").unwrap();
        let second = keypair.sign(b"same message").unwrap();
        assert_ne!(first, second, "each signature draws fresh randomness");
        assert!(keypair.verify(b"same message", &first).unwrap());
        assert!(keypair.verify(b"same message", &second).unwrap());
    }

    #[test]
    fn test_tampered_signature_and_foreign_key_do_not_verify() {
        let keypair = MLDsaKeypair::generate(MLDsaVariant::MLDsa44).unwrap();
        let other = MLDsaKeypair::generate(MLDsaVariant::MLDsa44).unwrap();
        let mut signature = keypair.sign(b"message").unwrap();

        assert!(!other.verify(b"message", &signature).unwrap());

        // The first bytes are the challenge hash, so this stays well formed.
        signature[0] ^= 0x01;
        assert!(!keypair.verify(b"message", &signature).unwrap());
    }

    #[test]
    fn test_batch_verify_reports_each_signature() {
        let keypair = MLDsaKeypair::generate(MLDsaVariant::MLDsa44).unwrap();
        let good = keypair.sign(b"one").unwrap();
        let other = keypair.sign(b"two").unwrap();

        let verifier = MLDsaBatchVerifier::new(keypair.public_key.clone(), MLDsaVariant::MLDsa44);
        let results = verifier
            .batch_verify(&[b"one", b"two"], &[good.as_slice(), other.as_slice()])
            .unwrap();
        assert_eq!(results, vec![true, true]);

        // A signature for another message is false, not an error.
        let results = verifier
            .batch_verify(&[b"one", b"two"], &[other.as_slice(), good.as_slice()])
            .unwrap();
        assert_eq!(results, vec![false, false]);
    }
}
