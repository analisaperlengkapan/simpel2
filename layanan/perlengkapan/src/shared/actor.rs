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
//! again, so the request's own identity is made available where the event is written:
//! [`scope`] wraps the rest of the request (the authentication layer calls it right after the
//! token is validated), and the audit sink asks [`resolve`] before it inserts.
//!
//! It is a task-local, so it belongs to THIS request: two sessions of one user that carry
//! different names (a rename while an older token is still valid) each record their own, which a
//! per-process directory keyed by user could not promise. The request handler and everything it
//! awaits run inside the scope; a writer that hands the event to `tokio::spawn` (helpdesk
//! tickets) resolves the name first, because a spawned task does not inherit it.
//!
//! ```text
//! label(Some("Budi Santoso"), Some("198001012005011001"), Some("budi"))
//!     == Some("Budi Santoso (198001012005011001)")
//! ```

use std::future::Future;

use uuid::Uuid;

use crate::shared::middleware::Claims;

/// What the trail shows for an event no person caused: a scheduler, a threshold reached by the
/// last of several approvals. The workflow engine runs those as the nil user.
pub const SISTEM: &str = "Sistem";

/// `perlengkapan.audit_log.actor_username` is `varchar(255)`. A label over that makes Postgres
/// refuse the WHOLE entry, and the callers swallow the error, so a long enough name would erase
/// the action from the trail instead of merely shortening it.
pub const MAKS_PANJANG: usize = 255;

/// The longest name kept before the NIP: leaves room for ` (` + a NIP + `)` inside
/// [`MAKS_PANJANG`], so the NIP, the part an auditor can check, is never the part cut off.
const MAKS_NAMA: usize = 200;
const MAKS_NIP: usize = 30;

/// The handlers and the token layer fill absent fields with this placeholder; it is not a name.
const PLACEHOLDER: &str = "unknown";

#[derive(Debug, Clone)]
struct Pelaku {
    id: Uuid,
    label: Option<String>,
}

tokio::task_local! {
    static PELAKU: Pelaku;
}

/// Run `kerja` as the request of `claims`: any audit event it writes for that user names them as
/// this request's token does.
pub async fn scope<F: Future>(claims: &Claims, kerja: F) -> F::Output {
    PELAKU
        .scope(
            Pelaku {
                id: claims.user_id,
                label: label_of(claims),
            },
            kerja,
        )
        .await
}

fn usable(s: Option<&str>) -> Option<&str> {
    s.map(str::trim)
        .filter(|v| !v.is_empty() && !v.eq_ignore_ascii_case(PLACEHOLDER))
}

/// At most `n` characters (not bytes: the column counts characters), with an ellipsis when cut.
fn potong(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        return s.to_string();
    }
    let mut out: String = s.chars().take(n.saturating_sub(1)).collect();
    out.push('…');
    out
}

/// `Nama (NIP)` when both are known, else whichever is, else the login name. `None` only when
/// the token carried none of the three. Never longer than [`MAKS_PANJANG`].
///
/// The NIP is part of the label because two people can share a name, and the NIP is the
/// identifier an auditor of a government system can actually check.
pub fn label(nama: Option<&str>, nip: Option<&str>, username: Option<&str>) -> Option<String> {
    let (nama, nip) = (usable(nama), usable(nip));
    match (nama, nip) {
        (Some(n), Some(p)) if n != p => Some(format!(
            "{} ({})",
            potong(n, MAKS_NAMA),
            potong(p, MAKS_NIP)
        )),
        (Some(n), _) => Some(potong(n, MAKS_PANJANG)),
        (None, Some(p)) => Some(potong(p, MAKS_PANJANG)),
        (None, None) => usable(username).map(|u| potong(u, MAKS_PANJANG)),
    }
}

/// [`label`] of a caller's claims.
///
/// An authenc that does not know a login hands the layer the user's UUID in its place; that is
/// not a login name, and printing it as one would make the trail unsearchable by the name the
/// person actually signs in with.
pub fn label_of(claims: &Claims) -> Option<String> {
    let login = claims.username.as_str();
    let login = (login != claims.user_id.to_string()).then_some(login);
    label(
        claims.name.as_deref().or(claims.nama.as_deref()),
        claims.nip.as_deref(),
        login,
    )
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
/// A name the writer supplied is kept (cut to fit the column). Otherwise the nil user is
/// [`SISTEM`], the user of the current request is their label from this request's token, and
/// anyone else (an event for a user who is not the one making this request, or one written
/// outside any request) is `Pengguna` plus the start of their id: still not blank, and the full
/// id sits in its own column.
pub fn resolve(id: Uuid, given: Option<&str>) -> String {
    if let Some(g) = given.filter(|g| !is_placeholder(g, id)) {
        return potong(g.trim(), MAKS_PANJANG);
    }
    if id.is_nil() {
        return SISTEM.to_string();
    }
    PELAKU
        .try_with(|p| (p.id == id).then(|| p.label.clone()).flatten())
        .ok()
        .flatten()
        .unwrap_or_else(|| format!("Pengguna {}", &id.simple().to_string()[..8]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn claims(id: Uuid, login: &str, nama: Option<&str>, nip: Option<&str>) -> Claims {
        Claims {
            name: nama.map(str::to_string),
            nip: nip.map(str::to_string),
            ..Claims::with_roles(id, login, ["operator_satker"])
        }
    }

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
    fn a_label_never_exceeds_the_column_and_keeps_the_nip() {
        // The user table allows long names; the audit column holds 255 characters and a longer
        // value would make Postgres refuse the whole entry.
        let nama = "Å".repeat(240);
        let nip = "198001012005011001";
        let l = label(Some(&nama), Some(nip), None).unwrap();
        assert!(l.chars().count() <= MAKS_PANJANG, "{}", l.chars().count());
        assert!(
            l.ends_with(&format!("({nip})")),
            "the NIP survives the cut: {l}"
        );
        assert!(l.contains('…'));
        let sendiri = label(Some(&"x".repeat(400)), None, None).unwrap();
        assert_eq!(sendiri.chars().count(), MAKS_PANJANG);
        assert_eq!(
            resolve(Uuid::new_v4(), Some(&"y".repeat(400)))
                .chars()
                .count(),
            MAKS_PANJANG
        );
    }

    #[test]
    fn a_login_that_is_just_the_users_uuid_is_not_a_login() {
        let id = Uuid::new_v4();
        let c = claims(id, &id.to_string(), None, None);
        assert_eq!(
            label_of(&c),
            None,
            "the placeholder is not searchable as a login"
        );
        let c = claims(id, "operator_bmn", None, None);
        assert_eq!(label_of(&c).as_deref(), Some("operator_bmn"));
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

    #[tokio::test]
    async fn placeholders_are_replaced_by_this_requests_identity() {
        let id = Uuid::new_v4();
        let c = claims(id, "sari", Some("Sari Dewi"), Some("199001"));
        scope(&c, async {
            for given in [None, Some(""), Some("  "), Some("unknown")] {
                assert_eq!(resolve(id, given), "Sari Dewi (199001)", "given {given:?}");
            }
            // The stand-in the ticket writer used to emit.
            assert_eq!(
                resolve(id, Some(&format!("user:{id}"))),
                "Sari Dewi (199001)"
            );
            // An event for ANOTHER user is not this request's person.
            let lain = Uuid::new_v4();
            assert!(resolve(lain, None).starts_with("Pengguna "));
        })
        .await;
    }

    #[tokio::test]
    async fn two_sessions_of_one_user_each_record_their_own_name() {
        // A rename while an older token is still valid: both tokens are valid, they carry
        // different names, and the entry must carry the name of the token that acted.
        let id = Uuid::new_v4();
        let lama = claims(id, "ani", Some("Ani"), Some("199001"));
        let baru = claims(id, "ani", Some("Anisa"), Some("199001"));
        let (a, b) = tokio::join!(
            scope(&lama, async {
                tokio::task::yield_now().await;
                resolve(id, None)
            }),
            scope(&baru, async {
                tokio::task::yield_now().await;
                resolve(id, None)
            }),
        );
        assert_eq!(a, "Ani (199001)");
        assert_eq!(b, "Anisa (199001)");
    }

    #[tokio::test]
    async fn outside_any_request_a_user_still_gets_a_readable_non_blank_name() {
        let id = Uuid::new_v4();
        let name = resolve(id, None);
        assert!(name.starts_with("Pengguna "), "{name}");
        assert_eq!(name.len(), "Pengguna ".len() + 8);
    }
}
