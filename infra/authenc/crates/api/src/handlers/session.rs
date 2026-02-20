//! Session management endpoint handlers

use std::sync::Arc;

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::ApiState;

/// Session information
#[derive(Debug, Serialize)]
pub struct SessionInfo {
    /// Session ID
    pub id: String,
    /// User ID
    pub user_id: Uuid,
    /// Session creation timestamp
    pub created_at: i64,
    /// Session expiration timestamp
    pub expires_at: i64,
    /// IP address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
    /// User agent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
}

/// List of sessions response
#[derive(Debug, Serialize)]
pub struct ListSessionsResponse {
    /// List of active sessions
    pub sessions: Vec<SessionInfo>,
}

/// Logout response
#[derive(Debug, Serialize)]
pub struct LogoutResponse {
    /// Success message
    pub message: String,
}

/// API error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    /// Error code
    pub error: String,
    /// Error message
    pub message: String,
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, Json(self)).into_response()
    }
}

/// Extract JWT token from Authorization header
fn extract_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}

/// GET /api/v1/sessions - List all sessions for current user
///
/// Returns a list of all active sessions for the authenticated user.
/// Requires valid JWT token in Authorization header.
pub async fn list_sessions_handler(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<ListSessionsResponse>, ErrorResponse> {
    // Extract JWT token from Authorization header
    let token = extract_token(&headers).ok_or_else(|| ErrorResponse {
        error: "unauthorized".to_string(),
        message: "Missing or invalid Authorization header".to_string(),
    })?;

    // Validate JWT token and extract user ID
    let claims = state
        .jwt_service
        .validate_token(&token)
        .map_err(|e| ErrorResponse {
            error: "unauthorized".to_string(),
            message: format!("Invalid token: {}", e),
        })?;

    let user_id = Uuid::parse_str(&claims.sub).map_err(|e| ErrorResponse {
        error: "invalid_user_id".to_string(),
        message: format!("Invalid user ID in token: {}", e),
    })?;

    // Get all sessions for user from session store
    let sessions = state
        .session_store
        .all_for_user(&user_id.to_string())
        .into_iter()
        .map(|session| SessionInfo {
            id: session.id,
            user_id,
            created_at: session.created_at,
            expires_at: session.expires_at,
            ip_address: session.ip_address,
            user_agent: session.user_agent,
        })
        .collect();

    Ok(Json(ListSessionsResponse { sessions }))
}

/// POST /api/v1/auth/logout - Logout and invalidate session
///
/// Invalidates the current session and removes it from the session store.
/// Requires valid JWT token in Authorization header.
pub async fn logout_handler(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<LogoutResponse>, ErrorResponse> {
    // Extract JWT token from Authorization header
    let token = extract_token(&headers).ok_or_else(|| ErrorResponse {
        error: "unauthorized".to_string(),
        message: "Missing or invalid Authorization header".to_string(),
    })?;

    // Remove session from session store
    // Note: We don't validate the token here because even if it's expired,
    // we still want to remove the session
    if let Err(e) = state.session_store.remove(&token) {
        tracing::warn!("Failed to remove session: {}", e);
        // Don't fail the logout request if session removal fails
        // The token is already invalid on the client side
    }

    Ok(Json(LogoutResponse {
        message: "Successfully logged out".to_string(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn test_extract_token_valid() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            HeaderValue::from_static("Bearer test_token_123"),
        );

        let token = extract_token(&headers);
        assert_eq!(token, Some("test_token_123".to_string()));
    }

    #[test]
    fn test_extract_token_missing() {
        let headers = HeaderMap::new();
        let token = extract_token(&headers);
        assert_eq!(token, None);
    }

    #[test]
    fn test_extract_token_invalid_format() {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", HeaderValue::from_static("InvalidFormat"));

        let token = extract_token(&headers);
        assert_eq!(token, None);
    }

    #[test]
    fn test_session_info_serialization() {
        let session = SessionInfo {
            id: "session123".to_string(),
            user_id: Uuid::new_v4(),
            created_at: 1234567890,
            expires_at: 1234567890 + 3600,
            ip_address: Some("192.168.1.1".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
        };

        let json = serde_json::to_string(&session).unwrap();
        assert!(json.contains("session123"));
        assert!(json.contains("192.168.1.1"));
    }

    #[test]
    fn test_logout_response_serialization() {
        let response = LogoutResponse {
            message: "Successfully logged out".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("Successfully logged out"));
    }
}
</content>
