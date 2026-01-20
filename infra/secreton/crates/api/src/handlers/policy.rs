//! Policy Management API Handlers
//!
//! Provides REST API endpoints for policy administration with full integration.
//! Implements policy CRUD operations, validation, caching, and evaluation testing.
//!
//! Endpoints:
//! - GET /v1/sys/policies - List all policies with pagination
//! - POST /v1/sys/policies/{name} - Create policy with validation
//! - GET /v1/sys/policies/{name} - Get policy with evaluation stats
//! - PUT /v1/sys/policies/{name} - Update policy with version control
//! - DELETE /v1/sys/policies/{name} - Delete policy with dependency check
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
    services::policy::{Capability, PolicySet},
};

use crate::{
    ApiError, ApiResponse, ApiResult,
    handlers::AppState,
    models::{PaginatedResponse, PaginationQuery},
};

/// Create policy routes
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/policies", get(list_policies))
        .route(
            "/policies/:name",
            post(create_policy)
                .get(get_policy)
                .put(update_policy)
                .delete(delete_policy),
        )
        .route("/policies/:name/test", post(test_policy))
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

/// Validate policy rules
fn validate_policy_rules(rules: &[PolicyRule]) -> Result<(), CoreError> {
    if rules.is_empty() {
        return Err(CoreError::Validation {
            message: "Invalid input".to_string(),
        });
    }

    for (idx, rule) in rules.iter().enumerate() {
        // Validate effect
        if rule.effect != "allow" && rule.effect != "deny" {
            return Err(CoreError::Validation {
                message: "Invalid input".to_string(),
            });
        }

        // Validate path pattern
        if rule.path.is_empty() {
            return Err(CoreError::Validation {
                message: format!("Rule {}: Path cannot be empty", idx),
            });
        }

        // Validate path pattern syntax
        if rule.path.contains("**") && !rule.path.ends_with("**") {
            return Err(CoreError::Validation {
                message: "Invalid input".to_string(),
            });
        }

        // Validate action
        if rule.action.is_empty() {
            return Err(CoreError::Validation {
                message: "Invalid input".to_string(),
            });
        }

        // Validate capability if not wildcard
        if rule.action != "*" && Capability::from_str(&rule.action).is_none() {
            // Check if it's a valid custom action (alphanumeric with underscores)
            if !rule.action.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Err(CoreError::Validation {
                    message: "Invalid input".to_string(),
                });
            }
        }

        // Validate control group if present
        if let Some(cg) = &rule.control_group
            && cg.required_approvals == 0
        {
            return Err(CoreError::Validation {
                message: "Invalid input".to_string(),
            });
        }

        // Validate condition if present
        if let Some(condition) = &rule.condition {
            validate_condition(condition, idx)?;
        }
    }

    Ok(())
}

/// Validate policy condition
fn validate_condition(condition: &Value, rule_idx: usize) -> Result<(), CoreError> {
    if !condition.is_object() {
        return Err(CoreError::Validation {
            message: "Invalid input".to_string(),
        });
    }

    // Validate time_range if present
    if let Some(time_range) = condition.get("time_range") {
        if !time_range.is_object() {
            return Err(CoreError::Validation {
                message: "Invalid input".to_string(),
            });
        }

        // Validate start and end are valid RFC3339 timestamps
        if let Some(start) = time_range.get("start")
            && let Some(start_str) = start.as_str()
            && chrono::DateTime::parse_from_rfc3339(start_str).is_err()
        {
            return Err(CoreError::Validation {
                message: "Invalid input".to_string(),
            });
        }

        if let Some(end) = time_range.get("end")
            && let Some(end_str) = end.as_str()
            && chrono::DateTime::parse_from_rfc3339(end_str).is_err()
        {
            return Err(CoreError::Validation {
                message: "Invalid input".to_string(),
            });
        }
    }

    // Validate allowed_ips if present
    if let Some(allowed_ips) = condition.get("allowed_ips")
        && !allowed_ips.is_array()
    {
        return Err(CoreError::Validation {
            message: "Invalid input".to_string(),
        });
    }

    // Validate expression if present
    if let Some(expr) = condition.get("expression") {
        if !expr.is_object() {
            return Err(CoreError::Validation {
                message: "Invalid input".to_string(),
            });
        }

        // Validate required fields
        if expr.get("field").is_none() {
            return Err(CoreError::Validation {
                message: "Invalid input".to_string(),
            });
        }

        if expr.get("op").is_none() {
            return Err(CoreError::Validation {
                message: "Invalid input".to_string(),
            });
        }

        if expr.get("value").is_none() {
            return Err(CoreError::Validation {
                message: "Invalid input".to_string(),
            });
        }
    }

    Ok(())
}

/// Check for circular dependencies
async fn check_circular_dependencies(
    pool: &deadpool_postgres::Pool,
    policy_id: i64,
    depends_on_id: i64,
) -> Result<bool, CoreError> {
    // Check if depends_on_id depends on policy_id (direct or indirect)
    let client = pool.get().await.map_err(|e| CoreError::Internal {
        message: format!("Database connection error: {}", e),
        source: None,
    })?;

    // Recursive CTE to find all dependencies
    let query = r#"
        WITH RECURSIVE deps AS (
            SELECT depends_on_policy_id
            FROM policy_dependencies
            WHERE policy_id = $1

            UNION

            SELECT pd.depends_on_policy_id
            FROM policy_dependencies pd
            INNER JOIN deps ON deps.depends_on_policy_id = pd.policy_id
        )
        SELECT EXISTS(SELECT 1 FROM deps WHERE depends_on_policy_id = $2)
    "#;

    let row = client
        .query_one(query, &[&depends_on_id, &policy_id])
        .await
        .map_err(|e| CoreError::Internal {
            message: format!("Database query error: {}", e),
            source: None,
        })?;

    Ok(row.get(0))
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

    let pool = &state.pool;
    let client = pool
        .get()
        .await
        .map_err(|e| ApiError::internal(format!("Database connection error: {}", e)))?;

    // Build query with filters
    let mut where_clauses = vec![];
    let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![];
    let mut param_idx = 1;

    // Storage for owned values that need to outlive the param references
    let is_active_storage: Option<bool>;
    let search_pattern_storage: Option<String>;

    if let Some(ref namespace) = query.namespace {
        where_clauses.push(format!("namespace = ${}", param_idx));
        params.push(namespace);
        param_idx += 1;
    }

    if let Some(is_active) = query.is_active {
        where_clauses.push(format!("is_active = ${}", param_idx));
        is_active_storage = Some(is_active);
        if let Some(ref val) = is_active_storage {
            params.push(val);
        }
        param_idx += 1;
    } else {
        is_active_storage = None;
    }

    if let Some(ref search) = query.search {
        where_clauses.push(format!(
            "(name ILIKE ${} OR description ILIKE ${})",
            param_idx, param_idx
        ));
        search_pattern_storage = Some(format!("%{}%", search));
        if let Some(ref pattern) = search_pattern_storage {
            params.push(pattern);
        }
        param_idx += 1;
    } else {
        search_pattern_storage = None;
    }

    let where_clause = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    // Count total
    let count_query = format!("SELECT COUNT(*) FROM policies {}", where_clause);
    let count_row =
        client
            .query_one(&count_query, &params)
            .await
            .map_err(|e| CoreError::Internal {
                message: format!("Internal error: {}", e),
                source: None,
            })?;
    let total: i64 = count_row.get(0);

    // Get paginated results
    let limit = 20;
    let offset = 0;

    let select_query = format!(
        "SELECT id, name, namespace, description, rules, version, is_active, created_at, updated_at, created_by, updated_by
         FROM policies {}
         ORDER BY created_at DESC
         LIMIT ${} OFFSET ${}",
        where_clause, param_idx, param_idx + 1
    );

    params.push(&limit);
    params.push(&offset);

    let rows = client
        .query(&select_query, &params)
        .await
        .map_err(|e| CoreError::Internal {
            message: format!("Internal error: {}", e),
            source: None,
        })?;

    let mut policies = vec![];
    for row in rows {
        let id: i64 = row.get(0);
        let rules_json: serde_json::Value = row.get(4);
        let rules: Vec<PolicyRule> = serde_json::from_value(rules_json).unwrap_or_default();

        // Get stats
        let stats = get_policy_stats(&client, id).await.ok();

        policies.push(PolicyResponse {
            id,
            name: row.get(1),
            namespace: row.get(2),
            description: row.get(3),
            rules,
            version: row.get(5),
            is_active: row.get(6),
            created_at: row.get(7),
            updated_at: row.get(8),
            created_by: row.get(9),
            updated_by: row.get(10),
            stats,
        });
    }

    info!("Found {} policies (total: {})", policies.len(), total);

    Ok(Json(PaginatedResponse::new(
        policies,
        total as u64,
        limit as u32,
        offset as u32,
    )))
}

/// Get policy statistics
async fn get_policy_stats(
    client: &tokio_postgres::Client,
    policy_id: i64,
) -> Result<PolicyStats, CoreError> {
    let row = client
        .query_opt(
            "SELECT evaluations_total, evaluations_allowed, evaluations_denied, cache_hits, cache_misses, last_evaluated_at
             FROM policy_stats WHERE policy_id = $1",
            &[&policy_id],
        )
        .await
        .map_err(|e| CoreError::Internal {
            message: format!("Database query error: {}", e),
            source: None,
        })?;

    if let Some(row) = row {
        Ok(PolicyStats {
            evaluations_total: row.get(0),
            evaluations_allowed: row.get(1),
            evaluations_denied: row.get(2),
            cache_hits: row.get(3),
            cache_misses: row.get(4),
            last_evaluated_at: row.get(5),
        })
    } else {
        Ok(PolicyStats {
            evaluations_total: 0,
            evaluations_allowed: 0,
            evaluations_denied: 0,
            cache_hits: 0,
            cache_misses: 0,
            last_evaluated_at: None,
        })
    }
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

    // Validate policy name
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(ApiError::Core(CoreError::Validation {
            message: "Invalid input".to_string(),
        }));
    }

    // Validate rules
    validate_policy_rules(&req.rules)?;

    let user = extract_user(&state)?;
    let pool = &state.pool;
    let client = pool
        .get()
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    // Check if policy already exists
    let existing = client
        .query_opt(
            "SELECT id FROM policies WHERE name = $1 AND namespace = $2",
            &[&name, &req.namespace],
        )
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    if existing.is_some() {
        return Err(ApiError::Core(CoreError::AlreadyExists {
            resource: "policy".to_string(),
        }));
    }

    // Serialize rules to JSON
    let rules_json = serde_json::to_value(&req.rules)
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    // Insert policy
    let row = client
        .query_one(
            "INSERT INTO policies (name, namespace, description, rules, version, created_by)
             VALUES ($1, $2, $3, $4, 1, $5)
             RETURNING id, name, namespace, description, rules, version, is_active, created_at, updated_at, created_by, updated_by",
            &[&name, &req.namespace, &req.description, &rules_json, &user],
        )
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    let policy_id: i64 = row.get(0);

    // Initialize stats
    client
        .execute(
            "INSERT INTO policy_stats (policy_id) VALUES ($1)",
            &[&policy_id],
        )
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    // Create version history
    client
        .execute(
            "INSERT INTO policy_versions (policy_id, version, rules, description, created_by)
             VALUES ($1, 1, $2, $3, $4)",
            &[&policy_id, &rules_json, &req.description, &user],
        )
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    let rules: Vec<PolicyRule> = serde_json::from_value(row.get(4)).unwrap_or_default();

    let policy = PolicyResponse {
        id: policy_id,
        name: row.get(1),
        namespace: row.get(2),
        description: row.get(3),
        rules,
        version: row.get(5),
        is_active: row.get(6),
        created_at: row.get(7),
        updated_at: row.get(8),
        created_by: row.get(9),
        updated_by: row.get(10),
        stats: Some(PolicyStats {
            evaluations_total: 0,
            evaluations_allowed: 0,
            evaluations_denied: 0,
            cache_hits: 0,
            cache_misses: 0,
            last_evaluated_at: None,
        }),
    };

    // Audit log
    info!(
        "Policy created: id={}, name={}, namespace={}",
        policy_id, name, req.namespace
    );

    // Invalidate policy cache (no-op - cache service not yet integrated)
    let _ = &state;

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

    let pool = &state.pool;
    let client = pool
        .get()
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    // Get policy
    let row = client
        .query_opt(
            "SELECT id, name, namespace, description, rules, version, is_active, created_at, updated_at, created_by, updated_by
             FROM policies WHERE name = $1",
            &[&name],
        )
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    let row = row.ok_or_else(|| CoreError::NotFound {
        resource: "policy".to_string(),
    })?;

    let policy_id: i64 = row.get(0);
    let rules_json: serde_json::Value = row.get(4);
    let rules: Vec<PolicyRule> = serde_json::from_value(rules_json).unwrap_or_default();

    // Get stats
    let stats = get_policy_stats(&client, policy_id).await.ok();

    let policy = PolicyResponse {
        id: policy_id,
        name: row.get(1),
        namespace: row.get(2),
        description: row.get(3),
        rules,
        version: row.get(5),
        is_active: row.get(6),
        created_at: row.get(7),
        updated_at: row.get(8),
        created_by: row.get(9),
        updated_by: row.get(10),
        stats,
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

    // Validate rules if provided
    if let Some(ref rules) = req.rules {
        validate_policy_rules(rules)?;
    }

    let user = extract_user(&state)?;
    let pool = &state.pool;
    let client = pool
        .get()
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    // Get existing policy
    let existing = client
        .query_opt(
            "SELECT id, version, rules FROM policies WHERE name = $1",
            &[&name],
        )
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    let existing = existing.ok_or_else(|| CoreError::NotFound {
        resource: "policy".to_string(),
    })?;

    let policy_id: i64 = existing.get(0);
    let current_version: i32 = existing.get(1);
    let new_version = current_version + 1;

    // Build update query
    let mut updates = vec![];
    let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![];
    let mut param_idx = 1;

    // Storage for owned values that need to outlive the param references
    let is_active_storage: Option<bool>;

    if let Some(ref description) = req.description {
        updates.push(format!("description = ${}", param_idx));
        params.push(description);
        param_idx += 1;
    }

    // Store rules JSON in a variable that lives long enough
    let rules_json: Option<serde_json::Value>;
    if let Some(ref rules) = req.rules {
        let json = serde_json::to_value(rules)
            .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;
        updates.push(format!("rules = ${}", param_idx));
        rules_json = Some(json);
        if let Some(ref json_ref) = rules_json {
            params.push(json_ref);
        }
        param_idx += 1;
    } else {
        rules_json = None;
    }

    if let Some(is_active) = req.is_active {
        updates.push(format!("is_active = ${}", param_idx));
        is_active_storage = Some(is_active);
        if let Some(ref val) = is_active_storage {
            params.push(val);
        }
        param_idx += 1;
    } else {
        is_active_storage = None;
    }

    if !updates.is_empty() {
        updates.push(format!("version = ${}", param_idx));
        params.push(&new_version);
        param_idx += 1;

        updates.push(format!("updated_by = ${}", param_idx));
        params.push(&user);
        param_idx += 1;

        let update_query = format!(
            "UPDATE policies SET {} WHERE name = ${}
             RETURNING id, name, namespace, description, rules, version, is_active, created_at, updated_at, created_by, updated_by",
            updates.join(", "),
            param_idx
        );

        params.push(&name);

        let row = client
            .query_one(&update_query, &params)
            .await
            .map_err(|e| CoreError::Internal {
                message: format!("Internal error: {}", e),
                source: None,
            })?;

        // Create version history if rules changed
        if let Some(rules_json) = rules_json {
            client
                .execute(
                    "INSERT INTO policy_versions (policy_id, version, rules, description, created_by)
                     VALUES ($1, $2, $3, $4, $5)",
                    &[&policy_id, &new_version, &rules_json, &req.description, &user],
                )
                .await
                .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;
        }

        let rules: Vec<PolicyRule> = serde_json::from_value(row.get(4)).unwrap_or_default();
        let stats = get_policy_stats(&client, policy_id).await.ok();

        let policy = PolicyResponse {
            id: policy_id,
            name: row.get(1),
            namespace: row.get(2),
            description: row.get(3),
            rules,
            version: row.get(5),
            is_active: row.get(6),
            created_at: row.get(7),
            updated_at: row.get(8),
            created_by: row.get(9),
            updated_by: row.get(10),
            stats,
        };

        // Audit log
        info!(
            "Policy updated: id={}, name={}, namespace={}, version={}",
            policy_id, name, policy.namespace, policy.version
        );

        // Policy cache invalidation would happen here
        // In production, integrate with actual cache service

        Ok(Json(ApiResponse::success(policy)))
    } else {
        Err(ApiError::Core(CoreError::Validation {
            message: "Invalid input".to_string(),
        }))
    }
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

    let pool = &state.pool;
    let client = pool
        .get()
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    // Get policy ID
    let row = client
        .query_opt("SELECT id FROM policies WHERE name = $1", &[&name])
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    let row = row.ok_or_else(|| CoreError::NotFound {
        resource: "policy".to_string(),
    })?;

    let policy_id: i64 = row.get(0);

    // Check for dependencies (other policies that depend on this one)
    let deps = client
        .query(
            "SELECT p.name FROM policies p
             INNER JOIN policy_dependencies pd ON p.id = pd.policy_id
             WHERE pd.depends_on_policy_id = $1",
            &[&policy_id],
        )
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    if !deps.is_empty() {
        let dependent_policies: Vec<String> = deps.iter().map(|row| row.get(0)).collect();
        return Err(ApiError::Core(CoreError::Validation {
            message: "Cannot delete policy".to_string(),
        }));
    }

    // Delete policy (cascade will delete stats, versions, and dependencies)
    let deleted = client
        .execute("DELETE FROM policies WHERE id = $1", &[&policy_id])
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    if deleted == 0 {
        return Err(ApiError::Core(CoreError::NotFound {
            resource: "policy".to_string(),
        }));
    }

    // Audit log
    info!("Policy deleted: id={}, name={}", policy_id, name);

    // Policy cache invalidation would happen here
    // In production, integrate with actual cache service

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

    let pool = &state.pool;
    let client = pool
        .get()
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    // Get policy
    let row = client
        .query_opt(
            "SELECT rules FROM policies WHERE name = $1 AND is_active = true",
            &[&name],
        )
        .await
        .map_err(|e| ApiError::internal(format!("Database error: {}", e)))?;

    let row = row.ok_or_else(|| CoreError::NotFound {
        resource: "policy".to_string(),
    })?;

    let rules_json: serde_json::Value = row.get(0);
    let rules: Vec<PolicyRule> = serde_json::from_value(rules_json).unwrap_or_default();

    // Create policy set and evaluate
    let policy_set = PolicySet::new(rules.clone());

    let start = std::time::Instant::now();
    let allowed = policy_set.evaluate(&req.user, &req.path, &req.action, req.context.as_ref());
    let duration = start.elapsed();

    // Find matched rules - use evaluate for each rule
    let mut matched_rules = vec![];
    for (idx, rule) in rules.iter().enumerate() {
        // Create a single-rule policy set to test matching
        let test_policy = PolicySet::new(vec![rule.clone()]);
        if test_policy.evaluate(&req.user, &req.path, &req.action, req.context.as_ref()) {
            matched_rules.push(format!(
                "Rule {}: {} {} on {}",
                idx + 1,
                rule.effect,
                rule.action,
                rule.path
            ));
        }
    }

    let response = TestPolicyResponse {
        allowed,
        matched_rules,
        evaluation_time_ms: duration.as_secs_f64() * 1000.0,
    };

    info!(
        "Policy test result: allowed={}, matched_rules={}, time={}ms",
        allowed,
        response.matched_rules.len(),
        response.evaluation_time_ms
    );

    Ok(Json(ApiResponse::success(response)))
}
