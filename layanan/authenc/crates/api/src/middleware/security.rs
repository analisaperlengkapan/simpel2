use axum::{
    body::Body,
    extract::{ConnectInfo, Request, State},
    http::{Method, StatusCode},
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tracing::{error, info, warn};

use authenc_core::services::pg_audit_log_store::PgAuditLogStore;

/// The actor a handler or guard resolved while serving a request — see
/// [`authenc_types::domain::audit_log::AuthenticatedActor`] for why this handoff
/// exists. Re-exported so `crate::middleware::security::AuthenticatedActor`
/// keeps working for the login handler; the type itself lives in
/// `authenc-types` because the IAM admin guard (a different crate) produces it
/// too.
pub use authenc_types::domain::audit_log::AuthenticatedActor;

/// Configuration for security monitoring
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SecurityMonitoringConfig {
    /// Whether security monitoring is enabled
    pub enabled: bool,
    /// Threshold for suspicious activity alerts (requests per minute)
    pub suspicious_threshold_rpm: u64,
    /// Paths to monitor for security events
    pub monitored_paths: Vec<String>,
    /// Whether to log all authentication attempts
    pub log_auth_attempts: bool,
    /// Whether to log all authorization failures
    pub log_authz_failures: bool,
}

impl Default for SecurityMonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            suspicious_threshold_rpm: 100,
            // Prefix-matched (`starts_with`) against the request path, so
            // these must be the real mounted prefixes. The old values
            // (`/auth/login`, `/admin`, …) lacked the `/api/v1` prefix and
            // never matched anything — no security event was ever recorded.
            monitored_paths: vec![
                "/api/v1/auth/login".to_string(),
                "/api/v1/auth/logout".to_string(),
                "/api/v1/auth/revoke".to_string(),
                "/api/v1/iam/".to_string(),
                // MFA lifecycle: enrolling, removing or re-issuing a second
                // factor is exactly what an account-takeover does next.
                // (`/mfa/status` is deliberately absent — it is polled.)
                "/api/v1/auth/mfa/setup".to_string(),
                "/api/v1/auth/mfa/backup-codes".to_string(),
                "/api/v1/auth/totp/enable".to_string(),
                "/api/v1/auth/totp/disable".to_string(),
                "/api/v1/auth/me/password".to_string(),
            ],
            log_auth_attempts: true,
            log_authz_failures: true,
        }
    }
}

/// Security monitoring state
#[derive(Clone)]
pub struct SecurityMonitoringState {
    config: SecurityMonitoringConfig,
    audit_store: Option<Arc<PgAuditLogStore>>,
}

impl SecurityMonitoringState {
    /// Create a new security monitoring state with the given configuration and audit store
    pub fn new(
        config: SecurityMonitoringConfig,
        audit_store: Option<Arc<PgAuditLogStore>>,
    ) -> Self {
        Self {
            config,
            audit_store,
        }
    }

    /// Log a security event.
    ///
    /// `status` must be one of the three outcomes `audit_logs.status` accepts
    /// (see [`PgAuditLogStore::add_log`]) — it is the event's outcome, which
    /// only the caller knows. The previous code read a `status` key that the
    /// details payload never carried and fell back to `"unknown"`, so every
    /// insert was rejected by `audit_logs_status_check`.
    async fn log_security_event(&self, event_type: &str, status: &str, details: serde_json::Value) {
        if let Some(audit_store) = &self.audit_store {
            let event = authenc_types::domain::audit_log::AuditLog {
                timestamp: chrono::Utc::now(),
                event: event_type.to_string(),
                user_id: details
                    .get("user_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                client_id: details
                    .get("client_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                status: status.to_string(),
                detail: Some(serde_json::to_string(&details).unwrap_or_default()),
                ip_address: details
                    .get("ip")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
            };

            if let Err(e) = audit_store.add_log(&event).await {
                error!("Failed to store security audit event: {}", e);
            }
        } else {
            // Fallback to logging if audit store is not available
            info!("Security event: {} - {}", event_type, details);
        }
    }
}

/// The end user's address, resolved without letting the caller choose it.
///
/// `ConnectInfo` only ever reports the immediate TCP peer, and behind the
/// ingress that is the proxy — so an audit trail built on it alone records the
/// ingress as every user's address. The forwarding headers carry the real client,
/// **but only a proxy we operate may be believed about them**: the leftmost
/// `X-Forwarded-For` entry is whatever the caller typed (the previous version
/// recorded it verbatim, so an audit row could name any address the attacker
/// liked). `lib_backend::client_ip` trusts the headers only from a peer listed in
/// `TRUSTED_PROXY_CIDRS`, walks the list from the right, and returns something
/// that is always a valid address — the audit column is a Postgres `inet`.
fn client_ip_for(peer: SocketAddr, request: &Request<Body>) -> String {
    let header = |name: &str| request.headers().get(name).and_then(|v| v.to_str().ok());
    lib_backend::client_ip::client_ip_string(
        peer.ip(),
        header("x-forwarded-for"),
        header("x-real-ip"),
    )
}

/// Middleware for security monitoring and alerting
pub async fn security_monitoring_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<SecurityMonitoringState>>,
    request: Request<Body>,
    next: Next,
) -> Response<Body> {
    if !state.config.enabled {
        return next.run(request).await;
    }

    let start_time = Instant::now();
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let ip = addr.ip().to_string();
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    // `ConnectInfo` is the address of the immediate peer — behind the ingress
    // that is the proxy, not the user. The audit row kept the peer address in a
    // `details.ip` string while `audit_logs.ip_address` (an `inet` column, the
    // one the admin table shows) stayed NULL, so every row displayed `-`.
    // Prefer the forwarded client, and record it in the column built for it.
    // Read BEFORE `next.run` consumes the request.
    let client_ip = client_ip_for(addr, &request);

    // Check for suspicious patterns
    let suspicious_indicators = detect_suspicious_activity(&request, &ip);

    let mut response = next.run(request).await;
    let duration = start_time.elapsed();
    let status_code = response.status();

    // The handler is the only party that knows who the caller turned out to be
    // (see `AuthenticatedActor`). Take it before the response is returned.
    let actor = response.extensions().get::<AuthenticatedActor>().cloned();

    // Log security events
    if state
        .config
        .monitored_paths
        .iter()
        .any(|p| path.starts_with(p))
    {
        let mut event_details = serde_json::json!({
            "ip": client_ip,
            "method": method.as_str(),
            "path": path,
            "status_code": status_code.as_u16(),
            "duration_ms": duration.as_millis(),
            "user_agent": user_agent,
            "suspicious_indicators": suspicious_indicators
        });
        // Attribute the event to whoever the handler verified, so `user_id` is
        // populated on the `inet`/uuid columns the admin table actually reads
        // rather than only surviving as prose in `details`.
        if let Some(actor) = &actor {
            event_details["user_id"] = serde_json::Value::String(actor.user_id.clone());
            if let Some(client_id) = &actor.client_id {
                event_details["client_id"] = serde_json::Value::String(client_id.clone());
            }
        }

        // Log authentication attempts
        if state.config.log_auth_attempts && path.contains("/auth/") {
            let (event_type, status) = auth_event(&path, status_code);
            state
                .log_security_event(event_type, status, event_details.clone())
                .await;
        }

        // Log authorization failures
        if state.config.log_authz_failures && status_code == StatusCode::FORBIDDEN {
            state
                .log_security_event("AUTHZ_FAILURE", "failure", event_details.clone())
                .await;
        }

        // Log suspicious activity
        if !suspicious_indicators.is_empty() {
            warn!(
                "Suspicious activity detected: {} from IP {}",
                suspicious_indicators.join(", "),
                client_ip
            );
            state
                .log_security_event("SUSPICIOUS_ACTIVITY", "warning", event_details)
                .await;
        }
    }

    // Add security headers to response (Defense in Depth)
    let headers = response.headers_mut();

    // X-Security-Monitoring
    headers.insert(
        "X-Security-Monitoring",
        format!("monitored; duration={}ms", duration.as_millis())
            .parse()
            .unwrap(),
    );

    // X-Content-Type-Options: nosniff
    headers.insert(
        axum::http::header::X_CONTENT_TYPE_OPTIONS,
        axum::http::HeaderValue::from_static("nosniff"),
    );

    // X-Frame-Options: DENY
    headers.insert(
        axum::http::header::X_FRAME_OPTIONS,
        axum::http::HeaderValue::from_static("DENY"),
    );

    // X-XSS-Protection: 1; mode=block
    headers.insert(
        "X-XSS-Protection",
        axum::http::HeaderValue::from_static("1; mode=block"),
    );

    // Referrer-Policy: strict-origin-when-cross-origin
    headers.insert(
        axum::http::header::REFERRER_POLICY,
        axum::http::HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // Strict-Transport-Security: max-age=31536000; includeSubDomains
    headers.insert(
        axum::http::header::STRICT_TRANSPORT_SECURITY,
        axum::http::HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );

    // Content-Security-Policy (API-specific: restrictive by default)
    let csp = "default-src 'none'; frame-ancestors 'none'; base-uri 'none'; form-action 'none'";
    headers.insert(
        axum::http::header::CONTENT_SECURITY_POLICY,
        axum::http::HeaderValue::from_static(csp),
    );

    response
}

/// Audit event name and outcome for a request under `/auth/`.
///
/// Most paths are a login/logout/revoke and read as `AUTH_*`. The MFA and
/// password-change routes are *account-recovery* surface — labelling a backup
/// code regeneration `AUTH_SUCCESS` would bury it among ordinary logins — so
/// they get their own event names, and a 400 there (wrong step-up code) is a
/// failure, not a generic `warning`.
fn auth_event(path: &str, status: StatusCode) -> (&'static str, &'static str) {
    let lifecycle = match path {
        "/api/v1/auth/mfa/backup-codes" => Some("MFA_BACKUP_CODES"),
        "/api/v1/auth/mfa/setup" | "/api/v1/auth/totp/enable" => Some("MFA_SETUP"),
        "/api/v1/auth/totp/disable" => Some("MFA_DISABLE"),
        "/api/v1/auth/me/password" => Some("PASSWORD_CHANGE"),
        _ => None,
    };
    match (lifecycle, status) {
        (Some(event), s) if s.is_success() => (event, "success"),
        (Some(event), s) if s.is_client_error() => (event, "failure"),
        (Some(event), _) => (event, "warning"),
        (None, StatusCode::OK) => ("AUTH_SUCCESS", "success"),
        (None, StatusCode::UNAUTHORIZED) => ("AUTH_FAILURE", "failure"),
        (None, _) => ("AUTH_ATTEMPT", "warning"),
    }
}

/// Detect suspicious activity patterns
fn detect_suspicious_activity(request: &Request<Body>, _ip: &str) -> Vec<String> {
    let mut indicators = Vec::new();
    let path = request.uri().path();
    let method = request.method();
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    // Check for common attack patterns
    if path.contains("..") || path.contains("\\") {
        indicators.push("path_traversal".to_string());
    }

    if method == Method::TRACE && !path.starts_with("/health") {
        indicators.push("trace_method".to_string());
    }

    if user_agent.is_empty() || user_agent == "unknown" {
        indicators.push("missing_user_agent".to_string());
    }

    // Check for SQL injection patterns (basic detection)
    let query = request.uri().query().unwrap_or("");
    let decoded_query = urlencoding::decode(query).unwrap_or_else(|_| query.into());
    if decoded_query.contains("'")
        || decoded_query.contains("1=1")
        || decoded_query.contains("OR 1=1")
    {
        indicators.push("potential_sql_injection".to_string());
    }

    // Check for XSS patterns in query parameters
    if decoded_query.contains("<script") || decoded_query.contains("javascript:") {
        indicators.push("potential_xss".to_string());
    }

    // Check for unusual request patterns
    if request.headers().get("x-forwarded-for").is_some()
        && request.headers().get("x-real-ip").is_some()
    {
        indicators.push("multiple_proxy_headers".to_string());
    }

    indicators
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, body::Body, extract::Request, routing::get};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_suspicious_activity_detection() {
        let request = Request::builder()
            .uri("/admin?query=1%3D1")
            .method("GET")
            .header("user-agent", "suspicious-scanner")
            .body(Body::empty())
            .unwrap();

        let indicators = detect_suspicious_activity(&request, "127.0.0.1");
        assert!(indicators.contains(&"potential_sql_injection".to_string()));
    }

    #[tokio::test]
    async fn test_security_monitoring_middleware() {
        let state = Arc::new(SecurityMonitoringState::new(
            SecurityMonitoringConfig::default(),
            None, // No audit store for test
        ));

        let app =
            Router::new()
                .route("/test", get(|| async { "OK" }))
                .layer(axum::middleware::from_fn(move |req, next| {
                    let state = state.clone();
                    security_monitoring_middleware(
                        axum::extract::ConnectInfo(std::net::SocketAddr::from((
                            [127, 0, 0, 1],
                            8080,
                        ))),
                        axum::extract::State(state),
                        req,
                        next,
                    )
                }));

        let response = app
            .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::OK);
        assert!(response.headers().contains_key("x-security-monitoring"));

        // Verify standard security headers
        assert_eq!(
            response
                .headers()
                .get(axum::http::header::X_CONTENT_TYPE_OPTIONS)
                .unwrap(),
            "nosniff"
        );
        assert_eq!(
            response
                .headers()
                .get(axum::http::header::X_FRAME_OPTIONS)
                .unwrap(),
            "DENY"
        );
        assert_eq!(
            response.headers().get("X-XSS-Protection").unwrap(),
            "1; mode=block"
        );
        assert_eq!(
            response
                .headers()
                .get(axum::http::header::REFERRER_POLICY)
                .unwrap(),
            "strict-origin-when-cross-origin"
        );
        assert_eq!(
            response
                .headers()
                .get(axum::http::header::STRICT_TRANSPORT_SECURITY)
                .unwrap(),
            "max-age=31536000; includeSubDomains"
        );
        assert_eq!(
            response
                .headers()
                .get(axum::http::header::CONTENT_SECURITY_POLICY)
                .unwrap(),
            "default-src 'none'; frame-ancestors 'none'; base-uri 'none'; form-action 'none'"
        );
    }

    #[tokio::test]
    async fn test_security_monitoring_disabled() {
        let config = SecurityMonitoringConfig {
            enabled: false,
            ..Default::default()
        };
        let state = Arc::new(SecurityMonitoringState::new(config, None));

        let app =
            Router::new()
                .route("/test", get(|| async { "OK" }))
                .layer(axum::middleware::from_fn(move |req, next| {
                    let state = state.clone();
                    security_monitoring_middleware(
                        axum::extract::ConnectInfo(std::net::SocketAddr::from((
                            [127, 0, 0, 1],
                            8080,
                        ))),
                        axum::extract::State(state),
                        req,
                        next,
                    )
                }));

        let response = app
            .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::OK);
        // Should not add security monitoring header when disabled
        assert!(!response.headers().contains_key("x-security-monitoring"));
    }

    #[tokio::test]
    async fn test_suspicious_activity_path_traversal() {
        let request = Request::builder()
            .uri("/admin/../../../etc/passwd")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let indicators = detect_suspicious_activity(&request, "127.0.0.1");
        assert!(indicators.contains(&"path_traversal".to_string()));
    }

    #[tokio::test]
    async fn test_suspicious_activity_trace_method() {
        let request = Request::builder()
            .uri("/admin/users")
            .method("TRACE")
            .body(Body::empty())
            .unwrap();

        let indicators = detect_suspicious_activity(&request, "127.0.0.1");
        assert!(indicators.contains(&"trace_method".to_string()));
    }

    #[tokio::test]
    async fn test_suspicious_activity_missing_user_agent() {
        let request = Request::builder()
            .uri("/admin")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let indicators = detect_suspicious_activity(&request, "127.0.0.1");
        assert!(indicators.contains(&"missing_user_agent".to_string()));
    }

    #[tokio::test]
    async fn test_suspicious_activity_sql_injection() {
        let request = Request::builder()
            .uri("/search?q=%27%20OR%20%271%27%3D%271")
            .method("GET")
            .header("user-agent", "test-agent")
            .body(Body::empty())
            .unwrap();

        let indicators = detect_suspicious_activity(&request, "127.0.0.1");
        assert!(indicators.contains(&"potential_sql_injection".to_string()));
    }

    #[tokio::test]
    async fn test_suspicious_activity_xss() {
        let request = Request::builder()
            .uri("/search?q=%3Cscript%3Ealert%28%27xss%27%29%3C%2Fscript%3E")
            .method("GET")
            .header("user-agent", "test-agent")
            .body(Body::empty())
            .unwrap();

        let indicators = detect_suspicious_activity(&request, "127.0.0.1");
        assert!(indicators.contains(&"potential_xss".to_string()));
    }

    #[tokio::test]
    async fn test_suspicious_activity_multiple_proxy_headers() {
        let request = Request::builder()
            .uri("/admin")
            .method("GET")
            .header("user-agent", "test-agent")
            .header("x-forwarded-for", "192.168.1.1")
            .header("x-real-ip", "10.0.0.1")
            .body(Body::empty())
            .unwrap();

        let indicators = detect_suspicious_activity(&request, "127.0.0.1");
        assert!(indicators.contains(&"multiple_proxy_headers".to_string()));
    }

    /// A caller must not be able to choose the address an audit row records.
    /// With no trusted proxy configured (the default), `X-Forwarded-For` is
    /// ignored and the TCP peer is what gets attributed — including when the
    /// header is not an address at all, which used to reach the `inet` column
    /// and fail the whole audit INSERT.
    #[tokio::test]
    async fn forged_forwarding_headers_do_not_choose_the_audited_address() {
        let peer: SocketAddr = ([203, 0, 113, 9], 4242).into();
        for forged in [
            "1.2.3.4",
            "1.2.3.4, 5.6.7.8",
            "x",
            "1.2.3.4:80",
            "[::1]:1",
            "",
        ] {
            let request = Request::builder()
                .uri("/api/v1/auth/login")
                .header("x-forwarded-for", forged)
                .header("x-real-ip", "9.9.9.9")
                .body(Body::empty())
                .unwrap();
            let got = client_ip_for(peer, &request);
            assert_eq!(got, "203.0.113.9", "header {forged:?} must not be believed");
            assert!(got.parse::<std::net::IpAddr>().is_ok());
        }
    }

    #[tokio::test]
    async fn test_suspicious_activity_no_indicators() {
        let request = Request::builder()
            .uri("/api/users")
            .method("GET")
            .header("user-agent", "Mozilla/5.0")
            .body(Body::empty())
            .unwrap();

        let indicators = detect_suspicious_activity(&request, "127.0.0.1");
        assert!(indicators.is_empty());
    }

    #[tokio::test]
    async fn test_security_monitoring_config_defaults() {
        let config = SecurityMonitoringConfig::default();

        assert!(config.enabled);
        assert_eq!(config.suspicious_threshold_rpm, 100);
        // Prefix-matched against real mounted paths — every default must
        // carry the /api/v1 prefix or it will never match a request.
        assert!(
            config
                .monitored_paths
                .contains(&"/api/v1/auth/login".to_string())
        );
        assert!(config.monitored_paths.contains(&"/api/v1/iam/".to_string()));
        assert!(
            config
                .monitored_paths
                .iter()
                .all(|p| p.starts_with("/api/v1/"))
        );
        assert!(config.log_auth_attempts);
        assert!(config.log_authz_failures);
    }

    #[tokio::test]
    async fn test_security_monitoring_trace_on_health_allowed() {
        let request = Request::builder()
            .uri("/health")
            .method("TRACE")
            .body(Body::empty())
            .unwrap();

        let indicators = detect_suspicious_activity(&request, "127.0.0.1");
        // TRACE method should not be flagged as suspicious for health endpoints
        assert!(!indicators.contains(&"trace_method".to_string()));
    }

    /// Every default monitored MFA path must be one `auth_event` knows by name,
    /// and the polled `/mfa/status` must stay out of the audit trail.
    #[test]
    fn mfa_lifecycle_paths_have_their_own_audit_events() {
        let config = SecurityMonitoringConfig::default();
        assert!(
            !config
                .monitored_paths
                .iter()
                .any(|p| "/api/v1/auth/mfa/status".starts_with(p.as_str())),
            "/mfa/status is polled; auditing it would drown real events"
        );

        for (path, event) in [
            ("/api/v1/auth/mfa/backup-codes", "MFA_BACKUP_CODES"),
            ("/api/v1/auth/mfa/setup", "MFA_SETUP"),
            ("/api/v1/auth/totp/enable", "MFA_SETUP"),
            ("/api/v1/auth/totp/disable", "MFA_DISABLE"),
            ("/api/v1/auth/me/password", "PASSWORD_CHANGE"),
        ] {
            assert!(
                config
                    .monitored_paths
                    .iter()
                    .any(|p| path.starts_with(p.as_str())),
                "{path} must be monitored"
            );
            assert_eq!(auth_event(path, StatusCode::OK), (event, "success"));
            assert_eq!(
                auth_event(path, StatusCode::BAD_REQUEST),
                (event, "failure"),
                "a wrong step-up code is a failure of {event}"
            );
        }
    }

    #[test]
    fn ordinary_auth_paths_keep_the_login_event_names() {
        assert_eq!(
            auth_event("/api/v1/auth/login", StatusCode::OK),
            ("AUTH_SUCCESS", "success")
        );
        assert_eq!(
            auth_event("/api/v1/auth/login", StatusCode::UNAUTHORIZED),
            ("AUTH_FAILURE", "failure")
        );
        assert_eq!(
            auth_event("/api/v1/auth/login", StatusCode::TOO_MANY_REQUESTS),
            ("AUTH_ATTEMPT", "warning")
        );
    }
}
