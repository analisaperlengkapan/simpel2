//! Source-contract smoke tests for workflow pages across `pemakaian_bmn`
//! and `penghapusan_bmn`. The refactor established two invariants that any
//! future edit must preserve:
//!
//! 1. Every fetch-backed list renders an explicit loading branch gated by
//!    the `loading` signal (no silent fallthrough to a blank panel).
//! 2. Empty state is guarded by `!loading.get()` so we never flash "Tidak
//!    ada data" over a request that is still in flight.
//!
//! Badge colour helpers and workflow-specific layout primitives were
//! *proposed* during planning but never extracted; asserting on names that
//! don't exist would leave the tests red without telling us anything useful
//! about the code. Instead these tests anchor on the patterns that actually
//! shipped.

fn source(path_from_crate_root: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/{}",
        env!("CARGO_MANIFEST_DIR"),
        path_from_crate_root
    ))
    .expect("source file should be readable")
}

fn assert_workflow_list_loading_and_empty_contract(path: &str) {
    let page = source(path);

    assert!(
        page.contains("loading.get().then(|| view!"),
        "{path} should gate its loading panel with loading.get().then(|| view!) — otherwise the spinner leaks through after data arrives"
    );
    assert!(
        page.contains("is_empty() && !loading.get()"),
        "{path} should guard the empty-state branch with !loading.get() so we don't flash 'no data' during fetch"
    );
}

fn assert_creation_page_loading_contract(path: &str) {
    let page = source(path);

    assert!(
        page.contains("loading.get().then(|| view!"),
        "{path} should gate its loading panel with loading.get().then(|| view!)"
    );
}

#[test]
fn penghapusan_review_page_keeps_list_loading_and_empty_contract() {
    assert_workflow_list_loading_and_empty_contract("src/pages/penghapusan_bmn/review_page.rs");
}

#[test]
fn penghapusan_sk_generation_page_keeps_list_loading_and_empty_contract() {
    assert_workflow_list_loading_and_empty_contract(
        "src/pages/penghapusan_bmn/sk_generation_page.rs",
    );
}

#[test]
fn penghapusan_sk_view_page_keeps_list_loading_and_empty_contract() {
    assert_workflow_list_loading_and_empty_contract("src/pages/penghapusan_bmn/sk_view_page.rs");
}

#[test]
fn pemakaian_document_management_page_keeps_list_loading_and_empty_contract() {
    assert_workflow_list_loading_and_empty_contract(
        "src/pages/pemakaian_bmn/document_management_page.rs",
    );
}

#[test]
fn penghapusan_request_creation_page_keeps_loading_contract() {
    assert_creation_page_loading_contract("src/pages/penghapusan_bmn/request_creation_page.rs");
}

#[test]
fn pemakaian_permit_creation_page_keeps_loading_contract() {
    assert_creation_page_loading_contract("src/pages/pemakaian_bmn/permit_creation_page.rs");
}

#[test]
fn penghapusan_review_page_uses_app_error() {
    let page = source("src/pages/penghapusan_bmn/review_page.rs");

    assert!(
        page.contains("AppError"),
        "penghapusan review page should map fetch failures through AppError"
    );
}

#[test]
fn penghapusan_sk_generation_page_uploads_signed_sk_via_backend_endpoint() {
    let page = source("src/pages/penghapusan_bmn/sk_generation_page.rs");

    assert!(
        page.contains("upload-signed-sk") || page.contains("upload_signed_sk"),
        "sk_generation page should wire the signed-SK upload to the backend /upload-signed-sk endpoint"
    );
    assert!(
        page.contains("uploading"),
        "sk_generation page should track upload-in-progress state separately from list loading"
    );
}
