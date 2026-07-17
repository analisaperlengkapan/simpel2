//! Source-contract smoke tests: list pages across modules must reuse the
//! shared loading / empty / error primitives under `components::layout`,
//! instead of re-rolling the same spinner markup that used to diverge
//! across lists.
//!
//! (The legacy `list_feedback` duplicates and the superseded
//! `components/{pemakaian_bmn_list,penghapusan_list}.rs` implementations
//! were deleted in the FE audit — the live lists are
//! `pages/{pemakaian_bmn,penghapusan_bmn}/list_page.rs`.)

fn source(path_from_crate_root: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/{}",
        env!("CARGO_MANIFEST_DIR"),
        path_from_crate_root
    ))
    .expect("source file should be readable")
}

fn uses_shared_loading_primitive(content: &str) -> bool {
    content.contains("components::layout")
        && content.contains("LoadingState")
        && content.contains("EmptyState")
}

fn has_no_inline_spinner(content: &str, path: &str) {
    assert!(
        !content.contains("animate-spin rounded-full h-8 w-8"),
        "{path} should not reintroduce the legacy inline spinner markup; use shared loading primitive"
    );
}

#[test]
fn layout_primitives_are_exported() {
    let modules = source("src/components/mod.rs");
    assert!(
        modules.contains("pub mod layout"),
        "components mod should export layout primitives"
    );
}

#[test]
fn kebutuhan_bmn_list_uses_shared_loading_and_empty_state() {
    let path = "src/components/kebutuhan_bmn_list.rs";
    let page = source(path);

    assert!(
        uses_shared_loading_primitive(&page),
        "{path} should use the shared layout LoadingState/EmptyState"
    );
    has_no_inline_spinner(&page, path);
}

#[test]
fn pemakaian_bmn_list_page_uses_shared_feedback() {
    let path = "src/pages/pemakaian_bmn/list_page.rs";
    let page = source(path);

    assert!(
        uses_shared_loading_primitive(&page),
        "{path} should use the shared layout LoadingState/EmptyState"
    );
    has_no_inline_spinner(&page, path);
}

#[test]
fn penghapusan_list_page_uses_shared_feedback() {
    let path = "src/pages/penghapusan_bmn/list_page.rs";
    let page = source(path);

    assert!(
        uses_shared_loading_primitive(&page),
        "{path} should use the shared layout LoadingState/EmptyState"
    );
    has_no_inline_spinner(&page, path);
}
