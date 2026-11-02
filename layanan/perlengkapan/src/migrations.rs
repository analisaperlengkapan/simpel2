//! Schema migration runner.
//!
//! Migrations live under `migrations/V<NNN>__<name>.sql` and are embedded at
//! compile time via the [`refinery::embed_migrations!`] macro. The runner
//! creates / reads a `refinery_schema_history` table to track which
//! migrations have already been applied, so calling [`run`] repeatedly is
//! safe — it only applies what is new.
//!
//! Wired from `main.rs` after the deadpool-postgres pool is initialised and
//! before [`Database::migrate`](crate::shared::db::Database::migrate)'s legacy
//! hand-coded `CREATE TABLE` statements. Eventually the legacy code path
//! will be removed entirely and the embedded migrations become the single
//! source of truth.

mod embedded {
    refinery::embed_migrations!("migrations");
}

use deadpool_postgres::Pool;
use tracing::info;

/// Run all pending embedded migrations against the given pool.
pub async fn run(pool: &Pool) -> anyhow::Result<()> {
    let mut client = pool.get().await?;
    // `**client` yields the `&mut tokio_postgres::Client` refinery expects.
    let report = embedded::migrations::runner()
        // A rolled-back release is OLDER than the schema it wakes up on: the
        // newer release applied migrations this binary has never heard of.
        // refinery's default (`abort_missing = true`) turns that into
        // `MissingVersion` and, because `main` propagates this error, into a
        // pod that exits before serving — so `helm rollback` after ANY
        // migration, however additive, produced a crash loop and the
        // expand/contract rule in layanan/AGENTS.md bought nothing. With this
        // off, refinery logs the unknown migration at ERROR and carries on.
        //
        // `abort_divergent` stays on: an applied migration whose CHECKSUM no
        // longer matches means someone edited a file after it shipped, which
        // is a different problem and should stop the deploy.
        //
        // Only a binary built WITH this line benefits: the release that
        // introduced it still cannot be rolled back onto one that predates it.
        .set_abort_missing(false)
        .run_async(&mut **client)
        .await?;
    let applied = report.applied_migrations();
    if applied.is_empty() {
        info!("Refinery: no new migrations to apply");
    } else {
        info!("Refinery: applied {} migration(s)", applied.len());
        for m in applied {
            info!("  ↳ V{} {}", m.version(), m.name());
        }
    }
    drop(client);

    // Refinery runs a version once. Anything that has to hold after EVERY
    // start — because an older release may have written rows in the meantime —
    // cannot live in a versioned migration.
    normalize_pemakaian_items(pool).await
}

/// SQL that repairs `pemakaian_bmn_items` rows written without the V014 fields.
const NORMALIZE_PEMAKAIAN_ITEMS: &str = include_str!("../sql/normalize_pemakaian_items.sql");

/// Give every permit's assets a position, a kind and a period.
///
/// Runs on every startup, after refinery and before the service serves. Its job
/// is the rows that exist BEFORE V014's triggers do: legacy items and
/// header-held first assets from a release that predates V014, which have no
/// position and would reach `PemakaianBmnItem::from_row` as NULLs and panic the
/// first request that touches their permit. Refinery runs V014 once, so this
/// is the repair — and it is also the first-apply backfill.
///
/// It does NOT cover writes that arrive after it ran. During a rolling upgrade
/// old pods keep serving next to new ones, and a startup pass cannot see what
/// they insert afterwards; V014 section 6 normalises those at insert with
/// triggers. A start with nothing to repair writes nothing.
///
/// One transaction, serialised across replicas by an advisory lock inside the
/// SQL, idempotent.
pub async fn normalize_pemakaian_items(pool: &Pool) -> anyhow::Result<()> {
    let mut client = pool.get().await?;
    let tx = client.transaction().await?;
    tx.batch_execute(NORMALIZE_PEMAKAIAN_ITEMS).await?;
    tx.commit().await?;
    Ok(())
}
