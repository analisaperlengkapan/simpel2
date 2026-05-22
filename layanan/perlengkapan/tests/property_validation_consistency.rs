//! Property 14 — validator-derive consistency.
//!
//! Asserts that the `#[derive(Validate)]` rules wired onto cross-module
//! request DTOs in `lib_perlengkapan::models::*` actually reject the
//! obvious bad cases and accept the obvious good cases. Catches
//! regressions where a rename or refactor silently drops a
//! `#[validate(...)]` attribute.
//!
//! We focus on `CreateKebutuhanBmnRequest` here — it has every category
//! of validator-derive rule the perlengkapan domain uses
//! (`length(min, max)` + `range(min)`), so exercising it covers the
//! invariants every list endpoint relies on.

use lib_perlengkapan::models::CreateKebutuhanBmnRequest;
use proptest::prelude::*;
use uuid::Uuid;
use validator::Validate;

fn make(
    kode_barang: String,
    nama_barang: String,
    jumlah_kebutuhan: i32,
) -> CreateKebutuhanBmnRequest {
    CreateKebutuhanBmnRequest {
        satker_id: Uuid::nil(),
        kode_barang,
        nama_barang,
        jumlah_kebutuhan,
        tahun_anggaran: 2026,
        justifikasi: None,
        estimasi_harga_satuan: None,
    }
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 64, .. ProptestConfig::default() })]

    /// Empty `kode_barang` must be rejected (`length(min=1)`).
    #[test]
    fn empty_kode_barang_rejected(
        nama in "[a-zA-Z ]{1,32}",
        jumlah in 1i32..1000,
    ) {
        let req = make(String::new(), nama, jumlah);
        prop_assert!(req.validate().is_err(),
            "empty kode_barang must fail validation");
    }

    /// Empty `nama_barang` must be rejected (`length(min=1)`).
    #[test]
    fn empty_nama_barang_rejected(
        kode in "[A-Z0-9-]{1,32}",
        jumlah in 1i32..1000,
    ) {
        let req = make(kode, String::new(), jumlah);
        prop_assert!(req.validate().is_err(),
            "empty nama_barang must fail validation");
    }

    /// `jumlah_kebutuhan < 1` must be rejected (`range(min=1)`).
    #[test]
    fn non_positive_jumlah_rejected(
        kode in "[A-Z0-9-]{1,32}",
        nama in "[a-zA-Z ]{1,32}",
        jumlah in i32::MIN..=0,
    ) {
        let req = make(kode, nama, jumlah);
        prop_assert!(req.validate().is_err(),
            "jumlah_kebutuhan < 1 must fail validation");
    }

    /// Sensible inputs must pass.
    #[test]
    fn sensible_request_validates(
        kode in "[A-Z0-9-]{1,30}",
        nama in "[a-zA-Z ]{1,200}",
        jumlah in 1i32..1_000_000,
    ) {
        let req = make(kode, nama, jumlah);
        prop_assert!(req.validate().is_ok(),
            "sensible kode/nama/jumlah should pass validation");
    }

    /// `kode_barang` longer than 50 must be rejected (`length(max=50)`).
    #[test]
    fn over_long_kode_barang_rejected(
        kode_padding in "[A-Z0-9]{51,80}",
        nama in "[a-zA-Z ]{1,32}",
        jumlah in 1i32..1000,
    ) {
        let req = make(kode_padding, nama, jumlah);
        prop_assert!(req.validate().is_err(),
            "kode_barang longer than 50 chars must fail validation");
    }
}
