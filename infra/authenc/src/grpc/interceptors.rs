//! gRPC interceptors for authentication, logging, and metrics
//!
//! This module provides middleware interceptors that can be applied to gRPC services
//! to add cross-cutting concerns like authentication, request logging, and metrics collection.

use std::time::Instant;
use tonic::{Request, Status};
use tracing::{debug, info};

/// Authentication interceptor
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
/// Applies rate limiting based on client IP or user ID to prevent abuse.
#[derive(Clone)]
pub struct RateLimitInterceptor {
    /// Requests per minute limit
    pub limit: u32,
}

impl RateLimitInterceptor {
    /// Create a new rate limiting interceptor
    pub fn new(limit: u32) -> Self {
        Self { limit }
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
}

impl tonic::service::Interceptor for RateLimitInterceptor {
    fn call(&mut self, request: Request<()>) -> Result<Request<()>, Status> {
        let client_id = self.extract_client_id(&request);

        // TODO: Implement actual rate limiting logic
        // This will be implemented in task 7.4 (adaptive rate limiting)
        // For now, just pass th
        debug!(client_id = %client_id, limit = %self.limit, "Rate limit check");

        Ok(request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::metadata::MetadataValue;

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
}
