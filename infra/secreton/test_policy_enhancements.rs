// Standalone test file for policy enhancements
// Run with: cargo test --test test_policy_enhancements

use serde_json::json;

// Mock the necessary types for testing
#[derive(Debug, Clone)]
struct PolicyRule {
    effect: String,
    action: String,
    path: String,
    condition: Option<serde_json::Value>,
    control_group: Option<ControlGroup>,
    mfa: Option<bool>,
}

#[derive(Debug, Clone)]
struct ControlGroup {
    required_approvals: u32,
    approved_by: Vec<String>,
}

// Include the PolicySet implementation
include!("crates/core/src/services/policy.rs");

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_cidr_matching() {
        let mut rule = create_test_rule("allow", "read", "secret/*");
        rule.condition = Some(json!({
            "allowed_ips": ["192.168.1.0/24"]
        }));
        let policy_set = PolicySet::new(vec![rule]);

        // IP in CIDR range
        let context = json!({ "client_ip": "192.168.1.100" });
        assert!(policy_set.evaluate("user1", "secret/foo", "read", Some(&context)));

        // IP outside CIDR range
        let context = json!({ "client_ip": "10.0.0.1" });
        assert!(!policy_set.evaluate("user1", "secret/foo", "read", Some(&context)));
    }

    #[test]
    fn test_expression_evaluation() {
        let mut rule = create_test_rule("allow", "read", "secret/*");
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
        assert!(policy_set.evaluate("user1", "secret/foo", "read", Some(&context)));

        // User level insufficient
        let context = json!({ "user_level": 3 });
        assert!(!policy_set.evaluate("user1", "secret/foo", "read", Some(&context)));
    }

    #[test]
    fn test_contains_operator() {
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
        let context = json!({ "tags": ["user", "admin"] });
        assert!(policy_set.evaluate("user1", "secret/foo", "read", Some(&context)));

        // Tags don't contain admin
        let context = json!({ "tags": ["user"] });
        assert!(!policy_set.evaluate("user1", "secret/foo", "read", Some(&context)));
    }
}
