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
use lib_perlengkapan::response::PaginatedResponse;

use super::models::{AuditTrailEntry, AuditTrailQuery};
use super::repository;

/// GET /audit — paginated cross-module audit trail.
///
/// RBAC: cross-satker (admin/pusat) only. BPK reviewers use the `entity` +
/// `resource_id` filters to pull the complete history of a single record.
pub async fn list_audit_trail(
    State(pool): State<Pool>,
    claims: Claims,
    Query(query): Query<AuditTrailQuery>,
) -> AppResult<Json<PaginatedResponse<AuditTrailEntry>>> {
    if !claims.is_cross_satker_role() {
        return Err(AppError::Authorization(
            "Audit log lintas-modul hanya tersedia untuk role pusat/admin".to_string(),
        ));
    }

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
