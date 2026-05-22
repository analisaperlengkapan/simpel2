//! # Admin HTTP Handlers
//!
//! Thin axum wrappers over `repository` that enforce cross-satker/admin
//! authorization and translate to the standard envelope types.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use deadpool_postgres::Pool;

use crate::shared::error::{AppError, AppResult};
use crate::shared::middleware::Claims;
use crate::models::{ApiResponse, PaginatedResponse};

use super::models::{
    AuditFilter, AuditLogEntry, MasterListQuery, MasterRecord, MasterSource, MasterUpsertRequest,
};
use super::repository;

fn require_admin(claims: &Claims) -> AppResult<()> {
    if claims.is_cross_satker_role() {
        Ok(())
    } else {
        Err(AppError::Authorization(
            "Endpoint admin hanya tersedia untuk role pusat/admin".to_string(),
        ))
    }
}

/// GET /admin/audit
pub async fn list_audit_logs(
    State(pool): State<Pool>,
    claims: Claims,
    Query(filter): Query<AuditFilter>,
) -> AppResult<Json<PaginatedResponse<AuditLogEntry>>> {
    require_admin(&claims)?;
    let page = filter.page.unwrap_or(1).max(1);
    let per_page = filter.per_page.unwrap_or(25).clamp(1, 200);

    let (entries, total) = repository::list_audit_logs(&pool, &filter).await?;

    Ok(Json(PaginatedResponse::new(
        entries,
        total,
        page,
        per_page,
        "Audit log retrieved successfully".to_string(),
    )))
}

/// GET /admin/master
pub async fn list_master_sources(
    State(pool): State<Pool>,
    claims: Claims,
) -> AppResult<Json<ApiResponse<Vec<MasterSource>>>> {
    require_admin(&claims)?;
    let sources = repository::list_master_sources(&pool).await?;
    Ok(Json(ApiResponse::success(
        sources,
        "Master data catalog retrieved successfully".to_string(),
    )))
}

/// GET /admin/master/:source
pub async fn list_master_records(
    State(pool): State<Pool>,
    claims: Claims,
    Path(source): Path<String>,
    Query(query): Query<MasterListQuery>,
) -> AppResult<Json<PaginatedResponse<MasterRecord>>> {
    require_admin(&claims)?;
    let table = repository::find_master_table(&source)
        .ok_or_else(|| AppError::NotFound(format!("Master source '{}' tidak dikenal", source)))?;

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(25).clamp(1, 200);

    let (records, total) =
        repository::list_master_records(&pool, table, page, per_page, query.q.as_deref()).await?;

    Ok(Json(PaginatedResponse::new(
        records,
        total,
        page,
        per_page,
        format!("{} records retrieved", table.label),
    )))
}

/// POST /admin/master/:source
pub async fn create_master_record(
    State(pool): State<Pool>,
    claims: Claims,
    Path(source): Path<String>,
    Json(req): Json<MasterUpsertRequest>,
) -> AppResult<(StatusCode, Json<ApiResponse<MasterRecord>>)> {
    require_admin(&claims)?;
    let table = repository::find_master_table(&source)
        .ok_or_else(|| AppError::NotFound(format!("Master source '{}' tidak dikenal", source)))?;

    let record = repository::create_master_record(&pool, table, &req).await?;
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            record,
            format!("{} berhasil ditambahkan", table.label),
        )),
    ))
}

/// PUT /admin/master/:source/:id
pub async fn update_master_record(
    State(pool): State<Pool>,
    claims: Claims,
    Path((source, id)): Path<(String, String)>,
    Json(req): Json<MasterUpsertRequest>,
) -> AppResult<Json<ApiResponse<MasterRecord>>> {
    require_admin(&claims)?;
    let table = repository::find_master_table(&source)
        .ok_or_else(|| AppError::NotFound(format!("Master source '{}' tidak dikenal", source)))?;

    let record = repository::update_master_record(&pool, table, &id, &req).await?;
    Ok(Json(ApiResponse::success(
        record,
        format!("{} berhasil diperbarui", table.label),
    )))
}

/// DELETE /admin/master/:source/:id
pub async fn delete_master_record(
    State(pool): State<Pool>,
    claims: Claims,
    Path((source, id)): Path<(String, String)>,
) -> AppResult<impl IntoResponse> {
    require_admin(&claims)?;
    let table = repository::find_master_table(&source)
        .ok_or_else(|| AppError::NotFound(format!("Master source '{}' tidak dikenal", source)))?;

    repository::delete_master_record(&pool, table, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}
