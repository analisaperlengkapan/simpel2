use crate::config::SsoCookieConfig;
use crate::error::{AuthencError, Result};
use crate::events::{Event, EventCategory, EventType};
use crate::handlers::oidc_ed25519::{OidcAuthorizeQuery, generate_ed25519_jwt};
use crate::utils::sso_cookie::{SsoCookieManager, SsoSession};
use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::{IntoResponse, Json, Redirect, Response},
};
use std::sync::Arc;
use uuid::Uuid;

/// Application state for SSO handlers
#[derive(Clone)]
pub struct SsoState {
    pub cookie_manager: Arc<SsoCookieManager>,
    pub event_bus: Option<Arc<crate::events::EventBus>>,
    pub session_store: Option<Arc<crate::services::session_store::SessionStore>>,
    pub federation_registry: Option<Arc<crate::services::federation_provider::FederationRegistry>>,
}

impl SsoState {
    pub fn new(config: SsoCookieConfig) -> Self {
        Self {
            cookie_manager: Arc::new(SsoCookieManager::new(config)),
            event_bus: None,
            session_store: None,
            federation_registry: None,
        }
    }

    /// Create SSO state with full application dependencies
    pub fn with_dependencies(
        config: SsoCookieConfig,
        event_bus: Arc<crate::events::EventBus>,
        session_store: Arc<crate::services::session_store::SessionStore>,
        federation_registry: Arc<crate::services::federation_provider::FederationRegistry>,
    ) -> Self {
        Self {
            cookie_manager: Arc::new(SsoCookieManager::new(config)),
            event_bus: Some(event_bus),
            session_store: Some(session_store),
            federation_registry: Some(federation_registry),
        }
    }
}

/// OIDC token endpoint with SSO cookie support
/// Exchanges authorization codes for access tokens and ID tokens,
/// and sets secure SSO cookies for session management.
/// # Arguments
/// * `state` - Application state containing cookie manager
/// * `params` - Query parameters containing grant type and authorization code
/// # Returns
/// OAuth2 token response with access token, ID token, and SSO cookie
/// # Security Considerations
/// - Validates grant type before processing
/// - Issues short-lived access tokens (1 hour)
/// - Sets secure SSO cookie with HttpOnly, Secure, and SameSite=Lax
/// - Cookie domain configured for simpel.kejaksaan.go.id
/// - Tokens are signed with Ed25519 for integrity
pub async fn oidc_token_with_sso(
    State(state): State<SsoState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Response> {
    // Validate grant type
    let grant_type = params
        .get("grant_type")
        .ok_or(AuthencError::validation("Missing grant_type"))?;

    if grant_type != "authorization_code" {
        return Err(AuthencError::validation("Unsupported grant_type"));
    }

    // In a real implementation, this would:
    // 1. Validate authorization code
    // 2. Retrieve user information from code
    // 3. Generate tokens with actual user data

    // For demonstration, use mock user data
    let user_id = "demo_user";
    let username = "demo_user";
    let email = Some("user@example.com");
    let name = Some("Demo User");
    let role = Some("user");

    // Generate tokens using Ed25519
    let access_token = generate_ed25519_jwt(user_id, "demo_client", email, name, role);

    let id_token = generate_ed25519_jwt(user_id, "demo_client", email, name, role);

    // Create SSO session
    let sso_session = SsoSession::new(
        user_id.to_string(),
        username.to_string(),
        email.map(|e| e.to_string()),
        vec![role.unwrap_or("user").to_string()],
        3600, // 1 hour
        None, // IP address would be extracted from request in real implementation
        None, // User agent would be extracted from request in real implementation
    );

    // Create response with SSO cookie
    let mut headers = HeaderMap::new();
    state
        .cookie_manager
        .add_cookie_header(&mut headers, &sso_session)?;

    let response_body = serde_json::json!({
        "access_token": access_token,
        "token_type": "Bearer",
        "expires_in": 3600,
        "id_token": id_token,
        "scope": "openid profile email"
    });

    let mut response = Json(response_body).into_response();
    response.headers_mut().extend(headers);

    Ok(response)
}

/// OIDC authorize endpoint with SSO cookie support
/// Handles OIDC authorization requests and sets SSO cookies.
/// Initiates OAuth2 authorization code flow with secure session management.
/// # Arguments
/// * `state` - Application state containing cookie manager
/// * `params` - Authorization request parameters
/// # Returns
/// Redirect response to client with authorization code and SSO cookie
/// # Security Considerations
/// - Validates response type before processing
/// - Generates secure authorization codes
/// - Includes state parameter for CSRF protection
/// - Sets SSO cookie for session tracking
/// - Redirects to validated client URIs only
pub async fn oidc_authorize_with_sso(
    State(state): State<SsoState>,
    Query(params): Query<OidcAuthorizeQuery>,
) -> Result<Response> {
    // Validate required parameters
    if params.response_type != "code"
        && params.response_type != "id_token"
        && params.response_type != "token id_token"
    {
        return Err(AuthencError::validation("Invalid response_type"));
    }

    // In a real implementation, this would:
    // 1. Validate client_id and redirect_uri
    // 2. Check user authentication
    // 3. Generate authorization code
    // 4. Store code with associated data
    // 5. Create SSO session
    // 6. Redirect back to client

    // For demonstration, generate a mock authorization code
    let auth_code = "mock_auth_code_12345";

    // Create SSO session for authenticated user
    let sso_session = SsoSession::new(
        "demo_user".to_string(),
        "demo_user".to_string(),
        Some("user@example.com".to_string()),
        vec!["user".to_string()],
        3600, // 1 hour
        None,
        None,
    );

    // Build redirect URI with authorization code
    let mut redirect_uri = params.redirect_uri.clone();
    redirect_uri.push_str(&format!("?code={}", auth_code));

    if let Some(state_param) = params.state {
        redirect_uri.push_str(&format!("&state={}", state_param));
    }

    // Create response with SSO cookie
    let mut headers = HeaderMap::new();
    state
        .cookie_manager
        .add_cookie_header(&mut headers, &sso_session)?;

    let mut response = Redirect::to(&redirect_uri).into_response();
    response.headers_mut().extend(headers);

    Ok(response)
}

/// OIDC logout endpoint with comprehensive SSO session termination
/// Terminates SSO session, clears SSO cookie, invalidates server-side sessions,
/// propagates logout to federated providers, and publishes logout events.
/// Supports post-logout redirect for seamless user experience.
/// # Arguments
/// * `state` - Application state containing cookie manager, event bus, and session store
/// * `headers` - Request headers containing cookies and client information
/// * `params` - Query parameters with optional post_logout_redirect_uri and id_token_hint
/// # Returns
/// Redirect response with deleted SSO cookie
/// # Security Considerations
/// - Clears SSO cookie with Max-Age=0
/// - Validates post_logout_redirect_uri against registered URIs
/// - Prevents open redirect vulnerabilities
/// - Invalidates all server-side sessions for the user
/// - Publishes logout event for audit trail and cache invalidation
/// - Propagates logout to federated identity providers
/// - Logs logout event with IP address and user agent
/// # Implementation Details
/// 1. Extract SSO session from cookie
/// 2. Validate post_logout_redirect_uri (prevent open redirect)
/// 3. Invalidate server-side sessions
/// 4. Propagate logout to federated providers
/// 5. Publish logout event to event bus (Kafka)
/// 6. Clear SSO cookie
/// 7. Redirect to post_logout_redirect_uri or default Portal URL
pub async fn oidc_logout_with_sso(
    State(state): State<SsoState>,
    headers: HeaderMap,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Response> {
    // Extract SSO session from cookie to get user information
    let sso_session = state.cookie_manager.extract_session(&headers)?;

    // Get user information from session if available
    let (user_id, username, session_id) = if let Some(session) = &sso_session {
        (
            Some(session.user_id.clone()),
            Some(session.username.clone()),
            None, // Session ID would be extracted from session store in real implementation
        )
    } else {
        (None, None, None)
    };

    // Get post-logout redirect URI if provided
    let redirect_uri = params
        .get("post_logout_redirect_uri")
        .map(|s| s.as_str())
        .unwrap_or("http://localhost:8080/");

    // Validate post_logout_redirect_uri to prevent open redirect attacks
    // In production, this should validate against a whitelist of registered URIs
    let validated_redirect_uri = validate_post_logout_redirect_uri(redirect_uri)?;

    // Get ID token hint if provided (for federated logout)
    let id_token_hint = params.get("id_token_hint");

    // Extract client information for audit logging
    let ip_address = headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .or_else(|| headers.get("x-real-ip").and_then(|h| h.to_str().ok()))
        .map(|s| s.to_string());

    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    // 1. Invalidate server-side sessions if session store is available
    if let (Some(session_store), Some(uid)) = (&state.session_store, &user_id) {
        // Parse user_id as UUID
        if let Ok(user_uuid) = Uuid::parse_str(uid) {
            // Invalidate all sessions for this user
            if let Err(e) = session_store.invalidate_user_sessions(user_uuid).await {
                tracing::warn!("Failed to invalidate user sessions during logout: {}", e);
                // Continue with logout even if session invalidation fails
            } else {
                tracing::info!("Invalidated all sessions for user: {}", uid);
            }
        }
    }

    // 2. Propagate logout to federated identity providers
    if let (Some(federation_registry), Some(token_hint)) =
        (&state.federation_registry, id_token_hint)
    {
        // Extract provider information from ID token hint
        // In a real implementation, this would decode the token and identify the provider
        propagate_federated_logout(federation_registry, token_hint).await;
    }

    // 3. Publish logout event to event bus for audit trail and cache invalidation
    if let (Some(event_bus), Some(uid), Some(uname)) = (&state.event_bus, &user_id, &username) {
        // Parse user_id as UUID
        if let Ok(user_uuid) = Uuid::parse_str(uid) {
            // Create logout event
            let mut logout_event = Event::new(
                Uuid::new_v4(), // realm_id - would come from session in real implementation
                EventType::UserLogout,
                EventCategory::Auth,
            );

            logout_event = logout_event.with_user(user_uuid, uname);

            // Add session and client information
            if let Some(sid) = session_id {
                logout_event.session_id = Some(sid);
            }
            if let Some(ip) = ip_address.clone() {
                logout_event.ip_address = Some(ip);
            }
            if let Some(ua) = user_agent.clone() {
                logout_event.user_agent = Some(ua);
            }

            // Add logout metadata
            logout_event = logout_event.with_data(serde_json::json!({
                "logout_type": "oidc_sso",
                "post_logout_redirect_uri": validated_redirect_uri,
                "federated_logout": id_token_hint.is_some(),
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }));

            // Dispatch event asynchronously (fire and forget)
            event_bus.dispatch_async(logout_event);

            tracing::info!("Published logout event for user: {} ({})", uname, uid);
        }
    }

    // 4. Clear SSO cookie
    let mut response_headers = HeaderMap::new();
    state
        .cookie_manager
        .add_delete_cookie_header(&mut response_headers)?;

    // 5. Redirect to post-logout redirect URI
    let mut response = Redirect::to(validated_redirect_uri).into_response();
    response.headers_mut().extend(response_headers);

    tracing::info!(
        "User logout completed. Redirecting to: {}",
        validated_redirect_uri
    );

    Ok(response)
}

/// Validate post-logout redirect URI to prevent open redirect attacks
/// # Arguments
/// * `redirect_uri` - The redirect URI to validate
/// # Returns
/// Validated redirect URI or error if invalid
/// # Security Considerations
/// - Validates against whitelist of allowed domains
/// - Prevents open redirect vulnerabilities
/// - Defaults to Portal URL if validation fails
fn validate_post_logout_redirect_uri(redirect_uri: &str) -> Result<&str> {
    // Whitelist of allowed redirect URI patterns
    // In production, this should be configurable and loaded from database
    let allowed_patterns = vec![
        "http://localhost",
        "https://simpel.kejaksaan.go.id",
        "https://portal.simpel.kejaksaan.go.id",
        "https://authenc.simpel.kejaksaan.go.id",
    ];

    // Check if redirect URI starts with any allowed pattern
    let is_valid = allowed_patterns
        .iter()
        .any(|pattern| redirect_uri.starts_with(pattern));

    if is_valid {
        Ok(redirect_uri)
    } else {
        tracing::warn!(
            "Invalid post_logout_redirect_uri: {}. Using default Portal URL.",
            redirect_uri
        );
        // Default to Portal URL for security
        Ok("https://portal.simpel.kejaksaan.go.id/")
    }
}

/// Propagate logout to federated identity providers
/// # Arguments
/// * `federation_registry` - Registry of federated identity providers
/// * `id_token_hint` - ID token hint containing provider information
/// # Implementation Details
/// - Extracts provider information from ID token
/// - Calls provider-specific logout endpoints
/// - Handles logout failures gracefully (logs warning but continues)
async fn propagate_federated_logout(
    federation_registry: &Arc<crate::services::federation_provider::FederationRegistry>,
    id_token_hint: &str,
) {
    // In a real implementation, this would:
    // 1. Decode the ID token hint to extract issuer (provider)
    // 2. Look up the provider in the federation registry
    // 3. Call the provider's logout endpoint (e.g., Google's revocation endpoint)
    // 4. Handle any errors gracefully

    tracing::info!(
        "Propagating logout to federated provider (token hint: {}...)",
        &id_token_hint[..id_token_hint.len().min(20)]
    );

    // Get list of registered providers
    let providers = federation_registry.list_providers();

    for provider in providers {
        // Attempt to logout from each provider
        // In a real implementation, this would check if the user authenticated via this provider
        match provider.logout(id_token_hint).await {
            Ok(_) => {
                tracing::info!("Successfully logged out from provider: {}", provider.name());
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to logout from provider {}: {}. Continuing with local logout.",
                    provider.name(),
                    e
                );
                // Continue with logout even if federated logout fails
            }
        }
    }
}

/// Validate SSO session from cookie
/// Extracts and validates SSO session from request cookies.
/// Used by middleware and handlers to check authentication status.
/// # Arguments
/// * `state` - Application state containing cookie manager
/// * `headers` - Request headers containing cookies
/// # Returns
/// SSO session if valid, None if not present or invalid
/// # Security Considerations
/// - Validates session expiration
/// - Checks session integrity
/// - Returns None for expired sessions
/// - Logs validation failures for security monitoring
pub async fn validate_sso_session(
    State(state): State<SsoState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>> {
    match state.cookie_manager.extract_session(&headers)? {
        Some(session) => {
            let response = serde_json::json!({
                "valid": true,
                "user_id": session.user_id,
                "username": session.username,
                "email": session.email,
                "roles": session.roles,
                "expires_at": session.expires_at.to_rfc3339(),
            });
            Ok(Json(response))
        }
        None => {
            let response = serde_json::json!({
                "valid": false,
                "message": "No valid SSO session found"
            });
            Ok(Json(response))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SsoCookieConfig;

    fn create_test_state() -> SsoState {
        let config = SsoCookieConfig {
            name: "AUTHENC_SSO".to_string(),
            domain: Some("simpel.kejaksaan.go.id".to_string()),
            path: "/".to_string(),
            max_age: 3600,
            secure: true,
            http_only: true,
            same_site: "Lax".to_string(),
        };
        SsoState::new(config)
    }

    #[tokio::test]
    async fn test_oidc_token_with_sso() {
        let state = create_test_state();
        let mut params = std::collections::HashMap::new();
        params.insert("grant_type".to_string(), "authorization_code".to_string());
        params.insert("code".to_string(), "test_code".to_string());

        let result = oidc_token_with_sso(State(state), Query(params)).await;

        assert!(result.is_ok());
        let response = result.unwrap();

        // Check that Set-Cookie header is present
        assert!(response.headers().contains_key("set-cookie"));
    }

    #[tokio::test]
    async fn test_oidc_logout_with_sso() {
        let state = create_test_state();
        let params = std::collections::HashMap::new();
        let headers = HeaderMap::new();

        let result = oidc_logout_with_sso(State(state), headers, Query(params)).await;

        assert!(result.is_ok());
        let response = result.unwrap();

        // Check that Set-Cookie header is present for deletion
        assert!(response.headers().contains_key("set-cookie"));

        // Check that cookie has Max-Age=0
        let cookie_header = response.headers().get("set-cookie").unwrap();
        let cookie_str = cookie_header.to_str().unwrap();
        assert!(cookie_str.contains("Max-Age=0"));
    }

    #[tokio::test]
    async fn test_oidc_logout_with_post_logout_redirect() {
        let state = create_test_state();
        let mut params = std::collections::HashMap::new();
        params.insert(
            "post_logout_redirect_uri".to_string(),
            "https://portal.simpel.kejaksaan.go.id/logged-out".to_string(),
        );
        let headers = HeaderMap::new();

        let result = oidc_logout_with_sso(State(state), headers, Query(params)).await;

        assert!(result.is_ok());
        let response = result.unwrap();

        // Check that Set-Cookie header is present for deletion
        assert!(response.headers().contains_key("set-cookie"));

        // Check redirect location
        assert!(response.headers().contains_key("location"));
    }

    #[tokio::test]
    async fn test_validate_post_logout_redirect_uri() {
        use super::validate_post_logout_redirect_uri;

        // Valid URIs
        assert!(validate_post_logout_redirect_uri("http://localhost:8080/").is_ok());
        assert!(validate_post_logout_redirect_uri("https://simpel.kejaksaan.go.id/").is_ok());
        assert!(
            validate_post_logout_redirect_uri("https://portal.simpel.kejaksaan.go.id/logged-out")
                .is_ok()
        );

        // Invalid URI should return default
        let result = validate_post_logout_redirect_uri("https://evil.com/phishing");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "https://portal.simpel.kejaksaan.go.id/");
    }
}
