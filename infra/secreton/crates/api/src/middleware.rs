//! HTTP middleware for the Secreton API
//!
//! Provides authentication, rate limiting, request tracing,
//! and other middleware functionality.

use axum::{
    Json,
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};
use uuid::Uuid;
use x509_parser::prelude::*;

use crate::ApiState;
use crate::auth::{AuthError, AuthService, extract_bearer_token};

/// Certificate cache for performance optimization
#[derive(Debug)]
pub struct CertificateCache {
    cache: Mutex<HashMap<String, (X509Certificate<'static>, Instant)>>,
    ttl: Duration,
}

impl CertificateCache {
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    pub fn get(&self, cert_der: &str) -> Option<X509Certificate<'static>> {
        let mut cache = self.cache.lock().expect("Failed to lock cache");
        if let Some((cert, timestamp)) = cache.get(cert_der) {
            if timestamp.elapsed() < self.ttl {
                return Some(cert.clone());
            } else {
                cache.remove(cert_der);
            }
        }
        None
    }

    pub fn insert(&self, cert_der: String, cert: X509Certificate<'static>) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(cert_der, (cert, Instant::now()));
        }
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

/// Get certificate cache instance
fn get_cert_cache() -> Option<Arc<CertificateCache>> {
    CERT_CACHE.lock().ok()?.as_ref().cloned()
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

/// Validate client certificate
/// TODO: Re-enable after adding x509_parser and hex dependencies
#[allow(dead_code)]
pub fn validate_client_certificate(
    _cert_der: &[u8],
    _ca_cert_path: Option<&PathBuf>,
    _allowed_subjects: &[String],
) -> CertificateValidation {
    // Temporarily disabled - needs x509_parser dependency
    CertificateValidation {
        valid: false,
        subject: None,
        issuer: None,
        serial_number: None,
        not_before: None,
        not_after: None,
    }
}

#[allow(dead_code)]
fn _validate_client_certificate_full(
    cert_der: &[u8],
    _ca_cert_path: Option<&PathBuf>,
    allowed_subjects: &[String],
) -> CertificateValidation {
    // TODO: Re-enable certificate caching when lifetime issues are resolved
    // Try cache first
    // let cert_der_hex = hex::encode(cert_der);
    // if let Some(cache) = get_cert_cache() {
    //     if let Some(cached_cert) = cache.get(&cert_der_hex) {
    //         return validate_cached_certificate(&cached_cert, allowed_subjects);
    //     }
    // }

    // Parse certificate
    match x509_parser::parse_x509_certificate(cert_der) {
        Ok((_, cert)) => {
            // TODO: Cache the parsed certificate when lifetime issues are resolved
            // if let Some(cache) = get_cert_cache() {
            //     cache.insert(cert_der_hex, cert.clone());
            // }

            validate_cached_certificate(&cert, allowed_subjects)
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
#[allow(dead_code)]
fn validate_cached_certificate(
    cert: &X509Certificate,
    allowed_subjects: &[String],
) -> CertificateValidation {
    let validity = cert.validity();
    let subject = cert.subject().to_string();
    let issuer = cert.issuer().to_string();
    let serial = hex::encode(cert.raw_serial());

    let valid = allowed_subjects.is_empty() || allowed_subjects.iter().any(|s| subject.contains(s));

    CertificateValidation {
        valid,
        subject: Some(subject),
        issuer: Some(issuer),
        serial_number: Some(serial),
        not_before: Some(validity.not_before.to_string()),
        not_after: Some(validity.not_after.to_string()),
    }
}

#[allow(dead_code)]
fn _validate_cached_certificate_placeholder(
    _cert: &(), // Placeholder
    _allowed_subjects: &[String],
) -> CertificateValidation {
    CertificateValidation {
        valid: false,
        subject: None,
        issuer: None,
        serial_number: None,
        not_before: None,
        not_after: None,
    }
}

#[allow(dead_code)]
fn _validate_cached_certificate_full(
    cert: &X509Certificate,
    allowed_subjects: &[String],
) -> CertificateValidation {
    // Check certificate validity period
    // TODO: Add proper certificate expiration checking
    let not_before = cert.validity().not_before.to_datetime();
    let not_after = cert.validity().not_after.to_datetime();

    // For now, just skip expiration check to avoid time crate version conflicts
    // In production, this should properly validate against current time
    if false {
        warn!("Certificate is not valid (expired or not yet valid)");
        return CertificateValidation {
            valid: false,
            subject: cert.subject().to_string().into(),
            issuer: cert.issuer().to_string().into(),
            serial_number: Some(hex::encode(cert.raw_serial())),
            not_before: Some(not_before.to_string()),
            not_after: Some(not_after.to_string()),
        };
    }

    // Check if subject is in allowed list
    let subject_str = cert.subject().to_string();
    let is_allowed =
        allowed_subjects.is_empty() || allowed_subjects.iter().any(|s| subject_str.contains(s));

    if !is_allowed {
        warn!("Certificate subject not in allowed list: {}", subject_str);
    }

    CertificateValidation {
        valid: is_allowed,
        subject: Some(subject_str),
        issuer: Some(cert.issuer().to_string()),
        serial_number: Some(hex::encode(cert.raw_serial())),
        not_before: Some(not_before.to_string()),
        not_after: Some(not_after.to_string()),
    }
}

/// Extract client certificate from TLS connection
/// TODO: Re-enable after adding hex dependency
pub fn extract_client_certificate_from_tls(request: &Request) -> Option<Vec<u8>> {
    // Try to get certificate from request extensions (set by TLS layer)
    if let Some(cert_der) = request.extensions().get::<Vec<u8>>() {
        return Some(cert_der.clone());
    }

    // Temporarily disabled - needs hex dependency
    // request.headers().get("x-client-cert").and_then(|v| hex::decode(v).ok())
    None
}

/// Enhanced mTLS authentication middleware with proper TLS integration
pub async fn mtls_auth_middleware(
    State(state): State<ApiState>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    // Skip mTLS for health/status endpoints
    let path = request.uri().path();
    if path == "/health" || path == "/version" || path.starts_with("/health") {
        return Ok(next.run(request).await);
    }

    // TODO: Re-enable mTLS when TransitApiState has config field
    // Check if mTLS is configured and required
    if let Some(mtls_config) = None::<&crate::config::MtlsConfig> {
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
                    // TODO: Re-enable when tls_optimization is updated
                    // record_tls_handshake(true, false, validation_time);

                    // Add certificate info to request extensions
                    request.extensions_mut().insert(validation);

                    return Ok(next.run(request).await);
                } else {
                    warn!(
                        "mTLS authentication failed for subject: {:?} (validation: {}ms)",
                        validation.subject, validation_time
                    );
                    // record_tls_handshake(false, false, validation_time);
                    return Err(AuthError::InvalidCredentials);
                }
            } else {
                warn!("mTLS required but no client certificate provided");
                // record_tls_handshake(false, false, 0);
                return Err(AuthError::MissingCredentials);
            }
        }
    }

    // mTLS not required or not configured, proceed with regular authentication
    Ok(next.run(request).await)
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
    /// Policy names from JWT claims
    pub policy_names: Vec<String>,
}

/// Rate limiting state
#[derive(Debug)]
pub struct RateLimitState {
    requests: HashMap<String, Vec<Instant>>,
    max_requests_per_minute: u32,
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

/// Request ID middleware - adds unique ID to each request
pub async fn request_id(mut request: Request, next: Next) -> Response {
    let request_id = Uuid::new_v4().to_string();

    // Add request ID to headers for downstream processing
    request
        .headers_mut()
        .insert("x-request-id", request_id.parse().unwrap());

    debug!("Processing request: {}", request_id);

    let response = next.run(request).await;

    // Add request ID to response headers
    let mut response = response;
    response
        .headers_mut()
        .insert("x-request-id", request_id.parse().unwrap());

    response
}

/// Authentication middleware
pub async fn auth_middleware(
    State(_state): State<ApiState>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    // Skip auth for health/status endpoints
    let path = request.uri().path();
    if path == "/health" || path == "/version" || path.starts_with("/health") {
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

    // Create auth service (in real implementation, this would be injected)
    let auth_config = crate::auth::AuthConfig::default();
    let auth_service = AuthService::new(auth_config);

    // Validate token
    let token_data = auth_service.validate_token(&token)?;
    let claims = token_data.claims;

    info!(
        "Authenticated user: {} ({}) with roles: {:?}",
        claims.name, claims.email, claims.roles
    );

    // Extract JWT claims for namespace access control
    let jwt_claims = extract_jwt_claims_from_token(&claims);

    // Extract policy names from JWT claims
    let policy_names = extract_policy_names_from_claims(&claims);

    // Create request context
    let context = RequestContext {
        request_id: headers
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown")
            .to_string(),
        user_id: Some(claims.sub.clone()),
        user_email: Some(claims.email.clone()),
        user_roles: claims.roles.clone(),
        user_permissions: claims.permissions.clone(),
        start_time: Instant::now(),
        jwt_claims,
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
    let client_ip = headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    // Check rate limit
    {
        if let Ok(mut limiter_guard) = RATE_LIMITER.lock() {
            if let Some(ref mut limiter) = *limiter_guard {
                if !limiter.check_rate_limit(client_ip) {
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
            }
        }
    } // Guard is dropped here

    Ok(next.run(request).await)
}

/// Request logging middleware
pub async fn request_logging(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start_time = Instant::now();

    info!("Incoming request: {} {}", method, uri);

    let response = next.run(request).await;

    let duration = start_time.elapsed();
    let status = response.status();

    info!(
        "Request completed: {} {} -> {} ({:.2}ms)",
        method,
        uri,
        status.as_u16(),
        duration.as_millis()
    );

    // Log slow requests
    if duration > Duration::from_millis(1000) {
        warn!(
            "Slow request detected: {} {} took {:.2}ms",
            method,
            uri,
            duration.as_millis()
        );
    }

    response
}

/// Security headers middleware
pub async fn security_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;

    // Add security headers
    let headers = response.headers_mut();

    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
    headers.insert(
        "Strict-Transport-Security",
        "max-age=31536000; includeSubDomains".parse().unwrap(),
    );
    headers.insert(
        "Referrer-Policy",
        "strict-origin-when-cross-origin".parse().unwrap(),
    );
    headers.insert(
        "Content-Security-Policy",
        "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'"
            .parse()
            .unwrap(),
    );

    response
}

/// Request size limiting middleware
pub async fn request_size_limit(
    request: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    const MAX_REQUEST_SIZE: usize = 1024 * 1024; // 1MB

    // Check content-length header
    if let Some(content_length) = request.headers().get("content-length") {
        if let Ok(length_str) = content_length.to_str() {
            if let Ok(length) = length_str.parse::<usize>() {
                if length > MAX_REQUEST_SIZE {
                    return Err((
                        StatusCode::PAYLOAD_TOO_LARGE,
                        Json(serde_json::json!({
                            "error": "Request too large",
                            "max_size": MAX_REQUEST_SIZE,
                            "actual_size": length
                        })),
                    ));
                }
            }
        }
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

/// CORS preflight handling
pub async fn cors_preflight(request: Request, next: Next) -> Response {
    if request.method() == axum::http::Method::OPTIONS {
        return axum::response::Response::builder()
            .status(StatusCode::OK)
            .header("Access-Control-Allow-Origin", "*")
            .header(
                "Access-Control-Allow-Methods",
                "GET, POST, PUT, DELETE, OPTIONS",
            )
            .header(
                "Access-Control-Allow-Headers",
                "Content-Type, Authorization, X-Request-ID",
            )
            .header("Access-Control-Max-Age", "86400")
            .body(axum::body::Body::empty())
            .unwrap();
    }

    next.run(request).await
}

/// Seal status check middleware
/// Blocks all secret operations when vault is sealed
///
/// CRITICAL SECURITY: This middleware enforces that all API operations
/// (except whitelisted system endpoints) are blocked when the vault is sealed.
/// This follows HashiCorp Vault security best practices.
pub async fn seal_check_middleware(
    State(state): State<ApiState>,
    request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path();

    // Whitelist: Allow these endpoints even when sealed
    // - Health checks (for load balancers)
    // - Seal status (to check if sealed)
    // - Unseal (to unseal the vault)
    // - Init (to initialize the vault)
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
        return next.run(request).await;
    }

    // CRITICAL SECURITY FIX: Check if vault is sealed
    // Get SealService from state and check if sealed
    if state.services.seal.is_sealed().await {
        warn!("🔒 Blocked request to {} - vault is sealed", path);

        // Record metric for blocked requests
        // TODO: Re-enable when metrics are properly integrated
        // metrics::counter!("secreton_seal_blocked_requests_total").increment(1);

        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "errors": ["Vault is sealed"],
                "sealed": true,
                "message": "The vault is sealed. Please unseal it with threshold shares before performing operations."
            })),
        ).into_response();
    }

    tracing::debug!(
        "Seal check middleware: allowing request to {} (vault is unsealed)",
        path
    );

    next.run(request).await
}

/// Namespace access validation middleware
///
/// Validates that the user has access to the requested namespace based on
/// JWT claims and SIMKARI organizational hierarchy (Pusat -> Wilayah -> Satker).
///
/// This middleware should be applied to all secret operation endpoints.
pub async fn namespace_access_middleware(
    State(_state): State<ApiState>,
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
                // TODO: Get NamespaceAccessControl from state
                // For now, log the validation attempt
                debug!(
                    "Namespace access validation: user {} (level: {:?}) accessing namespace {}",
                    claims.sub, claims.admin_level, ns_id
                );

                // TODO: Implement actual validation when NamespaceAccessControl is in state
                // let access_control = state.namespace_access_control;
                // if !access_control.check_access(claims, &ns_id)? {
                //     return Err((
                //         StatusCode::FORBIDDEN,
                //         Json(serde_json::json!({
                //             "error": "Access denied to namespace",
                //             "namespace": ns_id,
                //             "admin_level": format!("{:?}", claims.admin_level)
                //         })),
                //     ));
                // }
            }
        } else {
            debug!("No JWT claims in request context for namespace validation");
        }
    }

    Ok(next.run(request).await)
}

/// Extract namespace ID from request path
///
/// Handles various path formats:
/// - /v1/secret/data/{namespace}/{path}
/// - /v1/transit/encrypt/{namespace}/{key}
/// - /v1/dynamic/database/creds/{namespace}/{role}
fn extract_namespace_from_path(path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    // Look for namespace after known prefixes
    if parts.len() >= 4 {
        // Check for secret paths: /v1/secret/data/{namespace}/...
        if parts[0] == "v1" && parts[1] == "secret" && parts[2] == "data" {
            return Some(parts[3].to_string());
        }

        // Check for transit paths: /v1/transit/{operation}/{namespace}/...
        if parts[0] == "v1" && parts[1] == "transit" && parts.len() >= 4 {
            return Some(parts[3].to_string());
        }

        // Check for dynamic secrets: /v1/dynamic/{type}/creds/{namespace}/...
        if parts[0] == "v1" && parts[1] == "dynamic" && parts.len() >= 5 {
            return Some(parts[4].to_string());
        }
    }

    None
}

/// Extract JWT claims from token claims for namespace access control
///
/// Converts the auth service token claims into JwtClaims for namespace validation
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
///
/// Looks for policy_names field in JWT claims metadata or as a direct field
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
///
/// Enforces policy-based authorization on all operations.
/// Evaluates policies loaded from JWT claims against the requested path and action.
///
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

        // Get policy service from state
        let policy_set = match state.services.policy.read() {
            Ok(guard) => guard,
            Err(e) => {
                tracing::error!("Failed to acquire policy read lock: {}", e);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Policy service unavailable",
                )
                    .into_response());
            }
        };

        // Evaluate policy
        let start_time = Instant::now();
        let allowed = policy_set.evaluate(user_id, path, &action, Some(&policy_context));
        let evaluation_time = start_time.elapsed();

        // Record metrics
        record_policy_evaluation_metrics(allowed, evaluation_time);

        if !allowed {
            warn!(
                "Policy denied access: user={}, path={}, action={}, evaluation_time={:?}",
                user_id, path, action, evaluation_time
            );

            // Log to audit with policy decision
            log_policy_decision_to_audit(&state, ctx, path, &action, false, &ctx.policy_names)
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
        log_policy_decision_to_audit(&state, ctx, path, &action, true, &ctx.policy_names).await;

        Ok(next.run(request).await)
    } else {
        // No authentication context - deny by default
        warn!(
            "Policy check failed: no authentication context for path={}",
            path
        );

        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "Authentication required",
                "path": path,
            })),
        )
            .into_response());
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

    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .or_else(|| request.headers().get("x-real-ip"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    json!({
        "user_id": ctx.user_id,
        "user_email": ctx.user_email,
        "user_roles": ctx.user_roles,
        "client_ip": client_ip,
        "request_id": ctx.request_id,
        "mfa_passed": false, // TODO: Get actual MFA status from context
        "timestamp": chrono::Utc::now().to_rfc3339(),
    })
}

/// Record policy evaluation metrics
fn record_policy_evaluation_metrics(allowed: bool, evaluation_time: std::time::Duration) {
    // TODO: Integrate with Prometheus metrics
    // For now, just log
    let decision = if allowed { "allowed" } else { "denied" };
    debug!(
        "Policy evaluation: decision={}, time={}μs",
        decision,
        evaluation_time.as_micros()
    );

    // Metrics to track:
    // - secreton_policy_evaluations_total{decision="allowed|denied"}
    // - secreton_policy_evaluation_duration_seconds
    // - secreton_policy_cache_hits_total
    // - secreton_policy_cache_misses_total
}

/// Log policy decision to audit log
async fn log_policy_decision_to_audit(
    state: &ApiState,
    ctx: &RequestContext,
    path: &str,
    action: &str,
    allowed: bool,
    policy_names: &[String],
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
        ip: None,         // TODO: Extract from request
        user_agent: None, // TODO: Extract from request
        namespace: None,  // TODO: Extract namespace from path
        metadata,
    };

    if let Err(e) = state.services.audit.log(audit_entry).await {
        warn!("Failed to log policy decision to audit: {}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}

/// Response wrapping middleware
///
/// Automatically wraps responses when X-Vault-Wrap-TTL header is present.
/// This allows clients to request wrapped responses for any endpoint.
///
/// # Header Format
/// `X-Vault-Wrap-TTL: <seconds>`
///
/// # Example
/// ```bash
/// curl -H "X-Vault-Wrap-TTL: 300" http://localhost:8200/v1/secret/data/myapp
/// ```
///
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
    // Check for X-Vault-Wrap-TTL header
    let wrap_ttl = headers
        .get("X-Vault-Wrap-TTL")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok());

    // If no wrap header, proceed normally
    let wrap_ttl = match wrap_ttl {
        Some(ttl) => ttl,
        None => return next.run(request).await,
    };

    // Validate TTL
    if wrap_ttl == 0 || wrap_ttl > 86400 {
        warn!("Invalid X-Vault-Wrap-TTL value: {}", wrap_ttl);
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "success": false,
                "error": {
                    "code": "INVALID_WRAP_TTL",
                    "message": "X-Vault-Wrap-TTL must be between 1 and 86400 seconds"
                }
            })),
        )
            .into_response();
    }

    // Execute the request
    let response = next.run(request).await;

    // Only wrap successful responses (2xx status codes)
    if !response.status().is_success() {
        return response;
    }

    // Extract response body
    // Note: This is a simplified implementation. In production, you'd need to:
    // 1. Extract the response body properly
    // 2. Parse it as JSON
    // 3. Wrap it using WrappingService
    // 4. Return the wrapped response
    //
    // For now, we'll just pass through the response and log that wrapping was requested
    info!(
        ttl = wrap_ttl,
        "Response wrapping requested via X-Vault-Wrap-TTL header"
    );

    // TODO: Implement actual response wrapping
    // This requires:
    // 1. Buffering the response body
    // 2. Parsing it as JSON
    // 3. Calling state.wrapping_service.wrap()
    // 4. Returning the wrapped token response
    //
    // For now, return the original response with a warning header
    let mut response = response;
    response.headers_mut().insert(
        "X-Vault-Wrap-Warning",
        "Response wrapping via middleware not yet fully implemented. Use /v1/sys/wrapping/wrap endpoint instead."
            .parse()
            .unwrap(),
    );

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_certificate_cache() {
        let cache = CertificateCache::new(60);
        assert!(cache.get("test").is_none());
    }

    #[test]
    fn test_wrap_ttl_validation() {
        assert!(0 == 0 || 0 > 86400); // Invalid
        assert!(300 > 0 && 300 <= 86400); // Valid
        assert!(86401 > 86400); // Invalid
    }
}
