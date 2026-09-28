//! REST handlers for workflow delegation management.
//!
//! Wraps [`super::delegation::DelegationManager`] with axum extractors and
//! the same `ApiResponse<T>` envelope used by the rest of the workflow
//! surface. These endpoints back the `/admin/workflow/delegation` page in
//! the Perlengkapan MFE.
//!
//! Auth: handlers read `Claims::sub` to identify the caller. The frontend
//! is responsible for restricting access to admin-realm roles; backend
//! validation happens in the JWT middleware before the handler runs.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::middleware::Claims;

use super::delegation::{CreateDelegationRequest, Delegation, DelegationError, DelegationManager};
use super::handlers::ApiResponse;
use crate::AppState;

// ---------------------------------------------------------------------------
// Request / query DTOs
// ---------------------------------------------------------------------------

/// Body for `POST /workflow/delegations`. Mirrors
/// [`CreateDelegationRequest`] but accepts `valid_from` as `Option` so the
/// caller can omit it for "starts immediately" — common in the UI.
#[derive(Debug, Deserialize)]
pub struct CreateDelegationBody {
    pub delegate_user_id: Uuid,
    pub role: String,
    #[serde(default)]
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_until: DateTime<Utc>,
    #[serde(default)]
    pub reason: Option<String>,
}

/// Query string for `GET /workflow/delegations`. Default behaviour
/// (no filter) returns delegations *created by* the caller — the "what
/// did I delegate?" view. `as_delegate=true` flips that to "what is
/// delegated to me?".
#[derive(Debug, Default, Deserialize)]
pub struct ListDelegationsQuery {
    #[serde(default)]
    pub as_delegate: bool,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `POST /workflow/delegations` — create a delegation. The delegator is
/// always the authenticated caller; the body cannot spoof a different
/// `delegator_user_id`.
pub async fn create_delegation_handler(
    State(state): State<AppState>,
    claims: Claims,
    Json(body): Json<CreateDelegationBody>,
) -> Result<impl IntoResponse, DelegationApiError> {
    let manager = DelegationManager::new(state.db_pool.clone());

    let req = CreateDelegationRequest {
        delegator_user_id: claims.user_id,
        delegate_user_id: body.delegate_user_id,
        role: body.role,
        valid_from: body.valid_from.unwrap_or_else(Utc::now),
        valid_until: body.valid_until,
        reason: body.reason,
    };

    let delegation = manager.create_delegation(req).await?;
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            delegation,
            "Delegasi berhasil dibuat".to_string(),
        )),
    ))
}

/// `GET /workflow/delegations` — list delegations the caller is involved in.
pub async fn list_delegations_handler(
    State(state): State<AppState>,
    claims: Claims,
    Query(query): Query<ListDelegationsQuery>,
) -> Result<impl IntoResponse, DelegationApiError> {
    let manager = DelegationManager::new(state.db_pool.clone());

    let delegations: Vec<Delegation> = if query.as_delegate {
        manager
            .get_active_delegations_for_user(claims.user_id)
            .await?
    } else {
        manager.get_delegations_by_delegator(claims.user_id).await?
    };

    Ok(Json(ApiResponse::success(delegations, "Success")))
}

/// `POST /workflow/delegations/{id}/revoke` — caller (must be the
/// original delegator) revokes an active or scheduled delegation.
pub async fn revoke_delegation_handler(
    State(state): State<AppState>,
    claims: Claims,
    Path(delegation_id): Path<Uuid>,
) -> Result<impl IntoResponse, DelegationApiError> {
    let manager = DelegationManager::new(state.db_pool.clone());
    manager
        .revoke_delegation(delegation_id, claims.user_id)
        .await?;
    Ok(Json(ApiResponse::<()>::success(
        (),
        "Delegasi berhasil dicabut".to_string(),
    )))
}

// ---------------------------------------------------------------------------
// Error mapping
// ---------------------------------------------------------------------------

/// REST-facing error for the delegation endpoints. Maps the rich
/// [`DelegationError`] variants onto stable HTTP shapes so the frontend
/// doesn't need to special-case backend error wording.
#[derive(Debug, Serialize)]
pub struct DelegationApiError {
    pub error: String,
    pub message: String,
    #[serde(skip)]
    pub status: StatusCode,
}

impl IntoResponse for DelegationApiError {
    fn into_response(self) -> axum::response::Response {
        (self.status, Json(self)).into_response()
    }
}

impl From<DelegationError> for DelegationApiError {
    fn from(value: DelegationError) -> Self {
        match value {
            DelegationError::InvalidRequest(msg) => Self {
                error: "invalid_request".to_string(),
                message: msg,
                status: StatusCode::BAD_REQUEST,
            },
            DelegationError::DelegationNotFound(_) => Self {
                error: "not_found".to_string(),
                message: "Delegasi tidak ditemukan".to_string(),
                status: StatusCode::NOT_FOUND,
            },
            DelegationError::Unauthorized(msg) => Self {
                error: "unauthorized".to_string(),
                message: msg,
                status: StatusCode::FORBIDDEN,
            },
            DelegationError::InvalidStatus(msg) => Self {
                error: "invalid_status".to_string(),
                message: msg,
                status: StatusCode::CONFLICT,
            },
            other => Self {
                error: "internal".to_string(),
                message: other.to_string(),
                status: StatusCode::INTERNAL_SERVER_ERROR,
            },
        }
    }
}
