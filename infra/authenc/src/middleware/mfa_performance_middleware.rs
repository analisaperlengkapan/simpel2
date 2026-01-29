//! MFA Performance Monitoring Middleware
//!
//! This middleware automatically collects performance metrics for MFA-related
//! HTTP endpoints and operations.

use crate::services::mfa_performance_monitor::MfaPerformanceMonitor;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, warn};

/// MFA performance monitoring middleware
pub async fn mfa_performance_middleware(
    State(monitor): State<Arc<MfaPerformanceMonitor>>,
    request: Request,
    next: Next,
) -> Response {
    let start_time = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path();

    // Determine operation type based on path
    let operation_type = match path {
        p if p.contains("/mfa/setup") => Some("mfa_setup"),
        p if p.contains("/mfa/verify") => Some("mfa_verification"),
        p if p.contains("/mfa/status") => Some("mfa_status_lookup"),
        p if p.contains("/mfa/") => Some("mfa_other"),
        _ => None,
    };

    // Only monitor MFA-related endpoints
    if operation_type.is_none() {
        return next.run(request).await;
    }

    let operation = operation_type.unwrap();

    debug!(
        method = %method,
        path = %path,
        operation = operation,
        "Starting MFA operation monitoring"
    );

    // Execute the request
    let response = next.run(request).await;

    // Calculate duration and determine success
    let duration = start_time.elapsed();
    let success = response.status().is_success();
    let status_code = response.status();

    // Determine error type for failed operations
    let error_type = if !success {
        match status_code {
            StatusCode::BAD_REQUEST => Some("invalid_request"),
            StatusCode::UNAUTHORIZED => Some("invalid_otp"),
            StatusCode::TOO_MANY_REQUESTS => Some("rate_limit"),
            StatusCode::INTERNAL_SERVER_ERROR => Some("internal_error"),
            StatusCode::SERVICE_UNAVAILABLE => Some("service_unavailable"),
            _ => Some("other"),
        }
    } else {
        None
    };

    // Record metrics based on operation type
    match operation {
        "mfa_setup" => {
            monitor.record_setup_operation(duration, success).await;
        }
        "mfa_verification" => {
            monitor
                .record_verification_operation(duration, success, error_type)
                .await;
        }
        "mfa_status_lookup" => {
            // For status lookup, we assume cache hit if response is fast (<50ms)
            let cache_hit = success && duration.as_millis() < 50;
            monitor
                .record_status_lookup(duration, success, cache_hit)
                .await;
        }
        _ => {
            // Record as generic MFA operation
            debug!(
                operation = operation,
                duration_ms = duration.as_millis(),
                success = success,
                "Recorded generic MFA operation"
            );
        }
    }

    if !success {
        warn!(
            method = %method,
            path = %path,
            operation = operation,
            status_code = %status_code,
            duration_ms = duration.as_millis(),
            error_type = error_type,
            "MFA operation failed"
        );
    } else {
        debug!(
            method = %method,
            path = %path,
            operation = operation,
            duration_ms = duration.as_millis(),
            "MFA operation completed successfully"
        );
    }

    response
}

/// Middleware for monitoring database operations in MFA context
pub struct MfaDatabaseMiddleware {
    monitor: Arc<MfaPerformanceMonitor>,
}

impl MfaDatabaseMiddleware {
    /// Create new database monitoring middleware
    pub fn new(monitor: Arc<MfaPerformanceMonitor>) -> Self {
        Self { monitor }
    }

    /// Monitor a database query execution
    pub async fn monitor_query<F, T>(
        &self,
        query_type: &str,
        query_fn: F,
    ) -> crate::error::Result<T>
    where
        F: std::future::Future<Output = crate::error::Result<T>>,
    {
        let start_time = Instant::now();
        let result = query_fn.await;
        let duration = start_time.elapsed();
        let success = result.is_ok();

        self.monitor
            .record_database_query(query_type, duration, success)
            .await;

        result
    }
}

/// Middleware for monitoring cache operations in MFA context
pub struct MfaCacheMiddleware {
    monitor: Arc<MfaPerformanceMonitor>,
}

impl MfaCacheMiddleware {
    /// Create new cache monitoring middleware
    pub fn new(monitor: Arc<MfaPerformanceMonitor>) -> Self {
        Self { monitor }
    }

    /// Monitor a cache operation execution
    pub async fn monitor_cache_operation<F, T>(
        &self,
        operation: &str,
        cache_fn: F,
    ) -> crate::error::Result<T>
    where
        F: std::future::Future<Output = crate::error::Result<T>>,
    {
        let start_time = Instant::now();
        let result = cache_fn.await;
        let duration = start_time.elapsed();
        let success = result.is_ok();

        self.monitor
            .record_cache_operation(operation, duration, success)
            .await;

        result
    }
}

/// Performance monitoring wrapper for MFA service operations
pub struct MfaServiceMonitor {
    monitor: Arc<MfaPerformanceMonitor>,
}

impl MfaServiceMonitor {
    /// Create new MFA service monitor
    pub fn new(monitor: Arc<MfaPerformanceMonitor>) -> Self {
        Self { monitor }
    }

    /// Monitor MFA setup operation
    pub async fn monitor_setup<F, T>(&self, setup_fn: F) -> crate::error::Result<T>
    where
        F: std::future::Future<Output = crate::error::Result<T>>,
    {
        let start_time = Instant::now();
        let result = setup_fn.await;
        let duration = start_time.elapsed();
        let success = result.is_ok();

        self.monitor.record_setup_operation(duration, success).await;

        result
    }

    /// Monitor MFA verification operation
    pub async fn monitor_verification<F, T>(&self, verify_fn: F) -> crate::error::Result<T>
    where
        F: std::future::Future<Output = crate::error::Result<T>>,
    {
        let start_time = Instant::now();
        let result = verify_fn.await;
        let duration = start_time.elapsed();
        let success = result.is_ok();

        let error_type = if let Err(ref error) = result {
            match error {
                crate::error::AuthencError::InvalidOtpCode => Some("invalid_otp"),
                crate::error::AuthencError::RateLimitExceeded { .. } => Some("rate_limit"),
                crate::error::AuthencError::DatabaseError { .. } => Some("database"),
                _ => Some("other"),
            }
        } else {
            None
        };

        self.monitor
            .record_verification_operation(duration, success, error_type)
            .await;

        result
    }

    /// Monitor MFA status lookup operation
    pub async fn monitor_status_lookup<F, T>(
        &self,
        lookup_fn: F,
        cache_hit: bool,
    ) -> crate::error::Result<T>
    where
        F: std::future::Future<Output = crate::error::Result<T>>,
    {
        let start_time = Instant::now();
        let result = lookup_fn.await;
        let duration = start_time.elapsed();
        let success = result.is_ok();

        self.monitor
            .record_status_lookup(duration, success, cache_hit)
            .await;

        result
    }

    /// Monitor rate limiting check
    pub async fn monitor_rate_limit_check<F, T>(&self, check_fn: F) -> crate::error::Result<T>
    where
        F: std::future::Future<Output = crate::error::Result<T>>,
    {
        let start_time = Instant::now();
        let result = check_fn.await;
        let duration = start_time.elapsed();

        let violated = match &result {
            Err(crate::error::AuthencError::RateLimitExceeded { .. }) => true,
            _ => false,
        };

        self.monitor
            .record_rate_limit_check(duration, violated)
            .await;

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::mfa_performance_monitor::MfaPerformanceMonitor;
    use axum::{
        Router,
        body::Body,
        http::{Method, Request, StatusCode},
        middleware,
        routing::get,
    };
    use std::sync::Arc;
    use tower::ServiceExt;

    async fn test_handler() -> Result<&'static str, StatusCode> {
        Ok("test response")
    }

    async fn failing_handler() -> Result<&'static str, StatusCode> {
        Err(StatusCode::BAD_REQUEST)
    }

    #[tokio::test]
    async fn test_mfa_performance_middleware_success() {
        let monitor = Arc::new(MfaPerformanceMonitor::new(None));

        let app = Router::new()
            .route("/api/auth/mfa/setup", get(test_handler))
            .layer(middleware::from_fn_with_state(
                monitor.clone(),
                mfa_performance_middleware,
            ));

        let request = Request::builder()
            .method(Method::GET)
            .uri("/api/auth/mfa/setup")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Check that metrics were recorded
        let metrics = monitor.get_metrics().await;
        assert_eq!(metrics.setup_metrics.total_operations, 1);
        assert_eq!(metrics.setup_metrics.successful_operations, 1);
    }

    #[tokio::test]
    async fn test_mfa_performance_middleware_failure() {
        let monitor = Arc::new(MfaPerformanceMonitor::new(None));

        let app = Router::new()
            .route("/api/auth/mfa/verify", get(failing_handler))
            .layer(middleware::from_fn_with_state(
                monitor.clone(),
                mfa_performance_middleware,
            ));

        let request = Request::builder()
            .method(Method::GET)
            .uri("/api/auth/mfa/verify")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Check that metrics were recorded
        let metrics = monitor.get_metrics().await;
        assert_eq!(metrics.verification_metrics.total_operations, 1);
        assert_eq!(metrics.verification_metrics.failed_operations, 1);
    }

    #[tokio::test]
    async fn test_mfa_service_monitor() {
        let monitor = Arc::new(MfaPerformanceMonitor::new(None));
        let service_monitor = MfaServiceMonitor::new(monitor.clone());

        // Test successful operation
        let result = service_monitor
            .monitor_setup(async { Ok::<_, crate::error::AuthencError>("success") })
            .await;

        assert!(result.is_ok());

        let metrics = monitor.get_metrics().await;
        assert_eq!(metrics.setup_metrics.total_operations, 1);
        assert_eq!(metrics.setup_metrics.successful_operations, 1);
    }
}
