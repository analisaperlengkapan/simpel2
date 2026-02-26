//! UMA 2.0 Policy Store
//!
//! Database operations for UMA policies, delegation policies, and claims gathering state.

use crate::services::uma::policy_engine::{DecisionStrategy, Logic, PolicyType, UmaPolicy};
use crate::services::uma::resource_owner_auth::DelegationPolicy;
use authenc_storage::Database;
use authenc_types::{AuthencError, Result};
use std::sync::Arc;
use uuid::Uuid;

/// UMA policy storage operations
pub struct UmaPolicyStore {
    database: Arc<Database>,
}

impl UmaPolicyStore {
    /// Create new UMA policy store
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    /// Create a new policy
    pub async fn create_policy(&self, policy: &UmaPolicy) -> Result<()> {
        let client = self.database.get_connection().await?;

        client
            .execute(
                "INSERT INTO uma_policies (id, name, description, policy_type, logic, decision_strategy, config, enabled, realm_id, resource_server_id, created_at, updated_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW(), NOW())",
                &[
                    &policy.id,
                    &policy.name,
                    &policy.description,
                    &format!("{:?}", policy.policy_type).to_lowercase(),
                    &format!("{:?}", policy.logic),
                    &format!("{:?}", policy.decision_strategy),
                    &serde_json::to_value(&policy.config).map_err(|e| {
                        AuthencError::internal(format!("Failed to serialize policy config: {}", e))
                    })?,
                    &policy.enabled,
                    &Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(), // TODO: Get realm_id
                    &Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(), // TODO: Get resource_server_id
                ],
            )
            .await
            .map_err(|e| AuthencError::database(e.to_string()))?;

        // Insert policy-resource mappings
        for resource_id in &policy.resources {
            if let Ok(resource_uuid) = Uuid::parse_str(resource_id) {
                let _ = client
                    .execute(
                        "INSERT INTO uma_policy_resources (policy_id, resource_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                        &[&policy.id, &resource_uuid],
                    )
                    .await;
            }
        }

        // Insert policy-scope mappings
        for scope_id in &policy.scopes {
            if let Ok(scope_uuid) = Uuid::parse_str(scope_id) {
                let _ = client
                    .execute(
                        "INSERT INTO uma_policy_scopes (policy_id, scope_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                        &[&policy.id, &scope_uuid],
                    )
                    .await;
            }
        }

        Ok(())
    }

    /// Get policy by ID
    pub async fn get_policy(&self, policy_id: &Uuid) -> Result<Option<UmaPolicy>> {
        let client = self.database.get_connection().await?;

        let row = client
            .query_opt(
                "SELECT id, name, description, policy_type, logic, decision_strategy, config, enabled
                 FROM uma_policies WHERE id = $1",
                &[policy_id],
            )
            .await
            .map_err(|e| AuthencError::database(e.to_string()))?;

        if let Some(row) = row {
            let policy_type_str: String = row.get(3);
            let logic_str: String = row.get(4);
            let decision_strategy_str: String = row.get(5);
            let config_json: serde_json::Value = row.get(6);

            // Get associated resources
            let resource_rows = client
                .query(
                    "SELECT resource_id FROM uma_policy_resources WHERE policy_id = $1",
                    &[policy_id],
                )
                .await
                .map_err(|e| AuthencError::database(e.to_string()))?;

            let resources: Vec<String> = resource_rows
                .iter()
                .map(|r| r.get::<_, Uuid>(0).to_string())
                .collect();

            // Get associated scopes
            let scope_rows = client
                .query(
                    "SELECT scope_id FROM uma_policy_scopes WHERE policy_id = $1",
                    &[policy_id],
                )
                .await
                .map_err(|e| AuthencError::database(e.to_string()))?;

            let scopes: Vec<String> = scope_rows
                .iter()
                .map(|r| r.get::<_, Uuid>(0).to_string())
                .collect();

            Ok(Some(UmaPolicy {
                id: row.get(0),
                name: row.get(1),
                description: row.get(2),
                policy_type: self.parse_policy_type(&policy_type_str)?,
                logic: self.parse_logic(&logic_str)?,
                decision_strategy: self.parse_decision_strategy(&decision_strategy_str)?,
                config: serde_json::from_value(config_json).map_err(|e| {
                    AuthencError::internal(format!("Failed to parse policy config: {}", e))
                })?,
                resources,
                scopes,
                clients: None, // TODO: Load from uma_policy_clients
                enabled: row.get(7),
            }))
        } else {
            Ok(None)
        }
    }

    /// Get policies for a resource
    pub async fn get_policies_for_resource(&self, resource_id: &Uuid) -> Result<Vec<UmaPolicy>> {
        let client = self.database.get_connection().await?;

        let rows = client
            .query(
                "SELECT DISTINCT p.id FROM uma_policies p
                 INNER JOIN uma_policy_resources pr ON p.id = pr.policy_id
                 WHERE pr.resource_id = $1 AND p.enabled = true",
                &[resource_id],
            )
            .await
            .map_err(|e| AuthencError::database(e.to_string()))?;

        let mut policies = Vec::new();
        for row in rows {
            let policy_id: Uuid = row.get(0);
            if let Some(policy) = self.get_policy(&policy_id).await? {
                policies.push(policy);
            }
        }

        Ok(policies)
    }

    /// Delete a policy
    pub async fn delete_policy(&self, policy_id: &Uuid) -> Result<()> {
        let client = self.database.get_connection().await?;

        client
            .execute("DELETE FROM uma_policies WHERE id = $1", &[policy_id])
            .await
            .map_err(|e| AuthencError::database(e.to_string()))?;

        Ok(())
    }

    /// Helper to parse policy type from string
    fn parse_policy_type(&self, s: &str) -> Result<PolicyType> {
        match s.to_lowercase().as_str() {
            "role" => Ok(PolicyType::Role),
            "user" => Ok(PolicyType::User),
            "group" => Ok(PolicyType::Group),
            "time" => Ok(PolicyType::Time),
            "attribute" => Ok(PolicyType::Attribute),
            "javascript" => Ok(PolicyType::JavaScript),
            "aggregate" => Ok(PolicyType::Aggregate),
            "client" => Ok(PolicyType::Client),
            _ => Err(AuthencError::internal(format!(
                "Unknown policy type: {}",
                s
            ))),
        }
    }

    /// Helper to parse logic from string
    fn parse_logic(&self, s: &str) -> Result<Logic> {
        match s.to_uppercase().as_str() {
            "POSITIVE" => Ok(Logic::Positive),
            "NEGATIVE" => Ok(Logic::Negative),
            _ => Err(AuthencError::internal(format!("Unknown logic: {}", s))),
        }
    }

    /// Helper to parse decision strategy from string
    fn parse_decision_strategy(&self, s: &str) -> Result<DecisionStrategy> {
        match s.to_uppercase().as_str() {
            "UNANIMOUS" => Ok(DecisionStrategy::Unanimous),
            "AFFIRMATIVE" => Ok(DecisionStrategy::Affirmative),
            "CONSENSUS" => Ok(DecisionStrategy::Consensus),
            _ => Err(AuthencError::internal(format!(
                "Unknown decision strategy: {}",
                s
            ))),
        }
    }
}

/// UMA delegation policy storage operations
pub struct UmaDelegationPolicyStore {
    database: Arc<Database>,
}

impl UmaDelegationPolicyStore {
    /// Create new UMA delegation policy store
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    /// Create a new delegation policy
    pub async fn create_delegation_policy(&self, policy: &DelegationPolicy) -> Result<()> {
        let client = self.database.get_connection().await?;

        client
            .execute(
                "INSERT INTO uma_delegation_policies (id, owner_id, resource_id, scopes, delegates, conditions, enabled, valid_from, valid_until, realm_id, created_at, updated_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW(), NOW())",
                &[
                    &policy.id,
                    &policy.owner_id,
                    &policy.resource_id.as_ref().map(|s| Uuid::parse_str(s).ok()).flatten(),
                    &policy.scopes,
                    &policy.delegates,
                    &serde_json::to_value(&policy.conditions).map_err(|e| {
                        AuthencError::internal(format!("Failed to serialize conditions: {}", e))
                    })?,
                    &policy.enabled,
                    &policy.valid_from.map(|ts| chrono::DateTime::from_timestamp(ts, 0).unwrap()),
                    &policy.valid_until.map(|ts| chrono::DateTime::from_timestamp(ts, 0).unwrap()),
                    &Uuid::parse_str(&policy.realm_id).map_err(|_| AuthencError::validation("Invalid realm_id".to_string()))?,
                ],
            )
            .await
            .map_err(|e| AuthencError::database(e.to_string()))?;

        Ok(())
    }

    /// Get delegation policies for an owner
    pub async fn get_delegation_policies_by_owner(
        &self,
        owner_id: &str,
        realm_id: &str,
    ) -> Result<Vec<DelegationPolicy>> {
        let client = self.database.get_connection().await?;

        let realm_uuid = Uuid::parse_str(realm_id)
            .map_err(|_| AuthencError::validation("Invalid realm_id".to_string()))?;

        let rows = client
            .query(
                "SELECT id, owner_id, resource_id, scopes, delegates, conditions, enabled, valid_from, valid_until, realm_id
                 FROM uma_delegation_policies
                 WHERE owner_id = $1 AND realm_id = $2",
                &[&owner_id, &realm_uuid],
            )
            .await
            .map_err(|e| AuthencError::database(e.to_string()))?;

        let policies: Vec<DelegationPolicy> = rows
            .iter()
            .filter_map(|row| {
                Some(DelegationPolicy {
                    id: row.get(0),
                    owner_id: row.get(1),
                    resource_id: row.get::<_, Option<Uuid>>(2).map(|u| u.to_string()),
                    scopes: row.get(3),
                    delegates: row.get(4),
                    conditions: serde_json::from_value(row.get(5)).ok()?,
                    enabled: row.get(6),
                    valid_from: row
                        .get::<_, Option<chrono::DateTime<chrono::Utc>>>(7)
                        .map(|dt| dt.timestamp()),
                    valid_until: row
                        .get::<_, Option<chrono::DateTime<chrono::Utc>>>(8)
                        .map(|dt| dt.timestamp()),
                    realm_id: row.get::<_, Uuid>(9).to_string(),
                })
            })
            .collect();

        Ok(policies)
    }

    /// Delete a delegation policy
    pub async fn delete_delegation_policy(&self, policy_id: &Uuid) -> Result<()> {
        let client = self.database.get_connection().await?;

        client
            .execute(
                "DELETE FROM uma_delegation_policies WHERE id = $1",
                &[policy_id],
            )
            .await
            .map_err(|e| AuthencError::database(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    #[ignore = "Requires database"]
    async fn test_policy_store_operations() {
        // Test would require database setup
        // This is a placeholder for integration tests
    }
}
