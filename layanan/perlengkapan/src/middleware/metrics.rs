// Metrics middleware for HTTP request tracking
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::time::Instant;

/// Middleware to track HTTP request metrics
pub async fn track_metrics(req: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = req.method().to_string();
    let path = req.uri().path().to_string();

    // Call the next middleware/handler
    let response = next.run(req).await;

    // Record metrics
    let duration = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    // Record HTTP request duration
    crate::metrics::http_request_duration()
        .with_label_values(&[&method, &path, &status])
        .observe(duration);

    // Increment HTTP request counter
    crate::metrics::http_requests_total()
        .with_label_values(&[&method, &path, &status])
        .inc();

    response
}

/// Handler for Prometheus metrics endpoint
pub async fn metrics_handler() -> impl IntoResponse {
    match crate::metrics::export_metrics() {
        Ok(metrics) => (StatusCode::OK, metrics),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to export metrics: {}", e),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
        middleware,
        routing::get,
    };
    use tower::ServiceExt;

    async fn test_handler() -> &'static str {
        "OK"
    }

    #[tokio::test]
    async fn test_metrics_middleware() {
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(track_metrics));

        let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_metrics_handler() {
        let response = metrics_handler().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
