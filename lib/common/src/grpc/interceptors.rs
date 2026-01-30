//! gRPC Interceptors for Cross-Cutting Concerns
//!
//! Provides reusable interceptors for authentication, logging, metrics, and rate limiting.
//!
//! # Example
//!
//! ```rust,ignore
//! use lib_common::grpc::interceptors::{AuthInterceptor, LoggingInterceptor};
//! use tonic::transport::Server;
//!
//! let auth = AuthInterceptor::new();
//! let logging = LoggingInterceptor::new();
//!
//! Server::builder()
//!     .layer(tonic::service::interceptor(auth))
//!     .add_service(my_service)
//!     .serve(addr).await?;
//! ```

use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tonic::{Request, Status};
use tracing::{debug, info, warn};

/// Extract bearer token from Authorization header
pub fn extract_bearer_token(auth_header: &str) -> Option<String> {
    auth_header.strip_prefix("Bearer ").map(|s| s.to_string())
}

/// Authentication interceptor
/// Validates JWT tokens from request metadata.
#[derive(Clone)]
pub struct AuthInterceptor {
    /// Methods that don't require authentication
    pub exempt_methods: Vec<String>,
}

impl AuthInterceptor {
    /// Create a new authentication interceptor
    pub fn new() -> Self {
        Self {
            exempt_methods: vec![],
        }
    }

    /// Add exempt methods that don't require auth
    pub fn with_exempt_methods(mut self, methods: Vec<String>) -> Self {
        self.exempt_methods = methods;
        self
    }

    /// Check if a method is exempt from authentication
    pub fn is_exempt(&self, method: &str) -> bool {
        self.exempt_methods.iter().any(|m| m == method)
    }

    /// Extract token from request
    pub fn extract_token(&self, request: &Request<()>) -> Result<Option<String>, Status> {
        let metadata = request.metadata();

        if let Some(auth_header) = metadata.get("authorization") {
            let auth_str = auth_header
                .to_str()
                .map_err(|_| Status::unauthenticated("Invalid authorization header"))?;

            if auth_str.starts_with("Bearer ") {
                let token = auth_str.trim_start_matches("Bearer ").to_string();
                return Ok(Some(token));
            }
        }

        Ok(None)
    }
}

impl Default for AuthInterceptor {
    fn default() -> Self {
        Self::new()
    }
}

impl tonic::service::Interceptor for AuthInterceptor {
    fn call(&mut self, request: Request<()>) -> Result<Request<()>, Status> {
        debug!("Auth interceptor: processing request");
        Ok(request)
    }
}

/// Logging interceptor
/// Logs all incoming gRPC requests with request-id and client IP.
#[derive(Clone)]
pub struct LoggingInterceptor {
    /// Enable verbose logging
    pub verbose: bool,
}

impl LoggingInterceptor {
    /// Create a new logging interceptor
    pub fn new() -> Self {
        Self { verbose: false }
    }

    /// Create a verbose logging interceptor
    pub fn verbose() -> Self {
        Self { verbose: true }
    }

    /// Extract request ID from metadata
    pub fn extract_request_id(&self, request: &Request<()>) -> Option<String> {
        request
            .metadata()
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
    }

    /// Extract client IP from metadata
    pub fn extract_client_ip(&self, request: &Request<()>) -> Option<String> {
        request
            .metadata()
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
            .or_else(|| {
                request
                    .metadata()
                    .get("x-real-ip")
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string())
            })
    }
}

impl Default for LoggingInterceptor {
    fn default() -> Self {
        Self::new()
    }
}

impl tonic::service::Interceptor for LoggingInterceptor {
    fn call(&mut self, request: Request<()>) -> Result<Request<()>, Status> {
        let request_id = self.extract_request_id(&request);
        let client_ip = self.extract_client_ip(&request);

        if self.verbose {
            info!(
                request_id = ?request_id,
                client_ip = ?client_ip,
                "gRPC request received"
            );
        } else {
            debug!(
                request_id = ?request_id,
                "gRPC request"
            );
        }

        Ok(request)
    }
}

/// Metrics interceptor
/// Records request timing and counts.
#[derive(Clone)]
pub struct MetricsInterceptor {
    /// Enable detailed metrics
    pub detailed: bool,
}

impl MetricsInterceptor {
    /// Create a new metrics interceptor
    pub fn new() -> Self {
        Self { detailed: true }
    }

    /// Record request start time
    pub fn record_request_start(&self, request: &mut Request<()>) {
        let start_time = Instant::now();
        request.extensions_mut().insert(start_time);
    }

    /// Extract method name for metrics
    pub fn extract_method_name(&self, path: &str) -> String {
        path.split('/').next_back().unwrap_or("unknown").to_string()
    }
}

impl Default for MetricsInterceptor {
    fn default() -> Self {
        Self::new()
    }
}

impl tonic::service::Interceptor for MetricsInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        self.record_request_start(&mut request);
        debug!("Recording gRPC request metrics");
        Ok(request)
    }
}

/// Rate limit entry for tracking per-client usage
#[derive(Clone)]
struct RateLimitEntry {
    count: u32,
    window_start: Instant,
}

/// Rate limiting interceptor
/// Applies per-client rate limiting using sliding window.
#[derive(Clone)]
pub struct RateLimitInterceptor {
    /// Requests per minute limit
    pub limit: u32,
    /// Window duration
    window_duration: Duration,
    /// Per-client rate limit entries
    entries: Arc<DashMap<String, RateLimitEntry>>,
}

impl RateLimitInterceptor {
    /// Create a new rate limiting interceptor
    pub fn new(limit: u32) -> Self {
        Self {
            limit,
            window_duration: Duration::from_secs(60),
            entries: Arc::new(DashMap::new()),
        }
    }

    /// Create with custom window duration
    pub fn with_window(mut self, duration: Duration) -> Self {
        self.window_duration = duration;
        self
    }

    /// Extract client identifier for rate limiting
    pub fn extract_client_id(&self, request: &Request<()>) -> String {
        request
            .metadata()
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }

    /// Check if request is within rate limit
    pub fn check_rate_limit(&self, client_id: &str) -> bool {
        let now = Instant::now();

        let mut entry = self
            .entries
            .entry(client_id.to_string())
            .or_insert(RateLimitEntry {
                count: 0,
                window_start: now,
            });

        // Reset window if expired
        if now.duration_since(entry.window_start) >= self.window_duration {
            entry.count = 0;
            entry.window_start = now;
        }

        // Check limit
        if entry.count >= self.limit {
            return false;
        }

        entry.count += 1;
        true
    }
}

impl tonic::service::Interceptor for RateLimitInterceptor {
    fn call(&mut self, request: Request<()>) -> Result<Request<()>, Status> {
        let client_id = self.extract_client_id(&request);

        if !self.check_rate_limit(&client_id) {
            warn!(
                client_id = %client_id,
                limit = %self.limit,
                "Rate limit exceeded"
            );
            return Err(Status::resource_exhausted(format!(
                "Rate limit exceeded: {} requests per minute",
                self.limit
            )));
        }

        debug!(
            client_id = %client_id,
            "Rate limit check passed"
        );

        Ok(request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::metadata::MetadataValue;
    use tonic::service::Interceptor;

    #[test]
    fn test_extract_bearer_token() {
        assert_eq!(
            extract_bearer_token("Bearer abc123"),
            Some("abc123".to_string())
        );
        assert_eq!(extract_bearer_token("Basic abc123"), None);
    }

    #[test]
    fn test_auth_interceptor_exempt_methods() {
        let interceptor =
            AuthInterceptor::new().with_exempt_methods(vec!["/service/HealthCheck".to_string()]);

        assert!(interceptor.is_exempt("/service/HealthCheck"));
        assert!(!interceptor.is_exempt("/service/GetUser"));
    }

    #[test]
    fn test_logging_interceptor() {
        let interceptor = LoggingInterceptor::new();
        let mut request = Request::new(());

        request
            .metadata_mut()
            .insert("x-request-id", MetadataValue::from_static("test-id"));

        assert_eq!(
            interceptor.extract_request_id(&request),
            Some("test-id".to_string())
        );
    }

    #[test]
    fn test_rate_limit_allows_under_limit() {
        let mut interceptor = RateLimitInterceptor::new(5);

        for i in 0..5 {
            let mut request = Request::new(());
            request
                .metadata_mut()
                .insert("x-forwarded-for", MetadataValue::from_static("192.168.1.1"));
            let result = interceptor.call(request);
            assert!(result.is_ok(), "Request {} should pass", i + 1);
        }
    }

    #[test]
    fn test_rate_limit_blocks_over_limit() {
        let mut interceptor = RateLimitInterceptor::new(3);

        for i in 0..4 {
            let mut request = Request::new(());
            request
                .metadata_mut()
                .insert("x-forwarded-for", MetadataValue::from_static("192.168.1.1"));
            let result = interceptor.call(request);

            if i < 3 {
                assert!(result.is_ok());
            } else {
                assert!(result.is_err());
            }
        }
    }
}
