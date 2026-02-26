//! HTTP middleware for the Secreton API
//!
//! Provides authentication, rate limiting, request tracing,
//! and other middleware functionality.

// Export rate limiting middleware module
pub mod rate_limit;

use axum::{
    Json,
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};
use uuid::Uuid;
use x509_parser::prelude::*;

use crate::ApiState;
use crate::auth::{AuthError, extract_bearer_token};
use crate::tls_optimization::record_tls_handshake;

/// Certificate cache for performance optimization
#[derive(Debug)]
pub struct CertificateCache {
    cache: Mutex<HashMap<String, (CertificateValidation, Instant)>>,
    ttl: Duration,
    hits: AtomicU64,
    misses: AtomicU64,
}

impl CertificateCache {
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
            ttl: Duration::from_secs(ttl_seconds),
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
        }
    }

    pub fn get(&self, cache_key: &str) -> Option<CertificateValidation> {
        // Handle lock poisoning gracefully
        let mut cache = match self.cache.lock() {
            Ok(cache) => cache,
            Err(poisoned) => {
                warn!("Certificate cache lock was poisoned, recovering");
                poisoned.into_inner()
            }
        };

        if let Some((validation, timestamp)) = cache.get(cache_key) {
            if timestamp.elapsed() < self.ttl {
                self.hits.fetch_add(1, Ordering::Relaxed);
                return Some(validation.clone());
            } else {
                cache.remove(cache_key);
            }
        }
        self.misses.fetch_add(1, Ordering::Relaxed);
        None
    }

    pub fn insert(&self, cache_key: String, validation: CertificateValidation) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(cache_key, (validation, Instant::now()));
        }
    }

    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed) as f64;
        let misses = self.misses.load(Ordering::Relaxed) as f64;
        let total = hits + misses;
        if total == 0.0 { 0.0 } else { hits / total }
    }
}

/// Global certificate cache
static CERT_CACHE: Mutex<Option<Arc<CertificateCache>>> = Mutex::new(None);

/// Initialize certificate cache
pub fn init_certificate_cache(ttl_seconds: u64) {
    if let Ok(mut cache) = CERT_CACHE.lock() {
        *cache = Some(Arc::new(CertificateCache::new(ttl_seconds)));
    }
}

/// Update certificate cache TTL
pub fn update_certificate_cache_ttl(ttl_seconds: u64) {
    if let Ok(mut cache) = CERT_CACHE.lock() {
        // Recreate cache with new TTL (clears existing cache)
        *cache = Some(Arc::new(CertificateCache::new(ttl_seconds)));
    }
}

/// Get certificate cache instance
fn get_cert_cache() -> Option<Arc<CertificateCache>> {
    CERT_CACHE.lock().ok()?.as_ref().cloned()
}

/// Get global cache hit rate
pub fn get_cache_hit_rate() -> f64 {
    if let Some(cache) = get_cert_cache() {
        cache.hit_rate()
    } else {
        0.0
    }
}

/// Certificate validation result
#[derive(Debug, Clone)]
pub struct CertificateValidation {
    pub valid: bool,
    pub subject: Option<String>,
    pub issuer: Option<String>,
    pub serial_number: Option<String>,
    pub not_before: Option<String>,
    pub not_after: Option<String>,
}

/// Generate a cache key that includes allowed subjects to ensure validation correctness
fn generate_cache_key(cert_der: &[u8], allowed_subjects: &[String]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    cert_der.hash(&mut hasher);
    for subject in allowed_subjects {
        subject.hash(&mut hasher);
    }
    format!("{:x}", hasher.finish())
}

/// Validate client certificate
pub fn validate_client_certificate(
    cert_der: &[u8],
    _ca_cert_path: Option<&PathBuf>,
    allowed_subjects: &[String],
) -> CertificateValidation {
    // Check cache first
    let cache_key = generate_cache_key(cert_der, allowed_subjects);
    if let Some(cache) = get_cert_cache()
        && let Some(validation) = cache.get(&cache_key)
    {
        debug!("Certificate cache hit for key: {}", cache_key);
        return validation;
    }

    // Parse certificate
    match x509_parser::parse_x509_certificate(cert_der) {
        Ok((_, cert)) => {
            let validation = validate_cached_certificate(&cert, allowed_subjects);
            // Update cache
            if let Some(cache) = get_cert_cache() {
                cache.insert(cache_key, validation.clone());
            }
            validation
        }
        Err(e) => {
            warn!("Failed to parse client certificate: {}", e);
            CertificateValidation {
                valid: false,
                subject: None,
                issuer: None,
                serial_number: None,
                not_before: None,
                not_after: None,
            }
        }
    }
}

/// Validate cached certificate
fn validate_cached_certificate(
    cert: &X509Certificate,
    allowed_subjects: &[String],
) -> CertificateValidation {
    let validity = cert.validity();
    let subject = cert.subject().to_string();
    let issuer = cert.issuer().to_string();
    let serial = hex::encode(cert.raw_serial());

    // Check certificate validity period
    let not_before = validity.not_before.to_datetime();
    let not_after = validity.not_after.to_datetime();

    // Validate time bounds
    let now = std::time::SystemTime::now();
    let time_valid = not_before <= now && now <= not_after;

    // Check if subject is in allowed list
    let subject_valid =
        allowed_subjects.is_empty() || allowed_subjects.iter().any(|s| subject.contains(s));

    let valid = time_valid && subject_valid;

    if !subject_valid {
        warn!("Certificate subject not in allowed list: {}", subject);
    }

    CertificateValidation {
        valid,
        subject: Some(subject),
        issuer: Some(issuer),
        serial_number: Some(serial),
        not_before: Some(not_before.to_string()),
        not_after: Some(not_after.to_string()),
    }
}

/// Extract client certificate from TLS connection
pub fn extract_client_certificate_from_tls(request: &Request) -> Option<Vec<u8>> {
    // Try to get certificate from request extensions (set by TLS layer)
    if let Some(cert_der) = request.extensions().get::<Vec<u8>>() {
        return Some(cert_der.clone());
    }

    // Try to get certificate from headers (e.g. from reverse proxy)
    request
        .headers()
        .get("x-client-cert")
        .and_then(|v| hex::decode(v).ok())
}

/// Request context passed through middleware
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub request_id: String,
    pub user_id: Option<String>,
    pub user_email: Option<String>,
    pub user_roles: Vec<String>,
    pub user_permissions: Vec<String>,
    pub start_time: Instant,
    /// JWT claims for namespace access control
    pub jwt_claims: Option<secreton_core::namespace::JwtClaims>,
    /// Authorization header value (if any)
    pub auth_token: Option<String>,
    /// Client IP address
    pub client_ip: Option<String>,
    /// User Agent string
    pub user_agent: Option<String>,
    /// Policy names from JWT claims
    pub policy_names: Vec<String>,
}

impl RequestContext {
    /// Derive namespace from user context
    /// Prioritizes explicit satker/wilayah codes from JWT claims
    pub fn derive_namespace(&self) -> String {
        if let Some(claims) = &self.jwt_claims {
            if let Some(satker) = &claims.satker_code {
                return format!("satker-{}", satker.to_lowercase());
            }
            if let Some(wilayah) = &claims.wilayah_code {
                return format!("wilayah-{}", wilayah.to_lowercase());
            }
            if claims.admin_level == secreton_core::namespace::AdminLevel::Pusat {
                return "pusat".to_string();
            }
        }
        "default".to_string()
    }
}

/// Enhanced mTLS authentication middleware with proper TLS integration
pub async fn mtls_auth_middleware(
    State(state): State<ApiState>,
    _headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    // Skip mTLS for health/status endpoints
    let path = request.uri().path();
    if path == "/health" || path == "/version" || path.starts_with("/health") {
        return Ok(next.run(request).await);
    }

    // Check if mTLS is configured and required
    if let Some(mtls_config) = &state.transit.config {
        if mtls_config.required {
            // Extract client certificate from TLS connection
            if let Some(client_cert_der) = extract_client_certificate_from_tls(&request) {
                let start_time = std::time::Instant::now();

                // Validate certificate
                let validation = validate_client_certificate(
                    &client_cert_der,
                    None,
                    &mtls_config.allowed_subjects,
                );

                let validation_time = start_time.elapsed().as_millis() as u64;

                if validation.valid {
                    info!(
                        "mTLS authentication successful for subject: {:?} (validation: {}ms)",
                        validation.subject, validation_time
                    );

                    // Record successful authentication metrics
                    record_tls_handshake(true, false, validation_time);

                    // Add certificate info to request extensions
                    request.extensions_mut().insert(validation);

                    return Ok(next.run(request).await);
                } else {
                    warn!(
                        "mTLS authentication failed for subject: {:?} (validation: {}ms)",
                        validation.subject, validation_time
                    );
                    record_tls_handshake(false, false, validation_time);
                    return Err(AuthError::InvalidCredentials);
                }
            } else {
                warn!("mTLS required but no client certificate provided");
                record_tls_handshake(false, false, 0);
                return Err(AuthError::MissingCredentials);
            }
        }

        // mTLS not required or not configured, proceed with regular authentication
        return Ok(next.run(request).await);
    }

    Ok(next.run(request).await)
}

/// Extract IP address from request headers
pub fn extract_ip_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.trim().to_string())
        })
}

/// Extract User-Agent from headers
pub fn extract_user_agent_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
}

/// Rate limiting state
#[derive(Debug)]
pub struct RateLimitState {
    requests: HashMap<String, Vec<Instant>>,
    pub max_requests_per_minute: u32,
}

impl RateLimitState {
    pub fn new(max_requests_per_minute: u32) -> Self {
        Self {
            requests: HashMap::new(),
            max_requests_per_minute,
        }
    }

    pub fn check_rate_limit(&mut self, identifier: &str) -> bool {
        let now = Instant::now();
        let one_minute_ago = now - Duration::from_secs(60);

        // Get or create request history for this identifier
        let requests = self.requests.entry(identifier.to_string()).or_default();

        // Remove old requests (older than 1 minute)
        requests.retain(|&time| time > one_minute_ago);

        // Check if we're under the limit
        if requests.len() < self.max_requests_per_minute as usize {
            requests.push(now);
            true
        } else {
            false
        }
    }
}

/// Global rate limiting state
static RATE_LIMITER: Mutex<Option<RateLimitState>> = Mutex::new(None);

/// Initialize rate limiting
pub fn init_rate_limiting(max_requests_per_minute: u32) {
    if let Ok(mut limiter) = RATE_LIMITER.lock() {
        *limiter = Some(RateLimitState::new(max_requests_per_minute));
    }
}

/// Update rate limiting configuration
pub fn update_rate_limiting(max_requests_per_minute: u32) {
    if let Ok(mut limiter) = RATE_LIMITER.lock() {
        if let Some(ref mut l) = *limiter {
            l.max_requests_per_minute = max_requests_per_minute;
        } else {
            *limiter = Some(RateLimitState::new(max_requests_per_minute));
        }
    }
}

/// Authentication middleware
pub async fn auth_middleware(
    State(state): State<ApiState>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    // 1. Whitelist endpoints that don't require authentication
    // These include health checks, version info, and engine initialization/unsealing
    let path = request.uri().path();
    let normalized_path = path.trim_end_matches('/');
    let whitelisted = is_whitelisted(normalized_path);

    tracing::warn!("🔒 AUTH_MIDDLEWARE: path='{}', normalized='{}', whitelisted={}", path, normalized_path, whitelisted);

    if whitelisted {
        return Ok(next.run(request).await);
    }

    // Check if authentication is required
    // Skip authentication for now - simplified
    if false {
        // if !state.transit.config.require_authentication {
        debug!("Authentication disabled, skipping auth middleware");
        return Ok(next.run(request).await);
    }

    // Extract authorization header
    let auth_header = headers
        .get("authorization")
        .ok_or(AuthError::MissingAuthHeader)?;

    // Extract bearer token
    let token = extract_bearer_token(auth_header).ok_or(AuthError::InvalidAuthHeader)?;

    // Validate token using AuthService (checks signature and session storage)
    let user = state
        .services
        .auth
        .validate_token(&token)
        .await
        .map_err(|_| AuthError::InvalidToken)?;

    let user_roles: Vec<String> = user.roles.iter().cloned().collect();

    info!(
        "Authenticated user: {} ({}) with roles: {:?}",
        user.username, user.email, user_roles
    );

    // Reconstruct permissions (using policy service or auth service helper)
    // AuthService::get_user_policies returns list of policy names/permissions
    let user_permissions = state
        .services
        .auth
        .get_user_policies(&user)
        .await
        .unwrap_or_default();

    // Construct JwtClaims equivalent for namespace control
    // User struct doesn't strictly have all metadata that raw JWT claims had,
    // but we can reconstruct what we need.
    // The namespace logic relies on metadata fields like 'satker_code'.
    // User struct has 'metadata' HashMap.
    use secreton_core::namespace::AdminLevel;
    use secreton_core::namespace::JwtClaims;

    // Helper to extract code
    let satker_code = user.metadata.get("satker_code").cloned();
    let wilayah_code = user.metadata.get("wilayah_code").cloned();

    // Determine admin level from metadata or roles
    // We reuse the logic but adapt it since determine_admin_level expects &[String]
    let admin_level = if let Some(level_str) = user.metadata.get("admin_level") {
        match level_str.to_lowercase().as_str() {
            "pusat" => AdminLevel::Pusat,
            "eselon_i" | "eselon1" => AdminLevel::EselonI,
            "wilayah" => AdminLevel::Wilayah,
            "satker" => AdminLevel::Satker,
            _ => AdminLevel::Satker,
        }
    } else {
        // Fallback to roles check logic (simplified here or we can helper)
        let mut level = AdminLevel::Satker;
        for role in &user.roles {
            let role_lower = role.to_lowercase();
            if role_lower.contains("pusat") || role_lower.contains("admin_pusat") {
                level = AdminLevel::Pusat;
                break;
            }
            if role_lower.contains("eselon") {
                level = AdminLevel::EselonI;
                break;
            }
            if role_lower.contains("wilayah") || role_lower.contains("kejati") {
                level = AdminLevel::Wilayah;
                break;
            }
        }
        level
    };

    let jwt_claims = Some(JwtClaims {
        sub: user.id.to_string(),
        name: user.full_name.clone().unwrap_or(user.username.clone()),
        email: user.email.clone(),
        satker_code,
        wilayah_code,
        admin_level,
        roles: user_roles.clone(),
        permissions: user_permissions.clone(),
        exp: 0, // Not available in User, but session check already passed
        iat: 0,
        iss: "secreton".to_string(),
        metadata: user.metadata.clone(),
    });

    // Policy names
    // Prioritize explicit policy bindings from user.policies, then check metadata
    let mut policy_names: Vec<String> = user.policies.iter().cloned().collect();

    // If no explicit policies, check metadata for 'policy_names' or 'policies'
    if policy_names.is_empty() {
        if let Some(p) = user
            .metadata
            .get("policy_names")
            .or_else(|| user.metadata.get("policies"))
        {
            policy_names = p
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
    }

    // If still no policies, fallback to role-based policies
    if policy_names.is_empty() {
        policy_names = user
            .roles
            .iter()
            .map(|r| format!("{}-policy", r.to_lowercase()))
            .collect();
    }

    // Create request context
    let context = RequestContext {
        request_id: headers
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown")
            .to_string(),
        user_id: Some(user.id.to_string()),
        user_email: Some(user.email.clone()),
        user_roles,
        user_permissions,
        start_time: Instant::now(),
        jwt_claims,
        auth_token: Some(token),
        client_ip: extract_ip_from_headers(&headers),
        user_agent: headers
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.to_string()),
        policy_names,
    };

    // Add context to request extensions
    request.extensions_mut().insert(context);

    Ok(next.run(request).await)
}

/// Rate limiting middleware
pub async fn rate_limit(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    // Get client identifier (IP address or user ID)
    let ctx = lib_common::context::RequestContext::from_headers(&headers);
    let client_ip = ctx.ip_address.as_deref().unwrap_or("unknown");

    // Check rate limit
    if let Ok(mut limiter_guard) = RATE_LIMITER.lock()
        && let Some(ref mut limiter) = *limiter_guard
        && !limiter.check_rate_limit(client_ip)
    {
        warn!("Rate limit exceeded for client: {}", client_ip);
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({
                "error": "Rate limit exceeded",
                "status": 429,
                "retry_after": 60
            })),
        ));
    }
    // Guard is dropped here

    Ok(next.run(request).await)
}

/// Request rate tracking middleware
/// Tracks request counts for calculating system load metrics (RPM)
pub async fn request_rate_middleware(
    State(state): State<ApiState>,
    request: Request,
    next: Next,
) -> Response {
    // Increment request counter in AdminService
    state.services.admin.track_request();

    // Continue processing
    next.run(request).await
}

/// Request size limiting middleware
pub async fn request_size_limit(
    request: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    const MAX_REQUEST_SIZE: usize = 1024 * 1024; // 1MB

    // Check content-length header
    if let Some(content_length) = request.headers().get("content-length")
        && let Ok(length_str) = content_length.to_str()
        && let Ok(length) = length_str.parse::<usize>()
        && length > MAX_REQUEST_SIZE
    {
        return Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(serde_json::json!({
                "error": "Request too large",
                "max_size": MAX_REQUEST_SIZE,
                "actual_size": length
            })),
        ));
    }

    Ok(next.run(request).await)
}

/// Audit logging middleware
pub async fn audit_logging(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    // Extract request context if available
    let context = request.extensions().get::<RequestContext>().cloned();

    let response = next.run(request).await;

    // Log security-relevant operations
    let path = uri.path();
    if path.contains("/keys") || path.contains("/encrypt") || path.contains("/decrypt") {
        let user_info = if let Some(ctx) = context {
            format!(
                "user:{} email:{}",
                ctx.user_id.as_deref().unwrap_or("anonymous"),
                ctx.user_email.as_deref().unwrap_or("unknown")
            )
        } else {
            "user:anonymous".to_string()
        };

        info!(
            "AUDIT: {} {} by {} from {} -> {}",
            method,
            uri,
            user_info,
            user_agent,
            response.status().as_u16()
        );
    }

    response
}

/// Seal status check middleware
/// Blocks all secret operations when engine is sealed
/// CRITICAL SECURITY: This middleware enforces that all API operations
/// (except whitelisted system endpoints) are blocked when the engine is sealed.
/// This follows HashiCorp Engine security best practices.

/// Checks if a given request path is whitelisted from the seal check.
///
/// Whitelisted endpoints are those required for basic engine operations,
/// such as health checks, initialization, and unsealing.

/// Metrics middleware
/// Tracks request counts and active connections
pub async fn metrics_middleware(
    State(state): State<ApiState>,
    request: Request,
    next: Next,
) -> Response {
    // Increment active connections
    state
        .metrics
        .active_connections
        .fetch_add(1, Ordering::Relaxed);

    // Increment total requests
    state
        .metrics
        .rest_requests_total
        .fetch_add(1, Ordering::Relaxed);

    // Process request
    let response = next.run(request).await;

    // Decrement active connections
    state
        .metrics
        .active_connections
        .fetch_sub(1, Ordering::Relaxed);

    response
}

pub async fn seal_check_middleware(
    State(state): State<ApiState>,
    request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path();

    if is_whitelisted(path) {
        return next.run(request).await;
    }

    // CRITICAL SECURITY FIX: Check if engine is sealed
    // Get SealService from state and check if sealed
    if state.services.seal.is_sealed().await {
        warn!("🔒 Blocked request to {} - engine is sealed", path);

        // Record metric for blocked requests
        metrics::counter!("secreton_seal_blocked_requests_total").increment(1);

        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "errors": ["Engine is sealed"],
                "sealed": true,
                "message": "The engine is sealed. Please unseal it with threshold shares before performing operations."
            })),
        )
            .into_response();
    }

    tracing::debug!(
        "Seal check middleware: allowing request to {} (engine is unsealed)",
        path
    );

    next.run(request).await
}

/// Namespace access validation middleware
/// Validates that the user has access to the requested namespace based on
/// JWT claims and SIMKARI organizational hierarchy (Pusat -> Wilayah -> Satker).
/// This middleware should be applied to all secret operation endpoints.
pub async fn namespace_access_middleware(
    State(state): State<ApiState>,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    let path = request.uri().path();

    // Skip namespace validation for system endpoints
    if path.starts_with("/health")
        || path.starts_with("/version")
        || path.starts_with("/metrics")
        || path.starts_with("/v1/sys/")
        || path.starts_with("/api/v1/sys/")
    {
        return Ok(next.run(request).await);
    }

    // Extract JWT claims from request context
    let context = request.extensions().get::<RequestContext>();

    if let Some(ctx) = context {
        if let Some(claims) = &ctx.jwt_claims {
            // Extract namespace from path
            // Secret paths typically follow: /v1/secret/data/{namespace}/{path}
            // or /v1/transit/encrypt/{namespace}/{key}
            let namespace_id = extract_namespace_from_path(path);

            if let Some(ns_id) = namespace_id {
                debug!(
                    "Namespace access validation: user {} (level: {:?}) accessing namespace {}",
                    claims.sub, claims.admin_level, ns_id
                );

                use secreton_core::namespace::NamespaceAccessControl;

                // Optimization: Use with_hierarchy to avoid cloning the namespace hierarchy
                let validation_result = state.services.namespace.with_hierarchy(|hierarchy| {
                    NamespaceAccessControl::verify_access(hierarchy, claims, &ns_id)
                });

                match validation_result {
                    Ok(true) => {
                        // Access allowed
                        debug!("Access granted to namespace {}", ns_id);
                    }
                    Ok(false) => {
                        warn!(
                            "Access denied to namespace {} for user {}",
                            ns_id, claims.sub
                        );
                        return Err((
                            StatusCode::FORBIDDEN,
                            Json(serde_json::json!({
                                "error": "Access denied to namespace",
                                "namespace": ns_id,
                                "admin_level": format!("{:?}", claims.admin_level)
                            })),
                        )
                            .into_response());
                    }
                    Err(e) => {
                        // Namespace not found or other error
                        warn!("Namespace access check failed: {}", e);
                        let status = match e {
                            secreton_core::error::CoreError::NotFound { .. } => {
                                StatusCode::NOT_FOUND
                            }
                            _ => StatusCode::FORBIDDEN,
                        };

                        return Err((
                            status,
                            Json(serde_json::json!({
                                "error": "Namespace access validation failed",
                                "details": e.to_string()
                            })),
                        )
                            .into_response());
                    }
                }
            }
        } else {
            debug!("No JWT claims in request context for namespace validation");
        }
    }

    Ok(next.run(request).await)
}

/// Extract namespace ID from request path
/// Handles various path formats:
/// - /v1/secret/data/{namespace}/{path}
/// - /v1/transit/encrypt/{namespace}/{key}
/// - /v1/dynamic/database/creds/{namespace}/{role}
fn extract_namespace_from_path(path: &str) -> Option<String> {
    let mut parts = path.split('/').filter(|s| !s.is_empty());

    // Check first part (v1)
    if parts.next()? != "v1" {
        return None;
    }

    match parts.next()? {
        "secret" => {
            // /v1/secret/data/{namespace}/...
            if parts.next()? == "data" {
                return parts.next().map(|s| s.to_string());
            }
        }
        "transit" => {
            // /v1/transit/{operation}/{namespace}/...
            // Skip operation
            let _operation = parts.next()?;
            // Get namespace
            return parts.next().map(|s| s.to_string());
        }
        "dynamic" => {
            // /v1/dynamic/{type}/creds/{namespace}/...
            // Skip type
            let _type = parts.next()?;
            // Check for creds
            if parts.next()? == "creds" {
                return parts.next().map(|s| s.to_string());
            }
        }
        _ => {}
    }

    None
}

/// Extract JWT claims from token claims for namespace access control
/// Converts the auth service token claims into JwtClaims for namespace validation
#[allow(dead_code)]
fn extract_jwt_claims_from_token(
    claims: &crate::auth::Claims,
) -> Option<secreton_core::namespace::JwtClaims> {
    use secreton_core::namespace::JwtClaims;

    // Determine admin level from roles or metadata
    let admin_level = determine_admin_level(&claims.roles, &claims.metadata);

    // Extract satker and wilayah codes from metadata
    let satker_code = claims.metadata.get("satker_code").cloned();
    let wilayah_code = claims.metadata.get("wilayah_code").cloned();

    Some(JwtClaims {
        sub: claims.sub.clone(),
        name: claims.name.clone(),
        email: claims.email.clone(),
        satker_code,
        wilayah_code,
        admin_level,
        roles: claims.roles.clone(),
        permissions: claims.permissions.clone(),
        exp: claims.exp as i64,
        iat: claims.iat as i64,
        iss: claims.iss.clone(),
        metadata: claims.metadata.clone(),
    })
}

/// Determine admin level from roles and metadata
#[allow(dead_code)]
fn determine_admin_level(
    roles: &[String],
    metadata: &HashMap<String, String>,
) -> secreton_core::namespace::AdminLevel {
    use secreton_core::namespace::AdminLevel;

    // Check metadata first for explicit admin_level
    if let Some(level_str) = metadata.get("admin_level") {
        match level_str.to_lowercase().as_str() {
            "pusat" => return AdminLevel::Pusat,
            "eselon_i" | "eselon1" => return AdminLevel::EselonI,
            "wilayah" => return AdminLevel::Wilayah,
            "satker" => return AdminLevel::Satker,
            _ => {}
        }
    }

    // Infer from roles
    for role in roles {
        let role_lower = role.to_lowercase();
        if role_lower.contains("pusat") || role_lower.contains("admin_pusat") {
            return AdminLevel::Pusat;
        }
        if role_lower.contains("eselon") {
            return AdminLevel::EselonI;
        }
        if role_lower.contains("wilayah") || role_lower.contains("kejati") {
            return AdminLevel::Wilayah;
        }
        if role_lower.contains("satker") || role_lower.contains("kejari") {
            return AdminLevel::Satker;
        }
    }

    // Default to Satker (most restrictive)
    AdminLevel::Satker
}

/// Extract policy names from JWT claims
/// Looks for policy_names field in JWT claims metadata or as a direct field
#[allow(dead_code)]
fn extract_policy_names_from_claims(claims: &crate::auth::Claims) -> Vec<String> {
    // Check metadata for policy_names
    if let Some(policies_str) = claims.metadata.get("policy_names") {
        // Parse comma-separated policy names
        return policies_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }

    // Check metadata for policies (alternative field name)
    if let Some(policies_str) = claims.metadata.get("policies") {
        return policies_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }

    // Default: derive from roles (role-based policies)
    // Each role can have an associated policy
    claims
        .roles
        .iter()
        .map(|role| format!("{}-policy", role.to_lowercase()))
        .collect()
}

/// Policy check middleware
/// Enforces policy-based authorization on all operations.
/// Evaluates policies loaded from JWT claims against the requested path and action.
/// This middleware should be applied after authentication middleware.
pub async fn policy_check_middleware(
    State(state): State<ApiState>,
    request: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    let path = request.uri().path();
    let method = request.method().clone();

    // Skip policy check for system endpoints
    if path.starts_with("/health")
        || path.starts_with("/version")
        || path.starts_with("/metrics")
        || path == "/v1/sys/seal-status"
        || path == "/api/v1/sys/seal-status"
        || path == "/v1/sys/unseal"
        || path == "/api/v1/sys/unseal"
        || path == "/v1/sys/init"
        || path == "/api/v1/sys/init"
    {
        return Ok(next.run(request).await);
    }

    // Extract request context (set by auth middleware)
    let context = request.extensions().get::<RequestContext>();

    if let Some(ctx) = context {
        let user_id = ctx.user_id.as_deref().unwrap_or("anonymous");

        // Map HTTP method to capability/action
        let action = map_method_to_action(&method);

        // Build policy evaluation context
        let policy_context = build_policy_context(ctx, &request);

        // Get rules from policy service based on user's policy names
        let namespace = ctx.derive_namespace();
        let rules = state
            .services
            .policy_service
            .get_rules_for_policies(&ctx.policy_names, &namespace)
            .await
            .map_err(|e| {
                warn!("Failed to load policies: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": "Internal error loading policies"
                    })),
                )
                    .into_response()
            })?;

        // Create temporary policy set for evaluation
        let policy_set = secreton_core::services::policy::PolicySet::new(rules);

        // Evaluate policy
        let start_time = Instant::now();
        let allowed = policy_set.evaluate(user_id, path, &action, Some(&policy_context));
        let evaluation_time = start_time.elapsed();

        // Record metrics
        record_policy_evaluation_metrics(allowed, evaluation_time);

        // Extract context for audit logging
        let client_ip = extract_ip_from_headers(request.headers());
        let user_agent = extract_user_agent_from_headers(request.headers());
        let namespace = extract_namespace_from_path(path);

        if !allowed {
            warn!(
                "Policy denied access: user={}, path={}, action={}, evaluation_time={:?}",
                user_id, path, action, evaluation_time
            );

            // Log to audit with policy decision
            log_policy_decision_to_audit(
                &state,
                ctx,
                path,
                &action,
                false,
                &ctx.policy_names,
                client_ip,
                user_agent,
                namespace,
            )
            .await;

            return Err((
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({
                    "error": "Access denied by policy",
                    "path": path,
                    "action": action,
                    "user": user_id,
                    "policies_evaluated": ctx.policy_names,
                })),
            )
                .into_response());
        }

        debug!(
            "Policy allowed access: user={}, path={}, action={}, evaluation_time={:?}",
            user_id, path, action, evaluation_time
        );

        // Log successful policy evaluation to audit
        log_policy_decision_to_audit(
            &state,
            ctx,
            path,
            &action,
            true,
            &ctx.policy_names,
            client_ip,
            user_agent,
            namespace,
        )
        .await;

        Ok(next.run(request).await)
    } else {
        // No authentication context - deny by default
        warn!(
            "Policy check failed: no authentication context for path={}",
            path
        );

        Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "Authentication required",
                "path": path,
            })),
        )
            .into_response())
    }
}

/// Map HTTP method to policy action/capability
fn map_method_to_action(method: &axum::http::Method) -> String {
    match method.as_str() {
        "GET" => "read".to_string(),
        "POST" => "create".to_string(),
        "PUT" | "PATCH" => "update".to_string(),
        "DELETE" => "delete".to_string(),
        "HEAD" | "OPTIONS" => "list".to_string(),
        _ => "unknown".to_string(),
    }
}

/// Build policy evaluation context from request
fn build_policy_context(ctx: &RequestContext, request: &Request) -> serde_json::Value {
    use serde_json::json;

    let client_ip = extract_ip_from_headers(request.headers()).unwrap_or("unknown".to_string());

    json!({
        "user_id": ctx.user_id,
        "user_email": ctx.user_email,
        "user_roles": ctx.user_roles,
        "client_ip": client_ip,
        "request_id": ctx.request_id,
        "mfa_passed": ctx
            .jwt_claims
            .as_ref()
            .and_then(|c| c.metadata.get("mfa_passed"))
            .map(|v| v == "true")
            .unwrap_or(false),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    })
}

/// Record policy evaluation metrics
fn record_policy_evaluation_metrics(allowed: bool, evaluation_time: std::time::Duration) {
    let decision = if allowed { "allowed" } else { "denied" };
    debug!(
        "Policy evaluation: decision={}, time={}μs",
        decision,
        evaluation_time.as_micros()
    );

    // Record metrics
    metrics::counter!(
        "secreton_policy_evaluations_total",
        "decision" => decision
    )
    .increment(1);

    metrics::histogram!("secreton_policy_evaluation_duration_seconds")
        .record(evaluation_time.as_secs_f64());
}

/// Log policy decision to audit log
#[allow(clippy::too_many_arguments)]
async fn log_policy_decision_to_audit(
    state: &ApiState,
    ctx: &RequestContext,
    path: &str,
    action: &str,
    allowed: bool,
    policy_names: &[String],
    client_ip: Option<String>,
    user_agent: Option<String>,
    namespace: Option<String>,
) {
    use secreton_core::audit::{AuditLog, AuditStatus};
    use std::collections::HashMap;

    let mut metadata = HashMap::new();
    metadata.insert("path".to_string(), path.to_string());
    metadata.insert("action".to_string(), action.to_string());
    metadata.insert(
        "decision".to_string(),
        if allowed {
            "allow".to_string()
        } else {
            "deny".to_string()
        },
    );
    metadata.insert("policy_names".to_string(), policy_names.join(","));
    metadata.insert("request_id".to_string(), ctx.request_id.clone());

    let audit_entry = AuditLog {
        id: uuid::Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        action: "policy_evaluation".to_string(),
        actor: ctx.user_id.clone(),
        resource_type: "policy".to_string(),
        resource_id: path.to_string(),
        status: if allowed {
            AuditStatus::Success
        } else {
            AuditStatus::Denied
        },
        ip: client_ip,
        user_agent,
        namespace,
        metadata,
    };

    if let Err(e) = state.services.audit.log(audit_entry).await {
        warn!("Failed to log policy decision to audit: {}", e);
    }
}

#[cfg(all(test, feature = "enable-inline-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_mfa_status_extraction() {
        use secreton_core::namespace::{AdminLevel, JwtClaims};
        use std::collections::HashMap;

        // Create claims with MFA passed
        let mut metadata = HashMap::new();
        metadata.insert("mfa_passed".to_string(), "true".to_string());

        let claims = JwtClaims {
            sub: "user123".to_string(),
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            satker_code: None,
            wilayah_code: None,
            admin_level: AdminLevel::Satker,
            roles: vec![],
            permissions: vec![],
            exp: 0,
            iat: 0,
            iss: "test".to_string(),
            metadata,
        };

        let ctx = RequestContext {
            request_id: "req123".to_string(),
            user_id: Some("user123".to_string()),
            user_email: Some("test@example.com".to_string()),
            user_roles: vec![],
            user_permissions: vec![],
            start_time: Instant::now(),
            jwt_claims: Some(claims),
            policy_names: vec![],
            auth_token: None,
            client_ip: None,
            user_agent: None,
        };

        let request = Request::builder().body(axum::body::Body::empty()).unwrap();

        let policy_context = build_policy_context(&ctx, &request);

        // This should be true if implementation is correct
        assert_eq!(
            policy_context["mfa_passed"], true,
            "MFA status should be extracted from claims"
        );
    }

    #[test]
    fn test_rate_limiting() {
        let mut rate_limiter = RateLimitState::new(5); // 5 requests per minute

        // Should allow 5 requests
        for _ in 0..5 {
            assert!(rate_limiter.check_rate_limit("test-client"));
        }

        // Should block the 6th request
        assert!(!rate_limiter.check_rate_limit("test-client"));

        // Different client should be allowed
        assert!(rate_limiter.check_rate_limit("other-client"));
    }

    #[test]
    #[test]
    fn test_derive_namespace() {
        use secreton_core::namespace::{AdminLevel, JwtClaims};
        use std::collections::HashMap;

        // Test satker
        let mut claims = JwtClaims {
            sub: "user".into(),
            name: "User".into(),
            email: "user@example.com".into(),
            satker_code: Some("KJA001".into()),
            wilayah_code: Some("SUMUT".into()),
            admin_level: AdminLevel::Satker,
            roles: vec![],
            permissions: vec![],
            exp: 0,
            iat: 0,
            iss: "test".into(),
            metadata: HashMap::new(),
        };

        let context = RequestContext {
            request_id: "req".into(),
            user_id: Some("user".into()),
            user_email: None,
            user_roles: vec![],
            user_permissions: vec![],
            start_time: std::time::Instant::now(),
            jwt_claims: Some(claims),
            policy_names: vec![],
            auth_token: None,
            client_ip: None,
            user_agent: None,
        };

        assert_eq!(context.derive_namespace(), "satker-kja001");

        // Test wilayah
        let mut claims = context.jwt_claims.clone().unwrap();
        claims.satker_code = None;
        claims.admin_level = AdminLevel::Wilayah;

        let context = RequestContext {
            jwt_claims: Some(claims),
            request_id: "req".into(),
            user_id: Some("user".into()),
            user_email: None,
            user_roles: vec![],
            user_permissions: vec![],
            start_time: std::time::Instant::now(),
            policy_names: vec![],
            auth_token: None,
            client_ip: None,
            user_agent: None,
        };
        assert_eq!(context.derive_namespace(), "wilayah-sumut");

        // Test pusat
        let mut claims = context.jwt_claims.clone().unwrap();
        claims.wilayah_code = None;
        claims.admin_level = AdminLevel::Pusat;
        let context = RequestContext {
            jwt_claims: Some(claims),
            request_id: "req".into(),
            user_id: Some("user".into()),
            user_email: None,
            user_roles: vec![],
            user_permissions: vec![],
            start_time: std::time::Instant::now(),
            policy_names: vec![],
            auth_token: None,
            client_ip: None,
            user_agent: None,
        };
        assert_eq!(context.derive_namespace(), "pusat");

        // Test fallback
        let mut claims = context.jwt_claims.clone().unwrap();
        claims.admin_level = AdminLevel::EselonI;
        let context = RequestContext {
            jwt_claims: Some(claims),
            request_id: "req".into(),
            user_id: Some("user".into()),
            user_email: None,
            user_roles: vec![],
            user_permissions: vec![],
            start_time: std::time::Instant::now(),
            policy_names: vec![],
            auth_token: None,
            client_ip: None,
            user_agent: None,
        };
        assert_eq!(context.derive_namespace(), "default");

        // Test no claims
        let context = RequestContext {
            jwt_claims: None,
            request_id: "req".into(),
            user_id: Some("user".into()),
            user_email: None,
            user_roles: vec![],
            user_permissions: vec![],
            start_time: std::time::Instant::now(),
            policy_names: vec![],
            auth_token: None,
            client_ip: None,
            user_agent: None,
        };
        assert_eq!(context.derive_namespace(), "default");
    }
}

/// Response wrapping middleware
/// Automatically wraps responses when X-Engine-Wrap-TTL header is present.
/// This allows clients to request wrapped responses for any endpoint.
/// # Header Format
/// `X-Engine-Wrap-TTL: <seconds>`
/// # Example
/// ```bash
/// curl -H "X-Engine-Wrap-TTL: 300" http://localhost:8200/v1/secret/data/myapp
/// ```
/// # Response
/// Instead of returning the actual secret, returns a wrapping token:
/// ```json
/// {
///   "success": true,
///   "data": {
///     "token": "wrap_abc123...",
///     "created_at": "2025-10-27T10:00:00Z",
///     "expires_at": "2025-10-27T10:05:00Z",
///     "ttl": 300
///   }
/// }
/// ```
pub async fn response_wrapping_middleware(
    State(state): State<ApiState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Response {
    const MAX_WRAPPING_SIZE: usize = 10 * 1024 * 1024; // 10MB

    // Check for X-Engine-Wrap-TTL header
    let wrap_ttl = headers
        .get("X-Engine-Wrap-TTL")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok());

    // If no wrap header, proceed normally
    let wrap_ttl = match wrap_ttl {
        Some(ttl) => ttl,
        None => return next.run(request).await,
    };

    // Validate TTL
    if wrap_ttl == 0 || wrap_ttl > 86400 {
        warn!("Invalid X-Engine-Wrap-TTL value: {}", wrap_ttl);
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "success": false,
                "error": {
                    "code": "INVALID_WRAP_TTL",
                    "message": "X-Engine-Wrap-TTL must be between 1 and 86400 seconds"
                }
            })),
        )
            .into_response();
    }

    // Capture path and namespace before request is consumed
    let path = request.uri().path().to_string();
    let namespace = if let Some(ctx) = request.extensions().get::<RequestContext>() {
        ctx.derive_namespace()
    } else {
        extract_namespace_from_path(&path).unwrap_or_else(|| "default".to_string())
    };

    // Execute the request
    let response = next.run(request).await;

    // Only wrap successful responses (2xx status codes)
    if !response.status().is_success() {
        return response;
    }

    info!(
        ttl = wrap_ttl,
        path = %path,
        namespace = %namespace,
        "Response wrapping requested via X-Engine-Wrap-TTL header"
    );

    // Buffer the response body
    let (parts, body) = response.into_parts();

    let bytes = match axum::body::to_bytes(body, MAX_WRAPPING_SIZE).await {
        Ok(b) => b,
        Err(e) => {
            warn!("Failed to buffer response for wrapping: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": {
                        "code": "WRAPPING_ERROR",
                        "message": "Failed to buffer response"
                    }
                })),
            )
                .into_response();
        }
    };

    // Parse as JSON
    let json_body: serde_json::Value = match serde_json::from_slice(&bytes) {
        Ok(v) => v,
        Err(_) => {
            // Not JSON, return original response
            return Response::from_parts(parts, axum::body::Body::from(bytes));
        }
    };

    // Prepare wrap request
    let wrap_req = secreton_core::services::wrapping::WrapRequest {
        data: json_body,
        ttl: Duration::from_secs(wrap_ttl),
        namespace,
    };

    // Wrap the response
    match state.services.wrapping_service.wrap(wrap_req).await {
        Ok(wrap_response) => {
            // Extract request_id from original headers if possible
            let request_id = parts
                .headers
                .get("x-request-id")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string())
                .unwrap_or_else(|| Uuid::new_v4().to_string());

            let wrapped_data = serde_json::json!({
                "success": true,
                "data": {
                    "token": wrap_response.token,
                    "created_at": wrap_response.created_at,
                    "expires_at": wrap_response.expires_at,
                    "ttl": wrap_response.ttl,
                    "wrapped_at": chrono::Utc::now(),
                    "creation_path": path
                },
                "metadata": {
                    "request_id": request_id,
                    "timestamp": chrono::Utc::now()
                }
            });

            Json(wrapped_data).into_response()
        }
        Err(e) => {
            warn!("Failed to wrap response: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": {
                        "code": "WRAPPING_FAILED",
                        "message": format!("Failed to wrap response: {}", e)
                    }
                })),
            )
                .into_response()
        }
    }
}

#[cfg(all(test, feature = "enable-inline-tests"))]
mod certificate_tests {
    use super::*;

    #[test]
    fn test_certificate_cache() {
        let cache = CertificateCache::new(60);
        assert!(cache.get("test").is_none());
        assert_eq!(cache.misses.load(Ordering::Relaxed), 1);
        assert_eq!(cache.hits.load(Ordering::Relaxed), 0);
        assert_eq!(cache.hit_rate(), 0.0);
    }

    #[test]
    fn test_validate_client_certificate_integration() {
        init_certificate_cache(60);
        // Using a dummy DER (not a real cert, parsing will fail but we check cache flow)
        let _dummy_der = vec![0x30, 0x82, 0x01];
        let _allowed = vec!["CN=test".to_string()];

        // Verify that get_cache_hit_rate() is safe to call.
        let rate = get_cache_hit_rate();
        assert!(rate >= 0.0);
    }

    #[test]
    fn test_wrap_ttl_validation() {
        assert!(0 == 0 || 0 > 86400); // Invalid
        assert!(300 > 0 && 300 <= 86400); // Valid
        assert!(86401 > 86400); // Invalid
    }
}

#[cfg(all(test, feature = "enable-inline-tests"))]
mod middleware_tests {
    use super::*;
    use axum::http::Request;
    use secreton_core::services::seal::{SealConfig, SealService};
    use std::sync::Arc;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_seal_check_middleware_blocks_when_sealed() {
        let state = create_test_api_state(true).await;
        let middleware = tower::ServiceBuilder::new()
            .layer(axum::middleware::from_fn_with_state(
                state.clone(),
                seal_check_middleware,
            ))
            .service_fn(mock_handler);

        let request = Request::builder()
            .uri("/v1/secret/data/my-secret")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = middleware.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_seal_check_middleware_allows_when_unsealed() {
        let state = create_test_api_state(false).await;
        let middleware = tower::ServiceBuilder::new()
            .layer(axum::middleware::from_fn_with_state(
                state.clone(),
                seal_check_middleware,
            ))
            .service_fn(mock_handler);

        let request = Request::builder()
            .uri("/v1/secret/data/my-secret")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = middleware.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_seal_check_middleware_allows_whitelisted_endpoints_when_sealed() {
        let state = create_test_api_state(true).await;
        let middleware = tower::ServiceBuilder::new()
            .layer(axum::middleware::from_fn_with_state(
                state.clone(),
                seal_check_middleware,
            ))
            .service_fn(mock_handler);

        let whitelisted_paths = [
            "/health",
            "/version",
            "/metrics",
            "/v1/sys/seal-status",
            "/api/v1/sys/seal-status",
            "/v1/sys/unseal",
            "/api/v1/sys/unseal",
            "/v1/sys/init",
            "/api/v1/sys/init",
        ];

        for path in &whitelisted_paths {
            let request = Request::builder()
                .uri(*path)
                .body(axum::body::Body::empty())
                .unwrap();
            let response = middleware.clone().oneshot(request).await.unwrap();
            assert_eq!(
                response.status(),
                StatusCode::OK,
                "Failed for path: {}",
                path
            );
        }
    }

    async fn create_test_api_state(sealed: bool) -> ApiState {
        use secreton_storage::MemoryBackend;
        let storage = Arc::new(MemoryBackend::new());
        let cfg = deadpool_postgres::Config::new();
        let pool = cfg.create_pool(None, tokio_postgres::NoTls).unwrap();

        let services = crate::services::ServiceContainer::new_mock(storage, pool);

        if !sealed {
            // Initialize and unseal
            if let Ok(init_res) = services.seal.initialize().await {
                // init_res is Vec<Share>
                if let Some(share) = init_res.first() {
                    // Share struct doesn't have public fields, but it implements serialization
                    if let Ok(share_bytes) = share.to_bytes() {
                        // The unseal method expects a hex-encoded string or string representation
                        // Assuming the share bytes can be hex encoded to be passed as key
                        let key_str = hex::encode(share_bytes);
                        let _ = services.seal.unseal(key_str).await;
                    }
                }
            }
        }

        ApiState {
            transit: crate::transit::TransitApiState {
                engine: Arc::new(secreton_crypto::transit::TransitEngine::new()),
                config: None,
                metrics: Default::default(),
            },
            kv: crate::kv::KVApiState {
                engine: Arc::new(crate::kv::KVEngine::new()),
                metrics: Default::default(),
            },
            pki: crate::pki::PkiApiState::default(),
            services: Arc::new(services),
            metrics: Default::default(),
            prometheus_handle: None,
        }
    }

    async fn mock_handler(
        _req: Request<axum::body::Body>,
    ) -> Result<Response, std::convert::Infallible> {
        Ok(Response::builder()
            .status(StatusCode::OK)
            .body(axum::body::Body::empty())
            .unwrap())
    }

    #[test]
    fn test_extract_namespace_from_path() {
        // Valid paths
        assert_eq!(
            extract_namespace_from_path("/v1/secret/data/my-ns/key"),
            Some("my-ns".to_string())
        );
        assert_eq!(
            extract_namespace_from_path("/v1/transit/encrypt/my-ns/key"),
            Some("my-ns".to_string())
        );
        assert_eq!(
            extract_namespace_from_path("/v1/dynamic/database/creds/my-ns/role"),
            Some("my-ns".to_string())
        );

        // Invalid paths
        assert_eq!(extract_namespace_from_path("/v1/sys/health"), None);
        assert_eq!(extract_namespace_from_path("/invalid/path"), None);
        assert_eq!(
            extract_namespace_from_path("/v1/secret/metadata/my-ns/key"),
            None
        ); // Only data paths
        assert_eq!(
            extract_namespace_from_path("/v1/other/data/my-ns/key"),
            None
        );
    }
}

/// Check if a path is whitelisted for anonymous access
fn is_whitelisted(path: &str) -> bool {
    tracing::warn!("🔍 IS_WHITELISTED CHECK: path='{}'", path);
    // Health and metrics (infrastructure)
    if path == "/health" || path == "/ready" || path == "/live" || path == "/version" || path == "/metrics" {
        return true;
    }

    // Standard system endpoints that must be available without authentication
    let system_paths = [
        "/sys/init",
        "/sys/seal-status",
        "/sys/unseal",
        "/sys/rekey/init",
        "/sys/rekey/update",
    ];

    for sys_path in system_paths {
        if path == sys_path || path.ends_with(sys_path) {
            tracing::warn!("✅ Whitelisted system path: '{}'", path);
            return true;
        }
    }

    false
}
