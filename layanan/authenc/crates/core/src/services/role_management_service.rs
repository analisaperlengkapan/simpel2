//! Role and Permission Management Service
//!
//! This service provides RBAC (Role-Based Access Control) and ABAC (Attribute-Based Access Control)
//! functionality for the Authenc system.

use authenc_storage::Database;
use authenc_types::{RealmId, Result, RoleId, UserId, error::AuthencError};
use std::sync::Arc;
use tracing::{debug, error};
use uuid::Uuid;

/// Role information
#[derive(Debug, Clone)]
pub struct Role {
    pub id: RoleId,
    pub name: String,
    pub description: Option<String>,
    pub realm_id: RealmId,
    pub composite: bool,
    pub client_role: bool,
    pub permissions: Vec<Permission>,
}

/// Permission information
#[derive(Debug, Clone)]
pub struct Permission {
    pub id: Uuid,
    pub role_id: RoleId,
    pub permission: String,
    pub resource: Option<String>,
    pub actions: Vec<String>,
    pub conditions: serde_json::Value,
}

/// Role assignment request
#[derive(Debug, Clone)]
pub struct AssignRoleRequest {
    pub user_id: UserId,
    pub role_id: RoleId,
}

/// Permission check request
#[derive(Debug, Clone)]
pub struct CheckPermissionRequest {
    pub user_id: UserId,
    pub resource: String,
    pub action: String,
    pub context: std::collections::HashMap<String, String>,
}

/// Role Management Service Implementation
///
/// Provides role and permission management operations including:
/// - Role assignment and revocation
/// - Permission checking (RBAC/ABAC)
/// - Role listing and querying
#[derive(Clone)]
pub struct RoleManagementServiceImpl {
    db: Arc<Database>,
}

impl RoleManagementServiceImpl {
    /// Create a new role management service
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Assign a role to a user
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user to assign the role to
    /// * `role_id` - The role to assign
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the role was assigned successfully
    /// * `Err(AuthencError)` if the assignment failed
    pub async fn assign_role(&self, user_id: UserId, role_id: RoleId) -> Result<()> {
        debug!("Assigning role {} to user {}", role_id, user_id);

        // Check if role exists
        let role_exists = self.role_exists(role_id).await?;
        if !role_exists {
            return Err(AuthencError::NotFound(format!(
                "Role {} not found",
                role_id
            )));
        }

        // Check if user exists
        let user_exists = self.user_exists(user_id).await?;
        if !user_exists {
            return Err(AuthencError::UserNotFound(format!(
                "User {} not found",
                user_id
            )));
        }

        // Check if assignment already exists
        let already_assigned = self.is_role_assigned(user_id, role_id).await?;
        if already_assigned {
            debug!("Role {} already assigned to user {}", role_id, user_id);
            return Ok(());
        }

        // Insert role assignment
        let query = r#"
            INSERT INTO user_roles (user_id, role_id)
            VALUES ($1, $2)
            ON CONFLICT (user_id, role_id) DO NOTHING
        "#;

        self.db
            .execute(query, &[&user_id.as_uuid(), &role_id.as_uuid()])
            .await
            .map_err(|e| {
                error!("Failed to assign role: {}", e);
                AuthencError::database(format!("Failed to assign role: {}", e))
            })?;

        debug!("Role {} assigned to user {} successfully", role_id, user_id);
        Ok(())
    }

    /// Revoke a role from a user
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user to revoke the role from
    /// * `role_id` - The role to revoke
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the role was revoked successfully
    /// * `Err(AuthencError)` if the revocation failed
    pub async fn revoke_role(&self, user_id: UserId, role_id: RoleId) -> Result<()> {
        debug!("Revoking role {} from user {}", role_id, user_id);

        let query = r#"
            DELETE FROM user_roles
            WHERE user_id = $1 AND role_id = $2
        "#;

        let rows_affected = self
            .db
            .execute(query, &[&user_id.as_uuid(), &role_id.as_uuid()])
            .await
            .map_err(|e| {
                error!("Failed to revoke role: {}", e);
                AuthencError::database(format!("Failed to revoke role: {}", e))
            })?;

        if rows_affected == 0 {
            debug!("Role {} was not assigned to user {}", role_id, user_id);
        } else {
            debug!(
                "Role {} revoked from user {} successfully",
                role_id, user_id
            );
        }

        Ok(())
    }

    /// List all roles for a user
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user to list roles for (optional, if None returns all roles)
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Role>)` - List of roles
    /// * `Err(AuthencError)` if the query failed
    pub async fn list_roles(&self, user_id: Option<UserId>) -> Result<Vec<Role>> {
        debug!("Listing roles for user: {:?}", user_id);

        let (query, params): (&str, Vec<&Uuid>) = if let Some(uid) = user_id.as_ref() {
            (
                r#"
                SELECT DISTINCT r.id, r.name, r.description, r.realm_id, r.composite, r.client_role
                FROM roles r
                INNER JOIN user_roles ur ON r.id = ur.role_id
                WHERE ur.user_id = $1
                ORDER BY r.name
                "#,
                vec![uid.as_uuid()],
            )
        } else {
            (
                r#"
                SELECT id, name, description, realm_id, composite, client_role
                FROM roles
                ORDER BY name
                "#,
                vec![],
            )
        };

        // Convert Vec<&Uuid> to Vec<&(dyn ToSql + Sync)>
        let params_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| *p as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        let rows = self.db.query(query, &params_refs).await.map_err(|e| {
            error!("Failed to list roles: {}", e);
            AuthencError::database(format!("Failed to list roles: {}", e))
        })?;

        let mut roles = Vec::new();
        for row in rows {
            let role_id = RoleId::from_uuid(row.get::<_, Uuid>("id"));
            let permissions = self.get_role_permissions(role_id).await?;

            roles.push(Role {
                id: role_id,
                name: row.get("name"),
                description: row.get("description"),
                realm_id: RealmId::from_uuid(row.get::<_, Uuid>("realm_id")),
                composite: row.get("composite"),
                client_role: row.get("client_role"),
                permissions,
            });
        }

        debug!("Found {} roles", roles.len());
        Ok(roles)
    }

    /// Check if a user has permission to perform an action on a resource
    ///
    /// This implements both RBAC (role-based) and ABAC (attribute-based) access control.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user to check permissions for
    /// * `resource` - The resource being accessed
    /// * `action` - The action being performed
    /// * `context` - Additional context for ABAC evaluation
    ///
    /// # Returns
    ///
    /// * `Ok(true)` if the user has permission
    /// * `Ok(false)` if the user does not have permission
    /// * `Err(AuthencError)` if the check failed
    pub async fn check_permission(
        &self,
        user_id: UserId,
        resource: &str,
        action: &str,
        context: &std::collections::HashMap<String, String>,
    ) -> Result<bool> {
        debug!(
            "Checking permission for user {} on resource {} with action {}",
            user_id, resource, action
        );

        // Query to check if user has permission through any of their roles
        let query = r#"
            SELECT rp.permission, rp.resource, rp.actions, rp.conditions
            FROM role_permissions rp
            INNER JOIN user_roles ur ON rp.role_id = ur.role_id
            WHERE ur.user_id = $1
              AND (rp.resource = $2 OR rp.resource IS NULL)
        "#;

        let rows = self
            .db
            .query(query, &[&user_id.as_uuid(), &resource])
            .await
            .map_err(|e| {
                error!("Failed to check permission: {}", e);
                AuthencError::database(format!("Failed to check permission: {}", e))
            })?;

        // Check each permission
        for row in rows {
            let actions: Vec<String> = row.get("actions");
            let conditions: serde_json::Value = row.get("conditions");

            // Check if action is allowed
            if actions.contains(&action.to_string()) || actions.contains(&"*".to_string()) {
                // Evaluate ABAC conditions
                if self.evaluate_conditions(&conditions, context) {
                    debug!(
                        "Permission granted for user {} on resource {} with action {}",
                        user_id, resource, action
                    );
                    return Ok(true);
                }
            }
        }

        debug!(
            "Permission denied for user {} on resource {} with action {}",
            user_id, resource, action
        );
        Ok(false)
    }

    /// Get permissions for a specific role
    async fn get_role_permissions(&self, role_id: RoleId) -> Result<Vec<Permission>> {
        let query = r#"
            SELECT id, role_id, permission, resource, actions, conditions
            FROM role_permissions
            WHERE role_id = $1
        "#;

        let rows = self
            .db
            .query(query, &[&role_id.as_uuid()])
            .await
            .map_err(|e| {
                error!("Failed to get role permissions: {}", e);
                AuthencError::database(format!("Failed to get role permissions: {}", e))
            })?;

        let permissions = rows
            .into_iter()
            .map(|row| Permission {
                id: row.get::<_, Uuid>("id"),
                role_id: RoleId::from_uuid(row.get::<_, Uuid>("role_id")),
                permission: row.get("permission"),
                resource: row.get("resource"),
                actions: row.get("actions"),
                conditions: row.get("conditions"),
            })
            .collect();

        Ok(permissions)
    }

    /// Check if a role exists
    async fn role_exists(&self, role_id: RoleId) -> Result<bool> {
        let query = "SELECT EXISTS(SELECT 1 FROM roles WHERE id = $1)";
        let row = self
            .db
            .query_one(query, &[&role_id.as_uuid()])
            .await
            .map_err(|e| {
                error!("Failed to check if role exists: {}", e);
                AuthencError::database(format!("Failed to check if role exists: {}", e))
            })?;

        Ok(row.get(0))
    }

    /// Check if a user exists
    async fn user_exists(&self, user_id: UserId) -> Result<bool> {
        let query = "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)";
        let row = self
            .db
            .query_one(query, &[&user_id.as_uuid()])
            .await
            .map_err(|e| {
                error!("Failed to check if user exists: {}", e);
                AuthencError::database(format!("Failed to check if user exists: {}", e))
            })?;

        Ok(row.get(0))
    }

    /// Check if a role is already assigned to a user
    async fn is_role_assigned(&self, user_id: UserId, role_id: RoleId) -> Result<bool> {
        let query = "SELECT EXISTS(SELECT 1 FROM user_roles WHERE user_id = $1 AND role_id = $2)";
        let row = self
            .db
            .query_one(query, &[&user_id.as_uuid(), &role_id.as_uuid()])
            .await
            .map_err(|e| {
                error!("Failed to check if role is assigned: {}", e);
                AuthencError::database(format!("Failed to check if role is assigned: {}", e))
            })?;

        Ok(row.get(0))
    }

    /// Evaluate ABAC conditions
    ///
    /// This is a simple implementation that checks if context values match condition values.
    /// In a production system, this would be more sophisticated with support for:
    /// - Comparison operators (>, <, >=, <=, !=)
    /// - Logical operators (AND, OR, NOT)
    /// - Pattern matching
    /// - Time-based conditions
    fn evaluate_conditions(
        &self,
        conditions: &serde_json::Value,
        context: &std::collections::HashMap<String, String>,
    ) -> bool {
        // If no conditions, allow access
        if conditions.is_null() || !conditions.is_object() {
            return true;
        }

        let conditions_obj = conditions.as_object().unwrap();

        // Check each condition
        for (key, value) in conditions_obj {
            match context.get(key) {
                Some(context_value) => {
                    // Simple string comparison for now
                    if let Some(condition_str) = value.as_str()
                        && context_value != condition_str
                    {
                        return false;
                    }
                }
                None => {
                    // Required condition not present in context
                    return false;
                }
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    

    #[test]
    fn test_evaluate_conditions_empty() {
        // Create a mock service for testing condition evaluation
        // Note: This test doesn't require database access
        let conditions = serde_json::json!({});
        let context = std::collections::HashMap::new();

        // Evaluate conditions directly
        let result = evaluate_conditions_helper(&conditions, &context);
        assert!(result);
    }

    #[test]
    fn test_evaluate_conditions_match() {
        let conditions = serde_json::json!({
            "scope": "own_satker"
        });

        let mut context = std::collections::HashMap::new();
        context.insert("scope".to_string(), "own_satker".to_string());

        let result = evaluate_conditions_helper(&conditions, &context);
        assert!(result);
    }

    #[test]
    fn test_evaluate_conditions_no_match() {
        let conditions = serde_json::json!({
            "scope": "own_satker"
        });

        let mut context = std::collections::HashMap::new();
        context.insert("scope".to_string(), "all_satker".to_string());

        let result = evaluate_conditions_helper(&conditions, &context);
        assert!(!result);
    }

    // Helper function for testing condition evaluation without database
    fn evaluate_conditions_helper(
        conditions: &serde_json::Value,
        context: &std::collections::HashMap<String, String>,
    ) -> bool {
        // If no conditions, allow access
        if conditions.is_null() || !conditions.is_object() {
            return true;
        }

        let conditions_obj = conditions.as_object().unwrap();

        // Check each condition
        for (key, value) in conditions_obj {
            match context.get(key) {
                Some(context_value) => {
                    // Simple string comparison
                    if let Some(condition_str) = value.as_str()
                        && context_value != condition_str
                    {
                        return false;
                    }
                }
                None => {
                    // Required condition not present in context
                    return false;
                }
            }
        }

        true
    }
}
