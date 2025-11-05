//! Integration tests for policy check middleware

use secreton_api::middleware::{RequestContext, policy_check_middleware};
use secreton_core::models::PolicyRule;
use secreton_core::services::policy::{Capability, PolicySet};
use std::sync::{Arc, RwLock};

#[tokio::test]
async fn test_policy_middleware_allows_matching_policy() {
    // Create a policy that allows read access to secret/*
    let rules = vec![PolicyRule {
        effect: "allow".to_string(),
        action: "read".to_string(),
        path: "secret/*".to_string(),
        condition: None,
        control_group: None,
        mfa: None,
    }];

    let policy_set = Arc::new(RwLock::new(PolicySet::new(rules)));

    // Test that the policy allows access
    let policy_set_read = policy_set.read();
    let allowed = policy_set_read.evaluate("user123", "secret/database/password", "read", None);

    assert!(
        allowed,
        "Policy should allow read access to secret/database/password"
    );
}

#[tokio::test]
async fn test_policy_middleware_denies_non_matching_policy() {
    // Create a policy that only allows read access to secret/public/*
    let rules = vec![PolicyRule {
        effect: "allow".to_string(),
        action: "read".to_string(),
        path: "secret/public/*".to_string(),
        condition: None,
        control_group: None,
        mfa: None,
    }];

    let policy_set = Arc::new(RwLock::new(PolicySet::new(rules)));

    // Test that the policy denies access to other paths
    let policy_set_read = policy_set.read();
    let allowed = policy_set_read.evaluate("user123", "secret/private/password", "read", None);

    assert!(
        !allowed,
        "Policy should deny read access to secret/private/password"
    );
}

#[tokio::test]
async fn test_policy_middleware_deny_overrides_allow() {
    // Create policies where deny overrides allow
    let rules = vec![
        PolicyRule {
            effect: "allow".to_string(),
            action: "read".to_string(),
            path: "secret/*".to_string(),
            condition: None,
            control_group: None,
            mfa: None,
        },
        PolicyRule {
            effect: "deny".to_string(),
            action: "read".to_string(),
            path: "secret/protected/*".to_string(),
            condition: None,
            control_group: None,
            mfa: None,
        },
    ];

    let policy_set = Arc::new(RwLock::new(PolicySet::new(rules)));

    // Test that deny overrides allow
    let policy_set_read = policy_set.read();

    // Should allow access to non-protected secrets
    let allowed = policy_set_read.evaluate("user123", "secret/database/password", "read", None);
    assert!(
        allowed,
        "Policy should allow read access to secret/database/password"
    );

    // Should deny access to protected secrets
    let denied = policy_set_read.evaluate("user123", "secret/protected/admin", "read", None);
    assert!(
        !denied,
        "Policy should deny read access to secret/protected/admin"
    );
}

#[tokio::test]
async fn test_policy_middleware_caching() {
    // Create a simple policy
    let rules = vec![PolicyRule {
        effect: "allow".to_string(),
        action: "read".to_string(),
        path: "secret/*".to_string(),
        condition: None,
        control_group: None,
        mfa: None,
    }];

    let policy_set = Arc::new(RwLock::new(PolicySet::new(rules)));

    // First evaluation (cache miss)
    let policy_set_read = policy_set.read();
    let allowed1 = policy_set_read.evaluate("user123", "secret/database/password", "read", None);
    assert!(allowed1);

    // Second evaluation (cache hit)
    let allowed2 = policy_set_read.evaluate("user123", "secret/database/password", "read", None);
    assert!(allowed2);

    // Check cache stats
    let (total, _expired) = policy_set_read.cache_stats();
    assert_eq!(total, 1, "Should have 1 cached entry");
}

#[tokio::test]
async fn test_policy_middleware_wildcard_matching() {
    // Test single wildcard (*)
    let rules = vec![PolicyRule {
        effect: "allow".to_string(),
        action: "read".to_string(),
        path: "secret/data/*".to_string(),
        condition: None,
        control_group: None,
        mfa: None,
    }];

    let policy_set = Arc::new(RwLock::new(PolicySet::new(rules)));
    let policy_set_read = policy_set.read();

    // Should match single level
    assert!(policy_set_read.evaluate("user123", "secret/data/foo", "read", None));

    // Should not match nested paths
    assert!(!policy_set_read.evaluate("user123", "secret/data/foo/bar", "read", None));
}

#[tokio::test]
async fn test_policy_middleware_double_wildcard_matching() {
    // Test double wildcard (**)
    let rules = vec![PolicyRule {
        effect: "allow".to_string(),
        action: "read".to_string(),
        path: "secret/data/**".to_string(),
        condition: None,
        control_group: None,
        mfa: None,
    }];

    let policy_set = Arc::new(RwLock::new(PolicySet::new(rules)));
    let policy_set_read = policy_set.read();

    // Should match all levels
    assert!(policy_set_read.evaluate("user123", "secret/data/foo", "read", None));
    assert!(policy_set_read.evaluate("user123", "secret/data/foo/bar", "read", None));
    assert!(policy_set_read.evaluate("user123", "secret/data/foo/bar/baz", "read", None));
}
