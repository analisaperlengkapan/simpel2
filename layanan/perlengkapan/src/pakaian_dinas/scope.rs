//! Campaign-visibility scope for the pakaian-dinas pengajuan list (#72, the
//! last of the #66 tiered-RBAC trilogy — sibling of `kebutuhan_bmn::scope`).
//!
//! Like kebutuhan-BMN, a `pengajuan_pakaian_dinas` is a **pusat-authored
//! campaign** (created by `validator_pusat`) that TARGETS satkers and is
//! responded to per-satker. So the question is "does this campaign target the
//! caller?", not "does the caller own it". Two schema differences from
//! kebutuhan drive a bespoke condition here:
//!   - `scope_satker` is a nullable `varchar(20)` with NO check constraint, and
//!     legacy/create paths may write `'semua'`, `'all'`, or leave it NULL — all
//!     mean nationwide. We treat NULL / `'semua'` / `'all'` as nationwide.
//!   - the targeted-satker child `pengajuan_pakaian_dinas_satker_terpilih` keys
//!     satkers by MySIMKARI `kode_satker`, same as kebutuhan (V006/#94 — it
//!     used to be a `uuid` compared against a `bigint` PK, which was invalid
//!     SQL). The satker tier can therefore compare the column directly; only
//!     the wilayah tier still joins, and it joins `integrasi.v_satker_wilayah`
//!     — the one definition of the tier, shared with every other scope.
//!
//! Tiers (derived via [`SatkerScope`], reusing its role mapping):
//! - `All`     — cross-satker roles (pusat/validator_pusat/admin = the campaign
//!   authors): see everything.
//! - `Wilayah` — `validator_wilayah`: campaigns nationwide, targeting their
//!   wilayah, or targeting any satker in their wilayah.
//! - `Satker`  — operator/validator_satker: campaigns nationwide, targeting
//!   their wilayah, or explicitly targeting their satker.
//! - `Denied`  — no satker identity: fail-closed.
//!
//! No `created_by` escape hatch is needed: campaigns can only be authored by
//! `validator_pusat`, who is a cross-satker (`All`) role and already sees all.

use crate::shared::satker_scope::SatkerScope;

type BoxedParam = Box<dyn tokio_postgres::types::ToSql + Sync + Send>;

/// Build the campaign-visibility predicate to be ANDed onto a query whose outer
/// alias for `perlengkapan.pengajuan_pakaian_dinas` is `p` (so the predicate
/// keys off `p.id`). Pushes the caller's MySIMKARI `kode_satker` bind param
/// (when any) onto `params`; returns `None` for `All` (no filter).
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
                "p.id IN (SELECT pp.id FROM perlengkapan.pengajuan_pakaian_dinas pp \
                 WHERE pp.scope_satker IS NULL OR pp.scope_satker IN ('semua', 'all') \
                    OR (pp.scope_satker = 'wilayah' AND pp.wilayah_id = \
                        (SELECT s.wilayah_code FROM integrasi.v_satker_wilayah s \
                         WHERE s.kode_satker = ${i})) \
                    OR EXISTS (SELECT 1 FROM perlengkapan.pengajuan_pakaian_dinas_satker_terpilih pt \
                        WHERE pt.pengajuan_id = pp.id AND pt.satker_id = ${i}))"
            ))
        }
        SatkerScope::Wilayah(code) => {
            params.push(Box::new(code.clone()));
            let i = params.len();
            Some(format!(
                "p.id IN (SELECT pp.id FROM perlengkapan.pengajuan_pakaian_dinas pp \
                 WHERE pp.scope_satker IS NULL OR pp.scope_satker IN ('semua', 'all') \
                    OR (pp.scope_satker = 'wilayah' AND pp.wilayah_id = \
                        (SELECT s.wilayah_code FROM integrasi.v_satker_wilayah s \
                         WHERE s.kode_satker = ${i})) \
                    OR EXISTS (SELECT 1 FROM perlengkapan.pengajuan_pakaian_dinas_satker_terpilih pt \
                        JOIN integrasi.v_satker_wilayah ms ON ms.kode_satker = pt.satker_id \
                        WHERE pt.pengajuan_id = pp.id AND ms.wilayah_code = \
                          (SELECT s2.wilayah_code FROM integrasi.v_satker_wilayah s2 \
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
    fn satker_sees_nationwide_wilayah_and_explicit_targets() {
        let mut p: Vec<BoxedParam> = Vec::new();
        let cond = campaign_visibility_condition(&SatkerScope::Satker("02.28".to_string()), &mut p)
            .unwrap();
        // nationwide includes NULL / 'semua' / 'all'
        assert!(cond.contains("pp.scope_satker IS NULL"));
        assert!(cond.contains("'semua', 'all'"));
        // wilayah tier resolves the caller's wilayah name
        assert!(cond.contains("pp.scope_satker = 'wilayah'"));
        // Explicit targets compare kode_satker DIRECTLY — no join through
        // the wilayah view, because the column now holds kode_satker (V006/#94).
        assert!(cond.contains("pengajuan_pakaian_dinas_satker_terpilih pt"));
        assert!(cond.contains("pt.satker_id = $1"));
        assert!(
            !cond.contains("JOIN integrasi.v_satker_wilayah ms"),
            "satker tier must not need the surrogate-id join any more"
        );
        assert_eq!(p.len(), 1);
    }

    #[test]
    fn wilayah_matches_any_satker_in_region() {
        let mut p: Vec<BoxedParam> = Vec::new();
        let cond =
            campaign_visibility_condition(&SatkerScope::Wilayah("02.28".to_string()), &mut p)
                .unwrap();
        assert!(
            cond.contains("JOIN integrasi.v_satker_wilayah ms ON ms.kode_satker = pt.satker_id")
        );
        assert!(cond.contains("ms.wilayah_code ="));
        // The column this tier used to read groups 15 Kejati together; it must
        // not come back.
        assert!(!cond.contains("mysimkari_satker"));
        assert!(cond.contains("$1"));
        assert_eq!(p.len(), 1);
    }
}
