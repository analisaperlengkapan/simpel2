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
//! The migrations are idempotent (`CREATE SCHEMA/TABLE IF NOT EXISTS`,
//! self-qualified via `SET search_path TO integrasi, public`), so re-running is
//! safe; we apply them in lexical order on every invocation. Each file is one
//! implicit transaction via `batch_execute` — a faithful match to how the SQL
//! was validated against a fresh Postgres locally.
//!
//! Exit code: 0 only if every migration applied; 1 on the first failure.

use tokio_postgres::NoTls;
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
];

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
            error!("Failed to connect to database: {e}");
            std::process::exit(1);
        }
    };
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("Database connection error: {e}");
        }
    });

    for (name, sql) in MIGRATIONS {
        info!("integrasi-migrate: applying {name}…");
        if let Err(e) = client.batch_execute(sql).await {
            error!("integrasi-migrate: {name} failed: {e}");
            std::process::exit(1);
        }
    }

    info!(
        "integrasi-migrate: schema up to date ✅ ({} migrations)",
        MIGRATIONS.len()
    );
}
