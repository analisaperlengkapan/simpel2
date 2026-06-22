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
