use crate::error::CoreError;
use crate::models::PolicyRule;
use anyhow::Result;
use chrono::{DateTime, Utc};
use secreton_storage::{QueryParams, SecretEntry, SecurityLevel, StorageBackend};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

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

#[derive(Clone)]
struct CachedPolicy {
    definition: PolicyDefinition,
    fetched_at: Instant,
}

pub struct PolicyService {
    storage: Arc<dyn StorageBackend + Send + Sync>,
    cache: RwLock<HashMap<String, CachedPolicy>>, // Key: "{namespace}:{name}"
}

const CACHE_TTL: Duration = Duration::from_secs(60);

impl PolicyService {
    pub fn new(storage: Arc<dyn StorageBackend + Send + Sync>) -> Self {
        Self {
            storage,
            cache: RwLock::new(HashMap::new()),
        }
    }

    fn cache_key(namespace: &str, name: &str) -> String {
        format!("{}:{}", namespace, name)
    }

    /// Aggregate rules from a list of policy names
    pub async fn get_rules_for_policies(
        &self,
        names: &[String],
        namespace: &str,
    ) -> Result<Vec<PolicyRule>, CoreError> {
        let mut rules = Vec::new();

        for name in names {
            let key = Self::cache_key(namespace, name);

            // Check cache first
            let cached_policy = match self.cache.read() {
                Ok(cache) => cache.get(&key).cloned(),
                Err(poisoned) => {
                    // Handle poisoned lock by clearing it (if possible) or just ignoring cache
                    // RwLockReadGuard from poisoned lock can be accessed via into_inner(),
                    // but that might return a guard to potentially inconsistent data.
                    // For a cache, it might be safer to ignore it or clear it.
                    // Here we try to recover data but log warning.
                    tracing::warn!("Policy cache lock poisoned, attempting recovery");
                    poisoned.into_inner().get(&key).cloned()
                }
            };

            if let Some(entry) = cached_policy {
                if entry.fetched_at.elapsed() < CACHE_TTL {
                    if entry.definition.is_active {
                        rules.extend(entry.definition.rules);
                    }
                    continue;
                }
            }

            // Fallback to storage (not in cache or expired)
            match self.get_policy(name).await {
                Ok(policy) => {
                    // Validate namespace matches
                    if policy.namespace != namespace {
                        continue;
                    }

                    if policy.is_active {
                        // Update cache
                        if let Ok(mut cache) = self.cache.write() {
                            cache.insert(
                                key,
                                CachedPolicy {
                                    definition: policy.clone(),
                                    fetched_at: Instant::now(),
                                },
                            );
                        }
                        rules.extend(policy.rules);
                    }
                }
                Err(CoreError::NotFound { .. }) => {
                    // Ignore missing policies, but log if needed
                    continue;
                }
                Err(e) => {
                    // Propagate other errors (e.g. storage failure)
                    return Err(e);
                }
            }
        }
        Ok(rules)
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
        if self
            .storage
            .exists(&path)
            .await
            .map_err(|e| CoreError::Internal {
                message: e.to_string(),
                source: None,
            })?
        {
            return Err(CoreError::AlreadyExists {
                resource: format!("Policy {}", name),
            });
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
        let entry = self
            .storage
            .get_by_path(&path)
            .await
            .map_err(|e| CoreError::Internal {
                message: e.to_string(),
                source: None,
            })?;

        if let Some(entry) = entry {
            let policy: PolicyDefinition =
                serde_json::from_slice(&entry.encrypted_data).map_err(|e| CoreError::Internal {
                    message: format!("Failed to deserialize policy: {}", e),
                    source: None,
                })?;
            Ok(policy)
        } else {
            Err(CoreError::NotFound {
                resource: format!("Policy {}", name),
            })
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
        // We need the policy to know the namespace for cache invalidation
        let policy = self.get_policy(name).await?;
        let key = Self::cache_key(&policy.namespace, name);

        let path = format!("sys/policies/{}", name);
        let deleted =
            self.storage
                .delete_by_path(&path)
                .await
                .map_err(|e| CoreError::Internal {
                    message: e.to_string(),
                    source: None,
                })?;

        if deleted {
            // Remove from cache
            if let Ok(mut cache) = self.cache.write() {
                cache.remove(&key);
            }
            Ok(())
        } else {
            Err(CoreError::NotFound {
                resource: format!("Policy {}", name),
            })
        }
    }

    pub async fn list_policies(
        &self,
        namespace: Option<String>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<(Vec<PolicyDefinition>, u64), CoreError> {
        let params = QueryParams::new().with_path_prefix("sys/policies/".to_string());
        // StorageBackend list returns EngineEntries, we need to deserialize them.
        // And pagination/filtering might need to happen in memory if storage doesn't support deep query.

        // Basic implementation: list all, filter/paginate in memory
        // Optimization: storage.list usually returns metadata/entries.

        let entries = self
            .storage
            .list(&params)
            .await
            .map_err(|e| CoreError::Internal {
                message: e.to_string(),
                source: None,
            })?;

        let mut policies = Vec::new();
        for entry in entries {
            let policy: PolicyDefinition =
                serde_json::from_slice(&entry.encrypted_data).map_err(|e| CoreError::Internal {
                    message: format!("Failed to deserialize policy: {}", e),
                    source: None,
                })?;

            if let Some(ns) = &namespace
                && &policy.namespace != ns
            {
                continue;
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
        let data = serde_json::to_vec(policy).map_err(|e| CoreError::Internal {
            message: e.to_string(),
            source: None,
        })?;

        let entry = SecretEntry::new(
            path,
            data,
            serde_json::json!({
                "type": "policy",
                "version": policy.version
            }),
            SecurityLevel::Internal, // Policies are internal config
            policy.created_by.clone(),
        );

        self.storage
            .store(&entry)
            .await
            .map_err(|e| CoreError::Internal {
                message: e.to_string(),
                source: None,
            })?;

        // Update cache
        let key = Self::cache_key(&policy.namespace, &policy.name);
        if let Ok(mut cache) = self.cache.write() {
            cache.insert(
                key,
                CachedPolicy {
                    definition: policy.clone(),
                    fetched_at: Instant::now(),
                },
            );
        }

        Ok(())
    }
}
