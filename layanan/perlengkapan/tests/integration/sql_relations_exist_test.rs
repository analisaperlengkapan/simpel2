//! Guard against the "phantom relation" defect class (#113, #118, #123).
//!
//! Three separate times now, a module has shipped SQL against a table that
//! exists in no environment and never did. Every existing signal missed it:
//!
//! - `cargo check` cannot see inside a SQL string literal.
//! - `psql` eyeballing only proves the table you happened to look at.
//! - The failures are frequently SWALLOWED — `WorkflowEngine::transition`
//!   logs document/notification errors and carries on, so approving a
//!   kebutuhan campaign silently produced no SK and no notification for as
//!   long as the feature has existed.
//! - Some sites are simply never called (`get_current_state`,
//!   `NotificationScheduler`), so the broken query is never issued and the
//!   rot is invisible until someone wires it up.
//!
//! So this test does not check a hand-written list of queries — a list would
//! rot the same way. It SCANS the crate's own source for every schema-qualified
//! relation the code references and asserts each one exists in a freshly
//! migrated database. Add a query against a table nobody created and this test
//! names the file, the line, and the relation.
//!
//! Scope: `perlengkapan.*` only. Relations in `authenc.*` / `integrasi.*` are
//! owned by other services and deliberately absent from this isolated harness
//! (`common::setup_test_db` stands up only an `authenc.users` stub), so
//! asserting them here would fail for the wrong reason. They are collected and
//! printed for visibility instead.

use crate::common::{setup_test_db, teardown_test_db};
use regex::Regex;
use std::collections::BTreeSet;
use std::path::Path;

/// One relation reference found in the source.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct RelationRef {
    schema: String,
    relation: String,
    location: String,
}

/// Pull the contents of every raw string literal (`r#"..."#`) out of a source
/// file, together with the line each one starts on.
///
/// Raw strings are where all of this crate's SQL lives, and restricting the
/// scan to them is what keeps prose out of the results — this very file, and
/// the fix commentary in `documents.rs`, mention `perlengkapan.kebutuhan_bmn`
/// in comments. A comment-stripping heuristic would be the fragile way to
/// handle that; only ever looking inside raw literals is the exact way.
fn raw_string_literals(src: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let bytes = src.as_bytes();
    let mut i = 0;

    while let Some(start) = src[i..].find("r#\"") {
        let open = i + start;
        let content_start = open + 3;
        match src[content_start..].find("\"#") {
            Some(rel_end) => {
                let content_end = content_start + rel_end;
                let line = bytes[..open].iter().filter(|&&b| b == b'\n').count() + 1;
                out.push((line, &src[content_start..content_end]));
                i = content_end + 2;
            }
            // Unterminated literal: cannot happen in code that compiles.
            None => break,
        }
    }

    out
}

/// Walk `src/` and collect every `<schema>.<relation>` that appears in a
/// FROM / JOIN / INTO / UPDATE position inside a raw string literal.
fn scan_relation_refs(src_dir: &Path) -> Vec<RelationRef> {
    // `INTO` also covers `INSERT INTO`; `UPDATE` covers the bare form. The
    // relation part deliberately does not match a trailing `(`, so CTE and
    // function calls do not register as tables.
    let re =
        Regex::new(r"(?i)\b(?:FROM|JOIN|INTO|UPDATE)\s+([a-z_][a-z0-9_]*)\.([a-z_][a-z0-9_]*)\b")
            .expect("static regex compiles");

    let mut found = Vec::new();
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

            for (start_line, literal) in raw_string_literals(&src) {
                for caps in re.captures_iter(literal) {
                    let offset = caps
                        .get(0)
                        .map(|m| literal[..m.start()].matches('\n').count())
                        .unwrap_or(0);
                    found.push(RelationRef {
                        schema: caps[1].to_ascii_lowercase(),
                        relation: caps[2].to_ascii_lowercase(),
                        location: format!(
                            "{}:{}",
                            path.strip_prefix(src_dir.parent().unwrap_or(src_dir))
                                .unwrap_or(&path)
                                .display(),
                            start_line + offset
                        ),
                    });
                }
            }
        }
    }

    found
}

#[tokio::test]
async fn every_perlengkapan_relation_referenced_in_sql_actually_exists() {
    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let refs = scan_relation_refs(&src_dir);

    assert!(
        !refs.is_empty(),
        "the scanner found no schema-qualified SQL at all — it has broken, \
         which would make this guard silently vacuous"
    );

    let (db, db_name) = setup_test_db().await;

    // Every relation the migrated database actually provides, tables and views
    // alike (several modules read views, so `information_schema.tables` alone
    // is not enough — it excludes nothing here but the intent matters).
    let client = db.pool().get().await.expect("pool checkout");
    let rows = client
        .query(
            "SELECT table_schema, table_name FROM information_schema.tables
             UNION
             SELECT table_schema, table_name FROM information_schema.views",
            &[],
        )
        .await
        .expect("catalog query");

    let existing: BTreeSet<(String, String)> = rows
        .iter()
        .map(|r| {
            (
                r.get::<_, String>("table_schema").to_ascii_lowercase(),
                r.get::<_, String>("table_name").to_ascii_lowercase(),
            )
        })
        .collect();

    let mut missing = BTreeSet::new();
    let mut foreign = BTreeSet::new();

    for r in &refs {
        if r.schema != "perlengkapan" {
            foreign.insert(format!("{}.{}", r.schema, r.relation));
            continue;
        }
        if !existing.contains(&(r.schema.clone(), r.relation.clone())) {
            missing.insert(format!(
                "  {} references perlengkapan.{} — no migration creates it",
                r.location, r.relation
            ));
        }
    }

    if !foreign.is_empty() {
        println!(
            "cross-service relations referenced (not asserted here, owned by \
             another service): {}",
            foreign.into_iter().collect::<Vec<_>>().join(", ")
        );
    }

    teardown_test_db(&db_name).await;

    assert!(
        missing.is_empty(),
        "SQL references {} perlengkapan relation(s) that do not exist in a \
         freshly migrated database:\n{}",
        missing.len(),
        missing.into_iter().collect::<Vec<_>>().join("\n")
    );
}
