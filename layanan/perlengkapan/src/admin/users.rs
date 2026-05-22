//! Admin endpoints for `/admin/users` — user catalog + role assignment.
//!
//! Reads the `v_user_role_summary` view (defined in migration V018) which
//! joins `perlengkapan_users` with `perlengkapan_user_roles` and projects
//! every user with their list of assigned roles + active role. Writes go
//! through `perlengkapan_user_roles` directly via the assignment
//! endpoints below.
//!
//! Auth: cross-satker/admin only — mirrors the rest of `/admin/*`.

use axum::{
    Json,
    extract::{Path, Query, State},
};
use lib_perlengkapan::audit::{AuditAction, AuditEvent};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::ApiResponse;
use crate::shared::error::{AppError, AppResult};
use crate::shared::middleware::{Claims, ClientIp};
use crate::state::AppState;

fn require_admin(claims: &Claims) -> AppResult<()> {
    if claims.is_cross_satker_role() {
        Ok(())
    } else {
        Err(AppError::Authorization(
            "Endpoint admin/users hanya tersedia untuk role pusat/admin".to_string(),
        ))
    }
}

/// Row shape returned by `v_user_role_summary`. Mirrors the
/// `UserRoleAssignment` DTO the frontend already expects so the swap
/// from the demo data to the real endpoint is a single fn-body change
/// on the frontend side.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRoleAssignment {
    pub nip: String,
    pub nama: String,
    pub jabatan: Option<String>,
    pub golongan: Option<String>,
    pub satker_code: Option<String>,
    pub satker_name: Option<String>,
    pub assigned_roles: Vec<String>,
    pub active_role: Option<String>,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct ListUsersQuery {
    /// Substring match against `nip` / `nama` / `jabatan` (case-insensitive).
    #[serde(default)]
    pub search: Option<String>,
    /// Filter to users that hold this specific role (matches a row in
    /// `perlengkapan_user_roles.role_name`).
    #[serde(default)]
    pub role: Option<String>,
    /// Filter to a single satker (kode).
    #[serde(default)]
    pub satker_code: Option<String>,
}

/// GET /admin/users
pub async fn list_users(
    State(state): State<AppState>,
    claims: Claims,
    Query(q): Query<ListUsersQuery>,
) -> AppResult<Json<ApiResponse<Vec<UserRoleAssignment>>>> {
    require_admin(&claims)?;

    let client = state
        .db_pool
        .get()
        .await
        .map_err(|e| AppError::Internal(format!("admin/users pool: {e}")))?;

    // Build the query incrementally. We always select from the view
    // (which already aggregates roles into an array) and append WHERE
    // clauses based on which filters the caller supplied.
    let mut sql = String::from(
        "SELECT nip, nama, jabatan, golongan, satker_code, satker_name,
                status, assigned_roles, active_role
         FROM v_user_role_summary
         WHERE 1 = 1",
    );
    let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();
    let search_pattern: Option<String> = q
        .search
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .map(|s| format!("%{}%", s.trim().to_lowercase()));
    if let Some(p) = &search_pattern {
        sql.push_str(&format!(
            " AND (LOWER(nip) LIKE ${idx} OR LOWER(nama) LIKE ${idx} OR LOWER(COALESCE(jabatan, '')) LIKE ${idx})",
            idx = params.len() + 1
        ));
        params.push(p);
    }
    let role_filter = q.role.as_ref().filter(|s| !s.trim().is_empty());
    if let Some(r) = role_filter {
        sql.push_str(&format!(" AND ${} = ANY(assigned_roles)", params.len() + 1));
        params.push(r);
    }
    let satker_filter = q.satker_code.as_ref().filter(|s| !s.trim().is_empty());
    if let Some(s) = satker_filter {
        sql.push_str(&format!(" AND satker_code = ${}", params.len() + 1));
        params.push(s);
    }
    sql.push_str(" ORDER BY nama LIMIT 500");

    let rows = client
        .query(&sql, &params)
        .await
        .map_err(|e| AppError::Database(format!("admin/users query: {e}")))?;

    let items = rows
        .into_iter()
        .map(|r| UserRoleAssignment {
            nip: r.get("nip"),
            nama: r.get("nama"),
            jabatan: r.try_get("jabatan").ok().flatten(),
            golongan: r.try_get("golongan").ok().flatten(),
            satker_code: r.try_get("satker_code").ok().flatten(),
            satker_name: r.try_get("satker_name").ok().flatten(),
            assigned_roles: r
                .try_get::<_, Vec<String>>("assigned_roles")
                .unwrap_or_default(),
            active_role: r.try_get("active_role").ok().flatten(),
            status: r.get("status"),
        })
        .collect();

    Ok(Json(ApiResponse::success(
        items,
        "Users retrieved".to_string(),
    )))
}

/// GET /admin/users/{nip}
pub async fn get_user(
    State(state): State<AppState>,
    Path(nip): Path<String>,
    claims: Claims,
) -> AppResult<Json<ApiResponse<UserRoleAssignment>>> {
    require_admin(&claims)?;

    let client = state
        .db_pool
        .get()
        .await
        .map_err(|e| AppError::Internal(format!("admin/users pool: {e}")))?;

    let row = client
        .query_opt(
            "SELECT nip, nama, jabatan, golongan, satker_code, satker_name,
                    status, assigned_roles, active_role
             FROM v_user_role_summary
             WHERE nip = $1",
            &[&nip],
        )
        .await
        .map_err(|e| AppError::Database(format!("admin/users query: {e}")))?
        .ok_or_else(|| AppError::NotFound(format!("User NIP={} tidak ditemukan", nip)))?;

    Ok(Json(ApiResponse::success(
        UserRoleAssignment {
            nip: row.get("nip"),
            nama: row.get("nama"),
            jabatan: row.try_get("jabatan").ok().flatten(),
            golongan: row.try_get("golongan").ok().flatten(),
            satker_code: row.try_get("satker_code").ok().flatten(),
            satker_name: row.try_get("satker_name").ok().flatten(),
            assigned_roles: row
                .try_get::<_, Vec<String>>("assigned_roles")
                .unwrap_or_default(),
            active_role: row.try_get("active_role").ok().flatten(),
            status: row.get("status"),
        },
        "User retrieved".to_string(),
    )))
}

#[derive(Debug, Deserialize)]
pub struct AssignRoleRequest {
    /// One of: operator_satker, validator_wilayah, validator_pusat, admin.
    pub role: String,
}

/// POST /admin/users/{nip}/roles
///
/// Assigns a role to the user identified by NIP. Idempotent — re-posting
/// the same role updates `last_synced_at` but the unique
/// `(user_id, role_name)` constraint prevents duplicates.
pub async fn assign_role(
    State(state): State<AppState>,
    Path(nip): Path<String>,
    claims: Claims,
    ClientIp(ip): ClientIp,
    Json(body): Json<AssignRoleRequest>,
) -> AppResult<Json<ApiResponse<UserRoleAssignment>>> {
    require_admin(&claims)?;

    const ALLOWED: &[&str] = &[
        "operator_satker",
        "validator_wilayah",
        "validator_pusat",
        "admin",
    ];
    if !ALLOWED.contains(&body.role.as_str()) {
        return Err(AppError::BadRequest(format!(
            "role tidak valid; pakai salah satu dari {ALLOWED:?}"
        )));
    }

    let client = state
        .db_pool
        .get()
        .await
        .map_err(|e| AppError::Internal(format!("admin/users pool: {e}")))?;

    // Look up the canonical `user_id` + `username` for this NIP. We have
    // `perlengkapan_users.nip` but no `user_id` column there (it's the
    // authenc UUID); fall back to deterministic UUIDv5(nip) so the row is
    // stable even if the user hasn't logged in yet.
    let user_row = client
        .query_opt(
            "SELECT id, nip, nama FROM perlengkapan_users WHERE nip = $1",
            &[&nip],
        )
        .await
        .map_err(|e| AppError::Database(format!("user lookup: {e}")))?;
    let (user_id, username) = match user_row {
        Some(r) => {
            let id: Uuid = r.get("id");
            let nama: String = r.get("nama");
            (id, nama)
        }
        None => {
            return Err(AppError::NotFound(format!(
                "User NIP={nip} tidak ditemukan"
            )));
        }
    };

    let role_id = Uuid::new_v4();
    client
        .execute(
            r#"
            INSERT INTO perlengkapan_user_roles
                (id, user_id, username, nip, role_name, last_synced_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
            ON CONFLICT (user_id, role_name) DO UPDATE
                SET last_synced_at = NOW()
            "#,
            &[&role_id, &user_id, &username, &nip, &body.role],
        )
        .await
        .map_err(|e| AppError::Database(format!("role assignment: {e}")))?;

    // Audit: record the assignment via the cross-module sink. Best-effort.
    let event = AuditEvent::new("admin", AuditAction::Custom, "user_role")
        .actor(claims.user_id, claims.username.clone())
        .ip(ip)
        .resource_id(nip.clone())
        .message(format!("assign role={} to nip={}", body.role, nip));
    let mut event = event;
    event.action_name = Some("user_role.assign".to_string());
    let _ = state.audit_sink.log(event).await;

    // Return the refreshed view row.
    get_user(State(state), Path(nip), claims).await
}

/// DELETE /admin/users/{nip}/roles/{role}
pub async fn unassign_role(
    State(state): State<AppState>,
    Path((nip, role)): Path<(String, String)>,
    claims: Claims,
    ClientIp(ip): ClientIp,
) -> AppResult<Json<ApiResponse<UserRoleAssignment>>> {
    require_admin(&claims)?;

    let client = state
        .db_pool
        .get()
        .await
        .map_err(|e| AppError::Internal(format!("admin/users pool: {e}")))?;

    let deleted = client
        .execute(
            "DELETE FROM perlengkapan_user_roles WHERE nip = $1 AND role_name = $2",
            &[&nip, &role],
        )
        .await
        .map_err(|e| AppError::Database(format!("role unassign: {e}")))?;
    if deleted == 0 {
        return Err(AppError::NotFound(format!(
            "Role {role} tidak ditemukan pada user NIP={nip}"
        )));
    }

    let mut event = AuditEvent::new("admin", AuditAction::Custom, "user_role")
        .actor(claims.user_id, claims.username.clone())
        .ip(ip)
        .resource_id(nip.clone())
        .message(format!("unassign role={} from nip={}", role, nip));
    event.action_name = Some("user_role.unassign".to_string());
    let _ = state.audit_sink.log(event).await;

    get_user(State(state), Path(nip), claims).await
}
