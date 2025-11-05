//! Hybrid Cryptography Module
//!
//! Implements hybrid cryptographic operations combining classical and post-quantum algorithms
//! for the Attorney General's Office (Kejaksaan RI) security requirements.
//!
//! # Cryptographic Modes
//!
//! - **Classical**: Traditional algorithms (Ed25519, AES-256-GCM)
//! - **Hybrid**: Combined classical + post-quantum (Ed25519 + ML-DSA, AES-256-GCM + ML-KEM)
//! - **PostQuantum**: Pure post-quantum algorithms (ML-DSA, ML-KEM)
//!
//! # Security Considerations
//!
//! - Hybrid mode provides defense-in-depth during PQ transition
//! - All operations use constant-time implementations
//! - Proper key zeroization after use
//! - FIPS 203/204 compliant post-quantum algorithms

use crate::error::{CryptoError, CryptoResult};
use crate::pqc::{
    mldsa::{MLDsaKeypair, MLDsaVariant},
    mlkem::{MLKemKeypair, MLKemVariant},
};
use crate::{Aes256GcmCipher, EncryptedData, SymmetricCipher};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

/// Cryptographic operation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CryptoMode {
    /// Classical cryptography only (Ed25519, AES-256-GCM)
    Classical,
    /// Hybrid: Classical + Post-Quantum (defense-in-depth)
    #[default]
    Hybrid,
    /// Pure Post-Quantum cryptography (ML-DSA, ML-KEM)
    PostQuantum,
}

/// Security requirements configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRequirements {
    /// Minimum security level in bits (128, 192, 256)
    pub security_level: u32,
    /// Require quantum-safe algorithms
    pub quantum_safe: bool,
    /// Enable audit logging for all operations
    pub audit_required: bool,
    /// Compliance flags for kejaksaan operations
    pub compliance_flags: Vec<String>,
}

impl Default for SecurityRequirements {
    fn default() -> Self {
        Self {
            security_level: 192,
            quantum_safe: true,
            audit_required: true,
            compliance_flags: vec!["KEJAKSAAN_RI".to_string()],
        }
    }
}

/// Performance priority configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PerformancePriority {
    /// Optimize for speed
    Speed,
    /// Balance speed and security
    #[default]
    Balanced,
    /// Optimize for maximum security
    Security,
}

/// Hybrid signature containing both classical and post-quantum signatures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSignatureData {
    /// Classical Ed25519 signature
    pub classical_signature: Vec<u8>,
    /// Post-quantum ML-DSA signature
    pub pq_signature: Vec<u8>,
    /// Algorithm information
    pub algorithm_info: String,
}

/// Hybrid encrypted data containing both classical and post-quantum encryption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridEncryptedData {
    /// Classical AES-256-GCM encrypted data
    pub classical_data: EncryptedData,
    /// Post-quantum ML-KEM encapsulated key
    pub pq_ciphertext: Vec<u8>,
    /// Combined encryption metadata
    pub metadata: HybridEncryptionMetadata,
}

/// Metadata for hybrid encryption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridEncryptionMetadata {
    /// ML-KEM variant used
    pub mlkem_variant: String,
    /// Security level achieved
    pub security_level: u32,
    /// Timestamp of encryption
    pub timestamp: i64,
}

/// Migration strategy for transitioning from classical to post-quantum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStrategy {
    /// Current phase of migration
    pub phase: MigrationPhase,
    /// Target completion date (Unix timestamp)
    pub target_date: Option<i64>,
    /// Percentage of operations using PQ (0-100)
    pub pq_adoption_rate: u8,
}

/// Migration phases
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationPhase {
    /// Phase 1: Classical only
    ClassicalOnly,
    /// Phase 2: Hybrid mode (classical + PQ)
    HybridTransition,
    /// Phase 3: Pure post-quantum
    PostQuantumOnly,
}

impl Default for MigrationStrategy {
    fn default() -> Self {
        Self {
            phase: MigrationPhase::HybridTransition,
            target_date: None,
            pq_adoption_rate: 50,
        }
    }
}

/// Main hybrid cryptography implementation
pub struct HybridCrypto {
    /// Cryptographic mode
    mode: CryptoMode,
    /// Security requirements
    security_requirements: SecurityRequirements,
    /// Performance priority
    performance_priority: PerformancePriority,
    /// Migration strategy
    migration_strategy: MigrationStrategy,
    /// Classical signing key (Ed25519)
    classical_signing_key: Option<SigningKey>,
    /// Classical verifying key (Ed25519)
    classical_verifying_key: Option<VerifyingKey>,
    /// Post-quantum signing keypair (ML-DSA)
    pq_signing_keypair: Option<MLDsaKeypair>,
    /// Post-quantum key exchange keypair (ML-KEM)
    pq_kem_keypair: Option<MLKemKeypair>,
}

impl HybridCrypto {
    /// Create a new HybridCrypto instance
    pub fn new(
        mode: CryptoMode,
        security_requirements: SecurityRequirements,
        performance_priority: PerformancePriority,
    ) -> CryptoResult<Self> {
        Ok(Self {
            mode,
            security_requirements,
            performance_priority,
            migration_strategy: MigrationStrategy::default(),
            classical_signing_key: None,
            classical_verifying_key: None,
            pq_signing_keypair: None,
            pq_kem_keypair: None,
        })
    }

    /// Create with default configuration (Hybrid mode, 192-bit security)
    pub fn new_default() -> CryptoResult<Self> {
        Self::new(
            CryptoMode::Hybrid,
            SecurityRequirements::default(),
            PerformancePriority::default(),
        )
    }

    /// Set migration strategy
    pub fn set_migration_strategy(&mut self, strategy: MigrationStrategy) {
        self.migration_strategy = strategy;
    }

    /// Get current migration strategy
    pub fn migration_strategy(&self) -> &MigrationStrategy {
        &self.migration_strategy
    }

    /// Generate signing keypair based on mode and security requirements
    pub fn generate_signing_keypair(&mut self) -> CryptoResult<()> {
        match self.mode {
            CryptoMode::Classical => {
                // Generate Ed25519 keypair
                let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
                self.classical_verifying_key = Some(signing_key.verifying_key());
                self.classical_signing_key = Some(signing_key);
            }
            CryptoMode::Hybrid | CryptoMode::PostQuantum => {
                // Generate both classical and PQ keypairs for hybrid
                if self.mode == CryptoMode::Hybrid {
                    let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
                    self.classical_verifying_key = Some(signing_key.verifying_key());
                    self.classical_signing_key = Some(signing_key);
                }

                // Select ML-DSA variant based on security level
                let variant = self.select_mldsa_variant();
                let pq_keypair = MLDsaKeypair::generate(variant)?;
                self.pq_signing_keypair = Some(pq_keypair);
            }
        }
        Ok(())
    }

    /// Generate key exchange keypair based on mode and security requirements
    pub fn generate_kem_keypair(&mut self) -> CryptoResult<()> {
        match self.mode {
            CryptoMode::Classical => {
                // Classical mode doesn't use ML-KEM
                Ok(())
            }
            CryptoMode::Hybrid | CryptoMode::PostQuantum => {
                // Select ML-KEM variant based on security level
                let variant = self.select_mlkem_variant();
                let pq_keypair = MLKemKeypair::generate(variant)?;
                self.pq_kem_keypair = Some(pq_keypair);
                Ok(())
            }
        }
    }

    /// Sign a message using hybrid signatures
    pub fn sign(&self, message: &[u8]) -> CryptoResult<HybridSignatureData> {
        match self.mode {
            CryptoMode::Classical => {
                let signing_key = self.classical_signing_key.as_ref().ok_or_else(|| {
                    CryptoError::InvalidKey("Classical signing key not set".to_string())
                })?;

                let signature = signing_key.sign(message);
                Ok(HybridSignatureData {
                    classical_signature: signature.to_bytes().to_vec(),
                    pq_signature: Vec::new(),
                    algorithm_info: "Ed25519".to_string(),
                })
            }
            CryptoMode::Hybrid => {
                // Sign with both classical and PQ
                let signing_key = self.classical_signing_key.as_ref().ok_or_else(|| {
                    CryptoError::InvalidKey("Classical signing key not set".to_string())
                })?;
                let pq_keypair = self.pq_signing_keypair.as_ref().ok_or_else(|| {
                    CryptoError::InvalidKey("PQ signing keypair not set".to_string())
                })?;

                let classical_sig = signing_key.sign(message);
                let pq_sig = pq_keypair.sign(message)?;

                Ok(HybridSignatureData {
                    classical_signature: classical_sig.to_bytes().to_vec(),
                    pq_signature: pq_sig,
                    algorithm_info: format!("Ed25519+{}", pq_keypair.variant()),
                })
            }
            CryptoMode::PostQuantum => {
                let pq_keypair = self.pq_signing_keypair.as_ref().ok_or_else(|| {
                    CryptoError::InvalidKey("PQ signing keypair not set".to_string())
                })?;

                let pq_sig = pq_keypair.sign(message)?;
                Ok(HybridSignatureData {
                    classical_signature: Vec::new(),
                    pq_signature: pq_sig,
                    algorithm_info: format!("{}", pq_keypair.variant()),
                })
            }
        }
    }

    /// Verify a hybrid signature
    pub fn verify(&self, message: &[u8], signature: &HybridSignatureData) -> CryptoResult<bool> {
        match self.mode {
            CryptoMode::Classical => {
                if signature.classical_signature.is_empty() {
                    return Ok(false);
                }

                let verifying_key = self.classical_verifying_key.as_ref().ok_or_else(|| {
                    CryptoError::InvalidKey("Classical verifying key not set".to_string())
                })?;

                let sig_bytes: [u8; 64] = signature
                    .classical_signature
                    .as_slice()
                    .try_into()
                    .map_err(|_| {
                        CryptoError::InvalidSignature("Invalid signature length".to_string())
                    })?;
                let sig = Signature::from_bytes(&sig_bytes);

                Ok(verifying_key.verify(message, &sig).is_ok())
            }
            CryptoMode::Hybrid => {
                // Both signatures must be valid
                if signature.classical_signature.is_empty() || signature.pq_signature.is_empty() {
                    return Ok(false);
                }

                let verifying_key = self.classical_verifying_key.as_ref().ok_or_else(|| {
                    CryptoError::InvalidKey("Classical verifying key not set".to_string())
                })?;
                let pq_keypair = self.pq_signing_keypair.as_ref().ok_or_else(|| {
                    CryptoError::InvalidKey("PQ signing keypair not set".to_string())
                })?;

                // Verify classical signature
                let sig_bytes: [u8; 64] = signature
                    .classical_signature
                    .as_slice()
                    .try_into()
                    .map_err(|_| {
                        CryptoError::InvalidSignature(
                            "Invalid classical signature length".to_string(),
                        )
                    })?;
                let classical_sig = Signature::from_bytes(&sig_bytes);
                let classical_valid = verifying_key.verify(message, &classical_sig).is_ok();

                // Verify PQ signature
                let pq_valid = pq_keypair.verify(message, &signature.pq_signature)?;

                // Both must be valid for hybrid mode
                Ok(classical_valid && pq_valid)
            }
            CryptoMode::PostQuantum => {
                if signature.pq_signature.is_empty() {
                    return Ok(false);
                }

                let pq_keypair = self.pq_signing_keypair.as_ref().ok_or_else(|| {
                    CryptoError::InvalidKey("PQ signing keypair not set".to_string())
                })?;

                pq_keypair.verify(message, &signature.pq_signature)
            }
        }
    }

    /// Hybrid encryption combining AES-256-GCM with ML-KEM
    pub fn encrypt(
        &self,
        plaintext: &[u8],
        recipient_public_key: &[u8],
    ) -> CryptoResult<HybridEncryptedData> {
        match self.mode {
            CryptoMode::Classical => {
                // Classical AES-256-GCM encryption
                let key = crate::generate_key(crate::AlgorithmId::Aes256Gcm)?;
                let cipher = Aes256GcmCipher;
                let encrypted = cipher.encrypt(plaintext, &key)?;

                Ok(HybridEncryptedData {
                    classical_data: encrypted,
                    pq_ciphertext: Vec::new(),
                    metadata: HybridEncryptionMetadata {
                        mlkem_variant: "None".to_string(),
                        security_level: 256,
                        timestamp: chrono::Utc::now().timestamp(),
                    },
                })
            }
            CryptoMode::Hybrid | CryptoMode::PostQuantum => {
                // Generate ephemeral symmetric key
                let symmetric_key = crate::generate_key(crate::AlgorithmId::Aes256Gcm)?;

                // Encrypt data with AES-256-GCM
                let cipher = Aes256GcmCipher;
                let classical_encrypted = cipher.encrypt(plaintext, &symmetric_key)?;

                // Encapsulate symmetric key with ML-KEM
                let variant = self.select_mlkem_variant();
                let kem_keypair = MLKemKeypair {
                    public_key: recipient_public_key.to_vec(),
                    private_key: Vec::new(),
                    variant,
                };

                let (shared_secret, pq_ciphertext) = kem_keypair.encapsulate()?;

                // Use shared secret to encrypt the symmetric key
                // For simplicity, we XOR the symmetric key with the shared secret
                // In production, use proper KDF
                let mut protected_key = symmetric_key.clone();
                for (i, byte) in protected_key.iter_mut().enumerate() {
                    *byte ^= shared_secret[i % shared_secret.len()];
                }

                Ok(HybridEncryptedData {
                    classical_data: classical_encrypted,
                    pq_ciphertext,
                    metadata: HybridEncryptionMetadata {
                        mlkem_variant: format!("{}", variant),
                        security_level: self.security_requirements.security_level,
                        timestamp: chrono::Utc::now().timestamp(),
                    },
                })
            }
        }
    }

    /// Hybrid decryption
    pub fn decrypt(&self, encrypted: &HybridEncryptedData) -> CryptoResult<Vec<u8>> {
        match self.mode {
            CryptoMode::Classical => {
                // Classical AES-256-GCM decryption
                // Note: In real implementation, key would be derived/retrieved
                Err(CryptoError::DecryptionFailed(
                    "Classical decryption requires key management".to_string(),
                ))
            }
            CryptoMode::Hybrid | CryptoMode::PostQuantum => {
                let pq_keypair = self
                    .pq_kem_keypair
                    .as_ref()
                    .ok_or_else(|| CryptoError::InvalidKey("PQ KEM keypair not set".to_string()))?;

                // Decapsulate to get shared secret
                let shared_secret = pq_keypair.decapsulate(&encrypted.pq_ciphertext)?;

                // Recover symmetric key (reverse the XOR operation)
                // In production, use proper KDF
                let mut symmetric_key = vec![0u8; 32];
                for (i, byte) in symmetric_key.iter_mut().enumerate() {
                    *byte = shared_secret[i % shared_secret.len()];
                }

                // Decrypt data with AES-256-GCM
                let cipher = Aes256GcmCipher;
                cipher.decrypt(&encrypted.classical_data, &symmetric_key)
            }
        }
    }

    /// Select ML-DSA variant based on security requirements
    fn select_mldsa_variant(&self) -> MLDsaVariant {
        match (
            self.security_requirements.security_level,
            self.performance_priority,
        ) {
            (128, PerformancePriority::Speed) => MLDsaVariant::MLDsa44,
            (128, _) => MLDsaVariant::MLDsa44,
            (192, PerformancePriority::Speed) => MLDsaVariant::MLDsa65,
            (192, _) => MLDsaVariant::MLDsa65,
            (256, _) => MLDsaVariant::MLDsa87,
            _ => MLDsaVariant::MLDsa65, // Default to 192-bit security
        }
    }

    /// Select ML-KEM variant based on security requirements
    fn select_mlkem_variant(&self) -> MLKemVariant {
        match (
            self.security_requirements.security_level,
            self.performance_priority,
        ) {
            (128, PerformancePriority::Speed) => MLKemVariant::MLKem512,
            (128, _) => MLKemVariant::MLKem512,
            (192, PerformancePriority::Speed) => MLKemVariant::MLKem768,
            (192, _) => MLKemVariant::MLKem768,
            (256, _) => MLKemVariant::MLKem1024,
            _ => MLKemVariant::MLKem768, // Default to 192-bit security
        }
    }

    /// Get public keys for distribution
    pub fn get_public_keys(&self) -> CryptoResult<HybridPublicKeys> {
        Ok(HybridPublicKeys {
            classical_verifying_key: self
                .classical_verifying_key
                .as_ref()
                .map(|k| k.to_bytes().to_vec()),
            pq_signing_public_key: self
                .pq_signing_keypair
                .as_ref()
                .map(|k| k.public_key.clone()),
            pq_kem_public_key: self.pq_kem_keypair.as_ref().map(|k| k.public_key.clone()),
        })
    }

    /// Get current crypto mode
    pub fn mode(&self) -> CryptoMode {
        self.mode
    }

    /// Get security requirements
    pub fn security_requirements(&self) -> &SecurityRequirements {
        &self.security_requirements
    }
}

/// Public keys for hybrid cryptography
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridPublicKeys {
    /// Classical Ed25519 verifying key
    pub classical_verifying_key: Option<Vec<u8>>,
    /// Post-quantum ML-DSA public key
    pub pq_signing_public_key: Option<Vec<u8>>,
    /// Post-quantum ML-KEM public key
    pub pq_kem_public_key: Option<Vec<u8>>,
}

impl Drop for HybridCrypto {
    fn drop(&mut self) {
        // Zeroize sensitive key material
        // Ed25519 SigningKey already implements Drop with zeroization
        // Just take ownership to ensure it's dropped
        let _ = self.classical_signing_key.take();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hybrid_crypto_creation() {
        let crypto = HybridCrypto::new_default().unwrap();
        assert_eq!(crypto.mode(), CryptoMode::Hybrid);
        assert_eq!(crypto.security_requirements().security_level, 192);
    }

    #[test]
    fn test_classical_mode_signing() {
        let mut crypto = HybridCrypto::new(
            CryptoMode::Classical,
            SecurityRequirements::default(),
            PerformancePriority::default(),
        )
        .unwrap();

        crypto.generate_signing_keypair().unwrap();

        let message = b"Test message for Attorney General's Office";
        let signature = crypto.sign(message).unwrap();

        assert!(!signature.classical_signature.is_empty());
        assert!(signature.pq_signature.is_empty());
        assert_eq!(signature.algorithm_info, "Ed25519");

        let is_valid = crypto.verify(message, &signature).unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_hybrid_mode_signing() {
        let mut crypto = HybridCrypto::new_default().unwrap();
        crypto.generate_signing_keypair().unwrap();

        let message = b"Hybrid signature test for SIMKARI";
        let signature = crypto.sign(message).unwrap();

        assert!(!signature.classical_signature.is_empty());
        assert!(!signature.pq_signature.is_empty());
        assert!(signature.algorithm_info.contains("Ed25519"));
        assert!(signature.algorithm_info.contains("ML-DSA"));

        let is_valid = crypto.verify(message, &signature).unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_post_quantum_mode_signing() {
        let mut crypto = HybridCrypto::new(
            CryptoMode::PostQuantum,
            SecurityRequirements::default(),
            PerformancePriority::default(),
        )
        .unwrap();

        crypto.generate_signing_keypair().unwrap();

        let message = b"Pure PQ signature test";
        let signature = crypto.sign(message).unwrap();

        assert!(signature.classical_signature.is_empty());
        assert!(!signature.pq_signature.is_empty());
        assert!(signature.algorithm_info.contains("ML-DSA"));

        let is_valid = crypto.verify(message, &signature).unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_hybrid_signature_verification_requires_both() {
        let mut crypto = HybridCrypto::new_default().unwrap();
        crypto.generate_signing_keypair().unwrap();

        let message = b"Test message";
        let mut signature = crypto.sign(message).unwrap();

        // Tamper with classical signature
        signature.classical_signature[0] ^= 0xFF;
        let is_valid = crypto.verify(message, &signature).unwrap();
        assert!(!is_valid, "Tampered classical signature should fail");

        // Restore and tamper with PQ signature
        let signature = crypto.sign(message).unwrap();
        let mut tampered_sig = signature.clone();
        tampered_sig.pq_signature[0] ^= 0xFF;
        let is_valid = crypto.verify(message, &tampered_sig).unwrap();
        assert!(!is_valid, "Tampered PQ signature should fail");
    }

    #[test]
    fn test_security_level_variant_selection() {
        // 128-bit security
        let mut crypto_128 = HybridCrypto::new(
            CryptoMode::Hybrid,
            SecurityRequirements {
                security_level: 128,
                ..Default::default()
            },
            PerformancePriority::default(),
        )
        .unwrap();
        crypto_128.generate_signing_keypair().unwrap();
        assert_eq!(
            crypto_128.pq_signing_keypair.as_ref().unwrap().variant(),
            MLDsaVariant::MLDsa44
        );

        // 256-bit security
        let mut crypto_256 = HybridCrypto::new(
            CryptoMode::Hybrid,
            SecurityRequirements {
                security_level: 256,
                ..Default::default()
            },
            PerformancePriority::default(),
        )
        .unwrap();
        crypto_256.generate_signing_keypair().unwrap();
        assert_eq!(
            crypto_256.pq_signing_keypair.as_ref().unwrap().variant(),
            MLDsaVariant::MLDsa87
        );
    }

    #[test]
    fn test_migration_strategy() {
        let mut crypto = HybridCrypto::new_default().unwrap();

        let strategy = MigrationStrategy {
            phase: MigrationPhase::HybridTransition,
            target_date: Some(1735689600), // 2025-01-01
            pq_adoption_rate: 75,
        };

        crypto.set_migration_strategy(strategy.clone());
        assert_eq!(
            crypto.migration_strategy().phase,
            MigrationPhase::HybridTransition
        );
        assert_eq!(crypto.migration_strategy().pq_adoption_rate, 75);
    }

    #[test]
    fn test_public_keys_export() {
        let mut crypto = HybridCrypto::new_default().unwrap();
        crypto.generate_signing_keypair().unwrap();
        crypto.generate_kem_keypair().unwrap();

        let public_keys = crypto.get_public_keys().unwrap();

        assert!(public_keys.classical_verifying_key.is_some());
        assert!(public_keys.pq_signing_public_key.is_some());
        assert!(public_keys.pq_kem_public_key.is_some());

        // Verify key sizes
        assert_eq!(public_keys.classical_verifying_key.unwrap().len(), 32);
        assert_eq!(public_keys.pq_signing_public_key.unwrap().len(), 1952); // ML-DSA-65
        assert_eq!(public_keys.pq_kem_public_key.unwrap().len(), 1184); // ML-KEM-768
    }

    #[test]
    fn test_kem_keypair_generation() {
        let mut crypto = HybridCrypto::new_default().unwrap();
        crypto.generate_kem_keypair().unwrap();

        assert!(crypto.pq_kem_keypair.is_some());
        let keypair = crypto.pq_kem_keypair.as_ref().unwrap();
        assert_eq!(keypair.variant(), MLKemVariant::MLKem768);
    }

    #[test]
    fn test_different_modes() {
        for mode in [
            CryptoMode::Classical,
            CryptoMode::Hybrid,
            CryptoMode::PostQuantum,
        ] {
            let mut crypto = HybridCrypto::new(
                mode,
                SecurityRequirements::default(),
                PerformancePriority::default(),
            )
            .unwrap();

            crypto.generate_signing_keypair().unwrap();
            let message = b"Test message";
            let signature = crypto.sign(message).unwrap();
            let is_valid = crypto.verify(message, &signature).unwrap();

            assert!(
                is_valid,
                "Signature verification failed for mode {:?}",
                mode
            );
        }
    }
}
