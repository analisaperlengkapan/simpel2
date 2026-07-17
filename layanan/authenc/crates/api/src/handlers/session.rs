//! Self-service session management (REQ-PORTAL-008).
//!
//! The caller manages their OWN sessions only — identity always comes from the
//! verified JWT (`sub` + `sid`), never from the request body. Admin-wide
//! session management is a separate iam-api concern.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use serde::Serialize;
use uuid::Uuid;

use crate::state::ApiState;

use super::auth::ErrorResponse;
use super::auth_helpers::extract_bearer_token;

/// One active session of the calling user, shaped for the portal Sesi Aktif
/// page (`antarmuka/portal` `SessionInfo`).
#[derive(Debug, Serialize)]
pub struct SessionInfo {
    pub id: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: String,
    pub last_active: String,
    pub expires_at: String,
    /// True when this row is the session the presented token belongs to.
    pub is_current: bool,
}

#[derive(Debug, Serialize)]
pub struct ListSessionsResponse {
    pub sessions: Vec<SessionInfo>,
}

/// Verify the bearer token and return `(user_id, sid)`.
fn caller_identity(
    state: &Arc<ApiState>,
    headers: &HeaderMap,
) -> Result<(Uuid, Option<String>), ErrorResponse> {
    let token = extract_bearer_token(headers).map_err(|e| ErrorResponse {
        status_code: axum::http::StatusCode::UNAUTHORIZED,
        error: "unauthorized".to_string(),
        message: e.message,
    })?;

    let claims = state
        .jwt_service
        .verify_token(&token)
        .map_err(|e| ErrorResponse {
            status_code: axum::http::StatusCode::UNAUTHORIZED,
            error: "unauthorized".to_string(),
            message: format!("Invalid token: {}", e),
        })?;

    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| ErrorResponse {
        status_code: axum::http::StatusCode::UNAUTHORIZED,
        error: "unauthorized".to_string(),
        message: "Invalid user ID in token".to_string(),
    })?;

    Ok((user_id, claims.sid))
}

/// GET /api/v1/auth/sessions — list the caller's active sessions.
pub async fn list_sessions_handler(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<ListSessionsResponse>, ErrorResponse> {
    let (user_id, sid) = caller_identity(&state, &headers)?;

    let rows = state
        .database
        .query(
            r#"
            SELECT id, ip_address, user_agent, created_at, last_activity_at, expires_at
            FROM sessions
            WHERE user_id = $1 AND expires_at > NOW()
            ORDER BY last_activity_at DESC
            "#,
            &[&user_id],
        )
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to list sessions");
            ErrorResponse {
                status_code: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                error: "internal_error".to_string(),
                message: "Gagal memuat daftar sesi.".to_string(),
            }
        })?;

    let sessions = rows
        .iter()
        .map(|row| {
            let id: Uuid = row.get("id");
            SessionInfo {
                id: id.to_string(),
                ip_address: row
                    .get::<_, Option<std::net::IpAddr>>("ip_address")
                    .map(|ip| ip.to_string()),
                user_agent: row.get("user_agent"),
                created_at: row
                    .get::<_, chrono::DateTime<chrono::Utc>>("created_at")
                    .to_rfc3339(),
                last_active: row
                    .get::<_, chrono::DateTime<chrono::Utc>>("last_activity_at")
                    .to_rfc3339(),
                expires_at: row
                    .get::<_, chrono::DateTime<chrono::Utc>>("expires_at")
                    .to_rfc3339(),
                is_current: sid.as_deref() == Some(id.to_string().as_str()),
            }
        })
        .collect();

    Ok(Json(ListSessionsResponse { sessions }))
}

/// DELETE /api/v1/auth/sessions/{id} — terminate one of the caller's OTHER
/// sessions. The current session must end via logout (revokes the refresh
/// token too); a foreign or unknown id returns 404 so session existence is
/// never leaked across users.
pub async fn terminate_session_handler(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Path(session_id): Path<String>,
) -> Result<axum::http::StatusCode, ErrorResponse> {
    let (user_id, sid) = caller_identity(&state, &headers)?;

    let target = Uuid::parse_str(&session_id).map_err(|_| ErrorResponse {
        status_code: axum::http::StatusCode::NOT_FOUND,
        error: "not_found".to_string(),
        message: "Sesi tidak ditemukan.".to_string(),
    })?;

    if sid.as_deref() == Some(session_id.as_str()) {
        return Err(ErrorResponse {
            status_code: axum::http::StatusCode::BAD_REQUEST,
            error: "invalid_request".to_string(),
            message: "Gunakan logout untuk mengakhiri sesi saat ini.".to_string(),
        });
    }

    let row = state
        .database
        .query(
            "SELECT user_id, expires_at FROM sessions WHERE id = $1",
            &[&target],
        )
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to look up session");
            ErrorResponse {
                status_code: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                error: "internal_error".to_string(),
                message: "Gagal menghentikan sesi.".to_string(),
            }
        })?;

    let Some(row) = row.first() else {
        return Err(not_found());
    };
    if row.get::<_, Uuid>("user_id") != user_id {
        return Err(not_found());
    }
    let expires_at: chrono::DateTime<chrono::Utc> = row.get("expires_at");

    // Revoke first (fail-closed for any access token carrying this sid), then
    // drop the session row so refresh + listing stop seeing it.
    if let Err(e) = state
        .revocation_store
        .revoke_session(&session_id, expires_at, Some("user_terminated"))
        .await
    {
        tracing::warn!(error = %e, "Failed to record session revocation");
    }
    {
        use authenc_types::traits::AuthenticationService;
        let _ = state
            .auth_service
            .logout(authenc_types::SessionId(target))
            .await;
    }

    Ok(axum::http::StatusCode::NO_CONTENT)
}

fn not_found() -> ErrorResponse {
    ErrorResponse {
        status_code: axum::http::StatusCode::NOT_FOUND,
        error: "not_found".to_string(),
        message: "Sesi tidak ditemukan.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_info_serializes_portal_shape() {
        let session = SessionInfo {
            id: "session123".to_string(),
            ip_address: Some("192.168.1.1".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
            created_at: "2026-01-01T00:00:00+00:00".to_string(),
            last_active: "2026-01-01T01:00:00+00:00".to_string(),
            expires_at: "2026-01-02T00:00:00+00:00".to_string(),
            is_current: true,
        };

        let json = serde_json::to_string(&session).unwrap();
        assert!(json.contains("session123"));
        assert!(json.contains("192.168.1.1"));
        assert!(json.contains("\"is_current\":true"));
        assert!(json.contains("last_active"));
    }
}
