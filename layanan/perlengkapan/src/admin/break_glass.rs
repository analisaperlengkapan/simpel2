//! # Break-glass endpoints
//!
//! `POST /admin/break-glass/{module}/{id}/transition` — the audited emergency
//! override — and `GET /admin/break-glass` — its review log.
//!
//! The rules (mandatory reason, log-before-act, tagged note) are described in
//! [`crate::shared::break_glass`], which holds the parts that need no database.
//! This file is the part that does: read the current state, write the log row,
//! run the transition through the SAME service method the normal endpoints use
//! (so the workflow's own validity rules — an unreachable state is still
//! refused — keep applying), then settle the log row.
//!
//! Break-glass moves an entity along an edge the workflow defines; it does not
//! invent edges. Being an administrator lets you take a decision out of a stuck
//! queue, not take a state the process could never have reached.

use axum::{
    Json,
    extract::{Path, Query, State},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::penghapusan_bmn::services::TransitionActor;
use crate::shared::break_glass::{BreakGlassModule, BreakGlassRequest, ValidBreakGlass};
use crate::shared::error::{AppError, AppResult};
use crate::shared::middleware::{Claims, ClientIp};
use crate::shared::satker_scope::SatkerScope;
use crate::state::AppState;
use crate::workflow::engine::INTERNAL_ACTOR_ROLE;
use lib_perlengkapan::audit::{AuditAction, AuditEvent};
use lib_perlengkapan::response::ApiResponse;

/// What a successful break-glass call returns.
#[derive(Debug, Serialize)]
pub struct BreakGlassResult {
    pub module: &'static str,
    pub entity_id: Uuid,
    pub from_state: String,
    pub to_state: String,
    /// The `break_glass_log` row this action is recorded in.
    pub log_id: Uuid,
}

/// POST /admin/break-glass/{module}/{id}/transition
pub async fn break_glass_transition(
    State(state): State<AppState>,
    claims: Claims,
    ClientIp(ip): ClientIp,
    Path((module, id)): Path<(String, Uuid)>,
    Json(request): Json<BreakGlassRequest>,
) -> AppResult<Json<ApiResponse<BreakGlassResult>>> {
    // The administrator surface: `Capability::Administer`, and nothing else.
    // Validator Pusat may READ the log (below) but cannot use the override —
    // overriding a colleague's decision is not their job either.
    claims.require_admin()?;
    let module = BreakGlassModule::parse(&module)?;
    let request = request.validated()?;

    let from_state = current_state(&state.db_pool, module, id).await?;
    let log_id = open_log(
        &state.db_pool,
        module,
        id,
        &from_state,
        &request,
        &claims,
        &ip,
    )
    .await?;

    let outcome = run_transition(&state, module, id, &request, &claims, &ip).await;

    settle_log(&state.db_pool, log_id, &outcome).await;
    audit(
        &state,
        module,
        id,
        &from_state,
        &request,
        &claims,
        &ip,
        &outcome,
    )
    .await;

    match outcome {
        Ok(()) => Ok(Json(ApiResponse::success(
            BreakGlassResult {
                module: module.as_str(),
                entity_id: id,
                from_state,
                to_state: request.target_status,
                log_id,
            },
            "Break-glass berhasil dijalankan dan dicatat".to_string(),
        ))),
        Err(e) => Err(e),
    }
}

/// Run the transition as the engine's internal actor, attributed to the
/// administrator's own user id and tagged in the note.
async fn run_transition(
    state: &AppState,
    module: BreakGlassModule,
    id: Uuid,
    request: &ValidBreakGlass,
    claims: &Claims,
    ip: &str,
) -> AppResult<()> {
    let note = request.activity_note();
    let role = INTERNAL_ACTOR_ROLE.to_string();
    // An administrator's break-glass reaches every satker's records.
    let scope = SatkerScope::All;

    match module {
        BreakGlassModule::PemakaianBmn => {
            state
                .pemakaian_bmn_service
                .transition_permit_status(
                    id,
                    crate::pemakaian_bmn::models::WorkflowTransitionRequest {
                        target_status: request.target_status.clone(),
                        catatan: Some(note),
                    },
                    claims.user_id,
                    role,
                    ip.to_string(),
                    &scope,
                )
                .await?;
        }
        BreakGlassModule::PenghapusanBmn => {
            state
                .penghapusan_bmn_service
                .transition(
                    id,
                    request.target_status.clone(),
                    TransitionActor {
                        user_id: claims.user_id,
                        user_role: role,
                        catatan: Some(note),
                        ip_address: ip.to_string(),
                    },
                    &scope,
                )
                .await?;
        }
        BreakGlassModule::KebutuhanBmn => {
            let status = crate::kebutuhan_bmn::models::KebutuhanBmnStatus::from_state_name(
                &request.target_status,
            )
            .ok_or_else(|| {
                AppError::BadRequest(format!(
                    "Status '{}' tidak dikenal untuk kebutuhan_bmn",
                    request.target_status
                ))
            })?;
            state
                .kebutuhan_bmn_service
                .transition_pengajuan_status(
                    id,
                    crate::kebutuhan_bmn::models::WorkflowTransitionRequest {
                        target_status: status.to_code(),
                        komentar: Some(note),
                    },
                    Some(claims.user_id),
                    Some(crate::kebutuhan_bmn::repository::UserInfo {
                        nip: claims.nip.clone(),
                        nama: claims.nama.clone(),
                        pangkat: None,
                        jabatan: claims.jabatan.clone(),
                        role: Some(INTERNAL_ACTOR_ROLE.to_string()),
                    }),
                    role,
                    ip.to_string(),
                )
                .await?;
        }
    }
    Ok(())
}

/// The entity's current canonical state, or 404.
async fn current_state(
    pool: &deadpool_postgres::Pool,
    module: BreakGlassModule,
    id: Uuid,
) -> AppResult<String> {
    let client = pool.get().await?;
    // `status_table` is a fixed literal per module, not caller input.
    let sql = format!(
        "SELECT status FROM perlengkapan.{} WHERE id = $1",
        module.status_table()
    );
    let row = client.query_opt(&sql, &[&id]).await?;
    row.map(|r| r.get::<_, String>("status"))
        .ok_or_else(|| AppError::NotFound(format!("{} {id} tidak ditemukan", module.as_str())))
}

/// Write the log row BEFORE acting. If this fails the caller gets the error and
/// the transition never runs: an override that cannot be recorded is not made.
async fn open_log(
    pool: &deadpool_postgres::Pool,
    module: BreakGlassModule,
    id: Uuid,
    from_state: &str,
    request: &ValidBreakGlass,
    claims: &Claims,
    ip: &str,
) -> AppResult<Uuid> {
    let client = pool.get().await?;
    let roles: Vec<String> = claims.role_set().iter().map(str::to_string).collect();
    let row = client
        .query_one(
            "INSERT INTO perlengkapan.break_glass_log
                 (module, entity_id, from_state, to_state, reason, reference,
                  actor_user_id, actor_username, actor_roles, ip_address)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
             RETURNING id",
            &[
                &module.as_str(),
                &id,
                &from_state,
                &request.target_status,
                &request.alasan,
                &request.referensi,
                &claims.user_id,
                &claims.username,
                &roles,
                &ip,
            ],
        )
        .await?;
    Ok(row.get("id"))
}

/// Settle the row to `success` / `failure`. Best-effort by necessity — the
/// transition has already happened — but a row that stays `pending` is itself
/// visible to a reviewer, so a failure here is loud rather than silent.
async fn settle_log(pool: &deadpool_postgres::Pool, log_id: Uuid, outcome: &AppResult<()>) {
    let (state, error) = match outcome {
        Ok(()) => ("success", None),
        Err(e) => ("failure", Some(e.to_string())),
    };
    let result = async {
        let client = pool.get().await?;
        client
            .execute(
                "UPDATE perlengkapan.break_glass_log
                    SET outcome = $2, error = $3, settled_at = now()
                  WHERE id = $1",
                &[&log_id, &state, &error],
            )
            .await?;
        Ok::<_, AppError>(())
    }
    .await;
    if let Err(e) = result {
        tracing::error!(%log_id, error = %e, "break-glass: could not settle log row (left 'pending')");
    }
}

/// Mirror the action into the cross-module audit trail (`/audit`), where the
/// people who read it already look.
#[allow(clippy::too_many_arguments)]
async fn audit(
    state: &AppState,
    module: BreakGlassModule,
    id: Uuid,
    from_state: &str,
    request: &ValidBreakGlass,
    claims: &Claims,
    ip: &str,
    outcome: &AppResult<()>,
) {
    let mut event = AuditEvent::new(module.as_str(), AuditAction::Custom, module.status_table())
        .actor(claims.user_id, claims.username.clone())
        .ip(ip.to_string())
        .resource_id(id.to_string())
        .action_name("workflow.break_glass")
        .metadata(serde_json::json!({
            "from_state": from_state,
            "to_state": request.target_status,
            "alasan": request.alasan,
            "referensi": request.referensi,
        }));
    event = match outcome {
        Ok(()) => event.message(request.activity_note()),
        Err(e) => event.failure(e.to_string()),
    };
    if let Err(e) = state.audit_sink.log(event).await {
        tracing::error!(error = %e, "break-glass: audit sink write failed");
    }
}

// ---------------------------------------------------------------------------
// Review log
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct BreakGlassListQuery {
    pub module: Option<String>,
    pub entity_id: Option<Uuid>,
    /// Newest first; default 50, at most 200.
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct BreakGlassEntry {
    pub id: Uuid,
    pub module: String,
    pub entity_id: Uuid,
    pub from_state: Option<String>,
    pub to_state: String,
    pub outcome: String,
    pub error: Option<String>,
    pub reason: String,
    pub reference: Option<String>,
    pub actor_user_id: Uuid,
    pub actor_username: Option<String>,
    pub actor_roles: Vec<String>,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
    pub settled_at: Option<DateTime<Utc>>,
}

/// GET /admin/break-glass
///
/// Open to administrators AND Validator Pusat: the second are the people whose
/// decisions an override replaces, and a review log nobody on the other side of
/// the override can read is not oversight.
pub async fn list_break_glass(
    State(pool): State<deadpool_postgres::Pool>,
    claims: Claims,
    Query(query): Query<BreakGlassListQuery>,
) -> AppResult<Json<ApiResponse<Vec<BreakGlassEntry>>>> {
    claims.require_any_role_or_admin(&["validator_pusat"])?;

    let module = query
        .module
        .as_deref()
        .map(BreakGlassModule::parse)
        .transpose()?
        .map(BreakGlassModule::as_str);
    let limit = query.limit.unwrap_or(50).clamp(1, 200);

    let client = pool.get().await?;
    let rows = client
        .query(
            "SELECT id, module, entity_id, from_state, to_state, outcome, error, reason,
                    reference, actor_user_id, actor_username, actor_roles, ip_address,
                    created_at, settled_at
               FROM perlengkapan.break_glass_log
              WHERE ($1::text IS NULL OR module = $1)
                AND ($2::uuid IS NULL OR entity_id = $2)
              ORDER BY created_at DESC
              LIMIT $3",
            &[&module, &query.entity_id, &limit],
        )
        .await?;

    let entries = rows
        .iter()
        .map(|r| BreakGlassEntry {
            id: r.get("id"),
            module: r.get("module"),
            entity_id: r.get("entity_id"),
            from_state: r.get("from_state"),
            to_state: r.get("to_state"),
            outcome: r.get("outcome"),
            error: r.get("error"),
            reason: r.get("reason"),
            reference: r.get("reference"),
            actor_user_id: r.get("actor_user_id"),
            actor_username: r.get("actor_username"),
            actor_roles: r.get("actor_roles"),
            ip_address: r.get("ip_address"),
            created_at: r.get("created_at"),
            settled_at: r.get("settled_at"),
        })
        .collect::<Vec<_>>();
    let count = entries.len();
    Ok(Json(ApiResponse::success(
        entries,
        format!("{count} catatan break-glass"),
    )))
}
