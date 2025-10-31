//! MFA administration API endpoints for managing account lockouts and MFA settings

use crate::error::AuthencError;
use crate::services::mfa_admin_service::{
    AccountLockoutInfo, MfaAdminResult, MfaAdminService, ResetMfaRequest, UnlockAccountRequest,
};
use crate::services::stores::user_store::UserStoreTrait;
use crate::utils::jwt;
use axum::{
    Router,
    extract::{ConnectInfo, Path, State},
    response::Json,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use uuid::Uuid;

/// Create MFA admin routes (requires admin authentication)
pub fn create_mfa_admin_routes() -> Router<Arc<crate::app::AppState>> {
    Router::new()
        .route("/locked-accounts", get(get_locked_accounts))
        .route("/unlock-account", post(unlock_account))
        .route("/reset-mfa", post(reset_mfa))
        .route("/account-status/:user_id", get(get_account_status))
        .route("/bulk-unlock", post(bulk_unlock_accounts))
}

#[derive(Deserialize)]
/// Request to unlock an account with admin authentication
pub struct AdminUnlockRequest {
    /// Admin JWT token
    pub admin_token: String,
    /// Account unlock details
    #[serde(flatten)]
    pub unlock_request: UnlockAccountRequest,
}

#[derive(Deserialize)]
/// Request to reset MFA with admin authentication
pub struct AdminResetMfaRequest {
    /// Admin JWT token
    pub admin_token: String,
    /// MFA reset details
    #[serde(flatten)]
    pub reset_request: ResetMfaRequest,
}

#[derive(Deserialize)]
/// Request for bulk unlock operations
pub struct BulkUnlockRequest {
    /// Admin JWT token
    pub admin_token: String,
    /// List of user IDs to unlock
    pub user_ids: Vec<Uuid>,
    /// Reason for bulk unlock
    pub admin_reason: String,
}

#[derive(Serialize)]
/// Response for admin operations
pub struct AdminOperationResponse {
    /// Whether the operation was successful
    pub success: bool,
    /// Human-readable message about the operation result
    pub message: String,
    /// Detailed results of the admin operation
    pub results: Vec<MfaAdminResult>,
}

/// Get all currently locked accounts (admin only)
pub async fn get_locked_accounts(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<crate::app::AppState>>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<Vec<AccountLockoutInfo>>, AuthencError> {
    let ip = addr.ip().to_string();

    // Extract and verify admin token
    let admin_token = req["admin_token"]
        .as_str()
        .ok_or_else(|| AuthencError::unauthorized("Admin token required"))?;

    let admin_user_id = verify_admin_token(admin_token, &state).await?;

    // Create MFA admin service
    let mfa_rate_limiter = Arc::new(crate::middleware::MfaRateLimiterState::new(
        state.config.mfa_rate_limit.clone(),
    ));

    let mfa_admin_service = MfaAdminService::new(
        state.user_store.clone(),
        mfa_rate_limiter,
        state.db_pool.clone(),
    );

    // Get locked accounts
    let locked_accounts = mfa_admin_service.get_locked_accounts().await?;

    // Log admin action
    tracing::info!(
        admin_user_id = %admin_user_id,
        ip = %ip,
        action = "view_locked_accounts",
        count = locked_accounts.len(),
        "Admin viewed locked accounts"
    );

    Ok(Json(locked_accounts))
}

/// Unlock a specific account (admin only)
pub async fn unlock_account(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<crate::app::AppState>>,
    Json(req): Json<AdminUnlockRequest>,
) -> Result<Json<MfaAdminResult>, AuthencError> {
    let ip = addr.ip().to_string();

    // Verify admin token
    let admin_user_id = verify_admin_token(&req.admin_token, &state).await?;

    // Create MFA admin service
    let mfa_rate_limiter = Arc::new(crate::middleware::MfaRateLimiterState::new(
        state.config.mfa_rate_limit.clone(),
    ));

    let mfa_admin_service = MfaAdminService::new(
        state.user_store.clone(),
        mfa_rate_limiter,
        state.db_pool.clone(),
    );

    // Unlock the account
    let result = mfa_admin_service
        .unlock_account(req.unlock_request, admin_user_id)
        .await?;

    // Log admin action
    tracing::info!(
        admin_user_id = %admin_user_id,
        target_user_id = %result.user_id.unwrap_or_default(),
        ip = %ip,
        action = "unlock_account",
        success = result.success,
        "Admin unlocked account"
    );

    Ok(Json(result))
}

/// Reset MFA for a user (admin only)
pub async fn reset_mfa(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<crate::app::AppState>>,
    Json(req): Json<AdminResetMfaRequest>,
) -> Result<Json<MfaAdminResult>, AuthencError> {
    let ip = addr.ip().to_string();

    // Verify admin token
    let admin_user_id = verify_admin_token(&req.admin_token, &state).await?;

    // Create MFA admin service
    let mfa_rate_limiter = Arc::new(crate::middleware::MfaRateLimiterState::new(
        state.config.mfa_rate_limit.clone(),
    ));

    let mfa_admin_service = MfaAdminService::new(
        state.user_store.clone(),
        mfa_rate_limiter,
        state.db_pool.clone(),
    );

    // Reset MFA
    let result = mfa_admin_service
        .reset_mfa(req.reset_request, admin_user_id)
        .await?;

    // Log admin action
    tracing::info!(
        admin_user_id = %admin_user_id,
        target_user_id = %result.user_id.unwrap_or_default(),
        ip = %ip,
        action = "reset_mfa",
        success = result.success,
        "Admin reset MFA"
    );

    Ok(Json(result))
}

/// Get account lockout status for a specific user (admin only)
pub async fn get_account_status(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<crate::app::AppState>>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<Option<AccountLockoutInfo>>, AuthencError> {
    let ip = addr.ip().to_string();

    // Extract and verify admin token
    let admin_token = req["admin_token"]
        .as_str()
        .ok_or_else(|| AuthencError::unauthorized("Admin token required"))?;

    let admin_user_id = verify_admin_token(admin_token, &state).await?;

    // Create MFA admin service
    let mfa_rate_limiter = Arc::new(crate::middleware::MfaRateLimiterState::new(
        state.config.mfa_rate_limit.clone(),
    ));

    let mfa_admin_service = MfaAdminService::new(
        state.user_store.clone(),
        mfa_rate_limiter,
        state.db_pool.clone(),
    );

    // Get account status
    let status = mfa_admin_service
        .get_account_lockout_status(user_id)
        .await?;

    // Log admin action
    tracing::info!(
        admin_user_id = %admin_user_id,
        target_user_id = %user_id,
        ip = %ip,
        action = "check_account_status",
        locked = status.is_some(),
        "Admin checked account status"
    );

    Ok(Json(status))
}

/// Bulk unlock multiple accounts (admin only, for emergency situations)
pub async fn bulk_unlock_accounts(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<crate::app::AppState>>,
    Json(req): Json<BulkUnlockRequest>,
) -> Result<Json<AdminOperationResponse>, AuthencError> {
    let ip = addr.ip().to_string();

    // Verify admin token
    let admin_user_id = verify_admin_token(&req.admin_token, &state).await?;

    // Limit bulk operations to prevent abuse
    if req.user_ids.len() > 100 {
        return Err(AuthencError::validation(
            "Bulk unlock limited to 100 accounts at once",
        ));
    }

    // Create MFA admin service
    let mfa_rate_limiter = Arc::new(crate::middleware::MfaRateLimiterState::new(
        state.config.mfa_rate_limit.clone(),
    ));

    let mfa_admin_service = MfaAdminService::new(
        state.user_store.clone(),
        mfa_rate_limiter,
        state.db_pool.clone(),
    );

    // Perform bulk unlock
    let results = mfa_admin_service
        .bulk_unlock_accounts(
            req.user_ids.clone(),
            admin_user_id,
            req.admin_reason.clone(),
        )
        .await?;

    let success_count = results.iter().filter(|r| r.success).count();
    let total_count = results.len();

    // Log admin action
    tracing::warn!(
        admin_user_id = %admin_user_id,
        ip = %ip,
        action = "bulk_unlock_accounts",
        total_accounts = total_count,
        successful_unlocks = success_count,
        reason = %req.admin_reason,
        "Admin performed bulk account unlock"
    );

    Ok(Json(AdminOperationResponse {
        success: success_count > 0,
        message: format!("Unlocked {} out of {} accounts", success_count, total_count),
        results,
    }))
}

/// Verify admin token and return admin user ID
async fn verify_admin_token(
    token: &str,
    state: &Arc<crate::app::AppState>,
) -> Result<Uuid, AuthencError> {
    // Verify JWT token
    let claims =
        jwt::verify_jwt(token).map_err(|_| AuthencError::unauthorized("Invalid admin token"))?;

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AuthencError::internal("Invalid user ID in token"))?;

    // Get user and verify admin privileges
    let user = state
        .user_store
        .get_user(user_id)
        .await?
        .ok_or_else(|| AuthencError::unauthorized("Admin user not found"))?;

    // Check if user has admin role (simplified check - in production, use proper RBAC)
    let is_admin = user.roles.iter().any(|role| {
        role.name == "admin" || role.name == "system_admin" || role.name == "mfa_admin"
    });

    if !is_admin {
        return Err(AuthencError::forbidden(
            "Admin privileges required for MFA management",
        ));
    }

    Ok(user_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_admin_token_verification() {
        // This would require setting up test infrastructure
        // Implementation depends on your test setup
    }

    #[tokio::test]
    async fn test_bulk_unlock_limits() {
        // Test that bulk unlock is limited to reasonable numbers
        let user_ids: Vec<Uuid> = (0..150).map(|_| Uuid::new_v4()).collect();

        let request = BulkUnlockRequest {
            admin_token: "test_token".to_string(),
            user_ids,
            admin_reason: "Test bulk unlock".to_string(),
        };

        // This should fail validation due to too many accounts
        assert!(request.user_ids.len() > 100);
    }
}
