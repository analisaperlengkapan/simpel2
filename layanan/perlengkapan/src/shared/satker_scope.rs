//! Tiered RBAC data-visibility scope for perlengkapan **workflow** tables that
//! carry an authoritative MySIMKARI `satker_code` (e.g. `izin_pemakaian_bmn`,
//! `penghapusan_bmn` — see migration V003).
//!
//! This is the sibling of `bank_aset::AsetScope`. The difference is only the
//! column each one filters: `AsetScope` filters SIMAN assets by `kdsatker_keu`
//! via the cross-ref view `integrasi.v_satker_code_map`, whereas workflow rows
//! store the caller's MySIMKARI `kode_satker` directly. Both resolve the wilayah
//! tier from the SAME definition, `integrasi.v_satker_wilayah` — see the note on
//! [`SatkerScope::push_condition`] for why that matters.
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
pub type BoxedParam = Box<dyn tokio_postgres::types::ToSql + Sync + Send>;

/// Borrow a boxed param list in the shape `query`/`execute` want.
pub fn as_refs(params: &[BoxedParam]) -> Vec<&(dyn tokio_postgres::types::ToSql + Sync)> {
    params
        .iter()
        .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
        .collect()
}

/// `scope` as an extra `AND` over `col`, plus its bind parameter.
///
/// Returns an empty string for the unrestricted tier so callers can splice it
/// in unconditionally. `col` MUST be a trusted, fixed column name.
///
/// Numbering is the part no type can check: the predicate takes whatever
/// placeholder comes after the binds already in `params`, so callers must push
/// their value binds FIRST. Bound too early it compares the satker column
/// against some unrelated value and quietly matches nothing — which reads as a
/// working deny to anyone who only tested the negative case. Each call site
/// pins its own expected `$n` in a test.
pub fn scope_and(scope: &SatkerScope, col: &str, params: &mut Vec<BoxedParam>) -> String {
    match scope.push_condition(col, params) {
        Some(cond) => format!(" AND {cond}"),
        None => String::new(),
    }
}

/// Visibility scope for a single request against a workflow table keyed by
/// MySIMKARI `satker_code`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SatkerScope {
    /// No row restriction (cross-satker roles).
    All,
    /// Restrict to the caller's wilayah. Holds the caller MySIMKARI `kode_satker`.
    Wilayah(String),
    /// Restrict to ONE wilayah by its Kejati `wilayah_code`. Produced only by
    /// [`SatkerScope::narrowed_to_wilayah`] when a cross-satker reader drills
    /// into a region; never derived from claims.
    WilayahKode(String),
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
        // Any held role counts (widest wins): a caller who is both operator and
        // validator_wilayah reads their wilayah, and must not be narrowed to
        // their own satker because the token happened to list operator first.
        if claims.holds_role("validator_wilayah") {
            Self::Wilayah(code.to_string())
        } else {
            // operator_satker, validator_satker, and any other satker-bound role.
            Self::Satker(code.to_string())
        }
    }

    /// Object-level counterpart of [`Self::push_condition`]: apakah `code` masuk
    /// dalam scope ini, untuk tier yang **tak butuh DB**?
    ///
    /// Mengembalikan `None` untuk [`Self::Wilayah`] — satu-satunya tier yang
    /// harus di-resolve lewat `integrasi.v_satker_wilayah`, jadi keputusannya
    /// milik repository (lihat `KebutuhanBmnRepository::satker_code_in_scope`).
    /// Memisahkannya begini membuat tier murni bisa diuji tanpa Postgres,
    /// sekaligus menjaga satu sumber kebenaran: repository mendelegasikan ke
    /// sini alih-alih menyalin ulang aturannya.
    pub fn contains_code_local(&self, code: &str) -> Option<bool> {
        match self {
            Self::All => Some(true),
            Self::Denied => Some(false),
            Self::Satker(own) => Some(own == code),
            Self::Wilayah(_) | Self::WilayahKode(_) => None,
        }
    }

    /// Narrow this scope to a SINGLE satker for a drill-down.
    ///
    /// `in_scope` is the database answer to "is `code` inside this scope",
    /// needed only for the wilayah tiers — pass the result of the repository's
    /// `satker_code_in_scope`; it is ignored for the tiers
    /// [`Self::contains_code_local`] can settle without a query.
    ///
    /// This is the whole safety argument for the dashboard drill-down, and the
    /// reason it is a scope TRANSFORM rather than an extra `AND` at each of the
    /// thirteen places a scope is applied: narrowing cannot widen. A caller who
    /// may not see `code` gets [`Self::Denied`], and every query downstream
    /// inherits it without needing to remember anything.
    pub fn narrowed_to(&self, code: &str, in_scope: bool) -> Self {
        if self.contains_code_local(code).unwrap_or(in_scope) {
            Self::Satker(code.to_string())
        } else {
            Self::Denied
        }
    }

    /// Narrow to one wilayah, identified by its Kejati `wilayah_code`.
    ///
    /// `caller_wilayah` is the caller's own region (`None` for cross-satker
    /// roles, who have no single one). Only a reader who already sees the whole
    /// country, or who is a validator of exactly that region, may select it —
    /// anyone else gets [`Self::Denied`].
    pub fn narrowed_to_wilayah(&self, wilayah_code: &str, caller_wilayah: Option<&str>) -> Self {
        match self {
            Self::All => Self::WilayahKode(wilayah_code.to_string()),
            Self::Wilayah(_) if caller_wilayah == Some(wilayah_code) => {
                Self::WilayahKode(wilayah_code.to_string())
            }
            // A satker-tier reader selecting their own region would be widening.
            _ => Self::Denied,
        }
    }

    /// The one query behind every repository's `satker_code_in_scope`: is a
    /// SINGLE `code` ($1) inside the wilayah of the caller ($2)?
    ///
    /// Sibling of the Wilayah arm of [`Self::push_condition`] — same definition
    /// of the tier, same view — but shaped as EXISTS because it tests one code
    /// instead of filtering a set. It lives here as a constant because two
    /// repositories had already grown their own verbatim copy of it, and a
    /// third was about to; a rule with N copies drifts at the first edit.
    pub const WILAYAH_MEMBERSHIP_SQL: &'static str = r#"
        SELECT EXISTS (
            SELECT 1
            FROM integrasi.v_satker_wilayah s
            WHERE s.kode_satker = $1
              AND s.wilayah_code = (
                  SELECT w.wilayah_code
                  FROM integrasi.v_satker_wilayah w
                  WHERE w.kode_satker = $2
              )
        ) AS in_scope
    "#;

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
            Self::WilayahKode(wilayah) => {
                params.push(Box::new(wilayah.clone()));
                let i = params.len();
                // The region named directly, rather than resolved from a member
                // satker — same view, same column, one less indirection.
                Some(format!(
                    "{col} IN (SELECT s.kode_satker FROM integrasi.v_satker_wilayah s \
                     WHERE s.wilayah_code = ${i})"
                ))
            }
            Self::Wilayah(code) => {
                params.push(Box::new(code.clone()));
                let i = params.len();
                // Rows whose satker shares the caller's wilayah, where wilayah
                // means the Kejaksaan Tinggi above them — resolved through
                // `integrasi.v_satker_wilayah`, the single definition every
                // scope reads (migration 005).
                //
                // This used to compare `integrasi.mysimkari_satker.wilayah`
                // instead. That column does not hold a Kejati: measured against
                // the real MySIMKARI snapshot it holds `I`/`II`/`III`, the
                // JAM-level supervision grouping, and `I` alone spans 15
                // Kejaksaan Tinggi = 238 satkers. A validator at Kejati Kepri
                // was reading Sumatera Utara and Kalimantan Selatan rows.
                //
                // A caller with no Kejati above them (Kejagung / pusat units)
                // has no row in the view, so the inner SELECT yields NULL and
                // the comparison matches nothing: fail closed, as intended.
                Some(format!(
                    "{col} IN (SELECT s.kode_satker FROM integrasi.v_satker_wilayah s \
                     WHERE s.wilayah_code = (\
                       SELECT w.wilayah_code FROM integrasi.v_satker_wilayah w \
                       WHERE w.kode_satker = ${i}))"
                ))
            }
        }
    }
}

/// Resolve the drill-down a request asked for, against the scope its claims
/// grant.
///
/// One place, because the alternative is every dashboard-ish endpoint growing
/// its own membership check — which is exactly how `satker_code_in_scope` came
/// to have three verbatim copies before it moved into this module.
///
/// Returns the pair the caller should actually query with. When the request
/// names no drill-down, the scopes come back untouched; when it names one the
/// caller may not see, both come back [`SatkerScope::Denied`] /
/// [`crate::bank_aset::AsetScope::Denied`] — an empty dashboard, never someone
/// else's.
pub async fn narrow_for_request(
    pool: &deadpool_postgres::Pool,
    scope: &SatkerScope,
    aset: &crate::bank_aset::AsetScope,
    satker: Option<&str>,
    wilayah: Option<&str>,
) -> Result<(SatkerScope, crate::bank_aset::AsetScope), crate::shared::error::AppError> {
    // A single satker wins over a region: it is the narrower of the two, and
    // accepting both would leave "which one applies?" to query-string order.
    if let Some(code) = satker.map(str::trim).filter(|s| !s.is_empty()) {
        // Only the wilayah tiers need the database to answer; the others are
        // settled by `contains_code_local` and the flag is ignored.
        let in_scope = match scope.contains_code_local(code) {
            Some(v) => v,
            None => {
                let SatkerScope::Wilayah(caller) = scope else {
                    // WilayahKode is never a claims-derived tier, so a
                    // drill-down from one is a second narrowing: allow it only
                    // within the region already selected.
                    return Ok((SatkerScope::Denied, crate::bank_aset::AsetScope::Denied));
                };
                let client = pool.get().await?;
                let row = client
                    .query_one(SatkerScope::WILAYAH_MEMBERSHIP_SQL, &[&code, &caller])
                    .await
                    .map_err(|e| {
                        crate::shared::error::AppError::Database(format!("satker membership: {e}"))
                    })?;
                row.get::<_, bool>("in_scope")
            }
        };
        return Ok((
            scope.narrowed_to(code, in_scope),
            aset.narrowed_to(code, in_scope),
        ));
    }

    if let Some(w) = wilayah.map(str::trim).filter(|s| !s.is_empty()) {
        // `w` is SIMAN's `wilayah_kode` (digits 6-9 of `kdsatker_keu`) — what
        // `/bank-aset/filter-options` serves. The two scopes speak DIFFERENT
        // code systems for the same region: `AsetScope` compares that SIMAN
        // code, while `SatkerScope` compares the Kejati's MySIMKARI
        // `wilayah_code`. Handing one string to both would silently scope the
        // perlengkapan half to nothing while the SIMAN half narrowed
        // correctly — a half-filtered dashboard, which is worse than an
        // unfiltered one because it looks like it worked.
        //
        // They ARE a measured bijection (see the note in bank_aset::scope), so
        // the translation is a lookup, not a guess.
        let client = pool.get().await?;
        let kejati: Option<String> = client
            .query_opt(
                "SELECT w.wilayah_code
                   FROM integrasi.v_satker_code_map m
                   JOIN integrasi.v_satker_wilayah  w ON w.kode_satker = m.kode_satker
                  WHERE m.wilayah_kode = $1
                  LIMIT 1",
                &[&w],
            )
            .await
            .map_err(|e| {
                crate::shared::error::AppError::Database(format!("wilayah translation: {e}"))
            })?
            .map(|r| r.get::<_, String>("wilayah_code"));

        // The caller's own region in BOTH spellings, so each half compares
        // like with like. `None` for cross-satker roles, who have no single
        // region and may therefore select any.
        let (caller_kejati, caller_siman) = match scope {
            SatkerScope::Wilayah(code) => {
                let row = client
                    .query_opt(
                        "SELECT w.wilayah_code, m.wilayah_kode
                           FROM integrasi.v_satker_wilayah w
                           LEFT JOIN integrasi.v_satker_code_map m ON m.kode_satker = w.kode_satker
                          WHERE w.kode_satker = $1",
                        &[code],
                    )
                    .await
                    .map_err(|e| {
                        crate::shared::error::AppError::Database(format!("caller wilayah: {e}"))
                    })?;
                match row {
                    Some(r) => (
                        Some(r.get::<_, String>("wilayah_code")),
                        r.get::<_, Option<String>>("wilayah_kode"),
                    ),
                    None => (None, None),
                }
            }
            _ => (None, None),
        };

        // An unmappable region fails closed rather than falling through to an
        // unnarrowed scope.
        let satker_scope = match kejati.as_deref() {
            Some(k) => scope.narrowed_to_wilayah(k, caller_kejati.as_deref()),
            None => SatkerScope::Denied,
        };
        return Ok((
            satker_scope,
            aset.narrowed_to_wilayah(w, caller_siman.as_deref()),
        ));
    }

    Ok((scope.clone(), aset.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn claims(role: &str, satker: Option<&str>) -> Claims {
        let mut c = Claims::with_roles(Uuid::nil(), "u", [role]);
        c.satker_code = satker.map(|s| s.to_string());
        c
    }

    #[test]
    fn scope_uses_every_held_role_not_the_first() {
        let mut c = Claims::with_roles(Uuid::nil(), "u", ["operator_satker", "validator_wilayah"]);
        c.satker_code = Some("02.28".to_string());
        assert_eq!(
            SatkerScope::from_claims(&c),
            SatkerScope::Wilayah("02.28".to_string()),
            "widest held role wins, regardless of token order"
        );
        let mut both =
            Claims::with_roles(Uuid::nil(), "u", ["validator_wilayah", "validator_pusat"]);
        both.satker_code = Some("02.28".to_string());
        assert_eq!(SatkerScope::from_claims(&both), SatkerScope::All);
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

    // ---- contains_code_local: object-level tiers (#93) ------------------

    #[test]
    fn contains_code_local_all_admits_any_code() {
        assert_eq!(SatkerScope::All.contains_code_local("02.28"), Some(true));
        assert_eq!(SatkerScope::All.contains_code_local(""), Some(true));
    }

    #[test]
    fn contains_code_local_denied_admits_nothing() {
        assert_eq!(
            SatkerScope::Denied.contains_code_local("02.28"),
            Some(false)
        );
    }

    #[test]
    fn contains_code_local_satker_is_exact_match() {
        let scope = SatkerScope::Satker("02.28".to_string());
        assert_eq!(scope.contains_code_local("02.28"), Some(true));
        // The cross-satker read that #93 was filed for.
        assert_eq!(scope.contains_code_local("02.29"), Some(false));
        // No prefix/substring leniency: a satker code is compared whole.
        assert_eq!(scope.contains_code_local("02.280"), Some(false));
        assert_eq!(scope.contains_code_local("02.2"), Some(false));
    }

    #[test]
    fn contains_code_local_defers_wilayah_to_the_database() {
        assert_eq!(
            SatkerScope::Wilayah("02.28".to_string()).contains_code_local("02.29"),
            None
        );
    }

    #[test]
    fn push_condition_all_is_none() {
        let mut p: Vec<BoxedParam> = Vec::new();
        assert!(
            SatkerScope::All
                .push_condition("satker_code", &mut p)
                .is_none()
        );
        assert_eq!(p.len(), 0);
    }

    #[test]
    fn push_condition_denied_is_false() {
        let mut p: Vec<BoxedParam> = Vec::new();
        assert_eq!(
            SatkerScope::Denied
                .push_condition("satker_code", &mut p)
                .as_deref(),
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
    fn push_condition_wilayah_resolves_via_the_shared_kejati_view() {
        let mut p: Vec<BoxedParam> = Vec::new();
        let cond = SatkerScope::Wilayah("02.28".to_string())
            .push_condition("satker_code", &mut p)
            .unwrap();
        assert!(cond.contains("integrasi.v_satker_wilayah"));
        assert!(cond.contains("wilayah_code"));
        // The column that used to back this tier must not come back.
        assert!(!cond.contains("mysimkari_satker"));
        assert!(cond.contains("satker_code IN"));
        assert!(cond.contains("$1"));
        assert_eq!(p.len(), 1);
    }
}
