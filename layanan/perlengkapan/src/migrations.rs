//! Schema migration runner.
//!
//! Migrations live under `migrations/V<NNN>__<name>.sql` and are embedded at
//! compile time via the [`refinery::embed_migrations!`] macro. The runner
//! creates / reads a `refinery_schema_history` table to track which
//! migrations have already been applied, so calling [`run`] repeatedly is
//! safe — it only applies what is new.
//!
//! Wired from `main.rs` after the deadpool-postgres pool is initialised and
//! before [`Database::migrate`](crate::database::Database::migrate)'s legacy
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
    Ok(())
}
