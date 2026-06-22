//! Campaign-visibility scope for the kebutuhan-BMN pengajuan list (#66 part 3).
//!
//! Unlike pemakaian/penghapusan (one record = one owning satker), a kebutuhan
//! `pengajuan_kebutuhan_bmn` is a **pusat-authored RKBMN campaign** (created by
//! `validator_pusat`) that TARGETS satkers and is responded to per-satker via
//! `pengajuan_kebutuhan_bmn_satker` children. So scoping is "does this campaign
//! target the caller?", not "does the caller own it":
//!
//! - `scope_satker = 'semua'`   → nationwide: every satker sees it.
//! - `scope_satker = 'wilayah'` → `wilayah_id` (a wilayah NAME, matches
//!   `integrasi.mysimkari_satker.wilayah`): satkers in that wilayah see it.
//! - `scope_satker = 'sebagian'`→ only the satkers listed as children see it.
//!
//! Tiers (derived via [`SatkerScope`], reusing its role mapping):
//! - `All`     — cross-satker roles (pusat/validator_pusat/admin = the campaign
//!   authors): see everything.
//! - `Wilayah` — `validator_wilayah`: campaigns targeting their wilayah.
//! - `Satker`  — operator/validator_satker: campaigns targeting their satker.
//! - `Denied`  — no satker identity: fail-closed.
//!
//! No `created_by` escape hatch is needed: campaigns can only be authored by
//! `validator_pusat`, who is a cross-satker (`All`) role and already sees all.

use crate::shared::satker_scope::SatkerScope;

type BoxedParam = Box<dyn tokio_postgres::types::ToSql + Sync + Send>;

/// Build the campaign-visibility predicate to be ANDed onto a query whose `id`
/// column equals `perlengkapan.pengajuan_kebutuhan_bmn.id` (e.g. the
/// `vw_kebutuhan_bmn_summary` view). Pushes the caller's MySIMKARI `kode_satker`
/// bind param (when any) onto `params`; returns `None` for `All` (no filter).
///
/// The bound `$n` placeholder is referenced multiple times within the returned
/// subquery — that is valid (one param, many references).
pub fn campaign_visibility_condition(
    scope: &SatkerScope,
    params: &mut Vec<BoxedParam>,
) -> Option<String> {
    match scope {
        SatkerScope::All => None,
        SatkerScope::Denied => Some("FALSE".to_string()),
        SatkerScope::Satker(code) => {
            params.push(Box::new(code.clone()));
            let i = params.len();
            Some(format!(
                "id IN (SELECT p.id FROM perlengkapan.pengajuan_kebutuhan_bmn p \
                 WHERE p.scope_satker = 'semua' \
                    OR (p.scope_satker = 'wilayah' AND p.wilayah_id = \
                        (SELECT s.wilayah FROM integrasi.mysimkari_satker s \
                         WHERE s.kode_satker = ${i})) \
                    OR EXISTS (SELECT 1 FROM perlengkapan.pengajuan_kebutuhan_bmn_satker ps \
                        WHERE ps.pengajuan_id = p.id AND ps.satker_id = ${i}))"
            ))
        }
        SatkerScope::Wilayah(code) => {
            params.push(Box::new(code.clone()));
            let i = params.len();
            Some(format!(
                "id IN (SELECT p.id FROM perlengkapan.pengajuan_kebutuhan_bmn p \
                 WHERE p.scope_satker = 'semua' \
                    OR (p.scope_satker = 'wilayah' AND p.wilayah_id = \
                        (SELECT s.wilayah FROM integrasi.mysimkari_satker s \
                         WHERE s.kode_satker = ${i})) \
                    OR EXISTS (SELECT 1 FROM perlengkapan.pengajuan_kebutuhan_bmn_satker ps \
                        JOIN integrasi.mysimkari_satker ms ON ms.kode_satker = ps.satker_id \
                        WHERE ps.pengajuan_id = p.id AND ms.wilayah = \
                          (SELECT s2.wilayah FROM integrasi.mysimkari_satker s2 \
                           WHERE s2.kode_satker = ${i})))"
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_adds_no_predicate() {
        let mut p: Vec<BoxedParam> = Vec::new();
        assert!(campaign_visibility_condition(&SatkerScope::All, &mut p).is_none());
        assert_eq!(p.len(), 0);
    }

    #[test]
    fn denied_is_false() {
        let mut p: Vec<BoxedParam> = Vec::new();
        assert_eq!(
            campaign_visibility_condition(&SatkerScope::Denied, &mut p).as_deref(),
            Some("FALSE")
        );
        assert_eq!(p.len(), 0);
    }

    #[test]
    fn satker_targets_semua_wilayah_and_children() {
        let mut p: Vec<BoxedParam> = Vec::new();
        let cond =
            campaign_visibility_condition(&SatkerScope::Satker("02.28".to_string()), &mut p)
                .unwrap();
        assert!(cond.contains("scope_satker = 'semua'"));
        assert!(cond.contains("p.scope_satker = 'wilayah'"));
        assert!(cond.contains("pengajuan_kebutuhan_bmn_satker ps"));
        assert!(cond.contains("ps.satker_id = $1"));
        assert_eq!(p.len(), 1);
    }

    #[test]
    fn wilayah_matches_any_satker_in_region() {
        let mut p: Vec<BoxedParam> = Vec::new();
        let cond =
            campaign_visibility_condition(&SatkerScope::Wilayah("02.28".to_string()), &mut p)
                .unwrap();
        assert!(cond.contains("JOIN integrasi.mysimkari_satker ms"));
        assert!(cond.contains("ms.wilayah ="));
        assert!(cond.contains("$1"));
        assert_eq!(p.len(), 1);
    }
}
