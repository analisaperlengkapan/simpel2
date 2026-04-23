use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use std::time::Duration;
use tokio::time::timeout;

/// Middleware for request timeouts
pub async fn timeout_middleware(
    duration: Duration,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    match timeout(duration, next.run(request)).await {
        Ok(response) => Ok(response),
        Err(_) => Err(StatusCode::REQUEST_TIMEOUT),
    }
}
