use std::sync::Arc;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use secreton_storage::{StorageBackend, VaultEntry, SecurityLevel, QueryParams};
use crate::models::{PolicyRule, ControlGroup};
use crate::error::CoreError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDefinition {
    pub name: String,
    pub description: Option<String>,
    pub rules: Vec<PolicyRule>,
    pub namespace: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String,
    pub updated_by: Option<String>,
    pub version: u32,
    pub is_active: bool,
}

pub struct PolicyService {
    storage: Arc<dyn StorageBackend + Send + Sync>,
}

impl PolicyService {
    pub fn new(storage: Arc<dyn StorageBackend + Send + Sync>) -> Self {
        Self { storage }
    }

    pub async fn create_policy(
        &self,
        name: String,
        namespace: String,
        description: Option<String>,
        rules: Vec<PolicyRule>,
        created_by: String,
    ) -> Result<PolicyDefinition, CoreError> {
        let path = format!("sys/policies/{}", name);

        // Check if exists
        if self.storage.exists(&path).await.map_err(|e| CoreError::Internal { message: e.to_string(), source: None })? {
            return Err(CoreError::AlreadyExists { resource: format!("Policy {}", name) });
        }

        let now = Utc::now();
        let policy = PolicyDefinition {
            name: name.clone(),
            description,
            rules,
            namespace,
            created_at: now,
            updated_at: now,
            created_by: created_by.clone(),
            updated_by: None,
            version: 1,
            is_active: true,
        };

        self.save_policy(&policy).await?;
        Ok(policy)
    }

    pub async fn get_policy(&self, name: &str) -> Result<PolicyDefinition, CoreError> {
        let path = format!("sys/policies/{}", name);
        let entry = self.storage.get_by_path(&path).await.map_err(|e| CoreError::Internal { message: e.to_string(), source: None })?;

        if let Some(entry) = entry {
            let policy: PolicyDefinition = serde_json::from_slice(&entry.encrypted_data)
                .map_err(|e| CoreError::Internal { message: format!("Failed to deserialize policy: {}", e), source: None })?;
            Ok(policy)
        } else {
            Err(CoreError::NotFound { resource: format!("Policy {}", name) })
        }
    }

    pub async fn update_policy(
        &self,
        name: &str,
        description: Option<String>,
        rules: Option<Vec<PolicyRule>>,
        is_active: Option<bool>,
        updated_by: String,
    ) -> Result<PolicyDefinition, CoreError> {
        let mut policy = self.get_policy(name).await?;

        if let Some(desc) = description {
            policy.description = Some(desc);
        }
        if let Some(r) = rules {
            policy.rules = r;
        }
        if let Some(active) = is_active {
            policy.is_active = active;
        }

        policy.updated_at = Utc::now();
        policy.updated_by = Some(updated_by);
        policy.version += 1;

        self.save_policy(&policy).await?;
        Ok(policy)
    }

    pub async fn delete_policy(&self, name: &str) -> Result<(), CoreError> {
        let path = format!("sys/policies/{}", name);
        let deleted = self.storage.delete_by_path(&path).await.map_err(|e| CoreError::Internal { message: e.to_string(), source: None })?;

        if deleted {
            Ok(())
        } else {
            Err(CoreError::NotFound { resource: format!("Policy {}", name) })
        }
    }

    pub async fn list_policies(
        &self,
        namespace: Option<String>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<(Vec<PolicyDefinition>, u64), CoreError> {
        let mut params = QueryParams::new().with_path_prefix("sys/policies/".to_string());
        // StorageBackend list returns VaultEntries, we need to deserialize them.
        // And pagination/filtering might need to happen in memory if storage doesn't support deep query.

        // Basic implementation: list all, filter/paginate in memory
        // Optimization: storage.list usually returns metadata/entries.

        let entries = self.storage.list(&params).await.map_err(|e| CoreError::Internal { message: e.to_string(), source: None })?;

        let mut policies = Vec::new();
        for entry in entries {
             let policy: PolicyDefinition = serde_json::from_slice(&entry.encrypted_data)
                .map_err(|e| CoreError::Internal { message: format!("Failed to deserialize policy: {}", e), source: None })?;

            if let Some(ns) = &namespace {
                if &policy.namespace != ns {
                    continue;
                }
            }

            policies.push(policy);
        }

        let total = policies.len() as u64;

        // Pagination
        let start = offset.unwrap_or(0) as usize;
        let end = if let Some(l) = limit {
            (start + l as usize).min(policies.len())
        } else {
            policies.len()
        };

        if start >= policies.len() {
            Ok((Vec::new(), total))
        } else {
            Ok((policies[start..end].to_vec(), total))
        }
    }

    async fn save_policy(&self, policy: &PolicyDefinition) -> Result<(), CoreError> {
        let path = format!("sys/policies/{}", policy.name);
        let data = serde_json::to_vec(policy).map_err(|e| CoreError::Internal { message: e.to_string(), source: None })?;

        let entry = VaultEntry::new(
            path,
            data,
            serde_json::json!({
                "type": "policy",
                "version": policy.version
            }),
            SecurityLevel::Internal, // Policies are internal config
            policy.created_by.clone(),
        );

        self.storage.store(&entry).await.map_err(|e| CoreError::Internal { message: e.to_string(), source: None })?;
        Ok(())
    }
}
