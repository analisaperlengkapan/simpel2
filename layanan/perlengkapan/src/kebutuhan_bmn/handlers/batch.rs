use super::params::*;
use axum::{Json, extract::State};
use tracing::info;

use crate::shared::error::AppError;
use crate::shared::middleware::{Claims, ClientIp};
use lib_perlengkapan::response::ApiResponse;

use crate::kebutuhan_bmn::services::KebutuhanBmnService;

// ============================================================================
// Batch Operations Handlers
// ============================================================================

/// POST /kebutuhan-bmn/batch/approve
/// Batch approve multiple kebutuhan
pub async fn batch_approve_kebutuhan(
    State(service): State<KebutuhanBmnService>,
    claims: Claims,
    ClientIp(ip): ClientIp,
    Json(request): Json<crate::kebutuhan_bmn::models::BatchApproveRequest>,
) -> Result<Json<ApiResponse<crate::kebutuhan_bmn::models::BatchOperationResponse>>, AppError> {
    use validator::Validate;

    // Validate request
    request
        .validate()
        .map_err(|e| AppError::BadRequest(format!("Validation error: {}", e)))?;

    info!(
        "Batch approve requested for {} items by user {}",
        request.kebutuhan_ids.len(),
        claims.user_id
    );

    let user_id = Some(claims.user_id);
    let user_info = Some(extract_user_info(&claims));

    let response = service
        .batch_approve_kebutuhan(request, user_id, user_info, claims.role.clone(), ip)
        .await?;

    Ok(Json(ApiResponse::success(
        response,
        "Batch approval completed".to_string(),
    )))
}
/// POST /kebutuhan-bmn/batch/reject
/// Batch reject multiple kebutuhan
pub async fn batch_reject_kebutuhan(
    State(service): State<KebutuhanBmnService>,
    claims: Claims,
    ClientIp(ip): ClientIp,
    Json(request): Json<crate::kebutuhan_bmn::models::BatchRejectRequest>,
) -> Result<Json<ApiResponse<crate::kebutuhan_bmn::models::BatchOperationResponse>>, AppError> {
    use validator::Validate;

    // Validate request
    request
        .validate()
        .map_err(|e| AppError::BadRequest(format!("Validation error: {}", e)))?;

    info!(
        "Batch reject requested for {} items by user {}",
        request.kebutuhan_ids.len(),
        claims.user_id
    );

    let user_id = Some(claims.user_id);
    let user_info = Some(extract_user_info(&claims));

    let response = service
        .batch_reject_kebutuhan(request, user_id, user_info, claims.role.clone(), ip)
        .await?;

    Ok(Json(ApiResponse::success(
        response,
        "Batch rejection completed".to_string(),
    )))
}
/// POST /kebutuhan-bmn/batch/update-status
/// Batch update status for multiple kebutuhan
pub async fn batch_update_status(
    State(service): State<KebutuhanBmnService>,
    claims: Claims,
    ClientIp(ip): ClientIp,
    Json(request): Json<crate::kebutuhan_bmn::models::BatchUpdateStatusRequest>,
) -> Result<Json<ApiResponse<crate::kebutuhan_bmn::models::BatchOperationResponse>>, AppError> {
    use validator::Validate;

    // Validate request
    request
        .validate()
        .map_err(|e| AppError::BadRequest(format!("Validation error: {}", e)))?;

    info!(
        "Batch update status requested for {} items to status {} by user {}",
        request.kebutuhan_ids.len(),
        request.target_status,
        claims.user_id
    );

    let user_id = Some(claims.user_id);
    let user_info = Some(extract_user_info(&claims));

    let response = service
        .batch_update_status(request, user_id, user_info, claims.role.clone(), ip)
        .await?;

    Ok(Json(ApiResponse::success(
        response,
        "Batch status update completed".to_string(),
    )))
}
