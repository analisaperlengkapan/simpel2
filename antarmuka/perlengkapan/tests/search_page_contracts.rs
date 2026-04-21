//! Source-contract smoke tests for the search page. The page must surface
//! explicit loading / empty / error branches (never a silent fallthrough)
//! and type errors via `AppError` so the unified error-handling pipeline
//! applies here just like in the rest of the frontend.

fn source(path_from_crate_root: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/{}",
        env!("CARGO_MANIFEST_DIR"),
        path_from_crate_root
    ))
    .expect("source file should be readable")
}

#[test]
fn search_page_has_explicit_loading_and_empty_branches() {
    let page = source("src/pages/search_page.rs");

    assert!(
        page.contains("loading.get()"),
        "search page should gate UI on loading signal, not fall through silently"
    );
    assert!(
        page.contains("is_empty()"),
        "search page should branch on empty results, not render nothing"
    );
    assert!(
        page.contains("Tidak ditemukan"),
        "search page should show an explicit empty-state message in Bahasa Indonesia"
    );
}

#[test]
fn search_page_errors_are_app_error_typed() {
    let page = source("src/pages/search_page.rs");

    assert!(
        page.contains("crate::api::AppError"),
        "search page fetch should return AppError, not String or gloo_net::Error"
    );
}

#[test]
fn search_page_renders_result_badges_with_lib_ui_variants() {
    let page = source("src/pages/search_page.rs");

    assert!(
        page.contains("BadgeVariant::"),
        "search page should map result module/status to lib-ui BadgeVariant values"
    );
    assert!(
        page.contains("result.module.as_str()") && page.contains("result.status.as_str()"),
        "search page should surface result.module and result.status as filterable badges"
    );
}
