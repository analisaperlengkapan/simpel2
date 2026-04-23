use axum::http::{HeaderValue, Method};
use tower_http::cors::{Any, CorsLayer};

/// Helper to create a standard CORS layer
pub fn standard_cors(allowed_origins: Vec<String>) -> CorsLayer {
    let mut origins = Vec::new();
    for origin in allowed_origins {
        if let Ok(value) = origin.parse::<HeaderValue>() {
            origins.push(value);
        }
    }

    let layer = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
            Method::PATCH,
        ])
        .allow_headers(Any);

    if origins.is_empty() || origins.iter().any(|o| o == "*") {
        layer.allow_origin(Any)
    } else {
        layer.allow_origin(origins)
    }
}
