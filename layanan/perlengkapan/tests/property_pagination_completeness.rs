//! Property 12 — pagination completeness.
//!
//! Properties verified for `PageParams`:
//!   1. After `resolved()`, both `page` and `per_page` are within their
//!      documented bounds: `page >= 1`, `1 <= per_page <= MAX_PER_PAGE`.
//!   2. `offset() == (page - 1) * per_page` for the resolved values.
//!   3. `limit() == per_page` for the resolved values.
//!   4. Defaulting works: `PageParams::default().resolved() == (1, 20)`.
//!
//! These are the invariants every list endpoint relies on. Without them
//! a sufficiently-bad caller could request `page=0` and get back negative
//! offsets, or `per_page=10_000` and exhaust the DB.

use lib_perlengkapan::pagination::{DEFAULT_PER_PAGE, MAX_PER_PAGE, PageParams};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig { cases: 128, .. ProptestConfig::default() })]

    #[test]
    fn resolved_clamps_page_to_min_one(page in 0u32..1000, per_page in 1u32..200) {
        let params = PageParams { page: Some(page), per_page: Some(per_page) };
        let (rp, _) = params.resolved();
        prop_assert!(rp >= 1, "resolved page must be >= 1, got {rp}");
    }

    #[test]
    fn resolved_clamps_per_page(per_page in 0u32..10_000) {
        let params = PageParams { page: None, per_page: Some(per_page) };
        let (_, rpp) = params.resolved();
        prop_assert!((1..=MAX_PER_PAGE).contains(&rpp),
            "resolved per_page out of bounds: {rpp} (max {MAX_PER_PAGE})");
    }

    #[test]
    fn offset_matches_resolved(page in 1u32..1000, per_page in 1u32..200) {
        let params = PageParams { page: Some(page), per_page: Some(per_page) };
        let (rp, rpp) = params.resolved();
        let expected = ((rp - 1) as u64) * (rpp as u64);
        prop_assert_eq!(params.offset(), expected);
    }

    #[test]
    fn limit_equals_resolved_per_page(page in 1u32..1000, per_page in 1u32..200) {
        let params = PageParams { page: Some(page), per_page: Some(per_page) };
        let (_, rpp) = params.resolved();
        prop_assert_eq!(params.limit(), rpp as u64);
    }
}

#[test]
fn defaults_match_documented_constants() {
    let p = PageParams::default();
    let (page, per_page) = p.resolved();
    assert_eq!(page, 1);
    assert_eq!(per_page, DEFAULT_PER_PAGE);
    assert_eq!(p.offset(), 0);
    assert_eq!(p.limit(), DEFAULT_PER_PAGE as u64);
}
