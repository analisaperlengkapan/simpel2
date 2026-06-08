use crate::bantuan::error::AppError;
use crate::bantuan::export_import::ExportImportService;
use crate::bantuan::gdpr::GdprService;
use axum::{
    extract::{Json, Path, Query, State},
    response::IntoResponse,
};
use deadpool_postgres::Pool;
use serde_json::json;
use uuid::Uuid;

pub async fn export_resource(
    State(pool): State<Pool>,
    Path(resource): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let export_import = ExportImportService::new(pool);
    let data = export_import.export_resource(&resource).await?;
    Ok(Json(data))
}
pub async fn import_resource(
    State(pool): State<Pool>,
    Path(resource): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let export_import = ExportImportService::new(pool.clone());
    export_import.import_resource(&resource, payload).await?;
    // TODO(audit-log): emit AuditEvent via state.audit_sink once the bantuan router is mounted into the unified app + claims are plumbed through (deferred per plan A.5).
    Ok(Json(json!({"status": "imported", "resource": resource})))
}
pub async fn gdpr_request_delete(
    State(pool): State<Pool>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = payload["user_id"]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(AppError::Validation("user_id wajib".to_string()))?;
    let details = payload.get("details").cloned().unwrap_or(json!({}));
    let gdpr = GdprService::new(pool.clone());
    let req = gdpr.request_delete(user_id, details).await?;
    // TODO(audit-log): emit AuditEvent via state.audit_sink once the bantuan router is mounted into the unified app + claims are plumbed through (deferred per plan A.5).
    Ok(Json(req))
}
pub async fn gdpr_request_download(
    State(pool): State<Pool>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = payload["user_id"]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(AppError::Validation("user_id wajib".to_string()))?;
    let details = payload.get("details").cloned().unwrap_or(json!({}));
    let gdpr = GdprService::new(pool.clone());
    let req = gdpr.request_download(user_id, details).await?;
    // TODO(audit-log): emit AuditEvent via state.audit_sink once the bantuan router is mounted into the unified app + claims are plumbed through (deferred per plan A.5).
    Ok(Json(req))
}
pub async fn gdpr_status(
    State(pool): State<Pool>,
    Query(params): Query<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = params["user_id"]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(AppError::Validation("user_id wajib".to_string()))?;
    let gdpr = GdprService::new(pool);
    let reqs = gdpr.get_status(user_id).await?;
    Ok(Json(reqs))
}
pub async fn gdpr_process_request(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let status = payload["status"]
        .as_str()
        .ok_or(AppError::Validation("status wajib".to_string()))?;
    let details = payload.get("details").cloned().unwrap_or(json!({}));
    let gdpr = GdprService::new(pool.clone());
    let req = gdpr.process_request(id, status, details).await?;
    // TODO(audit-log): emit AuditEvent via state.audit_sink once the bantuan router is mounted into the unified app + claims are plumbed through (deferred per plan A.5).
    Ok(Json(req))
}
pub async fn health() -> impl IntoResponse {
    Json(json!({"status": "ok"}))
}
