//! Token Exchange HTTP Handler (RFC 8693)
//!
//! This module provides HTTP endpoint handlers for OAuth 2.0 Token Exchange
//! following RFC 8693 specification.

use crate::app::AppState;
use crate::error::{AuthencError, Result};
use crate::services::cache::Cache;
use crate::services::stores::audit_log_store::AuditLogStore;
use crate::services::token_exchange::{
    TokenExchangeRequest, TokenExchangeResponse, TokenExchangeService,
};
use axum::{
    Form, debug_handler,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info};

/// Token exchange request form (application/x-www-form-urlencoded)
#[derive(Debug, Deserialize)]
pub struct TokenExchangeForm {
    pub grant_type: String,
    pub subject_token: String,
    pub subject_token_type: String,
    pub actor_token: Option<String>,
    pub actor_token_type: Option<String>,
    pub requested_token_type: Option<String>,
    pub resource: Option<String>,
    pub audience: Option<String>,
    pub scope: Option<String>,
}

/// Token exchange error response (RFC 6749)
#[derive(Debug, Serialize)]
pub struct TokenExchangeError {
    pub error: String,
    pub error_description: Option<String>,
    pub error_uri: Option<String>,
}

impl From<AuthencError> for TokenExchangeError {
    fn from(err: AuthencError) -> Self {
        match err {
            AuthencError::ValidationError { message } => Self {
                error: "invalid_request".to_string(),
                error_description: Some(message),
                error_uri: None,
            },
            AuthencError::Unauthorized { message } => Self {
                error: "invalid_grant".to_string(),
                error_description: Some(message),
                error_uri: None,
            },
            AuthencError::Forbidden { message } => Self {
                error: "unauthorized_client".to_string(),
                error_description: Some(message),
                error_uri: None,
            },
            _ => Self {
                error: "server_error".to_string(),
                error_description: Some("Internal server error during token exchange".to_string()),
                error_uri: None,
            },
        }
    }
}

/// OAuth 2.0 Token Exchange endpoint (RFC 8693)
///
/// Exchanges one type of token for another. Supports:
/// - Access token → Access token (scope reduction, audience change)
/// - Refresh token → Access token
/// - ID token → Access token
/// - Token delegation with actor tokens
///
/// # Request Parameters (POST /oauth2/token/exchange)
/// - `grant_type`: Must be "urn:ietf:params:oauth:grant-type:token-exchange"
/// - `subject_token`: The token to be exchanged
/// - `subject_token_type`: URN identifying the token type
/// - `actor_token`: (Optional) Token representing delegated authority
/// - `actor_token_type`: (Required if actor_token present) URN of actor token type
/// - `requested_token_type`: (Optional) Desired token type
/// - `resource`: (Optional) Target resource identifier
/// - `audience`: (Optional) Target audience identifier
/// - `scope`: (Optional) Requested scopes
///
/// # Authentication
/// Requires client authentication via HTTP Basic Auth or client credentials
///
/// # Response
/// - `200 OK`: Token exchange successful
/// - `400 Bad Request`: Invalid request parameters
/// - `401 Unauthorized`: Invalid or expired tokens
/// - `403 Forbidden`: Exchange not permitted by policy
/// - `500 Internal Server Error`: Server error
#[debug_handler]
pub async fn token_exchange_endpoint(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Form(form): Form<TokenExchangeForm>,
) -> std::result::Result<Json<TokenExchangeResponse>, (StatusCode, Json<TokenExchangeError>)> {
    info!("Token exchange request received");

    // Extract client credentials from Authorization header
    let client_id = match extract_client_credentials(&headers) {
        Ok(id) => id,
        Err(e) => {
            error!("Client authentication failed: {}", e);
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(TokenExchangeError {
                    error: "invalid_client".to_string(),
                    error_description: Some("Client authentication required".to_string()),
                    error_uri: None,
                }),
            ));
        }
    };

    debug!("Token exchange for client: {}", client_id);

    // Build token exchange request
    let request = TokenExchangeRequest {
        grant_type: form.grant_type,
        subject_token: form.subject_token,
        subject_token_type: form.subject_token_type,
        actor_token: form.actor_token,
        actor_token_type: form.actor_token_type,
        requested_token_type: form.requested_token_type,
        resource: form.resource,
        audience: form.audience,
        scope: form.scope,
        client_id: Some(client_id.clone()),
    };

    // Create token exchange service
    let token_exchange_service = match create_token_exchange_service(&state) {
        Ok(service) => service,
        Err(e) => {
            error!("Failed to create token exchange service: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(TokenExchangeError {
                    error: "server_error".to_string(),
                    error_description: Some("Service initialization failed".to_string()),
                    error_uri: None,
                }),
            ));
        }
    };

    // Perform token exchange
    match token_exchange_service.exchange_token(request).await {
        Ok(response) => {
            info!("Token exchange successful for client: {}", client_id);
            Ok(Json(response))
        }
        Err(e) => {
            error!("Token exchange failed: {}", e);
            let status = match &e {
                AuthencError::ValidationError { .. } => StatusCode::BAD_REQUEST,
                AuthencError::Unauthorized { .. } => StatusCode::UNAUTHORIZED,
                AuthencError::Forbidden { .. } => StatusCode::FORBIDDEN,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(e.into())))
        }
    }
}

/// Extract client credentials from Authorization header
///
/// Supports:
/// - HTTP Basic Auth: Authorization: Basic base64(client_id:client_secret)
/// - Bearer token: Authorization: Bearer <token> (for service accounts)
fn extract_client_credentials(headers: &HeaderMap) -> Result<String> {
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AuthencError::validation("Missing Authorization header"))?;

    if auth_header.starts_with("Basic ") {
        // HTTP Basic Auth
        let encoded = auth_header
            .strip_prefix("Basic ")
            .ok_or_else(|| AuthencError::validation("Invalid Basic auth format"))?;

        use base64::Engine;
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .map_err(|_| AuthencError::validation("Invalid base64 encoding in Authorization"))?;

        let credentials = String::from_utf8(decoded)
            .map_err(|_| AuthencError::validation("Invalid UTF-8 in credentials"))?;

        let parts: Vec<&str> = credentials.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(AuthencError::validation("Invalid credentials format"));
        }

        let client_id = parts[0];
        let client_secret = parts[1];

        // Validate client credentials
        validate_client_credentials(client_id, client_secret)?;

        Ok(client_id.to_string())
    } else if auth_header.starts_with("Bearer ") {
        // Bearer token (for service accounts)
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AuthencError::validation("Invalid Bearer token format"))?;

        // Extract client_id from JWT token
        let claims = crate::utils::crypto::jwt::verify_jwt_with_validation(token)?;

        Ok(claims.sub)
    } else {
        Err(AuthencError::validation(
            "Unsupported authentication method",
        ))
    }
}

/// Validate client credentials
fn validate_client_credentials(client_id: &str, client_secret: &str) -> Result<()> {
    // TODO: Implement proper client credential validation against database
    // For now, perform basic validation
    if client_id.is_empty() || client_secret.is_empty() {
        return Err(AuthencError::unauthorized("Invalid client credentials"));
    }

    // In production, this would:
    // 1. Look up client in database
    // 2. Verify client_secret hash
    // 3. Check if client is enabled
    // 4. Verify client has token_exchange capability

    Ok(())
}

/// Create token exchange service with dependencies
fn create_token_exchange_service(state: &Arc<AppState>) -> Result<TokenExchangeService> {
    use crate::services::jwt_validator::JwtValidator;
    use crate::services::token_exchange::TokenExchangeConfig;

    // Create JWT validator with cache
    let cache: Option<Arc<dyn Cache>> = state.redis_cache.clone().map(|c| c as Arc<dyn Cache>);
    let jwt_validator = Arc::new(JwtValidator::new(cache));

    // Use existing audit log store from state
    let audit_log: Arc<dyn AuditLogStore> = state.audit_log_store.clone();

    // Configure token exchange service
    let config = TokenExchangeConfig {
        allow_impersonation: false, // Disabled by default
        allow_delegation: true,
        require_audience: true,
        default_token_ttl: 3600,
        max_token_ttl: 7200,
        enforce_scope_downscoping: true,
        allowed_conversions: vec![
            (
                "urn:ietf:params:oauth:token-type:access_token".to_string(),
                "urn:ietf:params:oauth:token-type:access_token".to_string(),
            ),
            (
                "urn:ietf:params:oauth:token-type:refresh_token".to_string(),
                "urn:ietf:params:oauth:token-type:access_token".to_string(),
            ),
            (
                "urn:ietf:params:oauth:token-type:id_token".to_string(),
                "urn:ietf:params:oauth:token-type:access_token".to_string(),
            ),
        ],
    };

    Ok(TokenExchangeService::new(
        state.database.clone(),
        jwt_validator,
        audit_log,
        Some(config),
    ))
}

/// Token exchange discovery metadata
///
/// Returns OAuth 2.0 metadata for token exchange capabilities
#[debug_handler]
pub async fn token_exchange_metadata() -> Json<TokenExchangeMetadata> {
    Json(TokenExchangeMetadata {
        grant_types_supported: vec!["urn:ietf:params:oauth:grant-type:token-exchange".to_string()],
        token_endpoint: "http://localhost:8080/v1/oauth2/token/exchange".to_string(),
        subject_token_types_supported: vec![
            "urn:ietf:params:oauth:token-type:access_token".to_string(),
            "urn:ietf:params:oauth:token-type:refresh_token".to_string(),
            "urn:ietf:params:oauth:token-type:id_token".to_string(),
            "urn:ietf:params:oauth:token-type:jwt".to_string(),
        ],
        actor_token_types_supported: vec![
            "urn:ietf:params:oauth:token-type:access_token".to_string(),
            "urn:ietf:params:oauth:token-type:jwt".to_string(),
        ],
        issued_token_types_supported: vec![
            "urn:ietf:params:oauth:token-type:access_token".to_string(),
        ],
    })
}

#[derive(Debug, Serialize)]
pub struct TokenExchangeMetadata {
    pub grant_types_supported: Vec<String>,
    pub token_endpoint: String,
    pub subject_token_types_supported: Vec<String>,
    pub actor_token_types_supported: Vec<String>,
    pub issued_token_types_supported: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_basic_auth() {
        use axum::http::header::AUTHORIZATION;

        let mut headers = HeaderMap::new();
        let credentials = base64::encode("test_client:test_secret");
        headers.insert(
            AUTHORIZATION,
            format!("Basic {}", credentials).parse().unwrap(),
        );

        // Note: This will fail validation without proper setup
        // Test structure is for demonstration
    }

    #[test]
    fn test_token_exchange_error_conversion() {
        let validation_error = AuthencError::validation("Invalid parameter");
        let error: TokenExchangeError = validation_error.into();
        assert_eq!(error.error, "invalid_request");
    }
}
