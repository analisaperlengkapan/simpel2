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
    Extension, Router,
    extract::{Path, Query, State},
    response::Json,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::info;

use secreton_core::{error::CoreError, models::PolicyRule, namespace::AdminLevel};

use crate::{
    ApiError, ApiResponse, ApiResult, PaginationQuery, handlers::AppState,
    middleware::RequestContext, models::PaginatedResponse,
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

    /// Namespace (defaults to "default")
    #[serde(default = "default_namespace")]
    pub namespace: String,
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

/// Namespace query parameters for item operations
#[derive(Debug, Deserialize)]
pub struct NamespaceQuery {
    /// Namespace
    #[serde(default = "default_namespace")]
    pub namespace: String,
}

fn default_namespace() -> String {
    "default".to_string()
}

// ============================================================================
// Helper Functions
// ============================================================================

/// The caller's identity, for `created_by` / `updated_by` attribution.
///
/// This used to return the literal `"system"` regardless of who called, so every
/// policy write was attributed to a principal that never made it. The real
/// identity is put in the request extensions by
/// [`crate::middleware::auth_middleware`]; read it.
fn extract_user(context: &RequestContext) -> Result<String, ApiError> {
    context
        .user_email
        .clone()
        .or_else(|| context.user_id.clone())
        .ok_or(ApiError::Unauthorized)
}

/// Whether the caller may administer policies.
///
/// This used to be `Ok(true)`. Every `if !is_admin(&state)?` guard below was
/// therefore a branch on a constant, and any authenticated principal — of any
/// role — could create, update or delete the policies that are supposed to
/// govern access to secrets.
///
/// Root tokens minted by `/v1/sys/init` carry `roles: ["root", "admin"]`
/// (`handlers/seal.rs:774`), so bootstrap keeps working.
fn is_admin(context: &RequestContext) -> bool {
    if context
        .user_roles
        .iter()
        .any(|r| matches!(r.to_lowercase().as_str(), "admin" | "root" | "superuser"))
    {
        return true;
    }

    context
        .jwt_claims
        .as_ref()
        .is_some_and(|claims| claims.admin_level == AdminLevel::Pusat)
}

/// Reject a non-admin caller for `operation`.
fn require_admin(context: &RequestContext, operation: &str) -> Result<(), ApiError> {
    if is_admin(context) {
        return Ok(());
    }
    Err(ApiError::Core(CoreError::IamPermissionDenied {
        operation: operation.to_string(),
    }))
}

// ============================================================================
// API Handlers
// ============================================================================

/// List all policies with pagination
#[tracing::instrument(skip(state, context))]
pub async fn list_policies(
    State(state): State<AppState>,
    Extension(context): Extension<RequestContext>,
    Query(query): Query<ListPoliciesQuery>,
) -> ApiResult<Json<PaginatedResponse<PolicyResponse>>> {
    info!(
        "Listing policies with filters: namespace={:?}, is_active={:?}, search={:?}",
        query.namespace, query.is_active, query.search
    );

    require_admin(&context, "list_policies")?;

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

    info!(
        "Found {} policies (total: {})",
        response_policies.len(),
        total
    );

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
    Extension(context): Extension<RequestContext>,
    Json(req): Json<CreatePolicyRequest>,
) -> ApiResult<Json<ApiResponse<PolicyResponse>>> {
    info!(
        "Creating policy: name={}, namespace={}",
        name, req.namespace
    );

    require_admin(&context, "create_policy")?;

    let user = extract_user(&context)?;

    let p = state
        .policy_service
        .create_policy(name, req.namespace, req.description, req.rules, user)
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
#[tracing::instrument(skip(state, context))]
pub async fn get_policy(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Extension(context): Extension<RequestContext>,
    Query(query): Query<NamespaceQuery>,
) -> ApiResult<Json<ApiResponse<PolicyResponse>>> {
    info!(
        "Getting policy: name={}, namespace={}",
        name, query.namespace
    );

    require_admin(&context, "get_policy")?;

    let p = state
        .policy_service
        .get_policy(name, query.namespace)
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
    Extension(context): Extension<RequestContext>,
    Query(query): Query<NamespaceQuery>,
    Json(req): Json<UpdatePolicyRequest>,
) -> ApiResult<Json<ApiResponse<PolicyResponse>>> {
    info!(
        "Updating policy: name={}, namespace={}",
        name, query.namespace
    );

    require_admin(&context, "update_policy")?;

    let user = extract_user(&context)?;

    let p = state
        .policy_service
        .update_policy(
            name,
            query.namespace,
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
#[tracing::instrument(skip(state, context))]
pub async fn delete_policy(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Extension(context): Extension<RequestContext>,
    Query(query): Query<NamespaceQuery>,
) -> ApiResult<Json<ApiResponse<()>>> {
    info!(
        "Deleting policy: name={}, namespace={}",
        name, query.namespace
    );

    require_admin(&context, "delete_policy")?;

    state
        .policy_service
        .delete_policy(name, query.namespace)
        .await
        .map_err(ApiError::Core)?;

    Ok(Json(ApiResponse::success(())))
}

/// Test policy evaluation
#[tracing::instrument(skip(state, req))]
pub async fn test_policy(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Extension(context): Extension<RequestContext>,
    Json(req): Json<TestPolicyRequest>,
) -> ApiResult<Json<ApiResponse<TestPolicyResponse>>> {
    info!(
        "Testing policy: name={}, namespace={}, user={}, path={}, action={}",
        name, req.namespace, req.user, req.path, req.action
    );

    require_admin(&context, "test_policy")?;

    let res = state
        .policy_service
        .test_policy(
            name,
            req.namespace,
            req.user,
            req.path,
            req.action,
            req.context,
        )
        .await
        .map_err(ApiError::Core)?;

    let response = TestPolicyResponse {
        allowed: res.allowed,
        matched_rules: res.matched_rules,
        evaluation_time_ms: res.evaluation_time_ms,
    };

    Ok(Json(ApiResponse::success(response)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    fn ctx(roles: &[&str], admin_level: Option<AdminLevel>) -> RequestContext {
        RequestContext {
            request_id: "test".to_string(),
            user_id: Some("11111111-1111-1111-1111-111111111111".to_string()),
            user_email: Some("operator@kejaksaan.go.id".to_string()),
            user_roles: roles.iter().map(|r| r.to_string()).collect(),
            user_permissions: vec![],
            start_time: Instant::now(),
            jwt_claims: admin_level.map(|level| secreton_core::namespace::JwtClaims {
                sub: "11111111-1111-1111-1111-111111111111".to_string(),
                name: "Operator".to_string(),
                email: "operator@kejaksaan.go.id".to_string(),
                satker_code: Some("KEJARI-01".to_string()),
                wilayah_code: None,
                admin_level: level,
                roles: roles.iter().map(|r| r.to_string()).collect(),
                permissions: vec![],
                exp: 0,
                iat: 0,
                iss: "secreton".to_string(),
                metadata: Default::default(),
            }),
            auth_token: Some("dummy".to_string()),
            client_ip: None,
            user_agent: None,
            policy_names: vec![],
        }
    }

    /// The regression this whole change exists for: `is_admin` returned a
    /// constant `true`, so an ordinary authenticated user could rewrite the
    /// policies that gate every secret.
    #[test]
    fn an_ordinary_user_is_not_an_admin() {
        assert!(!is_admin(&ctx(
            &["operator_satker"],
            Some(AdminLevel::Satker)
        )));
        assert!(!is_admin(&ctx(&[], None)));
        assert!(!is_admin(&ctx(
            &["validator_wilayah"],
            Some(AdminLevel::Wilayah)
        )));
    }

    #[test]
    fn admin_root_and_pusat_are_admins() {
        assert!(is_admin(&ctx(&["admin"], None)));
        // Root tokens from /v1/sys/init carry both — bootstrap must keep working.
        assert!(is_admin(&ctx(&["root", "admin"], None)));
        assert!(
            is_admin(&ctx(&["ADMIN"], None)),
            "role match is case-insensitive"
        );
        assert!(is_admin(&ctx(
            &["operator_satker"],
            Some(AdminLevel::Pusat)
        )));
    }

    #[test]
    fn require_admin_names_the_operation_it_refused() {
        let err = require_admin(&ctx(&["operator_satker"], None), "delete_policy").unwrap_err();
        match err {
            ApiError::Core(CoreError::IamPermissionDenied { operation }) => {
                assert_eq!(operation, "delete_policy");
            }
            other => panic!("expected IamPermissionDenied, got {other:?}"),
        }
        assert!(require_admin(&ctx(&["admin"], None), "delete_policy").is_ok());
    }

    /// Attribution used to be the literal "system" for every caller.
    #[test]
    fn the_writer_is_the_caller_not_system() {
        let user = extract_user(&ctx(&["admin"], None)).unwrap();
        assert_eq!(user, "operator@kejaksaan.go.id");
        assert_ne!(user, "system");
    }
}
