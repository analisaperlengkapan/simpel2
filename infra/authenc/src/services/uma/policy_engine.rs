//! UMA 2.0 Policy Evaluation Engine
//!
//! Advanced policy engine supporting:
//! - Attribute-Based Access Control (ABAC)
//! - Role-Based Access Control (RBAC)
//! - Time-based policies
//! - Context-aware authorization
//! - JavaScript policy execution
//! - Policy aggregation and combination

use crate::database::Database;
use crate::error::Result;
use crate::services::resource_store::ResourceStoreTrait;
use chrono::{Datelike, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Policy evaluation context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyEvaluationContext {
    /// Subject (requesting party)
    pub subject: SubjectContext,
    /// Resource being accessed
    pub resource: ResourceContext,
    /// Action being performed
    pub action: String,
    /// Environment context
    pub environment: EnvironmentContext,
    /// Claims provided by requesting party
    pub claims: HashMap<String, serde_json::Value>,
}

/// Subject context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectContext {
    /// User ID
    pub user_id: String,
    /// Roles
    pub roles: Vec<String>,
    /// Attributes
    pub attributes: HashMap<String, serde_json::Value>,
    /// Groups
    pub groups: Vec<String>,
}

/// Resource context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContext {
    /// Resource ID
    pub id: String,
    /// Resource type
    pub resource_type: Option<String>,
    /// Resource owner
    pub owner: String,
    /// Resource attributes
    pub attributes: HashMap<String, Vec<String>>,
    /// Resource scopes
    pub scopes: Vec<String>,
}

/// Environment context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentContext {
    /// IP address
    pub ip_address: Option<String>,
    /// Time of request
    pub time: i64,
    /// User agent
    pub user_agent: Option<String>,
    /// Device fingerprint
    pub device_id: Option<String>,
    /// Geographic location
    pub location: Option<String>,
    /// MFA completed
    pub mfa_completed: bool,
    /// Trust score
    pub trust_score: Option<f64>,
}

/// Policy decision
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyDecision {
    Permit,
    /// Access denied
    Deny,
    /// Not applicable
    NotApplicable,
    /// Need more information (claims gathering)
    NeedInfo,
}

/// Policy evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
    /// Access granted
pub struct PolicyEvaluationResult {
    /// Final decision
    pub decision: PolicyDecision,
    /// Obligations (actions that must be performed)
    pub obligations: Vec<Obligation>,
    /// Advice (optional actions)
    pub advice: Vec<Advice>,
    /// Reason for decision
    pub reason: Option<String>,
    /// Policies that were evaluated
    pub evaluated_policies: Vec<String>,
    /// Missing claims if decision is NeedInfo
    pub required_claims: Option<Vec<super::ClaimRequirement>>,
}

/// Obligation - action that MUST be performed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Obligation {
    /// Obligation ID
    pub id: String,
    /// Action to perform
    pub action: String,
    /// Parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Advice - action that SHOULD be performed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Advice {
    /// Advice ID
    pub id: String,
    /// Action to perform
    pub action: String,
    /// Parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Policy definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UmaPolicy {
    /// Policy ID
    pub id: Uuid,
    /// Policy name
    pub name: String,
    /// Policy description
    pub description: Option<String>,
    /// Policy type
    pub policy_type: PolicyType,
    /// Logic (Positive/Negative)
    pub logic: Logic,
    /// Decision strategy
    pub decision_strategy: DecisionStrategy,
    /// Policy configuration
    pub config: PolicyConfig,
    /// Resources this policy applies to
    pub resources: Vec<String>,
    /// Scopes this policy applies to
    pub scopes: Vec<String>,
    /// Clients this policy applies to (optional)
    pub clients: Option<Vec<String>>,
    /// Whether policy is enabled
    pub enabled: bool,
}

/// Policy type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PolicyType {
    Role,
    /// User-based policy
    User,
    /// Group-based policy
    Group,
    /// Time-based policy
    Time,
    /// Attribute-based policy (ABAC)
    Attribute,
    /// JavaScript policy
    JavaScript,
    /// Aggregate policy
    Aggregate,
    /// Client-based policy
    Client,
}

/// Logic type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
    /// Role-based policy
pub enum Logic {
    Positive,
    /// Negative logic - deny if conditions met
    Negative,
}

/// Decision strategy for combining policies
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
    /// Positive logic - grant if conditions met
pub enum DecisionStrategy {
    Unanimous,
    /// At least one policy must permit
    Affirmative,
    /// Consensus (majority)
    Consensus,
}

/// Policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
    /// All policies must permit
pub struct PolicyConfig {
    /// Roles required (for role policy)
    #[serde(default)]
    pub roles: Vec<String>,
    /// Users allowed (for user policy)
    #[serde(default)]
    pub users: Vec<String>,
    /// Groups allowed (for group policy)
    #[serde(default)]
    pub groups: Vec<String>,
    /// Time constraints (for time policy)
    pub time_config: Option<TimeConfig>,
    /// Attribute conditions (for ABAC)
    #[serde(default)]
    pub attributes: HashMap<String, AttributeCondition>,
    /// JavaScript code (for JS policy)
    pub code: Option<String>,
    /// Aggregated policies (for aggregate policy)
    #[serde(default)]
    pub policies: Vec<String>,
}

/// Time configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeConfig {
    /// Not before (hour 0-23)
    pub not_before: Option<i32>,
    /// Not after (hour 0-23)
    pub not_after: Option<i32>,
    /// Days of week (0=Sunday, 6=Saturday)
    pub days_of_week: Option<Vec<i32>>,
    /// Start date (ISO 8601)
    pub start_date: Option<String>,
    /// End date (ISO 8601)
    pub end_date: Option<String>,
}

/// Attribute condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeCondition {
    /// Attribute name
    pub name: String,
    /// Operator (eq, ne, gt, lt, gte, lte, in, contains)
    pub operator: String,
    /// Expected value
    pub value: serde_json::Value,
}

/// Policy engine for evaluating policies
pub struct PolicyEngine {
    db: Arc<Database>,
    /// Resource store
    resource_store: Arc<dyn ResourceStoreTrait>,
}

impl PolicyEngine {
    /// Create new policy engine
    /// Database connection
    pub fn new(db: Arc<Database>, resource_store: Arc<dyn ResourceStoreTrait>) -> Self {
        Self { db, resource_store }
    }

    /// Evaluate policies for permission request
    pub async fn evaluate_policies(
        &self,
        context: &PolicyEvaluationContext,
        policies: &[UmaPolicy],
    ) -> Result<PolicyEvaluationResult> {
        let mut decisions = Vec::new();
        let mut evaluated_policies = Vec::new();
        let all_obligations = Vec::new();
        let all_advice = Vec::new();

        for policy in policies {
            if !policy.enabled {
                continue;
            }

            let decision = self.evaluate_single_policy(context, policy).await?;
            decisions.push((policy.clone(), decision));
            evaluated_policies.push(policy.name.clone());

            // Collect obligations and advice
            // In real implementation, these would come from policy config
        }

        // Apply decision strategy
        let final_decision = self.combine_decisions(&decisions, DecisionStrategy::Affirmative);

        Ok(PolicyEvaluationResult {
            decision: final_decision,
            obligations: all_obligations,
            advice: all_advice,
            reason: None,
            evaluated_policies,
            required_claims: None,
        })
    }

    /// Evaluate single policy
    async fn evaluate_single_policy(
        &self,
        context: &PolicyEvaluationContext,
        policy: &UmaPolicy,
    ) -> Result<PolicyDecision> {
        let decision = match policy.policy_type {
            PolicyType::Role => self.evaluate_role_policy(context, &policy.config),
            PolicyType::User => self.evaluate_user_policy(context, &policy.config),
            PolicyType::Group => self.evaluate_group_policy(context, &policy.config),
            PolicyType::Time => self.evaluate_time_policy(context, &policy.config),
            PolicyType::Attribute => self.evaluate_attribute_policy(context, &policy.config),
            PolicyType::JavaScript => {
                self.evaluate_javascript_policy(context, &policy.config)
                    .await?
            }
            PolicyType::Aggregate => {
                self.evaluate_aggregate_policy(context, &policy.config)
                    .await?
            }
            PolicyType::Client => PolicyDecision::NotApplicable,
        };

        // Apply logic (Positive or Negative)
        Ok(match policy.logic {
            Logic::Positive => decision,
            Logic::Negative => match decision {
                PolicyDecision::Permit => PolicyDecision::Deny,
                PolicyDecision::Deny => PolicyDecision::Permit,
                other => other,
            },
        })
    }

    /// Evaluate role-based policy
    fn evaluate_role_policy(
        &self,
        context: &PolicyEvaluationContext,
        config: &PolicyConfig,
    ) -> PolicyDecision {
        if config.roles.is_empty() {
            return PolicyDecision::NotApplicable;
        }

        // Check if user has any of the required roles
        let has_role = config
            .roles
            .iter()
            .any(|role| context.subject.roles.contains(role));

        if has_role {
            PolicyDecision::Permit
        } else {
            PolicyDecision::Deny
        }
    }

    /// Evaluate user-based policy
    fn evaluate_user_policy(
        &self,
        context: &PolicyEvaluationContext,
        config: &PolicyConfig,
    ) -> PolicyDecision {
        if config.users.is_empty() {
            return PolicyDecision::NotApplicable;
        }

        if config.users.contains(&context.subject.user_id) {
            PolicyDecision::Permit
        } else {
            PolicyDecision::Deny
        }
    }

    /// Evaluate group-based policy
    fn evaluate_group_policy(
        &self,
        context: &PolicyEvaluationContext,
        config: &PolicyConfig,
    ) -> PolicyDecision {
        if config.groups.is_empty() {
            return PolicyDecision::NotApplicable;
        }

        let has_group = config
            .groups
            .iter()
            .any(|group| context.subject.groups.contains(group));

        if has_group {
            PolicyDecision::Permit
        } else {
            PolicyDecision::Deny
        }
    }

    /// Evaluate time-based policy
    fn evaluate_time_policy(
        &self,
        context: &PolicyEvaluationContext,
        config: &PolicyConfig,
    ) -> PolicyDecision {
        let time_config = match &config.time_config {
            Some(tc) => tc,
            None => return PolicyDecision::NotApplicable,
        };

        let now = chrono::DateTime::<Utc>::from_timestamp(context.environment.time, 0)
            .unwrap_or_else(|| Utc::now());
        let hour = now.hour() as i32;
        let weekday = now.weekday().num_days_from_sunday() as i32;

        // Check hour constraints
        if let Some(not_before) = time_config.not_before {
            if hour < not_before {
                return PolicyDecision::Deny;
            }
        }

        if let Some(not_after) = time_config.not_after {
            if hour > not_after {
                return PolicyDecision::Deny;
            }
        }

        // Check day of week
        if let Some(ref days) = time_config.days_of_week {
            if !days.contains(&weekday) {
                return PolicyDecision::Deny;
            }
        }

        PolicyDecision::Permit
    }

    /// Evaluate attribute-based policy (ABAC)
    fn evaluate_attribute_policy(
        &self,
        context: &PolicyEvaluationContext,
        config: &PolicyConfig,
    ) -> PolicyDecision {
        if config.attributes.is_empty() {
            return PolicyDecision::NotApplicable;
        }

        for (_key, condition) in &config.attributes {
            let attribute_value = context
                .subject
                .attributes
                .get(&condition.name)
                .or_else(|| context.claims.get(&condition.name));

            if !self.evaluate_attribute_condition(attribute_value, condition) {
                return PolicyDecision::Deny;
            }
        }

        PolicyDecision::Permit
    }

    /// Evaluate attribute condition
    fn evaluate_attribute_condition(
        &self,
        actual: Option<&serde_json::Value>,
        condition: &AttributeCondition,
    ) -> bool {
        let actual = match actual {
            Some(v) => v,
            None => return false,
        };

        match condition.operator.as_str() {
            "eq" => actual == &condition.value,
            "ne" => actual != &condition.value,
            "gt" => {
                if let (Some(a), Some(b)) = (actual.as_f64(), condition.value.as_f64()) {
                    a > b
                } else {
                    false
                }
            }
            "lt" => {
                if let (Some(a), Some(b)) = (actual.as_f64(), condition.value.as_f64()) {
                    a < b
                } else {
                    false
                }
            }
            "gte" => {
                if let (Some(a), Some(b)) = (actual.as_f64(), condition.value.as_f64()) {
                    a >= b
                } else {
                    false
                }
            }
            "lte" => {
                if let (Some(a), Some(b)) = (actual.as_f64(), condition.value.as_f64()) {
                    a <= b
                } else {
                    false
                }
            }
            "in" => {
                if let Some(arr) = condition.value.as_array() {
                    arr.contains(actual)
                } else {
                    false
                }
            }
            "contains" => {
                if let (Some(str_val), Some(substring)) =
                    (actual.as_str(), condition.value.as_str())
                {
                    str_val.contains(substring)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    async fn evaluate_javascript_policy(
        &self,
        _context: &PolicyEvaluationContext,
        _config: &PolicyConfig,
    ) -> Result<PolicyDecision> {
        // SECURITY: JavaScript policies are currently disabled to prevent code injection attacks.
        // Implementation requires proper sandboxing with:
        // - Isolated V8/deno_core runtime
        // - Memory limits (10MB recommended)
        // - Execution timeout (100ms recommended)
        // - No access to system resources
        //
        // See: https://github.com/denoland/deno_core for sandboxing implementation
        tracing::error!("Attempted to execute JavaScript policy - currently disabled for security");

        Err(crate::error::AuthencError::ConfigurationError {
            message: "JavaScript policies require sandboxing implementation and are currently disabled for security. \
                     Please use Rule-based, Aggregate, Time-based, or Regex policies instead.".to_string()
        })
    }

    /// Evaluate aggregate policy
    async fn evaluate_aggregate_policy(
        &self,
        context: &PolicyEvaluationContext,
        config: &PolicyConfig,
    ) -> Result<PolicyDecision> {
        // Aggregate policy combines other policies
        // For now, return NotApplicable
        // TODO: Load and evaluate referenced policies
        Ok(PolicyDecision::NotApplicable)
    }

    /// Combine decisions based on strategy
    fn combine_decisions(
        &self,
        decisions: &[(UmaPolicy, PolicyDecision)],
        strategy: DecisionStrategy,
    ) -> PolicyDecision {
        if decisions.is_empty() {
            return PolicyDecision::Deny;
        }

        match strategy {
            DecisionStrategy::Unanimous => {
                // All must permit
                let all_permit = decisions.iter().all(|(_, d)| *d == PolicyDecision::Permit);
                if all_permit {
                    PolicyDecision::Permit
                } else {
                    PolicyDecision::Deny
                }
            }
            DecisionStrategy::Affirmative => {
                // At least one must permit
                let any_permit = decisions.iter().any(|(_, d)| *d == PolicyDecision::Permit);
                if any_permit {
                    PolicyDecision::Permit
                } else {
                    PolicyDecision::Deny
                }
            }
            DecisionStrategy::Consensus => {
                // Majority must permit
                let permit_count = decisions
                    .iter()
                    .filter(|(_, d)| *d == PolicyDecision::Permit)
                    .count();
                let total_count = decisions.len();

                if permit_count > total_count / 2 {
                    PolicyDecision::Permit
                } else {
                    PolicyDecision::Deny
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_context() -> PolicyEvaluationContext {
        PolicyEvaluationContext {
            subject: SubjectContext {
                user_id: "user123".to_string(),
                roles: vec!["admin".to_string()],
                attributes: HashMap::new(),
                groups: vec!["developers".to_string()],
            },
            resource: ResourceContext {
                id: "resource456".to_string(),
                resource_type: Some("document".to_string()),
                owner: "user789".to_string(),
                attributes: HashMap::new(),
                scopes: vec!["read".to_string()],
            },
            action: "read".to_string(),
            environment: EnvironmentContext {
                ip_address: Some("192.168.1.1".to_string()),
                time: Utc::now().timestamp(),
                user_agent: None,
                device_id: None,
                location: None,
                mfa_completed: false,
                trust_score: Some(0.8),
            },
            claims: HashMap::new(),
        }
    }

    #[test]
    fn test_role_policy_evaluation() {
        let context = create_test_context();

        let mut config = PolicyConfig {
            roles: vec!["admin".to_string()],
            users: vec![],
            groups: vec![],
            time_config: None,
            attributes: HashMap::new(),
            code: None,
            policies: vec![],
        };

        let db = Arc::new(tokio_test::block_on(Database::mock()));
        let resource_store = Arc::new(crate::services::resource_store::ResourceStore::new(
            db.clone(),
        ));
        let engine = PolicyEngine::new(db, resource_store);

        let decision = engine.evaluate_role_policy(&context, &config);
        assert_eq!(decision, PolicyDecision::Permit);

        config.roles = vec!["superuser".to_string()];
        let decision = engine.evaluate_role_policy(&context, &config);
        assert_eq!(decision, PolicyDecision::Deny);
    }
}
