//! Standalone layanan-integrasi database migration runner.
//!
//! integrasi does NOT migrate on app startup — historically its schema was
//! created ad-hoc at sync time by `db::ensure_table_exists`, which made fresh
//! deploys non-deterministic (tables only appeared once data flowed). That is
//! a problem for consumers that read `integrasi.*` cross-schema before any sync
//! has run (perlengkapan `bank_aset`, its dashboard views, authenc federation).
//!
//! This one-shot binary applies the SQL migrations deterministically and
//! out-of-band so multiple app replicas never race on schema creation:
//!   * docker-compose: the `integrasi-migrate` service runs this before any
//!     consumer (`authenc-migrate`, `perlengkapan`) so `integrasi.siman_aset`
//!     etc. exist up front.
//!   * Kubernetes: a Helm pre-install/pre-upgrade hook Job runs it before the
//!     Deployments roll. Bring-up order is integrasi → authenc → perlengkapan.
//!
//! There is no tracking table: EVERY migration is applied on EVERY invocation,
//! in lexical order, and this binary is a `pre-upgrade` hook — so every
//! migration must CONVERGE, not merely succeed once against an empty database.
//! That distinction is not academic; see `layanan/integrasi/AGENTS.md` §0 and
//! `infra/scripts/check-migration-replay.sh`, which applies the list below
//! twice to the same database precisely because CI otherwise only ever
//! exercises the first pass. Each file is one implicit transaction via
//! `batch_execute` — a faithful match to how the SQL was validated against a
//! fresh Postgres locally.
//!
//! Exit code: 0 only if every migration applied; 1 on the first failure. The
//! failure is reported with the server's own message (see `explain`).

use tokio_postgres::NoTls;
use tokio_postgres::error::{DbError, ErrorPosition};
use tracing::{error, info};

/// Migrations embedded at compile time (no reliance on the image shipping the
/// `migrations/` directory). Applied in array order. `002_rollback.sql` is the
/// down-migration and is deliberately NOT included here.
const MIGRATIONS: &[(&str, &str)] = &[
    (
        "001_init_schema.sql",
        include_str!("../../migrations/001_init_schema.sql"),
    ),
    (
        "002_enhance_integration_schema.sql",
        include_str!("../../migrations/002_enhance_integration_schema.sql"),
    ),
    (
        "003_satker_code_mapping.sql",
        include_str!("../../migrations/003_satker_code_mapping.sql"),
    ),
    // MUST stay after 003: it repoints `v_satker_code_map` at the materialized
    // snapshot, and 003 recreates the slow definition on every run.
    (
        "004_satker_code_map_materialized.sql",
        include_str!("../../migrations/004_satker_code_map_materialized.sql"),
    ),
    (
        "005_satker_wilayah.sql",
        include_str!("../../migrations/005_satker_wilayah.sql"),
    ),
];

/// Render a `tokio_postgres::Error` using the server's own words.
///
/// `Display` for `tokio_postgres::Error` prints the error *kind*. For a
/// statement the server rejected, that kind is the bare string `db error` —
/// the Postgres message, its SQLSTATE, and the DETAIL/HINT lines that say what
/// to do about it all sit in the `DbError` behind it and are never shown.
///
/// That is not a cosmetic gap. When the rc28 staging upgrade failed, the only
/// line this binary produced was:
///
/// ```text
/// integrasi-migrate: 003_satker_code_mapping.sql failed: db error
/// ```
///
/// which names the file and then withholds every fact needed to act on it.
/// Recovering the message the server had ALREADY SENT required replaying the
/// migration by hand against the live database inside a rolled-back
/// transaction. authenc's runner learned this first and reaches for
/// `as_db_error()`; this one had not.
fn explain(e: &tokio_postgres::Error) -> String {
    let Some(db) = e.as_db_error() else {
        // Not a server-side rejection (connection dropped, TLS, protocol): the
        // Display chain is the informative part there, so keep it whole.
        return format!("{e}");
    };

    let mut out = format!("[{}] {}: {}", db.code().code(), db.severity(), db.message());
    if let Some(detail) = db.detail() {
        out.push_str(&format!("\n  DETAIL: {detail}"));
    }
    if let Some(hint) = db.hint() {
        out.push_str(&format!("\n  HINT: {hint}"));
    }
    if let Some(where_) = db.where_() {
        out.push_str(&format!("\n  CONTEXT: {where_}"));
    }
    out
}

/// Translate a Postgres error cursor into `line:column` within the migration.
///
/// Postgres reports the position as a 1-based **character** index into the
/// statement it was given. `batch_execute` hands it the whole file, so that
/// index is directly usable as a location in the `.sql` on disk — which is the
/// difference between "003 failed" and "003 line 41".
fn line_col(sql: &str, position: u32) -> (usize, usize) {
    let chars_before = (position as usize).saturating_sub(1);
    let byte_idx = sql
        .char_indices()
        .nth(chars_before)
        .map_or(sql.len(), |(i, _)| i);
    let before = &sql[..byte_idx];
    let line = before.matches('\n').count() + 1;
    let col = before.rsplit('\n').next().map_or(0, |l| l.chars().count()) + 1;
    (line, col)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    if let Err(e) = dotenvy::dotenv() {
        info!("No .env file found or error loading: {e}");
    }

    let database_url = match std::env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            error!("DATABASE_URL not set");
            std::process::exit(1);
        }
    };

    info!("integrasi-migrate: connecting…");
    let (client, connection) = match tokio_postgres::connect(&database_url, NoTls).await {
        Ok(pair) => pair,
        Err(e) => {
            error!("Failed to connect to database: {}", explain(&e));
            std::process::exit(1);
        }
    };
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("Database connection error: {e}");
        }
    });

    // `--refresh-satker-map`: refresh ONLY the satker code-map snapshot, without
    // re-applying any DDL. Used by the `integrasi-satker-map-refresh` CronJob,
    // which has to run between deploys (see 004_satker_code_map_materialized.sql:
    // a stale snapshot makes satker-scoped users fail closed to zero rows). The
    // full migrate path also refreshes, but re-running every CREATE OR REPLACE
    // against a live database on a 6-hourly timer is more blast radius than a
    // refresh needs.
    if std::env::args().any(|a| a == "--refresh-satker-map") {
        info!("integrasi-migrate: refreshing satker code-map snapshot…");
        if let Err(e) = client
            .batch_execute("REFRESH MATERIALIZED VIEW integrasi.mv_satker_code_map_auto;")
            .await
        {
            error!(
                "integrasi-migrate: satker code-map refresh failed: {}",
                explain(&e)
            );
            std::process::exit(1);
        }
        info!("integrasi-migrate: satker code-map refreshed ✅");
        return;
    }

    for (name, sql) in MIGRATIONS {
        info!("integrasi-migrate: applying {name}…");
        if let Err(e) = client.batch_execute(sql).await {
            let where_ = match e.as_db_error().and_then(DbError::position) {
                Some(ErrorPosition::Original(p)) => {
                    let (line, col) = line_col(sql, *p);
                    format!("{name} line {line} col {col}")
                }
                // `Internal` means the failure is inside a function or DO block
                // the migration invoked, so the cursor indexes that generated
                // body rather than the file — reporting it as a file line would
                // point at the wrong text.
                Some(ErrorPosition::Internal { position, query }) => {
                    let (line, col) = line_col(query, *position);
                    format!("{name}, inside an executed body at line {line} col {col}")
                }
                None => name.to_string(),
            };
            error!("integrasi-migrate: {where_} failed: {}", explain(&e));
            std::process::exit(1);
        }
    }

    info!(
        "integrasi-migrate: schema up to date ✅ ({} migrations)",
        MIGRATIONS.len()
    );
}

#[cfg(test)]
mod tests {
    use super::{MIGRATIONS, line_col};

    /// Postgres counts the error cursor in CHARACTERS, 1-based. Every migration
    /// in this repository is Indonesian-commented, so multi-byte characters sit
    /// upstream of most real error positions — a byte-indexed implementation
    /// would drift a line or two off and point at innocent SQL, which is worse
    /// than reporting nothing.
    #[test]
    fn position_is_counted_in_characters_not_bytes() {
        // "-- ĉ\nSELECT 1;" — the accented char is 2 bytes but 1 character.
        let sql = "-- \u{109}\nSELECT 1;";
        // Cursor on the `S` of SELECT: characters are -, -, space, ĉ, \n, S = 6.
        assert_eq!(line_col(sql, 6), (2, 1));
    }

    #[test]
    fn position_maps_to_line_and_column() {
        let sql = "line one\nline two\nline three";
        assert_eq!(line_col(sql, 1), (1, 1));
        assert_eq!(line_col(sql, 10), (2, 1));
        assert_eq!(line_col(sql, 12), (2, 3));
    }

    /// Postgres has been observed to report position 0 for some errors, and a
    /// position past the end when the statement was truncated. Neither may
    /// panic: a diagnostic that crashes the diagnostic is worse than `db error`.
    #[test]
    fn out_of_range_positions_do_not_panic() {
        let sql = "SELECT 1;";
        assert_eq!(line_col(sql, 0), (1, 1));
        assert_eq!(line_col(sql, 9_999), (1, sql.chars().count() + 1));
    }

    /// Every `.sql` in `migrations/` is actually embedded here.
    ///
    /// This list is hand-written while the directory beside it grows on its
    /// own, which is the shape of bug this repository keeps re-finding: the
    /// gate is right, its scope is a list nobody remembers to extend. A
    /// migration left out of the array is not a compile error and not a
    /// runtime error — the deploy hook simply never applies it, and the first
    /// symptom is a missing relation in production.
    #[test]
    fn every_migration_file_is_embedded() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/migrations");
        let mut on_disk: Vec<String> = std::fs::read_dir(dir)
            .expect("migrations/ must be readable")
            .filter_map(Result::ok)
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.ends_with(".sql"))
            // The down-migration is deliberately excluded from the array.
            .filter(|n| n != "002_rollback.sql")
            .collect();
        on_disk.sort();

        let mut embedded: Vec<String> = MIGRATIONS.iter().map(|(n, _)| n.to_string()).collect();
        embedded.sort();

        assert_eq!(
            embedded, on_disk,
            "migrations/ and the MIGRATIONS array disagree — a file on disk \
             that is not in the array is never applied by the deploy hook"
        );
    }

    /// The array is applied in order, so its order must be the lexical order
    /// the filenames encode. 004 repointing a view that 003 recreates is only
    /// correct because 003 runs first.
    #[test]
    fn migrations_are_embedded_in_lexical_order() {
        let names: Vec<&str> = MIGRATIONS.iter().map(|(n, _)| *n).collect();
        let mut sorted = names.clone();
        sorted.sort_unstable();
        assert_eq!(
            names, sorted,
            "MIGRATIONS must be in lexical filename order"
        );
    }
}
