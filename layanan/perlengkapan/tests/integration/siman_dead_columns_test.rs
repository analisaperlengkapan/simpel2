//! Guard against the "dead SIMAN column" defect class (#829, #832, and the
//! BMN-utilisation report this test shipped with).
//!
//! `integrasi.siman_aset` carries two parallel sets of columns for the same
//! facts and only one set is ever written. The ingest builds its `INSERT`
//! column list **from the keys of the SIMAN API payload**
//! (`layanan/integrasi/src/db.rs` — `first_obj.keys()`), so every schema column
//! is silently optional: no `NOT NULL`, no error, no log. A query against the
//! wrong half compiles, runs, and returns a plausible-looking empty answer.
//!
//! It has now happened five times, in four modules, and every existing signal
//! missed all five:
//!
//! - `cargo check` cannot see inside a SQL string literal.
//! - `psql` on staging returns rows, because the *seed's own* rows populate the
//!   dead columns — the `5 / 624533` signature. A `LIMIT 10` spot-check can
//!   land on exactly those five and "confirm" the column works.
//! - The integration fixture used to fill the dead columns too, so the tests
//!   did not merely miss the bug, they CERTIFIED it.
//! - Nothing errors. `WHERE kondisi = 'BAIK'` matching 0 of 624 533 rows looks
//!   identical to "no assets are in good condition".
//!
//! So this guard does not check a hand-written list of columns — the previous
//! four fixes each carried a comment naming the earlier instances, and the
//! comments did not stop the next one. It DERIVES the dead set by asking the
//! fixture database which columns of `integrasi.siman_aset` came back empty,
//! then fails on any `src/` SQL that reads one of them.
//!
//! Deriving it is only meaningful because `common::setup_test_db` seeds the
//! table in production shape: the columns SIMAN fills are filled, the ones it
//! never fills are left NULL. The floor assertion below keeps that honest.
//!
//! Scope: this crate's `src/` only. The same defect exists across service
//! boundaries — `layanan/integrasi/src/grpc/service.rs` selected a bare `nup`
//! among COALESCE'd siblings — but that crate has its own tests and this
//! harness has no way to compile it.

use crate::common::{setup_test_db, teardown_test_db};
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Columns that MUST come back empty from the fixture.
///
/// This is a floor, not the scope: the derived set is what the test actually
/// uses, and it grows by itself when SIMAN stops filling something. The floor
/// exists for the opposite direction — someone "tidying" the fixture by filling
/// `nup` again would quietly remove `nup` from the derived set and switch this
/// guard off for the exact column it was written for. Measured over 624 533
/// staging rows: `kondisi`, `nama_barang`, `kode_barang`, `satker_id` and
/// `raw_data` in 0 rows; `kategori_aset` and `nup` in 5 — the e2e seed's own.
const KNOWN_DEAD: &[&str] = &[
    "kategori_aset",
    "kode_barang",
    "kondisi",
    "nama_barang",
    "nup",
    "raw_data",
    "satker_id",
];

/// Aliases bound to `integrasi.siman_aset` in this literal.
///
/// Needed because a dead column NAME can legitimately belong to another table
/// in the same statement: the gap-analysis query builds a CTE whose own output
/// column is `nama_barang`, and `r.nama_barang` there is nothing to do with
/// SIMAN. Only `<siman-alias>.<col>` and unqualified `<col>` are this table's.
fn siman_aliases(literal: &str) -> BTreeSet<String> {
    // Words that can follow the table name without being an alias.
    const NOT_ALIASES: &[&str] = &[
        "where", "group", "order", "limit", "on", "left", "right", "inner", "join", "cross",
        "union", "having", "as", "using", "and", "or", "sa",
    ];
    let re = Regex::new(r"(?i)integrasi\.siman_aset\s+(?:AS\s+)?([a-z_][a-z0-9_]*)")
        .expect("static regex compiles");
    re.captures_iter(literal)
        .map(|c| c[1].to_ascii_lowercase())
        .filter(|a| !NOT_ALIASES.contains(&a.as_str()) || a == "sa")
        .collect()
}

/// Byte spans of every balanced `COALESCE( … )` in the literal.
fn coalesce_spans(literal: &str) -> Vec<(usize, usize)> {
    let lower = literal.to_ascii_lowercase();
    let bytes = literal.as_bytes();
    let mut spans = Vec::new();
    let mut from = 0usize;

    while let Some(rel) = lower[from..].find("coalesce") {
        let kw = from + rel;
        let mut i = kw + "coalesce".len();
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() || bytes[i] != b'(' {
            from = kw + 1;
            continue;
        }
        let mut depth = 0usize;
        let open = i;
        while i < bytes.len() {
            match bytes[i] {
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
            i += 1;
        }
        let close = (i + 1).min(bytes.len());
        spans.push((open, close));
        from = close.max(kw + 1);
    }
    spans
}

/// Strip what is legitimately allowed to mention a dead column name.
///
/// 1. SQL line comments — prose about this very defect names the columns
///    constantly.
/// 2. Rust `format!` placeholders (`{nup}`, `{kondisi_baik}`). These are
///    interpolation slots; the SQL they expand to lives in
///    `shared::siman_columns`, which is where the mapping is reviewed.
/// 3. Output aliases (`… AS kode_barang`). Renaming a corrected expression back
///    to the canonical name is the whole point of the idiom.
/// 4. A `COALESCE( … )` that names a dead column **together with a live one** —
///    `COALESCE(kondisi, ur_kondisi, '')` is the correct reading of an asset's
///    condition and must not be flagged.
///
/// Note what (4) deliberately does NOT exempt: `COALESCE(kategori_aset,
/// '(tanpa kategori)')`. Coalescing a dead column to a string literal is not a
/// fix, it is a mask — it stops the NULL panic while still bucketing all 624 533
/// assets under one made-up label. The rule is "fall back to a column SIMAN
/// actually fills", not "fall back to anything".
fn strip_permitted(literal: &str, dead: &BTreeSet<String>, live: &BTreeSet<String>) -> String {
    let mut out = literal.to_string();

    // (4) first, while offsets still refer to the original text.
    let mut spans = coalesce_spans(&out);
    spans.sort_by_key(|(a, _)| std::cmp::Reverse(*a));
    for (open, close) in spans {
        let inner = &out[open..close];
        let names: BTreeSet<String> = Regex::new(r"(?i)\b[a-z_][a-z0-9_]*\b")
            .expect("static regex compiles")
            .find_iter(inner)
            .map(|m| m.as_str().to_ascii_lowercase())
            .collect();
        let has_dead = names.iter().any(|n| dead.contains(n));
        let has_live = names.iter().any(|n| live.contains(n));
        if has_dead && has_live {
            out.replace_range(open..close, " ");
        }
    }

    let comments = Regex::new(r"(?m)--[^\n]*").expect("static regex compiles");
    let placeholders = Regex::new(r"\{[^{}]*\}").expect("static regex compiles");
    let aliases = Regex::new(r#"(?i)\bAS\s+"?[a-z_][a-z0-9_]*"?"#).expect("static regex compiles");

    let out = comments.replace_all(&out, " ").into_owned();
    let out = placeholders.replace_all(&out, " ").into_owned();
    aliases.replace_all(&out, " ").into_owned()
}

/// Every `src/` string literal that names `integrasi.siman_aset`, with its file
/// and line. Reuses the literal extractor from the phantom-relation guard so
/// both walk the source the same way (comments skipped, raw AND plain literals).
fn siman_literals(src_dir: &Path) -> Vec<(PathBuf, usize, String)> {
    let mut out = Vec::new();
    let mut stack = vec![src_dir.to_path_buf()];

    while let Some(dir) = stack.pop() {
        let entries = std::fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()));
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let src = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
            for (line, literal) in super::sql_relations_exist_test::string_literals(&src) {
                if literal
                    .to_ascii_lowercase()
                    .contains("integrasi.siman_aset")
                {
                    out.push((path.clone(), line, literal));
                }
            }
        }
    }
    out
}

#[tokio::test]
async fn no_sql_reads_a_siman_column_the_ingest_never_writes() {
    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let literals = siman_literals(&src_dir);

    assert!(
        !literals.is_empty(),
        "the scanner found no SQL against integrasi.siman_aset at all — it has \
         broken, which would make this guard silently vacuous"
    );

    let (db, db_name) = setup_test_db().await;
    let client = db.pool().get().await.expect("pool checkout");

    // ── Derive the dead set from the fixture, column by column ──────────────
    let columns: Vec<String> = client
        .query(
            "SELECT column_name FROM information_schema.columns
             WHERE table_schema = 'integrasi' AND table_name = 'siman_aset'
             ORDER BY ordinal_position",
            &[],
        )
        .await
        .expect("catalog query")
        .iter()
        .map(|r| r.get::<_, String>("column_name"))
        .collect();

    assert!(
        columns.len() > 10,
        "expected the SoT-shaped siman_aset stub, got {} columns",
        columns.len()
    );

    let mut populated: BTreeMap<String, i64> = BTreeMap::new();
    for col in &columns {
        // `col` comes from the catalog, not from user input, and is quoted as
        // an identifier regardless.
        // `btrim`, not a bare `<> ''`. The first census of this table recorded
        // `nama` as populated in all 624 533 rows; trimmed, it is blank in
        // 138 607 of them (22%) — the column is full of whitespace. A column
        // holding only spaces is dead in every way that matters here, and a
        // test that cannot see that would hand back a too-small dead set.
        let sql = format!(
            "SELECT count(*) AS c FROM integrasi.siman_aset \
             WHERE \"{col}\" IS NOT NULL AND btrim(\"{col}\"::text) <> ''"
        );
        let n: i64 = client
            .query_one(sql.as_str(), &[])
            .await
            .unwrap_or_else(|e| panic!("census of {col} failed: {e}"))
            .get("c");
        populated.insert(col.clone(), n);
    }

    let dead: BTreeSet<String> = populated
        .iter()
        .filter(|(_, n)| **n == 0)
        .map(|(c, _)| c.clone())
        .collect();

    // Floor: the fixture must still leave the known-dead columns empty.
    for known in KNOWN_DEAD {
        assert!(
            dead.contains(*known),
            "`{known}` is populated by the integration fixture, but SIMAN never \
             writes it (measured empty in 624 533 staging rows). A fixture that \
             fills a dead column does not miss the bug, it certifies it — and it \
             removes `{known}` from this guard's derived set, disabling the check \
             for the very column it exists to protect. Leave it NULL."
        );
    }

    let live: BTreeSet<String> = populated
        .iter()
        .filter(|(_, n)| **n > 0)
        .map(|(c, _)| c.clone())
        .collect();

    // ── Fail on any src/ SQL that reads one ─────────────────────────────────
    let mut offences: Vec<String> = Vec::new();
    for (path, line, literal) in &literals {
        let aliases = siman_aliases(literal);
        let haystack = strip_permitted(literal, &dead, &live);
        for col in &dead {
            // An optional `<qualifier>.` prefix decides ownership: unqualified
            // names belong to siman_aset (it is in the FROM), and qualified ones
            // only when the qualifier is one of its aliases.
            let re = Regex::new(&format!(
                r"(?i)(?:\b([a-z_][a-z0-9_]*)\.)?\b{}\b",
                regex::escape(col)
            ))
            .expect("column names are plain identifiers");

            for caps in re.captures_iter(&haystack) {
                let owned_by_siman = match caps.get(1) {
                    Some(q) => aliases.contains(&q.as_str().to_ascii_lowercase()),
                    None => true,
                };
                if owned_by_siman {
                    offences.push(format!(
                        "  {}:{} reads `{}`",
                        path.strip_prefix(src_dir.parent().unwrap_or(&src_dir))
                            .unwrap_or(path)
                            .display(),
                        line,
                        col
                    ));
                    break;
                }
            }
        }
    }
    offences.sort();
    offences.dedup();

    teardown_test_db(&db_name).await;

    assert!(
        offences.is_empty(),
        "SQL reads column(s) of integrasi.siman_aset that the SIMAN ingest \
         never writes, so the query returns empty/NULL against every real \
         asset while looking perfectly healthy:\n{}\n\n\
         Read the populated counterpart instead — the mapping is in \
         `crate::shared::siman_columns` (nup <- no_aset, kode_barang <- kd_brg, \
         nama_barang <- nama, kondisi <- ur_kondisi, kategori_aset <- \
         jenis_aset). If the column really is being written now, the fixture in \
         tests/common.rs must be updated first and this guard will follow.",
        offences.join("\n")
    );
}
