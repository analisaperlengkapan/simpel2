//! How an employee row is tied to a satker — the one place that knows.
//!
//! `integrasi.mysimkari_pegawai.satker_id` is TEXT, and every satker-scoped
//! query in this crate read it as if it held the MySIMKARI `kode_satker`. It
//! does not. It holds `mysimkari_satker.api_id` — the UUID the upstream API
//! assigns — and the two spaces do not overlap.
//!
//! Measured against staging on 2026-08-29, over 21 328 employee rows:
//!
//! | `satker_id` matches | rows |
//! |---|---|
//! | `mysimkari_satker.api_id` | **21 325** |
//! | `mysimkari_satker.kode_satker` | **3** (coincidences) |
//!
//! So `WHERE satker_id = '01.01'` returned **zero rows for every satker**, and
//! the consequences were not cosmetic:
//!
//! * the pakaian-dinas employee roster was empty for all 191 satkers that have
//!   people in them;
//! * the profile upsert splices the caller's scope as an extra `AND` on the
//!   SoT row, so for an `operator_satker` it selected nothing, inserted
//!   nothing, and silently affected 0 rows — the flow worked only for the
//!   cross-satker roles, whose scope adds no predicate at all;
//! * `pegawai_pakaian_dinas.kode_satker` was being written from
//!   `p.satker_id`, i.e. a UUID into a column named for a code.
//!
//! None of it raised an error. An empty roster and a successful-looking upsert
//! are what "no rows matched" looks like from the outside, which is why the
//! doc comment on the old query could assert the wrong column for as long as
//! it did. The lesson is the recorded one: a claim about a column's contents
//! is a measurement, not a comment.
//!
//! Everything that needs an employee's satker joins through here, so the next
//! caller cannot reintroduce the mismatch by writing the obvious thing.

/// Joins the satker an employee belongs to. Expects the employee table to be
/// aliased `p`; exposes the satker as `ms_peg`.
///
/// `LEFT` on purpose: 3 of 21 328 rows carry a `satker_id` that resolves to no
/// satker at all. They must still be visible to a cross-satker reader (whose
/// scope adds no predicate) rather than vanishing from every listing — an
/// employee with bad reference data is a data problem to see, not to hide.
pub const PEGAWAI_SATKER_JOIN_SQL: &str =
    "LEFT JOIN integrasi.mysimkari_satker ms_peg ON ms_peg.api_id = p.satker_id";

/// The employee's MySIMKARI `kode_satker`, resolved through the join above.
///
/// This — never `p.satker_id` — is what a `SatkerScope` predicate and any
/// stored `kode_satker` must be built on.
pub const PEGAWAI_KODE_SATKER_SQL: &str = "ms_peg.kode_satker";

#[cfg(test)]
mod tests {
    use super::*;

    /// The join and the column have to agree on the alias, and the alias has
    /// to be one a caller cannot collide with by accident.
    #[test]
    fn the_column_is_reachable_from_the_join() {
        assert!(PEGAWAI_SATKER_JOIN_SQL.contains("ms_peg"));
        assert!(PEGAWAI_KODE_SATKER_SQL.starts_with("ms_peg."));
        // The mismatch this module exists to prevent: scoping on the employee
        // table's own column instead of the resolved code.
        assert!(!PEGAWAI_KODE_SATKER_SQL.contains("p.satker_id"));
    }

    /// `api_id` is the join key, not `id`. `mysimkari_satker.id` is a
    /// BIGSERIAL surrogate; joining on it matches nothing.
    #[test]
    fn the_join_key_is_api_id() {
        assert!(PEGAWAI_SATKER_JOIN_SQL.contains("ms_peg.api_id = p.satker_id"));
    }
}
