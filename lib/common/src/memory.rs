use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, OnceCell};
use tracing::debug;
use zeroize::Zeroize;

use crate::error::{CommonError, Result};

/// Secure memory container for secrets that automatically zeroizes on drop
#[derive(Debug, Clone)]
pub struct SecureSecretMemory<T: Zeroize> {
    data: T,
    created_at: Instant,
    access_count: u64,
    sensitivity_level: SensitivityLevel,
}

impl<T: Zeroize> Drop for SecureSecretMemory<T> {
    fn drop(&mut self) {
        self.data.zeroize();
    }
}

/// Sensitivity level for memory management
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SensitivityLevel {
    /// Low sensitivity - standard memory management
    Low,
    /// Medium sensitivity - enhanced security measures
    #[default]
    Medium,
    /// High sensitivity - strict memory management
    High,
    /// Critical sensitivity - maximum security, immediate cleanup
    Critical,
}

impl<T: Zeroize> SecureSecretMemory<T> {
    /// Create a new secure memory container for secrets
    pub fn new(data: T, sensitivity_level: SensitivityLevel) -> Self {
        Self {
            data,
            created_at: Instant::now(),
            access_count: 0,
            sensitivity_level,
        }
    }

    /// Access the contained data (increments access count)
    pub fn access(&mut self) -> &T {
        self.access_count += 1;
        &self.data
    }

    /// Get read-only access to the data
    pub fn read(&self) -> &T {
        &self.data
    }

    /// Get the age of this memory container
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// Get the access count
    pub fn access_count(&self) -> u64 {
        self.access_count
    }

    /// Get the sensitivity level
    pub fn sensitivity_level(&self) -> SensitivityLevel {
        self.sensitivity_level
    }

    /// Check if this memory should be cleaned up based on sensitivity and age
    pub fn should_cleanup(&self, max_age: Duration) -> bool {
        let age = self.age();
        match self.sensitivity_level {
            SensitivityLevel::Critical => age > Duration::from_secs(60), // 1 minute
            SensitivityLevel::High => age > Duration::from_secs(300),    // 5 minutes
            SensitivityLevel::Medium => age > max_age / 2,
            SensitivityLevel::Low => age > max_age,
        }
    }

    /// Manually zeroize the data
    pub fn zeroize(&mut self) {
        self.data.zeroize();
    }
}

/// Secure string for secrets that zeroizes on drop
#[derive(Debug, Clone)]
pub struct SecureSecretString {
    data: String,
    sensitivity_level: SensitivityLevel,
}

impl Drop for SecureSecretString {
    fn drop(&mut self) {
        self.data.zeroize();
    }
}

impl SecureSecretString {
    /// Create a new secure secret string
    pub fn new(data: String, sensitivity_level: SensitivityLevel) -> Self {
        Self {
            data,
            sensitivity_level,
        }
    }

    /// Create from a string slice
    pub fn from_str(data: &str, sensitivity_level: SensitivityLevel) -> Self {
        Self {
            data: data.to_string(),
            sensitivity_level,
        }
    }

    /// Get the string data
    pub fn as_str(&self) -> &str {
        &self.data
    }

    /// Get the string length
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the string is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get the sensitivity level
    pub fn sensitivity_level(&self) -> SensitivityLevel {
        self.sensitivity_level
    }
}

impl Zeroize for SecureSecretString {
    fn zeroize(&mut self) {
        self.data.zeroize();
    }
}

impl From<&str> for SecureSecretString {
    fn from(s: &str) -> Self {
        Self::from_str(s, SensitivityLevel::Medium)
    }
}

impl From<String> for SecureSecretString {
    fn from(s: String) -> Self {
        Self::new(s, SensitivityLevel::Medium)
    }
}

/// Secure byte array for secret data that zeroizes on drop
#[derive(Debug, Clone)]
pub struct SecureSecretBytes {
    data: Vec<u8>,
    sensitivity_level: SensitivityLevel,
}

impl Drop for SecureSecretBytes {
    fn drop(&mut self) {
        self.data.zeroize();
    }
}

impl SecureSecretBytes {
    /// Create a new secure secret byte array
    pub fn new(data: Vec<u8>, sensitivity_level: SensitivityLevel) -> Self {
        Self {
            data,
            sensitivity_level,
        }
    }

    /// Create from a byte slice
    pub fn from_slice(data: &[u8], sensitivity_level: SensitivityLevel) -> Self {
        Self {
            data: data.to_vec(),
            sensitivity_level,
        }
    }

    /// Get the byte data
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Get the length
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get the sensitivity level
    pub fn sensitivity_level(&self) -> SensitivityLevel {
        self.sensitivity_level
    }
}

impl Zeroize for SecureSecretBytes {
    fn zeroize(&mut self) {
        self.data.zeroize();
    }
}

impl From<Vec<u8>> for SecureSecretBytes {
    fn from(v: Vec<u8>) -> Self {
        Self::new(v, SensitivityLevel::Medium)
    }
}

impl From<&[u8]> for SecureSecretBytes {
    fn from(v: &[u8]) -> Self {
        Self::from_slice(v, SensitivityLevel::Medium)
    }
}

/// Lazy-loaded cryptographic context for secret operations
pub struct LazySecretCryptoContext<T> {
    context: OnceCell<T>,
    initializer: Arc<dyn Fn() -> Result<T> + Send + Sync>,
    sensitivity_level: SensitivityLevel,
}

impl<T> LazySecretCryptoContext<T> {
    /// Create a new lazy secret crypto context
    pub fn new<F>(initializer: F, sensitivity_level: SensitivityLevel) -> Self
    where
        F: Fn() -> Result<T> + Send + Sync + 'static,
    {
        Self {
            context: OnceCell::new(),
            initializer: Arc::new(initializer),
            sensitivity_level,
        }
    }

    /// Get or initialize the context
    pub async fn get(&self) -> Result<&T> {
        if let Some(context) = self.context.get() {
            return Ok(context);
        }

        let context = (self.initializer)()?;
        self.context.set(context).map_err(|_| {
            CommonError::Internal("Failed to initialize secret crypto context".to_string())
        })?;

        Ok(self.context.get().unwrap())
    }

    /// Check if the context is initialized
    pub fn is_initialized(&self) -> bool {
        self.context.get().is_some()
    }

    /// Get the sensitivity level
    pub fn sensitivity_level(&self) -> SensitivityLevel {
        self.sensitivity_level
    }
}

/// Memory pool for reusing secret-related allocations
pub struct SecretMemoryPool<T> {
    pool: Arc<Mutex<Vec<T>>>,
    max_size: usize,
    factory: Arc<dyn Fn() -> T + Send + Sync>,
    sensitivity_level: SensitivityLevel,
}

impl<T> SecretMemoryPool<T>
where
    T: Send + Zeroize + 'static,
{
    /// Create a new secret memory pool
    pub fn new<F>(max_size: usize, factory: F, sensitivity_level: SensitivityLevel) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self {
            pool: Arc::new(Mutex::new(Vec::with_capacity(max_size))),
            max_size,
            factory: Arc::new(factory),
            sensitivity_level,
        }
    }

    /// Get an object from the pool or create a new one
    pub async fn get(&self) -> T {
        let mut pool = self.pool.lock().await;

        if let Some(item) = pool.pop() {
            debug!("Reused object from secret memory pool");
            item
        } else {
            debug!("Created new object for secret memory pool");
            (self.factory)()
        }
    }

    /// Return an object to the pool (with zeroization for sensitive data)
    pub async fn return_object(&self, mut item: T) {
        // Always zeroize before returning to pool for security
        item.zeroize();

        let mut pool = self.pool.lock().await;

        if pool.len() < self.max_size {
            pool.push(item);
            debug!("Returned zeroized object to secret memory pool");
        } else {
            debug!("Secret memory pool full, dropping zeroized object");
            // Object will be dropped here (already zeroized)
        }
    }

    /// Get current pool size
    pub async fn size(&self) -> usize {
        let pool = self.pool.lock().await;
        pool.len()
    }

    /// Clear the pool (with zeroization)
    pub async fn clear(&self) {
        let mut pool = self.pool.lock().await;

        // Zeroize all items before clearing
        for mut item in pool.drain(..) {
            item.zeroize();
        }

        debug!("Cleared and zeroized secret memory pool");
    }

    /// Get the sensitivity level
    pub fn sensitivity_level(&self) -> SensitivityLevel {
        self.sensitivity_level
    }
}

/// Memory usage tracker for secret operations
pub struct SecretMemoryTracker {
    allocations: Arc<Mutex<Vec<SecretAllocationInfo>>>,
    total_allocated: Arc<Mutex<usize>>,
    peak_usage: Arc<Mutex<usize>>,
    sensitive_allocations: Arc<Mutex<usize>>,
}

#[derive(Debug, Clone)]
pub struct SecretAllocationInfo {
    pub size: usize,
    pub timestamp: Instant,
    pub tag: String,
    pub sensitivity_level: SensitivityLevel,
}

impl SecretMemoryTracker {
    /// Create a new secret memory tracker
    pub fn new() -> Self {
        Self {
            allocations: Arc::new(Mutex::new(Vec::new())),
            total_allocated: Arc::new(Mutex::new(0)),
            peak_usage: Arc::new(Mutex::new(0)),
            sensitive_allocations: Arc::new(Mutex::new(0)),
        }
    }

    /// Track a secret allocation
    pub async fn track_allocation(
        &self,
        size: usize,
        tag: String,
        sensitivity_level: SensitivityLevel,
    ) {
        let mut allocations = self.allocations.lock().await;
        let mut total = self.total_allocated.lock().await;
        let mut peak = self.peak_usage.lock().await;
        let mut sensitive = self.sensitive_allocations.lock().await;

        allocations.push(SecretAllocationInfo {
            size,
            timestamp: Instant::now(),
            tag,
            sensitivity_level,
        });

        *total += size;
        if *total > *peak {
            *peak = *total;
        }

        if matches!(
            sensitivity_level,
            SensitivityLevel::High | SensitivityLevel::Critical
        ) {
            *sensitive += size;
        }

        debug!(
            "Tracked secret allocation: {} bytes ({:?}), total: {} bytes, sensitive: {} bytes",
            size, sensitivity_level, *total, *sensitive
        );
    }

    /// Track a secret deallocation
    pub async fn track_deallocation(&self, size: usize, sensitivity_level: SensitivityLevel) {
        let mut total = self.total_allocated.lock().await;
        let mut sensitive = self.sensitive_allocations.lock().await;

        *total = total.saturating_sub(size);

        if matches!(
            sensitivity_level,
            SensitivityLevel::High | SensitivityLevel::Critical
        ) {
            *sensitive = sensitive.saturating_sub(size);
        }

        debug!(
            "Tracked secret deallocation: {} bytes ({:?}), total: {} bytes, sensitive: {} bytes",
            size, sensitivity_level, *total, *sensitive
        );
    }

    /// Get current memory usage
    pub async fn current_usage(&self) -> usize {
        let total = self.total_allocated.lock().await;
        *total
    }

    /// Get peak memory usage
    pub async fn peak_usage(&self) -> usize {
        let peak = self.peak_usage.lock().await;
        *peak
    }

    /// Get sensitive memory usage
    pub async fn sensitive_usage(&self) -> usize {
        let sensitive = self.sensitive_allocations.lock().await;
        *sensitive
    }

    /// Get memory usage statistics
    pub async fn get_stats(&self) -> SecretMemoryStats {
        let allocations = self.allocations.lock().await;
        let total = self.total_allocated.lock().await;
        let peak = self.peak_usage.lock().await;
        let sensitive = self.sensitive_allocations.lock().await;

        // Calculate sensitivity distribution
        let mut sensitivity_counts = [0; 4]; // Low, Medium, High, Critical
        let mut sensitivity_sizes = [0; 4];

        for alloc in allocations.iter() {
            let index = match alloc.sensitivity_level {
                SensitivityLevel::Low => 0,
                SensitivityLevel::Medium => 1,
                SensitivityLevel::High => 2,
                SensitivityLevel::Critical => 3,
            };
            sensitivity_counts[index] += 1;
            sensitivity_sizes[index] += alloc.size;
        }

        SecretMemoryStats {
            current_usage: *total,
            peak_usage: *peak,
            sensitive_usage: *sensitive,
            allocation_count: allocations.len(),
            sensitivity_distribution: sensitivity_counts,
            sensitivity_sizes,
            allocations: allocations.clone(),
        }
    }

    /// Clean up old allocation records based on sensitivity
    pub async fn cleanup_old_records(&self, max_age: Duration) {
        let mut allocations = self.allocations.lock().await;
        let cutoff = Instant::now() - max_age;

        let initial_count = allocations.len();
        allocations.retain(|alloc| {
            // Keep records longer for sensitive allocations
            let keep_until = match alloc.sensitivity_level {
                SensitivityLevel::Critical => alloc.timestamp + max_age * 2,
                SensitivityLevel::High => alloc.timestamp + max_age * 3 / 2,
                _ => alloc.timestamp + max_age,
            };
            keep_until > cutoff
        });

        let removed_count = initial_count - allocations.len();
        if removed_count > 0 {
            debug!(
                "Cleaned up {} old secret allocation records, {} remaining",
                removed_count,
                allocations.len()
            );
        }
    }
}

impl Default for SecretMemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory usage statistics for secret operations
#[derive(Debug, Clone)]
pub struct SecretMemoryStats {
    pub current_usage: usize,
    pub peak_usage: usize,
    pub sensitive_usage: usize,
    pub allocation_count: usize,
    pub sensitivity_distribution: [usize; 4], // [Low, Medium, High, Critical]
    pub sensitivity_sizes: [usize; 4],        // Memory usage by sensitivity level
    pub allocations: Vec<SecretAllocationInfo>,
}

/// Memory optimization utilities for secret operations
pub struct SecretMemoryOptimizer {
    tracker: SecretMemoryTracker,
    string_pool: SecretMemoryPool<String>,
    vec_pool: SecretMemoryPool<Vec<u8>>,
}

impl SecretMemoryOptimizer {
    /// Create a new secret memory optimizer
    pub fn new() -> Self {
        Self {
            tracker: SecretMemoryTracker::new(),
            string_pool: SecretMemoryPool::new(
                50, // Smaller pool for security
                || String::with_capacity(256),
                SensitivityLevel::Medium,
            ),
            vec_pool: SecretMemoryPool::new(
                50, // Smaller pool for security
                || Vec::with_capacity(1024),
                SensitivityLevel::Medium,
            ),
        }
    }

    /// Get a string from the pool
    pub async fn get_string(&self) -> String {
        self.string_pool.get().await
    }

    /// Return a string to the pool (will be zeroized)
    pub async fn return_string(&self, s: String) {
        self.string_pool.return_object(s).await;
    }

    /// Get a byte vector from the pool
    pub async fn get_vec(&self) -> Vec<u8> {
        self.vec_pool.get().await
    }

    /// Return a byte vector to the pool (will be zeroized)
    pub async fn return_vec(&self, v: Vec<u8>) {
        self.vec_pool.return_object(v).await
    }

    /// Get memory tracker
    pub fn tracker(&self) -> &SecretMemoryTracker {
        &self.tracker
    }

    /// Perform security-focused cleanup
    pub async fn security_cleanup(&self, max_age: Duration) {
        // Clean up old records
        self.tracker.cleanup_old_records(max_age).await;

        // Clear pools for security (all items will be zeroized)
        self.string_pool.clear().await;
        self.vec_pool.clear().await;

        debug!("Performed security-focused memory cleanup");
    }

    /// Get memory usage statistics
    pub async fn get_stats(&self) -> SecretMemoryOptimizerStats {
        let memory_stats = self.tracker.get_stats().await;
        let string_pool_size = self.string_pool.size().await;
        let vec_pool_size = self.vec_pool.size().await;

        SecretMemoryOptimizerStats {
            memory_stats,
            string_pool_size,
            vec_pool_size,
        }
    }
}

impl Default for SecretMemoryOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Combined memory optimizer statistics for secret operations
#[derive(Debug, Clone)]
pub struct SecretMemoryOptimizerStats {
    pub memory_stats: SecretMemoryStats,
    pub string_pool_size: usize,
    pub vec_pool_size: usize,
}
