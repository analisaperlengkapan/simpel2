//! Cache middleware for HTTP responses
//!
//! Provides Axum middleware for caching HTTP responses based on sensitivity levels.

#[cfg(feature = "axum")]
use crate::cache::{AsyncLruCache, SensitivityLevel};
#[cfg(feature = "axum")]
use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderValue, StatusCode, header},
    middleware::Next,
    response::Response,
};
#[cfg(feature = "axum")]
use bytes::Bytes;
#[cfg(feature = "axum")]
use http_body_util::BodyExt;
#[cfg(feature = "axum")]
use std::sync::Arc;
#[cfg(feature = "axum")]
use std::time::Duration;

#[cfg(feature = "axum")]
#[derive(Clone)]
pub struct CacheConfig {
    pub enabled: bool,
    pub default_ttl: Duration,
    pub default_sensitivity: SensitivityLevel,
    pub cache_control_header: bool,
}

#[cfg(feature = "axum")]
impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_ttl: Duration::from_secs(300), // 5 minutes
            default_sensitivity: SensitivityLevel::Medium,
            cache_control_header: true,
        }
    }
}

#[cfg(feature = "axum")]
#[derive(Clone)]
pub struct CacheMiddlewareState {
    cache: AsyncLruCache<String, CachedResponse>,
    config: CacheConfig,
}

#[cfg(feature = "axum")]
#[derive(Clone)]
struct CachedResponse {
    status: StatusCode,
    headers: Vec<(String, String)>,
    body: Bytes,
}

#[cfg(feature = "axum")]
impl CacheMiddlewareState {
    pub fn new(capacity: usize, config: CacheConfig) -> Self {
        Self {
            cache: AsyncLruCache::new(capacity),
            config,
        }
    }

    pub fn with_default_config(capacity: usize) -> Self {
        Self::new(capacity, CacheConfig::default())
    }
}

#[cfg(feature = "axum")]
pub async fn cache_middleware(
    State(state): State<Arc<CacheMiddlewareState>>,
    request: Request,
    next: Next,
) -> std::result::Result<Response, (StatusCode, String)> {
    // Only cache GET requests
    if request.method() != axum::http::Method::GET || !state.config.enabled {
        return Ok(next.run(request).await);
    }

    // Build cache key from request path and query
    let cache_key = build_cache_key(&request);

    // Check cache
    if let Some(cached) = state.cache.get(&cache_key).await {
        return Ok(build_response_from_cache(cached, &state.config));
    }

    // Execute request
    let response = next.run(request).await;

    // Only cache successful responses
    if response.status().is_success() {
        // Clone response for caching
        let (parts, body) = response.into_parts();

        // Collect body bytes
        let body_bytes = match body.collect().await {
            Ok(collected) => collected.to_bytes(),
            Err(_) => {
                // If we can't collect the body, just return the original response
                return Ok(Response::from_parts(parts, Body::empty()));
            }
        };

        // Determine sensitivity from headers or use default
        let sensitivity = extract_sensitivity_from_headers(&parts.headers)
            .unwrap_or(state.config.default_sensitivity);

        // Determine TTL from headers or use default
        let ttl = extract_ttl_from_headers(&parts.headers).unwrap_or(state.config.default_ttl);

        // Cache the response
        let cached = CachedResponse {
            status: parts.status,
            headers: parts
                .headers
                .iter()
                .filter(|(name, _)| should_cache_header(name.as_str()))
                .map(|(name, value)| {
                    (
                        name.as_str().to_string(),
                        value.to_str().unwrap_or("").to_string(),
                    )
                })
                .collect(),
            body: body_bytes.clone(),
        };

        state
            .cache
            .insert_with_sensitivity(cache_key, cached, ttl, sensitivity)
            .await;

        // Rebuild response
        let mut response = Response::from_parts(parts, Body::from(body_bytes));

        // Add cache headers if enabled
        if state.config.cache_control_header {
            add_cache_headers(&mut response, ttl);
        }

        Ok(response)
    } else {
        Ok(response)
    }
}

#[cfg(feature = "axum")]
fn build_cache_key(request: &Request) -> String {
    let path = request.uri().path();
    let query = request.uri().query().unwrap_or("");

    if query.is_empty() {
        path.to_string()
    } else {
        format!("{}?{}", path, query)
    }
}

#[cfg(feature = "axum")]
fn build_response_from_cache(cached: CachedResponse, config: &CacheConfig) -> Response {
    let mut response = Response::builder().status(cached.status);

    // Add cached headers
    for (name, value) in cached.headers {
        if let Ok(header_value) = HeaderValue::from_str(&value) {
            response = response.header(name, header_value);
        }
    }

    // Add X-Cache header to indicate cache hit
    response = response.header("X-Cache", "HIT");

    // Add cache control headers if enabled
    if config.cache_control_header {
        response = response.header(
            header::CACHE_CONTROL,
            format!("max-age={}", config.default_ttl.as_secs()),
        );
    }

    response.body(Body::from(cached.body)).unwrap()
}

#[cfg(feature = "axum")]
fn should_cache_header(name: &str) -> bool {
    // Don't cache certain headers
    !matches!(
        name.to_lowercase().as_str(),
        "set-cookie"
            | "authorization"
            | "www-authenticate"
            | "proxy-authenticate"
            | "proxy-authorization"
            | "age"
            | "cache-control"
            | "expires"
            | "pragma"
            | "warning"
    )
}

#[cfg(feature = "axum")]
fn extract_sensitivity_from_headers(headers: &axum::http::HeaderMap) -> Option<SensitivityLevel> {
    headers
        .get("X-Cache-Sensitivity")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| match s.to_lowercase().as_str() {
            "low" => Some(SensitivityLevel::Low),
            "medium" => Some(SensitivityLevel::Medium),
            "high" => Some(SensitivityLevel::High),
            "critical" => Some(SensitivityLevel::Critical),
            _ => None,
        })
}

#[cfg(feature = "axum")]
fn extract_ttl_from_headers(headers: &axum::http::HeaderMap) -> Option<Duration> {
    headers
        .get("X-Cache-TTL")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .map(Duration::from_secs)
}

#[cfg(feature = "axum")]
fn add_cache_headers(response: &mut Response, ttl: Duration) {
    let headers = response.headers_mut();

    // Add Cache-Control header
    if let Ok(value) = HeaderValue::from_str(&format!("max-age={}", ttl.as_secs())) {
        headers.insert(header::CACHE_CONTROL, value);
    }

    // Add X-Cache header to indicate cache miss
    headers.insert("X-Cache", HeaderValue::from_static("MISS"));
}
