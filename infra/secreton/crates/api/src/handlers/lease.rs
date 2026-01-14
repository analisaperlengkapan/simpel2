//! Lease Management API Handlers
//!
//! Provides REST endpoints for lease operations including renewal,
//! revocation, lookup, and listing with full integration for namespace
//! isolation, authorization, audit logging, and monitoring.

use axum::{
    Router,
    async_trait,
    extract::{FromRequestParts, Path, Query, State},
    http::request::Parts,
    response::Json,
    routing::{get, post},
};

use crate::middleware::RequestContext;
use secreton_core::namespace::AdminLevel;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{ApiError, ApiResponse, ApiResult, handlers::AppState, models::PaginatedResponse};

use secreton_core::audit::AuditStatus;
use secreton_core::services::lease::{EnhancedLease, LeaseError};

/// Create lease management routes
pub fn create_routes() -> Router<AppState> {
    Router::new()
        // Lease operations
        .route("/leases/renew", post(renew_lease))
        .route("/leases/revoke", post(revoke_lease))
        .route("/leases/revoke-prefix", post(revoke_lease_prefix))
        .route("/leases/lookup/{lease_id}", get(lookup_lease))
        .route("/leases", get(list_leases))
        .route("/leases/stats", get(get_lease_stats))
}

/// Authenticated context for lease operations
pub struct LeaseAuth {
    pub user: String,
    pub namespace: String,
    pub is_admin: bool,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
}

#[async_trait]
impl<S> FromRequestParts<S> for LeaseAuth
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let context = parts
            .extensions
            .get::<RequestContext>()
            .ok_or(ApiError::Internal {
                message: "Request context not found".to_string(),
            })?;

        let user = context
            .user_id
            .clone()
            .unwrap_or_else(|| "anonymous".to_string());

        let namespace = context
            .jwt_claims
            .as_ref()
            .and_then(|c| c.satker_code.as_ref().or(c.wilayah_code.as_ref()))
            .cloned()
            .unwrap_or_else(|| "default".to_string());

        let is_admin = context
            .user_roles
            .iter()
            .any(|r| r == "admin" || r == "superuser")
            || context
                .jwt_claims
                .as_ref()
                .map(|c| matches!(c.admin_level, AdminLevel::Pusat))
                .unwrap_or(false);

        let ip = parts
            .headers
            .get("x-forwarded-for")
            .or_else(|| parts.headers.get("x-real-ip"))
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let user_agent = parts
            .headers
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        Ok(LeaseAuth {
            user,
            namespace,
            is_admin,
            ip,
            user_agent,
        })
    }
}

/// Request to renew a lease
#[derive(Debug, Deserialize)]
pub struct RenewLeaseRequest {
    /// Lease ID to renew
    pub lease_id: String,

    /// Increment in seconds (optional, uses default if not specified)
    pub increment: Option<i64>,
}

/// Response for lease renewal
#[derive(Debug, Serialize)]
pub struct RenewLeaseResponse {
    /// Lease ID
    pub lease_id: String,

    /// New expiration time
    pub expired_at: DateTime<Utc>,

    /// Lease duration in seconds
    pub lease_duration: i64,

    /// Whether lease is renewable
    pub renewable: bool,

    /// Number of times renewed
    pub renew_count: u32,

    /// Maximum renewals allowed (if set)
    pub max_renewals: Option<u32>,
}

/// Renew a lease by ID
pub async fn renew_lease(
    State(state): State<AppState>,
    auth: LeaseAuth,
    Json(request): Json<RenewLeaseRequest>,
) -> ApiResult<Json<ApiResponse<RenewLeaseResponse>>> {
    // Validate lease_id
    if request.lease_id.is_empty() {
        return Err(ApiError::BadRequest {
            message: "Lease ID cannot be empty".to_string(),
        });
    }

    // Get lease to check ownership
    let lease = state
        .lease_manager
        .lookup_lease(&request.lease_id)
        .await
        .map_err(|e| match e {
            LeaseError::LeaseNotFound(id) => ApiError::NotFound {
                resource: format!("Lease not found: {}", id),
            },
            LeaseError::LeaseExpired => ApiError::BadRequest {
                message: "Lease has expired".to_string(),
            },
            LeaseError::LeaseRevoked => ApiError::BadRequest {
                message: "Lease has been revoked".to_string(),
            },
            _ => ApiError::Internal {
                message: format!("Failed to lookup lease: {}", e),
            },
        })?;

    // Authorization check: user can only renew their own leases unless admin
    if !auth.is_admin && lease.user != auth.user {
        return Err(ApiError::Forbidden);
    }

    // Namespace isolation check
    if !auth.is_admin && lease.namespace != auth.namespace {
        return Err(ApiError::Forbidden);
    }

    // Use default increment if not specified (half of max_ttl)
    let increment = request.increment.unwrap_or(lease.max_ttl / 2);

    // Validate increment
    if increment <= 0 {
        return Err(ApiError::BadRequest {
            message: "Increment must be positive".to_string(),
        });
    }

    // Renew the lease
    let renewed_lease = state
        .lease_manager
        .renew_lease(&request.lease_id, increment)
        .await
        .map_err(|e| match e {
            LeaseError::RenewalNotAllowed => ApiError::BadRequest {
                message: "Lease renewal not allowed (max renewals reached or not renewable)"
                    .to_string(),
            },
            LeaseError::LeaseExpired => ApiError::BadRequest {
                message: "Lease has expired".to_string(),
            },
            LeaseError::LeaseRevoked => ApiError::BadRequest {
                message: "Lease has been revoked".to_string(),
            },
            LeaseError::InvalidTtl(msg) => ApiError::BadRequest { message: msg },
            _ => ApiError::Internal {
                message: format!("Failed to renew lease: {}", e),
            },
        })?;

    // Log audit event
    let mut metadata = HashMap::new();
    metadata.insert("increment".to_string(), increment.to_string());
    metadata.insert(
        "new_expiry".to_string(),
        renewed_lease.expired_at.to_rfc3339(),
    );

    if let Err(e) = state
        .audit
        .log_with_namespace(
            "lease.renew".to_string(),
            Some(auth.user.clone()),
            "lease".to_string(),
            request.lease_id,
            auth.namespace.clone(),
            AuditStatus::Success,
            auth.ip.clone(),
            auth.user_agent.clone(),
            metadata,
        )
        .await
    {
        tracing::error!("Failed to log audit event: {}", e);
        // Continue despite audit failure
    }

    let lease_duration = (renewed_lease.expired_at - renewed_lease.issued_at).num_seconds();

    let response = RenewLeaseResponse {
        lease_id: renewed_lease.id,
        expired_at: renewed_lease.expired_at,
        lease_duration,
        renewable: renewed_lease.renewable,
        renew_count: renewed_lease.renew_count,
        max_renewals: renewed_lease.max_renewals,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Request to revoke a lease
#[derive(Debug, Deserialize)]
pub struct RevokeLeaseRequest {
    /// Lease ID to revoke
    pub lease_id: String,
}

/// Response for lease revocation
#[derive(Debug, Serialize)]
pub struct RevokeLeaseResponse {
    /// Lease ID that was revoked
    pub lease_id: String,

    /// All lease IDs that were revoked (including children)
    pub revoked_ids: Vec<String>,

    /// Number of leases revoked
    pub revoked_count: usize,
}

/// Revoke a lease by ID (with cascade to children)
pub async fn revoke_lease(
    State(state): State<AppState>,
    auth: LeaseAuth,
    Json(request): Json<RevokeLeaseRequest>,
) -> ApiResult<Json<ApiResponse<RevokeLeaseResponse>>> {
    // Validate lease_id
    if request.lease_id.is_empty() {
        return Err(ApiError::BadRequest {
            message: "Lease ID cannot be empty".to_string(),
        });
    }

    // Get lease to check ownership
    let lease = state
        .lease_manager
        .lookup_lease(&request.lease_id)
        .await
        .map_err(|e| match e {
            LeaseError::LeaseNotFound(id) => ApiError::NotFound {
                resource: format!("Lease not found: {}", id),
            },
            _ => ApiError::Internal {
                message: format!("Failed to lookup lease: {}", e),
            },
        })?;

    // Authorization check
    if !auth.is_admin && lease.user != auth.user {
        return Err(ApiError::Forbidden);
    }

    // Namespace isolation check
    if !auth.is_admin && lease.namespace != auth.namespace {
        return Err(ApiError::Forbidden);
    }

    // Revoke the lease (cascades to children)
    let revoked_ids = state
        .lease_manager
        .revoke_lease(&request.lease_id)
        .await
        .map_err(|e| ApiError::Internal {
            message: format!("Failed to revoke lease: {}", e),
        })?;

    // Log audit event
    let mut metadata = HashMap::new();
    metadata.insert("revoked_count".to_string(), revoked_ids.len().to_string());

    if let Err(e) = state
        .audit
        .log_with_namespace(
            "lease.revoke".to_string(),
            Some(auth.user.clone()),
            "lease".to_string(),
            request.lease_id.clone(),
            auth.namespace.clone(),
            AuditStatus::Success,
            auth.ip.clone(),
            auth.user_agent.clone(),
            metadata,
        )
        .await
    {
        tracing::error!("Failed to log audit event: {}", e);
        // Continue despite audit failure
    }

    let response = RevokeLeaseResponse {
        lease_id: request.lease_id,
        revoked_count: revoked_ids.len(),
        revoked_ids,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Request to revoke leases by prefix
#[derive(Debug, Deserialize)]
pub struct RevokePrefixRequest {
    /// Resource path prefix
    pub prefix: String,
}

/// Response for prefix revocation
#[derive(Debug, Serialize)]
pub struct RevokePrefixResponse {
    /// Path prefix that was revoked
    pub prefix: String,

    /// All lease IDs that were revoked
    pub revoked_ids: Vec<String>,

    /// Number of leases revoked
    pub revoked_count: usize,
}

/// Revoke all leases under a path prefix
pub async fn revoke_lease_prefix(
    State(state): State<AppState>,
    auth: LeaseAuth,
    Json(request): Json<RevokePrefixRequest>,
) -> ApiResult<Json<ApiResponse<RevokePrefixResponse>>> {
    // Validate prefix
    if request.prefix.is_empty() {
        return Err(ApiError::BadRequest {
            message: "Prefix cannot be empty".to_string(),
        });
    }

    // Authorization check: only admin can revoke by prefix
    if !auth.is_admin {
        return Err(ApiError::Forbidden);
    }

    // List all leases matching the prefix
    let matching_leases = state
        .lease_manager
        .list_leases(
            None,
            Some(auth.namespace.clone()),
            None,
            Some("active".to_string()),
            None,
            None,
        )
        .await
        .map_err(|e| ApiError::Internal {
            message: format!("Failed to list leases: {}", e),
        })?;

    // Filter leases by prefix
    let leases_to_revoke: Vec<_> = matching_leases
        .into_iter()
        .filter(|lease| lease.resource.starts_with(&request.prefix))
        .collect();

    // Revoke each matching lease
    let mut all_revoked_ids = Vec::new();
    for lease in leases_to_revoke {
        match state.lease_manager.revoke_lease(&lease.id).await {
            Ok(mut revoked_ids) => {
                all_revoked_ids.append(&mut revoked_ids);
            }
            Err(e) => {
                tracing::warn!("Failed to revoke lease {}: {}", lease.id, e);
                // Continue with other leases
            }
        }
    }

    // Log audit event
    let mut metadata = HashMap::new();
    metadata.insert("prefix".to_string(), request.prefix.clone());
    metadata.insert(
        "revoked_count".to_string(),
        all_revoked_ids.len().to_string(),
    );

    if let Err(e) = state
        .audit
        .log_with_namespace(
            "lease.revoke_prefix".to_string(),
            Some(auth.user.clone()),
            "lease".to_string(),
            format!("prefix:{}", request.prefix),
            auth.namespace.clone(),
            AuditStatus::Success,
            auth.ip.clone(),
            auth.user_agent.clone(),
            metadata,
        )
        .await
    {
        tracing::error!("Failed to log audit event: {}", e);
        // Continue despite audit failure
    }

    let response = RevokePrefixResponse {
        prefix: request.prefix,
        revoked_count: all_revoked_ids.len(),
        revoked_ids: all_revoked_ids,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Response for lease lookup
#[derive(Debug, Serialize)]
pub struct LookupLeaseResponse {
    /// Lease ID
    pub lease_id: String,

    /// User/entity ID
    pub user: String,

    /// Resource path
    pub resource: String,

    /// Resource type
    pub resource_type: String,

    /// Namespace
    pub namespace: String,

    /// Issued timestamp
    pub issued_at: DateTime<Utc>,

    /// Expiration timestamp
    pub expired_at: DateTime<Utc>,

    /// Time until expiration (seconds)
    pub ttl: i64,

    /// Status (active, revoked, expired)
    pub status: String,

    /// Whether lease is renewable
    pub renewable: bool,

    /// Maximum TTL
    pub max_ttl: i64,

    /// Number of times renewed
    pub renew_count: u32,

    /// Maximum renewals allowed (if set)
    pub max_renewals: Option<u32>,

    /// Last renewed timestamp
    pub last_renewed_at: Option<DateTime<Utc>>,

    /// Parent lease ID (if any)
    pub parent_id: Option<String>,

    /// Child lease IDs
    pub child_ids: Vec<String>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl From<EnhancedLease> for LookupLeaseResponse {
    fn from(lease: EnhancedLease) -> Self {
        let ttl = (lease.expired_at - Utc::now()).num_seconds().max(0);

        Self {
            lease_id: lease.id,
            user: lease.user,
            resource: lease.resource,
            resource_type: lease.resource_type,
            namespace: lease.namespace,
            issued_at: lease.issued_at,
            expired_at: lease.expired_at,
            ttl,
            status: lease.status,
            renewable: lease.renewable,
            max_ttl: lease.max_ttl,
            renew_count: lease.renew_count,
            max_renewals: lease.max_renewals,
            last_renewed_at: lease.last_renewed_at,
            parent_id: lease.parent_id,
            child_ids: lease.child_ids,
            metadata: lease.metadata,
        }
    }
}

/// Lookup lease details by ID
pub async fn lookup_lease(
    State(state): State<AppState>,
    auth: LeaseAuth,
    Path(lease_id): Path<String>,
) -> ApiResult<Json<ApiResponse<LookupLeaseResponse>>> {
    // Validate lease_id
    if lease_id.is_empty() {
        return Err(ApiError::BadRequest {
            message: "Lease ID cannot be empty".to_string(),
        });
    }

    // Lookup the lease
    let lease = state
        .lease_manager
        .lookup_lease(&lease_id)
        .await
        .map_err(|e| match e {
            LeaseError::LeaseNotFound(id) => ApiError::NotFound {
                resource: format!("Lease not found: {}", id),
            },
            _ => ApiError::Internal {
                message: format!("Failed to lookup lease: {}", e),
            },
        })?;

    // Authorization check
    if !auth.is_admin && lease.user != auth.user {
        return Err(ApiError::Forbidden);
    }

    // Namespace isolation check
    if !auth.is_admin && lease.namespace != auth.namespace {
        return Err(ApiError::Forbidden);
    }

    // Log audit event
    let mut metadata = HashMap::new();
    metadata.insert("status".to_string(), lease.status.clone());
    metadata.insert("resource_type".to_string(), lease.resource_type.clone());

    if let Err(e) = state
        .audit
        .log_with_namespace(
            "lease.lookup".to_string(),
            Some(auth.user.clone()),
            "lease".to_string(),
            lease.id.clone(),
            auth.namespace.clone(),
            AuditStatus::Success,
            auth.ip.clone(),
            auth.user_agent.clone(),
            metadata,
        )
        .await
    {
        tracing::error!("Failed to log audit event: {}", e);
        // Continue despite audit failure
    }

    let response = LookupLeaseResponse::from(lease);

    Ok(Json(ApiResponse::success(response)))
}

/// Query parameters for listing leases
#[derive(Debug, Deserialize)]
pub struct ListLeasesQuery {
    /// Filter by user ID
    pub user_id: Option<String>,

    /// Filter by namespace
    pub namespace: Option<String>,

    /// Filter by resource type
    pub resource_type: Option<String>,

    /// Filter by status (active, revoked, expired)
    pub status: Option<String>,

    /// Pagination limit
    #[serde(default = "default_limit")]
    pub limit: i64,

    /// Pagination offset
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

/// List leases with filtering and pagination
pub async fn list_leases(
    State(state): State<AppState>,
    auth: LeaseAuth,
    Query(query): Query<ListLeasesQuery>,
) -> ApiResult<Json<ApiResponse<PaginatedResponse<LookupLeaseResponse>>>> {
    // Authorization: non-admin users can only see their own leases in their namespace
    let filter_user = if auth.is_admin {
        query.user_id.clone()
    } else {
        Some(auth.user.clone())
    };

    let filter_namespace = if auth.is_admin {
        query.namespace.clone()
    } else {
        Some(auth.namespace.clone())
    };

    // Validate pagination parameters
    if query.limit <= 0 || query.limit > 1000 {
        return Err(ApiError::BadRequest {
            message: "Limit must be between 1 and 1000".to_string(),
        });
    }

    if query.offset < 0 {
        return Err(ApiError::BadRequest {
            message: "Offset must be non-negative".to_string(),
        });
    }

    // List leases with filters
    let leases = state
        .lease_manager
        .list_leases(
            filter_user,
            filter_namespace,
            query.resource_type.clone(),
            query.status.clone(),
            Some(query.limit),
            Some(query.offset),
        )
        .await
        .map_err(|e| ApiError::Internal {
            message: format!("Failed to list leases: {}", e),
        })?;

    // Get total count (for pagination)
    // TODO: Implement count query in LeaseManager
    let total = leases.len() as u64;

    // Convert to response format
    let items: Vec<LookupLeaseResponse> =
        leases.into_iter().map(LookupLeaseResponse::from).collect();

    // Log audit event
    let mut metadata = HashMap::new();
    metadata.insert("count".to_string(), items.len().to_string());
    if let Some(user_id) = &query.user_id {
        metadata.insert("filter_user_id".to_string(), user_id.clone());
    }
    if let Some(resource_type) = &query.resource_type {
        metadata.insert("filter_resource_type".to_string(), resource_type.clone());
    }
    if let Some(status) = &query.status {
        metadata.insert("filter_status".to_string(), status.clone());
    }

    if let Err(e) = state
        .audit
        .log_with_namespace(
            "lease.list".to_string(),
            Some(auth.user.clone()),
            "lease".to_string(),
            "list".to_string(),
            auth.namespace.clone(),
            AuditStatus::Success,
            auth.ip.clone(),
            auth.user_agent.clone(),
            metadata,
        )
        .await
    {
        tracing::error!("Failed to log audit event: {}", e);
        // Continue despite audit failure
    }

    let response = PaginatedResponse::new(items, total, query.limit as u32, query.offset as u32);

    Ok(Json(ApiResponse::success(response)))
}

/// Lease statistics response
#[derive(Debug, Serialize)]
pub struct LeaseStatsResponse {
    /// Number of active leases
    pub active_count: usize,

    /// Number of revoked leases
    pub revoked_count: usize,

    /// Number of expired leases
    pub expired_count: usize,

    /// Number of leases expiring soon (within 5 minutes)
    pub expiring_soon_count: usize,

    /// Number of unique users with leases
    pub unique_users: usize,

    /// Number of unique namespaces with leases
    pub unique_namespaces: usize,

    /// Breakdown by resource type
    pub by_resource_type: HashMap<String, usize>,

    /// Breakdown by namespace
    pub by_namespace: HashMap<String, usize>,
}

/// Get lease statistics
pub async fn get_lease_stats(
    State(state): State<AppState>,
    auth: LeaseAuth,
) -> ApiResult<Json<ApiResponse<LeaseStatsResponse>>> {
    // Get stats from lease manager
    let stats = state
        .lease_manager
        .get_stats()
        .await
        .map_err(|e| ApiError::Internal {
            message: format!("Failed to get lease stats: {}", e),
        })?;

    // TODO: Implement breakdown by resource type and namespace
    let by_resource_type = HashMap::new();
    let by_namespace = HashMap::new();

    // Log audit event
    let mut metadata = HashMap::new();
    metadata.insert("active_count".to_string(), stats.active_count.to_string());
    metadata.insert("revoked_count".to_string(), stats.revoked_count.to_string());
    metadata.insert("expired_count".to_string(), stats.expired_count.to_string());

    if let Err(e) = state
        .audit
        .log_with_namespace(
            "lease.stats".to_string(),
            Some(auth.user.clone()),
            "lease".to_string(),
            "stats".to_string(),
            auth.namespace.clone(),
            AuditStatus::Success,
            auth.ip.clone(),
            auth.user_agent.clone(),
            metadata,
        )
        .await
    {
        tracing::error!("Failed to log audit event: {}", e);
        // Continue despite audit failure
    }

    let response = LeaseStatsResponse {
        active_count: stats.active_count,
        revoked_count: stats.revoked_count,
        expired_count: stats.expired_count,
        expiring_soon_count: stats.expiring_soon_count,
        unique_users: stats.unique_users,
        unique_namespaces: stats.unique_namespaces,
        by_resource_type,
        by_namespace,
    };

    Ok(Json(ApiResponse::success(response)))
}

#[cfg(all(test, feature = "enable-inline-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_default_limit() {
        assert_eq!(default_limit(), 50);
    }
}
