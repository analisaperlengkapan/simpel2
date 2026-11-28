//! Where a notification's `action_url` really points.
//!
//! The notifikasi service stores `action_url` as a path inside the module that raised the
//! notification (`/pakaian-dinas/pengajuan/<id>/satker`, `/pengelolaan/pemakaian/detail/<id>`):
//! the Perlengkapan router's own vocabulary, WITHOUT the place it is mounted at. Handed to the
//! browser as a plain `href`, such a path leaves the app for the host root, where nothing serves it:
//! a blank 404 (seen on production, from the inbox's "Buka" link). Every place that opens a
//! notification — the portal's bell and list, Perlengkapan's bell and inbox — resolves it here, so
//! they cannot disagree about it.

/// Where the Perlengkapan SPA is mounted (the `Router` `base` of `antarmuka/perlengkapan`).
pub const PERLENGKAPAN_BASE: &str = "/perlengkapan/simpel";

/// Mount points a path may already carry. Such a path is complete as it stands.
const MOUNTED_ROOTS: [&str; 2] = ["/portal", "/perlengkapan"];

/// The URL to open for a notification's `action_url`, or `None` when it must not be opened.
///
/// * A module-relative path (`/pakaian-dinas/...`) gets the Perlengkapan mount point.
/// * A path that already starts with a mount point (`/portal/...`, `/perlengkapan/simpel/...`) is kept.
/// * Query and fragment survive untouched.
/// * Anything that is not a plain root-relative path is refused: an absolute URL, `javascript:` /
///   `data:`, a protocol-relative `//host`, a backslash (some browsers read it as `/`), or a control
///   character. The producer only ever writes paths, so refusing the rest costs nothing, while
///   following it would let a stored row send a person to another site or run script.
pub fn resolve_action_url(raw: &str) -> Option<String> {
    let url = raw.trim();
    if !url.starts_with('/')
        || url.starts_with("//")
        || url.contains('\\')
        || url.chars().any(char::is_control)
    {
        return None;
    }
    // `/portal` and `/portal/x` are the mount point, `/portalx` is not.
    let already_mounted = MOUNTED_ROOTS.iter().any(|root| {
        url.strip_prefix(root)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with(['/', '?', '#']))
    });
    Some(if already_mounted {
        url.to_string()
    } else {
        format!("{PERLENGKAPAN_BASE}{url}")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_module_relative_path_gets_the_perlengkapan_mount_point() {
        assert_eq!(
            resolve_action_url("/pakaian-dinas/pengajuan/4f28e352/satker").as_deref(),
            Some("/perlengkapan/simpel/pakaian-dinas/pengajuan/4f28e352/satker")
        );
        assert_eq!(
            resolve_action_url("/pengelolaan/pemakaian/detail/d0283c05").as_deref(),
            Some("/perlengkapan/simpel/pengelolaan/pemakaian/detail/d0283c05")
        );
    }

    #[test]
    fn a_path_that_is_already_mounted_is_left_alone() {
        for already in [
            "/perlengkapan/simpel/pakaian-dinas/pengajuan/1/satker",
            "/perlengkapan/simpel",
            "/portal/notifications",
            "/portal",
        ] {
            assert_eq!(resolve_action_url(already).as_deref(), Some(already));
        }
    }

    #[test]
    fn a_name_that_only_starts_like_a_mount_point_is_not_one() {
        // `/portalx` is a Perlengkapan route that happens to share a prefix, not the portal.
        assert_eq!(
            resolve_action_url("/portalx/a").as_deref(),
            Some("/perlengkapan/simpel/portalx/a")
        );
        assert_eq!(
            resolve_action_url("/perlengkapanku").as_deref(),
            Some("/perlengkapan/simpel/perlengkapanku")
        );
    }

    #[test]
    fn query_and_fragment_survive() {
        assert_eq!(
            resolve_action_url("/pakaian-dinas/laporan?pengajuan_id=7#rekap").as_deref(),
            Some("/perlengkapan/simpel/pakaian-dinas/laporan?pengajuan_id=7#rekap")
        );
        assert_eq!(
            resolve_action_url("/portal?x=1").as_deref(),
            Some("/portal?x=1")
        );
        assert_eq!(
            resolve_action_url("/portal#top").as_deref(),
            Some("/portal#top")
        );
    }

    #[test]
    fn surrounding_whitespace_is_ignored() {
        assert_eq!(
            resolve_action_url("  /dashboard \n").as_deref(),
            Some("/perlengkapan/simpel/dashboard")
        );
    }

    #[test]
    fn anything_that_is_not_a_plain_path_is_refused() {
        for bad in [
            "",
            "   ",
            "pakaian-dinas/pengajuan",
            "https://evil.example/phish",
            "http://simpel.kejaksaan.go.id/portal",
            "javascript:alert(1)",
            "JaVaScRiPt:alert(1)",
            "data:text/html;base64,PHNjcmlwdD4=",
            "//evil.example/x",
            "/\\evil.example",
            "/a\\b",
            "/a\nb",
            "/a\u{0}b",
        ] {
            assert_eq!(resolve_action_url(bad), None, "{bad:?} must be refused");
        }
    }
}
