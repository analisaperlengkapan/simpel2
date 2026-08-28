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

/// Pull the contents of every string literal — raw (`r#"…"#`, `r"…"`) AND
/// plain (`"…"`) — out of a source file, with the line each one starts on.
///
/// Comments are skipped outright, which is what keeps prose out of the results:
/// this file and the fix commentary in `documents.rs` both mention
/// `perlengkapan.kebutuhan_bmn` in comments. Stripping comments is exact here
/// because we are walking the source, not pattern-matching it.
///
/// Plain literals must be scanned too, not just raw ones: `admin/users.rs`
/// builds its statement with `String::from("SELECT … FROM v_user_role_summary")`
/// — a plain literal. A raw-only scanner has a blind spot precisely where #123
/// lives, which is how that bug survived the first version of this guard.
///
/// `pub(super)` because `siman_dead_columns_test` walks the same source the
/// same way; two extractors would drift and one of them would develop a blind
/// spot, which is exactly how #123 survived the first version of this guard.
pub(super) fn string_literals(src: &str) -> Vec<(usize, String)> {
    let b = src.as_bytes();
    let mut out = Vec::new();
    let mut i = 0usize;
    let mut line = 1usize;

    while i < b.len() {
        match b[i] {
            b'\n' => {
                line += 1;
                i += 1;
            }
            // Line comment
            b'/' if i + 1 < b.len() && b[i + 1] == b'/' => {
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                }
            }
            // Block comment
            b'/' if i + 1 < b.len() && b[i + 1] == b'*' => {
                i += 2;
                while i + 1 < b.len() && !(b[i] == b'*' && b[i + 1] == b'/') {
                    if b[i] == b'\n' {
                        line += 1;
                    }
                    i += 1;
                }
                i = (i + 2).min(b.len());
            }
            // Raw literal: r, then any number of #, then "
            b'r' if i + 1 < b.len() && (b[i + 1] == b'#' || b[i + 1] == b'"') => {
                let mut j = i + 1;
                let mut hashes = 0usize;
                while j < b.len() && b[j] == b'#' {
                    hashes += 1;
                    j += 1;
                }
                if j >= b.len() || b[j] != b'"' {
                    i += 1;
                    continue;
                }
                let start_line = line;
                let content_start = j + 1;
                let terminator = format!("\"{}", "#".repeat(hashes));
                match src[content_start..].find(&terminator) {
                    Some(rel) => {
                        let content = &src[content_start..content_start + rel];
                        line += content.matches('\n').count();
                        out.push((start_line, content.to_string()));
                        i = content_start + rel + terminator.len();
                    }
                    None => break,
                }
            }
            // Plain literal, honouring backslash escapes
            b'"' => {
                let start_line = line;
                let mut j = i + 1;
                let mut content = String::new();
                while j < b.len() {
                    match b[j] {
                        b'\\' => {
                            // Keep a space so `\n` joins tokens rather than
                            // gluing them together.
                            content.push(' ');
                            j += 2;
                        }
                        b'"' => break,
                        c => {
                            if c == b'\n' {
                                line += 1;
                            }
                            content.push(c as char);
                            j += 1;
                        }
                    }
                }
                out.push((start_line, content));
                i = j + 1;
            }
            _ => i += 1,
        }
    }

    out
}

/// Does this literal actually contain a SQL statement?
///
/// Applied to every literal, because now that plain strings are scanned,
/// ordinary prose would otherwise register as relations — "failed to send
/// update to client" reads as `UPDATE to`, and "batch update status completed"
/// as `UPDATE status`.
///
/// So a bare verb is not enough: each pattern below pairs the verb with the
/// clause that must accompany it in real SQL (`SELECT … FROM`, `UPDATE … SET`).
/// English prose essentially never satisfies those pairs.
fn looks_like_sql(literal: &str) -> bool {
    let re = Regex::new(
        r"(?is)\bSELECT\b.*\bFROM\b|\bINSERT\s+INTO\b|\bUPDATE\b.*\bSET\b|\bDELETE\s+FROM\b",
    )
    .expect("static regex compiles");
    re.is_match(literal)
}

/// SQL keywords that can legally follow FROM/JOIN/INTO/UPDATE without being a
/// relation name. These are grammar, not tables:
///   `JOIN LATERAL (…)`, `EXTRACT(YEAR FROM CURRENT_DATE)`,
///   `INSERT … ON CONFLICT DO UPDATE SET`, `FROM ONLY tbl`.
const SQL_KEYWORDS: &[&str] = &[
    "lateral",
    "set",
    "only",
    "select",
    "values",
    "current_date",
    "current_time",
    "current_timestamp",
    "unnest",
    "generate_series",
];

/// Names bound by `WITH <name> AS (` / `, <name> AS (`.
///
/// Collected per FILE, not per literal: several repositories assemble one
/// statement from multiple raw literals via `format!`, so the CTE can be
/// declared in a different literal than the one that reads it. Per-literal
/// scoping produced false positives for exactly that reason
/// (`workflow/monitoring.rs` `current_state` / `sla`).
///
/// A CTE is the only relation that is legitimately unqualified, so this set is
/// what makes the unqualified check below trustworthy.
fn cte_names(src: &str) -> BTreeSet<String> {
    // Matches `name AS (` and `name(col, …) AS (`, and does NOT require a
    // preceding `WITH`/`,`: `workflow/monitoring.rs` keeps a CTE body in its
    // own const fragment that is interpolated after a `WITH`, so the binding
    // has no prefix in the text where it is written.
    //
    // `<name> [ (cols) ] AS (` is an unambiguous CTE signature — a column
    // alias is never followed by an opening parenthesis.
    let re = Regex::new(r"(?i)\b([a-z_][a-z0-9_]*)\s*(?:\([^()]*\))?\s+AS\s*\(")
        .expect("static regex compiles");
    re.captures_iter(src)
        .map(|c| c[1].to_ascii_lowercase())
        .collect()
}

/// Walk `src/` and collect every relation that appears in a
/// FROM / JOIN / INTO / UPDATE position inside a raw string literal.
///
/// Both qualified (`schema.table`) and UNQUALIFIED (`table`) references are
/// collected. The unqualified half matters just as much here: this service
/// never sets `search_path` (0 hits in `src/`), so an unqualified identifier
/// resolves against the default `"$user", public` and fails in EVERY
/// environment — while all of its tables live in the `perlengkapan` schema.
/// #123 (`FROM v_user_role_summary`) is exactly that shape, and it slipped
/// past the first version of this guard, which only looked at qualified names.
/// Blank out SQL comments (`-- …` to end of line, and `/* … */`) inside an SQL
/// literal, preserving every newline so reported line offsets stay exact.
///
/// The scanner strips *Rust* comments before it ever sees a literal, but SQL
/// written inside a raw string can carry its own commentary, and prose is not
/// SQL. Without this, a note reading "infer the type from the target column
/// the way it does for VALUES" parses as `FROM the` and the guard reports a
/// phantom relation named `the`. That is the third time a guard in this repo
/// has read a comment as code (#856, #857); strip first, match second.
pub(super) fn strip_sql_comments(literal: &str) -> String {
    let bytes: Vec<char> = literal.chars().collect();
    let mut out = String::with_capacity(literal.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == '-' && i + 1 < bytes.len() && bytes[i + 1] == '-' {
            while i < bytes.len() && bytes[i] != '\n' {
                out.push(' ');
                i += 1;
            }
            continue;
        }
        if bytes[i] == '/' && i + 1 < bytes.len() && bytes[i + 1] == '*' {
            let mut depth = 1;
            out.push_str("  ");
            i += 2;
            while i < bytes.len() && depth > 0 {
                if bytes[i] == '*' && i + 1 < bytes.len() && bytes[i + 1] == '/' {
                    depth -= 1;
                    out.push_str("  ");
                    i += 2;
                    continue;
                }
                out.push(if bytes[i] == '\n' { '\n' } else { ' ' });
                i += 1;
            }
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    out
}

fn scan_relation_refs(src_dir: &Path) -> Vec<RelationRef> {
    // `INTO` also covers `INSERT INTO`; `UPDATE` covers the bare form. The
    // optional `\.<name>` group is what distinguishes the two cases: when it
    // is absent the reference is unqualified. A trailing `(` cannot match
    // `[a-z_]`, so `FROM (SELECT …)` and function calls do not register.
    let re = Regex::new(
        r"(?i)\b(?:FROM|JOIN|INTO|UPDATE)\s+([a-z_][a-z0-9_]*)(?:\.([a-z_][a-z0-9_]*))?\b",
    )
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

            let ctes = cte_names(&src);

            for (start_line, literal) in string_literals(&src) {
                if !looks_like_sql(&literal) {
                    continue;
                }
                // Prose inside the SQL is not SQL — see `strip_sql_comments`.
                let literal = strip_sql_comments(&literal);
                let literal = literal.as_str();
                for caps in re.captures_iter(literal) {
                    let offset = caps
                        .get(0)
                        .map(|m| literal[..m.start()].matches('\n').count())
                        .unwrap_or(0);

                    // `UPDATE perlengkapan.{} SET …` — the relation is a
                    // `format!` placeholder, so group 2 cannot match, but the
                    // reference IS schema-qualified. A trailing '.' is the
                    // tell; without this check `perlengkapan` itself would be
                    // reported as a bare relation
                    // (`workflow/engine/transition.rs`).
                    let m = caps.get(0).expect("group 0 always present");
                    if caps.get(2).is_none() && literal[m.end()..].starts_with('.') {
                        continue;
                    }

                    // Group 2 present => `schema.relation`; absent => bare name.
                    let (schema, relation) = match caps.get(2) {
                        Some(rel) => (
                            caps[1].to_ascii_lowercase(),
                            rel.as_str().to_ascii_lowercase(),
                        ),
                        None => (String::new(), caps[1].to_ascii_lowercase()),
                    };

                    // A CTE is the one relation that is *supposed* to be bare;
                    // a keyword is not a relation at all.
                    if schema.is_empty()
                        && (ctes.contains(&relation) || SQL_KEYWORDS.contains(&relation.as_str()))
                    {
                        continue;
                    }

                    found.push(RelationRef {
                        schema,
                        relation,
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
        // Unqualified, and not a CTE. This service never sets `search_path`,
        // so the name resolves against `"$user", public` while every table it
        // owns lives in `perlengkapan` — it cannot resolve in any environment,
        // whether or not something by that name exists somewhere.
        if r.schema.is_empty() {
            missing.insert(format!(
                "  {} references bare `{}` — unqualified, and this service \
                 never sets search_path, so it resolves against \"$user\",public \
                 and fails everywhere. Qualify it with its schema.",
                r.location, r.relation
            ));
            continue;
        }
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

/// The guard must not read prose as SQL — both directions.
#[test]
fn sql_comments_are_stripped_before_relations_are_matched() {
    let re = regex::Regex::new(
        r"(?i)\b(?:FROM|JOIN|INTO|UPDATE)\s+([a-z_][a-z0-9_]*)(?:\.([a-z_][a-z0-9_]*))?\b",
    )
    .unwrap();

    // The exact shape that produced a phantom relation named `the`.
    let sql = "-- infer the type from the target column the way it does\n\
               SELECT 1 FROM integrasi.mysimkari_pegawai p\n";
    let stripped = strip_sql_comments(sql);
    let hits: Vec<String> = re
        .captures_iter(&stripped)
        .map(|c| c[1].to_string())
        .collect();
    assert_eq!(hits, vec!["integrasi".to_string()], "prose must not match");

    // Line numbering must survive, or every report points at the wrong line.
    assert_eq!(stripped.matches('\n').count(), sql.matches('\n').count());

    // And the guard must still SEE real SQL: a stripper that ate everything
    // would make this whole test file pass while checking nothing.
    assert!(
        strip_sql_comments("SELECT 1 FROM perlengkapan.audit_log")
            .contains("FROM perlengkapan.audit_log")
    );

    // Block comments too.
    let block = "SELECT 1 /* FROM ghost_table */ FROM perlengkapan.audit_log";
    let stripped = strip_sql_comments(block);
    assert!(!stripped.contains("ghost_table"));
    assert!(stripped.contains("FROM perlengkapan.audit_log"));
}
