//! gRPC interceptors for authentication, logging, and metrics
//!
//! This module provides middleware interceptors that can be applied to gRPC services
//! to add cross-cutting concerns like authentication, request logging, and metrics collection.

use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tonic::{Request, Status};
use tracing::{debug, info, warn};

/// Authentication interceptor
///
/// Validates JWT tokens from request metadata and injects user context
/// into request extensions for downstream handlers.
#[derive(Clone)]
pub struct AuthInterceptor {
    /// Optional: List of methods that don't require authentication
    pub exempt_methods: Vec<String>,
}

impl AuthInterceptor {
    /// Create a new authentication interceptor
    pub fn new() -> Self {
        Self {
            exempt_methods: vec![
                "/authenc.v1.AuthencService/Authenticate".to_string(),
                "/authenc.v1.AuthencService/HealthCheck".to_string(),
                "/grpc.health.v1.Health/Check".to_string(),
            ],
        }
    }

    /// Check if a method is exempt from authentication
    fn is_exempt(&self, method: &str) -> bool {
        self.exempt_methods.iter().any(|m| m == method)
    }

    /// Extract and validate authorization token
    fn extract_token(&self, request: &Request<()>) -> Result<Option<String>, Status> {
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
        // TODO: Full implementation in task 1.4
        // For now, just pass through all requests
        // In the full implementation, we would:
        // 1. Extract method name from request
        // 2. Check if method is exempt
        // 3. Extract and validate JWT token
        // 4. Insert user context into request extensions

        debug!("Auth interceptor: passing through request");
        Ok(request)
    }
}

/// Logging interceptor
///
/// Logs all incoming gRPC requests with method name, metadata, and timing information.
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
    fn extract_request_id(&self, request: &Request<()>) -> Option<String> {
        request
            .metadata()
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
    }

    /// Extract client IP from metadata
    fn extract_client_ip(&self, request: &Request<()>) -> Option<String> {
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
///
/// Collects metrics for all gRPC requests including:
/// - Request count per method
/// - Request latency per method
/// - Error rate per method
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
    fn record_request_start(&self, request: &mut Request<()>) {
        let start_time = Instant::now();
        request.extensions_mut().insert(start_time);
    }

    /// Extract method name for metrics
    fn extract_method_name(&self, path: &str) -> String {
        // Extract method name from path like "/authenc.v1.AuthencService/Authenticate"
        path.split('/').last().unwrap_or("unknown").to_string()
    }
}

impl Default for MetricsInterceptor {
    fn default() -> Self {
        Self::new()
    }
}

impl tonic::service::Interceptor for MetricsInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        // Record request start time
        self.record_request_start(&mut request);

        // TODO: Increment request counter metric
        // This will be implemented when we add Prometheus metrics in task 11.2
        // metrics::counter!("grpc_requests_total").increment(1);

        debug!("Recording gRPC request metrics");

        Ok(request)
    }
}

/// Rate limiting interceptor
///
/// Applies rate limiting based on client IP or user ID to prevent abuse.
/// Uses a sliding window algorithm with per-client quota tracking.
#[derive(Clone)]
struct RateLimitEntry {
    count: u32,
    window_start: Instant,
}

#[derive(Clone)]
pub struct RateLimitInterceptor {
    /// Requests per minute limit
    pub limit: u32,
    /// Window duration (default: 1 minute)
    window_duration: Duration,
    /// Per-client rate limit entries
    entries: Arc<DashMap<String, RateLimitEntry>>,
}

impl RateLimitInterceptor {
    /// Create a new rate limiting interceptor
    pub fn new(limit: u32) -> Self {
        Self {
            limit,
            window_duration: Duration::from_secs(60), // 1 minute window
            entries: Arc::new(DashMap::new()),
        }
    }

    /// Extract client identifier for rate limiting
    fn extract_client_id(&self, request: &Request<()>) -> String {
        // Try to get user_id from extensions (set by auth interceptor)
        // Otherwise fall back to IP address

        // For now, use IP address
        request
            .metadata()
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }

    /// Check if request is within rate limit
    fn check_rate_limit(&self, client_id: &str) -> bool {
        let now = Instant::now();

        // Get or create entry for this client
        let mut entry = self
            .entries
            .entry(client_id.to_string())
            .or_insert(RateLimitEntry {
                count: 0,
                window_start: now,
            });

        // Check if window has expired and reset if needed
        if now.duration_since(entry.window_start) >= self.window_duration {
            entry.count = 0;
            entry.window_start = now;
        }

        // Check if client has exceeded limit
        if entry.count >= self.limit {
            return false; // Rate limit exceeded
        }

        // Increment counter
        entry.count += 1;
        true
    }
}

impl tonic::service::Interceptor for RateLimitInterceptor {
    fn call(&mut self, request: Request<()>) -> Result<Request<()>, Status> {
        let client_id = self.extract_client_id(&request);

        // Check rate limit
        if !self.check_rate_limit(&client_id) {
            warn!(
                client_id = %client_id,
                limit = %self.limit,
                "Rate limit exceeded"
            );
            return Err(Status::resource_exhausted(format!(
                "Rate limit exceeded: {} requests per minute. Please try again later.",
                self.limit
            )));
        }

        debug!(
            client_id = %client_id,
            limit = %self.limit,
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
    fn test_auth_interceptor_exempt_methods() {
        let interceptor = AuthInterceptor::new();

        assert!(interceptor.is_exempt("/authenc.v1.AuthencService/Authenticate"));
        assert!(interceptor.is_exempt("/authenc.v1.AuthencService/HealthCheck"));
        assert!(!interceptor.is_exempt("/authenc.v1.AuthencService/GetUser"));
    }

    #[test]
    fn test_logging_interceptor_extract_request_id() {
        let interceptor = LoggingInterceptor::new();
        let mut request = Request::new(());

        request.metadata_mut().insert(
            "x-request-id",
            MetadataValue::from_static("test-request-id"),
        );

        let request_id = interceptor.extract_request_id(&request);
        assert_eq!(request_id, Some("test-request-id".to_string()));
    }

    #[test]
    fn test_metrics_interceptor_extract_method_name() {
        let interceptor = MetricsInterceptor::new();

        let method = interceptor.extract_method_name("/authenc.v1.AuthencService/Authenticate");
        assert_eq!(method, "Authenticate");

        let method = interceptor.extract_method_name("/grpc.health.v1.Health/Check");
        assert_eq!(method, "Check");
    }

    #[test]
    fn test_rate_limit_allows_requests_under_limit() {
        let mut interceptor = RateLimitInterceptor::new(10);

        // Make 10 requests from same client - all should pass
        for i in 0..10 {
            let mut request = Request::new(());
            request
                .metadata_mut()
                .insert("x-forwarded-for", MetadataValue::from_static("192.168.1.1"));

            let result = interceptor.call(request);
            assert!(
                result.is_ok(),
                "Request {} should pass (under limit of 10)",
                i + 1
            );
        }
    }

    #[test]
    fn test_rate_limit_blocks_requests_over_limit() {
        let mut interceptor = RateLimitInterceptor::new(5);

        // Make 6 requests from same client
        for i in 0..6 {
            let mut request = Request::new(());
            request
                .metadata_mut()
                .insert("x-forwarded-for", MetadataValue::from_static("192.168.1.1"));

            let result = interceptor.call(request);

            if i < 5 {
                assert!(
                    result.is_ok(),
                    "Request {} should pass (under limit of 5)",
                    i + 1
                );
            } else {
                assert!(result.is_err(), "Request 6 should be blocked (over limit)");
                if let Err(status) = result {
                    assert_eq!(status.code(), tonic::Code::ResourceExhausted);
                    assert!(status.message().contains("Rate limit exceeded"));
                }
            }
        }
    }

    #[test]
    fn test_rate_limit_per_client_isolation() {
        let mut interceptor = RateLimitInterceptor::new(3);

        // Client A: 3 requests (all should pass)
        for i in 0..3 {
            let mut request = Request::new(());
            request
                .metadata_mut()
                .insert("x-forwarded-for", MetadataValue::from_static("192.168.1.1"));
            let result = interceptor.call(request);
            assert!(result.is_ok(), "Client A request {} should pass", i + 1);
        }

        // Client A: 4th request (should be blocked)
        let mut request = Request::new(());
        request
            .metadata_mut()
            .insert("x-forwarded-for", MetadataValue::from_static("192.168.1.1"));
        assert!(
            interceptor.call(request).is_err(),
            "Client A 4th request should be blocked"
        );

        // Client B: 3 requests (all should pass - independent quota)
        for i in 0..3 {
            let mut request = Request::new(());
            request
                .metadata_mut()
                .insert("x-forwarded-for", MetadataValue::from_static("192.168.1.2"));
            let result = interceptor.call(request);
            assert!(
                result.is_ok(),
                "Client B request {} should pass (independent quota)",
                i + 1
            );
        }
    }

    #[test]
    fn test_rate_limit_extract_client_id() {
        let interceptor = RateLimitInterceptor::new(100);

        // Test with x-forwarded-for
        let mut request = Request::new(());
        request.metadata_mut().insert(
            "x-forwarded-for",
            MetadataValue::from_static("203.0.113.42, 198.51.100.17"),
        );
        let client_id = interceptor.extract_client_id(&request);
        assert_eq!(client_id, "203.0.113.42");

        // Test without header (fallback to "unknown")
        let request2 = Request::new(());
        let client_id2 = interceptor.extract_client_id(&request2);
        assert_eq!(client_id2, "unknown");
    }
}
