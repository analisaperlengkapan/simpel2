//! Authorization tests for the IAM API.
//!
//! # Scope, and why it is what it is
//!
//! This file used to hold 26 `#[tokio::test]` stubs whose bodies were entirely
//! commented out — **zero assertions**, so `cargo test` reported 26 green
//! tests that could not fail. That is worse than no coverage: a suite that
//! cannot go red reads as a guarantee, and `test_admin_authorization` was a
//! `TODO` sitting exactly where the "a non-admin must be refused" assertion
//! belonged.
//!
//! Driving the real router end-to-end needs `IamApiState`, which holds a
//! `authenc_storage::Database` that connects eagerly in `Database::new`.
//! The unit-test CI job provisions a bare Postgres, not a migrated `authenc`
//! schema, so a router test here would either fail for the wrong reason or
//! have to assert nothing to stay green.
//!
//! What *is* testable without a database is the decision itself: the
//! role→capability table in `lib_core::authz` that both the middleware and
//! the microfrontends resolve through. The router-level
//! "operator gets 403, admin gets 200" assertion lives where a real stack
//! exists — `antarmuka/portal/tests/e2e/portal-core-workflow.spec.ts` drives
//! it against the running service, and the RBAC e2e suites in
//! `antarmuka/perlengkapan/tests/e2e/rbac-scoping.spec.ts` cover the
//! perlengkapan side.
//!
//! The tests below pin the invariants those e2e suites depend on, so breakage
//! surfaces here (fast, in the unit job) instead of only in e2e.

use lib_core::authz::{ADMIN_ROLES, Authorization, Capability};

/// The exact question `test_admin_authorization` was meant to answer: a
/// non-admin must not reach an IAM admin surface.
///
/// `admin_auth_middleware` gates every route in `create_iam_router` via
/// `route_layer`, so a caller fails unless they hold a role in `ADMIN_ROLES`.
/// These are the roles the e2e seed actually mints for non-admin users; none
/// may pass.
#[test]
fn satker_bound_roles_cannot_reach_iam_admin_routes() {
    for role in [
        "operator_satker",
        "validator_satker",
        "validator_wilayah",
        "validator_pusat",
        "approver_satker",
    ] {
        let authz = Authorization::from_slice(&[role.to_string()]);
        assert!(
            !authz.is_admin(),
            "'{role}' must be refused by admin_auth_middleware"
        );
        assert!(
            !authz.can(Capability::Administer),
            "'{role}' must not administer IAM"
        );
        assert!(
            !authz.can(Capability::ViewAudit),
            "'{role}' must not read the audit log"
        );
    }
}

/// The positive case the same middleware must allow — and the regression this
/// file exists for. `admin_pusat` and `superadmin` are administrators
/// everywhere else in the system, and the middleware used to accept only the
/// exact string `admin`, so they were 403'd on every route behind it.
#[test]
fn every_admin_role_reaches_iam_admin_routes() {
    for role in ADMIN_ROLES {
        let authz = Authorization::from_slice(&[role.to_string()]);
        assert!(authz.is_admin(), "'{role}' must pass admin_auth_middleware");
    }
}

/// Roles are database rows, so an operator can mint a name that *looks*
/// administrative. A prefix match would admit all of these; the guard is an
/// exact match against the named roles.
#[test]
fn administrative_looking_role_names_are_refused() {
    for lookalike in [
        "admin_readonly",
        "admin_audit_log",
        "administrative",
        "administrator",
        "super_admin",
        "admin_pusat_readonly",
    ] {
        assert!(
            !Authorization::from_slice(&[lookalike.to_string()]).is_admin(),
            "'{lookalike}' must not pass the admin guard"
        );
    }
}

/// A caller holding an admin role alongside others still passes — the guard
/// must scan the whole role set, not just its first entry, since role order
/// from `realm_access.roles` is not guaranteed.
#[test]
fn admin_among_several_roles_still_passes() {
    for roles in [
        &["operator_satker", "admin"][..],
        &["validator_wilayah", "admin_pusat"][..],
        &["approver_satker", "validator_satker", "superadmin"][..],
    ] {
        let owned: Vec<String> = roles.iter().map(|r| r.to_string()).collect();
        assert!(
            Authorization::from_slice(&owned).is_admin(),
            "{roles:?} contains an admin role and must pass"
        );
    }
}

/// An empty or absent role claim grants nothing privileged. This is the
/// fail-closed property: `admin_auth_middleware` falls back to a DB lookup
/// when the token carries no `realm_access.roles`, and if that lookup also
/// yields nothing the caller must be refused, not waved through.
#[test]
fn absent_roles_grant_nothing() {
    let none = Authorization::from_slice(&[]);
    assert!(!none.is_admin());
    assert!(!none.can(Capability::Administer));
    assert!(!none.can(Capability::ViewAudit));
    assert!(!none.can(Capability::Create));
}
