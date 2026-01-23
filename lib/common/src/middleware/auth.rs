#![cfg(feature = "axum")]

use crate::error::CommonError;
use axum::{
    body::Body,
    http::{Request, Response},
    middleware::Next,
};

/// API Key verification middleware
pub async fn api_key_middleware(
    expected_key: String,
    req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, CommonError> {
    let key = req
        .headers()
        .get("x-api-key")
        .and_then(|v: &axum::http::HeaderValue| v.to_str().ok());
    if key != Some(&expected_key) {
        return Err(CommonError::Internal(
            "Unauthorized: Invalid API Key".to_string(),
        ));
    }
    Ok(next.run(req).await)
}
