//! Namespace Management API Handlers
//!
//! Provides REST API endpoints for namespace administration with full SIMKARI integration.
//! Implements hierarchical access control based on JWT claims from Authenc.
//!
//! Endpoints:
//! - GET /v1/sys/namespaces - List all accessible namespaces
//! - POST /v1/sys/namespaces - Create namespace with parent validation
//! - GET /v1/sys/namespaces/{id} - Get namespace details with hierarchy
//! - PUT /v1/sys/namespaces/{id} - Update quotas, policies
//! - DELETE /v1/sys/namespaces/{id} - Delete namespace (if no children)
//! - GET /v1/sys/namespaces/{id}/stats - Usage statistics, quota enforcement

use axum::{
    Router,
    extract::{Path, Query, State},
    response::Json,
    routing::{get, post},
};
use metrics::{counter, gauge};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use secreton_core::{
    error::CoreError,
    namespace::{
        AdminLevel, JwtClaims, Namespace, NamespaceAccessControl, NamespaceHierarchy,
        NamespaceQuotas, NamespaceType, QuotaUsage,
    },
};

use crate::{
    ApiError, ApiResponse, ApiResult,
    handlers::AppState,
    models::{PaginatedResponse, PaginationQuery},
};

/// Create namespace routes
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/namespaces", post(create_namespace))
        .route("/namespaces", get(list_namespaces))
        .route(
            "/namespaces/{id}",
            get(get_namespace)
                .put(update_namespace)
                .delete(delete_namespace),
        )
        .route("/namespaces/{id}/stats", get(get_namespace_stats))
}

// ============================================================================
// Request/Response DTOs
// ============================================================================

/// Request to create a new namespace
#[derive(Debug, Deserialize)]
pub struct CreateNamespaceRequest {
    /// Namespace ID (e.g., "wilayah-sumut", "satker-kja001")
    pub id: String,

    /// Display name (e.g., "Kejaksaan Negeri Medan")
    pub name: String,

    /// Parent namespace ID (required for wilayah and satker)
    pub parent: Option<String>,

    /// Namespace type
    pub namespace_type: NamespaceType,

    /// Initial policies
    #[serde(default)]
    pub policies: Vec<String>,

    /// Resource quotas
    pub quotas: Option<NamespaceQuotas>,

    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Request to update namespace
#[derive(Debug, Deserialize)]
pub struct UpdateNamespaceRequest {
    /// Display name
    pub name: Option<String>,

    /// Policies to add/remove
    pub policies: Option<Vec<String>>,

    /// Updated quotas
    pub quotas: Option<NamespaceQuotas>,

    /// Updated metadata
    pub metadata: Option<HashMap<String, String>>,

    /// Active status
    pub is_active: Option<bool>,
}

/// Namespace response DTO
#[derive(Debug, Serialize)]
pub struct NamespaceResponse {
    pub id: String,
    pub path: String,
    pub parent: Option<String>,
    pub name: String,
    pub namespace_type: NamespaceType,
    pub policies: Vec<String>,
    pub quotas: NamespaceQuotas,
    pub metadata: HashMap<String, String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub created_by: String,
    pub is_active: bool,
    pub children_count: usize,
    pub ancestors: Vec<String>,
}

/// Namespace statistics response
#[derive(Debug, Serialize)]
pub struct NamespaceStatsResponse {
    pub namespace_id: String,
    pub quota_usage: QuotaUsage,
    pub quota_limits: NamespaceQuotas,
    pub usage_percentage: f64,
    pub is_quota_exceeded: bool,
    pub children_count: usize,
    pub total_descendants: usize,
    pub active_leases: u64,
    pub active_policies: usize,
}

/// List namespaces query parameters
#[derive(Debug, Deserialize)]
pub struct ListNamespacesQuery {
    /// Filter by namespace type
    pub namespace_type: Option<NamespaceType>,

    /// Filter by parent
    pub parent: Option<String>,

    /// Filter by active status
    pub is_active: Option<bool>,

    /// Search by name
    pub search: Option<String>,

    #[serde(flatten)]
    pub pagination: PaginationQuery,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Extract JWT claims from request (placeholder - integrate with actual auth middleware)
fn extract_jwt_claims(state: &AppState) -> Result<JwtClaims, CoreError> {
    // TODO: Extract from actual JWT token in Authorization header
    // For now, return a placeholder for testing
    // In production, this should be extracted from the validated JWT token
    // by the auth middleware and passed through request extensions

    // Placeholder implementation
    Ok(JwtClaims {
        sub: "admin".to_string(),
        name: "Admin User".to_string(),
        email: "admin@kejaksaan.go.id".to_string(),
        satker_code: None,
        wilayah_code: None,
        admin_level: AdminLevel::Pusat,
        roles: vec!["admin".to_string()],
        permissions: vec!["*".to_string()],
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
        iat: chrono::Utc::now().timestamp(),
        iss: "authenc".to_string(),
        metadata: HashMap::new(),
    })
}

/// Convert Namespace to NamespaceResponse
fn namespace_to_response(
    namespace: &Namespace,
    hierarchy: &NamespaceHierarchy,
) -> NamespaceResponse {
    let children_count = hierarchy.get_descendants(&namespace.id).len();
    let ancestors = hierarchy
        .get_ancestors(&namespace.id)
        .iter()
        .map(|ns| ns.id.clone())
        .collect();

    NamespaceResponse {
        id: namespace.id.clone(),
        path: namespace.path.clone(),
        parent: namespace.parent.clone(),
        name: namespace.name.clone(),
        namespace_type: namespace.namespace_type,
        policies: namespace.policies.clone(),
        quotas: namespace.quotas.clone(),
        metadata: namespace.metadata.clone(),
        created_at: namespace.created_at,
        updated_at: namespace.updated_at,
        created_by: namespace.created_by.clone(),
        is_active: namespace.is_active,
        children_count,
        ancestors,
    }
}

/// Validate namespace ID format
fn validate_namespace_id(id: &str, namespace_type: NamespaceType) -> Result<(), CoreError> {
    match namespace_type {
        NamespaceType::Pusat => {
            if id != "pusat" {
                return Err(CoreError::validation("Pusat namespace ID must be 'pusat'"));
            }
        }
        NamespaceType::Wilayah => {
            if !id.starts_with("wilayah-") {
                return Err(CoreError::validation(
                    "Wilayah namespace ID must start with 'wilayah-'",
                ));
            }
            // Validate format: wilayah-{code}
            let parts: Vec<&str> = id.split('-').collect();
            if parts.len() != 2 || parts[1].is_empty() {
                return Err(CoreError::validation(
                    "Wilayah namespace ID format: wilayah-{code}",
                ));
            }
        }
        NamespaceType::Satker => {
            if !id.starts_with("satker-") {
                return Err(CoreError::validation(
                    "Satker namespace ID must start with 'satker-'",
                ));
            }
            // Validate format: satker-{code}
            let parts: Vec<&str> = id.split('-').collect();
            if parts.len() != 2 || parts[1].is_empty() {
                return Err(CoreError::validation(
                    "Satker namespace ID format: satker-{code}",
                ));
            }
        }
    }

    // Check for invalid characters
    if !id
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(CoreError::validation(
            "Namespace ID can only contain alphanumeric characters, hyphens, and underscores",
        ));
    }

    // Check for reserved names
    let reserved_names = ["system", "admin", "root", "default", "internal"];
    if reserved_names.contains(&id) {
        return Err(CoreError::validation(format!(
            "Namespace ID '{}' is reserved",
            id
        )));
    }

    Ok(())
}

// ============================================================================
// API Handlers
// ============================================================================

/// List all accessible namespaces based on user's satker/wilayah
/// GET /v1/sys/namespaces
/// Authorization:
/// - Pusat: Can list all namespaces
/// - Eselon I: Can list all namespaces
/// - Wilayah: Can list their wilayah and child satkers
/// - Satker: Can list only their own satker
pub async fn list_namespaces(
    State(state): State<AppState>,
    Query(query): Query<ListNamespacesQuery>,
) -> ApiResult<Json<ApiResponse<PaginatedResponse<NamespaceResponse>>>> {
    // Extract JWT claims
    let claims = extract_jwt_claims(&state)?;

    // Get namespace hierarchy from state
    let hierarchy = state.namespace.hierarchy();

    // Get namespace access control
    let access_control = NamespaceAccessControl::new(hierarchy.clone());

    // Get accessible namespaces for this user
    let accessible_ids = access_control.get_accessible_namespaces(&claims);

    // Get all namespaces
    let mut namespaces: Vec<&Namespace> = hierarchy
        .list_all()
        .into_iter()
        .filter(|ns| accessible_ids.contains(&ns.id))
        .collect();

    // Apply filters
    if let Some(namespace_type) = query.namespace_type {
        namespaces.retain(|ns| ns.namespace_type == namespace_type);
    }

    if let Some(parent) = &query.parent {
        namespaces.retain(|ns| ns.parent.as_ref() == Some(parent));
    }

    if let Some(is_active) = query.is_active {
        namespaces.retain(|ns| ns.is_active == is_active);
    }

    if let Some(search) = &query.search {
        let search_lower = search.to_lowercase();
        namespaces.retain(|ns| {
            ns.name.to_lowercase().contains(&search_lower)
                || ns.id.to_lowercase().contains(&search_lower)
        });
    }

    // Calculate pagination
    let total = namespaces.len() as u64;
    let offset = query.pagination.offset as usize;
    let limit = query.pagination.limit as usize;

    // Apply pagination
    let paginated_namespaces: Vec<NamespaceResponse> = namespaces
        .iter()
        .skip(offset)
        .take(limit)
        .map(|ns| namespace_to_response(ns, &hierarchy))
        .collect();

    let response = PaginatedResponse::new(
        paginated_namespaces,
        total,
        query.pagination.limit,
        query.pagination.offset,
    );

    // Audit log
    tracing::info!(
        user_id = %claims.sub,
        admin_level = ?claims.admin_level,
        total_accessible = accessible_ids.len(),
        filtered_count = total,
        "Listed namespaces"
    );

    // Metrics
    counter!("namespace_operations_total", "operation" => "list").increment(1);

    Ok(Json(ApiResponse::success(response)))
}

/// Create a new namespace with parent validation
/// POST /v1/sys/namespaces
/// Authorization:
/// - Pusat: Can create any namespace
/// - Eselon I: Can create any namespace
/// - Wilayah: Can create satkers under their wilayah
/// - Satker: Cannot create namespaces
pub async fn create_namespace(
    State(state): State<AppState>,
    Json(request): Json<CreateNamespaceRequest>,
) -> ApiResult<Json<ApiResponse<NamespaceResponse>>> {
    // Extract JWT claims
    let claims = extract_jwt_claims(&state)?;

    // Validate namespace ID format
    validate_namespace_id(&request.id, request.namespace_type)?;

    // Get mutable access to hierarchy
    let mut hierarchy = state.namespace.hierarchy().clone();

    // Authorization check based on admin level
    match claims.admin_level {
        AdminLevel::Pusat | AdminLevel::EselonI => {
            // Can create any namespace
        }
        AdminLevel::Wilayah => {
            // Can only create satkers under their wilayah
            if request.namespace_type != NamespaceType::Satker {
                return Err(ApiError::Forbidden);
            }

            // Verify parent is their wilayah
            if let Some(wilayah_code) = &claims.wilayah_code {
                let expected_parent = format!("wilayah-{}", wilayah_code.to_lowercase());
                if request.parent.as_ref() != Some(&expected_parent) {
                    return Err(ApiError::Forbidden);
                }
            } else {
                return Err(ApiError::Forbidden);
            }
        }
        AdminLevel::Satker => {
            return Err(ApiError::Forbidden);
        }
    }

    // Create namespace based on type
    let namespace = match request.namespace_type {
        NamespaceType::Pusat => {
            return Err(ApiError::BadRequest {
                message: "Cannot create Pusat namespace (already exists)".to_string(),
            });
        }
        NamespaceType::Wilayah => {
            hierarchy.add_wilayah(request.id.clone(), request.name.clone(), claims.sub.clone())?
        }
        NamespaceType::Satker => {
            let parent = request
                .parent
                .ok_or_else(|| CoreError::validation("Satker namespace requires parent wilayah"))?;

            hierarchy.add_satker(
                request.id.clone(),
                request.name.clone(),
                parent,
                claims.sub.clone(),
            )?
        }
    };

    // Update namespace with additional properties
    if let Some(ns) = hierarchy.get_namespace_mut(&namespace.id) {
        // Add policies
        for policy in request.policies {
            ns.add_policy(policy);
        }

        // Update quotas if provided
        if let Some(quotas) = request.quotas {
            ns.update_quotas(quotas);
        }

        // Add metadata
        for (key, value) in request.metadata {
            ns.metadata.insert(key, value);
        }
    }

    // Update state with new hierarchy
    state.namespace.update_hierarchy(hierarchy.clone());

    // TODO: Persist to database

    let response = namespace_to_response(&namespace, &hierarchy);

    // Audit log
    tracing::info!(
        user_id = %claims.sub,
        namespace_id = %namespace.id,
        namespace_type = ?namespace.namespace_type,
        "Created namespace"
    );

    // Metrics
    counter!("namespace_operations_total", "operation" => "create").increment(1);

    Ok(Json(ApiResponse::success(response)))
}

/// Get namespace details with hierarchy
/// GET /v1/sys/namespaces/{id}
/// Authorization: User must have access to the namespace
pub async fn get_namespace(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<ApiResponse<NamespaceResponse>>> {
    // Extract JWT claims
    let claims = extract_jwt_claims(&state)?;

    // Get namespace hierarchy
    let hierarchy = state.namespace.hierarchy();

    // Get namespace access control
    let access_control = NamespaceAccessControl::new(hierarchy.clone());

    // Check access
    if !access_control.check_access(&claims, &id)? {
        return Err(ApiError::Authorization {
            message: format!("Access denied to namespace: {}", id),
        });
    }

    // Get namespace
    let namespace = hierarchy
        .get_namespace(&id)
        .ok_or_else(|| CoreError::not_found(format!("namespace: {}", id)))?;

    let response = namespace_to_response(namespace, &hierarchy);

    // Audit log
    tracing::info!(
        user_id = %claims.sub,
        namespace_id = %id,
        "Retrieved namespace details"
    );

    // Metrics
    counter!("namespace_operations_total", "operation" => "get").increment(1);

    Ok(Json(ApiResponse::success(response)))
}

/// Update namespace quotas and policies
/// PUT /v1/sys/namespaces/{id}
/// Authorization:
/// - Pusat: Can update any namespace
/// - Eselon I: Can update any namespace
/// - Wilayah: Can update their wilayah and child satkers
/// - Satker: Cannot update namespaces
pub async fn update_namespace(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateNamespaceRequest>,
) -> ApiResult<Json<ApiResponse<NamespaceResponse>>> {
    // Extract JWT claims
    let claims = extract_jwt_claims(&state)?;

    // Get mutable hierarchy
    let mut hierarchy = state.namespace.hierarchy().clone();

    // Get namespace access control
    let access_control = NamespaceAccessControl::new(hierarchy.clone());

    // Check access
    if !access_control.check_access(&claims, &id)? {
        return Err(ApiError::Authorization {
            message: format!("Access denied to namespace: {}", id),
        });
    }

    // Additional authorization check for updates
    match claims.admin_level {
        AdminLevel::Pusat | AdminLevel::EselonI => {
            // Can update any accessible namespace
        }
        AdminLevel::Wilayah => {
            // Can update their wilayah and child satkers
            // Already checked by access_control.check_access
        }
        AdminLevel::Satker => {
            return Err(ApiError::Authorization {
                message: "Satker admin cannot update namespaces".to_string(),
            });
        }
    }

    // Get mutable namespace
    let namespace = hierarchy
        .get_namespace_mut(&id)
        .ok_or_else(|| CoreError::not_found(format!("namespace: {}", id)))?;

    // Update fields
    if let Some(name) = request.name {
        namespace.name = name;
        namespace.updated_at = chrono::Utc::now();
    }

    if let Some(policies) = request.policies {
        namespace.policies = policies;
        namespace.updated_at = chrono::Utc::now();
    }

    if let Some(quotas) = request.quotas {
        namespace.update_quotas(quotas);
    }

    if let Some(metadata) = request.metadata {
        namespace.metadata = metadata;
        namespace.updated_at = chrono::Utc::now();
    }

    if let Some(is_active) = request.is_active {
        namespace.is_active = is_active;
        namespace.updated_at = chrono::Utc::now();
    }

    // Clone namespace ID before dropping mutable borrow
    let namespace_id_clone = namespace.id.clone();

    // Update state
    state.namespace.update_hierarchy(hierarchy.clone());

    // TODO: Persist to database

    // Get immutable reference after mutable borrow is dropped
    let namespace = hierarchy
        .get_namespace(&namespace_id_clone)
        .ok_or_else(|| CoreError::not_found(format!("namespace: {}", namespace_id_clone)))?;
    let response = namespace_to_response(namespace, &hierarchy);

    // Audit log
    tracing::info!(
        user_id = %claims.sub,
        namespace_id = %id,
        "Updated namespace"
    );

    // Metrics
    counter!("namespace_operations_total", "operation" => "update").increment(1);

    Ok(Json(ApiResponse::success(response)))
}

/// Delete namespace (if no children)
/// DELETE /v1/sys/namespaces/{id}
/// Authorization:
/// - Pusat: Can delete any namespace (except pusat itself)
/// - Eselon I: Can delete any namespace (except pusat)
/// - Wilayah: Can delete satkers under their wilayah
/// - Satker: Cannot delete namespaces
pub async fn delete_namespace(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<ApiResponse<()>>> {
    // Extract JWT claims
    let claims = extract_jwt_claims(&state)?;

    // Cannot delete pusat namespace
    if id == "pusat" {
        return Err(ApiError::BadRequest {
            message: "Cannot delete pusat namespace".to_string(),
        });
    }

    // Get mutable hierarchy
    let mut hierarchy = state.namespace.hierarchy().clone();

    // Get namespace access control
    let access_control = NamespaceAccessControl::new(hierarchy.clone());

    // Check access
    if !access_control.check_access(&claims, &id)? {
        return Err(ApiError::Authorization {
            message: format!("Access denied to namespace: {}", id),
        });
    }

    // Additional authorization check for deletion
    match claims.admin_level {
        AdminLevel::Pusat | AdminLevel::EselonI => {
            // Can delete any accessible namespace
        }
        AdminLevel::Wilayah => {
            // Can only delete satkers under their wilayah
            let namespace = hierarchy
                .get_namespace(&id)
                .ok_or_else(|| CoreError::not_found(format!("namespace: {}", id)))?;

            if namespace.namespace_type != NamespaceType::Satker {
                return Err(ApiError::Authorization {
                    message: "Wilayah admin can only delete satker namespaces".to_string(),
                });
            }

            // Verify parent is their wilayah
            if let Some(wilayah_code) = &claims.wilayah_code {
                let expected_parent = format!("wilayah-{}", wilayah_code.to_lowercase());
                if namespace.parent.as_ref() != Some(&expected_parent) {
                    return Err(ApiError::Authorization {
                        message: "Wilayah admin can only delete satkers under their own wilayah"
                            .to_string(),
                    });
                }
            }
        }
        AdminLevel::Satker => {
            return Err(ApiError::Authorization {
                message: "Satker admin cannot delete namespaces".to_string(),
            });
        }
    }

    // Check if namespace has children
    let descendants = hierarchy.get_descendants(&id);
    if !descendants.is_empty() {
        return Err(ApiError::Conflict {
            resource: format!("namespace {} with {} children", id, descendants.len()),
        });
    }

    // Check if namespace has active resources (secrets, leases, etc.)
    // TODO: Query storage backend for active resources
    // For now, we'll allow deletion if no children

    // Remove namespace from hierarchy
    hierarchy.delete_namespace(&id)?;

    // Update state
    state.namespace.update_hierarchy(hierarchy);

    // TODO: Persist to database
    // TODO: Cascade delete related resources (policies, quotas, etc.)

    // Audit log
    tracing::info!(
        user_id = %claims.sub,
        namespace_id = %id,
        "Deleted namespace"
    );

    // Metrics
    counter!("namespace_operations_total", "operation" => "delete").increment(1);

    Ok(Json(ApiResponse::success(())))
}

/// Get namespace usage statistics and quota enforcement
/// GET /v1/sys/namespaces/{id}/stats
/// Authorization: User must have access to the namespace
pub async fn get_namespace_stats(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<ApiResponse<NamespaceStatsResponse>>> {
    // Extract JWT claims
    let claims = extract_jwt_claims(&state)?;

    // Get namespace hierarchy
    let hierarchy = state.namespace.hierarchy();

    // Get namespace access control
    let access_control = NamespaceAccessControl::new(hierarchy.clone());

    // Check access
    if !access_control.check_access(&claims, &id)? {
        return Err(ApiError::Authorization {
            message: format!("Access denied to namespace: {}", id),
        });
    }

    // Get namespace
    let namespace = hierarchy
        .get_namespace(&id)
        .ok_or_else(|| CoreError::not_found(format!("namespace: {}", id)))?;

    // Calculate statistics
    let children_count = hierarchy.get_descendants(&id).len();
    let total_descendants = hierarchy.count_all_descendants(&id);

    // Get quota usage from namespace
    let quota_usage = namespace.quotas.current_usage.clone();
    let quota_limits = NamespaceQuotas {
        max_secrets: namespace.quotas.max_secrets,
        max_storage_bytes: namespace.quotas.max_storage_bytes,
        max_leases: namespace.quotas.max_leases,
        max_policies: namespace.quotas.max_policies,
        current_usage: QuotaUsage::default(), // Don't include usage in limits
    };

    // Calculate usage percentage
    let usage_percentage = if let Some(max_secrets) = quota_limits.max_secrets {
        if max_secrets > 0 {
            (quota_usage.secrets_count as f64 / max_secrets as f64) * 100.0
        } else {
            0.0
        }
    } else {
        0.0
    };

    // Check if quota exceeded
    let is_quota_exceeded = quota_limits
        .max_secrets
        .map(|max| quota_usage.secrets_count >= max)
        .unwrap_or(false)
        || quota_limits
            .max_storage_bytes
            .map(|max| quota_usage.storage_bytes >= max)
            .unwrap_or(false)
        || quota_limits
            .max_leases
            .map(|max| quota_usage.leases_count >= max)
            .unwrap_or(false);

    // TODO: Query storage backend for active leases count
    let active_leases = 0u64;

    // Get active policies count
    let active_policies = namespace.policies.len();

    let response = NamespaceStatsResponse {
        namespace_id: namespace.id.clone(),
        quota_usage,
        quota_limits,
        usage_percentage,
        is_quota_exceeded,
        children_count,
        total_descendants,
        active_leases,
        active_policies,
    };

    // Audit log
    tracing::debug!(
        user_id = %claims.sub,
        namespace_id = %id,
        usage_percentage = %usage_percentage,
        "Retrieved namespace statistics"
    );

    // Metrics
    counter!("namespace_operations_total", "operation" => "stats").increment(1);
    gauge!("namespace_quota_usage_percentage", "namespace_id" => id.clone()).set(usage_percentage);

    Ok(Json(ApiResponse::success(response)))
}
