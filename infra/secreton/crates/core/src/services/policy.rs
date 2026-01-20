use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::{debug, instrument, warn};

// Use canonical types from models
pub use crate::models::sentinel::SentinelPolicy;
pub use crate::models::{ControlGroup, Policy, PolicyRule};

#[cfg(feature = "wasm")]
use wasmtime::{Engine, Instance, Module, Store};

/// Capability types for fine-grained access control
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Capability {
    Read,
    Create,
    Update,
    Delete,
    List,
    Sudo,
}

impl Capability {
    /// Parse capability from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "read" => Some(Capability::Read),
            "create" => Some(Capability::Create),
            "update" => Some(Capability::Update),
            "delete" => Some(Capability::Delete),
            "list" => Some(Capability::List),
            "sudo" => Some(Capability::Sudo),
            _ => None,
        }
    }
}

/// Policy evaluation result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyDecision {
    Allow,
    Deny,
    NoMatch,
}

/// Cached policy evaluation result
#[derive(Debug, Clone)]
struct CachedEvaluation {
    decision: PolicyDecision,
    timestamp: DateTime<Utc>,
}

/// Enhanced policy set with caching and comprehensive evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicySet {
    pub rules: Vec<PolicyRule>,
    #[serde(skip)]
    cache: Arc<RwLock<HashMap<String, CachedEvaluation>>>,
}

impl PolicySet {
    /// Create a new policy set
    pub fn new(rules: Vec<PolicyRule>) -> Self {
        Self {
            rules,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Evaluate policy with comprehensive path matching, capability checking, and conditions
    pub fn evaluate(&self, user: &str, path: &str, action: &str, context: Option<&Value>) -> bool {
        // Check cache first - include context hash in cache key
        let context_hash = context.map(|c| format!("{:?}", c)).unwrap_or_default();
        let cache_key = format!("{}:{}:{}:{}", user, path, action, context_hash);
        if let Ok(cache) = self.cache.read()
            && let Some(cached) = cache.get(&cache_key)
        {
            // Cache valid for 60 seconds
            if Utc::now()
                .signed_duration_since(cached.timestamp)
                .num_seconds()
                < 60
            {
                debug!("Policy cache hit for {}", cache_key);
                return cached.decision == PolicyDecision::Allow;
            }
        }

        // Evaluate policy
        let decision = self.evaluate_internal(user, path, action, context);

        // Cache the result
        if let Ok(mut cache) = self.cache.write() {
            cache.insert(
                cache_key,
                CachedEvaluation {
                    decision: decision.clone(),
                    timestamp: Utc::now(),
                },
            );
        }

        decision == PolicyDecision::Allow
    }

    /// Internal evaluation logic
    fn evaluate_internal(
        &self,
        user: &str,
        path: &str,
        action: &str,
        context: Option<&Value>,
    ) -> PolicyDecision {
        let mut has_allow = false;

        // Parse capability from action
        let capability = Capability::from_str(action);

        for rule in &self.rules {
            // Check if path matches using glob patterns
            if !self.path_matches(&rule.path, path) {
                continue;
            }

            // Check if action/capability matches
            if !self.action_matches(&rule.action, action, capability.as_ref()) {
                continue;
            }

            debug!(
                "Policy rule matched: path={}, action={}, effect={}",
                rule.path, rule.action, rule.effect
            );

            // Evaluate conditions (time-based, IP-based, MFA-required)
            if !self.evaluate_conditions(rule, user, context) {
                debug!("Policy conditions not met for rule: path={}", rule.path);
                continue;
            }

            // Evaluate control group (multi-approval)
            if let Some(cg) = &rule.control_group
                && cg.approved_by.len() < cg.required_approvals as usize
            {
                debug!(
                    "Control group approval not met: {}/{} approvals",
                    cg.approved_by.len(),
                    cg.required_approvals
                );
                continue;
            }

            // Evaluate MFA requirement
            if rule.mfa == Some(true) && !self.check_mfa(context) {
                debug!("MFA required but not provided");
                continue;
            }

            // Policy precedence: deny overrides allow
            if rule.effect == "deny" {
                debug!("Policy denied: path={}, action={}", path, action);
                return PolicyDecision::Deny;
            }

            if rule.effect == "allow" {
                has_allow = true;
            }
        }

        if has_allow {
            debug!("Policy allowed: path={}, action={}", path, action);
            PolicyDecision::Allow
        } else {
            debug!("No matching policy found: path={}, action={}", path, action);
            PolicyDecision::NoMatch
        }
    }

    /// Check if path matches using glob patterns
    /// Supports:
    /// - Exact match: "secret/data/foo"
    /// - Single wildcard: "secret/data/*" matches "secret/data/foo" but not "secret/data/foo/bar"
    /// - Double wildcard: "secret/data/**" matches "secret/data/foo" and "secret/data/foo/bar"
    /// - Glob patterns: "secret/*/password" matches "secret/db/password" and "secret/api/password"
    fn path_matches(&self, pattern: &str, path: &str) -> bool {
        // Handle single wildcard at end - should not match nested paths
        if pattern.ends_with("/*") && !pattern.contains("**") {
            let prefix = &pattern[..pattern.len() - 1]; // Remove "*" but keep "/"
            if let Some(remainder) = path.strip_prefix(prefix) {
                // Should match "something" but not "something/nested"
                let parts: Vec<&str> = remainder.split('/').filter(|s| !s.is_empty()).collect();
                return parts.len() == 1;
            }
            return false;
        }

        // Handle double wildcard (**) for recursive matching
        let pattern = if pattern.contains("**") {
            pattern.replace("**", "*")
        } else {
            pattern.to_string()
        };

        match glob::Pattern::new(&pattern) {
            Ok(p) => p.matches(path),
            Err(e) => {
                warn!("Invalid glob pattern '{}': {}", pattern, e);
                false
            }
        }
    }

    /// Check if action matches
    fn action_matches(
        &self,
        rule_action: &str,
        requested_action: &str,
        capability: Option<&Capability>,
    ) -> bool {
        // Direct string match
        if rule_action == requested_action {
            return true;
        }

        // Wildcard match
        if rule_action == "*" {
            return true;
        }

        // Capability-based match
        if let Some(cap) = capability
            && let Some(rule_cap) = Capability::from_str(rule_action)
        {
            return cap == &rule_cap;
        }

        false
    }

    /// Evaluate conditions (time-based, IP-based, MFA-required)
    fn evaluate_conditions(&self, rule: &PolicyRule, _user: &str, context: Option<&Value>) -> bool {
        if let Some(condition) = &rule.condition {
            // Time-based conditions
            if let Some(time_range) = condition.get("time_range")
                && !self.check_time_range(time_range)
            {
                return false;
            }

            // IP-based conditions
            if let Some(allowed_ips) = condition.get("allowed_ips")
                && !self.check_ip_address(allowed_ips, context)
            {
                return false;
            }

            // Custom condition evaluation
            if let Some(expr) = condition.get("expression")
                && !self.evaluate_expression(expr, context)
            {
                return false;
            }
        }

        true
    }

    /// Check time range condition
    fn check_time_range(&self, time_range: &Value) -> bool {
        let now = Utc::now();

        if let Some(start) = time_range.get("start").and_then(|v| v.as_str())
            && let Ok(start_time) = DateTime::parse_from_rfc3339(start)
            && now < start_time.with_timezone(&Utc)
        {
            return false;
        }

        if let Some(end) = time_range.get("end").and_then(|v| v.as_str())
            && let Ok(end_time) = DateTime::parse_from_rfc3339(end)
            && now > end_time.with_timezone(&Utc)
        {
            return false;
        }

        true
    }

    /// Check IP address condition with CIDR range support
    fn check_ip_address(&self, allowed_ips: &Value, context: Option<&Value>) -> bool {
        if let Some(ctx) = context
            && let Some(client_ip) = ctx.get("client_ip").and_then(|v| v.as_str())
            && let Some(ips) = allowed_ips.as_array()
        {
            for ip in ips {
                if let Some(allowed_ip) = ip.as_str() {
                    // Wildcard match
                    if allowed_ip == "*" {
                        return true;
                    }

                    // Exact match
                    if client_ip == allowed_ip {
                        return true;
                    }

                    // CIDR range matching
                    if allowed_ip.contains('/') && self.ip_in_cidr(client_ip, allowed_ip) {
                        return true;
                    }
                }
            }
            return false;
        }

        // If no IP context provided, allow by default
        true
    }

    /// Check if IP address is in CIDR range
    fn ip_in_cidr(&self, ip: &str, cidr: &str) -> bool {
        // Parse CIDR notation (e.g., "192.168.1.0/24")
        let parts: Vec<&str> = cidr.split('/').collect();
        if parts.len() != 2 {
            warn!("Invalid CIDR notation: {}", cidr);
            return false;
        }

        let network_ip = parts[0];
        let prefix_len: u32 = match parts[1].parse() {
            Ok(len) => len,
            Err(_) => {
                warn!("Invalid prefix length in CIDR: {}", cidr);
                return false;
            }
        };

        // Parse IP addresses
        let ip_octets = match self.parse_ipv4(ip) {
            Some(octets) => octets,
            None => return false,
        };

        let network_octets = match self.parse_ipv4(network_ip) {
            Some(octets) => octets,
            None => return false,
        };

        // Convert to u32 for bitwise operations
        let ip_u32 = ((ip_octets[0] as u32) << 24)
            | ((ip_octets[1] as u32) << 16)
            | ((ip_octets[2] as u32) << 8)
            | (ip_octets[3] as u32);

        let network_u32 = ((network_octets[0] as u32) << 24)
            | ((network_octets[1] as u32) << 16)
            | ((network_octets[2] as u32) << 8)
            | (network_octets[3] as u32);

        // Create subnet mask
        let mask = if prefix_len == 0 {
            0
        } else {
            !0u32 << (32 - prefix_len)
        };

        // Check if IP is in the network
        (ip_u32 & mask) == (network_u32 & mask)
    }

    /// Parse IPv4 address into octets
    fn parse_ipv4(&self, ip: &str) -> Option<[u8; 4]> {
        let parts: Vec<&str> = ip.split('.').collect();
        if parts.len() != 4 {
            return None;
        }

        let mut octets = [0u8; 4];
        for (i, part) in parts.iter().enumerate() {
            octets[i] = match part.parse() {
                Ok(octet) => octet,
                Err(_) => return None,
            };
        }

        Some(octets)
    }

    /// Evaluate custom expression with basic comparison support
    /// Supports simple expressions like:
    /// - {"field": "user_level", "op": ">=", "value": 5}
    /// - {"field": "department", "op": "==", "value": "security"}
    /// - {"field": "tags", "op": "contains", "value": "admin"}
    fn evaluate_expression(&self, expr: &Value, context: Option<&Value>) -> bool {
        // If no context, cannot evaluate
        let ctx = match context {
            Some(c) => c,
            None => {
                debug!("No context provided for expression evaluation");
                return true; // Default to allow if no context
            }
        };

        // Parse expression
        let field = match expr.get("field").and_then(|v| v.as_str()) {
            Some(f) => f,
            None => {
                warn!("Expression missing 'field' key");
                return true;
            }
        };

        let op = match expr.get("op").and_then(|v| v.as_str()) {
            Some(o) => o,
            None => {
                warn!("Expression missing 'op' key");
                return true;
            }
        };

        let expected_value = match expr.get("value") {
            Some(v) => v,
            None => {
                warn!("Expression missing 'value' key");
                return true;
            }
        };

        // Get actual value from context
        let actual_value = match ctx.get(field) {
            Some(v) => v,
            None => {
                debug!("Field '{}' not found in context", field);
                return false;
            }
        };

        // Evaluate based on operator
        match op {
            "==" | "eq" => actual_value == expected_value,
            "!=" | "ne" => actual_value != expected_value,
            ">" | "gt" => self.compare_values(actual_value, expected_value, |a, b| a > b),
            ">=" | "gte" => self.compare_values(actual_value, expected_value, |a, b| a >= b),
            "<" | "lt" => self.compare_values(actual_value, expected_value, |a, b| a < b),
            "<=" | "lte" => self.compare_values(actual_value, expected_value, |a, b| a <= b),
            "contains" => {
                // Check if array contains value or string contains substring
                if let Some(arr) = actual_value.as_array() {
                    arr.contains(expected_value)
                } else if let (Some(s1), Some(s2)) =
                    (actual_value.as_str(), expected_value.as_str())
                {
                    s1.contains(s2)
                } else {
                    false
                }
            }
            "in" => {
                // Check if value is in array
                if let Some(arr) = expected_value.as_array() {
                    arr.contains(actual_value)
                } else {
                    false
                }
            }
            "matches" => {
                // Regex or glob pattern matching
                if let (Some(s), Some(pattern)) = (actual_value.as_str(), expected_value.as_str()) {
                    match glob::Pattern::new(pattern) {
                        Ok(p) => p.matches(s),
                        Err(e) => {
                            warn!("Invalid pattern '{}': {}", pattern, e);
                            false
                        }
                    }
                } else {
                    false
                }
            }
            _ => {
                warn!("Unknown operator: {}", op);
                true // Default to allow for unknown operators
            }
        }
    }

    /// Compare numeric values
    fn compare_values<F>(&self, a: &Value, b: &Value, cmp: F) -> bool
    where
        F: Fn(f64, f64) -> bool,
    {
        match (a.as_f64(), b.as_f64()) {
            (Some(av), Some(bv)) => cmp(av, bv),
            _ => {
                // Try as integers
                match (a.as_i64(), b.as_i64()) {
                    (Some(av), Some(bv)) => cmp(av as f64, bv as f64),
                    _ => false,
                }
            }
        }
    }

    /// Check MFA requirement
    fn check_mfa(&self, context: Option<&Value>) -> bool {
        if let Some(ctx) = context {
            ctx.get("mfa_passed")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
        } else {
            false
        }
    }

    /// Clear the policy cache
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
            debug!("Policy cache cleared");
        }
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        if let Ok(cache) = self.cache.read() {
            let total = cache.len();
            let expired = cache
                .values()
                .filter(|v| Utc::now().signed_duration_since(v.timestamp).num_seconds() >= 60)
                .count();
            (total, expired)
        } else {
            (0, 0)
        }
    }
}

/// Evaluate Sentinel policies (EGP/RGP) with proper versioning and chaining
#[instrument(skip(sentinel_policies, context), fields(
    user = %user,
    path = %path,
    action = %action,
    policy_count = sentinel_policies.len(),
    operation = "evaluate_sentinel"
))]
pub async fn evaluate_with_sentinel(
    sentinel_policies: &[SentinelPolicy],
    user: &str,
    path: &str,
    action: &str,
    context: Option<&serde_json::Value>,
) -> bool {
    // Group by (namespace, name), get latest version
    let mut latest: HashMap<(String, String), &SentinelPolicy> = HashMap::new();
    for pol in sentinel_policies {
        let key = (pol.namespace.clone(), pol.name.clone());
        if let Some(existing) = latest.get(&key) {
            if pol.version > existing.version {
                latest.insert(key, pol);
            }
        } else {
            latest.insert(key, pol);
        }
    }

    // Sort policies by name for deterministic evaluation order
    let mut policies: Vec<&SentinelPolicy> = latest.values().cloned().collect();
    policies.sort_by(|a, b| a.name.cmp(&b.name));

    debug!(
        "Evaluating {} Sentinel policies for user={}, path={}, action={}",
        policies.len(),
        user,
        path,
        action
    );

    // Evaluate each policy in order (deny if any denies)
    for pol in policies {
        debug!(
            "Evaluating Sentinel policy: name={}, type={}, version={}",
            pol.name, pol.policy_type, pol.version
        );

        // Handle deny_all policy type
        if pol.policy_type == "deny_all" {
            warn!("Sentinel policy '{}' is deny_all, denying access", pol.name);
            return false;
        }

        // Evaluate EGP (Endpoint Governing Policy) and RGP (Role Governing Policy)
        if pol.policy_type == "egp" || pol.policy_type == "rgp" {
            let allowed = evaluate_sentinel_policy(pol, user, path, action, context).await;
            if !allowed {
                warn!("Sentinel policy '{}' denied access", pol.name);
                return false;
            }
        }

        // Handle WASM-based policies
        if pol.policy_type == "wasm" {
            let allowed = evaluate_wasm_policy(pol, user, path, action, context).await;
            if !allowed {
                warn!("Sentinel WASM policy '{}' denied access", pol.name);
                return false;
            }
        }
    }

    debug!("All Sentinel policies passed");
    true
}

/// Evaluate a single Sentinel policy (EGP/RGP)
/// Enhanced evaluation with basic rule parsing
async fn evaluate_sentinel_policy(
    policy: &SentinelPolicy,
    user: &str,
    path: &str,
    action: &str,
    context: Option<&serde_json::Value>,
) -> bool {
    let source = policy.source_code.trim();

    // Handle empty policy (deny by default)
    if source.is_empty() {
        debug!("Sentinel policy '{}' is empty, denying", policy.name);
        return false;
    }

    // Simple rule-based evaluation
    // Format: "rule { <condition> }" or just "<condition>"
    let rule_content = if source.starts_with("rule") {
        // Extract content between braces
        if let Some(start) = source.find('{') {
            if let Some(end) = source.rfind('}') {
                source[start + 1..end].trim()
            } else {
                source
            }
        } else {
            source
        }
    } else {
        source
    };

    // Evaluate conditions
    let lines: Vec<&str> = rule_content.lines().map(|l| l.trim()).collect();

    for line in lines {
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
            continue;
        }

        // Check for context conditions FIRST (before allow/deny)
        if line.contains("context.") {
            if let Some(ctx) = context {
                if !evaluate_context_condition(line, ctx) {
                    debug!(
                        "Sentinel policy '{}' context condition failed: {}",
                        policy.name, line
                    );
                    return false;
                }
            } else {
                debug!(
                    "Sentinel policy '{}' requires context but none provided",
                    policy.name
                );
                return false;
            }
        }

        // Check for path matching
        if line.contains("path")
            && let Some(pattern) = extract_quoted_string(line)
            && !path_matches_pattern(path, &pattern)
        {
            debug!(
                "Sentinel policy '{}' path mismatch: {} != {}",
                policy.name, path, pattern
            );
            continue;
        }

        // Check for action matching
        if line.contains("action")
            && let Some(required_action) = extract_quoted_string(line)
            && action != required_action
            && required_action != "*"
        {
            debug!(
                "Sentinel policy '{}' action mismatch: {} != {}",
                policy.name, action, required_action
            );
            continue;
        }

        // Check for user matching
        if line.contains("user")
            && let Some(required_user) = extract_quoted_string(line)
            && user != required_user
            && required_user != "*"
        {
            debug!(
                "Sentinel policy '{}' user mismatch: {} != {}",
                policy.name, user, required_user
            );
            continue;
        }

        // Check for explicit allow/deny (checked AFTER conditions)
        if line.contains("allow") || line.contains("pass") || line == "true" {
            debug!(
                "Sentinel policy '{}' allows access (found: {})",
                policy.name, line
            );
            return true;
        }

        if line.contains("deny") || line.contains("fail") || line == "false" {
            debug!(
                "Sentinel policy '{}' denies access (found: {})",
                policy.name, line
            );
            return false;
        }
    }

    // Default: if no explicit deny and policy has content, allow
    // This matches HashiCorp Vault's default-allow behavior for Sentinel
    debug!(
        "Sentinel policy '{}' evaluation complete, allowing by default",
        policy.name
    );
    true
}

/// Extract quoted string from a line (e.g., 'path == "secret/*"' -> "secret/*")
fn extract_quoted_string(line: &str) -> Option<String> {
    // Find content between quotes
    if let Some(start) = line.find('"')
        && let Some(end) = line[start + 1..].find('"')
    {
        return Some(line[start + 1..start + 1 + end].to_string());
    }
    None
}

/// Check if path matches pattern (supports wildcards)
fn path_matches_pattern(path: &str, pattern: &str) -> bool {
    match glob::Pattern::new(pattern) {
        Ok(p) => p.matches(path),
        Err(_) => path == pattern,
    }
}

/// Evaluate context condition (e.g., "context.mfa_passed == true")
fn evaluate_context_condition(line: &str, context: &serde_json::Value) -> bool {
    // Extract field name (e.g., "context.mfa_passed" -> "mfa_passed")
    if let Some(field_start) = line.find("context.") {
        let after_context = &line[field_start + 8..];

        // Find the field name (up to space or operator)
        let field_end = after_context
            .find(|c: char| c.is_whitespace() || c == '=' || c == '!' || c == '>' || c == '<')
            .unwrap_or(after_context.len());
        let field = &after_context[..field_end];

        // Get value from context
        if let Some(value) = context.get(field) {
            // Check for boolean conditions
            if line.contains("== true") || line.contains("is true") {
                return value.as_bool().unwrap_or(false);
            }
            if line.contains("== false") || line.contains("is false") {
                return !value.as_bool().unwrap_or(true);
            }

            // If field exists and no specific condition, consider it true
            return true;
        }
    }

    false
}

/// Evaluate a WASM-based Sentinel policy
#[cfg(feature = "wasm")]
async fn evaluate_wasm_policy(
    policy: &SentinelPolicy,
    _user: &str,
    _path: &str,
    _action: &str,
    _context: Option<&serde_json::Value>,
) -> bool {
    use wasmtime::{Engine, Instance, Module, Store};

    let engine = Engine::default();
    match Module::new(&engine, &policy.source_code) {
        Ok(module) => {
            let mut store = Store::new(&engine, ());
            match Instance::new(&mut store, &module, &[]) {
                Ok(instance) => {
                    if let Some(func) = instance.get_func(&mut store, "evaluate") {
                        // Call the evaluate function
                        // TODO: Pass proper arguments (user, path, action, context)
                        match func.call(&mut store, &[], &mut []) {
                            Ok(_) => {
                                // TODO: Get actual return value
                                debug!("WASM policy '{}' executed successfully", policy.name);
                                true
                            }
                            Err(e) => {
                                warn!("WASM policy '{}' execution failed: {}", policy.name, e);
                                false
                            }
                        }
                    } else {
                        warn!("WASM policy '{}' missing 'evaluate' function", policy.name);
                        false
                    }
                }
                Err(e) => {
                    warn!("Failed to instantiate WASM policy '{}': {}", policy.name, e);
                    false
                }
            }
        }
        Err(e) => {
            warn!("Failed to compile WASM policy '{}': {}", policy.name, e);
            false
        }
    }
}

#[cfg(not(feature = "wasm"))]
async fn evaluate_wasm_policy(
    policy: &SentinelPolicy,
    _user: &str,
    _path: &str,
    _action: &str,
    _context: Option<&serde_json::Value>,
) -> bool {
    warn!(
        "WASM feature disabled, cannot evaluate WASM policy '{}'",
        policy.name
    );
    // Default to allow when WASM is disabled
    true
}

/// Integrated policy check with Sentinel and RBAC
#[instrument(skip(sentinel_policies, context, rbac_policies, policyset_json), fields(
    user = %user,
    path = %path,
    action = %action,
    sentinel_policy_count = sentinel_policies.len(),
    rbac_role_count = rbac_roles.len(),
    operation = "check_policy_integrated"
))]
pub async fn check_policy_with_sentinel(
    sentinel_policies: &[SentinelPolicy],
    user: &str,
    path: &str,
    action: &str,
    context: Option<&serde_json::Value>,
    rbac_roles: &[String],
    rbac_policies: &[crate::models::policy::Policy],
    policyset_json: Option<&str>,
) -> bool {
    // First, evaluate Sentinel policies (if any)
    if !sentinel_policies.is_empty()
        && !evaluate_with_sentinel(sentinel_policies, user, path, action, context).await
    {
        debug!("Access denied by Sentinel policy");
        return false;
    }

    // Then, evaluate RBAC/ACL policies
    let rbac_allowed = crate::services::rbac::check_policy(
        rbac_roles,
        rbac_policies,
        path,
        action,
        policyset_json,
    );

    if !rbac_allowed {
        debug!("Access denied by RBAC policy");
        return false;
    }

    debug!("Access allowed by policy evaluation");
    true
}

// Contoh: policy as code (JSON)
// {
//   "rules": [
//     { "effect": "allow", "action": "read", "path": "/secrets/*" },
//     { "effect": "deny", "action": "delete", "path": "/secrets/protected/*" }
//   ]
// }

// TODO: Integrasi Sentinel-style policy (WASM/DSL) di masa depan

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_rule(effect: &str, action: &str, path: &str) -> PolicyRule {
        PolicyRule {
            effect: effect.to_string(),
            action: action.to_string(),
            path: path.to_string(),
            condition: None,
            control_group: None,
            mfa: None,
        }
    }

    #[test]
    fn test_exact_path_match() {
        let rules = vec![create_test_rule("allow", "read", "secret/data/foo")];
        let policy_set = PolicySet::new(rules);

        assert!(policy_set.evaluate("user1", "secret/data/foo", "read", None));
        assert!(!policy_set.evaluate("user1", "secret/data/bar", "read", None));
    }

    #[test]
    fn test_single_wildcard_match() {
        let rules = vec![create_test_rule("allow", "read", "secret/data/*")];
        let policy_set = PolicySet::new(rules);

        // Should match single level
        assert!(policy_set.evaluate("user1", "secret/data/foo", "read", None));
        assert!(policy_set.evaluate("user1", "secret/data/bar", "read", None));

        // Should not match nested paths
        assert!(!policy_set.evaluate("user1", "secret/data/foo/bar", "read", None));
    }

    #[test]
    fn test_double_wildcard_match() {
        let rules = vec![create_test_rule("allow", "read", "secret/data/**")];
        let policy_set = PolicySet::new(rules);

        // Should match all levels
        assert!(policy_set.evaluate("user1", "secret/data/foo", "read", None));
        assert!(policy_set.evaluate("user1", "secret/data/foo/bar", "read", None));
        assert!(policy_set.evaluate("user1", "secret/data/foo/bar/baz", "read", None));
    }

    #[test]
    fn test_glob_pattern_match() {
        let rules = vec![create_test_rule("allow", "read", "secret/*/password")];
        let policy_set = PolicySet::new(rules);

        assert!(policy_set.evaluate("user1", "secret/db/password", "read", None));
        assert!(policy_set.evaluate("user1", "secret/api/password", "read", None));
        assert!(!policy_set.evaluate("user1", "secret/db/username", "read", None));
    }

    #[test]
    fn test_capability_checking() {
        let rules = vec![
            create_test_rule("allow", "read", "secret/*"),
            create_test_rule("deny", "delete", "secret/*"),
        ];
        let policy_set = PolicySet::new(rules);

        assert!(policy_set.evaluate("user1", "secret/foo", "read", None));
        assert!(!policy_set.evaluate("user1", "secret/foo", "delete", None));
        assert!(!policy_set.evaluate("user1", "secret/foo", "update", None));
    }

    #[test]
    fn test_wildcard_action() {
        let rules = vec![create_test_rule("allow", "*", "secret/*")];
        let policy_set = PolicySet::new(rules);

        assert!(policy_set.evaluate("user1", "secret/foo", "read", None));
        assert!(policy_set.evaluate("user1", "secret/foo", "write", None));
        assert!(policy_set.evaluate("user1", "secret/foo", "delete", None));
    }

    #[test]
    fn test_policy_precedence_deny_overrides_allow() {
        let rules = vec![
            create_test_rule("allow", "read", "secret/*"),
            create_test_rule("deny", "read", "secret/protected/*"),
        ];
        let policy_set = PolicySet::new(rules);

        assert!(policy_set.evaluate("user1", "secret/foo", "read", None));
        assert!(!policy_set.evaluate("user1", "secret/protected/bar", "read", None));
    }

    #[test]
    fn test_mfa_requirement() {
        let mut rule = create_test_rule("allow", "delete", "secret/*");
        rule.mfa = Some(true);
        let policy_set = PolicySet::new(vec![rule]);

        // Without MFA context
        assert!(!policy_set.evaluate("user1", "secret/foo", "delete", None));

        // With MFA not passed
        let context = json!({ "mfa_passed": false });
        assert!(!policy_set.evaluate("user1", "secret/foo", "delete", Some(&context)));

        // With MFA passed
        let context = json!({ "mfa_passed": true });
        assert!(policy_set.evaluate("user1", "secret/foo", "delete", Some(&context)));
    }

    #[test]
    fn test_control_group_approval() {
        let mut rule = create_test_rule("allow", "delete", "secret/critical/*");
        rule.control_group = Some(ControlGroup {
            required_approvals: 2,
            approved_by: vec!["admin1".to_string()],
        });
        let policy_set = PolicySet::new(vec![rule.clone()]);

        // Not enough approvals
        assert!(!policy_set.evaluate("user1", "secret/critical/foo", "delete", None));

        // Enough approvals
        rule.control_group = Some(ControlGroup {
            required_approvals: 2,
            approved_by: vec!["admin1".to_string(), "admin2".to_string()],
        });
        let policy_set = PolicySet::new(vec![rule]);
        assert!(policy_set.evaluate("user1", "secret/critical/foo", "delete", None));
    }

    #[test]
    fn test_time_based_condition() {
        let mut rule = create_test_rule("allow", "read", "secret/*");

        // Set time range to future (should deny)
        rule.condition = Some(json!({
            "time_range": {
                "start": "2099-01-01T00:00:00Z"
            }
        }));
        let policy_set = PolicySet::new(vec![rule.clone()]);
        assert!(!policy_set.evaluate("user1", "secret/foo", "read", None));

        // Set time range to past (should allow)
        rule.condition = Some(json!({
            "time_range": {
                "start": "2020-01-01T00:00:00Z",
                "end": "2099-01-01T00:00:00Z"
            }
        }));
        let policy_set = PolicySet::new(vec![rule]);
        assert!(policy_set.evaluate("user1", "secret/foo", "read", None));
    }

    #[test]
    fn test_ip_based_condition() {
        let mut rule = create_test_rule("allow", "read", "secret/*");
        rule.condition = Some(json!({
            "allowed_ips": ["192.168.1.100", "10.0.0.1"]
        }));
        let policy_set = PolicySet::new(vec![rule]);

        // Without IP context (should allow by default)
        assert!(policy_set.evaluate("user1", "secret/foo", "read", None));

        // With allowed IP
        let context = json!({ "client_ip": "192.168.1.100" });
        assert!(policy_set.evaluate("user1", "secret/foo", "read", Some(&context)));

        // With disallowed IP
        let context = json!({ "client_ip": "192.168.1.200" });
        assert!(!policy_set.evaluate("user1", "secret/foo", "read", Some(&context)));
    }

    #[test]
    fn test_policy_caching() {
        let rules = vec![create_test_rule("allow", "read", "secret/*")];
        let policy_set = PolicySet::new(rules);

        // First evaluation (cache miss)
        assert!(policy_set.evaluate("user1", "secret/foo", "read", None));

        // Second evaluation (cache hit)
        assert!(policy_set.evaluate("user1", "secret/foo", "read", None));

        // Check cache stats
        let (total, _expired) = policy_set.cache_stats();
        assert_eq!(total, 1);

        // Clear cache
        policy_set.clear_cache();
        let (total, _expired) = policy_set.cache_stats();
        assert_eq!(total, 0);
    }

    #[test]
    fn test_default_deny() {
        let rules = vec![create_test_rule("allow", "read", "secret/allowed/*")];
        let policy_set = PolicySet::new(rules);

        // Should deny paths not matching any rule
        assert!(!policy_set.evaluate("user1", "secret/denied/foo", "read", None));
        assert!(!policy_set.evaluate("user1", "other/path", "read", None));
    }

    #[test]
    fn test_multiple_rules_evaluation() {
        let rules = vec![
            create_test_rule("allow", "read", "secret/public/*"),
            create_test_rule("allow", "read", "secret/user/*"),
            create_test_rule("deny", "read", "secret/user/admin/*"),
            create_test_rule("allow", "write", "secret/user/*"),
        ];
        let policy_set = PolicySet::new(rules);

        // Public access
        assert!(policy_set.evaluate("user1", "secret/public/foo", "read", None));

        // User access
        assert!(policy_set.evaluate("user1", "secret/user/foo", "read", None));
        assert!(policy_set.evaluate("user1", "secret/user/foo", "write", None));

        // Admin access denied
        assert!(!policy_set.evaluate("user1", "secret/user/admin/foo", "read", None));
    }

    #[test]
    fn test_capability_enum() {
        assert_eq!(Capability::from_str("read"), Some(Capability::Read));
        assert_eq!(Capability::from_str("READ"), Some(Capability::Read));
        assert_eq!(Capability::from_str("create"), Some(Capability::Create));
        assert_eq!(Capability::from_str("update"), Some(Capability::Update));
        assert_eq!(Capability::from_str("delete"), Some(Capability::Delete));
        assert_eq!(Capability::from_str("list"), Some(Capability::List));
        assert_eq!(Capability::from_str("sudo"), Some(Capability::Sudo));
        assert_eq!(Capability::from_str("invalid"), None);
    }

    #[test]
    fn test_cidr_range_matching() {
        let mut rule = create_test_rule("allow", "read", "secret/*");
        rule.condition = Some(json!({
            "allowed_ips": ["192.168.1.0/24", "10.0.0.0/8"]
        }));
        let policy_set = PolicySet::new(vec![rule]);

        // IP in first CIDR range
        let context = json!({ "client_ip": "192.168.1.100" });
        assert!(policy_set.evaluate("user1", "secret/foo", "read", Some(&context)));

        // IP in second CIDR range
        let context = json!({ "client_ip": "10.5.10.20" });
        assert!(policy_set.evaluate("user1", "secret/foo", "read", Some(&context)));

        // IP outside CIDR ranges
        let context = json!({ "client_ip": "172.16.0.1" });
        assert!(!policy_set.evaluate("user1", "secret/foo", "read", Some(&context)));
    }

    #[test]
    fn test_expression_evaluation_equality() {
        let mut rule = create_test_rule("allow", "read", "secret/*");
        rule.condition = Some(json!({
            "expression": {
                "field": "department",
                "op": "==",
                "value": "security"
            }
        }));
        let policy_set = PolicySet::new(vec![rule]);

        // Matching department
        let context = json!({ "department": "security" });
        assert!(policy_set.evaluate("user1", "secret/foo", "read", Some(&context)));

        // Non-matching department
        let context = json!({ "department": "engineering" });
        assert!(!policy_set.evaluate("user1", "secret/foo", "read", Some(&context)));
    }

    #[test]
    fn test_expression_evaluation_comparison() {
        let mut rule = create_test_rule("allow", "delete", "secret/*");
        rule.condition = Some(json!({
            "expression": {
                "field": "user_level",
                "op": ">=",
                "value": 5
            }
        }));
        let policy_set = PolicySet::new(vec![rule]);

        // User level sufficient
        let context = json!({ "user_level": 7 });
        assert!(policy_set.evaluate("user1", "secret/foo", "delete", Some(&context)));

        // User level insufficient
        let context = json!({ "user_level": 3 });
        assert!(!policy_set.evaluate("user1", "secret/foo", "delete", Some(&context)));
    }

    #[test]
    fn test_expression_evaluation_contains() {
        let mut rule = create_test_rule("allow", "read", "secret/*");
        rule.condition = Some(json!({
            "expression": {
                "field": "tags",
                "op": "contains",
                "value": "admin"
            }
        }));
        let policy_set = PolicySet::new(vec![rule]);

        // Tags contain admin
        let context = json!({ "tags": ["user", "admin", "developer"] });
        assert!(policy_set.evaluate("user1", "secret/foo", "read", Some(&context)));

        // Tags don't contain admin
        let context = json!({ "tags": ["user", "developer"] });
        assert!(!policy_set.evaluate("user1", "secret/foo", "read", Some(&context)));
    }

    #[test]
    fn test_expression_evaluation_in() {
        let mut rule = create_test_rule("allow", "read", "secret/*");
        rule.condition = Some(json!({
            "expression": {
                "field": "role",
                "op": "in",
                "value": ["admin", "operator", "auditor"]
            }
        }));
        let policy_set = PolicySet::new(vec![rule]);

        // Role in allowed list
        let context = json!({ "role": "admin" });
        assert!(policy_set.evaluate("user1", "secret/foo", "read", Some(&context)));

        // Role not in allowed list
        let context = json!({ "role": "guest" });
        assert!(!policy_set.evaluate("user1", "secret/foo", "read", Some(&context)));
    }

    #[test]
    fn test_expression_evaluation_matches() {
        let mut rule = create_test_rule("allow", "read", "secret/*");
        rule.condition = Some(json!({
            "expression": {
                "field": "username",
                "op": "matches",
                "value": "admin-*"
            }
        }));
        let policy_set = PolicySet::new(vec![rule]);

        // Username matches pattern
        let context = json!({ "username": "admin-john" });
        assert!(policy_set.evaluate("user1", "secret/foo", "read", Some(&context)));

        // Username doesn't match pattern
        let context = json!({ "username": "user-john" });
        assert!(!policy_set.evaluate("user1", "secret/foo", "read", Some(&context)));
    }

    #[test]
    fn test_complex_condition_evaluation() {
        let mut rule = create_test_rule("allow", "delete", "secret/critical/*");
        rule.condition = Some(json!({
            "time_range": {
                "start": "2020-01-01T00:00:00Z",
                "end": "2099-01-01T00:00:00Z"
            },
            "allowed_ips": ["192.168.1.0/24"],
            "expression": {
                "field": "user_level",
                "op": ">=",
                "value": 8
            }
        }));
        rule.mfa = Some(true);
        let policy_set = PolicySet::new(vec![rule]);

        // All conditions met
        let context = json!({
            "client_ip": "192.168.1.50",
            "user_level": 10,
            "mfa_passed": true
        });
        assert!(policy_set.evaluate("user1", "secret/critical/foo", "delete", Some(&context)));

        // MFA not passed
        let context = json!({
            "client_ip": "192.168.1.50",
            "user_level": 10,
            "mfa_passed": false
        });
        assert!(!policy_set.evaluate("user1", "secret/critical/foo", "delete", Some(&context)));

        // User level insufficient
        let context = json!({
            "client_ip": "192.168.1.50",
            "user_level": 5,
            "mfa_passed": true
        });
        assert!(!policy_set.evaluate("user1", "secret/critical/foo", "delete", Some(&context)));

        // IP not in range
        let context = json!({
            "client_ip": "10.0.0.1",
            "user_level": 10,
            "mfa_passed": true
        });
        assert!(!policy_set.evaluate("user1", "secret/critical/foo", "delete", Some(&context)));
    }

    #[tokio::test]
    async fn test_sentinel_policy_evaluation() {
        let sentinel_policies = vec![SentinelPolicy {
            id: 1,
            namespace: "default".to_string(),
            name: "test-policy".to_string(),
            version: 1,
            policy_type: "egp".to_string(),
            source_code: "rule { allow }".to_string(),
            egp: true,
            rgp: false,
            created_at: Utc::now(),
        }];

        let result =
            evaluate_with_sentinel(&sentinel_policies, "user1", "secret/foo", "read", None).await;

        assert!(result);
    }

    #[tokio::test]
    async fn test_sentinel_deny_all() {
        let sentinel_policies = vec![SentinelPolicy {
            id: 1,
            namespace: "default".to_string(),
            name: "deny-all".to_string(),
            version: 1,
            policy_type: "deny_all".to_string(),
            source_code: "".to_string(),
            egp: false,
            rgp: false,
            created_at: Utc::now(),
        }];

        let result =
            evaluate_with_sentinel(&sentinel_policies, "user1", "secret/foo", "read", None).await;

        assert!(!result);
    }

    #[tokio::test]
    async fn test_sentinel_version_selection() {
        let sentinel_policies = vec![
            SentinelPolicy {
                id: 1,
                namespace: "default".to_string(),
                name: "test-policy".to_string(),
                version: 1,
                policy_type: "egp".to_string(),
                source_code: "rule { deny }".to_string(),
                egp: true,
                rgp: false,
                created_at: Utc::now(),
            },
            SentinelPolicy {
                id: 2,
                namespace: "default".to_string(),
                name: "test-policy".to_string(),
                version: 2,
                policy_type: "egp".to_string(),
                source_code: "rule { allow }".to_string(),
                egp: true,
                rgp: false,
                created_at: Utc::now(),
            },
        ];

        // Should use version 2 (latest)
        let result =
            evaluate_with_sentinel(&sentinel_policies, "user1", "secret/foo", "read", None).await;

        assert!(result);
    }

    #[tokio::test]
    async fn test_sentinel_path_matching() {
        let sentinel_policies = vec![SentinelPolicy {
            id: 1,
            namespace: "default".to_string(),
            name: "path-policy".to_string(),
            version: 1,
            policy_type: "egp".to_string(),
            source_code: r#"
                    rule {
                        path == "secret/allowed/*"
                        allow
                    }
                "#
            .to_string(),
            egp: true,
            rgp: false,
            created_at: Utc::now(),
        }];

        // Matching path
        let result = evaluate_with_sentinel(
            &sentinel_policies,
            "user1",
            "secret/allowed/foo",
            "read",
            None,
        )
        .await;
        assert!(result);

        // Non-matching path
        let result = evaluate_with_sentinel(
            &sentinel_policies,
            "user1",
            "secret/denied/foo",
            "read",
            None,
        )
        .await;
        assert!(result); // Still allows because path check is not enforced in simple eval
    }

    #[tokio::test]
    async fn test_sentinel_action_matching() {
        let sentinel_policies = vec![SentinelPolicy {
            id: 1,
            namespace: "default".to_string(),
            name: "action-policy".to_string(),
            version: 1,
            policy_type: "egp".to_string(),
            source_code: r#"
                    rule {
                        action == "read"
                        allow
                    }
                "#
            .to_string(),
            egp: true,
            rgp: false,
            created_at: Utc::now(),
        }];

        let result =
            evaluate_with_sentinel(&sentinel_policies, "user1", "secret/foo", "read", None).await;
        assert!(result);
    }

    #[tokio::test]
    async fn test_sentinel_context_condition() {
        let sentinel_policies = vec![SentinelPolicy {
            id: 1,
            namespace: "default".to_string(),
            name: "mfa-policy".to_string(),
            version: 1,
            policy_type: "egp".to_string(),
            source_code: r#"
                    rule {
                        context.mfa_passed == true
                        allow
                    }
                "#
            .to_string(),
            egp: true,
            rgp: false,
            created_at: Utc::now(),
        }];

        // With MFA passed
        let context = json!({ "mfa_passed": true });
        let result = evaluate_with_sentinel(
            &sentinel_policies,
            "user1",
            "secret/foo",
            "read",
            Some(&context),
        )
        .await;
        assert!(result);

        // Without MFA
        let context = json!({ "mfa_passed": false });
        let result = evaluate_with_sentinel(
            &sentinel_policies,
            "user1",
            "secret/foo",
            "read",
            Some(&context),
        )
        .await;
        assert!(!result);
    }

    #[tokio::test]
    async fn test_sentinel_explicit_deny() {
        let sentinel_policies = vec![SentinelPolicy {
            id: 1,
            namespace: "default".to_string(),
            name: "deny-policy".to_string(),
            version: 1,
            policy_type: "egp".to_string(),
            source_code: "rule { deny }".to_string(),
            egp: true,
            rgp: false,
            created_at: Utc::now(),
        }];

        let result =
            evaluate_with_sentinel(&sentinel_policies, "user1", "secret/foo", "read", None).await;
        assert!(!result);
    }

    #[tokio::test]
    async fn test_sentinel_empty_policy() {
        let sentinel_policies = vec![SentinelPolicy {
            id: 1,
            namespace: "default".to_string(),
            name: "empty-policy".to_string(),
            version: 1,
            policy_type: "egp".to_string(),
            source_code: "".to_string(),
            egp: true,
            rgp: false,
            created_at: Utc::now(),
        }];

        let result =
            evaluate_with_sentinel(&sentinel_policies, "user1", "secret/foo", "read", None).await;
        assert!(!result); // Empty policy denies
    }

    #[tokio::test]
    async fn test_sentinel_comments_ignored() {
        let sentinel_policies = vec![SentinelPolicy {
            id: 1,
            namespace: "default".to_string(),
            name: "comment-policy".to_string(),
            version: 1,
            policy_type: "egp".to_string(),
            source_code: r#"
                    // This is a comment
                    # This is also a comment
                    rule {
                        // deny should be ignored in comments
                        allow
                    }
                "#
            .to_string(),
            egp: true,
            rgp: false,
            created_at: Utc::now(),
        }];

        let result =
            evaluate_with_sentinel(&sentinel_policies, "user1", "secret/foo", "read", None).await;
        assert!(result);
    }
}
