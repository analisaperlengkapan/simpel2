//! Response compression middleware
//!
//! Provides response compression using tower-http's compression support.

use axum::{body::Body, extract::Request, middleware::Next, response::Response};
use serde::{Deserialize, Serialize};

/// Content encoding types supported
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentEncoding {
    /// Gzip compression
    Gzip,
    /// Deflate compression
    Deflate,
    /// Brotli compression
    Brotli,
    /// No compression
    Identity,
}

impl std::fmt::Display for ContentEncoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentEncoding::Gzip => write!(f, "gzip"),
            ContentEncoding::Deflate => write!(f, "deflate"),
            ContentEncoding::Brotli => write!(f, "br"),
            ContentEncoding::Identity => write!(f, "identity"),
        }
    }
}

/// Compression middleware function
///
/// Note: In production, use `tower_http::compression::CompressionLayer` directly
/// on the router instead. This middleware exists for custom compression logic.
pub async fn compression_middleware(request: Request<Body>, next: Next) -> Response {
    // The actual compression is handled by tower_http::compression::CompressionLayer
    // applied in the router. This middleware can add custom headers or logic.
    let response = next.run(request).await;
    response
}
