//! Who acted, as the audit trail prints them.
//!
//! An audit entry has to answer "who" for the person reading it a year later, so the actor is
//! recorded as a **snapshot at the time of the event**: the stable id, plus the name and NIP the
//! token carried at that moment. Resolving the name when the log is *read* gets it wrong the
//! day someone is renamed, transferred or deleted, and makes the trail depend on another
//! service being up (OWASP Logging Cheat Sheet; NIST SP 800-92).
//!
//! Most writers hold the caller's [`Claims`] and say who it was themselves. Three did not: the
//! workflow engine only receives a `user_id`, helpdesk tickets wrote `user:<uuid>`, and a
//! transition made by a background job has no person at all. Threading a name through every
//! service method between those writers and their handlers is wide and easy to get wrong
//! again, so the name is supplied at the single choke point instead: [`remember`] is called
//! every time a token is validated, and the audit sink asks [`resolve`] before it inserts.
//! The handler that produces the event always runs after the validation of the same request,
//! so the entry is the caller's current identity, not a guess.
//!
//! The directory is per process and bounded; a restart simply refills it on the next request
//! of each user, which always precedes any event that user can cause.
//!
//! ```text
//! label(Some("Budi Santoso"), Some("198001012005011001"), Some("budi"))
//!     == Some("Budi Santoso (198001012005011001)")
//! ```

use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

use uuid::Uuid;

use crate::shared::middleware::Claims;

/// What the trail shows for an event no person caused: a scheduler, a threshold reached by the
/// last of several approvals. The workflow engine runs those as the nil user.
pub const SISTEM: &str = "Sistem";

/// Upper bound on remembered identities. Far above the real number of active users; reaching it
/// clears the map instead of growing without limit, and the next request of each user refills
/// their entry.
const MAX_ENTRIES: usize = 20_000;

static DIRECTORY: LazyLock<RwLock<HashMap<Uuid, String>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// The handlers and the token layer fill absent fields with this placeholder; it is not a name.
const PLACEHOLDER: &str = "unknown";

fn usable(s: Option<&str>) -> Option<&str> {
    s.map(str::trim)
        .filter(|v| !v.is_empty() && !v.eq_ignore_ascii_case(PLACEHOLDER))
}

/// `Nama (NIP)` when both are known, else whichever is, else the login name. `None` only when
/// the token carried none of the three.
///
/// The NIP is part of the label because two people can share a name, and the NIP is the
/// identifier an auditor of a government system can actually check.
pub fn label(nama: Option<&str>, nip: Option<&str>, username: Option<&str>) -> Option<String> {
    let (nama, nip) = (usable(nama), usable(nip));
    match (nama, nip) {
        (Some(n), Some(p)) if n != p => Some(format!("{n} ({p})")),
        (Some(n), _) => Some(n.to_string()),
        (None, Some(p)) => Some(p.to_string()),
        (None, None) => usable(username).map(str::to_string),
    }
}

/// [`label`] of a caller's claims.
pub fn label_of(claims: &Claims) -> Option<String> {
    label(
        claims.name.as_deref().or(claims.nama.as_deref()),
        claims.nip.as_deref(),
        Some(claims.username.as_str()),
    )
}

/// Record who `claims` is, so an event written later in this request can name them.
pub fn remember(claims: &Claims) {
    let Some(label) = label_of(claims) else {
        return;
    };
    let Ok(mut dir) = DIRECTORY.write() else {
        return;
    };
    if dir.len() >= MAX_ENTRIES && !dir.contains_key(&claims.user_id) {
        dir.clear();
    }
    dir.insert(claims.user_id, label);
}

/// Is `given` something a writer put there because it had no name, rather than a name?
/// Empty, the `unknown` placeholder, and the `user:<uuid>` stand-in that helpdesk tickets used.
fn is_placeholder(given: &str, id: Uuid) -> bool {
    let given = given.trim();
    given.is_empty()
        || given.eq_ignore_ascii_case(PLACEHOLDER)
        || given.eq_ignore_ascii_case(&format!("user:{id}"))
}

/// The actor name an audit entry is stored with.
///
/// A name the writer supplied is kept as it is. Otherwise the nil user is [`SISTEM`], a user
/// seen by this process is their remembered label, and a user never seen (an event for someone
/// who has made no request since this process started) is `Pengguna` plus the start of their
/// id: still not blank, and the full id sits in its own column.
pub fn resolve(id: Uuid, given: Option<&str>) -> String {
    if let Some(g) = given.filter(|g| !is_placeholder(g, id)) {
        return g.trim().to_string();
    }
    if id.is_nil() {
        return SISTEM.to_string();
    }
    let known = DIRECTORY.read().ok().and_then(|d| d.get(&id).cloned());
    known.unwrap_or_else(|| format!("Pengguna {}", &id.simple().to_string()[..8]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_is_name_and_nip_when_both_are_known() {
        assert_eq!(
            label(Some("Budi Santoso"), Some("198001"), Some("budi")).as_deref(),
            Some("Budi Santoso (198001)")
        );
    }

    #[test]
    fn label_falls_back_name_then_nip_then_login() {
        assert_eq!(
            label(Some("Budi"), None, Some("b")).as_deref(),
            Some("Budi")
        );
        assert_eq!(
            label(None, Some("198001"), Some("b")).as_deref(),
            Some("198001")
        );
        assert_eq!(label(None, None, Some("budi")).as_deref(), Some("budi"));
    }

    #[test]
    fn label_never_prints_the_placeholder_or_blanks() {
        assert_eq!(
            label(Some("unknown"), Some("198001"), Some("b")).as_deref(),
            Some("198001")
        );
        assert_eq!(
            label(Some("  "), Some("UNKNOWN"), Some("budi")).as_deref(),
            Some("budi")
        );
        assert_eq!(label(None, None, Some("unknown")), None);
        assert_eq!(label(None, None, None), None);
    }

    #[test]
    fn a_nip_that_equals_the_name_is_not_printed_twice() {
        assert_eq!(
            label(Some("198001"), Some("198001"), None).as_deref(),
            Some("198001")
        );
    }

    #[test]
    fn a_name_the_writer_supplied_is_kept() {
        let id = Uuid::new_v4();
        assert_eq!(resolve(id, Some(" Budi (198001) ")), "Budi (198001)");
    }

    #[test]
    fn the_nil_user_is_the_system() {
        assert_eq!(resolve(Uuid::nil(), Some("")), SISTEM);
        assert_eq!(resolve(Uuid::nil(), None), SISTEM);
    }

    #[test]
    fn placeholders_are_replaced_by_the_remembered_identity() {
        let id = Uuid::new_v4();
        let claims = Claims {
            name: Some("Sari Dewi".into()),
            nip: Some("199001".into()),
            ..Claims::with_roles(id, "sari", ["operator_satker"])
        };
        remember(&claims);
        for given in [None, Some(""), Some("  "), Some("unknown")] {
            assert_eq!(resolve(id, given), "Sari Dewi (199001)", "given {given:?}");
        }
        // The stand-in the ticket writer used to emit.
        assert_eq!(
            resolve(id, Some(&format!("user:{id}"))),
            "Sari Dewi (199001)"
        );
    }

    #[test]
    fn a_user_never_seen_still_gets_a_readable_non_blank_name() {
        let id = Uuid::new_v4();
        let name = resolve(id, None);
        assert!(name.starts_with("Pengguna "), "{name}");
        assert_eq!(name.len(), "Pengguna ".len() + 8);
    }

    #[test]
    fn a_later_token_updates_the_snapshot() {
        let id = Uuid::new_v4();
        let mut c = Claims::with_roles(id, "lama", ["operator_satker"]);
        remember(&c);
        assert_eq!(resolve(id, None), "lama");
        c.username = "baru".into();
        remember(&c);
        assert_eq!(resolve(id, None), "baru");
    }
}
