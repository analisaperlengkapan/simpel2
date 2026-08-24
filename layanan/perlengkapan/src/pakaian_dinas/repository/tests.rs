use crate::pakaian_dinas::models::*;
use crate::pakaian_dinas::repository::laporan::{JenisPakaianPredicate, push_laporan_filters};
use tokio_postgres::types::ToSql;
use uuid::Uuid;

// Note: Most tests require a database connection
// Run integration tests with: cargo test --features test-db

#[test]
fn test_repository_new() {
    // This is a placeholder for integration tests
    // Actual tests would require a database connection
}

fn build(filter: &LaporanFilter, mode: JenisPakaianPredicate) -> (String, usize, Option<usize>) {
    let mut sql = String::from("WHERE ps.pengajuan_id = $1");
    let mut params: Vec<Box<dyn ToSql + Sync + Send>> = vec![Box::new(Uuid::nil())];
    let built = push_laporan_filters(&mut sql, &mut params, filter, 2, mode);
    // One bound value per placeholder, always — a mismatch here is the bug that
    // makes tokio-postgres fail at bind time rather than at compile time.
    assert_eq!(params.len(), built.next_idx - 1);
    (sql, built.next_idx, built.jenis_pakaian_idx)
}

/// Regression guard for Fase 0.1 — the laporan filter builder must use
/// parameter placeholders ($N) rather than concatenating user values, so
/// payloads like `' OR 1=1 --` cannot escape the SQL string.
///
/// This calls the production builder directly. An earlier version of this test
/// re-implemented the same `if let` chain and carried a note asking future
/// readers to keep the copy in sync; it then certified a builder that silently
/// dropped `satker_id`, because the copy dropped it too.
#[test]
fn filter_uses_parameter_binding_not_concat() {
    let malicious = "' OR 1=1 --";
    let filter = LaporanFilter {
        satker_id: Some(malicious.to_string()),
        jenis_kelamin: Some(malicious.to_string()),
        eselon: Some(malicious.to_string()),
        jenis: Some(malicious.to_string()),
        ..Default::default()
    };

    let (sql, next_idx, _) = build(&filter, JenisPakaianPredicate::Column);

    assert!(
        !sql.contains(malicious),
        "filter value leaked into SQL string: {sql}"
    );
    assert!(!sql.contains("' OR 1=1"));
    assert_eq!(next_idx, 6, "four filters must bind $2..$5");
    for p in ["$2", "$3", "$4", "$5"] {
        assert!(sql.contains(p), "missing placeholder {p} in: {sql}");
    }
}

/// Every filter the API accepts must reach the SQL. `satker_id` is named
/// explicitly because it was the one that did not: the rekap screen query
/// omitted it while the export built from the same `LaporanFilter` applied it,
/// so one selection produced a national table and a single-satker spreadsheet.
#[test]
fn every_filter_field_reaches_the_sql() {
    let filter = LaporanFilter {
        satker_id: Some("0200010".to_string()),
        jenis_pakaian_id: Some(Uuid::from_u128(7)),
        jenis_kelamin: Some("L".to_string()),
        eselon: Some("IV".to_string()),
        jenis: Some("0".to_string()),
        ..Default::default()
    };

    let (sql, _, _) = build(&filter, JenisPakaianPredicate::Column);

    for column in [
        "psp.jenis_kelamin",
        "ps.satker_id",
        "psp.eselon",
        "psp.jenis",
        "pp.jenis_pakaian_id",
    ] {
        assert!(
            sql.contains(column),
            "filter dropped: {column} not in {sql}"
        );
    }
}

/// An empty filter must add nothing at all — the report is national by default,
/// and an accidental `AND` would make every unfiltered report return zero rows.
#[test]
fn empty_filter_adds_no_predicates() {
    let (sql, next_idx, jenis_idx) =
        build(&LaporanFilter::default(), JenisPakaianPredicate::Column);

    assert_eq!(sql, "WHERE ps.pengajuan_id = $1");
    assert_eq!(next_idx, 2);
    assert!(jenis_idx.is_none());
}

/// The daftar queries share one WHERE clause between a COUNT that joins only
/// `psp`/`ps` and a data query that LEFT JOINs the item tables. A `pp.`
/// predicate would be an unknown identifier in the first and would turn the
/// LEFT JOIN inner in the second, so that mode must emit an EXISTS instead —
/// and must not mention `pp.` at all.
#[test]
fn exists_mode_does_not_reference_the_joined_item_alias() {
    let filter = LaporanFilter {
        jenis_pakaian_id: Some(Uuid::from_u128(7)),
        ..Default::default()
    };

    let (sql, _, jenis_idx) = build(&filter, JenisPakaianPredicate::ExistsOnPegawai);

    assert!(sql.contains("EXISTS"), "expected an EXISTS in: {sql}");
    assert!(
        !sql.contains("pp."),
        "EXISTS mode must not reference the outer `pp` alias: {sql}"
    );
    assert!(sql.contains("pux.pegawai_id = psp.id"));
    assert!(sql.contains("ppx.jenis_pakaian_id"));
    assert_eq!(
        jenis_idx,
        Some(2),
        "the daftar data query reuses this placeholder in its LEFT JOIN"
    );
}

/// The placeholder reported back must be the one actually written into the SQL;
/// the daftar data query binds nothing extra for its `LEFT JOIN … pp` ON clause
/// and relies on referencing this exact index a second time.
#[test]
fn reported_jenis_pakaian_placeholder_matches_the_sql() {
    let filter = LaporanFilter {
        jenis_kelamin: Some("P".to_string()),
        satker_id: Some("0200020".to_string()),
        jenis_pakaian_id: Some(Uuid::from_u128(7)),
        ..Default::default()
    };

    let (sql, next_idx, jenis_idx) = build(&filter, JenisPakaianPredicate::ExistsOnPegawai);

    let idx = jenis_idx.expect("filter set jenis_pakaian_id");
    assert_eq!(idx, 4, "two filters precede it");
    assert_eq!(next_idx, 5);
    assert!(
        sql.contains(&format!("ppx.jenis_pakaian_id = ${idx}")),
        "reported placeholder ${idx} is not the one in: {sql}"
    );
}
