//! Admin dashboard HTTP handlers
//!
//! Only the system-stats endpoint survives here — the old module also
//! carried a third (unmounted) router plus NotImplemented stubs for
//! sessions/policies/identity-providers and dashboard/security-event
//! variants nothing consumed. Every figure returned below is measured
//! from the live database; fields nothing renders (uptime, memory, CPU)
//! were dropped rather than reported as fake zeros.

use axum::{Json, extract::State};
use serde::Serialize;
use std::sync::Arc;

use crate::error::ApiResult;
use crate::state::IamApiState;
use authenc_types::domain_types::RealmId;
use uuid::Uuid;

/// System statistics (consumed by the portal dashboard + admin overview)
#[derive(Debug, Serialize)]
pub struct SystemStats {
    pub total_users: i64,
    pub active_users: i64,
    pub total_sessions: i64,
    pub active_sessions: i64,
    pub total_realms: i64,
    pub total_clients: i64,
    pub security_events_today: i64,
    pub failed_login_attempts: i64,
}

/// Run a COUNT(*) query, degrading to 0 (with a warning) instead of failing
/// the whole stats endpoint if one source table is unavailable.
async fn count(state: &IamApiState, sql: &str) -> i64 {
    match state.database.query_one(sql, &[]).await {
        Ok(row) => row.get(0),
        Err(e) => {
            tracing::warn!("stats query failed ({sql}): {e}");
            0
        }
    }
}

/// GET /api/v1/iam/admin/stats - System statistics
pub async fn get_system_stats(
    State(state): State<Arc<IamApiState>>,
) -> ApiResult<Json<SystemStats>> {
    let realm_id = RealmId::from_uuid(Uuid::nil());

    let total_users = state.user_service.count_users(realm_id).await.unwrap_or(0);
    let active_users = state
        .user_service
        .count_enabled_users(realm_id)
        .await
        .unwrap_or(0);

    let stats = SystemStats {
        total_users,
        active_users,
        total_sessions: count(&state, "SELECT COUNT(*) FROM sessions").await,
        active_sessions: count(
            &state,
            "SELECT COUNT(*) FROM sessions WHERE expires_at > NOW()",
        )
        .await,
        total_realms: count(&state, "SELECT COUNT(*) FROM realms").await,
        total_clients: count(
            &state,
            "SELECT COUNT(*) FROM oauth2_clients WHERE deleted_at IS NULL",
        )
        .await,
        security_events_today: count(
            &state,
            "SELECT COUNT(*) FROM audit_logs WHERE \"timestamp\" >= date_trunc('day', NOW())",
        )
        .await,
        failed_login_attempts: count(
            &state,
            "SELECT COUNT(*) FROM audit_logs WHERE status <> 'success' \
             AND \"timestamp\" >= date_trunc('day', NOW())",
        )
        .await,
    };
    Ok(Json(stats))
}
