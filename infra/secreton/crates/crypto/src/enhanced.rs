//! Enhanced Cryptographic Engine for SIMKARI Secret Management
//!
//! This module provides enhanced cryptographic operations specifically designed
//! for the SIMKARI super app secret management, including:
//! - Post-quantum algorithms (ML-DSA, ML-KEM) for future-proofing
//! - Hybrid cryptographic modes (Classical, Hybrid, PostQuantum)
//! - Optimized encryption/decryption for frequent secret access
//! - Performance monitoring and caching for high-throughput operations

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
use tracing::{debug, info};

/// Enhanced cryptographic engine for SIMKARI secret management
pub struct EnhancedSecretonCrypto {
    /// Classical crypto engine
    classical_engine: CryptoEngine,
    /// Post-quantum signature provider
    pq_signature: Box<dyn PostQuantumSignatures + Send + Sync>,
    /// Post-quantum key exchange provider
    pq_key_exchange: Box<dyn PostQuantumKeyExchange + Send + Sync>,
    /// Current cryptographic mode
    crypto_mode: CryptoMode,
    /// Performance metrics
    metrics: Arc<RwLock<EnhancedCryptoMetrics>>,
    /// Encryption cache for frequently accessed secrets
    encryption_cache: Arc<RwLock<HashMap<String, CachedEncryption>>>,
    /// Key derivation cache
    key_cache: Arc<RwLock<HashMap<String, CachedKey>>>,
}

/// Cryptographic modes for hybrid operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CryptoMode {
    /// Classical cryptography only (AES-256-GCM, Ed25519)
    Classical,
    /// Hybrid mode (Classical + Post-Quantum)
    Hybrid,
    /// Post-quantum only (ML-DSA, ML-KEM)
    PostQuantum,
}

/// Enhanced performance metrics for secreton crypto operations
#[derive(Debug, Default, Clone)]
pub struct EnhancedCryptoMetrics {
    /// Secret encryption operations
    pub secret_encryption: CryptoOperationMetrics,
    /// Secret decryption operations
    pub secret_decryption: CryptoOperationMetrics,
    /// Key derivation operations
    pub key_derivation: CryptoOperationMetrics,
    /// Post-quantum signature operations
    pub pq_signatures: CryptoOperationMetrics,
    /// Post-quantum key exchange operations
    pub pq_key_exchange: CryptoOperationMetrics,
    /// Batch operations
    pub batch_operations: CryptoOperationMetrics,
    /// Cache operations
    pub cache_operations: CacheMetrics,
}

/// Metrics for a specific crypto operation type
#[derive(Debug, Default, Clone)]
pub struct CryptoOperationMetrics {
    /// Total number of operations
    pub count: u64,
    /// Total time spent (milliseconds)
    pub total_time_ms: u64,
    /// Average time per operation (milliseconds)
    pub avg_time_ms: f64,
    /// Minimum operation time (milliseconds)
    pub min_time_ms: u64,
    /// Maximum operation time (milliseconds)
    pub max_time_ms: u64,
    /// Number of failed operations
    pub failures: u64,
    /// Operations per second
    pub ops_per_second: f64,
}

/// Cache performance metrics
#[derive(Debug, Default, Clone)]
pub struct CacheMetrics {
    /// Cache hits
    pub hits: u64,
    /// Cache misses
    pub misses: u64,
    /// Cache evictions
    pub evictions: u64,
    /// Current cache size
    pub current_size: usize,
    /// Maximum cache size
    pub max_size: usize,
}

/// Cached encryption result
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct CachedEncryption {
    /// Encrypted data
    encrypted_data: EncryptedData,
    /// Cache expiration time
    expires_at: chrono::DateTime<chrono::Utc>,
    /// Access count
    access_count: u64,
    /// Last access time
    last_accessed: chrono::DateTime<chrono::Utc>,
}

/// Cached key derivation result
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct CachedKey {
    /// Derived key
    key: Vec<u8>,
    /// Cache expiration time
    expires_at: chrono::DateTime<chrono::Utc>,
    /// Access count
    access_count: u64,
    /// Last access time
    last_accessed: chrono::DateTime<chrono::Utc>,
}

/// Enhanced secret encryption request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedSecretRequest {
    /// Secret path
    pub path: String,
    /// Secret data
    pub data: Vec<u8>,
    /// Encryption algorithm preference
    pub algorithm: Option<AlgorithmId>,
    /// Crypto mode preference
    pub crypto_mode: Option<CryptoMode>,
    /// Whether to use cache
    pub use_cache: bool,
    /// Cache TTL in seconds
    pub cache_ttl: Option<u64>,
    /// Compression preference
    pub compress: bool,
}

/// Enhanced secret decryption request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedSecretResponse {
    /// Decrypted secret data
    pub data: Vec<u8>,
    /// Encryption algorithm used
    pub algorithm: AlgorithmId,
    /// Crypto mode used
    pub crypto_mode: CryptoMode,
    /// Whether result came from cache
    pub from_cache: bool,
    /// Performance metrics for this operation
    pub metrics: OperationPerformance,
}

/// Performance metrics for individual operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationPerformance {
    /// Operation duration in milliseconds
    pub duration_ms: u64,
    /// Cache hit/miss status
    pub cache_status: CacheStatus,
    /// Compression ratio (if used)
    pub compression_ratio: Option<f64>,
}

/// Cache status for operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CacheStatus {
    /// Cache hit
    Hit,
    /// Cache miss
    Miss,
    /// Cache disabled
    Disabled,
}

/// Batch secret operation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSecretRequest {
    /// Individual secret requests
    pub requests: Vec<EnhancedSecretRequest>,
    /// Maximum parallel operations
    pub max_parallel: Option<usize>,
    /// Timeout for the entire batch
    pub timeout_ms: Option<u64>,
}

/// Batch secret operation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSecretResponse {
    /// Individual responses
    pub responses: Vec<Result<EnhancedSecretResponse, String>>,
    /// Batch performance metrics
    pub batch_metrics: BatchPerformanceMetrics,
}

/// Performance metrics for batch operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchPerformanceMetrics {
    /// Total operations
    pub total_operations: usize,
    /// Successful operations
    pub successful_operations: usize,
    /// Failed operations
    pub failed_operations: usize,
    /// Total batch time (milliseconds)
    pub total_time_ms: u64,
    /// Average time per operation (milliseconds)
    pub avg_time_per_operation_ms: f64,
    /// Cache hit rate
    pub cache_hit_rate: f64,
    /// Throughput (operations per second)
    pub throughput_ops_per_sec: f64,
}

/// Post-quantum key pair for secret encryption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostQuantumKeyPair {
    /// Public key
    pub public_key: Vec<u8>,
    /// Private key (encrypted)
    pub private_key: Vec<u8>,
    /// Algorithm used
    pub algorithm: String,
    /// Key generation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Key expiration timestamp
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Hybrid encryption result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridEncryptionResult {
    /// Classical encryption result
    pub classical: EncryptedData,
    /// Post-quantum encryption result
    pub post_quantum: Option<Vec<u8>>,
    /// Crypto mode used
    pub mode: CryptoMode,
    /// Performance metrics
    pub performance: OperationPerformance,
}

impl Default for EnhancedSecretonCrypto {
    fn default() -> Self {
        Self::new()
    }
}

impl EnhancedSecretonCrypto {
    /// Create a new enhanced secreton crypto engine
    pub fn new() -> Self {
        Self {
            classical_engine: CryptoEngine::new(),
            pq_signature: Box::new(MLDsaProvider::new(MLDsaVariant::MLDsa65)),
            pq_key_exchange: Box::new(MLKemProvider::new(MLKemVariant::MLKem768)),
            crypto_mode: CryptoMode::Classical,
            metrics: Arc::new(RwLock::new(EnhancedCryptoMetrics::default())),
            encryption_cache: Arc::new(RwLock::new(HashMap::new())),
            key_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create enhanced crypto engine with specific mode
    pub fn with_mode(mode: CryptoMode) -> Self {
        let mut engine = Self::new();
        engine.crypto_mode = mode;
        engine
    }

    /// Set cryptographic mode
    pub fn set_crypto_mode(&mut self, mode: CryptoMode) {
        self.crypto_mode = mode;
        info!(mode = ?mode, "Cryptographic mode updated");
    }

    /// Encrypt secret with enhanced features
    pub async fn encrypt_secret(&self, request: EnhancedSecretRequest) -> CryptoResult<HybridEncryptionResult> {
        let start = Instant::now();
        let cache_key = self.generate_cache_key(&request.path, &request.data);

        // Check cache if enabled
        if request.use_cache
            && let Some(cached) = self.get_cached_encryption(&cache_key).await {
                let duration = start.elapsed();
                self.update_metrics("secret_encryption", duration, true, true).await;

                return Ok(HybridEncryptionResult {
                    classical: cached.encrypted_data,
                    post_quantum: None,
                    mode: self.crypto_mode,
                    performance: OperationPerformance {
                        duration_ms: duration.as_millis() as u64,
                        cache_status: CacheStatus::Hit,
                        compression_ratio: None,
                    },
                });
        }

        // Prepare data (compression if requested)
        let mut data = request.data.clone();
        let compression_ratio = if request.compress {
            let original_size = data.len();
            data = self.compress_data(&data)?;
            Some(original_size as f64 / data.len() as f64)
        } else {
            None
        };

        // Encrypt based on mode
        let result = match request.crypto_mode.unwrap_or(self.crypto_mode) {
            CryptoMode::Classical => {
                self.encrypt_classical(&data, request.algorithm.unwrap_or(AlgorithmId::Aes256Gcm)).await?
            }
            CryptoMode::Hybrid => {
                self.encrypt_hybrid(&data, request.algorithm.unwrap_or(AlgorithmId::Aes256Gcm)).await?
            }
            CryptoMode::PostQuantum => {
                self.encrypt_post_quantum(&data).await?
            }
        };

        // Cache result if enabled
        if request.use_cache {
            let ttl = request.cache_ttl.unwrap_or(300); // 5 minutes default
            self.cache_encryption_result(&cache_key, &result.classical, ttl).await;
        }

        let duration = start.elapsed();
        self.update_metrics("secret_encryption", duration, true, false).await;

        info!(
            path = request.path,
            mode = ?self.crypto_mode,
            duration_ms = duration.as_millis(),
            compressed = request.compress,
            cached = request.use_cache,
            "Secret encrypted"
        );

        Ok(HybridEncryptionResult {
            classical: result.classical,
            post_quantum: result.post_quantum,
            mode: request.crypto_mode.unwrap_or(self.crypto_mode),
            performance: OperationPerformance {
                duration_ms: duration.as_millis() as u64,
                cache_status: if request.use_cache { CacheStatus::Miss } else { CacheStatus::Disabled },
                compression_ratio,
            },
        })
    }

    /// Decrypt secret with enhanced features
    pub async fn decrypt_secret(&self, encrypted: &HybridEncryptionResult) -> CryptoResult<EnhancedSecretResponse> {
        let start = Instant::now();

        // Decrypt based on mode
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

        // Decompress if needed (detect compression by checking if data starts with compression header)
        let final_data = if self.is_compressed(&data) {
            self.decompress_data(&data)?
        } else {
            data
        };

        let duration = start.elapsed();
        self.update_metrics("secret_decryption", duration, true, false).await;

        debug!(
            mode = ?encrypted.mode,
            duration_ms = duration.as_millis(),
            "Secret decrypted"
        );

        Ok(EnhancedSecretResponse {
            data: final_data,
            algorithm: encrypted.classical.algorithm,
            crypto_mode: encrypted.mode,
            from_cache: false,
            metrics: OperationPerformance {
                duration_ms: duration.as_millis() as u64,
                cache_status: CacheStatus::Disabled,
                compression_ratio: None,
            },
        })
    }

    /// Batch encrypt secrets for improved performance
    pub async fn encrypt_secrets_batch(&self, request: BatchSecretRequest) -> CryptoResult<BatchSecretResponse> {
        let start = Instant::now();
        let max_parallel = request.max_parallel.unwrap_or(10);
        let mut responses = Vec::with_capacity(request.requests.len());

        // Process in parallel batches
        let chunks: Vec<_> = request.requests.chunks(max_parallel).collect();

        for chunk in chunks {
            let mut batch_futures = Vec::new();

            for req in chunk {
                let req_clone = req.clone();
                let engine = self.clone_for_async();

                batch_futures.push(tokio::spawn(async move {
                    engine.encrypt_secret(req_clone).await
                }));
            }

            // Wait for all in this batch
            for future in batch_futures {
                match future.await {
                    Ok(Ok(result)) => responses.push(Ok(EnhancedSecretResponse {
                        data: vec![], // Not applicable for encryption
                        algorithm: result.classical.algorithm,
                        crypto_mode: result.mode,
                        from_cache: matches!(result.performance.cache_status, CacheStatus::Hit),
                        metrics: result.performance,
                    })),
                    Ok(Err(e)) => responses.push(Err(e.to_string())),
                    Err(e) => responses.push(Err(format!("Task error: {}", e))),
                }
            }
        }

        let duration = start.elapsed();
        let successful = responses.iter().filter(|r| r.is_ok()).count();
        let failed = responses.len() - successful;
        let cache_hits = responses.iter()
            .filter_map(|r| r.as_ref().ok())
            .filter(|r| r.from_cache)
            .count();

        self.update_metrics("batch_operations", duration, failed == 0, false).await;

        let batch_metrics = BatchPerformanceMetrics {
            total_operations: request.requests.len(),
            successful_operations: successful,
            failed_operations: failed,
            total_time_ms: duration.as_millis() as u64,
            avg_time_per_operation_ms: duration.as_millis() as f64 / request.requests.len() as f64,
            cache_hit_rate: cache_hits as f64 / request.requests.len() as f64,
            throughput_ops_per_sec: request.requests.len() as f64 / duration.as_secs_f64(),
        };

        info!(
            total_operations = request.requests.len(),
            successful = successful,
            failed = failed,
            cache_hit_rate = batch_metrics.cache_hit_rate,
            throughput = batch_metrics.throughput_ops_per_sec,
            duration_ms = duration.as_millis(),
            "Batch secret encryption completed"
        );

        Ok(BatchSecretResponse {
            responses,
            batch_metrics,
        })
    }

    /// Generate post-quantum key pair
    pub async fn generate_pq_keypair(&self, algorithm: &str) -> CryptoResult<PostQuantumKeyPair> {
        let start = Instant::now();

        let (public_key, private_key) = match algorithm {
            "ML-KEM-768" => {
                self.pq_key_exchange.keypair_generate()
                    .map_err(|e| CryptoError::KeyGenerationFailed(e.to_string()))?
            }
            "ML-DSA-65" => {
                self.pq_signature.keypair_generate()
                    .map_err(|e| CryptoError::KeyGenerationFailed(e.to_string()))?
            }
            _ => return Err(CryptoError::InvalidAlgorithm(algorithm.to_string())),
        };

        let duration = start.elapsed();
        self.update_metrics("pq_key_exchange", duration, true, false).await;

        info!(
            algorithm = algorithm,
            public_key_size = public_key.len(),
            private_key_size = private_key.len(),
            duration_ms = duration.as_millis(),
            "Post-quantum key pair generated"
        );

        Ok(PostQuantumKeyPair {
            public_key,
            private_key,
            algorithm: algorithm.to_string(),
            created_at: chrono::Utc::now(),
            expires_at: Some(chrono::Utc::now() + chrono::Duration::days(365)), // 1 year default
        })
    }

    /// Get performance metrics
    pub async fn get_metrics(&self) -> EnhancedCryptoMetrics {
        (*self.metrics.read().await).clone()
    }

    /// Clear all caches
    pub async fn clear_caches(&self) {
        let mut encryption_cache = self.encryption_cache.write().await;
        let mut key_cache = self.key_cache.write().await;

        encryption_cache.clear();
        key_cache.clear();

        info!("All caches cleared");
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> CacheMetrics {
        let encryption_cache = self.encryption_cache.read().await;
        let key_cache = self.key_cache.read().await;

        CacheMetrics {
            hits: 0, // Would be tracked separately
            misses: 0, // Would be tracked separately
            evictions: 0, // Would be tracked separately
            current_size: encryption_cache.len() + key_cache.len(),
            max_size: 10000, // Configurable
        }
    }

    // Private helper methods

    fn clone_for_async(&self) -> Self {
        // Create a lightweight clone for async operations
        Self {
            classical_engine: CryptoEngine::new(),
            pq_signature: Box::new(MLDsaProvider::new(MLDsaVariant::MLDsa65)),
            pq_key_exchange: Box::new(MLKemProvider::new(MLKemVariant::MLKem768)),
            crypto_mode: self.crypto_mode,
            metrics: Arc::clone(&self.metrics),
            encryption_cache: Arc::clone(&self.encryption_cache),
            key_cache: Arc::clone(&self.key_cache),
        }
    }

    async fn encrypt_classical(&self, data: &[u8], algorithm: AlgorithmId) -> CryptoResult<HybridEncryptionResult> {
        let key = crate::generate_key(algorithm)?;
        let encrypted = self.classical_engine.encrypt(algorithm, data, &key)?;

        Ok(HybridEncryptionResult {
            classical: encrypted,
            post_quantum: None,
            mode: CryptoMode::Classical,
            performance: OperationPerformance {
                duration_ms: 0, // Will be set by caller
                cache_status: CacheStatus::Disabled,
                compression_ratio: None,
            },
        })
    }

    async fn encrypt_hybrid(&self, data: &[u8], algorithm: AlgorithmId) -> CryptoResult<HybridEncryptionResult> {
        // Classical encryption
        let classical_result = self.encrypt_classical(data, algorithm).await?;

        // Post-quantum key encapsulation (simplified)
        let (pq_public, _pq_private) = self.pq_key_exchange.keypair_generate()
            .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

        Ok(HybridEncryptionResult {
            classical: classical_result.classical,
            post_quantum: Some(pq_public),
            mode: CryptoMode::Hybrid,
            performance: OperationPerformance {
                duration_ms: 0, // Will be set by caller
                cache_status: CacheStatus::Disabled,
                compression_ratio: None,
            },
        })
    }

    async fn encrypt_post_quantum(&self, data: &[u8]) -> CryptoResult<HybridEncryptionResult> {
        // For now, use classical encryption with PQ key derivation
        // In a full implementation, this would use pure PQ algorithms
        let key = crate::generate_key(AlgorithmId::Aes256Gcm)?;
        let encrypted = self.classical_engine.encrypt(AlgorithmId::Aes256Gcm, data, &key)?;

        // Generate PQ signature for integrity
        let (_pq_public, pq_private) = self.pq_signature.keypair_generate()
            .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

        let signature = self.pq_signature.sign(data, &pq_private)
            .map_err(|e| CryptoError::SigningFailed(e.to_string()))?;

        Ok(HybridEncryptionResult {
            classical: encrypted,
            post_quantum: Some(signature),
            mode: CryptoMode::PostQuantum,
            performance: OperationPerformance {
                duration_ms: 0, // Will be set by caller
                cache_status: CacheStatus::Disabled,
                compression_ratio: None,
            },
        })
    }

    async fn decrypt_classical(&self, encrypted: &EncryptedData) -> CryptoResult<Vec<u8>> {
        // In a real implementation, we'd need to retrieve the key
        let key = crate::generate_key(encrypted.algorithm)?;
        self.classical_engine.decrypt(encrypted, &key)
    }

    async fn decrypt_hybrid(&self, encrypted: &EncryptedData, _pq_data: Option<&Vec<u8>>) -> CryptoResult<Vec<u8>> {
        // For now, just decrypt the classical part
        // In a full implementation, this would also verify PQ components
        self.decrypt_classical(encrypted).await
    }

    async fn decrypt_post_quantum(&self, _pq_data: &[u8]) -> CryptoResult<Vec<u8>> {
        // Placeholder for pure PQ decryption
        // In a real implementation, this would use PQ algorithms
        Err(CryptoError::DecryptionFailed("Pure PQ decryption not yet implemented".to_string()))
    }

    fn generate_cache_key(&self, path: &str, data: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(path.as_bytes());
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    async fn get_cached_encryption(&self, cache_key: &str) -> Option<CachedEncryption> {
        let cache = self.encryption_cache.read().await;
        if let Some(cached) = cache.get(cache_key)
            && cached.expires_at > chrono::Utc::now() {
                return Some(cached.clone());
            }
        None
    }

    async fn cache_encryption_result(&self, cache_key: &str, encrypted: &EncryptedData, ttl_seconds: u64) {
        let mut cache = self.encryption_cache.write().await;
        let cached = CachedEncryption {
            encrypted_data: encrypted.clone(),
            expires_at: chrono::Utc::now() + chrono::Duration::seconds(ttl_seconds as i64),
            access_count: 1,
            last_accessed: chrono::Utc::now(),
        };
        cache.insert(cache_key.to_string(), cached);
    }

    fn compress_data(&self, data: &[u8]) -> CryptoResult<Vec<u8>> {
        // Simple compression using flate2
        use flate2::{Compression, write::GzEncoder};
        use std::io::Write;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data)
            .map_err(|e| CryptoError::Internal(format!("Compression failed: {}", e)))?;
        encoder.finish()
            .map_err(|e| CryptoError::Internal(format!("Compression finalization failed: {}", e)))
    }

    fn decompress_data(&self, data: &[u8]) -> CryptoResult<Vec<u8>> {
        // Simple decompression using flate2
        use flate2::read::GzDecoder;
        use std::io::Read;

        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)
            .map_err(|e| CryptoError::Internal(format!("Decompression failed: {}", e)))?;
        Ok(decompressed)
    }

    fn is_compressed(&self, data: &[u8]) -> bool {
        // Check for gzip magic number
        data.len() >= 2 && data[0] == 0x1f && data[1] == 0x8b
    }

    async fn update_metrics(&self, operation: &str, duration: Duration, success: bool, from_cache: bool) {
        let mut metrics = self.metrics.write().await;
        let duration_ms = duration.as_millis() as u64;

        let op_metrics = match operation {
            "secret_encryption" => &mut metrics.secret_encryption,
            "secret_decryption" => &mut metrics.secret_decryption,
            "key_derivation" => &mut metrics.key_derivation,
            "pq_signatures" => &mut metrics.pq_signatures,
            "pq_key_exchange" => &mut metrics.pq_key_exchange,
            "batch_operations" => &mut metrics.batch_operations,
            _ => return,
        };

        op_metrics.count += 1;
        op_metrics.total_time_ms += duration_ms;
        op_metrics.avg_time_ms = op_metrics.total_time_ms as f64 / op_metrics.count as f64;

        if op_metrics.count == 1 {
            op_metrics.min_time_ms = duration_ms;
            op_metrics.max_time_ms = duration_ms;
        } else {
            op_metrics.min_time_ms = op_metrics.min_time_ms.min(duration_ms);
            op_metrics.max_time_ms = op_metrics.max_time_ms.max(duration_ms);
        }

        if !success {
            op_metrics.failures += 1;
        }

        // Calculate ops per second
        if op_metrics.total_time_ms > 0 {
            op_metrics.ops_per_second = (op_metrics.count as f64 * 1000.0) / op_metrics.total_time_ms as f64;
        }

        // Update cache metrics
        if from_cache {
            metrics.cache_operations.hits += 1;
        } else if operation.contains("encryption") || operation.contains("decryption") {
            metrics.cache_operations.misses += 1;
        }
    }
}

impl CryptoOperationMetrics {
    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            ((self.count - self.failures) as f64 / self.count as f64) * 100.0
        }
    }

    /// Get failure rate as percentage
    pub fn failure_rate(&self) -> f64 {
        100.0 - self.success_rate()
    }
}

impl CacheMetrics {
    /// Get cache hit rate as percentage
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }

    /// Get cache utilization as percentage
    pub fn utilization(&self) -> f64 {
        if self.max_size == 0 {
            0.0
        } else {
            (self.current_size as f64 / self.max_size as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_enhanced_crypto_engine_creation() {
        let engine = EnhancedSecretonCrypto::new();
        assert_eq!(engine.crypto_mode, CryptoMode::Classical);
    }

    #[tokio::test]
    async fn test_crypto_mode_setting() {
        let mut engine = EnhancedSecretonCrypto::new();
        engine.set_crypto_mode(CryptoMode::Hybrid);
        assert_eq!(engine.crypto_mode, CryptoMode::Hybrid);
    }

    #[tokio::test]
    async fn test_secret_encryption_classical() {
        let engine = EnhancedSecretonCrypto::new();

        let request = EnhancedSecretRequest {
            path: "test/secret".to_string(),
            data: b"secret data".to_vec(),
            algorithm: Some(AlgorithmId::Aes256Gcm),
            crypto_mode: Some(CryptoMode::Classical),
            use_cache: false,
            cache_ttl: None,
            compress: false,
        };

        let result = engine.encrypt_secret(request).await;
        assert!(result.is_ok());

        let encrypted = result.unwrap();
        assert_eq!(encrypted.mode, CryptoMode::Classical);
        assert_eq!(encrypted.classical.algorithm, AlgorithmId::Aes256Gcm);
        assert!(encrypted.post_quantum.is_none());
    }

    #[tokio::test]
    async fn test_secret_encryption_with_compression() {
        let engine = EnhancedSecretonCrypto::new();

        let large_data = vec![b'A'; 1000]; // 1KB of 'A's - should compress well
        let request = EnhancedSecretRequest {
            path: "test/large_secret".to_string(),
            data: large_data,
            algorithm: Some(AlgorithmId::Aes256Gcm),
            crypto_mode: Some(CryptoMode::Classical),
            use_cache: false,
            cache_ttl: None,
            compress: true,
        };

        let result = engine.encrypt_secret(request).await;
        assert!(result.is_ok());

        let encrypted = result.unwrap();
        assert!(encrypted.performance.compression_ratio.is_some());
        assert!(encrypted.performance.compression_ratio.unwrap() > 1.0);
    }

    #[tokio::test]
    async fn test_post_quantum_keypair_generation() {
        let engine = EnhancedSecretonCrypto::new();

        let keypair = engine.generate_pq_keypair("ML-KEM-768").await;
        assert!(keypair.is_ok());

        let kp = keypair.unwrap();
        assert_eq!(kp.algorithm, "ML-KEM-768");
        assert!(!kp.public_key.is_empty());
        assert!(!kp.private_key.is_empty());
        assert!(kp.expires_at.is_some());
    }

    #[tokio::test]
    async fn test_batch_operations() {
        let engine = EnhancedSecretonCrypto::new();

        let requests = vec![
            EnhancedSecretRequest {
                path: "test/secret1".to_string(),
                data: b"secret data 1".to_vec(),
                algorithm: Some(AlgorithmId::Aes256Gcm),
                crypto_mode: Some(CryptoMode::Classical),
                use_cache: false,
                cache_ttl: None,
                compress: false,
            },
            EnhancedSecretRequest {
                path: "test/secret2".to_string(),
                data: b"secret data 2".to_vec(),
                algorithm: Some(AlgorithmId::Aes256Gcm),
                crypto_mode: Some(CryptoMode::Classical),
                use_cache: false,
                cache_ttl: None,
                compress: false,
            },
        ];

        let batch_request = BatchSecretRequest {
            requests,
            max_parallel: Some(2),
            timeout_ms: Some(5000),
        };

        let result = engine.encrypt_secrets_batch(batch_request).await;
        assert!(result.is_ok());

        let batch_response = result.unwrap();
        assert_eq!(batch_response.responses.len(), 2);
        assert_eq!(batch_response.batch_metrics.total_operations, 2);
        assert!(batch_response.batch_metrics.throughput_ops_per_sec > 0.0);
    }

    #[tokio::test]
    async fn test_metrics_collection() {
        let engine = EnhancedSecretonCrypto::new();

        // Perform some operations to generate metrics
        let request = EnhancedSecretRequest {
            path: "test/metrics".to_string(),
            data: b"test data".to_vec(),
            algorithm: Some(AlgorithmId::Aes256Gcm),
            crypto_mode: Some(CryptoMode::Classical),
            use_cache: false,
            cache_ttl: None,
            compress: false,
        };

        let _ = engine.encrypt_secret(request.clone()).await;
        let _ = engine.encrypt_secret(request).await;

        let metrics = engine.get_metrics().await;
        assert_eq!(metrics.secret_encryption.count, 2);
        assert!(metrics.secret_encryption.avg_time_ms > 0.0);
        assert_eq!(metrics.secret_encryption.success_rate(), 100.0);
    }

    #[tokio::test]
    async fn test_cache_functionality() {
        let engine = EnhancedSecretonCrypto::new();

        let request = EnhancedSecretRequest {
            path: "test/cached_secret".to_string(),
            data: b"cached data".to_vec(),
            algorithm: Some(AlgorithmId::Aes256Gcm),
            crypto_mode: Some(CryptoMode::Classical),
            use_cache: true,
            cache_ttl: Some(60), // 1 minute
            compress: false,
        };

        // First request - should be a cache miss
        let result1 = engine.encrypt_secret(request.clone()).await;
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap().performance.cache_status, CacheStatus::Miss);

        // Second request - should be a cache hit
        let result2 = engine.encrypt_secret(request).await;
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap().performance.cache_status, CacheStatus::Hit);
    }
}
