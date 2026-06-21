//! Tiered RBAC data-visibility scope for perlengkapan **workflow** tables that
//! carry an authoritative MySIMKARI `satker_code` (e.g. `izin_pemakaian_bmn`,
//! `penghapusan_bmn` — see migration V003).
//!
//! This is the sibling of `bank_aset::AsetScope`. The difference is the join
//! key: `AsetScope` filters SIMAN assets (`kdsatker_keu`) via the cross-ref view
//! `integrasi.v_satker_code_map`, whereas workflow rows store the caller's
//! MySIMKARI `kode_satker` directly, so we filter on that column and resolve the
//! wilayah tier straight from `integrasi.mysimkari_satker.wilayah`.
//!
//! Tiers (same model as AsetScope):
//! - [`SatkerScope::All`]     — cross-satker roles ([`Claims::is_cross_satker_role`]).
//! - [`SatkerScope::Wilayah`] — `validator_wilayah`: rows whose satker is in the
//!   caller's wilayah.
//! - [`SatkerScope::Satker`]  — operator / `validator_satker`: only the caller's
//!   own satker.
//! - [`SatkerScope::Denied`]  — authenticated but no satker identity: FAIL CLOSED.
//!
//! Fail-closed note: rows with a NULL `satker_code` (legacy/seed inserted before
//! identity-derived create) match neither the satker nor wilayah tier, so they
//! are visible only to cross-satker roles. That is the safe default.

use crate::shared::middleware::Claims;

/// Boxed bind parameter compatible with the repositories' dynamic-SQL builders.
type BoxedParam = Box<dyn tokio_postgres::types::ToSql + Sync + Send>;

/// Visibility scope for a single request against a workflow table keyed by
/// MySIMKARI `satker_code`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SatkerScope {
    /// No row restriction (cross-satker roles).
    All,
    /// Restrict to the caller's wilayah. Holds the caller MySIMKARI `kode_satker`.
    Wilayah(String),
    /// Restrict to the caller's own satker. Holds the caller MySIMKARI `kode_satker`.
    Satker(String),
    /// Fail closed — no satker identity. Yields zero rows.
    Denied,
}

impl SatkerScope {
    /// Derive the visibility scope from the authenticated caller's claims.
    pub fn from_claims(claims: &Claims) -> Self {
        if claims.is_cross_satker_role() {
            return Self::All;
        }
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

    /// Append this scope as a SQL condition over the given MySIMKARI satker-code
    /// column (`col`, e.g. `"satker_code"`). Pushes the bind parameter (when any)
    /// onto `params` and returns the condition string, or `None` when no
    /// restriction applies ([`SatkerScope::All`]).
    ///
    /// `col` MUST be a trusted, fixed column name (never user input). Only
    /// caller-supplied *values* are bound as positional placeholders (`$n`).
    pub fn push_condition(&self, col: &str, params: &mut Vec<BoxedParam>) -> Option<String> {
        match self {
            Self::All => None,
            Self::Denied => Some("FALSE".to_string()),
            Self::Satker(code) => {
                params.push(Box::new(code.clone()));
                let i = params.len();
                Some(format!("{col} = ${i}"))
            }
            Self::Wilayah(code) => {
                params.push(Box::new(code.clone()));
                let i = params.len();
                // Rows whose satker shares the caller's wilayah. Wilayah is
                // resolved from integrasi.mysimkari_satker (cross-schema SoT).
                Some(format!(
                    "{col} IN (SELECT s.kode_satker FROM integrasi.mysimkari_satker s \
                     WHERE s.wilayah IS NOT NULL AND s.wilayah = (\
                       SELECT w.wilayah FROM integrasi.mysimkari_satker w \
                       WHERE w.kode_satker = ${i}))"
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
            assert_eq!(
                SatkerScope::from_claims(&claims(role, Some("02.28"))),
                SatkerScope::All
            );
        }
    }

    #[test]
    fn validator_wilayah_is_wilayah_scoped() {
        assert_eq!(
            SatkerScope::from_claims(&claims("validator_wilayah", Some("02.28"))),
            SatkerScope::Wilayah("02.28".to_string())
        );
    }

    #[test]
    fn operator_and_validator_satker_are_satker_scoped() {
        for role in ["operator_satker", "validator_satker", "operator"] {
            assert_eq!(
                SatkerScope::from_claims(&claims(role, Some("02.28"))),
                SatkerScope::Satker("02.28".to_string())
            );
        }
    }

    #[test]
    fn missing_or_blank_satker_fails_closed() {
        assert_eq!(
            SatkerScope::from_claims(&claims("operator_satker", None)),
            SatkerScope::Denied
        );
        assert_eq!(
            SatkerScope::from_claims(&claims("operator_satker", Some("  "))),
            SatkerScope::Denied
        );
    }

    #[test]
    fn case_insensitive_role() {
        assert_eq!(
            SatkerScope::from_claims(&claims("Validator_Wilayah", Some("02.28"))),
            SatkerScope::Wilayah("02.28".to_string())
        );
    }

    #[test]
    fn push_condition_all_is_none() {
        let mut p: Vec<BoxedParam> = Vec::new();
        assert!(SatkerScope::All.push_condition("satker_code", &mut p).is_none());
        assert_eq!(p.len(), 0);
    }

    #[test]
    fn push_condition_denied_is_false() {
        let mut p: Vec<BoxedParam> = Vec::new();
        assert_eq!(
            SatkerScope::Denied.push_condition("satker_code", &mut p).as_deref(),
            Some("FALSE")
        );
        assert_eq!(p.len(), 0);
    }

    #[test]
    fn push_condition_satker_binds_next_placeholder() {
        let mut p: Vec<BoxedParam> = vec![Box::new(1i64)];
        let cond = SatkerScope::Satker("02.28".to_string())
            .push_condition("satker_code", &mut p)
            .unwrap();
        assert_eq!(cond, "satker_code = $2");
        assert_eq!(p.len(), 2);
    }

    #[test]
    fn push_condition_wilayah_resolves_via_mysimkari() {
        let mut p: Vec<BoxedParam> = Vec::new();
        let cond = SatkerScope::Wilayah("02.28".to_string())
            .push_condition("satker_code", &mut p)
            .unwrap();
        assert!(cond.contains("integrasi.mysimkari_satker"));
        assert!(cond.contains("satker_code IN"));
        assert!(cond.contains("$1"));
        assert_eq!(p.len(), 1);
    }
}
