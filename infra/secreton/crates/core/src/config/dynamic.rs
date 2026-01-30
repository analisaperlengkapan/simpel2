use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::watch;
use tracing::{debug, info, warn};

use crate::error::{CoreError, Result};

/// Performance metrics for load-based configuration adjustment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadMetrics {
    /// CPU usage percentage (0.0 to 1.0)
    pub cpu_usage: f64,
    /// Memory usage percentage (0.0 to 1.0)
    pub memory_usage: f64,
    /// Active connections count
    pub active_connections: u32,
    /// Secret operations per second
    pub operations_per_second: f64,
    /// Average operation time in milliseconds
    pub avg_operation_time_ms: f64,
    /// Error rate percentage (0.0 to 1.0)
    pub error_rate: f64,
    /// Storage I/O operations per second
    pub storage_iops: f64,
}

impl Default for LoadMetrics {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            active_connections: 0,
            operations_per_second: 0.0,
            avg_operation_time_ms: 0.0,
            error_rate: 0.0,
            storage_iops: 0.0,
        }
    }
}

/// Security threat levels for adaptive security posture
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ThreatLevel {
    /// Normal operations - standard security measures
    #[default]
    Low,
    /// Elevated threat - enhanced monitoring and validation
    Medium,
    /// High threat - strict security measures and rate limiting
    High,
    /// Critical threat - maximum security, minimal functionality
    Critical,
}

/// Cryptographic modes for post-quantum transition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CryptoMode {
    /// Classical cryptography (AES-256-GCM, Ed25519)
    #[default]
    Classical,
    /// Hybrid classical + post-quantum
    Hybrid,
    /// Pure post-quantum cryptography (ML-KEM, ML-DSA)
    PostQuantum,
}

/// Performance profiles for different operational modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PerformanceProfile {
    /// Optimized for low latency secret retrieval
    LowLatency,
    /// Balanced performance and security
    #[default]
    Balanced,
    /// Optimized for high throughput operations
    HighThroughput,
    /// Maximum security, performance secondary
    MaxSecurity,
}

/// Cache configuration with adaptive TTL for secrets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum cache size in entries
    pub max_size: usize,
    /// Base TTL in seconds for cached secrets
    pub base_ttl_seconds: u64,
    /// Whether to use adaptive TTL based on load
    pub adaptive_ttl: bool,
    /// Minimum TTL in seconds (for adaptive mode)
    pub min_ttl_seconds: u64,
    /// Maximum TTL in seconds (for adaptive mode)
    pub max_ttl_seconds: u64,
    /// Cache hit ratio threshold for size adjustment
    pub hit_ratio_threshold: f64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size: 50000,       // Larger cache for secrets
            base_ttl_seconds: 600, // 10 minutes for secrets
            adaptive_ttl: true,
            min_ttl_seconds: 120,  // 2 minutes
            max_ttl_seconds: 7200, // 2 hours
            hit_ratio_threshold: 0.8,
        }
    }
}

/// Security configuration that adapts to threat levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Rate limit operations per minute
    pub rate_limit_opm: u32,
    /// Token validation cache TTL in seconds
    pub token_cache_ttl_seconds: u64,
    /// Maximum concurrent operations per user
    pub max_concurrent_ops: u32,
    /// Audit log retention in days
    pub audit_retention_days: u32,
    /// Whether to require additional validation for sensitive operations
    pub require_enhanced_validation: bool,
    /// Secret access timeout in seconds
    pub secret_access_timeout_seconds: u64,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            rate_limit_opm: 1000,
            token_cache_ttl_seconds: 300,
            max_concurrent_ops: 10,
            audit_retention_days: 90,
            require_enhanced_validation: false,
            secret_access_timeout_seconds: 30,
        }
    }
}

/// Storage configuration that adapts to load
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Connection pool size
    pub connection_pool_size: u32,
    /// Query timeout in seconds
    pub query_timeout_seconds: u64,
    /// Batch operation size
    pub batch_size: u32,
    /// Whether to use write-through caching
    pub write_through_cache: bool,
    /// Compression threshold in bytes
    pub compression_threshold: usize,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            connection_pool_size: 20,
            query_timeout_seconds: 30,
            batch_size: 100,
            write_through_cache: true,
            compression_threshold: 1024, // 1KB
        }
    }
}

/// Dynamic configuration that adapts to runtime conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicConfig {
    /// Current cryptographic mode
    pub crypto_mode: CryptoMode,
    /// Cache configuration
    pub cache: CacheConfig,
    /// Current security threat level
    pub threat_level: ThreatLevel,
    /// Performance profile
    pub performance_profile: PerformanceProfile,
    /// Adaptive security configuration
    pub security: SecurityConfig,
    /// Storage configuration
    pub storage: StorageConfig,
    /// Last update timestamp
    #[serde(skip)]
    pub last_updated: Option<Instant>,
}

impl Default for DynamicConfig {
    fn default() -> Self {
        Self {
            crypto_mode: CryptoMode::default(),
            cache: CacheConfig::default(),
            threat_level: ThreatLevel::default(),
            performance_profile: PerformanceProfile::default(),
            security: SecurityConfig::default(),
            storage: StorageConfig::default(),
            last_updated: Some(Instant::now()),
        }
    }
}

impl DynamicConfig {
    /// Create a new dynamic configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Adapt configuration based on current system load
    pub fn adapt_to_load(&mut self, load_metrics: &LoadMetrics) {
        let old_profile = self.performance_profile;

        // Determine performance profile based on load
        self.performance_profile =
            if load_metrics.cpu_usage > 0.8 || load_metrics.memory_usage > 0.8 {
                PerformanceProfile::MaxSecurity // Reduce load by increasing security checks
            } else if load_metrics.operations_per_second > 500.0 {
                PerformanceProfile::HighThroughput
            } else if load_metrics.avg_operation_time_ms < 10.0 {
                PerformanceProfile::LowLatency
            } else {
                PerformanceProfile::Balanced
            };

        // Adjust cache configuration based on memory usage and hit ratio
        if load_metrics.memory_usage > 0.7 {
            self.cache.max_size = (self.cache.max_size as f64 * 0.8) as usize;
            self.cache.base_ttl_seconds = self.cache.base_ttl_seconds.saturating_sub(120);
        } else if load_metrics.memory_usage < 0.3 {
            self.cache.max_size = (self.cache.max_size as f64 * 1.2) as usize;
            self.cache.base_ttl_seconds = (self.cache.base_ttl_seconds + 120).min(7200);
        }

        // Adjust storage configuration based on I/O load
        if load_metrics.storage_iops > 1000.0 {
            self.storage.connection_pool_size = (self.storage.connection_pool_size + 5).min(50);
            self.storage.batch_size = (self.storage.batch_size + 50).min(500);
        } else if load_metrics.storage_iops < 100.0 {
            self.storage.connection_pool_size =
                (self.storage.connection_pool_size.saturating_sub(2)).max(5);
            self.storage.batch_size = (self.storage.batch_size.saturating_sub(20)).max(10);
        }

        // Adjust rate limiting based on operation load
        if load_metrics.operations_per_second > 300.0 {
            self.security.rate_limit_opm = (self.security.rate_limit_opm as f64 * 0.8) as u32;
        } else if load_metrics.operations_per_second < 50.0 {
            self.security.rate_limit_opm = (self.security.rate_limit_opm as f64 * 1.2) as u32;
        }

        self.last_updated = Some(Instant::now());

        if old_profile != self.performance_profile {
            info!(
                "Performance profile changed from {:?} to {:?} based on load metrics",
                old_profile, self.performance_profile
            );
        }

        debug!("Configuration adapted to load: {:?}", load_metrics);
    }

    /// Update security posture based on threat assessment
    pub fn update_security_posture(&mut self, threat_level: ThreatLevel) {
        let old_level = self.threat_level;
        self.threat_level = threat_level;

        match threat_level {
            ThreatLevel::Low => {
                self.security.rate_limit_opm = 1000;
                self.security.token_cache_ttl_seconds = 300;
                self.security.max_concurrent_ops = 10;
                self.security.require_enhanced_validation = false;
                self.security.secret_access_timeout_seconds = 30;
                self.security.audit_retention_days = 90;
            }
            ThreatLevel::Medium => {
                self.security.rate_limit_opm = 600;
                self.security.token_cache_ttl_seconds = 180;
                self.security.max_concurrent_ops = 7;
                self.security.require_enhanced_validation = true;
                self.security.secret_access_timeout_seconds = 20;
                self.security.audit_retention_days = 180;
            }
            ThreatLevel::High => {
                self.security.rate_limit_opm = 300;
                self.security.token_cache_ttl_seconds = 60;
                self.security.max_concurrent_ops = 5;
                self.security.require_enhanced_validation = true;
                self.security.secret_access_timeout_seconds = 15;
                self.security.audit_retention_days = 365;
                // Switch to hybrid crypto for enhanced security
                if self.crypto_mode == CryptoMode::Classical {
                    self.crypto_mode = CryptoMode::Hybrid;
                }
            }
            ThreatLevel::Critical => {
                self.security.rate_limit_opm = 100;
                self.security.token_cache_ttl_seconds = 30;
                self.security.max_concurrent_ops = 2;
                self.security.require_enhanced_validation = true;
                self.security.secret_access_timeout_seconds = 10;
                self.security.audit_retention_days = 730; // 2 years
                // Switch to post-quantum crypto for maximum security
                self.crypto_mode = CryptoMode::PostQuantum;
            }
        }

        self.last_updated = Some(Instant::now());

        if old_level != threat_level {
            warn!(
                "Security posture updated from {:?} to {:?}",
                old_level, threat_level
            );
        }
    }

    /// Get adaptive TTL based on current configuration and load
    pub fn get_adaptive_ttl(&self, base_ttl: Duration, load_factor: f64) -> Duration {
        if !self.cache.adaptive_ttl {
            return base_ttl;
        }

        let base_seconds = base_ttl.as_secs();
        let adjusted_seconds = match self.performance_profile {
            PerformanceProfile::LowLatency => {
                // Shorter TTL for fresher secrets
                (base_seconds as f64 * (1.0 - load_factor * 0.3)) as u64
            }
            PerformanceProfile::HighThroughput => {
                // Longer TTL to reduce storage load
                (base_seconds as f64 * (1.0 + load_factor * 0.5)) as u64
            }
            PerformanceProfile::Balanced => {
                // Moderate adjustment
                (base_seconds as f64 * (1.0 + load_factor * 0.2)) as u64
            }
            PerformanceProfile::MaxSecurity => {
                // Shorter TTL for security (fresher validation)
                (base_seconds as f64 * 0.5) as u64
            }
        };

        let clamped_seconds = adjusted_seconds
            .max(self.cache.min_ttl_seconds)
            .min(self.cache.max_ttl_seconds);

        Duration::from_secs(clamped_seconds)
    }

    /// Get optimal batch size based on current configuration
    pub fn get_optimal_batch_size(&self, _operation_type: &str) -> usize {
        let base_size = self.storage.batch_size as usize;

        match self.performance_profile {
            PerformanceProfile::LowLatency => base_size / 2,
            PerformanceProfile::HighThroughput => base_size * 2,
            PerformanceProfile::Balanced => base_size,
            PerformanceProfile::MaxSecurity => base_size / 4, // Smaller batches for security
        }
    }

    /// Check if configuration needs update based on time threshold
    pub fn needs_update(&self, update_interval: Duration) -> bool {
        self.last_updated
            .map(|last| last.elapsed() >= update_interval)
            .unwrap_or(true)
    }
}

/// Dynamic configuration manager with thread-safe updates
pub struct DynamicConfigManager {
    config: Arc<RwLock<DynamicConfig>>,
    update_sender: watch::Sender<DynamicConfig>,
    _update_receiver: watch::Receiver<DynamicConfig>,
}

impl DynamicConfigManager {
    /// Create a new dynamic configuration manager
    pub fn new() -> Self {
        let config = Arc::new(RwLock::new(DynamicConfig::new()));
        let (update_sender, update_receiver) = watch::channel(DynamicConfig::new());

        Self {
            config,
            update_sender,
            _update_receiver: update_receiver,
        }
    }

    /// Get a clone of the current configuration
    pub fn get_config(&self) -> Result<DynamicConfig> {
        self.config
            .read()
            .map_err(|_| CoreError::internal("Failed to read dynamic config"))
            .map(|config| config.clone())
    }

    /// Update configuration based on load metrics
    pub fn update_for_load(&self, load_metrics: &LoadMetrics) -> Result<()> {
        let mut config = self
            .config
            .write()
            .map_err(|_| CoreError::internal("Failed to write dynamic config"))?;

        config.adapt_to_load(load_metrics);

        // Notify subscribers of the update
        if self.update_sender.send(config.clone()).is_err() {
            warn!("No subscribers for dynamic config updates");
        }

        Ok(())
    }

    /// Update security posture based on threat level
    pub fn update_security_posture(&self, threat_level: ThreatLevel) -> Result<()> {
        let mut config = self
            .config
            .write()
            .map_err(|_| CoreError::internal("Failed to write dynamic config"))?;

        config.update_security_posture(threat_level);

        // Notify subscribers of the update
        if self.update_sender.send(config.clone()).is_err() {
            warn!("No subscribers for dynamic config updates");
        }

        Ok(())
    }

    /// Subscribe to configuration updates
    pub fn subscribe(&self) -> watch::Receiver<DynamicConfig> {
        self.update_sender.subscribe()
    }

    /// Force update the entire configuration
    pub fn update_config(&self, new_config: DynamicConfig) -> Result<()> {
        let mut config = self
            .config
            .write()
            .map_err(|_| CoreError::internal("Failed to write dynamic config"))?;

        *config = new_config.clone();

        // Notify subscribers of the update
        if self.update_sender.send(new_config).is_err() {
            warn!("No subscribers for dynamic config updates");
        }

        Ok(())
    }
}

impl Default for DynamicConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance profiler for collecting system metrics specific to secret operations
pub struct PerformanceProfiler {
    start_time: Instant,
    operation_count: Arc<RwLock<u64>>,
    error_count: Arc<RwLock<u64>>,
    operation_times: Arc<RwLock<Vec<f64>>>,
    storage_operations: Arc<RwLock<u64>>,
}

impl PerformanceProfiler {
    /// Create a new performance profiler
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            operation_count: Arc::new(RwLock::new(0)),
            error_count: Arc::new(RwLock::new(0)),
            operation_times: Arc::new(RwLock::new(Vec::new())),
            storage_operations: Arc::new(RwLock::new(0)),
        }
    }

    /// Record a secret operation completion
    pub fn record_operation(&self, operation_time_ms: f64, is_error: bool, is_storage_op: bool) {
        if let Ok(mut count) = self.operation_count.write() {
            *count += 1;
        }

        if is_error && let Ok(mut count) = self.error_count.write() {
            *count += 1;
        }

        if is_storage_op && let Ok(mut count) = self.storage_operations.write() {
            *count += 1;
        }

        if let Ok(mut times) = self.operation_times.write() {
            times.push(operation_time_ms);
            // Keep only recent measurements (last 1000 operations)
            if times.len() > 1000 {
                let drain_count = times.len() - 1000;
                times.drain(0..drain_count);
            }
        }
    }

    /// Get current load metrics
    pub fn get_load_metrics(&self) -> LoadMetrics {
        let elapsed = self.start_time.elapsed();
        let elapsed_seconds = elapsed.as_secs_f64();

        let operation_count = *self.operation_count.read().unwrap_or_else(|poisoned| {
            warn!("Operation count lock poisoned, using 0");
            poisoned.into_inner()
        });
        let error_count = *self.error_count.read().unwrap_or_else(|poisoned| {
            warn!("Error count lock poisoned, using 0");
            poisoned.into_inner()
        });
        let operation_times = self
            .operation_times
            .read()
            .unwrap_or_else(|poisoned| {
                warn!("Operation times lock poisoned, using empty vec");
                poisoned.into_inner()
            })
            .clone();
        let storage_operations = *self.storage_operations.read().unwrap_or_else(|poisoned| {
            warn!("Storage operations lock poisoned, using 0");
            poisoned.into_inner()
        });

        let operations_per_second = if elapsed_seconds > 0.0 {
            operation_count as f64 / elapsed_seconds
        } else {
            0.0
        };

        let storage_iops = if elapsed_seconds > 0.0 {
            storage_operations as f64 / elapsed_seconds
        } else {
            0.0
        };

        let error_rate = if operation_count > 0 {
            error_count as f64 / operation_count as f64
        } else {
            0.0
        };

        let avg_operation_time_ms = if !operation_times.is_empty() {
            operation_times.iter().sum::<f64>() / operation_times.len() as f64
        } else {
            0.0
        };

        // Get system metrics (simplified - in production, use proper system monitoring)
        let cpu_usage = Self::get_cpu_usage();
        let memory_usage = Self::get_memory_usage();
        let active_connections = Self::get_active_connections();

        LoadMetrics {
            cpu_usage,
            memory_usage,
            active_connections,
            operations_per_second,
            avg_operation_time_ms,
            error_rate,
            storage_iops,
        }
    }

    /// Get CPU usage from /proc/stat (Linux-specific)
    /// Returns CPU usage percentage (0.0-100.0)
    fn get_cpu_usage() -> f64 {
        use std::fs;
        use std::thread;
        use std::time::Duration;

        // Read /proc/stat twice with a small interval to calculate usage
        let read_cpu_stats = || -> Option<(u64, u64)> {
            let contents = fs::read_to_string("/proc/stat").ok()?;
            let first_line = contents.lines().next()?;

            if !first_line.starts_with("cpu ") {
                return None;
            }

            let values: Vec<u64> = first_line
                .split_whitespace()
                .skip(1)
                .filter_map(|s| s.parse().ok())
                .collect();

            if values.len() < 4 {
                return None;
            }

            // user + nice + system + idle
            let idle = values.get(3).copied()?;
            let total: u64 = values.iter().take(7).sum(); // First 7 fields

            Some((total, idle))
        };

        if let (Some((total1, idle1)), Some((total2, idle2))) = (read_cpu_stats(), {
            thread::sleep(Duration::from_millis(100));
            read_cpu_stats()
        }) {
            let total_diff = total2.saturating_sub(total1) as f64;
            let idle_diff = idle2.saturating_sub(idle1) as f64;

            if total_diff > 0.0 {
                let usage = ((total_diff - idle_diff) / total_diff) * 100.0;
                return usage.max(0.0).min(100.0);
            }
        }

        // Fallback to 0.0 if /proc/stat unavailable or parsing fails
        0.0
    }

    /// Get memory usage from /proc/meminfo (Linux-specific)
    /// Returns memory usage percentage (0.0-100.0)
    fn get_memory_usage() -> f64 {
        use std::fs;

        let contents = match fs::read_to_string("/proc/meminfo") {
            Ok(c) => c,
            Err(_) => return 0.0, // Fallback if /proc/meminfo unavailable
        };

        let mut mem_total = None;
        let mut mem_available = None;

        for line in contents.lines() {
            if line.starts_with("MemTotal:") {
                mem_total = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse::<u64>().ok());
            } else if line.starts_with("MemAvailable:") {
                mem_available = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse::<u64>().ok());
            }

            if mem_total.is_some() && mem_available.is_some() {
                break;
            }
        }

        if let (Some(total), Some(available)) = (mem_total, mem_available)
            && total > 0
        {
            let used = total.saturating_sub(available);
            let usage = (used as f64 / total as f64) * 100.0;
            return usage.max(0.0).min(100.0);
        }

        0.0
    }

    /// Get active TCP connections from /proc/net/tcp (Linux-specific)
    /// Returns count of active connections in ESTABLISHED state
    fn get_active_connections() -> u32 {
        use std::fs;

        let tcp_files = ["/proc/net/tcp", "/proc/net/tcp6"];
        let mut count = 0;

        for tcp_file in &tcp_files {
            let contents = match fs::read_to_string(tcp_file) {
                Ok(c) => c,
                Err(_) => continue,
            };

            // Skip header line and count ESTABLISHED connections (state 01)
            for line in contents.lines().skip(1) {
                let fields: Vec<&str> = line.split_whitespace().collect();
                if fields.len() >= 4 {
                    // TCP state is in the 4th column, "01" means ESTABLISHED
                    if fields[3] == "01" {
                        count += 1;
                    }
                }
            }
        }

        count
    }
}

impl Default for PerformanceProfiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_config_creation() {
        let config = DynamicConfig::new();
        assert_eq!(config.crypto_mode, CryptoMode::Classical);
        assert_eq!(config.threat_level, ThreatLevel::Low);
        assert_eq!(config.performance_profile, PerformanceProfile::Balanced);
    }

    #[test]
    fn test_load_adaptation() {
        let mut config = DynamicConfig::new();
        let high_load = LoadMetrics {
            cpu_usage: 0.9,
            memory_usage: 0.8,
            operations_per_second: 600.0,
            storage_iops: 1200.0,
            ..Default::default()
        };

        config.adapt_to_load(&high_load);
        assert_eq!(config.performance_profile, PerformanceProfile::MaxSecurity);
        assert!(config.storage.connection_pool_size > 20);
    }

    #[test]
    fn test_threat_level_adaptation() {
        let mut config = DynamicConfig::new();

        config.update_security_posture(ThreatLevel::High);
        assert_eq!(config.threat_level, ThreatLevel::High);
        assert_eq!(config.crypto_mode, CryptoMode::Hybrid);
        assert_eq!(config.security.rate_limit_opm, 300);

        config.update_security_posture(ThreatLevel::Critical);
        assert_eq!(config.crypto_mode, CryptoMode::PostQuantum);
        assert_eq!(config.security.rate_limit_opm, 100);
    }

    #[test]
    fn test_adaptive_ttl() {
        let config = DynamicConfig::new();
        let base_ttl = Duration::from_secs(600);
        let load_factor = 0.5;

        let adaptive_ttl = config.get_adaptive_ttl(base_ttl, load_factor);
        assert!(adaptive_ttl.as_secs() >= config.cache.min_ttl_seconds);
        assert!(adaptive_ttl.as_secs() <= config.cache.max_ttl_seconds);
    }

    #[test]
    fn test_optimal_batch_size() {
        let config = DynamicConfig::new();
        let batch_size = config.get_optimal_batch_size("secret_retrieval");
        assert!(batch_size > 0);
        assert!(batch_size <= 500);
    }

    #[test]
    fn test_config_manager() {
        let manager = DynamicConfigManager::new();
        let config = manager.get_config().unwrap();
        assert_eq!(config.threat_level, ThreatLevel::Low);

        let load_metrics = LoadMetrics {
            cpu_usage: 0.5,
            memory_usage: 0.4,
            operations_per_second: 100.0,
            storage_iops: 200.0,
            ..Default::default()
        };

        manager.update_for_load(&load_metrics).unwrap();
        let updated_config = manager.get_config().unwrap();
        assert!(updated_config.last_updated.is_some());
    }

    #[test]
    fn test_performance_profiler() {
        let profiler = PerformanceProfiler::new();

        profiler.record_operation(25.0, false, true);
        profiler.record_operation(50.0, true, false);

        let metrics = profiler.get_load_metrics();
        assert_eq!(metrics.error_rate, 0.5);
        assert_eq!(metrics.avg_operation_time_ms, 37.5);
    }
}
