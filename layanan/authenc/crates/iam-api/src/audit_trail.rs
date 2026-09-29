//! Audit trail for **administrative actions** on the identity provider.
//!
//! Before this, creating or deleting a user, resetting a password, switching a
//! user's MFA on or off, and granting or revoking a role wrote **no audit row at
//! all**: `security_monitoring_middleware` records only login/logout/revoke
//! requests, authorisation failures and suspicious traffic. The events the
//! admin UI already had labels for (`USER_CREATE`, `ROLE_ASSIGN`, …) were never
//! emitted by anything — so the one place a privilege change would be visible
//! was blank.
//!
//! OWASP Logging Cheat Sheet: every privilege grant, role change, account
//! creation/deletion and security toggle is logged with **actor, action,
//! target and what changed**. That is the shape of [`AdminAction`].
//!
//! The write happens *after* the change succeeds and is best-effort in the
//! sense that a failed audit insert is reported loudly (`error!`) rather than
//! undoing an already-committed change — but it is never skipped silently, and
//! the caller's outcome is unaffected.

use std::net::IpAddr;

use serde_json::{Value, json};
use uuid::Uuid;

use crate::middleware::admin_auth::{AdminUser, ClientAddr};
use crate::state::IamApiState;

/// One administrative action, as it will appear in `audit_logs`.
pub struct AdminAction {
    /// Machine-readable event tag, e.g. `USER_CREATE`, `ROLE_ASSIGN`. Stored as
    /// both `event_type` and `action` (`action` is NOT NULL, ≤50 chars).
    pub event: &'static str,
    /// What kind of thing was acted on: `user`, `role_assignment`, …
    pub resource_type: &'static str,
    /// The id of the target, when it has one.
    pub resource_id: Option<Uuid>,
    /// What changed. **Never put a secret here** — no passwords, no tokens; name
    /// the fields that changed, not their sensitive values.
    pub details: Value,
}

/// Record an administrative action performed by `admin`.
pub async fn record_admin_action(
    state: &IamApiState,
    admin: &AdminUser,
    addr: ClientAddr,
    action: AdminAction,
) {
    let ip: Option<IpAddr> = addr.0;
    let details = json!({
        "actor": {
            "user_id": admin.user_id,
            "username": admin.username,
            "roles": admin.roles,
        },
        "change": action.details,
    });

    // `user_id` is the ACTOR; the FK to users is nulled by subselect so an audit
    // write is never rejected because a row was deleted in the meantime.
    let result = state
        .database
        .execute(
            r#"INSERT INTO audit_logs
                   ("timestamp", event_type, user_id, resource_type, resource_id,
                    action, status, details, ip_address)
               VALUES (NOW(), $1, (SELECT id FROM users WHERE id = $2), $3, $4,
                       $1, 'success', $5, $6)"#,
            &[
                &action.event,
                &admin.user_id,
                &action.resource_type,
                &action.resource_id,
                &details,
                &ip,
            ],
        )
        .await;

    if let Err(e) = result {
        // Loud: a privileged change with no audit row is exactly what an
        // operator must be told about.
        tracing::error!(
            event = action.event,
            actor = %admin.user_id,
            "FAILED to write admin audit row for a committed change: {e}"
        );
    }
}

/// The names of the fields that a user update actually carries, for the audit
/// `details` — names only, so no PII or credential value is copied into the log.
pub fn changed_fields(pairs: &[(&'static str, bool)]) -> Vec<&'static str> {
    pairs
        .iter()
        .filter(|(_, present)| *present)
        .map(|(name, _)| *name)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changed_fields_lists_names_of_present_fields_only() {
        let got = changed_fields(&[("email", true), ("nip", false), ("enabled", true)]);
        assert_eq!(got, vec!["email", "enabled"]);
        assert!(changed_fields(&[("email", false)]).is_empty());
    }
}
