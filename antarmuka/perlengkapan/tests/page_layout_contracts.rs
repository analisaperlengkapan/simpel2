//! Source-contract smoke tests: newly rebuilt pages must use the shared
//! `PageLayout` primitive from `components/layout` so chrome, headers, and
//! spacing stay consistent.

fn source(path_from_crate_root: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/{}",
        env!("CARGO_MANIFEST_DIR"),
        path_from_crate_root
    ))
    .expect("source file should be readable")
}

fn assert_page_uses_page_layout(path: &str) {
    let page = source(path);
    assert!(
        page.contains("PageLayout"),
        "{path} should import/use the shared PageLayout primitive"
    );
    assert!(
        page.contains("<PageLayout"),
        "{path} should render <PageLayout …> as its outer wrapper"
    );
}

#[test]
fn bank_aset_pages_use_page_layout() {
    for path in [
        "src/pages/bank_aset/dashboard_page.rs",
        "src/pages/bank_aset/list_page.rs",
        "src/pages/bank_aset/detail_page.rs",
        "src/pages/bank_aset/sebaran_page.rs",
        "src/pages/bank_aset/qrcode_page.rs",
    ] {
        assert_page_uses_page_layout(path);
    }
}

#[test]
fn admin_pages_use_page_layout() {
    for path in [
        "src/pages/admin/audit_page.rs",
        "src/pages/admin/master_data_page.rs",
    ] {
        assert_page_uses_page_layout(path);
    }
}

#[test]
fn pakaian_dinas_spesifikasi_page_uses_page_layout() {
    assert_page_uses_page_layout("src/pages/pakaian_dinas/spesifikasi_page.rs");
}

#[test]
fn placeholder_page_is_not_referenced_by_new_pages() {
    for path in [
        "src/pages/pakaian_dinas/spesifikasi_page.rs",
        "src/pages/admin/audit_page.rs",
        "src/pages/admin/master_data_page.rs",
    ] {
        let page = source(path);
        assert!(
            !page.contains("PlaceholderPage"),
            "{path} should be a real page, not re-export PlaceholderPage"
        );
    }
}

#[test]
fn layout_primitives_module_is_exported() {
    let modules = source("src/components/mod.rs");
    assert!(
        modules.contains("pub mod layout"),
        "components mod should export the layout primitives module"
    );
}

#[test]
fn layout_module_exposes_required_primitives() {
    let layout_mod = source("src/components/layout/mod.rs");
    for symbol in [
        "page_layout",
        "section_card",
        "stat_card",
        "form_layout",
        "data_table",
        "loading_state",
        "empty_state",
        "error_state",
    ] {
        assert!(
            layout_mod.contains(symbol),
            "layout mod should expose the {symbol} primitive"
        );
    }
}
