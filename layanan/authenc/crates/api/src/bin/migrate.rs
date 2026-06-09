//! Standalone authenc database migration runner.
//!
//! authenc does NOT migrate on app startup (see `main.rs` — it connects and
//! serves). Schema is applied out-of-band by this one-shot binary so multiple
//! app replicas never race on migrations:
//!   * docker-compose: the `authenc-migrate` service runs this before `authenc`.
//!   * Kubernetes: a Helm pre-install/pre-upgrade hook Job runs it before the
//!     Deployment rolls.
//!
//! Topology (F5-B): authenc shares the `dbsimpelv2` database with perlengkapan.
//! Most authenc tables live in `public` (the app queries them unqualified); only
//! a few are referenced schema-qualified (`authenc.token_revocations` [050],
//! `authenc.totp_secrets` / `authenc.mfa_backup_codes` [049]). So we run with
//! the default search_path (public) and just ensure the `authenc` schema exists
//! up front for those qualified CREATE TABLEs.
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

    info!("authenc-migrate: connecting (default search_path = public)…");
    let db = match Database::new(&database_url, 5).await {
        Ok(db) => db,
        Err(e) => {
            error!("Failed to connect to database: {e}");
            std::process::exit(1);
        }
    };

    // Ensure the `authenc` schema exists before the schema-qualified CREATE
    // TABLEs (049/050). Unqualified tables go to public (default search_path).
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
