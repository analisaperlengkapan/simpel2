//! Authorization contract tests for the IAM API.
//!
//! # Scope, and why it is what it is
//!
//! This file used to hold 26 `#[tokio::test]` stubs whose bodies were entirely
//! commented out — **zero assertions**, so `cargo test` reported 26 green tests
//! that could not fail. A suite that cannot go red reads as a guarantee.
//!
//! Driving the real router end-to-end needs `IamApiState`, which holds a
//! `authenc_storage::Database` that connects eagerly in `Database::new`; the
//! unit-test CI job provisions a bare Postgres, not a migrated `authenc`
//! schema. So the *decision* the guard makes is unit-tested where it lives —
//! `middleware::admin_auth::tests` exercises `evaluate_admin` (MFA-pending,
//! IAM-admin-only, password change) with no database — and this file pins the
//! shared vocabulary that decision resolves through (`lib_core::authz`), so a
//! change to it surfaces here, in the fast unit job, instead of only in e2e.
//!
//! The router-level "operator gets 403, admin gets 200" assertion lives where a
//! real stack exists: `antarmuka/portal/tests/e2e/portal-core-workflow.spec.ts`.
//!
//! Note what this file does **not** claim: that the *application*-admin list
//! (`ADMIN_ROLES`) may enter the IAM console. It may not — the console is
//! `IAM_ADMIN_ROLES` (exact `admin`).

use lib_core::authz::{ADMIN_ROLES, Authorization, Capability, IAM_ADMIN_ROLES, is_iam_admin_role};

/// Roles the e2e seed mints for non-admin users; none may reach the IAM console
/// or read its audit log.
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
            !authz.is_iam_admin(),
            "'{role}' must be refused by the IAM guard"
        );
        assert!(
            !authz.can(Capability::AdministerIam),
            "'{role}' must not administer IAM"
        );
        assert!(
            !authz.can(Capability::ViewAudit),
            "'{role}' must not read the audit log"
        );
    }
}

/// The console is exact `admin`. The roles PR #930 admitted (`admin_pusat`,
/// `superadmin`) administer the *application*, not the identity provider.
#[test]
fn only_the_iam_admin_role_reaches_iam_admin_routes() {
    for role in IAM_ADMIN_ROLES {
        assert!(Authorization::from_slice(&[role.to_string()]).is_iam_admin());
    }
    for role in ADMIN_ROLES.iter().filter(|r| !IAM_ADMIN_ROLES.contains(r)) {
        let authz = Authorization::from_slice(&[role.to_string()]);
        assert!(
            !authz.is_iam_admin(),
            "'{role}' must not administer the IdP"
        );
        assert!(
            authz.can(Capability::Administer),
            "'{role}' still administers the app"
        );
    }
}

/// Roles are database rows, so an operator can mint a name that *looks*
/// administrative. A prefix match would admit all of these.
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
            !is_iam_admin_role(lookalike),
            "'{lookalike}' must not pass the IAM admin guard"
        );
    }
}

/// A caller holding an admin role alongside others still passes — the guard
/// must scan the whole role set, since role order in `realm_access.roles` is
/// not guaranteed.
#[test]
fn admin_among_several_roles_still_passes() {
    let owned: Vec<String> = ["operator_satker", "validator_wilayah", "admin"]
        .iter()
        .map(|r| r.to_string())
        .collect();
    assert!(Authorization::from_slice(&owned).is_iam_admin());
}

/// An empty role claim grants nothing privileged. Since the guard no longer
/// falls back to a database lookup, "no roles in the token" means "no authority"
/// — which is also what makes session-scoped role activation meaningful.
#[test]
fn absent_roles_grant_nothing() {
    let none = Authorization::from_slice(&[]);
    assert!(!none.is_iam_admin());
    assert!(!none.can(Capability::AdministerIam));
    assert!(!none.can(Capability::Administer));
    assert!(!none.can(Capability::ViewAudit));
    assert!(!none.can(Capability::Create));
    assert!(!none.can(Capability::View));
}
