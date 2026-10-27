//! Which of the four permit kinds a SIMAN asset belongs to.
//!
//! An izin pemakaian classifies every asset as `KENDARAAN_BERMOTOR`,
//! `RUMAH_NEGARA`, `LAPTOP` or `LAINNYA` (the CHECK on
//! `pemakaian_bmn_items.jenis_bmn`). SIMAN already says what an asset is —
//! `jenis_aset`, surfaced to the frontend as `BankAsetItem::kategori_aset` — so
//! the kind is DERIVED from the asset the operator picks rather than chosen in a
//! dropdown beside it. A dropdown is a second answer to a question the reference
//! data already answers, and the two can disagree.
//!
//! This table is the one place the mapping lives. The frontend calls
//! [`jenis_bmn_dari_jenis_aset`]; the backend builds its SQL `CASE` from
//! [`JENIS_BMN_DARI_JENIS_ASET`] (`shared::siman_columns::jenis_bmn_sql`). Before
//! this module the mapping existed only as that SQL string, so a frontend copy
//! would have been a second master — the shape that renamed workflow steps in
//! kebutuhan-BMN when its status vocabulary was copied five ways.

/// `(SIMAN jenis_aset, permit jenis_bmn)`. Anything not listed is `LAINNYA`.
///
/// `Peralatan Mesin Khusus TIK` maps to `LAPTOP` because LAPTOP is the permit
/// domain's only IT bucket; it is broader than its label, and that is kept
/// visible rather than folding the second-largest category into LAINNYA.
pub const JENIS_BMN_DARI_JENIS_ASET: &[(&str, &str)] = &[
    ("Alat Angkutan Bermotor", "KENDARAAN_BERMOTOR"),
    ("Rumah Negara", "RUMAH_NEGARA"),
    ("Peralatan Mesin Khusus TIK", "LAPTOP"),
];

/// The permit kind for assets of no listed `jenis_aset`.
pub const JENIS_BMN_LAINNYA: &str = "LAINNYA";

/// Permit kind of a SIMAN asset, from its `jenis_aset`.
pub fn jenis_bmn_dari_jenis_aset(jenis_aset: &str) -> &'static str {
    JENIS_BMN_DARI_JENIS_ASET
        .iter()
        .find(|(siman, _)| *siman == jenis_aset.trim())
        .map_or(JENIS_BMN_LAINNYA, |(_, jenis)| jenis)
}

/// Human label for a permit kind.
pub fn label_jenis_bmn(jenis_bmn: &str) -> &'static str {
    match jenis_bmn {
        "KENDARAAN_BERMOTOR" => "Kendaraan Bermotor",
        "RUMAH_NEGARA" => "Rumah Negara",
        "LAPTOP" => "Laptop / Peralatan TIK",
        _ => "Lainnya",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_listed_jenis_aset_maps_to_its_kind() {
        assert_eq!(jenis_bmn_dari_jenis_aset("Alat Angkutan Bermotor"), "KENDARAAN_BERMOTOR");
        assert_eq!(jenis_bmn_dari_jenis_aset("Rumah Negara"), "RUMAH_NEGARA");
        assert_eq!(jenis_bmn_dari_jenis_aset("Peralatan Mesin Khusus TIK"), "LAPTOP");
    }

    #[test]
    fn anything_else_is_lainnya() {
        for other in ["Tanah", "Gedung dan Bangunan", "TIDAK DIKETAHUI", ""] {
            assert_eq!(jenis_bmn_dari_jenis_aset(other), JENIS_BMN_LAINNYA, "{other}");
        }
    }
}
