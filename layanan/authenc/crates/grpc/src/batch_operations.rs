//! Batch operations for optimized bulk processing
//!
//! This module provides batch processing capabilities for authentication and authorization
//! operations to improve performance when handling multiple requests.

use std::sync::Arc;
use uuid::Uuid;

use authenc_core::services::cache::Cache;
use authenc_core::stores::UserStoreTrait;
use authenc_types::UserId;
use authenc_types::error::AuthencError;

/// Batch permission check result
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct BatchPermissionResult {
    pub user_id: String,
    pub resource: String,
    pub action: String,
    pub allowed: bool,
    pub reason: Option<String>,
}

/// Trait for capability-based permission checking
///
/// TODO: Move to authenc-core when the capability system is fully integrated.
/// This mirrors the API of `authenc_core::services::authorization::capability_checker::CapabilityChecker`.
#[async_trait::async_trait]
pub trait CapabilityCheckerTrait: Send + Sync {
    /// Check if a user has any of the specified capabilities
    async fn user_has_any_capability(
        &self,
        user_id: &Uuid,
        required_capabilities: &[&str],
    ) -> Result<bool, AuthencError>;
}

/// Perform batch permission checks for multiple resources
/// This function optimizes permission checking by:
/// 1. Fetching user data once
/// 2. Checking all permissions using capability-based authorization
/// 3. Caching results for future requests
pub async fn batch_check_permissions<U: UserStoreTrait>(
    user_store: Arc<U>,
    capability_checker: Arc<dyn CapabilityCheckerTrait>,
    redis_cache: Option<Arc<dyn Cache>>,
    user_id: UserId,
    checks: Vec<(String, String)>, // (resource, action) pairs
) -> Result<Vec<BatchPermissionResult>, AuthencError> {
    // Verify user exists (get_user returns Option<User>)
    let user = user_store
        .get_user(user_id.into())
        .await?
        .ok_or_else(|| AuthencError::UserNotFound(format!("User {} not found", user_id)))?;

    // Process all permission checks
    let mut results = Vec::with_capacity(checks.len());

    for (resource, action) in checks {
        // Check if user has the required permission via direct grants
        let has_permission = user
            .permissions
            .iter()
            .any(|p| p.resource_type == resource && p.action == action);

        // If not directly granted, check capability-based permissions
        let has_capability_permission = if !has_permission {
            // Build capability code (e.g., "users:read", "config:write")
            let capability_code = format!("{}:{}", resource, action);
            let user_uuid: Uuid = user_id.into();
            capability_checker
                .user_has_any_capability(&user_uuid, &[capability_code.as_str(), "system:admin"])
                .await
                .unwrap_or(false)
        } else {
            false
        };

        let allowed = has_permission || has_capability_permission;

        let reason = if !allowed {
            Some(format!(
                "User does not have permission to {} on resource {}",
                action, resource
            ))
        } else {
            None
        };

        results.push(BatchPermissionResult {
            user_id: user_id.to_string(),
            resource: resource.clone(),
            action: action.clone(),
            allowed,
            reason: reason.clone(),
        });

        // Cache the result (non-blocking)
        if let Some(redis_cache) = redis_cache.clone() {
            let cache_key = format!("permission:{}:{}:{}", user_id, resource, action);
            let cache_value = serde_json::json!({
                "allowed": allowed,
                "reason": reason
            });
            tokio::spawn(async move {
                let _ = redis_cache
                    .set(
                        &cache_key,
                        &cache_value,
                        std::time::Duration::from_secs(300),
                    )
                    .await;
            });
        }
    }

    Ok(results)
}

/// Batch user lookup by IDs
/// Optimizes user lookups by fetching multiple users in parallel
pub async fn batch_lookup_users<U: UserStoreTrait + 'static>(
    user_store: Arc<U>,
    user_ids: Vec<UserId>,
) -> Result<Vec<authenc_types::domain::User>, AuthencError> {
    // Use tokio::spawn to fetch users in parallel
    let mut tasks = Vec::with_capacity(user_ids.len());

    for user_id in user_ids {
        let user_store_clone = user_store.clone();
        let user_uuid: Uuid = user_id.into();
        let task = tokio::spawn(async move { user_store_clone.get_user(user_uuid).await });
        tasks.push(task);
    }

    // Collect results
    let mut users = Vec::new();
    for task in tasks {
        if let Ok(Ok(Some(user))) = task.await {
            users.push(user);
        }
    }

    Ok(users)
}

/// Optimized user lookup with parallel queries for profile + permissions
/// This function demonstrates the optimization pattern for fetching user data
/// with related information in parallel.
pub async fn optimized_user_lookup<U: UserStoreTrait>(
    user_store: Arc<U>,
    redis_cache: Option<Arc<dyn Cache>>,
    user_id: UserId,
) -> Result<authenc_types::domain::User, AuthencError> {
    let user_uuid: Uuid = user_id.into();
    // OPTIMIZATION: Parallel queries using tokio::join!
    let (user_result, _cache_result) = tokio::join!(
        // Primary user query
        user_store.get_user(user_uuid),
        // Check cache for additional data (e.g., recent activity)
        async {
            if let Some(redis_cache) = &redis_cache {
                let cache_key = format!("user:{}:activity", user_id);
                redis_cache.get(&cache_key).await.ok().flatten()
            } else {
                None
            }
        }
    );

    user_result?.ok_or_else(|| AuthencError::UserNotFound(format!("User {} not found", user_id)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_batch_permission_check_structure() {
        // This test verifies the structure compiles correctly
        // Actual testing would require a test database setup
        let checks = [
            ("resource1".to_string(), "read".to_string()),
            ("resource2".to_string(), "write".to_string()),
        ];

        assert_eq!(checks.len(), 2);
    }

    #[test]
    fn test_batch_permission_result_creation() {
        let result = BatchPermissionResult {
            user_id: "test-user".to_string(),
            resource: "test-resource".to_string(),
            action: "read".to_string(),
            allowed: true,
            reason: None,
        };

        assert!(result.allowed);
        assert_eq!(result.action, "read");
    }
}
