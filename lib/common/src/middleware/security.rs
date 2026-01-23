#![cfg(feature = "axum")]

use axum::{extract::Request, middleware::Next, response::Response};

/// Middleware for adding standard security headers to responses
pub async fn security_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;

    // Add security headers
    let headers = response.headers_mut();

    // Prevent MIME type sniffing
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());

    // Prevent clickjacking
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());

    // Enable XSS protection in older browsers
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());

    // Enforce HTTPS
    headers.insert(
        "Strict-Transport-Security",
        "max-age=31536000; includeSubDomains".parse().unwrap(),
    );

    // Control referrer information
    headers.insert(
        "Referrer-Policy",
        "strict-origin-when-cross-origin".parse().unwrap(),
    );

    // Basic Content Security Policy
    headers.insert(
        "Content-Security-Policy",
        "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; frame-ancestors 'none';".parse().unwrap(),
    );

    response
}
