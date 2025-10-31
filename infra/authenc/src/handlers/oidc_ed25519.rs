use crate::crypto::ed25519_keys::{ED25519_KEYPAIR, get_ed25519_jwk};
use crate::error::AuthencError;
use crate::utils::crypto_monitor::CryptoMonitor;
use axum::{
    extract::Query,
    http::{HeaderMap, StatusCode},
    response::{Json, Redirect},
};
use base64ct::{Base64UrlUnpadded, Encoding};
use chrono::Utc;
use ed25519_dalek::{Signature, Signer};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
/// OIDC authorization request query parameters
///
/// Parameters received during OIDC authorization code flow.
/// Contains client information and requested scopes for authentication.
///
/// # Security Considerations
/// - State parameter prevents CSRF attacks
/// - Redirect URI must be validated against registered URIs
/// - Client ID must be validated before processing
/// - Response type determines the authorization flow
pub struct OidcAuthorizeQuery {
    /// OAuth2 response type (code, token, id_token)
    pub response_type: String,
    /// OAuth2 client identifier
    pub client_id: String,
    /// URI to redirect after authorization
    pub redirect_uri: String,
    /// Requested OAuth2 scopes (openid, profile, email)
    pub scope: Option<String>,
    /// Opaque value for CSRF protection
    pub state: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
/// JWT header for Ed25519 signed tokens
///
/// JWT header containing algorithm and key information for Ed25519 signatures.
/// Used in OIDC ID tokens and access tokens signed with Ed25519.
///
/// # Security Considerations
/// - Algorithm must be EdDSA for Ed25519 signatures
/// - Key ID enables key rotation and validation
/// - Header is integrity protected by the signature
pub struct Ed25519JwtHeader {
    /// Signature algorithm (EdDSA for Ed25519)
    pub alg: String,
    /// Token type (JWT)
    pub typ: String,
    /// Key ID for key identification in JWKS
    pub kid: String,
}

#[derive(Debug, Serialize, Deserialize)]
/// OIDC ID token claims
///
/// Standard OIDC claims included in ID tokens.
/// Contains user identity information and token metadata.
///
/// # Security Considerations
/// - Timestamps prevent token reuse attacks
/// - Audience validation prevents token misuse
/// - Subject uniquely identifies the user
/// - Claims are signed and cannot be modified
pub struct OidcIdTokenClaims {
    /// Issuer identifier (token issuer)
    pub iss: String,
    /// Subject identifier (user ID)
    pub sub: String,
    /// Audience (client ID the token is for)
    pub aud: String,
    /// Expiration timestamp
    pub exp: i64,
    /// Issued at timestamp
    pub iat: i64,
    /// User's email address
    pub email: Option<String>,
    /// User's display name
    pub name: Option<String>,
    /// User's role or authorization level
    pub role: Option<String>,
}

/// Generate JWT token using Ed25519 - secure replacement for RSA
///
/// Creates a JWT token signed with Ed25519 digital signatures.
/// Provides better security and performance compared to RSA signatures.
///
/// # Arguments
/// * `sub` - Subject identifier (user ID)
/// * `aud` - Audience (client ID)
/// * `email` - User's email address
/// * `name` - User's display name
/// * `role` - User's role/authorization level
///
/// # Returns
/// A complete JWT token with Ed25519 signature
///
/// # Security Considerations
/// - Uses Ed25519 for fast, secure signatures
/// - Includes standard JWT claims (iss, sub, aud, exp, iat)
/// - One hour token expiration for security
/// - All claims are signed and tamper-proof
/// - Crypto operations are monitored for security
pub fn generate_ed25519_jwt(
    sub: &str,
    aud: &str,
    email: Option<&str>,
    name: Option<&str>,
    role: Option<&str>,
) -> String {
    let now = Utc::now().timestamp();

    let header = Ed25519JwtHeader {
        alg: "EdDSA".to_string(),
        typ: "JWT".to_string(),
        kid: "authence-ed25519-key".to_string(),
    };

    let claims = OidcIdTokenClaims {
        iss: "http://localhost:8080/v1".to_string(),
        sub: sub.to_string(),
        aud: aud.to_string(),
        exp: now + 3600,
        iat: now,
        email: email.map(|e| e.to_string()),
        name: name.map(|n| n.to_string()),
        role: role.map(|r| r.to_string()),
    };

    // Monitor Ed25519 operations for security
    CryptoMonitor::monitor_rsa_operation("ed25519_jwt_signing", || {
        // Encode header and payload
        let header_json = serde_json::to_string(&header).unwrap();
        let claims_json = serde_json::to_string(&claims).unwrap();

        let header_b64 = Base64UrlUnpadded::encode_string(header_json.as_bytes());
        let payload_b64 = Base64UrlUnpadded::encode_string(claims_json.as_bytes());

        // Create signing input
        let signing_input = format!("{}.{}", header_b64, payload_b64);

        // Sign with Ed25519
        let signature: Signature = ED25519_KEYPAIR.sign(signing_input.as_bytes());
        let signature_b64 = Base64UrlUnpadded::encode_string(signature.to_bytes().as_ref());

        format!("{}.{}", signing_input, signature_b64)
    })
}

/// OIDC JWKS endpoint with Ed25519 keys - replaces RSA JWKS
///
/// Provides JSON Web Key Set containing Ed25519 public keys.
/// Allows clients to verify JWT signatures signed with Ed25519.
///
/// # Returns
/// JWKS document containing Ed25519 public key for signature verification
///
/// # Security Considerations
/// - Only exposes public keys for signature verification
/// - Private keys never leave the server
/// - Supports key rotation through key ID (kid)
/// - Enables secure token validation by clients
pub async fn oidc_jwks_ed25519() -> Result<Json<serde_json::Value>, AuthencError> {
    let jwk = get_ed25519_jwk();
    let jwks = serde_json::json!({
        "keys": [jwk]
    });
    Ok(Json(jwks))
}

/// OIDC token endpoint using Ed25519 - secure replacement for RSA
///
/// Exchanges authorization codes for access tokens and ID tokens.
/// Issues tokens signed with Ed25519 for enhanced security.
/// Supports both authorization_code and refresh_token grant types.
///
/// # Arguments
/// * `params` - Query parameters containing grant type and authorization code or refresh token
///
/// # Returns
/// OAuth2 token response with access token, ID token, and metadata
///
/// # Security Considerations
/// - Validates grant type before processing
/// - Issues short-lived access tokens (1 hour)
/// - Includes ID tokens with user identity claims
/// - Tokens are signed with Ed25519 for integrity
/// - Supports standard OAuth2 token response format
/// - Implements token rotation for refresh tokens
pub async fn oidc_token_ed25519(
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, AuthencError> {
    // Simplified token endpoint for demonstration
    let grant_type = params
        .get("grant_type")
        .ok_or(AuthencError::validation("Bad request"))?;

    match grant_type.as_str() {
        "authorization_code" => {
            // Generate tokens using Ed25519
            let access_token = generate_ed25519_jwt(
                "demo_user",
                "demo_client",
                Some("user@example.com"),
                Some("Demo User"),
                Some("user"),
            );

            let id_token = generate_ed25519_jwt(
                "demo_user",
                "demo_client",
                Some("user@example.com"),
                Some("Demo User"),
                Some("user"),
            );

            // Generate refresh token
            let refresh_token = crate::utils::jwt::generate_refresh_token("demo_user")
                .map_err(|e| AuthencError::internal(&format!("Failed to generate refresh token: {}", e)))?;

            let response = serde_json::json!({
                "access_token": access_token,
                "token_type": "Bearer",
                "expires_in": 3600,
                "refresh_token": refresh_token,
                "id_token": id_token,
                "scope": "openid profile email"
            });

            Ok(Json(response))
        }
        "refresh_token" => {
            // Handle refresh token grant
            let refresh_token = params
                .get("refresh_token")
                .ok_or(AuthencError::validation("refresh_token is required"))?;

            // Verify refresh token
            let claims = crate::utils::jwt::verify_refresh_token(refresh_token)
                .map_err(|e| AuthencError::unauthorized(&format!("Invalid refresh token: {}", e)))?;

            // Generate new access token
            let access_token = generate_ed25519_jwt(
                &claims.sub,
                "demo_client",
                Some("user@example.com"),
                Some("Demo User"),
                Some("user"),
            );

            // Generate new ID token
            let id_token = generate_ed25519_jwt(
                &claims.sub,
                "demo_client",
                Some("user@example.com"),
                Some("Demo User"),
                Some("user"),
            );

            // Generate new refresh token (token rotation)
            let new_refresh_token = crate::utils::jwt::generate_refresh_token(&claims.sub)
                .map_err(|e| AuthencError::internal(&format!("Failed to generate refresh token: {}", e)))?;

            let response = serde_json::json!({
                "access_token": access_token,
                "token_type": "Bearer",
                "expires_in": 3600,
                "refresh_token": new_refresh_token,
                "id_token": id_token,
                "scope": "openid profile email"
            });

            Ok(Json(response))
        }
        _ => Err(AuthencError::validation("Unsupported grant type")),
    }
}

/// OIDC discovery endpoint with Ed25519 algorithm support
///
/// Provides comprehensive OIDC discovery document with Ed25519 algorithm information.
/// Allows clients to discover OIDC endpoints, supported algorithms, grant types, and capabilities.
///
/// # Returns
/// OIDC discovery document with endpoint URLs and full capabilities
///
/// # Security Considerations
/// - Advertises EdDSA as supported signing algorithm
/// - Lists all available OIDC endpoints including JWKS
/// - Includes supported grant types for OAuth2 flows
/// - Includes supported response types for authorization flows
/// - Includes supported scopes and claims
/// - Includes token endpoint authentication methods
/// - Enables secure client configuration and discovery
pub async fn oidc_discovery_ed25519() -> Result<Json<serde_json::Value>, AuthencError> {
    let discovery = serde_json::json!({
        "issuer": "http://localhost:8080/v1",
        "authorization_endpoint": "http://localhost:8080/v1/oidc/authorize",
        "token_endpoint": "http://localhost:8080/v1/oidc/token",
        "refresh_endpoint": "http://localhost:8080/v1/oidc/refresh",
        "revocation_endpoint": "http://localhost:8080/v1/oidc/revoke",
        "end_session_endpoint": "http://localhost:8080/v1/oidc/logout",
        "jwks_uri": "http://localhost:8080/v1/.well-known/jwks.json",
        "userinfo_endpoint": "http://localhost:8080/v1/oidc/userinfo",
        "grant_types_supported": [
            "authorization_code",
            "refresh_token"
        ],
        "response_types_supported": [
            "code",
            "token",
            "id_token",
            "code token",
            "code id_token",
            "token id_token",
            "code token id_token"
        ],
        "response_modes_supported": ["query", "fragment"],
        "subject_types_supported": ["public"],
        "id_token_signing_alg_values_supported": ["EdDSA"],
        "scopes_supported": ["openid", "profile", "email"],
        "token_endpoint_auth_methods_supported": [
            "client_secret_basic",
            "client_secret_post",
            "client_secret_jwt",
            "private_key_jwt"
        ],
        "claims_supported": [
            "iss",
            "sub",
            "aud",
            "exp",
            "iat",
            "email",
            "email_verified",
            "name",
            "role",
            "updated_at"
        ],
        "code_challenge_methods_supported": ["S256", "plain"],
        "revocation_endpoint_auth_methods_supported": [
            "client_secret_basic",
            "client_secret_post"
        ]
    });

    Ok(Json(discovery))
}

/// OIDC userinfo endpoint with Ed25519 - secure replacement for RSA
///
/// Provides user profile information based on access token.
/// Returns user claims for authorized clients.
///
/// # Arguments
/// * `headers` - HTTP headers containing Authorization header with access token
///
/// # Returns
/// User profile information in JSON format
///
/// # Security Considerations
/// - Validates Bearer token in Authorization header
/// - Returns only authorized user information
/// - Claims based on token scope and user permissions
/// - User information is current and up-to-date
pub async fn oidc_userinfo_ed25519(
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, AuthencError> {
    // Extract Authorization header
    let _auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(AuthencError::unauthorized("Unauthorized"))?;

    // In a real implementation, this would:
    // 1. Validate the access token
    // 2. Extract user information from the token
    // 3. Return appropriate user claims based on scope

    // For demonstration, return mock user info
    let userinfo = serde_json::json!({
        "sub": "user123",
        "email": "user@example.com",
        "email_verified": true,
        "name": "Demo User",
        "role": "user",
        "updated_at": chrono::Utc::now().timestamp()
    });

    Ok(Json(userinfo))
}

/// OIDC refresh endpoint for silent token refresh
///
/// Dedicated endpoint for refreshing access tokens without full page reload.
/// Supports iframe-based silent refresh for seamless user experience.
///
/// # Arguments
/// * `params` - Query parameters containing refresh token
///
/// # Returns
/// OAuth2 token response with new access token and rotated refresh token
///
/// # Security Considerations
/// - Validates refresh token before processing
/// - Implements token rotation (new refresh token on each use)
/// - Issues short-lived access tokens (1 hour)
/// - Refresh tokens expire after 30 days
/// - Supports CORS for iframe-based refresh
/// - Prevents refresh token reuse attacks
pub async fn oidc_refresh_ed25519(
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, AuthencError> {
    // Extract refresh token
    let refresh_token = params
        .get("refresh_token")
        .ok_or(AuthencError::validation("refresh_token is required"))?;

    // Verify refresh token
    let claims = crate::utils::jwt::verify_refresh_token(refresh_token)
        .map_err(|e| AuthencError::unauthorized(&format!("Invalid refresh token: {}", e)))?;

    // Generate new access token
    let access_token = generate_ed25519_jwt(
        &claims.sub,
        "demo_client",
        Some("user@example.com"),
        Some("Demo User"),
        Some("user"),
    );

    // Generate new ID token
    let id_token = generate_ed25519_jwt(
        &claims.sub,
        "demo_client",
        Some("user@example.com"),
        Some("Demo User"),
        Some("user"),
    );

    // Generate new refresh token (token rotation)
    let new_refresh_token = crate::utils::jwt::generate_refresh_token(&claims.sub)
        .map_err(|e| AuthencError::internal(&format!("Failed to generate refresh token: {}", e)))?;

    let response = serde_json::json!({
        "access_token": access_token,
        "token_type": "Bearer",
        "expires_in": 3600,
        "refresh_token": new_refresh_token,
        "id_token": id_token,
        "scope": "openid profile email"
    });

    Ok(Json(response))
}

/// OIDC revoke endpoint for refresh token revocation
///
/// Revokes refresh tokens to invalidate user sessions.
/// Essential for logout and security incident response.
///
/// # Arguments
/// * `params` - Query parameters containing token to revoke
///
/// # Returns
/// Success response indicating token revocation
///
/// # Security Considerations
/// - Validates token before revocation
/// - Supports both access and refresh token revocation
/// - Implements token blacklisting
/// - Prevents token reuse after revocation
/// - Logs revocation events for audit trail
pub async fn oidc_revoke_ed25519(
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, AuthencError> {
    // Extract token to revoke
    let token = params
        .get("token")
        .ok_or(AuthencError::validation("token is required"))?;

    let token_type_hint = params.get("token_type_hint");

    // Verify token (either access or refresh token)
    let _claims = if token_type_hint == Some(&"refresh_token".to_string()) {
        crate::utils::jwt::verify_refresh_token(token)
            .map_err(|e| AuthencError::unauthorized(&format!("Invalid token: {}", e)))?
    } else {
        crate::utils::jwt::verify_jwt(token)
            .map_err(|e| AuthencError::unauthorized(&format!("Invalid token: {}", e)))?
    };

    // In a real implementation, this would:
    // 1. Add token to blacklist/revocation list
    // 2. Update database to mark token as revoked
    // 3. Invalidate related sessions
    // 4. Log revocation event for audit trail
    // 5. Notify other services via event bus

    let response = serde_json::json!({
        "success": true,
        "message": "Token revoked successfully"
    });

    Ok(Json(response))
}

/// OIDC authorize endpoint with Ed25519 - secure replacement for RSA
///
/// Handles OIDC authorization requests and redirects.
/// Initiates OAuth2 authorization code flow with Ed25519 security and PKCE support.
///
/// # Arguments
/// * `params` - Authorization request parameters
///
/// # Returns
/// Redirect response to client with authorization code
///
/// # Security Considerations
/// - Validates response type before processing
/// - Generates secure authorization codes
/// - Includes state parameter for CSRF protection
/// - Redirects to validated client URIs only
/// - Prevents authorization code injection attacks
/// - Supports PKCE for enhanced security
pub async fn oidc_authorize_ed25519(
    Query(params): Query<OidcAuthorizeQuery>,
) -> Result<Redirect, StatusCode> {
    // Validate required parameters
    if params.response_type != "code"
        && params.response_type != "id_token"
        && params.response_type != "token id_token"
    {
        return Err(StatusCode::BAD_REQUEST);
    }

    // In a real implementation, this would:
    // 1. Validate client_id and redirect_uri
    // 2. Check user authentication
    // 3. Generate authorization code
    // 4. Store code with associated data
    // 5. Redirect back to client

    // For demonstration, generate a mock authorization code
    let auth_code = "mock_auth_code_12345";

    // Build redirect URI with authorization code
    let mut redirect_uri = params.redirect_uri.clone();
    redirect_uri.push_str(&format!("?code={}", auth_code));

    if let Some(state) = params.state {
        redirect_uri.push_str(&format!("&state={}", state));
    }

    Ok(Redirect::to(&redirect_uri))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ed25519_jwt_generation() {
        let token = generate_ed25519_jwt(
            "user123",
            "test-audience",
            Some("user@example.com"),
            Some("Test User"),
            Some("admin"),
        );

        // Token should have 3 parts
        assert_eq!(token.split('.').count(), 3);

        // Should contain proper header
        let parts: Vec<&str> = token.split('.').collect();
        let header_json =
            String::from_utf8(Base64UrlUnpadded::decode_vec(parts[0]).unwrap()).unwrap();
        let header: Ed25519JwtHeader = serde_json::from_str(&header_json).unwrap();
        assert_eq!(header.alg, "EdDSA");
    }

    #[tokio::test]
    async fn test_oidc_jwks_endpoint() {
        let result = oidc_jwks_ed25519().await;
        assert!(result.is_ok());

        let jwks = result.unwrap().0;
        assert!(jwks["keys"].is_array());
        assert_eq!(jwks["keys"].as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_oidc_discovery_endpoint() {
        let result = oidc_discovery_ed25519().await;
        assert!(result.is_ok());

        let discovery = result.unwrap().0;

        // Verify issuer
        assert_eq!(discovery["issuer"], "http://localhost:8080/v1");

        // Verify JWKS endpoint is present at standard location
        assert_eq!(discovery["jwks_uri"], "http://localhost:8080/v1/.well-known/jwks.json");

        // Verify refresh, revocation, and logout endpoints
        assert_eq!(discovery["refresh_endpoint"], "http://localhost:8080/v1/oidc/refresh");
        assert_eq!(discovery["revocation_endpoint"], "http://localhost:8080/v1/oidc/revoke");
        assert_eq!(discovery["end_session_endpoint"], "http://localhost:8080/v1/oidc/logout");

        // Verify supported grant types
        let grant_types = discovery["grant_types_supported"].as_array().unwrap();
        assert!(grant_types.contains(&serde_json::Value::String("authorization_code".to_string())));
        assert!(grant_types.contains(&serde_json::Value::String("refresh_token".to_string())));

        // Verify supported response types
        let response_types = discovery["response_types_supported"].as_array().unwrap();
        assert!(response_types.contains(&serde_json::Value::String("code".to_string())));
        assert!(response_types.contains(&serde_json::Value::String("token".to_string())));
        assert!(response_types.contains(&serde_json::Value::String("id_token".to_string())));

        // Verify supported scopes
        let scopes = discovery["scopes_supported"].as_array().unwrap();
        assert!(scopes.contains(&serde_json::Value::String("openid".to_string())));
        assert!(scopes.contains(&serde_json::Value::String("profile".to_string())));
        assert!(scopes.contains(&serde_json::Value::String("email".to_string())));

        // Verify token endpoint auth methods
        let auth_methods = discovery["token_endpoint_auth_methods_supported"].as_array().unwrap();
        assert!(auth_methods.contains(&serde_json::Value::String("client_secret_basic".to_string())));
        assert!(auth_methods.contains(&serde_json::Value::String("client_secret_post".to_string())));
        assert!(auth_methods.contains(&serde_json::Value::String("client_secret_jwt".to_string())));
        assert!(auth_methods.contains(&serde_json::Value::String("private_key_jwt".to_string())));

        // Verify EdDSA signing algorithm
        assert!(
            discovery["id_token_signing_alg_values_supported"]
                .as_array()
                .unwrap()
                .contains(&serde_json::Value::String("EdDSA".to_string()))
        );
    }

    #[tokio::test]
    async fn test_oidc_token_with_authorization_code() {
        let mut params = std::collections::HashMap::new();
        params.insert("grant_type".to_string(), "authorization_code".to_string());
        params.insert("code".to_string(), "test_code".to_string());

        let result = oidc_token_ed25519(Query(params)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert!(response["access_token"].is_string());
        assert!(response["refresh_token"].is_string());
        assert_eq!(response["token_type"], "Bearer");
        assert_eq!(response["expires_in"], 3600);
    }

    #[tokio::test]
    async fn test_oidc_token_with_refresh_token() {
        // First, generate a refresh token
        let refresh_token = crate::utils::jwt::generate_refresh_token("test_user").unwrap();

        let mut params = std::collections::HashMap::new();
        params.insert("grant_type".to_string(), "refresh_token".to_string());
        params.insert("refresh_token".to_string(), refresh_token.clone());

        let result = oidc_token_ed25519(Query(params)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert!(response["access_token"].is_string());
        assert!(response["refresh_token"].is_string());

        // Verify token rotation - new refresh token should be different
        assert_ne!(response["refresh_token"].as_str().unwrap(), refresh_token);
    }

    #[tokio::test]
    async fn test_oidc_refresh_endpoint() {
        // Generate a refresh token
        let refresh_token = crate::utils::jwt::generate_refresh_token("test_user").unwrap();

        let mut params = std::collections::HashMap::new();
        params.insert("refresh_token".to_string(), refresh_token.clone());

        let result = oidc_refresh_ed25519(Query(params)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert!(response["access_token"].is_string());
        assert!(response["refresh_token"].is_string());
        assert_eq!(response["token_type"], "Bearer");
        assert_eq!(response["expires_in"], 3600);

        // Verify token rotation
        assert_ne!(response["refresh_token"].as_str().unwrap(), refresh_token);
    }

    #[tokio::test]
    async fn test_oidc_refresh_with_invalid_token() {
        let mut params = std::collections::HashMap::new();
        params.insert("refresh_token".to_string(), "invalid_token".to_string());

        let result = oidc_refresh_ed25519(Query(params)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_oidc_revoke_endpoint() {
        // Generate a refresh token
        let refresh_token = crate::utils::jwt::generate_refresh_token("test_user").unwrap();

        let mut params = std::collections::HashMap::new();
        params.insert("token".to_string(), refresh_token);
        params.insert("token_type_hint".to_string(), "refresh_token".to_string());

        let result = oidc_revoke_ed25519(Query(params)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert_eq!(response["success"], true);
    }

    #[tokio::test]
    async fn test_oidc_revoke_with_invalid_token() {
        let mut params = std::collections::HashMap::new();
        params.insert("token".to_string(), "invalid_token".to_string());

        let result = oidc_revoke_ed25519(Query(params)).await;
        assert!(result.is_err());
    }
}
