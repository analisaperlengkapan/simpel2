use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::watch;
use tracing::{debug, info, warn};

use crate::error::{AuthencError, Result};

/// Performance metrics for load-based configuration adjustment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadMetrics {
    /// CPU usage percentage (0.0 to 1.0)
    pub cpu_usage: f64,
    /// Memory usage percentage (0.0 to 1.0)
    pub memory_usage: f64,
    /// Active connections count
    pub active_connections: u32,
    /// Requests per second
    pub requests_per_second: f64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Error rate percentage (0.0 to 1.0)
    pub error_rate: f64,
}

impl Default for LoadMetrics {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            active_connections: 0,
            requests_per_second: 0.0,
            avg_response_time_ms: 0.0,
            error_rate: 0.0,
        }
    }
}

/// Security threat levels for adaptive security posture
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreatLevel {
    Low,
    /// Elevated threat - enhanced monitoring and validation
    Medium,
    /// High threat - strict security measures and rate limiting
    High,
    /// Critical threat - maximum security, minimal functionality
    Critical,
}

impl Default for ThreatLevel {
    fn default() -> Self {
        ThreatLevel::Low
    }
}

/// Cryptographic modes for post-quantum transition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Normal operations - standard security measures
pub enum CryptoMode {
    Classical,
    /// Hybrid classical + post-quantum
    Hybrid,
    /// Pure post-quantum cryptography
    PostQuantum,
}

impl Default for CryptoMode {
    fn default() -> Self {
        CryptoMode::Classical
    }
}

impl std::fmt::Display for CryptoMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            CryptoMode::Classical => "Classical",
            CryptoMode::Hybrid => "Hybrid",
            CryptoMode::PostQuantum => "PostQuantum",
        };
        write!(f, "{}", s)
    }
}

/// Performance profiles for different operational modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    /// Classical cryptography (Ed25519, AES-256-GCM)
pub enum PerformanceProfile {
    LowLatency,
    /// Balanced performance and security
    Balanced,
    /// Optimized for high throughput
    HighThroughput,
    /// Maximum security, performance secondary
    MaxSecurity,
}

impl Default for PerformanceProfile {
    fn default() -> Self {
        PerformanceProfile::Balanced
    }
}

/// Cache configuration with adaptive TTL
#[derive(Debug, Clone, Serialize, Deserialize)]
    /// Optimized for low latency
pub struct CacheConfig {
    /// Maximum cache size in entries
    pub max_size: usize,
    /// Base TTL in seconds
    pub base_ttl_seconds: u64,
    /// Whether to use adaptive TTL based on load
    pub adaptive_ttl: bool,
    /// Minimum TTL in seconds (for adaptive mode)
    pub min_ttl_seconds: u64,
    /// Maximum TTL in seconds (for adaptive mode)
    pub max_ttl_seconds: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size: 10000,
            base_ttl_seconds: 300, // 5 minutes
            adaptive_ttl: true,
            min_ttl_seconds: 60,   // 1 minute
            max_ttl_seconds: 3600, // 1 hour
        }
    }
}

/// Security configuration that adapts to threat levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Rate limit requests per minute
    pub rate_limit_rpm: u32,
    /// JWT token expiry in seconds
    pub jwt_expiry_seconds: u64,
    /// Maximum failed login attempts
    pub max_failed_attempts: u32,
    /// Brute force detection window in seconds
    pub brute_force_window_seconds: u64,
    /// Whether to require MFA for admin operations
    pub require_mfa_admin: bool,
    /// Session timeout in seconds
    pub session_timeout_seconds: u64,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            rate_limit_rpm: 100,
            jwt_expiry_seconds: 3600,
            max_failed_attempts: 5,
            brute_force_window_seconds: 300,
            require_mfa_admin: false,
            session_timeout_seconds: 1800,
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
            } else if load_metrics.requests_per_second > 1000.0 {
                PerformanceProfile::HighThroughput
            } else if load_metrics.avg_response_time_ms < 50.0 {
                PerformanceProfile::LowLatency
            } else {
                PerformanceProfile::Balanced
            };

        // Adjust cache configuration based on memory usage
        if load_metrics.memory_usage > 0.7 {
            self.cache.max_size = (self.cache.max_size as f64 * 0.8) as usize;
            self.cache.base_ttl_seconds = self.cache.base_ttl_seconds.saturating_sub(60);
        } else if load_metrics.memory_usage < 0.3 {
            self.cache.max_size = (self.cache.max_size as f64 * 1.2) as usize;
            self.cache.base_ttl_seconds = (self.cache.base_ttl_seconds + 60).min(3600);
        }

        // Adjust rate limiting based on load
        if load_metrics.requests_per_second > 500.0 {
            self.security.rate_limit_rpm = (self.security.rate_limit_rpm as f64 * 0.8) as u32;
        } else if load_metrics.requests_per_second < 50.0 {
            self.security.rate_limit_rpm = (self.security.rate_limit_rpm as f64 * 1.2) as u32;
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
                self.security.rate_limit_rpm = 100;
                self.security.jwt_expiry_seconds = 3600;
                self.security.max_failed_attempts = 5;
                self.security.require_mfa_admin = false;
                self.security.session_timeout_seconds = 1800;
            }
            ThreatLevel::Medium => {
                self.security.rate_limit_rpm = 60;
                self.security.jwt_expiry_seconds = 1800;
                self.security.max_failed_attempts = 3;
                self.security.require_mfa_admin = true;
                self.security.session_timeout_seconds = 900;
            }
            ThreatLevel::High => {
                self.security.rate_limit_rpm = 30;
                self.security.jwt_expiry_seconds = 900;
                self.security.max_failed_attempts = 2;
                self.security.require_mfa_admin = true;
                self.security.session_timeout_seconds = 600;
                // Switch to hybrid crypto for enhanced security
                if self.crypto_mode == CryptoMode::Classical {
                    self.crypto_mode = CryptoMode::Hybrid;
                }
            }
            ThreatLevel::Critical => {
                self.security.rate_limit_rpm = 10;
                self.security.jwt_expiry_seconds = 300;
                self.security.max_failed_attempts = 1;
                self.security.require_mfa_admin = true;
                self.security.session_timeout_seconds = 300;
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
                // Shorter TTL for fresher data
                (base_seconds as f64 * (1.0 - load_factor * 0.3)) as u64
            }
            PerformanceProfile::HighThroughput => {
                // Longer TTL to reduce load
                (base_seconds as f64 * (1.0 + load_factor * 0.5)) as u64
            }
            PerformanceProfile::Balanced => {
                // Moderate adjustment
                (base_seconds as f64 * (1.0 + load_factor * 0.2)) as u64
            }
            PerformanceProfile::MaxSecurity => {
                // Shorter TTL for security
                (base_seconds as f64 * 0.5) as u64
            }
        };

        let clamped_seconds = adjusted_seconds
            .max(self.cache.min_ttl_seconds)
            .min(self.cache.max_ttl_seconds);

        Duration::from_secs(clamped_seconds)
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
    /// Channel sender for broadcasting configuration updates
    update_sender: watch::Sender<DynamicConfig>,
    /// Channel receiver for internal use (not directly accessible)
    _update_receiver: watch::Receiver<DynamicConfig>,
}

impl DynamicConfigManager {
    /// Create a new dynamic configuration manager
    /// Current dynamic configuration stored in thread-safe wrapper
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
            .map_err(|_| AuthencError::internal("Failed to read dynamic config"))
            .map(|config| config.clone())
    }

    /// Update configuration based on load metrics
    pub fn update_for_load(&self, load_metrics: &LoadMetrics) -> Result<()> {
        let mut config = self
            .config
            .write()
            .map_err(|_| AuthencError::internal("Failed to write dynamic config"))?;

        config.adapt_to_load(load_metrics);

        // Notify subscribers of the update
        if let Err(_) = self.update_sender.send(config.clone()) {
            warn!("No subscribers for dynamic config updates");
        }

        Ok(())
    }

    /// Update security posture based on threat level
    pub fn update_security_posture(&self, threat_level: ThreatLevel) -> Result<()> {
        let mut config = self
            .config
            .write()
            .map_err(|_| AuthencError::internal("Failed to write dynamic config"))?;

        config.update_security_posture(threat_level);

        // Notify subscribers of the update
        if let Err(_) = self.update_sender.send(config.clone()) {
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
            .map_err(|_| AuthencError::internal("Failed to write dynamic config"))?;

        *config = new_config.clone();

        // Notify subscribers of the update
        if let Err(_) = self.update_sender.send(new_config) {
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

/// Performance profiler for collecting system metrics
pub struct PerformanceProfiler {
    start_time: Instant,
    /// Total number of requests processed
    request_count: Arc<RwLock<u64>>,
    /// Number of requests that resulted in errors
    error_count: Arc<RwLock<u64>>,
    /// Recent response times in milliseconds
    response_times: Arc<RwLock<Vec<f64>>>,
}

impl PerformanceProfiler {
    /// Create a new performance profiler
    /// Time when profiling started
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            request_count: Arc::new(RwLock::new(0)),
            error_count: Arc::new(RwLock::new(0)),
            response_times: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Record a request completion
    pub fn record_request(&self, response_time_ms: f64, is_error: bool) {
        if let Ok(mut count) = self.request_count.write() {
            *count += 1;
        }

        if is_error {
            if let Ok(mut count) = self.error_count.write() {
                *count += 1;
            }
        }

        if let Ok(mut times) = self.response_times.write() {
            times.push(response_time_ms);
            // Keep only recent measurements (last 1000 requests)
            let current_len = times.len();
            if current_len > 1000 {
                times.drain(0..current_len - 1000);
            }
        }
    }

    /// Get current load metrics
    pub fn get_load_metrics(&self) -> LoadMetrics {
        let elapsed = self.start_time.elapsed();
        let elapsed_seconds = elapsed.as_secs_f64();

        let request_count = self
            .request_count
            .read()
            .map(|guard| *guard)
            .unwrap_or_else(|_| {
                warn!("Failed to read request count");
                0
            });
        let error_count = self
            .error_count
            .read()
            .map(|guard| *guard)
            .unwrap_or_else(|_| {
                warn!("Failed to read error count");
                0
            });
        let response_times = self
            .response_times
            .read()
            .map(|guard| guard.clone())
            .unwrap_or_else(|_| {
                warn!("Failed to read response times");
                Vec::new()
            });

        let requests_per_second = if elapsed_seconds > 0.0 {
            request_count as f64 / elapsed_seconds
        } else {
            0.0
        };

        let error_rate = if request_count > 0 {
            error_count as f64 / request_count as f64
        } else {
            0.0
        };

        let avg_response_time_ms = if !response_times.is_empty() {
            response_times.iter().sum::<f64>() / response_times.len() as f64
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
            requests_per_second,
            avg_response_time_ms,
            error_rate,
        }
    }

    /// Get CPU usage (simplified implementation)
    fn get_cpu_usage() -> f64 {
        // In production, use proper system monitoring libraries
        // This is a placeholder implementation
        0.0
    }

    /// Get memory usage (simplified implementation)
    fn get_memory_usage() -> f64 {
        // In production, use proper system monitoring libraries
        // This is a placeholder implementation
        0.0
    }

    /// Get active connections count (simplified implementation)
    fn get_active_connections() -> u32 {
        // In production, track actual connection count
        // This is a placeholder implementation
        0
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
            requests_per_second: 1500.0,
            ..Default::default()
        };

        config.adapt_to_load(&high_load);
        assert_eq!(config.performance_profile, PerformanceProfile::MaxSecurity);
    }

    #[test]
    fn test_threat_level_adaptation() {
        let mut config = DynamicConfig::new();

        config.update_security_posture(ThreatLevel::High);
        assert_eq!(config.threat_level, ThreatLevel::High);
        assert_eq!(config.crypto_mode, CryptoMode::Hybrid);
        assert_eq!(config.security.rate_limit_rpm, 30);

        config.update_security_posture(ThreatLevel::Critical);
        assert_eq!(config.crypto_mode, CryptoMode::PostQuantum);
        assert_eq!(config.security.rate_limit_rpm, 10);
    }

    #[test]
    fn test_adaptive_ttl() {
        let config = DynamicConfig::new();
        let base_ttl = Duration::from_secs(300);
        let load_factor = 0.5;

        let adaptive_ttl = config.get_adaptive_ttl(base_ttl, load_factor);
        assert!(adaptive_ttl.as_secs() >= config.cache.min_ttl_seconds);
        assert!(adaptive_ttl.as_secs() <= config.cache.max_ttl_seconds);
    }

    #[test]
    fn test_config_manager() {
        let manager = DynamicConfigManager::new();
        let config = manager.get_config().unwrap();
        assert_eq!(config.threat_level, ThreatLevel::Low);

        let load_metrics = LoadMetrics {
            cpu_usage: 0.5,
            memory_usage: 0.4,
            requests_per_second: 100.0,
            ..Default::default()
        };

        manager.update_for_load(&load_metrics).unwrap();
        let updated_config = manager.get_config().unwrap();
        assert!(updated_config.last_updated.is_some());
    }

    #[test]
    fn test_performance_profiler() {
        let profiler = PerformanceProfiler::new();

        profiler.record_request(50.0, false);
        profiler.record_request(100.0, true);

        let metrics = profiler.get_load_metrics();
        assert_eq!(metrics.error_rate, 0.5);
        assert_eq!(metrics.avg_response_time_ms, 75.0);
    }
}
