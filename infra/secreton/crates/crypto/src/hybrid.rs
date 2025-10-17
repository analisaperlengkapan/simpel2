//! Hybrid Cryptographic System for SIMKARI
//!
//! This module implements a hybrid cryptographic system that supports:
//! - Classical cryptography (Ed25519, AES-256-GCM)
//! - Post-quantum cryptography (ML-DSA, ML-KEM)
//! - Hybrid mode (Classical + Post-Quantum)
//! - Gradual migration strategy from classical to post-quantum

use crate::{
    encryption::{CryptoEngine, EncryptedData},
    error::{CryptoError, CryptoResult},
    pqc::{
        mldsa::{MLDsaProvider, MLDsaVariant},
        mlkem::{MLKemProvider, MLKemVariant},
        PostQuantumKeyExchange, PostQuantumSignatures,
    },
    AlgorithmId,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Cryptographic modes for hybrid operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CryptoMode {
    /// Classical cryptography only (Ed25519, AES-256-GCM)
    #[default]
    Classical,
    /// Hybrid mode (Classical + Post-Quantum for transition)
    Hybrid,
    /// Post-quantum only (ML-DSA, ML-KEM)
    PostQuantum,
}

impl std::fmt::Display for CryptoMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CryptoMode::Classical => write!(f, "Classical"),
            CryptoMode::Hybrid => write!(f, "Hybrid"),
            CryptoMode::PostQuantum => write!(f, "PostQuantum"),
        }
    }
}

/// Security requirements for algorithm selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRequirements {
    /// Minimum security level in bits (128, 192, 256)
    pub security_level: u32,
    /// Whether quantum resistance is required
    pub quantum_resistant: bool,
    /// Performance priority (High, Medium, Low)
    pub performance_priority: PerformancePriority,
    /// Compliance requirements
    pub compliance_requirements: Vec<String>,
}

/// Performance priority levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PerformancePriority {
    /// Prioritize speed over security level
    High,
    /// Balance speed and security
    Medium,
    /// Prioritize maximum security
    Low,
}

/// Migration strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStrategy {
    /// Current phase of migration
    pub current_phase: MigrationPhase,
    /// Target completion date
    pub target_date: Option<chrono::DateTime<chrono::Utc>>,
    /// Rollback capability enabled
    pub rollback_enabled: bool,
    /// Gradual rollout percentage (0-100)
    pub rollout_percentage: u8,
}

/// Migration phases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MigrationPhase {
    /// Phase 1: Classical only
    ClassicalOnly,
    /// Phase 2: Hybrid deployment (both classical and PQ)
    HybridDeployment,
    /// Phase 3: PQ preferred with classical fallback
    PostQuantumPreferred,
    /// Phase 4: Post-quantum only
    PostQuantumOnly,
}

/// Hybrid cryptographic engine
pub struct HybridCrypto {
    /// Classical cryptographic engine
    classical_engine: CryptoEngine,
    /// Post-quantum signature provider
    pq_signature: Box<dyn PostQuantumSignatures + Send + Sync>,
    /// Post-quantum key exchange provider
    pq_key_exchange: Box<dyn PostQuantumKeyExchange + Send + Sync>,
    /// Current cryptographic mode
    crypto_mode: CryptoMode,
    /// Migration strategy
    migration_strategy: MigrationStrategy,
    /// Algorithm selection cache
    algorithm_cache: Arc<RwLock<HashMap<String, AlgorithmSelection>>>,
    /// Performance metrics
    performance_metrics: Arc<RwLock<HybridPerformanceMetrics>>,
}

/// Algorithm selection result
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AlgorithmSelection {
    /// Selected mode
    mode: CryptoMode,
    /// Classical algorithm (if used)
    classical_algorithm: Option<AlgorithmId>,
    /// PQ signature algorithm (if used)
    pq_signature_algorithm: Option<String>,
    /// PQ key exchange algorithm (if used)
    pq_key_exchange_algorithm: Option<String>,
    /// Selection timestamp
    selected_at: chrono::DateTime<chrono::Utc>,
    /// Selection reason
    reason: String,
}

/// Performance metrics for hybrid operations
#[derive(Debug, Default, Clone)]
pub struct HybridPerformanceMetrics {
    /// Classical operations
    pub classical_ops: OperationMetrics,
    /// Hybrid operations
    pub hybrid_ops: OperationMetrics,
    /// Post-quantum operations
    pub post_quantum_ops: OperationMetrics,
    /// Algorithm selection metrics
    pub algorithm_selection: OperationMetrics,
    /// Migration metrics
    pub migration_metrics: MigrationMetrics,
}

/// Operation metrics
#[derive(Debug, Default, Clone)]
pub struct OperationMetrics {
    /// Total operations
    pub count: u64,
    /// Total time (milliseconds)
    pub total_time_ms: u64,
    /// Average time (milliseconds)
    pub avg_time_ms: f64,
    /// Success count
    pub success_count: u64,
    /// Failure count
    pub failure_count: u64,
}

/// Migration-specific metrics
#[derive(Debug, Default, Clone)]
pub struct MigrationMetrics {
    /// Operations by phase
    pub operations_by_phase: HashMap<String, u64>,
    /// Rollback events
    pub rollback_count: u64,
    /// Migration errors
    pub migration_errors: u64,
    /// Compatibility issues
    pub compatibility_issues: u64,
}

/// Hybrid encryption result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridEncryptionResult {
    /// Classical encryption (always present for compatibility)
    pub classical: EncryptedData,
    /// Post-quantum encryption (present in Hybrid/PostQuantum modes)
    pub post_quantum: Option<PostQuantumEncryption>,
    /// Mode used for encryption
    pub mode: CryptoMode,
    /// Algorithm information
    pub algorithm_info: AlgorithmInfo,
    /// Performance data
    pub performance: HybridOperationPerformance,
}

/// Post-quantum encryption data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostQuantumEncryption {
    /// Encapsulated key (ML-KEM)
    pub encapsulated_key: Vec<u8>,
    /// Encrypted data
    pub encrypted_data: Vec<u8>,
    /// Digital signature (ML-DSA)
    pub signature: Vec<u8>,
    /// Algorithm used
    pub algorithm: String,
}

/// Algorithm information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmInfo {
    /// Classical algorithm used
    pub classical: Option<String>,
    /// Post-quantum signature algorithm
    pub pq_signature: Option<String>,
    /// Post-quantum key exchange algorithm
    pub pq_key_exchange: Option<String>,
    /// Security level achieved
    pub security_level: u32,
}

/// Performance data for hybrid operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridOperationPerformance {
    /// Total operation time
    pub total_time_ms: u64,
    /// Classical operation time
    pub classical_time_ms: u64,
    /// Post-quantum operation time
    pub pq_time_ms: u64,
    /// Algorithm selection time
    pub selection_time_ms: u64,
}

impl Default for HybridCrypto {
    fn default() -> Self {
        Self::new()
    }
}

impl HybridCrypto {
    /// Create a new hybrid crypto engine
    pub fn new() -> Self {
        Self {
            classical_engine: CryptoEngine::new(),
            pq_signature: Box::new(MLDsaProvider::new(MLDsaVariant::MLDsa65)),
            pq_key_exchange: Box::new(MLKemProvider::new(MLKemVariant::MLKem768)),
            crypto_mode: CryptoMode::Classical,
            migration_strategy: MigrationStrategy {
                current_phase: MigrationPhase::ClassicalOnly,
                target_date: None,
                rollback_enabled: true,
                rollout_percentage: 0,
            },
            algorithm_cache: Arc::new(RwLock::new(HashMap::new())),
            performance_metrics: Arc::new(RwLock::new(HybridPerformanceMetrics::default())),
        }
    }

    /// Create hybrid crypto engine with specific mode
    pub fn with_mode(mode: CryptoMode) -> Self {
        let mut engine = Self::new();
        engine.crypto_mode = mode;
        engine
    }

    /// Create hybrid crypto engine with migration strategy
    pub fn with_migration_strategy(strategy: MigrationStrategy) -> Self {
        let mut engine = Self::new();
        // Set crypto mode based on migration phase
        engine.crypto_mode = match strategy.current_phase {
            MigrationPhase::ClassicalOnly => CryptoMode::Classical,
            MigrationPhase::HybridDeployment => CryptoMode::Hybrid,
            MigrationPhase::PostQuantumPreferred => CryptoMode::Hybrid,
            MigrationPhase::PostQuantumOnly => CryptoMode::PostQuantum,
        };

        engine.migration_strategy = strategy;
        engine
    }

    /// Set cryptographic mode
    pub fn set_crypto_mode(&mut self, mode: CryptoMode) {
        self.crypto_mode = mode;
        info!(mode = %mode, "Cryptographic mode updated");
    }

    /// Update migration strategy
    pub fn update_migration_strategy(&mut self, strategy: MigrationStrategy) {
        let old_phase = self.migration_strategy.current_phase.clone();
        self.migration_strategy = strategy;

        // Update crypto mode based on new phase
        self.crypto_mode = match self.migration_strategy.current_phase {
            MigrationPhase::ClassicalOnly => CryptoMode::Classical,
            MigrationPhase::HybridDeployment => CryptoMode::Hybrid,
            MigrationPhase::PostQuantumPreferred => CryptoMode::Hybrid,
            MigrationPhase::PostQuantumOnly => CryptoMode::PostQuantum,
        };

        info!(
            old_phase = ?old_phase,
            new_phase = ?self.migration_strategy.current_phase,
            new_mode = %self.crypto_mode,
            rollout_percentage = self.migration_strategy.rollout_percentage,
            "Migration strategy updated"
        );
    }

    /// Select algorithms based on security requirements
    pub async fn select_algorithms(&self, requirements: &SecurityRequirements) -> CryptoResult<AlgorithmSelection> {
        let start = Instant::now();

        // Generate cache key
        let cache_key = format!(
            "{}:{}:{}:{:?}",
            requirements.security_level,
            requirements.quantum_resistant,
            requirements.performance_priority as u8,
            requirements.compliance_requirements
        );

        // Check cache first
        {
            let cache = self.algorithm_cache.read().await;
            if let Some(cached) = cache.get(&cache_key) {
                // Use cached selection if it's recent (within 1 hour)
                if chrono::Utc::now().signed_duration_since(cached.selected_at).num_hours() < 1 {
                    return Ok(cached.clone());
                }
            }
        }

        // Select mode based on requirements and migration strategy
        let mode = self.select_mode_for_requirements(requirements).await?;

        // Select specific algorithms
        let classical_algorithm = if mode != CryptoMode::PostQuantum {
            Some(self.select_classical_algorithm(requirements))
        } else {
            None
        };

        let (pq_signature_algorithm, pq_key_exchange_algorithm) = if mode != CryptoMode::Classical {
            let sig_alg = self.select_pq_signature_algorithm(requirements);
            let kex_alg = self.select_pq_key_exchange_algorithm(requirements);
            (Some(sig_alg), Some(kex_alg))
        } else {
            (None, None)
        };

        let selection = AlgorithmSelection {
            mode,
            classical_algorithm,
            pq_signature_algorithm,
            pq_key_exchange_algorithm,
            selected_at: chrono::Utc::now(),
            reason: format!(
                "Security level: {}, Quantum resistant: {}, Performance: {:?}, Phase: {:?}",
                requirements.security_level,
                requirements.quantum_resistant,
                requirements.performance_priority,
                self.migration_strategy.current_phase
            ),
        };

        // Cache the selection
        {
            let mut cache = self.algorithm_cache.write().await;
            cache.insert(cache_key, selection.clone());
        }

        // Update metrics
        let duration = start.elapsed();
        self.update_selection_metrics(duration, true).await;

        debug!(
            mode = %selection.mode,
            classical = ?selection.classical_algorithm,
            pq_signature = ?selection.pq_signature_algorithm,
            pq_key_exchange = ?selection.pq_key_exchange_algorithm,
            duration_ms = duration.as_millis(),
            "Algorithm selection completed"
        );

        Ok(selection)
    }

    /// Encrypt data using hybrid approach
    pub async fn encrypt(&self, data: &[u8], requirements: &SecurityRequirements) -> CryptoResult<HybridEncryptionResult> {
        let start = Instant::now();

        // Select algorithms
        let selection = self.select_algorithms(requirements).await?;
        let selection_time = start.elapsed();

        // Perform encryption based on selected mode
        let result = match selection.mode {
            CryptoMode::Classical => {
                self.encrypt_classical(data, selection.classical_algorithm.unwrap()).await?
            }
            CryptoMode::Hybrid => {
                self.encrypt_hybrid(data, &selection).await?
            }
            CryptoMode::PostQuantum => {
                self.encrypt_post_quantum(data, &selection).await?
            }
        };

        let total_time = start.elapsed();

        // Update performance metrics
        self.update_operation_metrics(&selection.mode, total_time, true).await;

        info!(
            mode = %selection.mode,
            data_size = data.len(),
            total_time_ms = total_time.as_millis(),
            selection_time_ms = selection_time.as_millis(),
            "Hybrid encryption completed"
        );

        Ok(HybridEncryptionResult {
            classical: result.classical,
            post_quantum: result.post_quantum,
            mode: selection.mode,
            algorithm_info: AlgorithmInfo {
                classical: selection.classical_algorithm.map(|a| a.to_string()),
                pq_signature: selection.pq_signature_algorithm,
                pq_key_exchange: selection.pq_key_exchange_algorithm,
                security_level: requirements.security_level,
            },
            performance: HybridOperationPerformance {
                total_time_ms: total_time.as_millis() as u64,
                classical_time_ms: result.performance.classical_time_ms,
                pq_time_ms: result.performance.pq_time_ms,
                selection_time_ms: selection_time.as_millis() as u64,
            },
        })
    }

    /// Decrypt data using hybrid approach
    pub async fn decrypt(&self, encrypted: &HybridEncryptionResult) -> CryptoResult<Vec<u8>> {
        let start = Instant::now();

        let data = match encrypted.mode {
            CryptoMode::Classical => {
                self.decrypt_classical(&encrypted.classical).await?
            }
            CryptoMode::Hybrid => {
                self.decrypt_hybrid(&encrypted.classical, encrypted.post_quantum.as_ref()).await?
            }
            CryptoMode::PostQuantum => {
                if let Some(ref pq_data) = encrypted.post_quantum {
                    self.decrypt_post_quantum(pq_data).await?
                } else {
                    return Err(CryptoError::DecryptionFailed("Missing post-quantum data".to_string()));
                }
            }
        };

        let duration = start.elapsed();
        self.update_operation_metrics(&encrypted.mode, duration, true).await;

        debug!(
            mode = %encrypted.mode,
            data_size = data.len(),
            duration_ms = duration.as_millis(),
            "Hybrid decryption completed"
        );

        Ok(data)
    }

    /// Get current performance metrics
    pub async fn get_performance_metrics(&self) -> HybridPerformanceMetrics {
        (*self.performance_metrics.read().await).clone()
    }

    /// Clear algorithm cache
    pub async fn clear_algorithm_cache(&self) {
        let mut cache = self.algorithm_cache.write().await;
        cache.clear();
        info!("Algorithm cache cleared");
    }

    /// Check if migration to next phase is recommended
    pub async fn should_migrate_to_next_phase(&self) -> CryptoResult<bool> {
        let metrics = self.get_performance_metrics().await;

        match self.migration_strategy.current_phase {
            MigrationPhase::ClassicalOnly => {
                // Migrate to hybrid if PQ readiness is high
                Ok(self.migration_strategy.rollout_percentage >= 25)
            }
            MigrationPhase::HybridDeployment => {
                // Migrate to PQ preferred if hybrid operations are stable
                let hybrid_success_rate = if metrics.hybrid_ops.count > 0 {
                    metrics.hybrid_ops.success_count as f64 / metrics.hybrid_ops.count as f64
                } else {
                    0.0
                };
                Ok(hybrid_success_rate > 0.95 && self.migration_strategy.rollout_percentage >= 75)
            }
            MigrationPhase::PostQuantumPreferred => {
                // Migrate to PQ only if PQ operations are stable
                let pq_success_rate = if metrics.post_quantum_ops.count > 0 {
                    metrics.post_quantum_ops.success_count as f64 / metrics.post_quantum_ops.count as f64
                } else {
                    0.0
                };
                Ok(pq_success_rate > 0.99 && self.migration_strategy.rollout_percentage >= 95)
            }
            MigrationPhase::PostQuantumOnly => {
                // Already at final phase
                Ok(false)
            }
        }
    }

    // Private helper methods

    async fn select_mode_for_requirements(&self, requirements: &SecurityRequirements) -> CryptoResult<CryptoMode> {
        // If quantum resistance is required, use PQ or Hybrid
        if requirements.quantum_resistant {
            match self.migration_strategy.current_phase {
                MigrationPhase::ClassicalOnly => Ok(CryptoMode::Classical), // Not ready yet
                MigrationPhase::HybridDeployment => Ok(CryptoMode::Hybrid),
                MigrationPhase::PostQuantumPreferred => Ok(CryptoMode::PostQuantum),
                MigrationPhase::PostQuantumOnly => Ok(CryptoMode::PostQuantum),
            }
        } else {
            // Use current mode based on migration strategy
            Ok(self.crypto_mode)
        }
    }

    fn select_classical_algorithm(&self, requirements: &SecurityRequirements) -> AlgorithmId {
        match requirements.performance_priority {
            PerformancePriority::High => AlgorithmId::ChaCha20Poly1305, // Faster on some platforms
            PerformancePriority::Medium | PerformancePriority::Low => AlgorithmId::Aes256Gcm, // Standard choice
        }
    }

    fn select_pq_signature_algorithm(&self, requirements: &SecurityRequirements) -> String {
        match (requirements.security_level, &requirements.performance_priority) {
            (128, PerformancePriority::High) => "ML-DSA-44".to_string(),
            (128, _) => "ML-DSA-44".to_string(),
            (192, PerformancePriority::High) => "ML-DSA-65".to_string(),
            (192, _) => "ML-DSA-65".to_string(),
            (256, _) => "ML-DSA-87".to_string(),
            _ => "ML-DSA-65".to_string(), // Default
        }
    }

    fn select_pq_key_exchange_algorithm(&self, requirements: &SecurityRequirements) -> String {
        match (requirements.security_level, &requirements.performance_priority) {
            (128, PerformancePriority::High) => "ML-KEM-512".to_string(),
            (128, _) => "ML-KEM-512".to_string(),
            (192, PerformancePriority::High) => "ML-KEM-768".to_string(),
            (192, _) => "ML-KEM-768".to_string(),
            (256, _) => "ML-KEM-1024".to_string(),
            _ => "ML-KEM-768".to_string(), // Default
        }
    }

    async fn encrypt_classical(&self, data: &[u8], algorithm: AlgorithmId) -> CryptoResult<HybridEncryptionResult> {
        let start = Instant::now();

        let key = crate::generate_key(algorithm)?;
        let encrypted = self.classical_engine.encrypt(algorithm, data, &key)?;

        let duration = start.elapsed();

        Ok(HybridEncryptionResult {
            classical: encrypted,
            post_quantum: None,
            mode: CryptoMode::Classical,
            algorithm_info: AlgorithmInfo {
                classical: Some(algorithm.to_string()),
                pq_signature: None,
                pq_key_exchange: None,
                security_level: 256, // AES-256 equivalent
            },
            performance: HybridOperationPerformance {
                total_time_ms: duration.as_millis() as u64,
                classical_time_ms: duration.as_millis() as u64,
                pq_time_ms: 0,
                selection_time_ms: 0,
            },
        })
    }

    async fn encrypt_hybrid(&self, data: &[u8], selection: &AlgorithmSelection) -> CryptoResult<HybridEncryptionResult> {
        let start = Instant::now();

        // Classical encryption
        let classical_start = Instant::now();
        let classical_result = self.encrypt_classical(data, selection.classical_algorithm.unwrap()).await?;
        let classical_time = classical_start.elapsed();

        // Post-quantum operations
        let pq_start = Instant::now();

        // Generate PQ key pair for key encapsulation
        let (pq_public_key, _pq_private_key) = self.pq_key_exchange.keypair_generate()
            .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

        // Encapsulate a shared secret
        let (ciphertext, shared_secret) = self.pq_key_exchange.encapsulate(&pq_public_key)
            .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

        // Use shared secret to encrypt data (simplified - in practice would use KDF)
        let pq_key = if shared_secret.len() >= 32 {
            shared_secret[..32].to_vec()
        } else {
            // Extend with HKDF or similar
            shared_secret
        };

        let pq_encrypted = self.classical_engine.encrypt(AlgorithmId::Aes256Gcm, data, &pq_key)?;

        // Sign the data for integrity
        let (_sig_public_key, sig_private_key) = self.pq_signature.keypair_generate()
            .map_err(|e| CryptoError::SigningFailed(e.to_string()))?;

        let signature = self.pq_signature.sign(data, &sig_private_key)
            .map_err(|e| CryptoError::SigningFailed(e.to_string()))?;

        let pq_time = pq_start.elapsed();
        let total_time = start.elapsed();

        let post_quantum = PostQuantumEncryption {
            encapsulated_key: ciphertext,
            encrypted_data: pq_encrypted.ciphertext,
            signature,
            algorithm: format!("{}/{}",
                selection.pq_key_exchange_algorithm.as_ref().unwrap(),
                selection.pq_signature_algorithm.as_ref().unwrap()
            ),
        };

        Ok(HybridEncryptionResult {
            classical: classical_result.classical,
            post_quantum: Some(post_quantum),
            mode: CryptoMode::Hybrid,
            algorithm_info: AlgorithmInfo {
                classical: selection.classical_algorithm.map(|a| a.to_string()),
                pq_signature: selection.pq_signature_algorithm.clone(),
                pq_key_exchange: selection.pq_key_exchange_algorithm.clone(),
                security_level: 256, // Hybrid provides maximum security
            },
            performance: HybridOperationPerformance {
                total_time_ms: total_time.as_millis() as u64,
                classical_time_ms: classical_time.as_millis() as u64,
                pq_time_ms: pq_time.as_millis() as u64,
                selection_time_ms: 0,
            },
        })
    }

    async fn encrypt_post_quantum(&self, data: &[u8], selection: &AlgorithmSelection) -> CryptoResult<HybridEncryptionResult> {
        let start = Instant::now();

        // For now, use classical encryption with PQ key derivation
        // In a full implementation, this would use pure PQ algorithms
        let (pq_public_key, _pq_private_key) = self.pq_key_exchange.keypair_generate()
            .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

        let (ciphertext, shared_secret) = self.pq_key_exchange.encapsulate(&pq_public_key)
            .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

        // Use shared secret as encryption key
        let key = if shared_secret.len() >= 32 {
            shared_secret[..32].to_vec()
        } else {
            shared_secret
        };

        let encrypted = self.classical_engine.encrypt(AlgorithmId::Aes256Gcm, data, &key)?;

        // Generate PQ signature
        let (_sig_public_key, sig_private_key) = self.pq_signature.keypair_generate()
            .map_err(|e| CryptoError::SigningFailed(e.to_string()))?;

        let signature = self.pq_signature.sign(data, &sig_private_key)
            .map_err(|e| CryptoError::SigningFailed(e.to_string()))?;

        let duration = start.elapsed();

        let post_quantum = PostQuantumEncryption {
            encapsulated_key: ciphertext,
            encrypted_data: encrypted.ciphertext.clone(),
            signature,
            algorithm: format!("{}/{}",
                selection.pq_key_exchange_algorithm.as_ref().unwrap(),
                selection.pq_signature_algorithm.as_ref().unwrap()
            ),
        };

        Ok(HybridEncryptionResult {
            classical: encrypted, // Keep for compatibility
            post_quantum: Some(post_quantum),
            mode: CryptoMode::PostQuantum,
            algorithm_info: AlgorithmInfo {
                classical: None,
                pq_signature: selection.pq_signature_algorithm.clone(),
                pq_key_exchange: selection.pq_key_exchange_algorithm.clone(),
                security_level: 256,
            },
            performance: HybridOperationPerformance {
                total_time_ms: duration.as_millis() as u64,
                classical_time_ms: 0,
                pq_time_ms: duration.as_millis() as u64,
                selection_time_ms: 0,
            },
        })
    }

    async fn decrypt_classical(&self, encrypted: &EncryptedData) -> CryptoResult<Vec<u8>> {
        // In a real implementation, we'd need to retrieve the key
        let key = crate::generate_key(encrypted.algorithm)?;
        self.classical_engine.decrypt(encrypted, &key)
    }

    async fn decrypt_hybrid(&self, encrypted: &EncryptedData, pq_data: Option<&PostQuantumEncryption>) -> CryptoResult<Vec<u8>> {
        // For now, just decrypt the classical part
        // In a full implementation, this would verify both classical and PQ components
        if let Some(_pq) = pq_data {
            // TODO: Verify PQ signature and decrypt using PQ key
            warn!("PQ verification not yet implemented, falling back to classical");
        }
        self.decrypt_classical(encrypted).await
    }

    async fn decrypt_post_quantum(&self, _pq_data: &PostQuantumEncryption) -> CryptoResult<Vec<u8>> {
        // Placeholder for pure PQ decryption
        // In a real implementation, this would:
        // 1. Decapsulate the shared secret using ML-KEM
        // 2. Decrypt the data using the shared secret
        // 3. Verify the ML-DSA signature
        Err(CryptoError::DecryptionFailed("Pure PQ decryption not yet implemented".to_string()))
    }

    async fn update_operation_metrics(&self, mode: &CryptoMode, duration: Duration, success: bool) {
        let mut metrics = self.performance_metrics.write().await;
        let duration_ms = duration.as_millis() as u64;

        let op_metrics = match mode {
            CryptoMode::Classical => &mut metrics.classical_ops,
            CryptoMode::Hybrid => &mut metrics.hybrid_ops,
            CryptoMode::PostQuantum => &mut metrics.post_quantum_ops,
        };

        op_metrics.count += 1;
        op_metrics.total_time_ms += duration_ms;
        op_metrics.avg_time_ms = op_metrics.total_time_ms as f64 / op_metrics.count as f64;

        if success {
            op_metrics.success_count += 1;
        } else {
            op_metrics.failure_count += 1;
        }

        // Update migration metrics
        let phase_key = format!("{:?}", self.migration_strategy.current_phase);
        *metrics.migration_metrics.operations_by_phase.entry(phase_key).or_insert(0) += 1;
    }

    async fn update_selection_metrics(&self, duration: Duration, success: bool) {
        let mut metrics = self.performance_metrics.write().await;
        let duration_ms = duration.as_millis() as u64;

        metrics.algorithm_selection.count += 1;
        metrics.algorithm_selection.total_time_ms += duration_ms;
        metrics.algorithm_selection.avg_time_ms =
            metrics.algorithm_selection.total_time_ms as f64 / metrics.algorithm_selection.count as f64;

        if success {
            metrics.algorithm_selection.success_count += 1;
        } else {
            metrics.algorithm_selection.failure_count += 1;
        }
    }
}

impl OperationMetrics {
    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            (self.success_count as f64 / self.count as f64) * 100.0
        }
    }

    /// Get failure rate as percentage
    pub fn failure_rate(&self) -> f64 {
        100.0 - self.success_rate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hybrid_crypto_creation() {
        let hybrid = HybridCrypto::new();
        assert_eq!(hybrid.crypto_mode, CryptoMode::Classical);
    }

    #[tokio::test]
    async fn test_crypto_mode_setting() {
        let mut hybrid = HybridCrypto::new();
        hybrid.set_crypto_mode(CryptoMode::Hybrid);
        assert_eq!(hybrid.crypto_mode, CryptoMode::Hybrid);
    }

    #[tokio::test]
    async fn test_algorithm_selection() {
        let hybrid = HybridCrypto::new();

        let requirements = SecurityRequirements {
            security_level: 192,
            quantum_resistant: false,
            performance_priority: PerformancePriority::Medium,
            compliance_requirements: vec!["FIPS-140-2".to_string()],
        };

        let selection = hybrid.select_algorithms(&requirements).await;
        assert!(selection.is_ok());

        let selection = selection.unwrap();
        assert_eq!(selection.mode, CryptoMode::Classical);
        assert!(selection.classical_algorithm.is_some());
    }

    #[tokio::test]
    async fn test_classical_encryption() {
        let hybrid = HybridCrypto::new();

        let requirements = SecurityRequirements {
            security_level: 256,
            quantum_resistant: false,
            performance_priority: PerformancePriority::Medium,
            compliance_requirements: vec![],
        };

        let data = b"test secret data";
        let result = hybrid.encrypt(data, &requirements).await;
        assert!(result.is_ok());

        let encrypted = result.unwrap();
        assert_eq!(encrypted.mode, CryptoMode::Classical);
        assert!(encrypted.post_quantum.is_none());

        // Test decryption
        let decrypted = hybrid.decrypt(&encrypted).await;
        assert!(decrypted.is_ok());
        assert_eq!(decrypted.unwrap(), data);
    }

    #[tokio::test]
    async fn test_migration_strategy() {
        let strategy = MigrationStrategy {
            current_phase: MigrationPhase::HybridDeployment,
            target_date: Some(chrono::Utc::now() + chrono::Duration::days(90)),
            rollback_enabled: true,
            rollout_percentage: 50,
        };

        let mut hybrid = HybridCrypto::with_migration_strategy(strategy);
        assert_eq!(hybrid.crypto_mode, CryptoMode::Hybrid);

        // Update to next phase
        let new_strategy = MigrationStrategy {
            current_phase: MigrationPhase::PostQuantumPreferred,
            target_date: Some(chrono::Utc::now() + chrono::Duration::days(60)),
            rollback_enabled: true,
            rollout_percentage: 75,
        };

        hybrid.update_migration_strategy(new_strategy);
        assert_eq!(hybrid.crypto_mode, CryptoMode::Hybrid);
    }

    #[tokio::test]
    async fn test_performance_metrics() {
        let hybrid = HybridCrypto::new();

        let requirements = SecurityRequirements {
            security_level: 128,
            quantum_resistant: false,
            performance_priority: PerformancePriority::High,
            compliance_requirements: vec![],
        };

        // Perform some operations
        let data = b"test data";
        let _result = hybrid.encrypt(data, &requirements).await;

        let metrics = hybrid.get_performance_metrics().await;
        assert!(metrics.classical_ops.count > 0);
    }
}
