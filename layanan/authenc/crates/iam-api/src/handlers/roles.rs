//! Role management HTTP handlers
//!
//! The platform's role model is fixed and seeded (admin + operator_satker +
//! validator_wilayah + validator_pusat, `002_seed.sql`); login builds
//! `realm_access.roles` from the same `user_roles` → `roles` relation these
//! handlers manage. The surface is therefore read + assignment only — role
//! CRUD would drift the seeded model and had no real implementation or
//! consumer anyway.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::ApiResult;
use crate::state::IamApiState;
use authenc_types::AuthencError;

/// Role response DTO (shape shared with the portal `RoleInfo`)
#[derive(Debug, Serialize)]
pub struct RoleResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    /// `permission@resource` pairs from `role_permissions`
    pub permissions: Vec<String>,
    /// Number of users currently holding this role
    pub user_count: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// GET /api/v1/iam/roles - List roles with permissions and user counts
pub async fn list_roles(
    State(state): State<Arc<IamApiState>>,
) -> ApiResult<Json<Vec<RoleResponse>>> {
    let rows = state
        .database
        .query(
            r#"
            SELECT r.id,
                   r.name,
                   r.description,
                   r.created_at,
                   COALESCE(p.permissions, '{}') AS permissions,
                   COALESCE(u.user_count, 0)    AS user_count
            FROM roles r
            LEFT JOIN (
                SELECT role_id,
                       array_agg(permission || '@' || COALESCE(resource, '*')
                                 ORDER BY resource) AS permissions
                FROM role_permissions
                GROUP BY role_id
            ) p ON p.role_id = r.id
            LEFT JOIN (
                SELECT role_id, COUNT(*) AS user_count
                FROM user_roles
                GROUP BY role_id
            ) u ON u.role_id = r.id
            WHERE r.deleted_at IS NULL
            ORDER BY r.name
            "#,
            &[],
        )
        .await
        .map_err(crate::error::ApiError)?;

    let roles = rows
        .into_iter()
        .map(|row| RoleResponse {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            permissions: row.get("permissions"),
            user_count: row.get::<_, i64>("user_count") as u64,
            created_at: row.get("created_at"),
        })
        .collect();

    Ok(Json(roles))
}

/// POST /api/v1/iam/users/{user_id}/roles/{role_id} - Assign role to user
pub async fn assign_role_to_user(
    State(state): State<Arc<IamApiState>>,
    Path((user_id, role_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<StatusCode> {
    // Role and user must both exist — a nonexistent pair must 404, not
    // silently insert a dangling row (user_roles has no FK to roles).
    let role_exists = state
        .database
        .query(
            "SELECT 1 FROM roles WHERE id = $1 AND deleted_at IS NULL",
            &[&role_id],
        )
        .await
        .map_err(crate::error::ApiError)?;
    if role_exists.is_empty() {
        return Err(crate::error::ApiError(AuthencError::not_found("role")));
    }
    let user_exists = state
        .database
        .query("SELECT 1 FROM users WHERE id = $1", &[&user_id])
        .await
        .map_err(crate::error::ApiError)?;
    if user_exists.is_empty() {
        return Err(crate::error::ApiError(AuthencError::not_found("user")));
    }

    state
        .database
        .execute(
            r#"
            INSERT INTO user_roles (user_id, role_id)
            VALUES ($1, $2)
            ON CONFLICT (user_id, role_id) DO NOTHING
            "#,
            &[&user_id, &role_id],
        )
        .await
        .map_err(crate::error::ApiError)?;

    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /api/v1/iam/users/{user_id}/roles/{role_id} - Remove role from user
pub async fn remove_role_from_user(
    State(state): State<Arc<IamApiState>>,
    Path((user_id, role_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<StatusCode> {
    let affected = state
        .database
        .execute(
            "DELETE FROM user_roles WHERE user_id = $1 AND role_id = $2",
            &[&user_id, &role_id],
        )
        .await
        .map_err(crate::error::ApiError)?;

    if affected == 0 {
        return Err(crate::error::ApiError(AuthencError::not_found(
            "role assignment",
        )));
    }

    Ok(StatusCode::NO_CONTENT)
}
