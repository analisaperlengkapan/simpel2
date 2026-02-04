//! Policy Service
//!
//! Handles policy management operations using PostgreSQL storage.

use deadpool_postgres::Pool;
use serde::Serialize;
use serde_json::Value;

use secreton_core::{
    error::CoreError,
    models::PolicyRule,
    services::policy::{Capability, PolicySet},
};

#[derive(Debug, Serialize, Clone)]
pub struct PolicyServiceResponse {
    pub id: i64,
    pub name: String,
    pub namespace: String,
    pub description: Option<String>,
    pub rules: Vec<PolicyRule>,
    pub version: i32,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub created_by: String,
    pub updated_by: Option<String>,
    pub stats: Option<PolicyStats>,
}

#[derive(Debug, Serialize, Clone)]
pub struct PolicyStats {
    pub evaluations_total: i64,
    pub evaluations_allowed: i64,
    pub evaluations_denied: i64,
    pub cache_hits: i64,
    pub cache_misses: i64,
    pub last_evaluated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize)]
pub struct TestPolicyResult {
    pub allowed: bool,
    pub matched_rules: Vec<String>,
    pub evaluation_time_ms: f64,
}

pub struct PolicyService {
    pool: Pool,
}

impl PolicyService {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    pub async fn get_rules_for_policies(
        &self,
        names: &[String],
        namespace: &str,
    ) -> Result<Vec<PolicyRule>, CoreError> {
        if names.is_empty() {
            return Ok(Vec::new());
        }

        let client = self.pool.get().await.map_err(|e| CoreError::Internal {
            message: format!("Database connection error: {}", e),
            source: None,
        })?;

        let query = "SELECT rules FROM policies WHERE name = ANY($1) AND namespace = $2 AND is_active = true";
        let rows = client
            .query(query, &[&names, &namespace])
            .await
            .map_err(|e| CoreError::Internal {
                message: format!("Database query error: {}", e),
                source: None,
            })?;

        let mut all_rules = Vec::new();
        for row in rows {
            let rules_json: serde_json::Value = row.get(0);
            if let Ok(rules) = serde_json::from_value::<Vec<PolicyRule>>(rules_json) {
                all_rules.extend(rules);
            }
        }

        Ok(all_rules)
    }

    pub async fn list_policy_names(
        &self,
        namespace: Option<String>,
        limit: u32,
        offset: u32,
    ) -> Result<(Vec<String>, u64), CoreError> {
        let client = self.pool.get().await.map_err(|e| CoreError::Internal {
            message: format!("Database connection error: {}", e),
            source: None,
        })?;

        // Build query with filters
        let mut where_clauses = vec![];
        let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = vec![];
        let mut param_idx = 1;

        if let Some(ns) = namespace {
            where_clauses.push(format!("namespace = ${}", param_idx));
            params.push(Box::new(ns));
            param_idx += 1;
        }

        let where_clause = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        // Convert params to references for query
        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
            params.iter().map(|p| p.as_ref() as _).collect();

        // Count total
        let count_query = format!("SELECT COUNT(*) FROM policies {}", where_clause);
        let count_row = client
            .query_one(&count_query, &param_refs)
            .await
            .map_err(|e| CoreError::Internal {
                message: format!("Internal error: {}", e),
                source: None,
            })?;
        let total: i64 = count_row.get(0);

        let select_query = format!(
            "SELECT name FROM policies {} ORDER BY name ASC LIMIT ${} OFFSET ${}",
            where_clause,
            param_idx,
            param_idx + 1
        );

        // Add limit/offset to params
        let mut query_params = param_refs;
        let limit_i64 = limit as i64;
        let offset_i64 = offset as i64;
        query_params.push(&limit_i64);
        query_params.push(&offset_i64);

        let rows = client
            .query(&select_query, &query_params)
            .await
            .map_err(|e| CoreError::Internal {
                message: format!("Internal error: {}", e),
                source: None,
            })?;

        let names = rows.iter().map(|row| row.get(0)).collect();

        Ok((names, total as u64))
    }

    pub async fn list_policies(
        &self,
        namespace: Option<String>,
        is_active: Option<bool>,
        search: Option<String>,
        limit: u32,
        offset: u32,
    ) -> Result<(Vec<PolicyServiceResponse>, u64), CoreError> {
        let client = self.pool.get().await.map_err(|e| CoreError::Internal {
            message: format!("Database connection error: {}", e),
            source: None,
        })?;

        // Build query with filters
        let mut where_clauses = vec![];
        let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = vec![];
        let mut param_idx = 1;

        if let Some(ns) = namespace {
            where_clauses.push(format!("namespace = ${}", param_idx));
            params.push(Box::new(ns));
            param_idx += 1;
        }

        if let Some(active) = is_active {
            where_clauses.push(format!("is_active = ${}", param_idx));
            params.push(Box::new(active));
            param_idx += 1;
        }

        if let Some(s) = search {
            where_clauses.push(format!(
                "(name ILIKE ${} OR description ILIKE ${})",
                param_idx, param_idx
            ));
            let pattern = format!("%{}%", s);
            params.push(Box::new(pattern));
            param_idx += 1;
        }

        let where_clause = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        // Convert params to references for query
        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
            params.iter().map(|p| p.as_ref() as _).collect();

        // Count total
        let count_query = format!("SELECT COUNT(*) FROM policies {}", where_clause);
        let count_row = client
            .query_one(&count_query, &param_refs)
            .await
            .map_err(|e| CoreError::Internal {
                message: format!("Internal error: {}", e),
                source: None,
            })?;
        let total: i64 = count_row.get(0);

        let select_query = format!(
            "SELECT id, name, namespace, description, rules, version, is_active, created_at, updated_at, created_by, updated_by
             FROM policies {}
             ORDER BY created_at DESC
             LIMIT ${} OFFSET ${}",
            where_clause, param_idx, param_idx + 1
        );

        // Add limit/offset to params
        let mut query_params = param_refs;
        let limit_i64 = limit as i64;
        let offset_i64 = offset as i64;
        query_params.push(&limit_i64);
        query_params.push(&offset_i64);

        let rows = client
            .query(&select_query, &query_params)
            .await
            .map_err(|e| CoreError::Internal {
                message: format!("Internal error: {}", e),
                source: None,
            })?;

        let mut policies = vec![];
        for row in rows {
            let id: i64 = row.get(0);
            let rules_json: serde_json::Value = row.get(4);
            let rules: Vec<PolicyRule> = serde_json::from_value(rules_json).unwrap_or_default();

            // Get stats
            let stats = self.get_policy_stats(&client, id).await.ok();

            policies.push(PolicyServiceResponse {
                id,
                name: row.get(1),
                namespace: row.get(2),
                description: row.get(3),
                rules,
                version: row.get(5),
                is_active: row.get(6),
                created_at: row.get(7),
                updated_at: row.get(8),
                created_by: row.get(9),
                updated_by: row.get(10),
                stats,
            });
        }

        Ok((policies, total as u64))
    }

    pub async fn create_policy(
        &self,
        name: String,
        namespace: String,
        description: Option<String>,
        rules: Vec<PolicyRule>,
        created_by: String,
    ) -> Result<PolicyServiceResponse, CoreError> {
        // Validate policy name
        if name.is_empty()
            || !name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(CoreError::Validation {
                message: "Invalid input: Policy name must be alphanumeric".to_string(),
            });
        }

        // Validate rules
        Self::validate_policy_rules(&rules)?;

        let client = self.pool.get().await.map_err(|e| CoreError::Internal {
            message: format!("Database error: {}", e),
            source: None,
        })?;

        // Check if policy already exists
        let existing = client
            .query_opt(
                "SELECT id FROM policies WHERE name = $1 AND namespace = $2",
                &[&name, &namespace],
            )
            .await
            .map_err(|e| CoreError::Internal {
                message: format!("Database error: {}", e),
                source: None,
            })?;

        if existing.is_some() {
            return Err(CoreError::AlreadyExists {
                resource: "policy".to_string(),
            });
        }

        // Serialize rules to JSON
        let rules_json = serde_json::to_value(&rules).map_err(|e| CoreError::Internal {
            message: format!("Serialization error: {}", e),
            source: None,
        })?;

        // Insert policy
        let row = client
            .query_one(
                "INSERT INTO policies (name, namespace, description, rules, version, created_by)
                 VALUES ($1, $2, $3, $4, 1, $5)
                 RETURNING id, name, namespace, description, rules, version, is_active, created_at, updated_at, created_by, updated_by",
                &[&name, &namespace, &description, &rules_json, &created_by],
            )
            .await
            .map_err(|e| CoreError::Internal {
                message: format!("Database error: {}", e),
                source: None,
            })?;

        let policy_id: i64 = row.get(0);

        // Initialize stats
        client
            .execute(
                "INSERT INTO policy_stats (policy_id) VALUES ($1)",
                &[&policy_id],
            )
            .await
            .map_err(|e| CoreError::Internal {
                message: format!("Database error: {}", e),
                source: None,
            })?;

        // Create version history
        client
            .execute(
                "INSERT INTO policy_versions (policy_id, version, rules, description, created_by)
                 VALUES ($1, 1, $2, $3, $4)",
                &[&policy_id, &rules_json, &description, &created_by],
            )
            .await
            .map_err(|e| CoreError::Internal {
                message: format!("Database error: {}", e),
                source: None,
            })?;

        Ok(PolicyServiceResponse {
            id: policy_id,
            name: row.get(1),
            namespace: row.get(2),
            description: row.get(3),
            rules,
            version: row.get(5),
            is_active: row.get(6),
            created_at: row.get(7),
            updated_at: row.get(8),
            created_by: row.get(9),
            updated_by: row.get(10),
            stats: Some(PolicyStats {
                evaluations_total: 0,
                evaluations_allowed: 0,
                evaluations_denied: 0,
                cache_hits: 0,
                cache_misses: 0,
                last_evaluated_at: None,
            }),
        })
    }

    pub async fn get_policy(&self, name: String) -> Result<PolicyServiceResponse, CoreError> {
        let client = self.pool.get().await.map_err(|e| CoreError::Internal {
            message: format!("Database error: {}", e),
            source: None,
        })?;

        let row = client
            .query_opt(
                "SELECT id, name, namespace, description, rules, version, is_active, created_at, updated_at, created_by, updated_by
                 FROM policies WHERE name = $1",
                &[&name],
            )
            .await
            .map_err(|e| CoreError::Internal {
                message: format!("Database error: {}", e),
                source: None,
            })?;

        let row = row.ok_or_else(|| CoreError::NotFound {
            resource: "policy".to_string(),
        })?;

        let policy_id: i64 = row.get(0);
        let rules_json: serde_json::Value = row.get(4);
        let rules: Vec<PolicyRule> = serde_json::from_value(rules_json).unwrap_or_default();

        let stats = self.get_policy_stats(&client, policy_id).await.ok();

        Ok(PolicyServiceResponse {
            id: policy_id,
            name: row.get(1),
            namespace: row.get(2),
            description: row.get(3),
            rules,
            version: row.get(5),
            is_active: row.get(6),
            created_at: row.get(7),
            updated_at: row.get(8),
            created_by: row.get(9),
            updated_by: row.get(10),
            stats,
        })
    }

    pub async fn update_policy(
        &self,
        name: String,
        description: Option<String>,
        rules: Option<Vec<PolicyRule>>,
        is_active: Option<bool>,
        updated_by: String,
    ) -> Result<PolicyServiceResponse, CoreError> {
        // Validate rules if provided
        if let Some(ref r) = rules {
            Self::validate_policy_rules(r)?;
        }

        let client = self.pool.get().await.map_err(|e| CoreError::Internal {
            message: format!("Database error: {}", e),
            source: None,
        })?;

        let existing = client
            .query_opt("SELECT id, version FROM policies WHERE name = $1", &[&name])
            .await
            .map_err(|e| CoreError::Internal {
                message: format!("Database error: {}", e),
                source: None,
            })?;

        let existing = existing.ok_or_else(|| CoreError::NotFound {
            resource: "policy".to_string(),
        })?;

        let policy_id: i64 = existing.get(0);
        let current_version: i32 = existing.get(1);
        let new_version = current_version + 1;

        let mut updates = vec![];
        let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = vec![];
        let mut param_idx = 1;

        if let Some(d) = description.clone() {
            updates.push(format!("description = ${}", param_idx));
            params.push(Box::new(d));
            param_idx += 1;
        }

        let rules_json: Option<serde_json::Value>;
        if let Some(r) = rules {
            let json = serde_json::to_value(&r).map_err(|e| CoreError::Internal {
                message: format!("Serialization error: {}", e),
                source: None,
            })?;
            updates.push(format!("rules = ${}", param_idx));
            rules_json = Some(json.clone());
            params.push(Box::new(json));
            param_idx += 1;
        } else {
            rules_json = None;
        }

        if let Some(active) = is_active {
            updates.push(format!("is_active = ${}", param_idx));
            params.push(Box::new(active));
            param_idx += 1;
        }

        if updates.is_empty() {
            // Return current state if no updates
            return self.get_policy(name).await;
        }

        updates.push(format!("version = ${}", param_idx));
        params.push(Box::new(new_version));
        param_idx += 1;

        updates.push(format!("updated_by = ${}", param_idx));
        params.push(Box::new(updated_by.clone()));
        param_idx += 1;

        let update_query = format!(
            "UPDATE policies SET {} WHERE name = ${}
             RETURNING id, name, namespace, description, rules, version, is_active, created_at, updated_at, created_by, updated_by",
            updates.join(", "),
            param_idx
        );

        params.push(Box::new(name.clone()));

        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
            params.iter().map(|p| p.as_ref() as _).collect();

        let row = client
            .query_one(&update_query, &param_refs)
            .await
            .map_err(|e| CoreError::Internal {
                message: format!("Internal error: {}", e),
                source: None,
            })?;

        // Version history
        if let Some(json) = rules_json {
            client
                .execute(
                    "INSERT INTO policy_versions (policy_id, version, rules, description, created_by)
                     VALUES ($1, $2, $3, $4, $5)",
                    &[&policy_id, &new_version, &json, &description, &updated_by],
                )
                .await
                .map_err(|e| CoreError::Internal {
                    message: format!("Database error: {}", e),
                    source: None,
                })?;
        }

        let rules: Vec<PolicyRule> = serde_json::from_value(row.get(4)).unwrap_or_default();
        let stats = self.get_policy_stats(&client, policy_id).await.ok();

        Ok(PolicyServiceResponse {
            id: policy_id,
            name: row.get(1),
            namespace: row.get(2),
            description: row.get(3),
            rules,
            version: row.get(5),
            is_active: row.get(6),
            created_at: row.get(7),
            updated_at: row.get(8),
            created_by: row.get(9),
            updated_by: row.get(10),
            stats,
        })
    }

    pub async fn delete_policy(&self, name: String) -> Result<(), CoreError> {
        let client = self.pool.get().await.map_err(|e| CoreError::Internal {
            message: format!("Database error: {}", e),
            source: None,
        })?;

        let row = client
            .query_opt("SELECT id FROM policies WHERE name = $1", &[&name])
            .await
            .map_err(|e| CoreError::Internal {
                message: format!("Database error: {}", e),
                source: None,
            })?;

        let row = row.ok_or_else(|| CoreError::NotFound {
            resource: "policy".to_string(),
        })?;

        let policy_id: i64 = row.get(0);

        // Dependencies check
        let deps = client
            .query(
                "SELECT p.name FROM policies p
                 INNER JOIN policy_dependencies pd ON p.id = pd.policy_id
                 WHERE pd.depends_on_policy_id = $1",
                &[&policy_id],
            )
            .await
            .map_err(|e| CoreError::Internal {
                message: format!("Database error: {}", e),
                source: None,
            })?;

        if !deps.is_empty() {
            return Err(CoreError::Validation {
                message: "Cannot delete policy: other policies depend on it".to_string(),
            });
        }

        let deleted = client
            .execute("DELETE FROM policies WHERE id = $1", &[&policy_id])
            .await
            .map_err(|e| CoreError::Internal {
                message: format!("Database error: {}", e),
                source: None,
            })?;

        if deleted == 0 {
            return Err(CoreError::NotFound {
                resource: "policy".to_string(),
            });
        }

        Ok(())
    }

    pub async fn test_policy(
        &self,
        name: String,
        user: String,
        path: String,
        action: String,
        context: Option<Value>,
    ) -> Result<TestPolicyResult, CoreError> {
        let client = self.pool.get().await.map_err(|e| CoreError::Internal {
            message: format!("Database error: {}", e),
            source: None,
        })?;

        let row = client
            .query_opt(
                "SELECT rules FROM policies WHERE name = $1 AND is_active = true",
                &[&name],
            )
            .await
            .map_err(|e| CoreError::Internal {
                message: format!("Database error: {}", e),
                source: None,
            })?;

        let row = row.ok_or_else(|| CoreError::NotFound {
            resource: "policy".to_string(),
        })?;

        let rules_json: serde_json::Value = row.get(0);
        let rules: Vec<PolicyRule> = serde_json::from_value(rules_json).unwrap_or_default();

        let policy_set = PolicySet::new(rules.clone());

        let start = std::time::Instant::now();
        let allowed = policy_set.evaluate(&user, &path, &action, context.as_ref());
        let duration = start.elapsed();

        let mut matched_rules = vec![];
        for (idx, rule) in rules.iter().enumerate() {
            let test_policy = PolicySet::new(vec![rule.clone()]);
            if test_policy.evaluate(&user, &path, &action, context.as_ref()) {
                matched_rules.push(format!(
                    "Rule {}: {} {} on {}",
                    idx + 1,
                    rule.effect,
                    rule.action,
                    rule.path
                ));
            }
        }

        Ok(TestPolicyResult {
            allowed,
            matched_rules,
            evaluation_time_ms: duration.as_secs_f64() * 1000.0,
        })
    }

    // Helper functions
    async fn get_policy_stats(
        &self,
        client: &tokio_postgres::Client,
        policy_id: i64,
    ) -> Result<PolicyStats, CoreError> {
        let row = client
            .query_opt(
                "SELECT evaluations_total, evaluations_allowed, evaluations_denied, cache_hits, cache_misses, last_evaluated_at
                 FROM policy_stats WHERE policy_id = $1",
                &[&policy_id],
            )
            .await
            .map_err(|e| CoreError::Internal {
                message: format!("Database query error: {}", e),
                source: None,
            })?;

        if let Some(row) = row {
            Ok(PolicyStats {
                evaluations_total: row.get(0),
                evaluations_allowed: row.get(1),
                evaluations_denied: row.get(2),
                cache_hits: row.get(3),
                cache_misses: row.get(4),
                last_evaluated_at: row.get(5),
            })
        } else {
            Ok(PolicyStats {
                evaluations_total: 0,
                evaluations_allowed: 0,
                evaluations_denied: 0,
                cache_hits: 0,
                cache_misses: 0,
                last_evaluated_at: None,
            })
        }
    }

    fn validate_policy_rules(rules: &[PolicyRule]) -> Result<(), CoreError> {
        if rules.is_empty() {
            return Err(CoreError::Validation {
                message: "Policy must have at least one rule".to_string(),
            });
        }

        for (idx, rule) in rules.iter().enumerate() {
            if rule.effect != "allow" && rule.effect != "deny" {
                return Err(CoreError::Validation {
                    message: "Effect must be 'allow' or 'deny'".to_string(),
                });
            }

            if rule.path.is_empty() {
                return Err(CoreError::Validation {
                    message: format!("Rule {}: Path cannot be empty", idx),
                });
            }

            if rule.path.contains("**") && !rule.path.ends_with("**") {
                return Err(CoreError::Validation {
                    message: format!(
                        "Rule {}: Wildcard ** is only allowed at the end of path",
                        idx
                    ),
                });
            }

            if rule.action.is_empty() {
                return Err(CoreError::Validation {
                    message: format!("Rule {}: Action cannot be empty", idx),
                });
            }

            if rule.action != "*"
                && Capability::from_str(&rule.action).is_none()
                && !rule.action.chars().all(|c| c.is_alphanumeric() || c == '_')
            {
                return Err(CoreError::Validation {
                    message: format!("Rule {}: Invalid action format", idx),
                });
            }

            if let Some(cg) = &rule.control_group
                && cg.required_approvals == 0
            {
                return Err(CoreError::Validation {
                    message: format!("Rule {}: Control group approvals must be > 0", idx),
                });
            }

            if let Some(condition) = &rule.condition {
                Self::validate_condition(condition, idx)?;
            }
        }

        Ok(())
    }

    fn validate_condition(condition: &Value, rule_idx: usize) -> Result<(), CoreError> {
        if !condition.is_object() {
            return Err(CoreError::Validation {
                message: format!("Rule {}: Condition must be an object", rule_idx),
            });
        }

        if let Some(time_range) = condition.get("time_range") {
            if !time_range.is_object() {
                return Err(CoreError::Validation {
                    message: format!("Rule {}: time_range must be an object", rule_idx),
                });
            }

            if let Some(start) = time_range.get("start")
                && let Some(start_str) = start.as_str()
                && chrono::DateTime::parse_from_rfc3339(start_str).is_err()
            {
                return Err(CoreError::Validation {
                    message: format!("Rule {}: Invalid start time format", rule_idx),
                });
            }

            if let Some(end) = time_range.get("end")
                && let Some(end_str) = end.as_str()
                && chrono::DateTime::parse_from_rfc3339(end_str).is_err()
            {
                return Err(CoreError::Validation {
                    message: format!("Rule {}: Invalid end time format", rule_idx),
                });
            }
        }

        if let Some(allowed_ips) = condition.get("allowed_ips")
            && !allowed_ips.is_array()
        {
            return Err(CoreError::Validation {
                message: format!("Rule {}: allowed_ips must be an array", rule_idx),
            });
        }

        if let Some(expr) = condition.get("expression") {
            if !expr.is_object() {
                return Err(CoreError::Validation {
                    message: format!("Rule {}: expression must be an object", rule_idx),
                });
            }

            if expr.get("field").is_none()
                || expr.get("op").is_none()
                || expr.get("value").is_none()
            {
                return Err(CoreError::Validation {
                    message: format!(
                        "Rule {}: expression must have field, op, and value",
                        rule_idx
                    ),
                });
            }
        }

        Ok(())
    }

    // Unused in current implementation but kept for future
    #[allow(dead_code)]
    async fn check_circular_dependencies(
        &self,
        pool: &deadpool_postgres::Pool,
        policy_id: i64,
        depends_on_id: i64,
    ) -> Result<bool, CoreError> {
        let client = pool.get().await.map_err(|e| CoreError::Internal {
            message: format!("Database connection error: {}", e),
            source: None,
        })?;

        let query = r#"
            WITH RECURSIVE deps AS (
                SELECT depends_on_policy_id
                FROM policy_dependencies
                WHERE policy_id = $1

                UNION

                SELECT pd.depends_on_policy_id
                FROM policy_dependencies pd
                INNER JOIN deps ON deps.depends_on_policy_id = pd.policy_id
            )
            SELECT EXISTS(SELECT 1 FROM deps WHERE depends_on_policy_id = $2)
        "#;

        let row = client
            .query_one(query, &[&depends_on_id, &policy_id])
            .await
            .map_err(|e| CoreError::Internal {
                message: format!("Database query error: {}", e),
                source: None,
            })?;

        Ok(row.get(0))
    }
}
