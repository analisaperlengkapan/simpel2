//! ML-KEM (Module-Lattice Key Encapsulation Mechanism) Implementation
//!
//! NIST PQC Standard for Post-Quantum Key Exchange (FIPS 203), on the RustCrypto
//! `ml-kem` crate.
//!
//! # Private keys are seeds
//!
//! `MLKemKeypair::private_key` holds the 64-byte FIPS 203 seed (`d || z`), the
//! same length for every variant, not the 1,632 / 2,400 / 3,168-byte expanded
//! decapsulation key. The decapsulation key is re-derived from the seed when it is
//! needed. The seed is what RustCrypto recommends storing (it deprecates the
//! expanded form), it is always valid, and it is 25-50x smaller. The public key,
//! ciphertext and shared secret are the FIPS 203 byte strings and interoperate
//! with any other FIPS 203 implementation.
//!
//! Because the seed has the same length in every variant, a private key no longer
//! tells you which variant it belongs to by its length alone: keep the `variant`
//! next to it, as `MLKemKeypair` does. A ciphertext of the wrong variant is still
//! refused by its length.

use crate::error::{CryptoError, CryptoResult};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Length of the FIPS 203 seed `d || z`, identical for all three parameter sets.
const SEED_LEN: usize = 64;

/// One entry point per operation over the three RustCrypto parameter sets.
mod raw {
    use super::{CryptoError, CryptoResult, MLKemVariant, SEED_LEN};
    use ml_kem::{
        Key, Seed,
        kem::{Decapsulate, Encapsulate, Generate, KeyExport},
    };
    use zeroize::Zeroizing;

    /// Run `$body` with `$m` bound to the `ml_kem` module of the variant's parameter set.
    macro_rules! per_variant {
        ($variant:expr, $m:ident => $body:block) => {
            match $variant {
                MLKemVariant::MLKem512 => {
                    use ml_kem::ml_kem_512 as $m;
                    $body
                }
                MLKemVariant::MLKem768 => {
                    use ml_kem::ml_kem_768 as $m;
                    $body
                }
                MLKemVariant::MLKem1024 => {
                    use ml_kem::ml_kem_1024 as $m;
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

    /// The public key the seed belongs to.
    pub(super) fn public_key_of(variant: MLKemVariant, seed: &[u8]) -> CryptoResult<Vec<u8>> {
        per_variant!(variant, m => {
            let dk = decapsulation_key::<m::DecapsulationKey>(seed)?;
            Ok(dk.encapsulation_key().to_bytes().to_vec())
        })
    }

    /// Seed bytes -> decapsulation key. Always succeeds for a 64-byte seed.
    fn decapsulation_key<D: From<Seed>>(seed: &[u8]) -> CryptoResult<D> {
        let seed = Seed::try_from(seed).map_err(|_| {
            CryptoError::InvalidKey(format!(
                "Invalid private key size: expected {SEED_LEN} bytes, got {} bytes",
                seed.len()
            ))
        })?;
        Ok(D::from(seed))
    }

    /// `(shared secret, ciphertext)` for the given public key.
    pub(super) fn encapsulate(
        variant: MLKemVariant,
        public_key: &[u8],
    ) -> CryptoResult<(Vec<u8>, Vec<u8>)> {
        per_variant!(variant, m => {
            let invalid = || CryptoError::InvalidKey("Key validation failed".to_string());
            let key = Key::<m::EncapsulationKey>::try_from(public_key).map_err(|_| invalid())?;
            // `new` runs the FIPS 203 encapsulation-key checks (length, and that every
            // coefficient is reduced modulo q).
            let ek = m::EncapsulationKey::new(&key).map_err(|_| invalid())?;
            let (ciphertext, shared_secret) = ek.encapsulate();
            Ok((shared_secret.to_vec(), ciphertext.to_vec()))
        })
    }

    /// The decapsulation of every ciphertext, with the key derived once.
    pub(super) fn decapsulate_all(
        variant: MLKemVariant,
        private_key: &[u8],
        ciphertexts: &[&[u8]],
    ) -> CryptoResult<Vec<Vec<u8>>> {
        per_variant!(variant, m => {
            let dk = decapsulation_key::<m::DecapsulationKey>(private_key)?;
            ciphertexts
                .iter()
                .map(|ciphertext| {
                    let ct = m::Ciphertext::try_from(*ciphertext).map_err(|_| {
                        CryptoError::InvalidCiphertext("Ciphertext format invalid".to_string())
                    })?;
                    Ok(dk.decapsulate(&ct).to_vec())
                })
                .collect()
        })
    }
}

/// ML-KEM variant configurations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MLKemVariant {
    /// ML-KEM-512: 128-bit security, 800-byte public key, 768-byte ciphertext
    MLKem512,
    /// ML-KEM-768: 192-bit security, 1,184-byte public key, 1,088-byte ciphertext
    MLKem768,
    /// ML-KEM-1024: 256-bit security, 1,568-byte public key, 1,568-byte ciphertext
    MLKem1024,
}

impl MLKemVariant {
    /// Get the public key size in bytes for this variant
    pub fn public_key_size(&self) -> usize {
        match self {
            MLKemVariant::MLKem512 => 800,
            MLKemVariant::MLKem768 => 1184,
            MLKemVariant::MLKem1024 => 1568,
        }
    }

    /// Get the private key size in bytes for this variant
    ///
    /// The private key is the 64-byte FIPS 203 seed, the same for every variant
    /// (see the module documentation).
    pub fn private_key_size(&self) -> usize {
        SEED_LEN
    }

    /// Get the ciphertext size in bytes for this variant
    pub fn ciphertext_size(&self) -> usize {
        match self {
            MLKemVariant::MLKem512 => 768,
            MLKemVariant::MLKem768 => 1088,
            MLKemVariant::MLKem1024 => 1568,
        }
    }

    /// Get the shared secret size in bytes for this variant
    pub fn shared_secret_size(&self) -> usize {
        match self {
            MLKemVariant::MLKem512 => 32,
            MLKemVariant::MLKem768 => 32,
            MLKemVariant::MLKem1024 => 32,
        }
    }

    /// Get the security level name
    pub fn security_level(&self) -> &'static str {
        match self {
            MLKemVariant::MLKem512 => "128-bit",
            MLKemVariant::MLKem768 => "192-bit",
            MLKemVariant::MLKem1024 => "256-bit",
        }
    }
}

impl fmt::Display for MLKemVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MLKemVariant::MLKem512 => write!(f, "ML-KEM-512"),
            MLKemVariant::MLKem768 => write!(f, "ML-KEM-768"),
            MLKemVariant::MLKem1024 => write!(f, "ML-KEM-1024"),
        }
    }
}

/// ML-KEM keypair for key exchange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLKemKeypair {
    /// Public key for encapsulation
    pub public_key: Vec<u8>,
    /// Private key for decapsulation: the 64-byte FIPS 203 seed (see the module docs)
    pub private_key: Vec<u8>,
    /// Algorithm variant
    pub variant: MLKemVariant,
}

impl MLKemKeypair {
    /// Generate a new ML-KEM keypair
    pub fn generate(variant: MLKemVariant) -> CryptoResult<Self> {
        let seed = raw::random_seed()?;
        Self::from_seed(variant, &seed[..])
    }

    /// Rebuild a keypair from its 64-byte seed; the public key is derived from it.
    ///
    /// The seed is the `private_key` of a keypair, so this is how a keypair is
    /// recovered from the private key alone.
    pub fn from_seed(variant: MLKemVariant, seed: &[u8]) -> CryptoResult<Self> {
        let public_key = raw::public_key_of(variant, seed)?;
        Ok(Self {
            public_key,
            private_key: seed.to_vec(),
            variant,
        })
    }

    /// Encapsulate a shared secret using the public key
    ///
    /// # Returns
    /// Tuple of (shared_secret, ciphertext):
    /// - shared_secret: 32-byte symmetric key (all variants)
    /// - ciphertext: Variant-specific size to send to recipient
    ///
    /// # Security
    /// - Uses cryptographically secure randomness internally
    /// - FIPS 203 compliant ML-KEM implementation
    /// - Constant-time operations resistant to timing attacks
    pub fn encapsulate(&self) -> CryptoResult<(Vec<u8>, Vec<u8>)> {
        // Validate public key size
        if self.public_key.len() != self.variant.public_key_size() {
            return Err(CryptoError::InvalidKey(format!(
                "Invalid public key size: expected {} bytes, got {} bytes",
                self.variant.public_key_size(),
                self.public_key.len()
            )));
        }

        raw::encapsulate(self.variant, &self.public_key)
    }

    /// Decapsulate a shared secret using the private key and ciphertext
    ///
    /// # Arguments
    /// * `ciphertext` - The ciphertext from encapsulation
    ///
    /// # Returns
    /// The 32-byte shared secret (same as produced by encapsulate)
    ///
    /// # Security
    /// - FIPS 203 compliant ML-KEM implementation
    /// - Constant-time decapsulation
    /// - Validates ciphertext size before processing
    pub fn decapsulate(&self, ciphertext: &[u8]) -> CryptoResult<Vec<u8>> {
        // Validate ciphertext size
        if ciphertext.len() != self.variant.ciphertext_size() {
            return Err(CryptoError::InvalidCiphertext(format!(
                "Invalid ciphertext size: expected {} bytes, got {} bytes",
                self.variant.ciphertext_size(),
                ciphertext.len()
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

        let mut shared_secrets =
            raw::decapsulate_all(self.variant, &self.private_key, &[ciphertext])?;
        shared_secrets
            .pop()
            .ok_or_else(|| CryptoError::Internal("Decapsulation produced no output".to_string()))
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
    pub fn variant(&self) -> MLKemVariant {
        self.variant
    }
}

/// ML-KEM provider implementing the PostQuantumKeyExchange trait
pub struct MLKemProvider {
    variant: MLKemVariant,
}

impl MLKemProvider {
    /// Create a new ML-KEM provider with the specified variant
    pub fn new(variant: MLKemVariant) -> Self {
        Self { variant }
    }

    /// Generate a new keypair
    pub fn generate_keypair(&self) -> CryptoResult<MLKemKeypair> {
        MLKemKeypair::generate(self.variant)
    }

    /// Create provider from existing keypair
    pub fn from_keypair(keypair: MLKemKeypair) -> Self {
        Self {
            variant: keypair.variant,
        }
    }
}

impl crate::pqc::PostQuantumKeyExchange for MLKemProvider {
    fn algorithm_id(&self) -> &'static str {
        match self.variant {
            MLKemVariant::MLKem512 => "ML-KEM-512",
            MLKemVariant::MLKem768 => "ML-KEM-768",
            MLKemVariant::MLKem1024 => "ML-KEM-1024",
        }
    }

    fn keypair_generate(&self) -> crate::pqc::PQCResult<(Vec<u8>, Vec<u8>)> {
        let keypair = MLKemKeypair::generate(self.variant)?;
        Ok((keypair.public_key, keypair.private_key))
    }

    fn public_key_size(&self) -> usize {
        self.variant.public_key_size()
    }

    fn private_key_size(&self) -> usize {
        self.variant.private_key_size()
    }

    fn encapsulate(&self, public_key: &[u8]) -> crate::pqc::PQCResult<(Vec<u8>, Vec<u8>)> {
        let keypair = MLKemKeypair {
            public_key: public_key.to_vec(),
            private_key: Vec::new(), // Not needed for encapsulation
            variant: self.variant,
        };
        keypair
            .encapsulate()
            .map_err(|e| crate::pqc::PQCError::KeyExchangeError(e.to_string()))
    }

    fn decapsulate(&self, ciphertext: &[u8], private_key: &[u8]) -> crate::pqc::PQCResult<Vec<u8>> {
        let keypair = MLKemKeypair {
            public_key: Vec::new(), // Not needed for decapsulation
            private_key: private_key.to_vec(),
            variant: self.variant,
        };
        keypair
            .decapsulate(ciphertext)
            .map_err(|e| crate::pqc::PQCError::KeyExchangeError(e.to_string()))
    }

    fn ciphertext_size(&self) -> usize {
        self.variant.ciphertext_size()
    }

    fn shared_secret_size(&self) -> usize {
        self.variant.shared_secret_size()
    }
}

/// ML-KEM batch operations for performance
pub struct MLKemBatchEncapsulator {
    public_key: Vec<u8>,
    variant: MLKemVariant,
}

impl MLKemBatchEncapsulator {
    /// Create a new batch encapsulator
    pub fn new(public_key: Vec<u8>, variant: MLKemVariant) -> Self {
        Self {
            public_key,
            variant,
        }
    }

    /// Encapsulate multiple shared secrets efficiently
    pub fn batch_encapsulate(&self, count: usize) -> CryptoResult<Vec<(Vec<u8>, Vec<u8>)>> {
        let mut results = Vec::with_capacity(count);

        for _ in 0..count {
            results.push(raw::encapsulate(self.variant, &self.public_key)?);
        }

        Ok(results)
    }
}

/// ML-KEM batch decapsulator for performance
pub struct MLKemBatchDecapsulator {
    private_key: Vec<u8>,
    variant: MLKemVariant,
}

impl MLKemBatchDecapsulator {
    /// Create a new batch decapsulator
    pub fn new(private_key: Vec<u8>, variant: MLKemVariant) -> Self {
        Self {
            private_key,
            variant,
        }
    }

    /// Decapsulate multiple ciphertexts efficiently
    pub fn batch_decapsulate(&self, ciphertexts: &[&[u8]]) -> CryptoResult<Vec<Vec<u8>>> {
        raw::decapsulate_all(self.variant, &self.private_key, ciphertexts)
    }
}

/// Test key exchange correctness
pub fn test_key_exchange_correctness(variant: MLKemVariant) -> CryptoResult<bool> {
    // Generate keypair
    let keypair = MLKemKeypair::generate(variant)?;

    // Alice encapsulates
    let (alice_secret, ciphertext) = keypair.encapsulate()?;

    // Bob decapsulates
    let bob_secret = keypair.decapsulate(&ciphertext)?;

    // Secrets should match - use constant-time comparison
    use crate::pqc::constant_time::ct_eq;
    Ok(ct_eq(&alice_secret, &bob_secret))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pqc::PostQuantumKeyExchange;

    #[test]
    fn test_mlkem_variant_properties() {
        assert_eq!(MLKemVariant::MLKem512.public_key_size(), 800);
        assert_eq!(MLKemVariant::MLKem512.private_key_size(), 64);
        assert_eq!(MLKemVariant::MLKem512.ciphertext_size(), 768);
        assert_eq!(MLKemVariant::MLKem512.shared_secret_size(), 32);

        assert_eq!(MLKemVariant::MLKem768.public_key_size(), 1184);
        assert_eq!(MLKemVariant::MLKem768.private_key_size(), 64);
        assert_eq!(MLKemVariant::MLKem768.ciphertext_size(), 1088);
        assert_eq!(MLKemVariant::MLKem768.shared_secret_size(), 32);

        assert_eq!(MLKemVariant::MLKem1024.public_key_size(), 1568);
        assert_eq!(MLKemVariant::MLKem1024.private_key_size(), 64);
        assert_eq!(MLKemVariant::MLKem1024.ciphertext_size(), 1568);
        assert_eq!(MLKemVariant::MLKem1024.shared_secret_size(), 32);
    }

    #[test]
    fn test_mlkem_keypair_generation() {
        let keypair = MLKemKeypair::generate(MLKemVariant::MLKem512).unwrap();

        assert_eq!(keypair.public_key.len(), 800);
        assert_eq!(keypair.private_key.len(), 64);
        assert_eq!(keypair.variant, MLKemVariant::MLKem512);
    }

    #[test]
    fn test_mlkem_key_exchange() {
        let keypair = MLKemKeypair::generate(MLKemVariant::MLKem768).unwrap();

        // Alice encapsulates
        let (alice_secret, ciphertext) = keypair.encapsulate().unwrap();
        assert_eq!(alice_secret.len(), 32);
        assert_eq!(ciphertext.len(), 1088);

        // Bob decapsulates
        let bob_secret = keypair.decapsulate(&ciphertext).unwrap();
        assert_eq!(bob_secret.len(), 32);

        // Secrets should match
        assert_eq!(alice_secret, bob_secret);
    }

    #[test]
    fn test_mlkem_provider_trait() {
        let provider = MLKemProvider::new(MLKemVariant::MLKem1024);

        assert_eq!(provider.algorithm_id(), "ML-KEM-1024");
        assert_eq!(provider.public_key_size(), 1568);
        assert_eq!(provider.private_key_size(), 64);
        assert_eq!(provider.ciphertext_size(), 1568);
        assert_eq!(provider.shared_secret_size(), 32);
    }

    #[test]
    fn test_batch_operations() {
        let keypair = MLKemKeypair::generate(MLKemVariant::MLKem512).unwrap();

        let batch_encapsulator =
            MLKemBatchEncapsulator::new(keypair.public_key.clone(), MLKemVariant::MLKem512);
        let encapsulations = batch_encapsulator.batch_encapsulate(3).unwrap();

        assert_eq!(encapsulations.len(), 3);

        let ciphertexts: Vec<&[u8]> = encapsulations
            .iter()
            .map(|(_, cipher)| cipher.as_slice())
            .collect();
        let batch_decapsulator =
            MLKemBatchDecapsulator::new(keypair.private_key, MLKemVariant::MLKem512);
        let decryptions = batch_decapsulator.batch_decapsulate(&ciphertexts).unwrap();

        assert_eq!(decryptions.len(), 3);

        // Verify all decryptions match their corresponding encapsulations
        for ((alice_secret, _), bob_secret) in encapsulations.iter().zip(decryptions.iter()) {
            assert_eq!(alice_secret, bob_secret);
        }
    }

    #[test]
    fn test_different_variants() {
        for variant in [
            MLKemVariant::MLKem512,
            MLKemVariant::MLKem768,
            MLKemVariant::MLKem1024,
        ] {
            let keypair = MLKemKeypair::generate(variant).unwrap();

            let (alice_secret, ciphertext) = keypair.encapsulate().unwrap();
            let bob_secret = keypair.decapsulate(&ciphertext).unwrap();

            assert_eq!(alice_secret.len(), 32);
            assert_eq!(bob_secret.len(), 32);
            assert_eq!(alice_secret, bob_secret);
        }
    }

    #[test]
    fn test_key_exchange_correctness_fn() {
        for variant in [
            MLKemVariant::MLKem512,
            MLKemVariant::MLKem768,
            MLKemVariant::MLKem1024,
        ] {
            let is_correct = super::test_key_exchange_correctness(variant).unwrap();
            assert!(is_correct, "Key exchange failed for {:?}", variant);
        }
    }

    #[test]
    fn test_input_validation() {
        let keypair = MLKemKeypair::generate(MLKemVariant::MLKem512).unwrap();

        // Test invalid ciphertext size
        let invalid_ct = vec![0u8; 100]; // Wrong size
        let result = keypair.decapsulate(&invalid_ct);
        assert!(result.is_err(), "Should reject invalid ciphertext size");

        // Test invalid public key size for encapsulation
        let mut bad_keypair = keypair.clone();
        bad_keypair.public_key = vec![0u8; 100]; // Wrong size
        let result = bad_keypair.encapsulate();
        assert!(result.is_err(), "Should reject invalid public key size");

        // Test invalid private key size for decapsulation
        let (_, ciphertext) = keypair.encapsulate().unwrap();
        let mut bad_keypair2 = keypair.clone();
        bad_keypair2.private_key = vec![0u8; 100]; // Wrong size
        let result = bad_keypair2.decapsulate(&ciphertext);
        assert!(result.is_err(), "Should reject invalid private key size");
    }

    #[test]
    fn test_cross_variant_rejection() {
        // Generate keypairs for different variants
        let keypair_512 = MLKemKeypair::generate(MLKemVariant::MLKem512).unwrap();
        let keypair_768 = MLKemKeypair::generate(MLKemVariant::MLKem768).unwrap();

        // Encapsulate with 512
        let (_, ciphertext_512) = keypair_512.encapsulate().unwrap();

        // Try to decapsulate with 768 key (should fail due to size mismatch)
        let result = keypair_768.decapsulate(&ciphertext_512);
        assert!(result.is_err(), "Cross-variant decapsulation should fail");
    }

    #[test]
    fn test_shared_secret_size() {
        for variant in [
            MLKemVariant::MLKem512,
            MLKemVariant::MLKem768,
            MLKemVariant::MLKem1024,
        ] {
            let keypair = MLKemKeypair::generate(variant).unwrap();
            let (shared_secret, _) = keypair.encapsulate().unwrap();

            assert_eq!(
                shared_secret.len(),
                32,
                "All variants should produce 32-byte shared secrets"
            );
        }
    }

    #[test]
    fn test_private_key_is_a_seed_that_rebuilds_the_keypair() {
        for variant in [
            MLKemVariant::MLKem512,
            MLKemVariant::MLKem768,
            MLKemVariant::MLKem1024,
        ] {
            let keypair = MLKemKeypair::generate(variant).unwrap();
            assert_eq!(keypair.private_key.len(), 64);

            let rebuilt = MLKemKeypair::from_seed(variant, &keypair.private_key).unwrap();
            assert_eq!(rebuilt.public_key, keypair.public_key);

            // A key exchange that starts from the seed alone still completes.
            let (secret, ciphertext) = keypair.encapsulate().unwrap();
            assert_eq!(rebuilt.decapsulate(&ciphertext).unwrap(), secret);
        }
    }

    #[test]
    fn test_seed_of_wrong_length_is_refused() {
        for len in [0usize, 32, 63, 65, 2400] {
            let seed = vec![7u8; len];
            assert!(
                MLKemKeypair::from_seed(MLKemVariant::MLKem768, &seed).is_err(),
                "a {len}-byte seed must be refused"
            );
        }
    }

    /// Pins the byte format: a given seed must keep producing the same FIPS 203 public
    /// key. If a dependency upgrade ever changes this, keys already handed out stop
    /// matching, and this test is where that shows up.
    #[test]
    fn test_seed_to_public_key_is_stable() {
        let seed: Vec<u8> = (0u8..64).collect();
        let keypair = MLKemKeypair::from_seed(MLKemVariant::MLKem768, &seed).unwrap();
        assert_eq!(keypair.public_key.len(), 1184);
        assert_eq!(
            hex::encode(&keypair.public_key[..32]),
            "298aa10d423c8dda069d02bc59e6cdf03a096b8b3da4cab9b80ca4a14907672c"
        );
    }

    #[test]
    fn test_tampered_ciphertext_yields_a_different_secret() {
        // ML-KEM rejects implicitly: a corrupted ciphertext still decapsulates, to an
        // unrelated secret, so the caller cannot learn anything from an error.
        let keypair = MLKemKeypair::generate(MLKemVariant::MLKem768).unwrap();
        let (secret, mut ciphertext) = keypair.encapsulate().unwrap();
        ciphertext[10] ^= 0x01;
        let other = keypair.decapsulate(&ciphertext).unwrap();
        assert_eq!(other.len(), 32);
        assert_ne!(other, secret);
    }

    #[test]
    fn test_public_key_with_unreduced_coefficients_is_refused() {
        // 0xFF.. decodes to 12-bit coefficients above q = 3329, which FIPS 203 requires
        // an encapsulator to reject.
        let mut keypair = MLKemKeypair::generate(MLKemVariant::MLKem768).unwrap();
        keypair.public_key = vec![0xFF; MLKemVariant::MLKem768.public_key_size()];
        assert!(keypair.encapsulate().is_err());
    }

    #[test]
    fn test_encapsulations_are_fresh() {
        let keypair = MLKemKeypair::generate(MLKemVariant::MLKem768).unwrap();
        let (secret_a, ciphertext_a) = keypair.encapsulate().unwrap();
        let (secret_b, ciphertext_b) = keypair.encapsulate().unwrap();
        assert_ne!(ciphertext_a, ciphertext_b);
        assert_ne!(secret_a, secret_b);
    }
}
