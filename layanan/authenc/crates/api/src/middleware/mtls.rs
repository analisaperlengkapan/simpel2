// mTLS middleware
//! Mutual TLS (mTLS) client certificate authentication middleware.
//! TODO: Implement full mTLS validation with certificate chain verification.

use axum::{extract::Request, middleware::Next, response::Response};

/// Client certificate information extracted from mTLS handshake
#[derive(Debug, Clone)]
pub struct ClientCertInfo {
    /// Subject common name from the client certificate
    pub common_name: String,
    /// Certificate serial number
    pub serial_number: String,
}

/// mTLS configuration
#[derive(Debug, Clone, Default)]
pub struct MtlsConfig {
    /// Whether mTLS is required
    pub required: bool,
    /// Trusted CA certificate paths
    pub trusted_ca_paths: Vec<String>,
}

/// mTLS middleware - validates client certificates
/// TODO: Implement actual certificate extraction and validation
pub async fn mtls_middleware(request: Request, next: Next) -> Response {
    // TODO: Extract and validate client certificate from TLS connection
    next.run(request).await
}
