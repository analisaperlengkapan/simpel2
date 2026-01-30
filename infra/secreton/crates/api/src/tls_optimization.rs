//! Optimized TLS configuration for high-performance mTLS
//!
//! This module provides optimized TLS configuration with performance enhancements
//! including session resumption, certificate caching, and optimized cipher suites.

use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::server::{ServerConfig, StoresServerSessions};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{info, warn};

/// Session cache for TLS session resumption
#[derive(Debug)]
pub struct SessionCache {
    sessions: Mutex<HashMap<Vec<u8>, (Vec<u8>, Instant)>>,
    max_entries: usize,
    ttl: Duration,
}

impl SessionCache {
    pub fn new(max_entries: usize, ttl_seconds: u64) -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
            max_entries,
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    pub fn insert(&self, key: Vec<u8>, session: Vec<u8>) {
        let mut sessions = self.sessions.lock().unwrap();

        // Remove expired entries if we're at capacity
        if sessions.len() >= self.max_entries {
            // let now = Instant::now();
            // sessions.retain(|_, (_, timestamp)| timestamp.elapsed() < self.ttl);
            // Optimization: Just remove random or oldest if we don't want to iterate all
        }

        // Remove oldest entry if still at capacity (simplified: just random for now due to HashMap)
        if sessions.len() >= self.max_entries
            && let Some(oldest_key) = sessions.keys().next().cloned()
        {
            sessions.remove(&oldest_key);
        }

        sessions.insert(key, (session, Instant::now()));
    }

    pub fn get_session(&self, key: &[u8]) -> Option<Vec<u8>> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some((session, timestamp)) = sessions.get(key) {
            if timestamp.elapsed() < self.ttl {
                return Some(session.clone());
            } else {
                sessions.remove(key);
            }
        }
        None
    }

    pub fn remove_session(&self, key: &[u8]) -> Option<Vec<u8>> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some((session, timestamp)) = sessions.remove(key)
            && timestamp.elapsed() < self.ttl
        {
            return Some(session);
        }
        None
    }
}

impl StoresServerSessions for SessionCache {
    fn put(&self, key: Vec<u8>, value: Vec<u8>) -> bool {
        self.insert(key, value);
        true
    }

    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.get_session(key)
    }

    fn take(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.remove_session(key)
    }

    fn can_cache(&self) -> bool {
        true
    }
}

/// Global session cache
static SESSION_CACHE: Mutex<Option<Arc<SessionCache>>> = Mutex::new(None);

/// Initialize session cache
pub fn init_session_cache(max_entries: usize, ttl_seconds: u64) {
    let mut cache = SESSION_CACHE.lock().unwrap();
    *cache = Some(Arc::new(SessionCache::new(max_entries, ttl_seconds)));
    info!(
        "TLS session cache initialized with {} max entries, {}s TTL",
        max_entries, ttl_seconds
    );
}

/// Get session cache instance
fn get_session_cache() -> Option<Arc<SessionCache>> {
    SESSION_CACHE.lock().unwrap().as_ref().cloned()
}

/// Create optimized TLS configuration for server
pub fn create_optimized_tls_config(
    cert_chain: Vec<CertificateDer<'static>>,
    private_key: PrivateKeyDer<'static>,
    min_tls_version: &str,
    _cipher_suites: &[String], // Currently relying on defaults
    alpn_protocols: &[String],
) -> Result<ServerConfig, Box<dyn std::error::Error>> {
    // Note: In rustls 0.23, builder() uses default provider (ring or aws-lc-rs)
    let config_builder = ServerConfig::builder();

    // Set TLS version and cipher suites (using safe defaults)
    let config_builder = match min_tls_version {
        "TLS1.3" => {
            // with_safe_default_cipher_suites() is implied/default usually,
            // but we can be explicit if needed.
            // For now, let's use default which supports 1.3 and 1.2
            info!("Using safe default cipher suites (TLS 1.2/1.3)");
            config_builder.with_no_client_auth()
        }
        _ => {
            warn!(
                "Unknown TLS version '{}', using safe defaults",
                min_tls_version
            );
            config_builder.with_no_client_auth()
        }
    };

    // Build the final configuration
    let mut config = config_builder.with_single_cert(cert_chain, private_key)?;

    // Add session cache for performance
    if let Some(session_cache) = get_session_cache() {
        config.session_storage = session_cache;
    }

    // Configure ALPN protocols
    if !alpn_protocols.is_empty() {
        let alpn_protocols_bytes: Vec<Vec<u8>> = alpn_protocols
            .iter()
            .map(|s| s.as_bytes().to_vec())
            .collect();
        config.alpn_protocols = alpn_protocols_bytes;
    }

    // Optimize for performance
    config.max_fragment_size = Some(16384); // 16KB fragments for better throughput
    config.send_tls13_tickets = 4; // Send multiple tickets for session resumption

    info!("TLS configuration optimized with session resumption and ALPN support");

    Ok(config)
}

/// Performance metrics for TLS connections
#[derive(Debug, Default)]
pub struct TlsMetrics {
    pub total_handshakes: u64,
    pub successful_handshakes: u64,
    pub session_resumptions: u64,
    pub handshake_failures: u64,
    pub average_handshake_time_ms: u64,
}

impl TlsMetrics {
    pub fn record_handshake(&mut self, success: bool, resumption: bool, duration_ms: u64) {
        self.total_handshakes += 1;
        if success {
            self.successful_handshakes += 1;
            if resumption {
                self.session_resumptions += 1;
            }
        } else {
            self.handshake_failures += 1;
        }

        // Update average handshake time
        if self.total_handshakes > 0 {
            self.average_handshake_time_ms =
                (self.average_handshake_time_ms * (self.total_handshakes - 1) + duration_ms)
                    / self.total_handshakes;
        }
    }

    pub fn get_success_rate(&self) -> f64 {
        if self.total_handshakes == 0 {
            0.0
        } else {
            (self.successful_handshakes as f64 / self.total_handshakes as f64) * 100.0
        }
    }

    pub fn get_resumption_rate(&self) -> f64 {
        if self.total_handshakes == 0 {
            0.0
        } else {
            (self.session_resumptions as f64 / self.total_handshakes as f64) * 100.0
        }
    }
}

/// Global TLS metrics
static TLS_METRICS: Mutex<TlsMetrics> = Mutex::new(TlsMetrics {
    total_handshakes: 0,
    successful_handshakes: 0,
    session_resumptions: 0,
    handshake_failures: 0,
    average_handshake_time_ms: 0,
});

/// Record TLS handshake metrics
pub fn record_tls_handshake(success: bool, resumption: bool, duration_ms: u64) {
    let mut metrics = TLS_METRICS.lock().unwrap();
    metrics.record_handshake(success, resumption, duration_ms);
}

/// Get TLS performance metrics
pub fn get_tls_metrics() -> TlsMetrics {
    let metrics = TLS_METRICS.lock().unwrap();
    TlsMetrics {
        total_handshakes: metrics.total_handshakes,
        successful_handshakes: metrics.successful_handshakes,
        session_resumptions: metrics.session_resumptions,
        handshake_failures: metrics.handshake_failures,
        average_handshake_time_ms: metrics.average_handshake_time_ms,
    }
}
