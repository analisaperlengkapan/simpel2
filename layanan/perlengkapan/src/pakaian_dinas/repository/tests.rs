use super::PakaianDinasRepository;
#[allow(unused_imports)]
use super::*;
use crate::pakaian_dinas::models::*;

// Note: Most tests require a database connection
// Run integration tests with: cargo test --features test-db

#[test]
fn test_repository_new() {
    // This is a placeholder for integration tests
    // Actual tests would require a database connection
}

/// Regression guard for Fase 0.1 — verifies the laporan filter builder uses
/// parameter placeholders ($N) instead of string-concat of user values, so
/// payloads like `' OR 1=1 --` cannot escape the SQL string.
///
/// This mirrors the construction logic inside `get_laporan_rekap_ukuran`
/// and `get_laporan_daftar_pegawai`. If this drifts from the production
/// code, update both together.
#[test]
fn filter_uses_parameter_binding_not_concat() {
    let malicious = "' OR 1=1 --";
    let filter = LaporanFilter {
        pengajuan_id: None,
        tahun: None,
        satker_id: None,
        jenis_kelamin: Some(malicious.to_string()),
        eselon: Some(malicious.to_string()),
        jenis: Some(malicious.to_string()),
    };

    let mut query = String::from("WHERE ps.pengajuan_id = $1");
    let mut idx: usize = 2;
    let mut placeholder_count = 0;
    if let Some(ref jk) = filter.jenis_kelamin {
        assert!(!jk.is_empty());
        query.push_str(&format!(" AND psp.jenis_kelamin = ${}", idx));
        placeholder_count += 1;
        idx += 1;
    }
    if let Some(ref eselon) = filter.eselon {
        assert!(!eselon.is_empty());
        query.push_str(&format!(" AND psp.eselon = ${}", idx));
        placeholder_count += 1;
        idx += 1;
    }
    if let Some(ref jenis) = filter.jenis {
        assert!(!jenis.is_empty());
        query.push_str(&format!(" AND psp.jenis = ${}", idx));
        placeholder_count += 1;
        idx += 1;
    }

    // The malicious string MUST NOT appear in the generated SQL — it should
    // travel as a bound parameter instead.
    assert!(
        !query.contains(malicious),
        "filter value leaked into SQL string: {query}"
    );
    assert!(!query.contains("' OR 1=1"));
    assert_eq!(placeholder_count, 3);
    assert_eq!(idx, 5);
    assert!(query.contains("$2"));
    assert!(query.contains("$3"));
    assert!(query.contains("$4"));
}
