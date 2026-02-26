//! Policy Storage Extension for Raft
//!
//! This module extends the Raft state machine to support policy storage
//! for high-availability policy management following Vault best practices.
//!
//! # Design
//!
//! Policies are stored in Raft for:
//! - Strong consistency across cluster nodes
//! - Immediate availability after leader election
//! - Atomic policy updates
//!
//! Policy definitions (TOML) are parsed and stored as structured data.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

// ============================================================================
// POLICY ENTRY TYPES
// ============================================================================

/// Policy entry stored in Raft
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyEntry {
    /// Unique identifier
    pub id: Uuid,
    /// Policy name (unique key)
    pub name: String,
    /// Namespace for multi-tenancy
    pub namespace: String,
    /// Policy description
    pub description: Option<String>,
    /// Policy rules
    pub rules: Vec<PolicyRuleEntry>,
    /// Policy metadata
    pub metadata: serde_json::Value,
    /// When the policy was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// When the policy was last updated
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Version number for optimistic locking
    pub version: u64,
}

/// Policy rule entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRuleEntry {
    /// Rule effect: allow or deny
    pub effect: PolicyEffect,
    /// Path pattern (glob)
    pub path: String,
    /// Capabilities granted/denied
    pub capabilities: Vec<String>,
    /// MFA requirement
    pub mfa_required: bool,
    /// Time-based conditions
    pub time_constraints: Option<TimeConstraints>,
    /// IP-based conditions
    pub allowed_ips: Option<Vec<String>>,
    /// Required request parameters
    pub required_params: serde_json::Value,
}

/// Policy effect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PolicyEffect {
    Allow,
    Deny,
}

/// Time constraints for policy rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeConstraints {
    /// Start time (HH:MM format)
    pub start_time: Option<String>,
    /// End time (HH:MM format)
    pub end_time: Option<String>,
    /// Allowed days (0=Sunday, 6=Saturday)
    pub allowed_days: Option<Vec<u8>>,
    /// Timezone
    pub timezone: Option<String>,
}

// ============================================================================
// RAFT COMMANDS FOR POLICIES
// ============================================================================

/// Commands for policy operations in Raft
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyCommand {
    /// Store a new policy
    Store(PolicyEntry),

    /// Update an existing policy
    Update(PolicyEntry),

    /// Delete a policy by name and namespace
    Delete { name: String, namespace: String },

    /// Assign policy to user/token
    AssignPolicy {
        policy_name: String,
        namespace: String,
        target_type: PolicyTargetType,
        target_id: String,
    },

    /// Unassign policy from user/token
    UnassignPolicy {
        policy_name: String,
        namespace: String,
        target_type: PolicyTargetType,
        target_id: String,
    },
}

/// Types of targets that can have policies assigned
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PolicyTargetType {
    User,
    Token,
    Role,
    Group,
    AppRole,
}

/// Response from policy operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyResponse {
    /// Policy was stored
    Stored(Uuid),

    /// Policy was updated
    Updated(Uuid),

    /// Policy was deleted
    Deleted(bool),

    /// Policy was assigned
    Assigned,

    /// Policy was unassigned
    Unassigned,

    /// Error occurred
    Error(String),
}

// ============================================================================
// POLICY STATE MACHINE
// ============================================================================

/// Assignment record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyAssignment {
    pub policy_name: String,
    pub namespace: String,
    pub target_type: PolicyTargetType,
    pub target_id: String,
    pub assigned_at: chrono::DateTime<chrono::Utc>,
}

/// Policy state machine for Raft
pub struct PolicyStateMachine {
    /// Policies indexed by namespace/name
    policies: Arc<RwLock<HashMap<String, PolicyEntry>>>,

    /// Policy assignments indexed by target
    assignments: Arc<RwLock<HashMap<String, Vec<PolicyAssignment>>>>,
}

impl PolicyStateMachine {
    /// Create a new policy state machine
    pub fn new() -> Self {
        Self {
            policies: Arc::new(RwLock::new(HashMap::new())),
            assignments: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate policy key
    fn policy_key(namespace: &str, name: &str) -> String {
        format!("{}:{}", namespace, name)
    }

    /// Generate target key
    fn target_key(target_type: &PolicyTargetType, target_id: &str) -> String {
        let type_str = match target_type {
            PolicyTargetType::User => "user",
            PolicyTargetType::Token => "token",
            PolicyTargetType::Role => "role",
            PolicyTargetType::Group => "group",
            PolicyTargetType::AppRole => "approle",
        };
        format!("{}:{}", type_str, target_id)
    }

    /// Apply a policy command
    pub async fn apply_command(&self, cmd: PolicyCommand) -> PolicyResponse {
        match cmd {
            PolicyCommand::Store(entry) => {
                let mut policies = self.policies.write().await;
                let key = Self::policy_key(&entry.namespace, &entry.name);
                let id = entry.id;
                policies.insert(key, entry);
                PolicyResponse::Stored(id)
            }

            PolicyCommand::Update(entry) => {
                let mut policies = self.policies.write().await;
                let key = Self::policy_key(&entry.namespace, &entry.name);
                let id = entry.id;

                if policies.contains_key(&key) {
                    policies.insert(key, entry);
                    PolicyResponse::Updated(id)
                } else {
                    PolicyResponse::Error("Policy not found".to_string())
                }
            }

            PolicyCommand::Delete { name, namespace } => {
                let mut policies = self.policies.write().await;
                let key = Self::policy_key(&namespace, &name);

                if policies.remove(&key).is_some() {
                    // Also remove all assignments for this policy
                    let mut assignments = self.assignments.write().await;
                    for target_assignments in assignments.values_mut() {
                        target_assignments
                            .retain(|a| !(a.policy_name == name && a.namespace == namespace));
                    }
                    PolicyResponse::Deleted(true)
                } else {
                    PolicyResponse::Deleted(false)
                }
            }

            PolicyCommand::AssignPolicy {
                policy_name,
                namespace,
                target_type,
                target_id,
            } => {
                // Verify policy exists
                let policies = self.policies.read().await;
                let policy_key = Self::policy_key(&namespace, &policy_name);

                if !policies.contains_key(&policy_key) {
                    return PolicyResponse::Error("Policy not found".to_string());
                }
                drop(policies);

                let mut assignments = self.assignments.write().await;
                let target_key = Self::target_key(&target_type, &target_id);

                let target_list = assignments.entry(target_key).or_insert_with(Vec::new);

                // Check if already assigned
                if target_list
                    .iter()
                    .any(|a| a.policy_name == policy_name && a.namespace == namespace)
                {
                    return PolicyResponse::Error("Policy already assigned".to_string());
                }

                target_list.push(PolicyAssignment {
                    policy_name,
                    namespace,
                    target_type,
                    target_id,
                    assigned_at: chrono::Utc::now(),
                });

                PolicyResponse::Assigned
            }

            PolicyCommand::UnassignPolicy {
                policy_name,
                namespace,
                target_type,
                target_id,
            } => {
                let mut assignments = self.assignments.write().await;
                let target_key = Self::target_key(&target_type, &target_id);

                if let Some(target_list) = assignments.get_mut(&target_key) {
                    let before_len = target_list.len();
                    target_list
                        .retain(|a| !(a.policy_name == policy_name && a.namespace == namespace));

                    if target_list.len() < before_len {
                        PolicyResponse::Unassigned
                    } else {
                        PolicyResponse::Error("Assignment not found".to_string())
                    }
                } else {
                    PolicyResponse::Error("No assignments for target".to_string())
                }
            }
        }
    }

    /// Get a policy by name and namespace
    pub async fn get_policy(&self, namespace: &str, name: &str) -> Option<PolicyEntry> {
        let policies = self.policies.read().await;
        let key = Self::policy_key(namespace, name);
        policies.get(&key).cloned()
    }

    /// List all policies in a namespace
    pub async fn list_policies(&self, namespace: &str) -> Vec<PolicyEntry> {
        let policies = self.policies.read().await;
        let prefix = format!("{}:", namespace);

        policies
            .iter()
            .filter(|(k, _)| k.starts_with(&prefix))
            .map(|(_, v)| v.clone())
            .collect()
    }

    /// Get policies assigned to a target
    pub async fn get_assigned_policies(
        &self,
        target_type: &PolicyTargetType,
        target_id: &str,
    ) -> Vec<PolicyEntry> {
        let assignments = self.assignments.read().await;
        let target_key = Self::target_key(target_type, target_id);

        let Some(target_assignments) = assignments.get(&target_key) else {
            return Vec::new();
        };

        let policies = self.policies.read().await;

        target_assignments
            .iter()
            .filter_map(|a| {
                let policy_key = Self::policy_key(&a.namespace, &a.policy_name);
                policies.get(&policy_key).cloned()
            })
            .collect()
    }

    /// Check if path access is allowed for given policies
    pub async fn check_path_access(
        &self,
        policy_names: &[String],
        namespace: &str,
        path: &str,
        capability: &str,
    ) -> (bool, Option<String>) {
        let policies = self.policies.read().await;

        // Collect matching rules from all policies
        let mut deny_match: Option<String> = None;
        let mut allow_match: Option<String> = None;

        for policy_name in policy_names {
            let key = Self::policy_key(namespace, policy_name);

            if let Some(policy) = policies.get(&key) {
                for rule in &policy.rules {
                    if self.path_matches(&rule.path, path) {
                        let has_capability = rule
                            .capabilities
                            .iter()
                            .any(|c| c == "*" || c == capability);

                        if has_capability {
                            match rule.effect {
                                PolicyEffect::Deny => {
                                    // Deny takes precedence
                                    deny_match = Some(policy_name.clone());
                                }
                                PolicyEffect::Allow => {
                                    allow_match = Some(policy_name.clone());
                                }
                            }
                        }
                    }
                }
            }
        }

        // Deny takes precedence over allow
        if deny_match.is_some() {
            (false, deny_match)
        } else if allow_match.is_some() {
            (true, allow_match)
        } else {
            (false, None)
        }
    }

    /// Check if a path matches a pattern
    fn path_matches(&self, pattern: &str, path: &str) -> bool {
        if pattern == "*" || pattern == "/*" {
            return true;
        }

        // Handle ** for recursive matching
        if pattern.contains("**") {
            let parts: Vec<&str> = pattern.split("**").collect();
            if parts.len() == 2 {
                let prefix = parts[0].trim_end_matches('/');
                let suffix = parts[1].trim_start_matches('/');

                if !path.starts_with(prefix) {
                    return false;
                }

                if suffix.is_empty() {
                    return true;
                }

                return path.ends_with(suffix);
            }
        }

        // Handle single * for segment matching
        if pattern.ends_with("/*") {
            let prefix = &pattern[..pattern.len() - 1];
            return path.starts_with(prefix);
        }

        if pattern.ends_with('*') {
            let prefix = &pattern[..pattern.len() - 1];
            return path.starts_with(prefix);
        }

        pattern == path
    }

    /// Get snapshot of policy state
    pub async fn get_snapshot(&self) -> PolicySnapshot {
        PolicySnapshot {
            policies: self.policies.read().await.clone(),
            assignments: self.assignments.read().await.clone(),
        }
    }

    /// Restore from snapshot
    pub async fn restore_snapshot(&self, snapshot: PolicySnapshot) {
        *self.policies.write().await = snapshot.policies;
        *self.assignments.write().await = snapshot.assignments;
    }
}

impl Default for PolicyStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for PolicyStateMachine {
    fn clone(&self) -> Self {
        Self {
            policies: Arc::clone(&self.policies),
            assignments: Arc::clone(&self.assignments),
        }
    }
}

/// Snapshot of policy state for Raft
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicySnapshot {
    pub policies: HashMap<String, PolicyEntry>,
    pub assignments: HashMap<String, Vec<PolicyAssignment>>,
}

// ============================================================================
// TOML POLICY PARSER INTEGRATION
// ============================================================================

/// Parse a TOML policy string into a PolicyEntry
pub fn parse_toml_policy(toml_content: &str, namespace: &str) -> Result<PolicyEntry, String> {
    #[derive(Deserialize)]
    struct TomlPolicy {
        name: String,
        description: Option<String>,
        #[serde(default)]
        rules: Vec<TomlRule>,
    }

    #[derive(Clone, Deserialize)]
    struct TomlRule {
        effect: String,
        path: String,
        capabilities: Vec<String>,
        #[serde(default)]
        mfa: Option<bool>,
        #[serde(default)]
        condition: Option<TomlCondition>,
    }

    #[derive(Clone, Deserialize)]
    struct TomlCondition {
        time_range: Option<TomlTimeRange>,
        allowed_ips: Option<Vec<String>>,
    }

    #[derive(Clone, Deserialize)]
    struct TomlTimeRange {
        start: Option<String>,
        end: Option<String>,
        days: Option<Vec<u8>>,
        timezone: Option<String>,
    }

    let parsed: TomlPolicy =
        toml::from_str(toml_content).map_err(|e| format!("TOML parse error: {}", e))?;

    let rules = parsed
        .rules
        .into_iter()
        .map(|r| {
            let effect = match r.effect.to_lowercase().as_str() {
                "allow" => PolicyEffect::Allow,
                "deny" => PolicyEffect::Deny,
                _ => return Err(format!("Invalid effect: {}", r.effect)),
            };

            let time_constraints = r.condition.clone().and_then(|c| {
                c.time_range.map(|t| TimeConstraints {
                    start_time: t.start,
                    end_time: t.end,
                    allowed_days: t.days,
                    timezone: t.timezone,
                })
            });

            let allowed_ips = r.condition.and_then(|c| c.allowed_ips);

            Ok(PolicyRuleEntry {
                effect,
                path: r.path,
                capabilities: r.capabilities,
                mfa_required: r.mfa.unwrap_or(false),
                time_constraints,
                allowed_ips,
                required_params: serde_json::Value::Null,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    let now = chrono::Utc::now();

    Ok(PolicyEntry {
        id: Uuid::new_v4(),
        name: parsed.name,
        namespace: namespace.to_string(),
        description: parsed.description,
        rules,
        metadata: serde_json::Value::Object(serde_json::Map::new()),
        created_at: now,
        updated_at: now,
        version: 1,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_policy_store_and_retrieve() {
        let sm = PolicyStateMachine::new();

        let policy = PolicyEntry {
            id: Uuid::new_v4(),
            name: "test-policy".to_string(),
            namespace: "default".to_string(),
            description: Some("Test policy".to_string()),
            rules: vec![PolicyRuleEntry {
                effect: PolicyEffect::Allow,
                path: "secret/data/*".to_string(),
                capabilities: vec!["read".to_string(), "list".to_string()],
                mfa_required: false,
                time_constraints: None,
                allowed_ips: None,
                required_params: serde_json::Value::Null,
            }],
            metadata: serde_json::Value::Null,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            version: 1,
        };

        let response = sm.apply_command(PolicyCommand::Store(policy.clone())).await;
        assert!(matches!(response, PolicyResponse::Stored(_)));

        let retrieved = sm.get_policy("default", "test-policy").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test-policy");
    }

    #[tokio::test]
    async fn test_path_access_check() {
        let sm = PolicyStateMachine::new();

        let policy = PolicyEntry {
            id: Uuid::new_v4(),
            name: "read-policy".to_string(),
            namespace: "default".to_string(),
            description: None,
            rules: vec![
                PolicyRuleEntry {
                    effect: PolicyEffect::Allow,
                    path: "secret/data/*".to_string(),
                    capabilities: vec!["read".to_string()],
                    mfa_required: false,
                    time_constraints: None,
                    allowed_ips: None,
                    required_params: serde_json::Value::Null,
                },
                PolicyRuleEntry {
                    effect: PolicyEffect::Deny,
                    path: "secret/data/admin/*".to_string(),
                    capabilities: vec!["*".to_string()],
                    mfa_required: false,
                    time_constraints: None,
                    allowed_ips: None,
                    required_params: serde_json::Value::Null,
                },
            ],
            metadata: serde_json::Value::Null,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            version: 1,
        };

        sm.apply_command(PolicyCommand::Store(policy)).await;

        // Should allow read on normal path
        let (allowed, _) = sm
            .check_path_access(
                &["read-policy".to_string()],
                "default",
                "secret/data/app",
                "read",
            )
            .await;
        assert!(allowed);

        // Should deny on admin path
        let (allowed, _) = sm
            .check_path_access(
                &["read-policy".to_string()],
                "default",
                "secret/data/admin/config",
                "read",
            )
            .await;
        assert!(!allowed);
    }

    #[test]
    fn test_toml_parsing() {
        let toml = r#"
name = "test-policy"
description = "A test policy"

[[rules]]
effect = "allow"
path = "secret/data/*"
capabilities = ["read", "list"]

[[rules]]
effect = "deny"
path = "secret/data/admin/*"
capabilities = ["*"]
mfa = true
"#;

        let result = parse_toml_policy(toml, "default");
        assert!(result.is_ok());

        let policy = result.unwrap();
        assert_eq!(policy.name, "test-policy");
        assert_eq!(policy.rules.len(), 2);
        assert_eq!(policy.rules[0].effect, PolicyEffect::Allow);
        assert!(policy.rules[1].mfa_required);
    }
}
