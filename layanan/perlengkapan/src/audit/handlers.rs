//! # Cross-module audit trail — HTTP handler
//!
//! `GET /audit` exposes the canonical `perlengkapan.audit_log` to auditors.
//! Restricted to cross-satker roles (admin/pusat) since the trail spans every
//! satker and carries PII (actor identity, IP). For per-entity drill-down
//! pass `?entity=<resource_type>&resource_id=<id>`.

use axum::{
    Json,
    extract::{Query, State},
};
use deadpool_postgres::Pool;

use crate::shared::error::{AppError, AppResult};
use crate::shared::middleware::Claims;
use lib_perlengkapan::response::{ApiResponse, PaginatedResponse};

use super::models::{AuditFacets, AuditTrailEntry, AuditTrailQuery};
use super::repository;

/// Who may read the cross-module trail: administrators (`ViewAudit`) and the
/// pusat-level oversight roles — the same readership as before, now named instead
/// of borrowed from "may read across satkers", which is what the previous check
/// actually tested.
fn require_audit_reader(claims: &Claims) -> AppResult<()> {
    claims
        .require_any_role_or_admin(&["validator_pusat", "analis_pusat", "pusat"])
        .map_err(|_| {
            AppError::Authorization(
                "Audit log lintas-modul hanya tersedia untuk role pusat/admin".to_string(),
            )
        })
}

/// GET /audit — paginated cross-module audit trail.
///
/// RBAC: cross-satker (admin/pusat) only. BPK reviewers use the `entity` +
/// `resource_id` filters to pull the complete history of a single record.
pub async fn list_audit_trail(
    State(pool): State<Pool>,
    claims: Claims,
    Query(query): Query<AuditTrailQuery>,
) -> AppResult<Json<PaginatedResponse<AuditTrailEntry>>> {
    require_audit_reader(&claims)?;

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(25).clamp(1, 200);

    let (entries, total) = repository::list_audit_trail(&pool, &query).await?;

    Ok(Json(PaginatedResponse::new(
        entries,
        total,
        page,
        per_page,
        "Audit trail retrieved successfully".to_string(),
    )))
}

/// GET /audit/facets — the module / entity / action values that occur in the
/// trail, for the viewer's filter dropdowns. Same readership as the trail itself.
pub async fn list_audit_facets(
    State(pool): State<Pool>,
    claims: Claims,
) -> AppResult<Json<ApiResponse<AuditFacets>>> {
    require_audit_reader(&claims)?;
    let facets = repository::list_audit_facets(&pool).await?;
    Ok(Json(ApiResponse::success(
        facets,
        "Audit facets retrieved successfully".to_string(),
    )))
}
