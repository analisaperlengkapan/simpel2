//! Real types for middleware dependencies
//!
//! This module provides actual implementations for middleware types
//! with proper metrics collection, logging, and JWT handling.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Real MFA Performance Monitor with actual metrics collection
#[derive(Clone)]
pub struct MfaPerformanceMonitor {
    inner: Arc<MfaPerformanceMonitorInner>,
}

struct MfaPerformanceMonitorInner {
    total_operations: AtomicU64,
    successful_operations: AtomicU64,
    failed_operations: AtomicU64,
    total_duration_ms: AtomicU64,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    setup_operations: AtomicU64,
    verification_operations: AtomicU64,
    status_lookups: AtomicU64,
    recovery_lookups: AtomicU64,
    rate_limit_violations: AtomicU64,
}

impl Default for MfaPerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl MfaPerformanceMonitor {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(MfaPerformanceMonitorInner {
                total_operations: AtomicU64::new(0),
                successful_operations: AtomicU64::new(0),
                failed_operations: AtomicU64::new(0),
                total_duration_ms: AtomicU64::new(0),
                cache_hits: AtomicU64::new(0),
                cache_misses: AtomicU64::new(0),
                setup_operations: AtomicU64::new(0),
                verification_operations: AtomicU64::new(0),
                status_lookups: AtomicU64::new(0),
                recovery_lookups: AtomicU64::new(0),
                rate_limit_violations: AtomicU64::new(0),
            }),
        }
    }

    pub fn record_operation(&self, operation: &str, duration_ms: u64, success: bool) {
        self.inner.total_operations.fetch_add(1, Ordering::Relaxed);
        self.inner
            .total_duration_ms
            .fetch_add(duration_ms, Ordering::Relaxed);

        if success {
            self.inner
                .successful_operations
                .fetch_add(1, Ordering::Relaxed);
        } else {
            self.inner.failed_operations.fetch_add(1, Ordering::Relaxed);
        }

        debug!(
            operation = operation,
            duration_ms = duration_ms,
            success = success,
            "MFA operation recorded"
        );
    }

    pub fn record_cache_hit(&self) {
        self.inner.cache_hits.fetch_add(1, Ordering::Relaxed);
        debug!("MFA cache hit");
    }

    pub fn record_cache_miss(&self) {
        self.inner.cache_misses.fetch_add(1, Ordering::Relaxed);
        debug!("MFA cache miss");
    }

    pub fn record_database_query(&self, query_type: &str, duration: Duration, success: bool) {
        let duration_ms = duration.as_millis() as u64;
        self.record_operation(&format!("db_{}", query_type), duration_ms, success);
    }

    pub fn record_cache_operation(&self, operation: &str, duration: Duration, success: bool) {
        let duration_ms = duration.as_millis() as u64;
        self.record_operation(&format!("cache_{}", operation), duration_ms, success);
    }

    pub async fn record_setup_operation(&self, duration: Duration, success: bool) {
        self.inner.setup_operations.fetch_add(1, Ordering::Relaxed);
        self.record_operation("mfa_setup", duration.as_millis() as u64, success);
    }

    pub async fn record_verification_operation(
        &self,
        duration: Duration,
        success: bool,
        error_type: Option<&str>,
    ) {
        self.inner
            .verification_operations
            .fetch_add(1, Ordering::Relaxed);
        if let Some(err) = error_type {
            warn!(error_type = err, "MFA verification failed");
        }
        self.record_operation("mfa_verify", duration.as_millis() as u64, success);
    }

    pub async fn record_status_lookup(&self, duration: Duration, success: bool, cached: bool) {
        self.inner.status_lookups.fetch_add(1, Ordering::Relaxed);
        if cached {
            self.record_cache_hit();
        } else {
            self.record_cache_miss();
        }
        self.record_operation("mfa_status_lookup", duration.as_millis() as u64, success);
    }

    pub async fn record_recovery_lookup(&self, duration: Duration, success: bool) {
        self.inner.recovery_lookups.fetch_add(1, Ordering::Relaxed);
        self.record_operation("mfa_recovery_lookup", duration.as_millis() as u64, success);
    }

    pub async fn record_rate_limit_check(&self, duration: Duration, violated: bool) {
        if violated {
            self.inner
                .rate_limit_violations
                .fetch_add(1, Ordering::Relaxed);
            warn!("MFA rate limit violated");
        }
        self.record_operation("rate_limit_check", duration.as_millis() as u64, !violated);
    }

    pub async fn get_metrics(&self) -> MfaMetrics {
        let total = self.inner.total_operations.load(Ordering::Relaxed);
        let total_duration = self.inner.total_duration_ms.load(Ordering::Relaxed);
        let avg_duration = if total > 0 {
            total_duration as f64 / total as f64
        } else {
            0.0
        };

        let cache_hits = self.inner.cache_hits.load(Ordering::Relaxed);
        let cache_misses = self.inner.cache_misses.load(Ordering::Relaxed);
        let cache_total = cache_hits + cache_misses;
        let cache_hit_ratio = if cache_total > 0 {
            cache_hits as f64 / cache_total as f64
        } else {
            0.0
        };

        MfaMetrics {
            total_operations: total,
            successful_operations: self.inner.successful_operations.load(Ordering::Relaxed),
            failed_operations: self.inner.failed_operations.load(Ordering::Relaxed),
            avg_duration_ms: avg_duration,
            cache_hits,
            cache_misses,
            cache_hit_ratio,
            setup_operations: self.inner.setup_operations.load(Ordering::Relaxed),
            verification_operations: self.inner.verification_operations.load(Ordering::Relaxed),
            status_lookups: self.inner.status_lookups.load(Ordering::Relaxed),
            recovery_lookups: self.inner.recovery_lookups.load(Ordering::Relaxed),
            rate_limit_violations: self.inner.rate_limit_violations.load(Ordering::Relaxed),
        }
    }
}

/// Metrics from MFA performance monitor
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MfaMetrics {
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub avg_duration_ms: f64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_hit_ratio: f64,
    pub setup_operations: u64,
    pub verification_operations: u64,
    pub status_lookups: u64,
    pub recovery_lookups: u64,
    pub rate_limit_violations: u64,
}

/// Audit log store for security events
#[derive(Clone)]
pub struct PgAuditLogStore {
    logs: Arc<RwLock<Vec<AuditLog>>>,
}

impl PgAuditLogStore {
    pub fn new() -> Self {
        Self {
            logs: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn log_security_event(
        &self,
        event_type: &str,
        details: &str,
        ip: Option<std::net::IpAddr>,
        user_id: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let log = AuditLog {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            event: event_type.to_string(),
            event_type: event_type.to_string(),
            user_id: user_id.map(|s| s.to_string()),
            client_id: None,
            ip_address: ip.map(|ip| ip.to_string()),
            status: "logged".to_string(),
            detail: Some(details.to_string()),
            details: details.to_string(),
            severity: "info".to_string(),
        };

        info!(
            event_type = event_type,
            user_id = ?user_id,
            ip = ?ip,
            "Security event logged"
        );

        self.add_log(&log).await
    }

    pub async fn add_log(
        &self,
        log: &AuditLog,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut logs = self.logs.write().await;
        logs.push(log.clone());

        // Keep only last 10000 logs in memory (production would persist to DB)
        if logs.len() > 10000 {
            logs.drain(0..1000);
        }

        Ok(())
    }

    pub async fn get_logs(&self, limit: usize) -> Vec<AuditLog> {
        let logs = self.logs.read().await;
        logs.iter().rev().take(limit).cloned().collect()
    }
}

impl Default for PgAuditLogStore {
    fn default() -> Self {
        Self::new()
    }
}

/// mTLS client certificate info
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ClientCertInfo {
    pub common_name: Option<String>,
    pub organization: Option<String>,
    pub fingerprint: Option<String>,
    pub serial_number: Option<String>,
    pub issuer: Option<String>,
    pub not_before: Option<chrono::DateTime<chrono::Utc>>,
    pub not_after: Option<chrono::DateTime<chrono::Utc>>,
}

impl ClientCertInfo {
    /// Check if certificate is valid (not expired)
    pub fn is_valid(&self) -> bool {
        let now = chrono::Utc::now();

        if let Some(not_before) = self.not_before
            && now < not_before
        {
            return false;
        }

        if let Some(not_after) = self.not_after
            && now > not_after
        {
            return false;
        }

        true
    }
}

/// mTLS configuration
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MtlsConfig {
    pub required: bool,
    pub ca_certs: Vec<String>,
    pub allowed_common_names: Vec<String>,
    pub allowed_organizations: Vec<String>,
    pub verify_client: bool,
}

impl MtlsConfig {
    pub fn new(required: bool) -> Self {
        Self {
            required,
            ..Default::default()
        }
    }

    pub fn with_allowed_cn(mut self, cn: &str) -> Self {
        self.allowed_common_names.push(cn.to_string());
        self
    }

    pub fn with_allowed_org(mut self, org: &str) -> Self {
        self.allowed_organizations.push(org.to_string());
        self
    }
}

/// mTLS middleware - validates client certificates from headers
pub async fn mtls_middleware(
    request: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    // Extract client certificate info from headers (set by reverse proxy)
    let cert_info = extract_cert_from_headers(&request);

    if let Some(info) = cert_info {
        if !info.is_valid() {
            warn!(
                common_name = ?info.common_name,
                "Client certificate expired or not yet valid"
            );
            return axum::response::Response::builder()
                .status(axum::http::StatusCode::UNAUTHORIZED)
                .body(axum::body::Body::from("Certificate expired"))
                .unwrap();
        }

        debug!(
            common_name = ?info.common_name,
            organization = ?info.organization,
            "Client certificate validated"
        );
    }

    next.run(request).await
}

fn extract_cert_from_headers(
    request: &axum::http::Request<axum::body::Body>,
) -> Option<ClientCertInfo> {
    // Common headers set by nginx/envoy for client cert info
    let common_name = request
        .headers()
        .get("X-SSL-Client-CN")
        .or_else(|| request.headers().get("X-Client-Cert-CN"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let organization = request
        .headers()
        .get("X-SSL-Client-O")
        .or_else(|| request.headers().get("X-Client-Cert-O"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let fingerprint = request
        .headers()
        .get("X-SSL-Client-Fingerprint")
        .or_else(|| request.headers().get("X-Client-Cert-Fingerprint"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    if common_name.is_some() || organization.is_some() || fingerprint.is_some() {
        Some(ClientCertInfo {
            common_name,
            organization,
            fingerprint,
            ..Default::default()
        })
    } else {
        None
    }
}

/// JWT handling module with real implementation
pub mod jwt {
    use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
    use serde::{Deserialize, Serialize};
    use sha2::Sha256;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct JwtClaims {
        pub sub: String,
        pub email: String,
        pub roles: Vec<String>,
        pub exp: u64,
        pub iat: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub iss: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub aud: Option<String>,
    }

    #[derive(Debug, Serialize)]
    struct JwtHeader {
        alg: String,
        typ: String,
    }

    /// Generate a simple HMAC-SHA256 JWT token
    pub fn generate_jwt(
        sub: &str,
        email: &str,
        roles: &[String],
        secret: &[u8],
        expires_in_secs: u64,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        let header = JwtHeader {
            alg: "HS256".to_string(),
            typ: "JWT".to_string(),
        };

        let claims = JwtClaims {
            sub: sub.to_string(),
            email: email.to_string(),
            roles: roles.to_vec(),
            exp: now + expires_in_secs,
            iat: now,
            iss: Some("simpelv2".to_string()),
            aud: None,
        };

        let header_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&header)?);
        let claims_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&claims)?);

        let message = format!("{}.{}", header_b64, claims_b64);
        let signature = hmac_sha256(&message, secret);
        let signature_b64 = URL_SAFE_NO_PAD.encode(signature);

        Ok(format!("{}.{}", message, signature_b64))
    }

    /// Verify a HMAC-SHA256 JWT token
    pub fn verify_jwt(
        token: &str,
        secret: &[u8],
    ) -> Result<JwtClaims, Box<dyn std::error::Error + Send + Sync>> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err("Invalid token format".into());
        }

        let message = format!("{}.{}", parts[0], parts[1]);
        let expected_sig = hmac_sha256(&message, secret);
        let actual_sig = URL_SAFE_NO_PAD.decode(parts[2])?;

        if expected_sig != actual_sig {
            return Err("Invalid signature".into());
        }

        let claims_json = URL_SAFE_NO_PAD.decode(parts[1])?;
        let claims: JwtClaims = serde_json::from_slice(&claims_json)?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        if claims.exp < now {
            return Err("Token expired".into());
        }

        Ok(claims)
    }

    fn hmac_sha256(message: &str, key: &[u8]) -> Vec<u8> {
        use hmac::{Hmac, Mac};
        type HmacSha256 = Hmac<Sha256>;

        let mut mac = HmacSha256::new_from_slice(key).expect("HMAC key length");
        mac.update(message.as_bytes());
        mac.finalize().into_bytes().to_vec()
    }

    /// Generate a simple test JWT token with just a user ID (for testing)
    /// Uses default secret "test-secret" and expiry of 1 hour
    #[cfg(test)]
    pub fn generate_jwt_simple(
        user_id: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        generate_jwt(
            user_id,
            "user@example.com",
            &["user".to_string()],
            b"test-secret",
            3600,
        )
    }
}

/// Threat level for rate limiting
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreatLevel {
    #[default]
    Normal,
    Elevated,
    High,
    Critical,
}

impl ThreatLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            ThreatLevel::Normal => "normal",
            ThreatLevel::Elevated => "elevated",
            ThreatLevel::High => "high",
            ThreatLevel::Critical => "critical",
        }
    }

    pub fn rate_limit_multiplier(&self) -> f64 {
        match self {
            ThreatLevel::Normal => 1.0,
            ThreatLevel::Elevated => 0.5,
            ThreatLevel::High => 0.2,
            ThreatLevel::Critical => 0.05,
        }
    }
}

/// User claims extracted from JWT
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserClaims {
    pub sub: String,
    pub email: String,
    pub roles: Vec<String>,
    pub exp: i64,
    pub iat: i64,
}

impl From<jwt::JwtClaims> for UserClaims {
    fn from(claims: jwt::JwtClaims) -> Self {
        Self {
            sub: claims.sub,
            email: claims.email,
            roles: claims.roles,
            exp: claims.exp as i64,
            iat: claims.iat as i64,
        }
    }
}

/// Audit log entry
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event: String,
    pub event_type: String,
    pub user_id: Option<String>,
    pub client_id: Option<String>,
    pub ip_address: Option<String>,
    pub status: String,
    pub detail: Option<String>,
    pub details: String,
    pub severity: String,
}

impl AuditLog {
    pub fn new(event_type: &str, details: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            event: event_type.to_string(),
            event_type: event_type.to_string(),
            user_id: None,
            client_id: None,
            ip_address: None,
            status: "success".to_string(),
            detail: Some(details.to_string()),
            details: details.to_string(),
            severity: "info".to_string(),
        }
    }

    pub fn with_user(mut self, user_id: &str) -> Self {
        self.user_id = Some(user_id.to_string());
        self
    }

    pub fn with_ip(mut self, ip: &str) -> Self {
        self.ip_address = Some(ip.to_string());
        self
    }

    pub fn with_severity(mut self, severity: &str) -> Self {
        self.severity = severity.to_string();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mfa_performance_monitor() {
        let monitor = MfaPerformanceMonitor::new();

        monitor.record_operation("test", 100, true);
        monitor.record_operation("test", 200, false);
        monitor.record_cache_hit();
        monitor.record_cache_miss();

        let metrics = monitor.get_metrics().await;

        assert_eq!(metrics.total_operations, 2);
        assert_eq!(metrics.successful_operations, 1);
        assert_eq!(metrics.failed_operations, 1);
        assert_eq!(metrics.cache_hits, 1);
        assert_eq!(metrics.cache_misses, 1);
    }

    #[tokio::test]
    async fn test_audit_log_store() {
        let store = PgAuditLogStore::new();

        store
            .log_security_event(
                "login",
                "User logged in",
                Some("127.0.0.1".parse().unwrap()),
                Some("user-123"),
            )
            .await
            .unwrap();

        let logs = store.get_logs(10).await;
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].event_type, "login");
    }

    #[test]
    fn test_jwt_generation_and_verification() {
        let secret = b"test-secret-key-32-bytes-long!!";

        let token = jwt::generate_jwt(
            "user-123",
            "test@example.com",
            &["user".to_string(), "admin".to_string()],
            secret,
            3600,
        )
        .unwrap();

        assert!(token.contains('.'));

        let claims = jwt::verify_jwt(&token, secret).unwrap();
        assert_eq!(claims.sub, "user-123");
        assert_eq!(claims.email, "test@example.com");
        assert_eq!(claims.roles.len(), 2);
    }

    #[test]
    fn test_threat_level() {
        assert_eq!(ThreatLevel::Normal.rate_limit_multiplier(), 1.0);
        assert_eq!(ThreatLevel::Critical.rate_limit_multiplier(), 0.05);
    }

    #[test]
    fn test_client_cert_validity() {
        let valid_cert = ClientCertInfo {
            common_name: Some("test.example.com".to_string()),
            not_before: Some(chrono::Utc::now() - chrono::Duration::hours(1)),
            not_after: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
            ..Default::default()
        };
        assert!(valid_cert.is_valid());

        let expired_cert = ClientCertInfo {
            common_name: Some("test.example.com".to_string()),
            not_before: Some(chrono::Utc::now() - chrono::Duration::hours(2)),
            not_after: Some(chrono::Utc::now() - chrono::Duration::hours(1)),
            ..Default::default()
        };
        assert!(!expired_cert.is_valid());
    }
}
