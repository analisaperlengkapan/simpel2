//! Tiered RBAC data-visibility scope for SIMAN asset (`bank_aset`) queries.
//!
//! SIMAN assets live in `integrasi.siman_aset`, keyed by the SIMAN finance code
//! `kdsatker_keu`. The caller identity carried in the JWT is a MySIMKARI
//! `kode_satker` ([`Claims::satker_code`]) — a DIFFERENT code system (the two
//! are disjoint; only `nama_satker` overlaps — see integrasi migration
//! `003_satker_code_mapping.sql`). The tier predicates therefore map the caller
//! `kode_satker` → SIMAN `kdsatker_keu` / `wilayah_kode` via the cross-reference
//! view `integrasi.v_satker_code_map`.
//!
//! Tiers:
//! - [`AsetScope::All`]     — admin / pusat / validator_pusat (cross-satker
//!   roles, see [`Claims::is_cross_satker_role`]): no row restriction.
//! - [`AsetScope::Wilayah`] — `validator_wilayah`: every satker whose wilayah
//!   matches the caller's wilayah(s).
//! - [`AsetScope::Satker`]  — `operator_satker` / `validator_satker` (and any
//!   other satker-bound role): only the caller's own satker.
//! - [`AsetScope::Denied`]  — authenticated but no usable satker identity, or a
//!   satker that has no mapping: FAIL CLOSED (zero rows).
//!
//! Fail-closed note: when the caller's `kode_satker` has no row in
//! `v_satker_code_map` (e.g. an unmapped/unverified satker), the `Satker`/
//! `Wilayah` subqueries return the empty set, so `IN (…)` yields no rows. This
//! is intentional — an unmapped satker sees nothing rather than everything.
//!
//! WILAYAH MEANS THE SAME THING HERE AS EVERYWHERE ELSE — but it is spelled
//! differently, and that is deliberate. Every MySIMKARI-side scope resolves the
//! tier through `integrasi.v_satker_wilayah` (the Kejaksaan Tinggi above a
//! satker). This one keeps reading SIMAN's own `wilayah_kode`, because the two
//! are a measured bijection: across the 488 staging satkers carrying both, no
//! Kejati splits across two `wilayah_kode` and no `wilayah_kode` is shared by
//! two Kejati. Joining the view in here instead would add a join to the exact
//! predicate that caused the 11 s → sub-second incident in integrasi migration
//! 004, buying nothing today.
//!
//! That equivalence is a premise, so it is pinned by a test rather than left to
//! hold by luck: `wilayah_kode_and_kejati_are_the_same_partition` in
//! `tests/integration/satker_wilayah_test.rs` fails the build if MySIMKARI's
//! hierarchy and SIMAN's regional code ever stop agreeing. Without it, a
//! divergence would surface only as two different populations counted side by
//! side on the monitoring summary — silently.
//!
//! Keeping the prefix form also keeps SIMAN-only satkers visible: rows whose
//! `kdsatker_keu` is in the region but whose `kode_satker` MySIMKARI does not
//! know (65 such rows on staging) still belong to the wilayah's asset picture.

use crate::shared::middleware::Claims;

/// Boxed bind parameter compatible with the repository's dynamic-SQL builders.
type BoxedParam = Box<dyn tokio_postgres::types::ToSql + Sync + Send>;

/// Data-visibility scope for a single request against `integrasi.siman_aset`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AsetScope {
    /// No row restriction (cross-satker roles).
    All,
    /// Restrict to the caller's wilayah. Holds the caller MySIMKARI `kode_satker`.
    Wilayah(String),
    /// Restrict to ONE region by its SIMAN `wilayah_kode` (digits 6-9 of
    /// `kdsatker_keu`). Produced only by [`AsetScope::narrowed_to_wilayah`];
    /// never derived from claims.
    WilayahKode(String),
    /// Restrict to the caller's own satker. Holds the caller MySIMKARI `kode_satker`.
    Satker(String),
    /// Fail closed — no satker identity. Yields zero rows.
    Denied,
}

impl AsetScope {
    /// Derive the visibility scope from the authenticated caller's claims.
    pub fn from_claims(claims: &Claims) -> Self {
        // Cross-satker roles (admin / admin_pusat / superadmin / validator_pusat
        // / pusat / analis_pusat) see everything.
        if claims.is_cross_satker_role() {
            return Self::All;
        }
        // Everyone else needs a satker identity; without one, fail closed.
        let Some(code) = claims
            .satker_code
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        else {
            return Self::Denied;
        };
        match claims.role.to_ascii_lowercase().as_str() {
            "validator_wilayah" => Self::Wilayah(code.to_string()),
            // operator_satker, validator_satker, and any other satker-bound role.
            _ => Self::Satker(code.to_string()),
        }
    }

    /// Narrow to a SINGLE satker for a drill-down. Mirrors
    /// [`crate::shared::satker_scope::SatkerScope::narrowed_to`] so the two
    /// halves of a filtered dashboard cannot end up describing different
    /// populations.
    ///
    /// `code` is a MySIMKARI `kode_satker`; the mapping to `kdsatker_keu`
    /// happens in the predicate, as it does for the untouched tiers.
    pub fn narrowed_to(&self, code: &str, in_scope: bool) -> Self {
        let allowed = match self {
            Self::All => true,
            Self::Denied => false,
            Self::Satker(own) => own == code,
            Self::Wilayah(_) | Self::WilayahKode(_) => in_scope,
        };
        if allowed {
            Self::Satker(code.to_string())
        } else {
            Self::Denied
        }
    }

    /// Narrow to one region by SIMAN `wilayah_kode`. Only a reader who already
    /// sees everything may pick an arbitrary region; a regional validator may
    /// pick only their own, and a satker-tier reader may not widen to one.
    pub fn narrowed_to_wilayah(&self, wilayah_kode: &str, caller_wilayah: Option<&str>) -> Self {
        match self {
            Self::All => Self::WilayahKode(wilayah_kode.to_string()),
            Self::Wilayah(_) if caller_wilayah == Some(wilayah_kode) => {
                Self::WilayahKode(wilayah_kode.to_string())
            }
            _ => Self::Denied,
        }
    }

    /// Append this scope as a SQL condition over `integrasi.siman_aset` (bare
    /// `kdsatker_keu` column). Pushes the bind parameter (when any) onto
    /// `params` and returns the condition string, or `None` when no restriction
    /// applies ([`AsetScope::All`]).
    ///
    /// The returned condition is safe to splice into a `WHERE`/`AND` clause: it
    /// uses only positional placeholders (`$n`) for caller-supplied values and
    /// otherwise references fixed column/view names.
    pub fn push_condition(&self, params: &mut Vec<BoxedParam>) -> Option<String> {
        match self {
            Self::All => None,
            Self::Denied => Some("FALSE".to_string()),
            Self::Satker(code) => {
                params.push(Box::new(code.clone()));
                let i = params.len();
                Some(format!(
                    "kdsatker_keu IN (SELECT kdsatker_keu FROM integrasi.v_satker_code_map \
                     WHERE kode_satker = ${i} AND kdsatker_keu IS NOT NULL)"
                ))
            }
            Self::Wilayah(code) => {
                params.push(Box::new(code.clone()));
                let i = params.len();
                Some(format!(
                    "substring(kdsatker_keu FROM 6 FOR 4) IN \
                     (SELECT wilayah_kode FROM integrasi.v_satker_code_map \
                      WHERE kode_satker = ${i} AND wilayah_kode IS NOT NULL)"
                ))
            }
            Self::WilayahKode(wilayah) => {
                params.push(Box::new(wilayah.clone()));
                let i = params.len();
                // The region compared directly, which is what the tier above
                // resolves to anyway — one less subquery for the same rows.
                Some(format!("substring(kdsatker_keu FROM 6 FOR 4) = ${i}"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn claims(role: &str, satker: Option<&str>) -> Claims {
        Claims {
            user_id: Uuid::nil(),
            username: "u".to_string(),
            role: role.to_string(),
            permissions: vec![],
            nip: None,
            name: None,
            nama: None,
            jabatan: None,
            satker_code: satker.map(|s| s.to_string()),
        }
    }

    #[test]
    fn cross_satker_roles_are_unrestricted() {
        for role in [
            "admin",
            "admin_pusat",
            "superadmin",
            "validator_pusat",
            "pusat",
            "analis_pusat",
        ] {
            // Even with a satker_code present, cross-satker roles see all.
            assert_eq!(
                AsetScope::from_claims(&claims(role, Some("02.28"))),
                AsetScope::All,
                "role {role} should be All"
            );
        }
    }

    #[test]
    fn validator_wilayah_is_wilayah_scoped() {
        assert_eq!(
            AsetScope::from_claims(&claims("validator_wilayah", Some("02.28"))),
            AsetScope::Wilayah("02.28".to_string())
        );
    }

    #[test]
    fn operator_and_validator_satker_are_satker_scoped() {
        for role in ["operator_satker", "validator_satker", "operator"] {
            assert_eq!(
                AsetScope::from_claims(&claims(role, Some("02.28"))),
                AsetScope::Satker("02.28".to_string()),
                "role {role} should be Satker"
            );
        }
    }

    #[test]
    fn missing_or_blank_satker_fails_closed() {
        assert_eq!(
            AsetScope::from_claims(&claims("operator_satker", None)),
            AsetScope::Denied
        );
        assert_eq!(
            AsetScope::from_claims(&claims("operator_satker", Some("   "))),
            AsetScope::Denied
        );
    }

    #[test]
    fn role_matching_is_case_insensitive() {
        assert_eq!(
            AsetScope::from_claims(&claims("Validator_Wilayah", Some("02.28"))),
            AsetScope::Wilayah("02.28".to_string())
        );
    }

    #[test]
    fn push_condition_all_adds_nothing() {
        let mut params: Vec<BoxedParam> = Vec::new();
        assert!(AsetScope::All.push_condition(&mut params).is_none());
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn push_condition_denied_is_false_constant_no_param() {
        let mut params: Vec<BoxedParam> = Vec::new();
        let cond = AsetScope::Denied.push_condition(&mut params).unwrap();
        assert_eq!(cond, "FALSE");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn push_condition_satker_uses_next_placeholder() {
        // Simulate one pre-existing param so the scope must bind to $2.
        let mut params: Vec<BoxedParam> = vec![Box::new(1i64)];
        let cond = AsetScope::Satker("02.28".to_string())
            .push_condition(&mut params)
            .unwrap();
        assert!(cond.contains("kdsatker_keu IN"));
        assert!(cond.contains("kode_satker = $2"));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn push_condition_wilayah_decodes_region() {
        let mut params: Vec<BoxedParam> = Vec::new();
        let cond = AsetScope::Wilayah("02.28".to_string())
            .push_condition(&mut params)
            .unwrap();
        assert!(cond.contains("substring(kdsatker_keu FROM 6 FOR 4)"));
        assert!(cond.contains("wilayah_kode"));
        assert!(cond.contains("$1"));
        assert_eq!(params.len(), 1);
    }
}
