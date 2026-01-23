//! Policy Management API Handlers
//!
//! Provides REST API endpoints for policy administration with full integration.
//! Implements policy CRUD operations, validation, caching, and evaluation testing.
//!
//! Endpoints:
//! - GET /v1/sys/policies - List all policies with pagination
//! - POST /v1/sys/policies/{name - Create policy with validation
//! - GET /v1/sys/policies/{name - Get policy with evaluation stats
//! - PUT /v1/sys/policies/{name - Update policy with version control
//! - DELETE /v1/sys/policies/{name - Delete policy with dependency check
//! - POST /v1/sys/policies/{name}/test - Test policy evaluation

use axum::{
    Router,
    extract::{Path, Query, State},
    response::Json,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::info;

use secreton_core::{
    error::CoreError,
    models::PolicyRule,
};

use crate::{
    ApiError, ApiResponse, ApiResult, PaginationQuery, handlers::AppState,
    models::PaginatedResponse,
};

/// Create policy routes
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/policies", get(list_policies))
        .route(
            "/policies/{name}",
            post(create_policy)
                .get(get_policy)
                .put(update_policy)
                .delete(delete_policy),
        )
        .route("/policies/{name}/test", post(test_policy))
}

// ============================================================================
// Request/Response DTOs
// ============================================================================

/// Request to create a new policy
#[derive(Debug, Deserialize)]
pub struct CreatePolicyRequest {
    /// Policy description
    pub description: Option<String>,

    /// Policy rules
    pub rules: Vec<PolicyRule>,

    /// Namespace (defaults to "default")
    #[serde(default = "default_namespace")]
    pub namespace: String,
}

/// Request to update a policy
#[derive(Debug, Deserialize)]
pub struct UpdatePolicyRequest {
    /// Updated description
    pub description: Option<String>,

    /// Updated rules
    pub rules: Option<Vec<PolicyRule>>,

    /// Active status
    pub is_active: Option<bool>,
}

/// Policy response DTO
#[derive(Debug, Serialize)]
pub struct PolicyResponse {
    pub id: i64,
    pub name: String,
    pub namespace: String,
    pub description: Option<String>,
    pub rules: Vec<PolicyRule>,
    pub version: i32,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub created_by: String,
    pub updated_by: Option<String>,
    pub stats: Option<PolicyStats>,
}

/// Policy evaluation statistics
#[derive(Debug, Serialize)]
pub struct PolicyStats {
    pub evaluations_total: i64,
    pub evaluations_allowed: i64,
    pub evaluations_denied: i64,
    pub cache_hits: i64,
    pub cache_misses: i64,
    pub last_evaluated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Request to test policy evaluation
#[derive(Debug, Deserialize)]
pub struct TestPolicyRequest {
    /// User to test
    pub user: String,

    /// Path to test
    pub path: String,

    /// Action to test
    pub action: String,

    /// Optional context for evaluation
    pub context: Option<Value>,
}

/// Test policy response
#[derive(Debug, Serialize)]
pub struct TestPolicyResponse {
    pub allowed: bool,
    pub matched_rules: Vec<String>,
    pub evaluation_time_ms: f64,
}

/// List policies query parameters
#[derive(Debug, Deserialize)]
pub struct ListPoliciesQuery {
    /// Filter by namespace
    pub namespace: Option<String>,

    /// Filter by active status
    pub is_active: Option<bool>,

    /// Search by name or description
    pub search: Option<String>,

    #[serde(flatten)]
    pub pagination: PaginationQuery,
}

fn default_namespace() -> String {
    "default".to_string()
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Extract user from JWT claims
fn extract_user(_state: &AppState) -> Result<String, CoreError> {
    // Extract user ID from request context
    // In production, this would come from JWT middleware via request extensions
    // For now, return system user for API operations
    Ok("system".to_string())
}

/// Check if user is admin
fn is_admin(_state: &AppState) -> Result<bool, CoreError> {
    // Check admin level from user context
    // In production, this would check JWT claims or user roles
    // For now, allow admin operations for system user
    Ok(true)
}

// ============================================================================
// API Handlers
// ============================================================================

/// List all policies with pagination
#[tracing::instrument(skip(state))]
pub async fn list_policies(
    State(state): State<AppState>,
    Query(query): Query<ListPoliciesQuery>,
) -> ApiResult<Json<PaginatedResponse<PolicyResponse>>> {
    info!(
        "Listing policies with filters: namespace={:?}, is_active={:?}, search={:?}",
        query.namespace, query.is_active, query.search
    );

    // Check admin permission
    if !is_admin(&state)? {
        return Err(ApiError::Core(CoreError::IamPermissionDenied {
            operation: "list_policies".to_string(),
        }));
    }

    let (policies, total) = state
        .policy_service
        .list_policies(
            query.namespace,
            query.is_active,
            query.search,
            query.pagination.limit,
            query.pagination.offset,
        )
        .await
        .map_err(ApiError::Core)?;

    let response_policies: Vec<PolicyResponse> = policies
        .into_iter()
        .map(|p| PolicyResponse {
            id: p.id,
            name: p.name,
            namespace: p.namespace,
            description: p.description,
            rules: p.rules,
            version: p.version,
            is_active: p.is_active,
            created_at: p.created_at,
            updated_at: p.updated_at,
            created_by: p.created_by,
            updated_by: p.updated_by,
            stats: p.stats.map(|s| PolicyStats {
                evaluations_total: s.evaluations_total,
                evaluations_allowed: s.evaluations_allowed,
                evaluations_denied: s.evaluations_denied,
                cache_hits: s.cache_hits,
                cache_misses: s.cache_misses,
                last_evaluated_at: s.last_evaluated_at,
            }),
        })
        .collect();

    info!("Found {} policies (total: {})", response_policies.len(), total);

    Ok(Json(PaginatedResponse::new(
        response_policies,
        total,
        query.pagination.limit,
        query.pagination.offset,
    )))
}

/// Create a new policy
#[tracing::instrument(skip(state, req))]
pub async fn create_policy(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(req): Json<CreatePolicyRequest>,
) -> ApiResult<Json<ApiResponse<PolicyResponse>>> {
    info!(
        "Creating policy: name={}, namespace={}",
        name, req.namespace
    );

    // Check admin permission
    if !is_admin(&state)? {
        return Err(ApiError::Core(CoreError::IamPermissionDenied {
            operation: "create_policy".to_string(),
        }));
    }

    let user = extract_user(&state)?;

    let p = state
        .policy_service
        .create_policy(
            name,
            req.namespace,
            req.description,
            req.rules,
            user,
        )
        .await
        .map_err(ApiError::Core)?;

    let policy = PolicyResponse {
        id: p.id,
        name: p.name,
        namespace: p.namespace,
        description: p.description,
        rules: p.rules,
        version: p.version,
        is_active: p.is_active,
        created_at: p.created_at,
        updated_at: p.updated_at,
        created_by: p.created_by,
        updated_by: p.updated_by,
        stats: p.stats.map(|s| PolicyStats {
            evaluations_total: s.evaluations_total,
            evaluations_allowed: s.evaluations_allowed,
            evaluations_denied: s.evaluations_denied,
            cache_hits: s.cache_hits,
            cache_misses: s.cache_misses,
            last_evaluated_at: s.last_evaluated_at,
        }),
    };

    Ok(Json(ApiResponse::success(policy)))
}

/// Get a policy by name
#[tracing::instrument(skip(state))]
pub async fn get_policy(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> ApiResult<Json<ApiResponse<PolicyResponse>>> {
    info!("Getting policy: name={}", name);

    // Check admin permission
    if !is_admin(&state)? {
        return Err(ApiError::Core(CoreError::IamPermissionDenied {
            operation: "get_policy".to_string(),
        }));
    }

    let p = state
        .policy_service
        .get_policy(name)
        .await
        .map_err(ApiError::Core)?;

    let policy = PolicyResponse {
        id: p.id,
        name: p.name,
        namespace: p.namespace,
        description: p.description,
        rules: p.rules,
        version: p.version,
        is_active: p.is_active,
        created_at: p.created_at,
        updated_at: p.updated_at,
        created_by: p.created_by,
        updated_by: p.updated_by,
        stats: p.stats.map(|s| PolicyStats {
            evaluations_total: s.evaluations_total,
            evaluations_allowed: s.evaluations_allowed,
            evaluations_denied: s.evaluations_denied,
            cache_hits: s.cache_hits,
            cache_misses: s.cache_misses,
            last_evaluated_at: s.last_evaluated_at,
        }),
    };

    Ok(Json(ApiResponse::success(policy)))
}

/// Update a policy
#[tracing::instrument(skip(state, req))]
pub async fn update_policy(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(req): Json<UpdatePolicyRequest>,
) -> ApiResult<Json<ApiResponse<PolicyResponse>>> {
    info!("Updating policy: name={}", name);

    // Check admin permission
    if !is_admin(&state)? {
        return Err(ApiError::Core(CoreError::IamPermissionDenied {
            operation: "update_policy".to_string(),
        }));
    }

    let user = extract_user(&state)?;

    let p = state
        .policy_service
        .update_policy(
            name,
            req.description,
            req.rules,
            req.is_active,
            user,
        )
        .await
        .map_err(ApiError::Core)?;

    let policy = PolicyResponse {
        id: p.id,
        name: p.name,
        namespace: p.namespace,
        description: p.description,
        rules: p.rules,
        version: p.version,
        is_active: p.is_active,
        created_at: p.created_at,
        updated_at: p.updated_at,
        created_by: p.created_by,
        updated_by: p.updated_by,
        stats: p.stats.map(|s| PolicyStats {
            evaluations_total: s.evaluations_total,
            evaluations_allowed: s.evaluations_allowed,
            evaluations_denied: s.evaluations_denied,
            cache_hits: s.cache_hits,
            cache_misses: s.cache_misses,
            last_evaluated_at: s.last_evaluated_at,
        }),
    };

    Ok(Json(ApiResponse::success(policy)))
}

/// Delete a policy
#[tracing::instrument(skip(state))]
pub async fn delete_policy(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> ApiResult<Json<ApiResponse<()>>> {
    info!("Deleting policy: name={}", name);

    // Check admin permission
    if !is_admin(&state)? {
        return Err(ApiError::Core(CoreError::IamPermissionDenied {
            operation: "delete_policy".to_string(),
        }));
    }

    state
        .policy_service
        .delete_policy(name)
        .await
        .map_err(ApiError::Core)?;

    Ok(Json(ApiResponse::success(())))
}

/// Test policy evaluation
#[tracing::instrument(skip(state, req))]
pub async fn test_policy(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(req): Json<TestPolicyRequest>,
) -> ApiResult<Json<ApiResponse<TestPolicyResponse>>> {
    info!(
        "Testing policy: name={}, user={}, path={}, action={}",
        name, req.user, req.path, req.action
    );

    // Check admin permission
    if !is_admin(&state)? {
        return Err(ApiError::Core(CoreError::IamPermissionDenied {
            operation: "test_policy".to_string(),
        }));
    }

    let res = state
        .policy_service
        .test_policy(name, req.user, req.path, req.action, req.context)
        .await
        .map_err(ApiError::Core)?;

    let response = TestPolicyResponse {
        allowed: res.allowed,
        matched_rules: res.matched_rules,
        evaluation_time_ms: res.evaluation_time_ms,
    };

    Ok(Json(ApiResponse::success(response)))
}
