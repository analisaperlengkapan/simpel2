//! Are policy rules written in the same vocabulary the evaluator is handed?
//!
//! Every policy this system ships is written as a Vault **resource path** —
//! `sys/capabilities-self`, `auth/token/lookup-self`, `secret/data/simpelv1/*`.
//! Those are the rows seeded by
//! `migrations/20260205000002_dynamic_role_system.sql`, and the shape the Helm
//! `secretonAuth.policies` block renders.
//!
//! `policy_check_middleware` used to hand [`PolicySet::evaluate`] the raw HTTP
//! URI instead — `/v1/sys/capabilities-self`. [`PolicySet::path_matches`] is a
//! plain glob with no normalisation, so the two could never meet: the seeded
//! `default` policy matched nothing, and #773's enforcement collapsed to "deny
//! everything that is not root". Fail-closed, so not a hole — but the policy
//! engine could not *allow* anything either, which is the same
//! mounted-but-inert failure the middleware itself had been in.
//!
//! These tests pin the vocabulary as resource paths. They live in the core
//! crate because that is where the matcher is; the companion test that the
//! REST boundary actually normalises before calling it is
//! `secreton-api/tests/policy_enforcement.rs`.

use secreton_core::models::policy::PolicyRule;
use secreton_core::services::policy::PolicySet;

fn allow(path: &str, action: &str) -> PolicyRule {
    PolicyRule {
        effect: "allow".to_string(),
        action: action.to_string(),
        path: path.to_string(),
        condition: None,
        control_group: None,
        mfa: None,
    }
}

/// The exact rule `20260205000002_dynamic_role_system.sql` seeds for `default`.
#[test]
fn the_seeded_default_policy_matches_a_resource_path() {
    let policies = PolicySet::new(vec![allow("sys/capabilities-self", "read")]);

    assert!(
        policies.evaluate("caller", "sys/capabilities-self", "read", None),
        "the shipped `default` policy does not match the path it was written for"
    );
}

/// The regression this file exists for.
///
/// If someone routes an HTTP URI back into the evaluator, this goes red. It is
/// deliberately asserted as a *non-match* rather than left implicit: the URI
/// form is what the middleware passed for the entire period the policy engine
/// silently allowed nothing.
#[test]
fn an_http_uri_is_not_a_resource_path() {
    let policies = PolicySet::new(vec![allow("sys/capabilities-self", "read")]);

    assert!(
        !policies.evaluate("caller", "/v1/sys/capabilities-self", "read", None),
        "an HTTP URI matched a resource-path rule — the two vocabularies must not \
         be silently interchangeable, or a normalisation bug looks like success"
    );
}

/// Prefix globs are how every real policy is written (`secret/data/simpelv1/*`,
/// `kv/data/integrasi/tokens/*` in the Helm block), so the wildcard semantics
/// matter as much as the prefix.
#[test]
fn a_prefix_glob_admits_its_own_subtree_and_nothing_else() {
    let policies = PolicySet::new(vec![allow("secret/data/simpelv1/*", "read")]);

    assert!(
        policies.evaluate("simpelv1", "secret/data/simpelv1/app", "read", None),
        "a workload was denied the subtree its policy names"
    );
    assert!(
        !policies.evaluate("simpelv1", "secret/data/authenc/jwt", "read", None),
        "a workload reached another workload's secrets"
    );
}

/// Single `*` is one segment; `**` is the subtree. Worth pinning because the
/// Helm block uses single-star globs (`kv/data/simpelv1/*`) against paths that
/// may nest, and getting this backwards denies in production while passing
/// every shallow test.
#[test]
fn single_star_does_not_cross_a_segment_boundary() {
    let shallow = PolicySet::new(vec![allow("secret/data/simpelv1/*", "read")]);
    assert!(
        !shallow.evaluate("simpelv1", "secret/data/simpelv1/db/password", "read", None),
        "single-star matched a nested path; use ** when the subtree is intended"
    );

    let deep = PolicySet::new(vec![allow("secret/data/simpelv1/**", "read")]);
    assert!(
        deep.evaluate("simpelv1", "secret/data/simpelv1/db/password", "read", None),
        "double-star must match a nested path"
    );
}
