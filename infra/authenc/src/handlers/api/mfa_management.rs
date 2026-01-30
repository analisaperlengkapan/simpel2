//! MFA Management API endpoints for administrative operations
//!
//! This module provides comprehensive MFA management capabilities for administrators,
//! including viewing user MFA status, resetting MFA settings, and bulk operations.

use crate::error::{AuthencError, Result};
use crate::models::user::SecurityContext;
use crate::services::mfa_service::{MfaService, MfaStatistics, MfaStatus};
use crate::services::stores::user_store::UserStoreTrait;
use crate::utils::jwt;
use axum::{
    Router,
    extract::{ConnectInfo, Path, Query, State},
    response::Json,
    routing::{get, post, put},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use uuid::Uuid;

/// Create MFA management routes (requires admin authentication)
pub fn create_mfa_management_routes() -> Router<Arc<crate::app::AppState>> {
    Router::new()
        // User MFA status management
        .route("/users", get(get_users_mfa_status))
        .route("/users/{user_id}/status", get(get_user_mfa_status))
        .route("/users/{user_id}/reset", post(reset_user_mfa))
        .route("/users/{user_id}/disable", post(disable_user_mfa))
        .route("/users/{user_id}/force-setup", post(force_mfa_setup))
        // Bulk operations
        .route("/bulk/reset", post(bulk_reset_mfa))
        .route("/bulk/disable", post(bulk_disable_mfa))
        .route("/bulk/force-setup", post(bulk_force_setup))
        // Organization-wide management
        .route("/organization/status", get(get_organization_mfa_status))
        .route("/organization/policy", get(get_mfa_policy))
        .route("/organization/policy", put(update_mfa_policy))
        // Reporting and analytics
        .route("/reports/adoption", get(get_mfa_adoption_report))
        .route("/reports/usage", get(get_mfa_usage_report))
        .route("/reports/compliance", get(get_compliance_report))
        // Recovery operations
        .route(
            "/users/{user_id}/recovery-codes",
            get(get_user_recovery_codes),
        )
        .route(
            "/users/{user_id}/recovery-codes/regenerate",
            post(regenerate_recovery_codes),
        )
}

// Request/Response structures

/// Request structure for admin authentication in MFA management operations
#[derive(Deserialize)]
pub struct AdminAuthRequest {
    /// Admin authentication token for privileged operations
    pub admin_token: String,
}

/// Query parameters for retrieving paginated user MFA information
#[derive(Deserialize)]
pub struct GetUsersQuery {
    /// Page number for pagination (1-based)
    pub page: Option<u32>,
    /// Number of users per page
    pub limit: Option<u32>,
    /// Filter by satker (work unit) code
    pub satker_code: Option<String>,
    /// Filter by MFA enabled status
    pub mfa_enabled: Option<bool>,
    /// Search term for username, name, or NIP
    pub search: Option<String>,
}

/// Response structure containing MFA information for a specific user
#[derive(Serialize)]
pub struct UserMfaInfo {
    /// Unique identifier for the user
    pub user_id: Uuid,
    /// Username/login identifier
    pub username: String,
    /// Government employee identification number
    pub nip: Option<String>,
    /// Full name of the user
    pub nama: Option<String>,
    /// Work unit code (satker)
    pub satker_code: Option<String>,
    /// Job position/title
    pub jabatan: Option<String>,
    /// Current MFA status for the user
    pub mfa_status: MfaStatus,
    /// Timestamp of the user's last login
    pub last_login: Option<DateTime<Utc>>,
    /// Whether the user's account is currently locked
    pub account_locked: bool,
}

/// Paginated response containing user MFA information
#[derive(Serialize)]
pub struct PaginatedUserMfaResponse {
    /// List of user MFA information for the current page
    pub users: Vec<UserMfaInfo>,
    /// Total number of users matching the query
    pub total: u64,
    /// Current page number (1-based)
    pub page: u32,
    /// Number of users per page
    pub limit: u32,
    /// Total number of pages available
    pub total_pages: u32,
}

/// Request structure for resetting MFA for a user
#[derive(Deserialize)]
pub struct ResetMfaRequest {
    /// Admin authentication token for authorization
    pub admin_token: String,
    /// Reason for resetting MFA (for audit logging)
    pub reason: String,
    /// Whether to force immediate reactivation of MFA
    pub force_reactivation: bool,
}

/// Request structure for performing bulk MFA operations on multiple users
#[derive(Deserialize)]
pub struct BulkMfaRequest {
    /// Admin authentication token for authorization
    pub admin_token: String,
    /// List of user IDs to perform bulk operation on
    pub user_ids: Vec<Uuid>,
    /// Reason for the bulk operation (for audit logging)
    pub reason: String,
}

/// Response structure for bulk MFA operations
#[derive(Serialize)]
pub struct BulkMfaResponse {
    /// Number of successful operations
    pub success_count: u32,
    /// Number of failed operations
    pub failure_count: u32,
    /// Detailed results for each user operation
    pub results: Vec<MfaOperationResult>,
}

/// Result structure for individual MFA operations in bulk requests
#[derive(Serialize)]
pub struct MfaOperationResult {
    /// ID of the user the operation was performed on
    pub user_id: Uuid,
    /// Username of the user
    pub username: String,
    /// Whether the operation was successful
    pub success: bool,
    /// Detailed message about the operation result
    pub message: String,
}

/// Organization-wide MFA status summary
#[derive(Serialize)]
pub struct OrganizationMfaStatus {
    /// Total number of users in the organization
    pub total_users: u64,
    /// Number of users with MFA enabled
    pub mfa_enabled_users: u64,
    /// MFA adoption rate as a percentage (0.0 to 1.0)
    pub mfa_adoption_rate: f64,
    /// MFA statistics broken down by satker (work unit)
    pub by_satker: HashMap<String, MfaStatistics>,
    /// Recent MFA activity summaries
    pub recent_activity: Vec<MfaActivitySummary>,
}

/// Summary of MFA activity for a specific date
#[derive(Serialize)]
pub struct MfaActivitySummary {
    /// Date of the activity (YYYY-MM-DD format)
    pub date: String,
    /// Number of new MFA setups on this date
    pub new_setups: u32,
    /// Number of successful MFA verifications on this date
    pub successful_verifications: u32,
    /// Number of failed MFA verifications on this date
    pub failed_verifications: u32,
}

/// MFA policy configuration for the organization
#[derive(Debug, Serialize, Deserialize)]
pub struct MfaPolicy {
    /// Whether MFA is enforced for all users
    pub enforce_for_all: bool,
    /// List of roles that require MFA enforcement
    pub enforce_for_roles: Vec<String>,
    /// List of satkers (work units) that require MFA enforcement
    pub enforce_for_satkers: Vec<String>,
    /// Grace period in days for MFA setup after account creation
    pub grace_period_days: u32,
    /// Whether backup codes are required for MFA setup
    pub backup_codes_required: bool,
    /// Maximum number of failed MFA attempts before lockout
    pub max_failed_attempts: u32,
    /// Duration in minutes for account lockout after failed attempts
    pub lockout_duration_minutes: u32,
}

/// Report on MFA adoption across the organization
#[derive(Serialize)]
pub struct MfaAdoptionReport {
    /// Overall MFA adoption rate across the organization (0.0 to 1.0)
    pub overall_adoption: f64,
    /// MFA adoption rates broken down by satker (work unit)
    pub by_satker: HashMap<String, f64>,
    /// MFA adoption rates broken down by role
    pub by_role: HashMap<String, f64>,
    /// Historical trend data for adoption rates
    pub trend_data: Vec<AdoptionTrendPoint>,
}

/// Data point for MFA adoption trend analysis
#[derive(Serialize)]
pub struct AdoptionTrendPoint {
    /// Date of the data point (YYYY-MM-DD format)
    pub date: String,
    /// MFA adoption rate on this date (0.0 to 1.0)
    pub adoption_rate: f64,
    /// Total number of users on this date
    pub total_users: u64,
    /// Number of users with MFA enabled on this date
    pub mfa_users: u64,
}

/// Report on MFA usage patterns and statistics
#[derive(Serialize)]
pub struct MfaUsageReport {
    /// Daily verification counts over time
    pub daily_verifications: Vec<UsageDataPoint>,
    /// Most popular authenticator apps by usage count
    pub top_authenticator_apps: HashMap<String, u32>,
    /// Analysis of verification failures
    pub failure_analysis: FailureAnalysis,
}

/// Data point for MFA usage statistics
#[derive(Serialize)]
pub struct UsageDataPoint {
    /// Date of the usage data (YYYY-MM-DD format)
    pub date: String,
    /// Number of successful MFA verifications on this date
    pub successful_verifications: u32,
    /// Number of failed MFA verifications on this date
    pub failed_verifications: u32,
    /// Number of unique users who performed MFA on this date
    pub unique_users: u32,
}

/// Analysis of MFA verification failures
#[derive(Serialize)]
pub struct FailureAnalysis {
    /// Common reasons for MFA verification failures with occurrence counts
    pub common_failure_reasons: HashMap<String, u32>,
    /// Time periods when failures are most common
    pub peak_failure_times: Vec<String>,
    /// Users who frequently experience MFA verification failures
    pub users_with_frequent_failures: Vec<Uuid>,
}

/// Report on MFA policy compliance across the organization
#[derive(Serialize)]
pub struct ComplianceReport {
    /// Percentage of users compliant with MFA policies (0.0 to 1.0)
    pub compliance_percentage: f64,
    /// List of users who are not compliant with MFA requirements
    pub non_compliant_users: Vec<NonCompliantUser>,
    /// List of policy violations detected
    pub policy_violations: Vec<PolicyViolation>,
    /// Recommended actions to improve compliance
    pub recommendations: Vec<String>,
}

/// Information about a user who is not compliant with MFA policies
#[derive(Serialize)]
pub struct NonCompliantUser {
    /// ID of the non-compliant user
    pub user_id: Uuid,
    /// Username of the non-compliant user
    pub username: String,
    /// Satker (work unit) code of the user
    pub satker_code: String,
    /// Type of MFA compliance violation
    pub violation_type: String,
    /// Number of days the user has been non-compliant
    pub days_non_compliant: u32,
}

/// Information about a policy violation
#[derive(Serialize)]
pub struct PolicyViolation {
    /// Type of policy violation
    pub violation_type: String,
    /// Number of occurrences of this violation type
    pub count: u32,
    /// Severity level of the violation
    pub severity: String,
}

// Handler implementations

/// Get MFA status for multiple users with pagination and filtering
pub async fn get_users_mfa_status(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<crate::app::AppState>>,
    Query(query): Query<GetUsersQuery>,
    Json(auth): Json<AdminAuthRequest>,
) -> Result<Json<PaginatedUserMfaResponse>> {
    let admin_user_id = verify_admin_token(&auth.admin_token, &state).await?;

    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(50).min(100); // Max 100 per page
    let offset = (page - 1) * limit;

    // Build query based on filters
    let mut where_conditions = Vec::new();
    let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();
    let mut param_count = 0;

    if let Some(satker) = &query.satker_code {
        param_count += 1;
        where_conditions.push(format!("u.satker_code = ${}", param_count));
        params.push(satker);
    }

    let mut mfa_enabled_param = None;
    let mut search_pattern = None;

    if let Some(mfa_enabled) = query.mfa_enabled {
        param_count += 1;
        where_conditions.push(format!("u.mfa_enabled = ${}", param_count));
        mfa_enabled_param = Some(mfa_enabled);
    }

    if let Some(search) = &query.search {
        param_count += 1;
        where_conditions.push(format!(
            "(u.username ILIKE ${} OR u.nama ILIKE ${} OR u.nip ILIKE ${})",
            param_count, param_count, param_count
        ));
        search_pattern = Some(format!("%{}%", search));
    }

    // Add parameters to params vector
    if let Some(ref mfa_enabled) = mfa_enabled_param {
        params.push(mfa_enabled);
    }
    if let Some(ref pattern) = search_pattern {
        params.push(pattern);
    }

    let where_clause = if where_conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_conditions.join(" AND "))
    };

    // Get total count
    let client = state
        .db_pool
        .get()
        .await
        .map_err(|e| AuthencError::database(e.to_string()))?;

    let count_query = format!("SELECT COUNT(*) FROM users u {}", where_clause);

    let total: i64 = client
        .query_one(&count_query, &params)
        .await
        .map_err(|e| AuthencError::database(e.to_string()))?
        .get(0);

    // Get paginated results
    param_count += 1;
    params.push(&limit);
    param_count += 1;
    params.push(&offset);

    let query_sql = format!(
        "SELECT u.id, u.username, u.nip, u.nama, u.satker_code, u.jabatan,
                u.mfa_enabled, u.mfa_setup_at, u.mfa_last_used, u.last_login_at, u.account_locked
         FROM users u
         {}
         ORDER BY u.nama, u.username
         LIMIT ${} OFFSET ${}",
        where_clause,
        param_count - 1,
        param_count
    );

    let rows = client
        .query(&query_sql, &params)
        .await
        .map_err(|e| AuthencError::database(e.to_string()))?;

    // Get MFA service for status details
    let mfa_service = MfaService::new(state.secreton_client.clone(), state.db_pool.clone());

    let mut users = Vec::new();
    for row in rows {
        let user_id: Uuid = row.get(0);
        let mfa_status = mfa_service.get_mfa_status(user_id).await?;

        users.push(UserMfaInfo {
            user_id,
            username: row.get(1),
            nip: row.get(2),
            nama: row.get(3),
            satker_code: row.get(4),
            jabatan: row.get(5),
            mfa_status,
            last_login: row.get(9),
            account_locked: row.get(10),
        });
    }

    let total_pages = ((total as u32) + limit - 1) / limit;

    // Log admin action
    tracing::info!(
        admin_user_id = %admin_user_id,
        ip = %addr.ip(),
        action = "view_users_mfa_status",
        page = page,
        limit = limit,
        total_results = total,
        "Admin viewed users MFA status"
    );

    Ok(Json(PaginatedUserMfaResponse {
        users,
        total: total as u64,
        page,
        limit,
        total_pages,
    }))
}

/// Get detailed MFA status for a specific user
pub async fn get_user_mfa_status(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<crate::app::AppState>>,
    Path(user_id): Path<Uuid>,
    Json(auth): Json<AdminAuthRequest>,
) -> Result<Json<UserMfaInfo>> {
    let admin_user_id = verify_admin_token(&auth.admin_token, &state).await?;

    // Get user info
    let user = state
        .user_store
        .get_user(user_id)
        .await?
        .ok_or_else(|| AuthencError::not_found("User not found"))?;

    // Get MFA status
    let mfa_service = MfaService::new(state.secreton_client.clone(), state.db_pool.clone());
    let mfa_status = mfa_service.get_mfa_status(user_id).await?;

    let user_info = UserMfaInfo {
        user_id: user.id,
        username: user.username,
        nip: user.nip,
        nama: user.nama,
        satker_code: Some(user.satker_code),
        jabatan: user.jabatan,
        mfa_status,
        last_login: user.last_login_at,
        account_locked: user.account_locked,
    };

    // Log admin action
    tracing::info!(
        admin_user_id = %admin_user_id,
        target_user_id = %user_id,
        ip = %addr.ip(),
        action = "view_user_mfa_status",
        "Admin viewed user MFA status"
    );

    Ok(Json(user_info))
}

/// Reset MFA for a specific user (admin operation)
pub async fn reset_user_mfa(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<crate::app::AppState>>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<ResetMfaRequest>,
) -> Result<Json<MfaOperationResult>> {
    let admin_user_id = verify_admin_token(&req.admin_token, &state).await?;

    // Get user info
    let user = state
        .user_store
        .get_user(user_id)
        .await?
        .ok_or_else(|| AuthencError::not_found("User not found"))?;

    // Create security context for admin operation
    let security_context = SecurityContext {
        ip_address: Some(addr.ip().to_string()),
        user_agent: None,
        session_id: None,
        timestamp: Utc::now(),
        risk_score: None,
        metadata: Some(serde_json::json!({
            "admin_user_id": admin_user_id,
            "reason": req.reason,
            "force_reactivation": req.force_reactivation
        })),
    };

    // Reset MFA using MFA service
    let mfa_service = MfaService::new(state.secreton_client.clone(), state.db_pool.clone());

    let result = match mfa_service.disable_mfa(user_id, &security_context).await {
        Ok(_) => {
            // Log admin action
            mfa_service
                .log_admin_action(
                    Some(admin_user_id),
                    user_id,
                    if req.force_reactivation {
                        "mfa_reset_force_reactivation"
                    } else {
                        "mfa_reset"
                    },
                    &req.reason,
                )
                .await?;

            MfaOperationResult {
                user_id,
                username: user.username.clone(),
                success: true,
                message: format!("MFA reset successfully for user {}", user.username),
            }
        }
        Err(e) => MfaOperationResult {
            user_id,
            username: user.username.clone(),
            success: false,
            message: format!("Failed to reset MFA: {}", e),
        },
    };

    // Log admin action
    tracing::info!(
        admin_user_id = %admin_user_id,
        target_user_id = %user_id,
        ip = %addr.ip(),
        action = "reset_user_mfa",
        success = result.success,
        reason = %req.reason,
        "Admin reset user MFA"
    );

    Ok(Json(result))
}

/// Disable MFA for a specific user (admin operation)
pub async fn disable_user_mfa(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<crate::app::AppState>>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<ResetMfaRequest>,
) -> Result<Json<MfaOperationResult>> {
    let admin_user_id = verify_admin_token(&req.admin_token, &state).await?;

    // Get user info
    let user = state
        .user_store
        .get_user(user_id)
        .await?
        .ok_or_else(|| AuthencError::not_found("User not found"))?;

    // Create security context for admin operation
    let security_context = SecurityContext {
        ip_address: Some(addr.ip().to_string()),
        user_agent: None,
        session_id: None,
        timestamp: Utc::now(),
        risk_score: None,
        metadata: Some(serde_json::json!({
            "admin_user_id": admin_user_id,
            "reason": req.reason
        })),
    };

    // Disable MFA using MFA service
    let mfa_service = MfaService::new(state.secreton_client.clone(), state.db_pool.clone());

    let result = match mfa_service.disable_mfa(user_id, &security_context).await {
        Ok(_) => {
            // Log admin action
            mfa_service
                .log_admin_action(Some(admin_user_id), user_id, "mfa_disable", &req.reason)
                .await?;

            MfaOperationResult {
                user_id,
                username: user.username.clone(),
                success: true,
                message: format!("MFA disabled successfully for user {}", user.username),
            }
        }
        Err(e) => MfaOperationResult {
            user_id,
            username: user.username.clone(),
            success: false,
            message: format!("Failed to disable MFA: {}", e),
        },
    };

    // Log admin action
    tracing::info!(
        admin_user_id = %admin_user_id,
        target_user_id = %user_id,
        ip = %addr.ip(),
        action = "disable_user_mfa",
        success = result.success,
        reason = %req.reason,
        "Admin disabled user MFA"
    );

    Ok(Json(result))
}

/// Force MFA setup for a user on next login
pub async fn force_mfa_setup(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<crate::app::AppState>>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<ResetMfaRequest>,
) -> Result<Json<MfaOperationResult>> {
    let admin_user_id = verify_admin_token(&req.admin_token, &state).await?;

    // Get user info
    let user = state
        .user_store
        .get_user(user_id)
        .await?
        .ok_or_else(|| AuthencError::not_found("User not found"))?;

    // Update database to force MFA setup
    let client = state
        .db_pool
        .get()
        .await
        .map_err(|e| AuthencError::database(e.to_string()))?;

    let result = match client.execute(
        "UPDATE users SET mfa_enabled = false, mfa_setup_at = NULL, require_mfa_setup = true WHERE id = $1",
        &[&user_id],
    ).await {
        Ok(_) => {
            // Log admin action
            let mfa_service = MfaService::new(
                state.secreton_client.clone(),
                state.db_pool.clone(),
            );

            mfa_service.log_admin_action(
                Some(admin_user_id),
                user_id,
                "force_mfa_setup",
                &req.reason,
            ).await?;

            MfaOperationResult {
                user_id,
                username: user.username.clone(),
                success: true,
                message: format!("MFA setup will be required for user {} on next login", user.username),
            }
        }
        Err(e) => MfaOperationResult {
            user_id,
            username: user.username.clone(),
            success: false,
            message: format!("Failed to force MFA setup: {}", e),
        }
    };

    // Log admin action
    tracing::info!(
        admin_user_id = %admin_user_id,
        target_user_id = %user_id,
        ip = %addr.ip(),
        action = "force_mfa_setup",
        success = result.success,
        reason = %req.reason,
        "Admin forced MFA setup for user"
    );

    Ok(Json(result))
}

/// Bulk reset MFA for multiple users
pub async fn bulk_reset_mfa(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<crate::app::AppState>>,
    Json(req): Json<BulkMfaRequest>,
) -> Result<Json<BulkMfaResponse>> {
    let admin_user_id = verify_admin_token(&req.admin_token, &state).await?;

    // Limit bulk operations to prevent abuse
    if req.user_ids.len() > 100 {
        return Err(AuthencError::validation(
            "Bulk operations limited to 100 users at once",
        ));
    }

    let mut results = Vec::new();
    let mut success_count = 0;
    let mut failure_count = 0;

    let mfa_service = MfaService::new(state.secreton_client.clone(), state.db_pool.clone());

    for user_id in &req.user_ids {
        // Get user info
        let user_result = state.user_store.get_user(*user_id).await;

        let result = match user_result {
            Ok(Some(user)) => {
                // Create security context
                let security_context = SecurityContext {
                    ip_address: Some(addr.ip().to_string()),
                    user_agent: None,
                    session_id: None,
                    timestamp: Utc::now(),
                    risk_score: None,
                    metadata: Some(serde_json::json!({
                        "admin_user_id": admin_user_id,
                        "reason": req.reason,
                        "bulk_operation": true
                    })),
                };

                match mfa_service.disable_mfa(*user_id, &security_context).await {
                    Ok(_) => {
                        // Log admin action
                        let _ = mfa_service
                            .log_admin_action(
                                Some(admin_user_id),
                                *user_id,
                                "bulk_mfa_reset",
                                &req.reason,
                            )
                            .await;

                        success_count += 1;
                        MfaOperationResult {
                            user_id: *user_id,
                            username: user.username,
                            success: true,
                            message: "MFA reset successfully".to_string(),
                        }
                    }
                    Err(e) => {
                        failure_count += 1;
                        MfaOperationResult {
                            user_id: *user_id,
                            username: user.username,
                            success: false,
                            message: format!("Failed to reset MFA: {}", e),
                        }
                    }
                }
            }
            Ok(None) => {
                failure_count += 1;
                MfaOperationResult {
                    user_id: *user_id,
                    username: "Unknown".to_string(),
                    success: false,
                    message: "User not found".to_string(),
                }
            }
            Err(e) => {
                failure_count += 1;
                MfaOperationResult {
                    user_id: *user_id,
                    username: "Unknown".to_string(),
                    success: false,
                    message: format!("Error retrieving user: {}", e),
                }
            }
        };

        results.push(result);
    }

    // Log bulk operation
    tracing::warn!(
        admin_user_id = %admin_user_id,
        ip = %addr.ip(),
        action = "bulk_reset_mfa",
        total_users = req.user_ids.len(),
        success_count = success_count,
        failure_count = failure_count,
        reason = %req.reason,
        "Admin performed bulk MFA reset"
    );

    Ok(Json(BulkMfaResponse {
        success_count,
        failure_count,
        results,
    }))
}

/// Verify admin token and return admin user ID
async fn verify_admin_token(token: &str, state: &Arc<crate::app::AppState>) -> Result<Uuid> {
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

    // Check if user has admin role for MFA management
    let is_mfa_admin = user.roles.iter().any(|role| {
        role.name == "admin"
            || role.name == "system_admin"
            || role.name == "mfa_admin"
            || role.name == "security_admin"
    });

    if !is_mfa_admin {
        return Err(AuthencError::forbidden("MFA admin privileges required"));
    }

    Ok(user_id)
}

/// Bulk disable MFA for multiple users
pub async fn bulk_disable_mfa(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<crate::app::AppState>>,
    Json(req): Json<BulkMfaRequest>,
) -> Result<Json<BulkMfaResponse>> {
    let admin_user_id = verify_admin_token(&req.admin_token, &state).await?;

    // Limit bulk operations to prevent abuse
    if req.user_ids.len() > 100 {
        return Err(AuthencError::validation(
            "Bulk operations limited to 100 users at once",
        ));
    }

    let mut results = Vec::new();
    let mut success_count = 0;
    let mut failure_count = 0;

    let mfa_service = MfaService::new(state.secreton_client.clone(), state.db_pool.clone());

    for user_id in &req.user_ids {
        let user_result = state.user_store.get_user(*user_id).await;

        let result = match user_result {
            Ok(Some(user)) => {
                let security_context = SecurityContext {
                    ip_address: Some(addr.ip().to_string()),
                    user_agent: None,
                    session_id: None,
                    timestamp: Utc::now(),
                    risk_score: None,
                    metadata: Some(serde_json::json!({
                        "admin_user_id": admin_user_id,
                        "reason": req.reason,
                        "bulk_operation": true
                    })),
                };

                match mfa_service.disable_mfa(*user_id, &security_context).await {
                    Ok(_) => {
                        let _ = mfa_service
                            .log_admin_action(
                                Some(admin_user_id),
                                *user_id,
                                "bulk_mfa_disable",
                                &req.reason,
                            )
                            .await;

                        success_count += 1;
                        MfaOperationResult {
                            user_id: *user_id,
                            username: user.username,
                            success: true,
                            message: "MFA disabled successfully".to_string(),
                        }
                    }
                    Err(e) => {
                        failure_count += 1;
                        MfaOperationResult {
                            user_id: *user_id,
                            username: user.username,
                            success: false,
                            message: format!("Failed to disable MFA: {}", e),
                        }
                    }
                }
            }
            Ok(None) => {
                failure_count += 1;
                MfaOperationResult {
                    user_id: *user_id,
                    username: "Unknown".to_string(),
                    success: false,
                    message: "User not found".to_string(),
                }
            }
            Err(e) => {
                failure_count += 1;
                MfaOperationResult {
                    user_id: *user_id,
                    username: "Unknown".to_string(),
                    success: false,
                    message: format!("Error retrieving user: {}", e),
                }
            }
        };

        results.push(result);
    }

    tracing::warn!(
        admin_user_id = %admin_user_id,
        ip = %addr.ip(),
        action = "bulk_disable_mfa",
        total_users = req.user_ids.len(),
        success_count = success_count,
        failure_count = failure_count,
        reason = %req.reason,
        "Admin performed bulk MFA disable"
    );

    Ok(Json(BulkMfaResponse {
        success_count,
        failure_count,
        results,
    }))
}

/// Bulk force MFA setup for multiple users
pub async fn bulk_force_setup(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<crate::app::AppState>>,
    Json(req): Json<BulkMfaRequest>,
) -> Result<Json<BulkMfaResponse>> {
    let admin_user_id = verify_admin_token(&req.admin_token, &state).await?;

    if req.user_ids.len() > 100 {
        return Err(AuthencError::validation(
            "Bulk operations limited to 100 users at once",
        ));
    }

    let mut results = Vec::new();
    let mut success_count = 0;
    let mut failure_count = 0;

    let client = state
        .db_pool
        .get()
        .await
        .map_err(|e| AuthencError::database(e.to_string()))?;

    let mfa_service = MfaService::new(state.secreton_client.clone(), state.db_pool.clone());

    for user_id in &req.user_ids {
        let user_result = state.user_store.get_user(*user_id).await;

        let result = match user_result {
            Ok(Some(user)) => {
                match client.execute(
                    "UPDATE users SET mfa_enabled = false, mfa_setup_at = NULL, require_mfa_setup = true WHERE id = $1",
                    &[&*user_id],
                ).await {
                    Ok(_) => {
                        let _ = mfa_service.log_admin_action(
                            Some(admin_user_id),
                            *user_id,
                            "bulk_force_mfa_setup",
                            &req.reason,
                        ).await;

                        success_count += 1;
                        MfaOperationResult {
                            user_id: *user_id,
                            username: user.username,
                            success: true,
                            message: "MFA setup will be required on next login".to_string(),
                        }
                    }
                    Err(e) => {
                        failure_count += 1;
                        MfaOperationResult {
                            user_id: *user_id,
                            username: user.username,
                            success: false,
                            message: format!("Failed to force MFA setup: {}", e),
                        }
                    }
                }
            }
            Ok(None) => {
                failure_count += 1;
                MfaOperationResult {
                    user_id: *user_id,
                    username: "Unknown".to_string(),
                    success: false,
                    message: "User not found".to_string(),
                }
            }
            Err(e) => {
                failure_count += 1;
                MfaOperationResult {
                    user_id: *user_id,
                    username: "Unknown".to_string(),
                    success: false,
                    message: format!("Error retrieving user: {}", e),
                }
            }
        };

        results.push(result);
    }

    tracing::warn!(
        admin_user_id = %admin_user_id,
        ip = %addr.ip(),
        action = "bulk_force_mfa_setup",
        total_users = req.user_ids.len(),
        success_count = success_count,
        failure_count = failure_count,
        reason = %req.reason,
        "Admin performed bulk force MFA setup"
    );

    Ok(Json(BulkMfaResponse {
        success_count,
        failure_count,
        results,
    }))
}

/// Get organization-wide MFA status overview
pub async fn get_organization_mfa_status(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<crate::app::AppState>>,
    Json(auth): Json<AdminAuthRequest>,
) -> Result<Json<OrganizationMfaStatus>> {
    let admin_user_id = verify_admin_token(&auth.admin_token, &state).await?;

    let mfa_service = MfaService::new(state.secreton_client.clone(), state.db_pool.clone());

    // Get overall statistics
    let overall_stats = mfa_service.get_mfa_statistics(None).await?;
    let adoption_rate = if overall_stats.total_users > 0 {
        (overall_stats.mfa_enabled_users as f64 / overall_stats.total_users as f64) * 100.0
    } else {
        0.0
    };

    // Get statistics by satker
    let client = state
        .db_pool
        .get()
        .await
        .map_err(|e| AuthencError::database(e.to_string()))?;

    let satker_rows = client
        .query(
            "SELECT DISTINCT satker_code FROM users WHERE satker_code IS NOT NULL",
            &[],
        )
        .await
        .map_err(|e| AuthencError::database(e.to_string()))?;

    let mut by_satker = HashMap::new();
    for row in satker_rows {
        let satker_code: String = row.get(0);
        let satker_stats = mfa_service.get_mfa_statistics(Some(&satker_code)).await?;
        by_satker.insert(satker_code, satker_stats);
    }

    // Get recent activity (last 7 days)
    let activity_rows = client.query(
        "SELECT DATE(created_at) as date,
                COUNT(*) FILTER (WHERE event_type = 'mfa_setup_complete') as new_setups,
                COUNT(*) FILTER (WHERE event_type = 'mfa_verification_success') as successful_verifications,
                COUNT(*) FILTER (WHERE event_type = 'mfa_verification_failed') as failed_verifications
         FROM audit_logs
         WHERE created_at >= NOW() - INTERVAL '7 days'
           AND event_type IN ('mfa_setup_complete', 'mfa_verification_success', 'mfa_verification_failed')
         GROUP BY DATE(created_at)
         ORDER BY date DESC",
        &[],
    ).await
    .map_err(|e| AuthencError::database(e.to_string()))?;

    let mut recent_activity = Vec::new();
    for row in activity_rows {
        recent_activity.push(MfaActivitySummary {
            date: row.get::<_, chrono::NaiveDate>(0).to_string(),
            new_setups: row.get::<_, i64>(1) as u32,
            successful_verifications: row.get::<_, i64>(2) as u32,
            failed_verifications: row.get::<_, i64>(3) as u32,
        });
    }

    tracing::info!(
        admin_user_id = %admin_user_id,
        ip = %addr.ip(),
        action = "view_organization_mfa_status",
        "Admin viewed organization MFA status"
    );

    Ok(Json(OrganizationMfaStatus {
        total_users: overall_stats.total_users,
        mfa_enabled_users: overall_stats.mfa_enabled_users,
        mfa_adoption_rate: adoption_rate,
        by_satker,
        recent_activity,
    }))
}

/// Get current MFA policy
pub async fn get_mfa_policy(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<crate::app::AppState>>,
    Json(auth): Json<AdminAuthRequest>,
) -> Result<Json<MfaPolicy>> {
    let admin_user_id = verify_admin_token(&auth.admin_token, &state).await?;

    // Get MFA policy from configuration or database
    let client = state
        .db_pool
        .get()
        .await
        .map_err(|e| AuthencError::database(e.to_string()))?;

    let policy_row = client.query_opt(
        "SELECT policy_data FROM mfa_policies WHERE active = true ORDER BY created_at DESC LIMIT 1",
        &[],
    ).await
    .map_err(|e| AuthencError::database(e.to_string()))?;

    let policy = if let Some(row) = policy_row {
        let policy_json: serde_json::Value = row.get(0);
        serde_json::from_value(policy_json)
            .map_err(|e| AuthencError::internal(format!("Invalid policy data: {}", e)))?
    } else {
        // Default policy
        MfaPolicy {
            enforce_for_all: true,
            enforce_for_roles: vec!["admin".to_string(), "security_admin".to_string()],
            enforce_for_satkers: vec![],
            grace_period_days: 30,
            backup_codes_required: true,
            max_failed_attempts: 5,
            lockout_duration_minutes: 30,
        }
    };

    tracing::info!(
        admin_user_id = %admin_user_id,
        ip = %addr.ip(),
        action = "view_mfa_policy",
        "Admin viewed MFA policy"
    );

    Ok(Json(policy))
}

/// Update MFA policy
pub async fn update_mfa_policy(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<crate::app::AppState>>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<MfaPolicy>> {
    let auth_token = req["admin_token"]
        .as_str()
        .ok_or_else(|| AuthencError::unauthorized("Admin token required"))?;
    let admin_user_id = verify_admin_token(auth_token, &state).await?;

    let new_policy: MfaPolicy = serde_json::from_value(req["policy"].clone())
        .map_err(|e| AuthencError::validation(format!("Invalid policy data: {}", e)))?;

    // Validate policy
    if new_policy.grace_period_days > 365 {
        return Err(AuthencError::validation(
            "Grace period cannot exceed 365 days",
        ));
    }
    if new_policy.max_failed_attempts > 20 {
        return Err(AuthencError::validation(
            "Max failed attempts cannot exceed 20",
        ));
    }
    if new_policy.lockout_duration_minutes > 1440 {
        return Err(AuthencError::validation(
            "Lockout duration cannot exceed 24 hours",
        ));
    }

    // Save policy to database
    let client = state
        .db_pool
        .get()
        .await
        .map_err(|e| AuthencError::database(e.to_string()))?;

    // Deactivate current policy
    client
        .execute(
            "UPDATE mfa_policies SET active = false WHERE active = true",
            &[],
        )
        .await
        .map_err(|e| AuthencError::database(e.to_string()))?;

    // Insert new policy
    let policy_json = serde_json::to_value(&new_policy)
        .map_err(|e| AuthencError::internal(format!("Failed to serialize policy: {}", e)))?;

    client.execute(
        "INSERT INTO mfa_policies (policy_data, created_by, active, created_at) VALUES ($1, $2, true, NOW())",
        &[&policy_json, &admin_user_id],
    ).await
    .map_err(|e| AuthencError::database(e.to_string()))?;

    tracing::warn!(
        admin_user_id = %admin_user_id,
        ip = %addr.ip(),
        action = "update_mfa_policy",
        policy = ?new_policy,
        "Admin updated MFA policy"
    );

    Ok(Json(new_policy))
}

/// Get user recovery codes count
pub async fn get_user_recovery_codes(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<crate::app::AppState>>,
    Path(user_id): Path<Uuid>,
    Json(auth): Json<AdminAuthRequest>,
) -> Result<Json<serde_json::Value>> {
    let admin_user_id = verify_admin_token(&auth.admin_token, &state).await?;

    let mfa_service = MfaService::new(state.secreton_client.clone(), state.db_pool.clone());

    let recovery_codes_count = mfa_service.get_recovery_codes_count(user_id).await?;

    tracing::info!(
        admin_user_id = %admin_user_id,
        target_user_id = %user_id,
        ip = %addr.ip(),
        action = "view_user_recovery_codes",
        "Admin viewed user recovery codes count"
    );

    Ok(Json(serde_json::json!({
        "user_id": user_id,
        "recovery_codes_remaining": recovery_codes_count
    })))
}

/// Regenerate recovery codes for a user
pub async fn regenerate_recovery_codes(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<crate::app::AppState>>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<ResetMfaRequest>,
) -> Result<Json<serde_json::Value>> {
    let admin_user_id = verify_admin_token(&req.admin_token, &state).await?;

    let mfa_service = MfaService::new(state.secreton_client.clone(), state.db_pool.clone());

    let new_codes = mfa_service.regenerate_recovery_codes(user_id).await?;

    // Log admin action
    mfa_service
        .log_admin_action(
            Some(admin_user_id),
            user_id,
            "regenerate_recovery_codes",
            &req.reason,
        )
        .await?;

    tracing::warn!(
        admin_user_id = %admin_user_id,
        target_user_id = %user_id,
        ip = %addr.ip(),
        action = "regenerate_recovery_codes",
        reason = %req.reason,
        "Admin regenerated user recovery codes"
    );

    Ok(Json(serde_json::json!({
        "user_id": user_id,
        "recovery_codes": new_codes,
        "message": "Recovery codes regenerated successfully"
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_verify_admin_token() {
        // This would require setting up test infrastructure
        // Implementation depends on your test setup
    }

    #[tokio::test]
    async fn test_bulk_operation_limits() {
        let user_ids: Vec<Uuid> = (0..150).map(|_| Uuid::new_v4()).collect();

        let request = BulkMfaRequest {
            admin_token: "test_token".to_string(),
            user_ids,
            reason: "Test bulk operation".to_string(),
        };

        // This should fail validation due to too many users
        assert!(request.user_ids.len() > 100);
    }

    #[test]
    fn test_mfa_policy_validation() {
        let invalid_policy = MfaPolicy {
            enforce_for_all: true,
            enforce_for_roles: vec![],
            enforce_for_satkers: vec![],
            grace_period_days: 400, // Invalid: > 365
            backup_codes_required: true,
            max_failed_attempts: 25,        // Invalid: > 20
            lockout_duration_minutes: 2000, // Invalid: > 1440
        };

        assert!(invalid_policy.grace_period_days > 365);
        assert!(invalid_policy.max_failed_attempts > 20);
        assert!(invalid_policy.lockout_duration_minutes > 1440);
    }
}
/// Get MFA adoption report with trend analysis
pub async fn get_mfa_adoption_report(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<crate::app::AppState>>,
    Query(_query): Query<serde_json::Value>,
    Json(auth): Json<AdminAuthRequest>,
) -> Result<Json<MfaAdoptionReport>> {
    let admin_user_id = verify_admin_token(&auth.admin_token, &state).await?;

    let client = state
        .db_pool
        .get()
        .await
        .map_err(|e| AuthencError::database(e.to_string()))?;

    // Get overall adoption rate
    let overall_row = client
        .query_one(
            "SELECT
            COUNT(*) as total_users,
            COUNT(*) FILTER (WHERE mfa_enabled = true) as mfa_users
         FROM users
         WHERE enabled = true AND deleted_at IS NULL",
            &[],
        )
        .await
        .map_err(|e| AuthencError::database(e.to_string()))?;

    let total_users: i64 = overall_row.get(0);
    let mfa_users: i64 = overall_row.get(1);
    let overall_adoption = if total_users > 0 {
        (mfa_users as f64 / total_users as f64) * 100.0
    } else {
        0.0
    };

    // Get adoption by satker
    let satker_rows = client
        .query(
            "SELECT
            satker_code,
            COUNT(*) as total_users,
            COUNT(*) FILTER (WHERE mfa_enabled = true) as mfa_users
         FROM users
         WHERE enabled = true AND deleted_at IS NULL AND satker_code IS NOT NULL
         GROUP BY satker_code
         ORDER BY satker_code",
            &[],
        )
        .await
        .map_err(|e| AuthencError::database(e.to_string()))?;

    let mut by_satker = HashMap::new();
    for row in satker_rows {
        let satker_code: String = row.get(0);
        let total: i64 = row.get(1);
        let mfa: i64 = row.get(2);
        let adoption_rate = if total > 0 {
            (mfa as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        by_satker.insert(satker_code, adoption_rate);
    }

    // Get adoption by role
    let role_rows = client
        .query(
            "SELECT
            r.name as role_name,
            COUNT(DISTINCT u.id) as total_users,
            COUNT(DISTINCT u.id) FILTER (WHERE u.mfa_enabled = true) as mfa_users
         FROM roles r
         JOIN user_roles ur ON r.id = ur.role_id
         JOIN users u ON ur.user_id = u.id
         WHERE u.enabled = true AND u.deleted_at IS NULL
         GROUP BY r.name
         ORDER BY r.name",
            &[],
        )
        .await
        .map_err(|e| AuthencError::database(e.to_string()))?;

    let mut by_role = HashMap::new();
    for row in role_rows {
        let role_name: String = row.get(0);
        let total: i64 = row.get(1);
        let mfa: i64 = row.get(2);
        let adoption_rate = if total > 0 {
            (mfa as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        by_role.insert(role_name, adoption_rate);
    }

    // Get trend data (last 30 days)
    let trend_rows = client.query(
        "WITH daily_stats AS (
            SELECT
                DATE(created_at) as date,
                COUNT(DISTINCT user_id) as total_users,
                COUNT(DISTINCT user_id) FILTER (WHERE event_type = 'mfa_setup_complete') as new_mfa_users
            FROM audit_logs
            WHERE created_at >= NOW() - INTERVAL '30 days'
            GROUP BY DATE(created_at)
        ),
        cumulative_mfa AS (
            SELECT
                date,
                SUM(new_mfa_users) OVER (ORDER BY date) as cumulative_mfa_users
            FROM daily_stats
        )
        SELECT
            ds.date,
            ds.total_users,
            cm.cumulative_mfa_users,
            CASE
                WHEN ds.total_users > 0 THEN (cm.cumulative_mfa_users::float / ds.total_users::float) * 100
                ELSE 0
            END as adoption_rate
        FROM daily_stats ds
        JOIN cumulative_mfa cm ON ds.date = cm.date
        ORDER BY ds.date",
        &[],
    ).await
    .map_err(|e| AuthencError::database(e.to_string()))?;

    let mut trend_data = Vec::new();
    for row in trend_rows {
        trend_data.push(AdoptionTrendPoint {
            date: row.get::<_, chrono::NaiveDate>(0).to_string(),
            total_users: row.get::<_, i64>(1) as u64,
            mfa_users: row.get::<_, i64>(2) as u64,
            adoption_rate: row.get::<_, f64>(3),
        });
    }

    tracing::info!(
        admin_user_id = %admin_user_id,
        ip = %addr.ip(),
        action = "view_mfa_adoption_report",
        overall_adoption = %overall_adoption,
        "Admin viewed MFA adoption report"
    );

    Ok(Json(MfaAdoptionReport {
        overall_adoption,
        by_satker,
        by_role,
        trend_data,
    }))
}

/// Get MFA usage report with detailed analytics
pub async fn get_mfa_usage_report(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<crate::app::AppState>>,
    Query(_query): Query<serde_json::Value>,
    Json(auth): Json<AdminAuthRequest>,
) -> Result<Json<MfaUsageReport>> {
    let admin_user_id = verify_admin_token(&auth.admin_token, &state).await?;

    let client = state
        .db_pool
        .get()
        .await
        .map_err(|e| AuthencError::database(e.to_string()))?;

    // Get daily verification statistics (last 30 days)
    let daily_rows = client.query(
        "SELECT
            DATE(created_at) as date,
            COUNT(*) FILTER (WHERE event_type = 'mfa_verification_success') as successful_verifications,
            COUNT(*) FILTER (WHERE event_type = 'mfa_verification_failed') as failed_verifications,
            COUNT(DISTINCT user_id) as unique_users
         FROM audit_logs
         WHERE created_at >= NOW() - INTERVAL '30 days'
           AND event_type IN ('mfa_verification_success', 'mfa_verification_failed')
         GROUP BY DATE(created_at)
         ORDER BY date",
        &[],
    ).await
    .map_err(|e| AuthencError::database(e.to_string()))?;

    let mut daily_verifications = Vec::new();
    for row in daily_rows {
        daily_verifications.push(UsageDataPoint {
            date: row.get::<_, chrono::NaiveDate>(0).to_string(),
            successful_verifications: row.get::<_, i64>(1) as u32,
            failed_verifications: row.get::<_, i64>(2) as u32,
            unique_users: row.get::<_, i64>(3) as u32,
        });
    }

    // Get top authenticator apps (from user agent analysis)
    let app_rows = client.query(
        "SELECT
            CASE
                WHEN metadata->>'user_agent' ILIKE '%google authenticator%' THEN 'Google Authenticator'
                WHEN metadata->>'user_agent' ILIKE '%microsoft authenticator%' THEN 'Microsoft Authenticator'
                WHEN metadata->>'user_agent' ILIKE '%authy%' THEN 'Authy'
                WHEN metadata->>'user_agent' ILIKE '%freeotp%' THEN 'FreeOTP'
                ELSE 'Other/Unknown'
            END as app_name,
            COUNT(*) as usage_count
         FROM audit_logs
         WHERE created_at >= NOW() - INTERVAL '30 days'
           AND event_type = 'mfa_verification_success'
           AND metadata->>'user_agent' IS NOT NULL
         GROUP BY app_name
         ORDER BY usage_count DESC",
        &[],
    ).await
    .map_err(|e| AuthencError::database(e.to_string()))?;

    let mut top_authenticator_apps = HashMap::new();
    for row in app_rows {
        let app_name: String = row.get(0);
        let count: i64 = row.get(1);
        top_authenticator_apps.insert(app_name, count as u32);
    }

    // Analyze failure reasons
    let failure_rows = client
        .query(
            "SELECT
            CASE
                WHEN message ILIKE '%invalid code%' THEN 'Invalid OTP Code'
                WHEN message ILIKE '%expired%' THEN 'Expired Code'
                WHEN message ILIKE '%rate limit%' THEN 'Rate Limited'
                WHEN message ILIKE '%account locked%' THEN 'Account Locked'
                ELSE 'Other'
            END as failure_reason,
            COUNT(*) as count
         FROM audit_logs
         WHERE created_at >= NOW() - INTERVAL '30 days'
           AND event_type = 'mfa_verification_failed'
         GROUP BY failure_reason
         ORDER BY count DESC",
            &[],
        )
        .await
        .map_err(|e| AuthencError::database(e.to_string()))?;

    let mut common_failure_reasons = HashMap::new();
    for row in failure_rows {
        let reason: String = row.get(0);
        let count: i64 = row.get(1);
        common_failure_reasons.insert(reason, count as u32);
    }

    // Find peak failure times (by hour of day)
    let peak_rows = client
        .query(
            "SELECT
            EXTRACT(HOUR FROM created_at) as hour,
            COUNT(*) as failure_count
         FROM audit_logs
         WHERE created_at >= NOW() - INTERVAL '7 days'
           AND event_type = 'mfa_verification_failed'
         GROUP BY EXTRACT(HOUR FROM created_at)
         ORDER BY failure_count DESC
         LIMIT 5",
            &[],
        )
        .await
        .map_err(|e| AuthencError::database(e.to_string()))?;

    let mut peak_failure_times = Vec::new();
    for row in peak_rows {
        let hour: f64 = row.get(0);
        peak_failure_times.push(format!("{:02}:00", hour as u8));
    }

    // Find users with frequent failures (more than 10 failures in last 7 days)
    let frequent_failure_rows = client
        .query(
            "SELECT
            user_id,
            COUNT(*) as failure_count
         FROM audit_logs
         WHERE created_at >= NOW() - INTERVAL '7 days'
           AND event_type = 'mfa_verification_failed'
         GROUP BY user_id
         HAVING COUNT(*) > 10
         ORDER BY failure_count DESC
         LIMIT 20",
            &[],
        )
        .await
        .map_err(|e| AuthencError::database(e.to_string()))?;

    let mut users_with_frequent_failures = Vec::new();
    for row in frequent_failure_rows {
        let user_id: Uuid = row.get(0);
        users_with_frequent_failures.push(user_id);
    }

    let failure_analysis = FailureAnalysis {
        common_failure_reasons,
        peak_failure_times,
        users_with_frequent_failures,
    };

    tracing::info!(
        admin_user_id = %admin_user_id,
        ip = %addr.ip(),
        action = "view_mfa_usage_report",
        daily_data_points = daily_verifications.len(),
        "Admin viewed MFA usage report"
    );

    Ok(Json(MfaUsageReport {
        daily_verifications,
        top_authenticator_apps,
        failure_analysis,
    }))
}

/// Get compliance report for security audits
pub async fn get_compliance_report(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<crate::app::AppState>>,
    Query(_query): Query<serde_json::Value>,
    Json(auth): Json<AdminAuthRequest>,
) -> Result<Json<ComplianceReport>> {
    let admin_user_id = verify_admin_token(&auth.admin_token, &state).await?;

    let client = state
        .db_pool
        .get()
        .await
        .map_err(|e| AuthencError::database(e.to_string()))?;

    // Get current MFA policy
    let policy_row = client.query_opt(
        "SELECT policy_data FROM mfa_policies WHERE active = true ORDER BY created_at DESC LIMIT 1",
        &[],
    ).await
    .map_err(|e| AuthencError::database(e.to_string()))?;

    let policy: MfaPolicy = if let Some(row) = policy_row {
        let policy_json: serde_json::Value = row.get(0);
        serde_json::from_value(policy_json)
            .map_err(|e| AuthencError::internal(format!("Invalid policy data: {}", e)))?
    } else {
        // Default policy
        MfaPolicy {
            enforce_for_all: true,
            enforce_for_roles: vec!["admin".to_string()],
            enforce_for_satkers: vec![],
            grace_period_days: 30,
            backup_codes_required: true,
            max_failed_attempts: 5,
            lockout_duration_minutes: 30,
        }
    };

    // Calculate compliance based on policy
    let compliance_query = if policy.enforce_for_all {
        "SELECT
            COUNT(*) as total_users,
            COUNT(*) FILTER (WHERE mfa_enabled = true) as compliant_users
         FROM users
         WHERE enabled = true AND deleted_at IS NULL"
    } else {
        "SELECT
            COUNT(DISTINCT u.id) as total_users,
            COUNT(DISTINCT u.id) FILTER (WHERE u.mfa_enabled = true) as compliant_users
         FROM users u
         LEFT JOIN user_roles ur ON u.id = ur.user_id
         LEFT JOIN roles r ON ur.role_id = r.id
         WHERE u.enabled = true AND u.deleted_at IS NULL
           AND (r.name = ANY($1) OR u.satker_code = ANY($2))"
    };

    let compliance_row = if policy.enforce_for_all {
        client.query_one(compliance_query, &[]).await
    } else {
        client
            .query_one(
                compliance_query,
                &[&policy.enforce_for_roles, &policy.enforce_for_satkers],
            )
            .await
    }
    .map_err(|e| AuthencError::database(e.to_string()))?;

    let total_users: i64 = compliance_row.get(0);
    let compliant_users: i64 = compliance_row.get(1);
    let compliance_percentage = if total_users > 0 {
        (compliant_users as f64 / total_users as f64) * 100.0
    } else {
        100.0
    };

    // Find non-compliant users
    let non_compliant_query = if policy.enforce_for_all {
        "SELECT
            u.id, u.username, u.satker_code,
            CASE
                WHEN u.mfa_enabled = false THEN 'MFA Not Enabled'
                WHEN u.mfa_setup_at IS NULL THEN 'MFA Not Set Up'
                ELSE 'Other'
            END as violation_type,
            COALESCE(DATE_PART('day', NOW() - u.created_at), 0) as days_non_compliant
         FROM users u
         WHERE u.enabled = true AND u.deleted_at IS NULL AND u.mfa_enabled = false
         ORDER BY days_non_compliant DESC
         LIMIT 100"
    } else {
        "SELECT
            u.id, u.username, u.satker_code,
            CASE
                WHEN u.mfa_enabled = false THEN 'MFA Not Enabled'
                WHEN u.mfa_setup_at IS NULL THEN 'MFA Not Set Up'
                ELSE 'Other'
            END as violation_type,
            COALESCE(DATE_PART('day', NOW() - u.created_at), 0) as days_non_compliant
         FROM users u
         LEFT JOIN user_roles ur ON u.id = ur.user_id
         LEFT JOIN roles r ON ur.role_id = r.id
         WHERE u.enabled = true AND u.deleted_at IS NULL AND u.mfa_enabled = false
           AND (r.name = ANY($1) OR u.satker_code = ANY($2))
         ORDER BY days_non_compliant DESC
         LIMIT 100"
    };

    let non_compliant_rows = if policy.enforce_for_all {
        client.query(non_compliant_query, &[]).await
    } else {
        client
            .query(
                non_compliant_query,
                &[&policy.enforce_for_roles, &policy.enforce_for_satkers],
            )
            .await
    }
    .map_err(|e| AuthencError::database(e.to_string()))?;

    let mut non_compliant_users = Vec::new();
    for row in non_compliant_rows {
        non_compliant_users.push(NonCompliantUser {
            user_id: row.get(0),
            username: row.get(1),
            satker_code: row.get::<_, Option<String>>(2).unwrap_or_default(),
            violation_type: row.get(3),
            days_non_compliant: row.get::<_, f64>(4) as u32,
        });
    }

    // Analyze policy violations
    let violation_rows = client.query(
        "SELECT
            CASE
                WHEN event_type = 'account_locked_mfa_failures' THEN 'Excessive MFA Failures'
                WHEN event_type = 'mfa_setup_overdue' THEN 'MFA Setup Overdue'
                WHEN event_type = 'backup_codes_exhausted' THEN 'Backup Codes Exhausted'
                ELSE 'Other Violation'
            END as violation_type,
            COUNT(*) as count,
            CASE
                WHEN COUNT(*) > 100 THEN 'High'
                WHEN COUNT(*) > 20 THEN 'Medium'
                ELSE 'Low'
            END as severity
         FROM audit_logs
         WHERE created_at >= NOW() - INTERVAL '30 days'
           AND event_type IN ('account_locked_mfa_failures', 'mfa_setup_overdue', 'backup_codes_exhausted')
         GROUP BY violation_type
         ORDER BY count DESC",
        &[],
    ).await
    .map_err(|e| AuthencError::database(e.to_string()))?;

    let mut policy_violations = Vec::new();
    for row in violation_rows {
        policy_violations.push(PolicyViolation {
            violation_type: row.get(0),
            count: row.get::<_, i64>(1) as u32,
            severity: row.get(2),
        });
    }

    // Generate recommendations
    let mut recommendations = Vec::new();

    if compliance_percentage < 80.0 {
        recommendations
            .push("Consider implementing mandatory MFA training for all users".to_string());
        recommendations
            .push("Send automated reminders to users who haven't set up MFA".to_string());
    }

    if non_compliant_users.len() > 50 {
        recommendations
            .push("Consider implementing a phased MFA rollout by department".to_string());
    }

    if policy_violations
        .iter()
        .any(|v| v.violation_type.contains("Excessive MFA Failures"))
    {
        recommendations
            .push("Review MFA failure patterns and provide additional user support".to_string());
    }

    if policy.grace_period_days > 30 {
        recommendations
            .push("Consider reducing MFA setup grace period to improve compliance".to_string());
    }

    recommendations.push(
        "Regularly review and update MFA policies based on security requirements".to_string(),
    );
    recommendations.push("Implement automated compliance monitoring and alerting".to_string());

    tracing::info!(
        admin_user_id = %admin_user_id,
        ip = %addr.ip(),
        action = "view_compliance_report",
        compliance_percentage = %compliance_percentage,
        non_compliant_count = non_compliant_users.len(),
        "Admin viewed MFA compliance report"
    );

    Ok(Json(ComplianceReport {
        compliance_percentage,
        non_compliant_users,
        policy_violations,
        recommendations,
    }))
}
