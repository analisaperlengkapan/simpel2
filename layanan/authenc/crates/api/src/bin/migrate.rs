//! Standalone authenc database migration runner.
//!
//! authenc does NOT migrate on app startup (see `main.rs` — it connects and
//! serves). Schema is applied out-of-band by this one-shot binary so multiple
//! app replicas never race on migrations:
//!   * docker-compose: the `authenc-migrate` service runs this before `authenc`.
//!   * Kubernetes: a Helm pre-install/pre-upgrade hook Job runs it before the
//!     Deployment rolls.
//!
//! It pins `search_path=authenc,public` on every connection and creates the
//! `authenc` schema up front, so all tables are created in `authenc` from
//! migration 001 onward and `040_use_authenc_schema`'s in-place move becomes a
//! no-op — making the accreted migration set fresh-appliable (F5-B).
//!
//! Exit code: 0 only if every pending migration applied; 1 on any failure
//! (the runner is fail-soft per-migration, so we inspect the aggregate result).

use authenc_storage::{Database, run_migrations};
use tracing::{error, info};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let database_url = match std::env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            error!("DATABASE_URL not set");
            std::process::exit(1);
        }
    };

    info!("authenc-migrate: connecting (search_path=authenc,public)…");
    let db = match Database::new_with_search_path(&database_url, 5, Some("authenc,public")).await {
        Ok(db) => db,
        Err(e) => {
            error!("Failed to connect to database: {e}");
            std::process::exit(1);
        }
    };

    // Ensure the dedicated schema exists before any unqualified CREATE TABLE
    // (which would otherwise target the first entry in search_path = authenc).
    match db.get_connection().await {
        Ok(client) => {
            if let Err(e) = client
                .batch_execute("CREATE SCHEMA IF NOT EXISTS authenc")
                .await
            {
                error!("Failed to ensure authenc schema: {e}");
                std::process::exit(1);
            }
        }
        Err(e) => {
            error!("Failed to acquire connection: {e}");
            std::process::exit(1);
        }
    }

    info!("authenc-migrate: running migrations…");
    match run_migrations(db.pool()).await {
        Ok(result) => {
            info!(
                "authenc-migrate: {} applied, {} skipped, {} errors",
                result.applied,
                result.skipped,
                result.errors.len()
            );
            if !result.errors.is_empty() {
                for err in &result.errors {
                    error!("migration error: {err}");
                }
                std::process::exit(1);
            }
            info!("authenc-migrate: schema up to date ✅");
        }
        Err(e) => {
            error!("authenc-migrate: runner failed: {e:#}");
            std::process::exit(1);
        }
    }
}
