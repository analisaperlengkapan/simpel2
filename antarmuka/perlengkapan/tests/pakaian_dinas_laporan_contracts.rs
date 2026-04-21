//! Source-contract snapshot: the Pakaian Dinas laporan UI must expose the
//! same filter surface as the legacy `LaporanController.php` + `cetakRekapTemplateV.blade.php`
//! pair in simpel_web-main (jenis_kelamin, eselon, jenis pegawai TU/Jaksa,
//! pengajuan, satker). Missing any of these would regress the PDF/Excel
//! rekap that ops teams cross-check against the legacy output.

fn source(path_from_crate_root: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/{}",
        env!("CARGO_MANIFEST_DIR"),
        path_from_crate_root
    ))
    .expect("source file should be readable")
}

#[test]
fn laporan_component_wires_all_legacy_filter_params() {
    let page = source("src/components/pakaian_dinas_laporan.rs");

    for param in [
        "jenis_kelamin",
        "eselon",
        "jenis_pegawai",
        "pengajuan",
        "satker",
    ] {
        assert!(
            page.contains(param),
            "laporan component should wire the `{param}` filter to match legacy rekap"
        );
    }
}

#[test]
fn laporan_component_supports_pdf_and_excel_export() {
    let page = source("src/components/pakaian_dinas_laporan.rs");

    assert!(
        page.contains("laporan/cetak"),
        "laporan component should call the /pakaian-dinas/laporan/cetak export endpoint"
    );
    assert!(
        page.contains("jenis_file=pdf") || page.contains("\"pdf\""),
        "laporan component should offer PDF export"
    );
    assert!(
        page.contains("jenis_file=xlsx")
            || page.contains("\"xlsx\"")
            || page.contains("excel"),
        "laporan component should offer Excel/XLSX export"
    );
}

#[test]
fn laporan_component_uses_shared_layout_primitives() {
    let page = source("src/components/pakaian_dinas_laporan.rs");

    for primitive in ["PageLayout", "SectionCard", "LoadingState", "EmptyState", "ErrorState"] {
        assert!(
            page.contains(primitive),
            "laporan component should use shared layout primitive {primitive}"
        );
    }
}

#[test]
fn laporan_component_supports_rekap_and_daftar_tabs() {
    let page = source("src/components/pakaian_dinas_laporan.rs");

    assert!(
        page.contains("rekap"),
        "laporan component should expose rekap tab (per-subspesifikasi × gender × ukuran table)"
    );
    assert!(
        page.contains("daftar"),
        "laporan component should expose daftar tab (pengajuan list)"
    );
    assert!(
        page.contains("LaporanRekapUkuran"),
        "laporan component should bind LaporanRekapUkuran fetch result"
    );
    assert!(
        page.contains("LaporanDaftarPegawai"),
        "laporan component should bind LaporanDaftarPegawai fetch result"
    );
}
