use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, OnceCell};
use tracing::debug;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::error::{AuthencError, Result};

/// Secure memory container that automatically zeroizes on drop
#[derive(Debug, Clone)]
pub struct SecureMemory<T: Zeroize> {
    data: T,
    created_at: Instant,
    access_count: u64,
}

impl<T: Zeroize> Drop for SecureMemory<T> {
    fn drop(&mut self) {
        self.data.zeroize();
    }
}

impl<T: Zeroize> SecureMemory<T> {
    /// Create a new secure memory container
    pub fn new(data: T) -> Self {
        Self {
            data,
            created_at: Instant::now(),
            access_count: 0,
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

    /// Manually zeroize the data
    pub fn zeroize(&mut self) {
        self.data.zeroize();
    }
}

/// Secure string that zeroizes on drop
#[derive(Debug, Clone, ZeroizeOnDrop)]
pub struct SecureString {
    data: String,
}

impl SecureString {
    /// Create a new secure string
    pub fn new(data: String) -> Self {
        Self { data }
    }

    /// Create from a string slice
    pub fn from_str(data: &str) -> Self {
        Self {
            data: data.to_string(),
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
}

impl Zeroize for SecureString {
    fn zeroize(&mut self) {
        self.data.zeroize();
    }
}

impl From<String> for SecureString {
    fn from(data: String) -> Self {
        Self::new(data)
    }
}

impl From<&str> for SecureString {
    fn from(data: &str) -> Self {
        Self::from_str(data)
    }
}

/// Secure byte array that zeroizes on drop
#[derive(Debug, Clone, ZeroizeOnDrop)]
pub struct SecureBytes {
    data: Vec<u8>,
}

impl SecureBytes {
    /// Create a new secure byte array
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Create from a byte slice
    pub fn from_slice(data: &[u8]) -> Self {
        Self {
            data: data.to_vec(),
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
}

impl Zeroize for SecureBytes {
    fn zeroize(&mut self) {
        self.data.zeroize();
    }
}

impl From<Vec<u8>> for SecureBytes {
    fn from(data: Vec<u8>) -> Self {
        Self::new(data)
    }
}

impl From<&[u8]> for SecureBytes {
    fn from(data: &[u8]) -> Self {
        Self::from_slice(data)
    }
}

/// Lazy-loaded cryptographic context
pub struct LazyCryptoContext<T> {
    context: OnceCell<T>,
    initializer: Arc<dyn Fn() -> Result<T> + Send + Sync>,
}

impl<T> LazyCryptoContext<T> {
    /// Create a new lazy crypto context
    pub fn new<F>(initializer: F) -> Self
    where
        F: Fn() -> Result<T> + Send + Sync + 'static,
    {
        Self {
            context: OnceCell::new(),
            initializer: Arc::new(initializer),
        }
    }

    /// Get or initialize the context
    pub async fn get(&self) -> Result<&T> {
        if let Some(context) = self.context.get() {
            return Ok(context);
        }

        let context = (self.initializer)()?;
        self.context
            .set(context)
            .map_err(|_| AuthencError::internal("Failed to initialize crypto context"))?;

        Ok(self.context.get().unwrap())
    }

    /// Check if the context is initialized
    pub fn is_initialized(&self) -> bool {
        self.context.get().is_some()
    }
}

/// Memory pool for reusing allocations
pub struct MemoryPool<T> {
    pool: Arc<Mutex<Vec<T>>>,
    max_size: usize,
    factory: Arc<dyn Fn() -> T + Send + Sync>,
}

impl<T> MemoryPool<T>
where
    T: Send + 'static,
{
    /// Create a new memory pool
    pub fn new<F>(max_size: usize, factory: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self {
            pool: Arc::new(Mutex::new(Vec::with_capacity(max_size))),
            max_size,
            factory: Arc::new(factory),
        }
    }

    /// Get an object from the pool or create a new one
    pub async fn get(&self) -> T {
        let mut pool = self.pool.lock().await;

        if let Some(item) = pool.pop() {
            debug!("Reused object from memory pool");
            item
        } else {
            debug!("Created new object for memory pool");
            (self.factory)()
        }
    }

    /// Return an object to the pool
    pub async fn return_object(&self, item: T) {
        let mut pool = self.pool.lock().await;

        if pool.len() < self.max_size {
            // Reset the object if it implements a reset method
            // This would need to be implemented per type
            pool.push(item);
            debug!("Returned object to memory pool");
        } else {
            debug!("Memory pool full, dropping object");
            // Object will be dropped here
        }
    }

    /// Get current pool size
    pub async fn size(&self) -> usize {
        let pool = self.pool.lock().await;
        pool.len()
    }

    /// Clear the pool
    pub async fn clear(&self) {
        let mut pool = self.pool.lock().await;
        pool.clear();
        debug!("Cleared memory pool");
    }
}

/// Memory usage tracker
pub struct MemoryTracker {
    allocations: Arc<Mutex<Vec<AllocationInfo>>>,
    total_allocated: Arc<Mutex<usize>>,
    peak_usage: Arc<Mutex<usize>>,
}

#[derive(Debug, Clone)]
struct AllocationInfo {
    size: usize,
    timestamp: Instant,
    tag: String,
}

impl MemoryTracker {
    /// Create a new memory tracker
    pub fn new() -> Self {
        Self {
            allocations: Arc::new(Mutex::new(Vec::new())),
            total_allocated: Arc::new(Mutex::new(0)),
            peak_usage: Arc::new(Mutex::new(0)),
        }
    }

    /// Track an allocation
    pub async fn track_allocation(&self, size: usize, tag: String) {
        let mut allocations = self.allocations.lock().await;
        let mut total = self.total_allocated.lock().await;
        let mut peak = self.peak_usage.lock().await;

        allocations.push(AllocationInfo {
            size,
            timestamp: Instant::now(),
            tag,
        });

        *total += size;
        if *total > *peak {
            *peak = *total;
        }

        debug!(
            "Tracked allocation: {} bytes, total: {} bytes",
            size, *total
        );
    }

    /// Track a deallocation
    pub async fn track_deallocation(&self, size: usize) {
        let mut total = self.total_allocated.lock().await;
        *total = total.saturating_sub(size);
        debug!(
            "Tracked deallocation: {} bytes, total: {} bytes",
            size, *total
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

    /// Get memory usage statistics
    pub async fn get_stats(&self) -> MemoryStats {
        let allocations = self.allocations.lock().await;
        let total = self.total_allocated.lock().await;
        let peak = self.peak_usage.lock().await;

        MemoryStats {
            current_usage: *total,
            peak_usage: *peak,
            allocation_count: allocations.len(),
            allocations: allocations.clone(),
        }
    }

    /// Clean up old allocation records
    pub async fn cleanup_old_records(&self, max_age: Duration) {
        let mut allocations = self.allocations.lock().await;
        let cutoff = Instant::now() - max_age;

        allocations.retain(|alloc| alloc.timestamp > cutoff);
        debug!(
            "Cleaned up old allocation records, {} remaining",
            allocations.len()
        );
    }
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory usage statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    /// Current memory usage in bytes
    pub current_usage: usize,
    /// Peak memory usage in bytes
    pub peak_usage: usize,
    /// Total number of allocations
    pub allocation_count: usize,
    /// List of allocation information
    pub allocations: Vec<AllocationInfo>,
}

/// Memory optimization utilities
pub struct MemoryOptimizer {
    tracker: MemoryTracker,
    /// String memory pool
    string_pool: MemoryPool<String>,
    /// Vector memory pool
    vec_pool: MemoryPool<Vec<u8>>,
}

impl MemoryOptimizer {
    /// Create a new memory optimizer
    /// Memory usage tracker
    pub fn new() -> Self {
        Self {
            tracker: MemoryTracker::new(),
            string_pool: MemoryPool::new(100, || String::with_capacity(256)),
            vec_pool: MemoryPool::new(100, || Vec::with_capacity(1024)),
        }
    }

    /// Get a string from the pool
    pub async fn get_string(&self) -> String {
        self.string_pool.get().await
    }

    /// Return a string to the pool
    pub async fn return_string(&self, mut s: String) {
        s.clear();
        self.string_pool.return_object(s).await;
    }

    /// Get a byte vector from the pool
    pub async fn get_vec(&self) -> Vec<u8> {
        self.vec_pool.get().await
    }

    /// Return a byte vector to the pool
    pub async fn return_vec(&self, mut v: Vec<u8>) {
        v.clear();
        self.vec_pool.return_object(v).await;
    }

    /// Get memory tracker
    pub fn tracker(&self) -> &MemoryTracker {
        &self.tracker
    }

    /// Get memory usage statistics
    pub async fn get_stats(&self) -> MemoryOptimizerStats {
        let memory_stats = self.tracker.get_stats().await;
        let string_pool_size = self.string_pool.size().await;
        let vec_pool_size = self.vec_pool.size().await;

        MemoryOptimizerStats {
            memory_stats,
            string_pool_size,
            vec_pool_size,
        }
    }
}

impl Default for MemoryOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Combined memory optimizer statistics
#[derive(Debug, Clone)]
pub struct MemoryOptimizerStats {
    /// Memory usage statistics
    pub memory_stats: MemoryStats,
    /// Size of string memory pool
    pub string_pool_size: usize,
    /// Size of vector memory pool
    pub vec_pool_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_memory() {
        let mut secure_data = SecureMemory::new("sensitive_data".to_string());
        assert_eq!(secure_data.access(), "sensitive_data");
        assert_eq!(secure_data.access_count(), 1);

        secure_data.zeroize();
        // After zeroization, the data should be cleared
    }

    #[test]
    fn test_secure_string() {
        let secure_str = SecureString::from("password123");
        assert_eq!(secure_str.as_str(), "password123");
        assert_eq!(secure_str.len(), 11);
        assert!(!secure_str.is_empty());
    }

    #[test]
    fn test_secure_bytes() {
        let data = vec![1, 2, 3, 4, 5];
        let secure_bytes = SecureBytes::from(data.clone());
        assert_eq!(secure_bytes.as_bytes(), &data);
        assert_eq!(secure_bytes.len(), 5);
        assert!(!secure_bytes.is_empty());
    }

    #[tokio::test]
    async fn test_lazy_crypto_context() {
        let context = LazyCryptoContext::new(|| Ok("initialized".to_string()));

        assert!(!context.is_initialized());

        let value = context.get().await.unwrap();
        assert_eq!(value, "initialized");
        assert!(context.is_initialized());
    }

    #[tokio::test]
    async fn test_memory_pool() {
        let pool = MemoryPool::new(5, || String::new());

        let item1 = pool.get().await;
        let item2 = pool.get().await;

        pool.return_object(item1).await;
        assert_eq!(pool.size().await, 1);

        pool.return_object(item2).await;
        assert_eq!(pool.size().await, 2);
    }

    #[tokio::test]
    async fn test_memory_tracker() {
        let tracker = MemoryTracker::new();

        tracker
            .track_allocation(1024, "test_allocation".to_string())
            .await;
        assert_eq!(tracker.current_usage().await, 1024);
        assert_eq!(tracker.peak_usage().await, 1024);

        tracker.track_deallocation(512).await;
        assert_eq!(tracker.current_usage().await, 512);
        assert_eq!(tracker.peak_usage().await, 1024);
    }

    #[tokio::test]
    async fn test_memory_optimizer() {
        let optimizer = MemoryOptimizer::new();

        let string = optimizer.get_string().await;
        optimizer.return_string(string).await;

        let vec = optimizer.get_vec().await;
        optimizer.return_vec(vec).await;

        let stats = optimizer.get_stats().await;
        assert_eq!(stats.string_pool_size, 1);
        assert_eq!(stats.vec_pool_size, 1);
    }
}
