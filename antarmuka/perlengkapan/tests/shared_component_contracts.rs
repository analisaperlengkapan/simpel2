//! Source-contract smoke tests: list components across modules must reuse
//! shared loading / empty / error primitives and the shared pagination
//! controls, instead of re-rolling the same spinner markup and pager math.
//!
//! The acceptable sources are (a) the domain-agnostic layout primitives
//! under `components::layout`, or (b) the legacy `list_feedback` /
//! `pagination_controls` modules. Either avoids the inline spinner markup
//! that used to diverge across the three lists.

fn source(path_from_crate_root: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/{}",
        env!("CARGO_MANIFEST_DIR"),
        path_from_crate_root
    ))
    .expect("source file should be readable")
}

fn uses_shared_loading_primitive(content: &str) -> bool {
    content.contains("list_feedback::{EmptyState, LoadingState}")
        || content.contains("list_feedback::{LoadingState, EmptyState}")
        || (content.contains("components::layout")
            && content.contains("LoadingState")
            && content.contains("EmptyState"))
}

fn uses_shared_pagination(content: &str) -> bool {
    content.contains("pagination_controls::PaginationControls")
        || content.contains("PaginationControls")
}

fn has_no_inline_spinner(content: &str, path: &str) {
    assert!(
        !content.contains("animate-spin rounded-full h-8 w-8"),
        "{path} should not reintroduce the legacy inline spinner markup; use shared loading primitive"
    );
}

#[test]
fn shared_feedback_and_pagination_modules_are_exported() {
    let modules = source("src/components/mod.rs");
    assert!(
        modules.contains("pub mod list_feedback;"),
        "components mod should still export list_feedback for legacy callers"
    );
    assert!(
        modules.contains("pub mod pagination_controls;"),
        "components mod should still export pagination_controls for list callers"
    );
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
        "{path} should use a shared LoadingState/EmptyState (layout or list_feedback)"
    );
    has_no_inline_spinner(&page, path);
}

#[test]
fn pemakaian_bmn_list_uses_shared_feedback_and_pagination() {
    let path = "src/components/pemakaian_bmn_list.rs";
    let page = source(path);

    assert!(
        uses_shared_loading_primitive(&page),
        "{path} should use a shared LoadingState/EmptyState primitive"
    );
    assert!(
        uses_shared_pagination(&page),
        "{path} should use shared PaginationControls"
    );
    has_no_inline_spinner(&page, path);
}

#[test]
fn penghapusan_list_uses_shared_feedback_and_pagination() {
    let path = "src/components/penghapusan_list.rs";
    let page = source(path);

    assert!(
        uses_shared_loading_primitive(&page),
        "{path} should use a shared LoadingState/EmptyState primitive"
    );
    assert!(
        uses_shared_pagination(&page),
        "{path} should use shared PaginationControls"
    );
    has_no_inline_spinner(&page, path);
}
