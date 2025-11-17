// ! Client Policy Store
//!
//! Database operations for client policies, profiles, and assignments.
//! Supports CRUD operations and complex queries for policy management.

use crate::error::{AuthencError, Result};
use crate::models::client_policy::*;
use deadpool_postgres::Pool;
use std::sync::Arc;
use tracing::{debug, error, info};
use uuid::Uuid;

/// Client Policy Store
///
/// Manages persistence of client policies, profiles, and their assignments.
pub struct ClientPolicyStore {
    pool: Arc<Pool>,
}

impl ClientPolicyStore {
    /// Create a new client policy store
    pub fn new(pool: Arc<Pool>) -> Self {
        Self { pool }
    }

    // ========== Client Policy CRUD ==========

    /// Create a new client policy
    pub async fn create_policy(
        &self,
        realm_id: Uuid,
        request: CreateClientPolicyRequest,
        created_by: Option<Uuid>,
    ) -> Result<ClientPolicyModel> {
        let client = self.pool.get().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AuthencError::database(format!("Database connection error: {}", e))
        })?;

        let id = Uuid::new_v4();
        let now = chrono::Utc::now();

        let row = client
            .query_one(
                "INSERT INTO client_policies (
                    id, realm_id, name, description, enabled,
                    conditions, condition_config, executors, executor_config,
                    priority, policy_type, created_at, updated_at, created_by
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
                RETURNING *",
                &[
                    &id,
                    &realm_id,
                    &request.name,
                    &request.description,
                    &request.enabled,
                    &request.conditions,
                    &request.condition_config,
                    &request.executors,
                    &request.executor_config,
                    &request.priority,
                    &request.policy_type,
                    &now,
                    &now,
                    &created_by,
                ],
            )
            .await
            .map_err(|e| {
                error!("Failed to create client policy: {}", e);
                AuthencError::database(format!("Failed to create policy: {}", e))
            })?;

        let policy = ClientPolicyModel::try_from(row)?;
        info!("Created client policy: {} ({})", policy.name, policy.id);
        Ok(policy)
    }

    /// Get a client policy by ID
    pub async fn get_policy(&self, policy_id: Uuid) -> Result<Option<ClientPolicyModel>> {
        let client = self.pool.get().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AuthencError::database(format!("Database connection error: {}", e))
        })?;

        let row = client
            .query_opt("SELECT * FROM client_policies WHERE id = $1", &[&policy_id])
            .await
            .map_err(|e| {
                error!("Failed to get client policy: {}", e);
                AuthencError::database(format!("Failed to get policy: {}", e))
            })?;

        match row {
            Some(row) => Ok(Some(ClientPolicyModel::try_from(row)?)),
            None => Ok(None),
        }
    }

    /// List all policies for a realm
    pub async fn list_policies(&self, realm_id: Uuid) -> Result<Vec<ClientPolicyModel>> {
        let client = self.pool.get().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AuthencError::database(format!("Database connection error: {}", e))
        })?;

        let rows = client
            .query(
                "SELECT * FROM client_policies WHERE realm_id = $1 ORDER BY priority DESC, name ASC",
                &[&realm_id],
            )
            .await
            .map_err(|e| {
                error!("Failed to list client policies: {}", e);
                AuthencError::database(format!("Failed to list policies: {}", e))
            })?;

        let mut policies = Vec::new();
        for row in rows {
            policies.push(ClientPolicyModel::try_from(row)?);
        }

        debug!("Listed {} policies for realm {}", policies.len(), realm_id);
        Ok(policies)
    }

    /// Update a client policy
    pub async fn update_policy(
        &self,
        policy_id: Uuid,
        request: UpdateClientPolicyRequest,
    ) -> Result<ClientPolicyModel> {
        let client = self.pool.get().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AuthencError::database(format!("Database connection error: {}", e))
        })?;

        // Build dynamic SQL based on provided fields
        let mut updates = Vec::new();
        let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&policy_id];
        let mut param_count = 2;

        if let Some(name) = &request.name {
            updates.push(format!("name = ${}", param_count));
            params.push(name);
            param_count += 1;
        }

        if let Some(description) = &request.description {
            updates.push(format!("description = ${}", param_count));
            params.push(description);
            param_count += 1;
        }

        if let Some(enabled) = &request.enabled {
            updates.push(format!("enabled = ${}", param_count));
            params.push(enabled);
            param_count += 1;
        }

        if let Some(conditions) = &request.conditions {
            updates.push(format!("conditions = ${}", param_count));
            params.push(conditions);
            param_count += 1;
        }

        if let Some(condition_config) = &request.condition_config {
            updates.push(format!("condition_config = ${}", param_count));
            params.push(condition_config);
            param_count += 1;
        }

        if let Some(executors) = &request.executors {
            updates.push(format!("executors = ${}", param_count));
            params.push(executors);
            param_count += 1;
        }

        if let Some(executor_config) = &request.executor_config {
            updates.push(format!("executor_config = ${}", param_count));
            params.push(executor_config);
            param_count += 1;
        }

        if let Some(priority) = &request.priority {
            updates.push(format!("priority = ${}", param_count));
            params.push(priority);
            param_count += 1;
        }

        if updates.is_empty() {
            return Err(AuthencError::validation("No fields to update".to_string()));
        }

        let sql = format!(
            "UPDATE client_policies SET {} WHERE id = $1 RETURNING *",
            updates.join(", ")
        );

        let row = client.query_one(&sql, &params).await.map_err(|e| {
            error!("Failed to update client policy: {}", e);
            AuthencError::database(format!("Failed to update policy: {}", e))
        })?;

        let policy = ClientPolicyModel::try_from(row)?;
        info!("Updated client policy: {} ({})", policy.name, policy.id);
        Ok(policy)
    }

    /// Delete a client policy
    pub async fn delete_policy(&self, policy_id: Uuid) -> Result<()> {
        let client = self.pool.get().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AuthencError::database(format!("Database connection error: {}", e))
        })?;

        let deleted = client
            .execute("DELETE FROM client_policies WHERE id = $1", &[&policy_id])
            .await
            .map_err(|e| {
                error!("Failed to delete client policy: {}", e);
                AuthencError::database(format!("Failed to delete policy: {}", e))
            })?;

        if deleted == 0 {
            return Err(AuthencError::not_found("Policy not found"));
        }

        info!("Deleted client policy: {}", policy_id);
        Ok(())
    }

    // ========== Client Profile CRUD ==========

    /// Create a new client profile
    pub async fn create_profile(
        &self,
        realm_id: Uuid,
        request: CreateClientProfileRequest,
        created_by: Option<Uuid>,
    ) -> Result<ClientProfileModel> {
        let client = self.pool.get().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AuthencError::database(format!("Database connection error: {}", e))
        })?;

        let id = Uuid::new_v4();
        let now = chrono::Utc::now();

        let row = client
            .query_one(
                "INSERT INTO client_profiles (
                    id, realm_id, name, description, enabled,
                    policy_ids, profile_type, is_builtin,
                    created_at, updated_at, created_by
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                RETURNING *",
                &[
                    &id,
                    &realm_id,
                    &request.name,
                    &request.description,
                    &request.enabled,
                    &request.policy_ids,
                    &request.profile_type,
                    &false, // Custom profiles are not built-in
                    &now,
                    &now,
                    &created_by,
                ],
            )
            .await
            .map_err(|e| {
                error!("Failed to create client profile: {}", e);
                AuthencError::database(format!("Failed to create profile: {}", e))
            })?;

        let profile = ClientProfileModel::try_from(row)?;
        info!("Created client profile: {} ({})", profile.name, profile.id);
        Ok(profile)
    }

    /// Get a client profile by ID
    pub async fn get_profile(&self, profile_id: Uuid) -> Result<Option<ClientProfileModel>> {
        let client = self.pool.get().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AuthencError::database(format!("Database connection error: {}", e))
        })?;

        let row = client
            .query_opt(
                "SELECT * FROM client_profiles WHERE id = $1",
                &[&profile_id],
            )
            .await
            .map_err(|e| {
                error!("Failed to get client profile: {}", e);
                AuthencError::database(format!("Failed to get profile: {}", e))
            })?;

        match row {
            Some(row) => Ok(Some(ClientProfileModel::try_from(row)?)),
            None => Ok(None),
        }
    }

    /// Get a client profile with full policy details
    pub async fn get_profile_with_policies(
        &self,
        profile_id: Uuid,
    ) -> Result<Option<ClientProfileResponse>> {
        let client = self.pool.get().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AuthencError::database(format!("Database connection error: {}", e))
        })?;

        let row = client
            .query_opt(
                "SELECT * FROM v_client_profiles_with_policies WHERE id = $1",
                &[&profile_id],
            )
            .await
            .map_err(|e| {
                error!("Failed to get client profile with policies: {}", e);
                AuthencError::database(format!("Failed to get profile: {}", e))
            })?;

        match row {
            Some(row) => {
                let profile = ClientProfileModel::try_from(row.clone())?;
                let policies_json: serde_json::Value = row.try_get("policies")?;
                let policies: Vec<ClientPolicyResponse> =
                    serde_json::from_value(policies_json).unwrap_or_default();

                Ok(Some(ClientProfileResponse {
                    id: profile.id,
                    realm_id: profile.realm_id,
                    name: profile.name,
                    description: profile.description,
                    enabled: profile.enabled,
                    policy_ids: profile.policy_ids,
                    policies,
                    profile_type: profile.profile_type,
                    is_builtin: profile.is_builtin,
                    created_at: profile.created_at,
                    updated_at: profile.updated_at,
                }))
            }
            None => Ok(None),
        }
    }

    /// List all profiles for a realm
    pub async fn list_profiles(&self, realm_id: Uuid) -> Result<Vec<ClientProfileModel>> {
        let client = self.pool.get().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AuthencError::database(format!("Database connection error: {}", e))
        })?;

        let rows = client
            .query(
                "SELECT * FROM client_profiles WHERE realm_id = $1 ORDER BY name ASC",
                &[&realm_id],
            )
            .await
            .map_err(|e| {
                error!("Failed to list client profiles: {}", e);
                AuthencError::database(format!("Failed to list profiles: {}", e))
            })?;

        let mut profiles = Vec::new();
        for row in rows {
            profiles.push(ClientProfileModel::try_from(row)?);
        }

        debug!("Listed {} profiles for realm {}", profiles.len(), realm_id);
        Ok(profiles)
    }

    /// Update a client profile
    pub async fn update_profile(
        &self,
        profile_id: Uuid,
        request: UpdateClientProfileRequest,
    ) -> Result<ClientProfileModel> {
        let client = self.pool.get().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AuthencError::database(format!("Database connection error: {}", e))
        })?;

        // Check if profile is built-in (cannot be modified significantly)
        let existing = self.get_profile(profile_id).await?;
        if let Some(profile) = existing {
            if profile.is_builtin && request.name.is_some() {
                return Err(AuthencError::validation(
                    "Cannot modify name of built-in profiles".to_string(),
                ));
            }
        }

        // Build dynamic SQL
        let mut updates = Vec::new();
        let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&profile_id];
        let mut param_count = 2;

        if let Some(name) = &request.name {
            updates.push(format!("name = ${}", param_count));
            params.push(name);
            param_count += 1;
        }

        if let Some(description) = &request.description {
            updates.push(format!("description = ${}", param_count));
            params.push(description);
            param_count += 1;
        }

        if let Some(enabled) = &request.enabled {
            updates.push(format!("enabled = ${}", param_count));
            params.push(enabled);
            param_count += 1;
        }

        if let Some(policy_ids) = &request.policy_ids {
            updates.push(format!("policy_ids = ${}", param_count));
            params.push(policy_ids);
            param_count += 1;
        }

        if updates.is_empty() {
            return Err(AuthencError::validation("No fields to update".to_string()));
        }

        let sql = format!(
            "UPDATE client_profiles SET {} WHERE id = $1 RETURNING *",
            updates.join(", ")
        );

        let row = client.query_one(&sql, &params).await.map_err(|e| {
            error!("Failed to update client profile: {}", e);
            AuthencError::database(format!("Failed to update profile: {}", e))
        })?;

        let profile = ClientProfileModel::try_from(row)?;
        info!("Updated client profile: {} ({})", profile.name, profile.id);
        Ok(profile)
    }

    /// Delete a client profile
    pub async fn delete_profile(&self, profile_id: Uuid) -> Result<()> {
        let client = self.pool.get().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AuthencError::database(format!("Database connection error: {}", e))
        })?;

        // Check if profile is built-in (cannot be deleted)
        let existing = self.get_profile(profile_id).await?;
        if let Some(profile) = existing {
            if profile.is_builtin {
                return Err(AuthencError::validation(
                    "Cannot delete built-in profiles".to_string(),
                ));
            }
        }

        let deleted = client
            .execute(
                "DELETE FROM client_profiles WHERE id = $1 AND is_builtin = false",
                &[&profile_id],
            )
            .await
            .map_err(|e| {
                error!("Failed to delete client profile: {}", e);
                AuthencError::database(format!("Failed to delete profile: {}", e))
            })?;

        if deleted == 0 {
            return Err(AuthencError::not_found("Profile not found or is built-in"));
        }

        info!("Deleted client profile: {}", profile_id);
        Ok(())
    }

    // ========== Policy Assignments ==========

    /// Assign a policy to a client (direct assignment)
    pub async fn assign_policy_to_client(
        &self,
        client_id: Uuid,
        policy_id: Uuid,
        enabled: bool,
        priority_override: Option<i32>,
        assigned_by: Option<Uuid>,
    ) -> Result<ClientPolicyAssignment> {
        let client = self.pool.get().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AuthencError::database(format!("Database connection error: {}", e))
        })?;

        let id = Uuid::new_v4();
        let now = chrono::Utc::now();

        let row = client
            .query_one(
                "INSERT INTO client_policy_assignments (
                    id, client_id, policy_id, profile_id, assignment_type,
                    enabled, priority_override, assigned_at, assigned_by
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                ON CONFLICT (client_id, policy_id, profile_id)
                DO UPDATE SET
                    enabled = EXCLUDED.enabled,
                    priority_override = EXCLUDED.priority_override,
                    assigned_at = EXCLUDED.assigned_at,
                    assigned_by = EXCLUDED.assigned_by
                RETURNING *",
                &[
                    &id,
                    &client_id,
                    &Some(policy_id),
                    &None::<Uuid>,
                    &"direct",
                    &enabled,
                    &priority_override,
                    &now,
                    &assigned_by,
                ],
            )
            .await
            .map_err(|e| {
                error!("Failed to assign policy to client: {}", e);
                AuthencError::database(format!("Failed to assign policy: {}", e))
            })?;

        let assignment = ClientPolicyAssignment::try_from(row)?;
        info!("Assigned policy {} to client {}", policy_id, client_id);
        Ok(assignment)
    }

    /// Assign a profile to a client
    pub async fn assign_profile_to_client(
        &self,
        client_id: Uuid,
        profile_id: Uuid,
        enabled: bool,
        assigned_by: Option<Uuid>,
    ) -> Result<ClientPolicyAssignment> {
        let client = self.pool.get().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AuthencError::database(format!("Database connection error: {}", e))
        })?;

        let id = Uuid::new_v4();
        let now = chrono::Utc::now();

        let row = client
            .query_one(
                "INSERT INTO client_policy_assignments (
                    id, client_id, policy_id, profile_id, assignment_type,
                    enabled, priority_override, assigned_at, assigned_by
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                ON CONFLICT (client_id, policy_id, profile_id)
                DO UPDATE SET
                    enabled = EXCLUDED.enabled,
                    assigned_at = EXCLUDED.assigned_at,
                    assigned_by = EXCLUDED.assigned_by
                RETURNING *",
                &[
                    &id,
                    &client_id,
                    &None::<Uuid>,
                    &Some(profile_id),
                    &"profile",
                    &enabled,
                    &None::<i32>,
                    &now,
                    &assigned_by,
                ],
            )
            .await
            .map_err(|e| {
                error!("Failed to assign profile to client: {}", e);
                AuthencError::database(format!("Failed to assign profile: {}", e))
            })?;

        let assignment = ClientPolicyAssignment::try_from(row)?;
        info!("Assigned profile {} to client {}", profile_id, client_id);
        Ok(assignment)
    }

    /// Get all policies assigned to a client (including via profiles)
    pub async fn get_client_policies(&self, client_id: Uuid) -> Result<Vec<ClientPolicyModel>> {
        let client = self.pool.get().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AuthencError::database(format!("Database connection error: {}", e))
        })?;

        let rows = client
            .query(
                "SELECT DISTINCT cp.*
                FROM client_policies cp
                INNER JOIN (
                    -- Direct policy assignments
                    SELECT policy_id as id, priority_override
                    FROM client_policy_assignments
                    WHERE client_id = $1
                    AND assignment_type = 'direct'
                    AND enabled = true
                    AND policy_id IS NOT NULL

                    UNION

                    -- Policies from assigned profiles
                    SELECT unnest(prof.policy_ids) as id, NULL as priority_override
                    FROM client_policy_assignments cpa
                    INNER JOIN client_profiles prof ON cpa.profile_id = prof.id
                    WHERE cpa.client_id = $1
                    AND cpa.assignment_type = 'profile'
                    AND cpa.enabled = true
                    AND prof.enabled = true
                ) assigned ON cp.id = assigned.id
                WHERE cp.enabled = true
                ORDER BY COALESCE(assigned.priority_override, cp.priority) DESC, cp.name ASC",
                &[&client_id],
            )
            .await
            .map_err(|e| {
                error!("Failed to get client policies: {}", e);
                AuthencError::database(format!("Failed to get client policies: {}", e))
            })?;

        let mut policies = Vec::new();
        for row in rows {
            policies.push(ClientPolicyModel::try_from(row)?);
        }

        debug!(
            "Retrieved {} policies for client {}",
            policies.len(),
            client_id
        );
        Ok(policies)
    }

    /// Remove policy assignment from client
    pub async fn remove_policy_assignment(
        &self,
        client_id: Uuid,
        policy_id: Option<Uuid>,
        profile_id: Option<Uuid>,
    ) -> Result<()> {
        let client = self.pool.get().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AuthencError::database(format!("Database connection error: {}", e))
        })?;

        let deleted = client
            .execute(
                "DELETE FROM client_policy_assignments
                WHERE client_id = $1
                AND (policy_id = $2 OR profile_id = $3)",
                &[&client_id, &policy_id, &profile_id],
            )
            .await
            .map_err(|e| {
                error!("Failed to remove policy assignment: {}", e);
                AuthencError::database(format!("Failed to remove assignment: {}", e))
            })?;

        if deleted == 0 {
            return Err(AuthencError::not_found("Assignment not found"));
        }

        info!("Removed policy assignment from client {}", client_id);
        Ok(())
    }

    /// List all assignments for a client
    pub async fn list_client_assignments(
        &self,
        client_id: Uuid,
    ) -> Result<Vec<ClientPolicyAssignment>> {
        let client = self.pool.get().await.map_err(|e| {
            error!("Failed to get database connection: {}", e);
            AuthencError::database(format!("Database connection error: {}", e))
        })?;

        let rows = client
            .query(
                "SELECT * FROM client_policy_assignments WHERE client_id = $1",
                &[&client_id],
            )
            .await
            .map_err(|e| {
                error!("Failed to list client assignments: {}", e);
                AuthencError::database(format!("Failed to list assignments: {}", e))
            })?;

        let mut assignments = Vec::new();
        for row in rows {
            assignments.push(ClientPolicyAssignment::try_from(row)?);
        }

        Ok(assignments)
    }
}

#[cfg(test)]
mod tests {

    // Note: These tests require a running PostgreSQL database with the schema
    // For unit tests, consider using a mock or an in-memory database

    #[tokio::test]
    #[ignore] // Ignore by default, run with --ignored flag
    async fn test_create_policy() {
        // Test requires database setup
        // Implementation would go here
    }
}
