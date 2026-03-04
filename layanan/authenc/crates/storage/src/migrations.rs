//! Database migration utilities for Authenc
//!
//! This module provides migration management for the Authenc database schema.
//! Migrations are automatically run on application startup to ensure the database
//! schema is up-to-date.

use anyhow::{Context, Result};
use deadpool_postgres::Pool;
use std::path::Path;
use tokio::fs;
use tracing::{error, info, warn};

/// Migration entry representing a single migration file
#[derive(Debug, Clone)]
pub struct Migration {
    /// Migration version/order number
    pub version: i32,
    /// Migration name (from filename)
    pub name: String,
    /// SQL content
    pub sql: String,
}

/// Migration runner that applies migrations to the database
pub struct MigrationRunner {
    pool: Pool,
    migrations_dir: String,
}

impl MigrationRunner {
    /// Create a new migration runner
    pub fn new(pool: Pool, migrations_dir: impl Into<String>) -> Self {
        Self {
            pool,
            migrations_dir: migrations_dir.into(),
        }
    }

    /// Create the schema_migrations tracking table if it doesn't exist
    async fn ensure_migrations_table(&self) -> Result<()> {
        let client = self
            .pool
            .get()
            .await
            .context("Failed to get database connection")?;

        client
            .execute(
                r#"
            CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                checksum VARCHAR(64)
            )
            "#,
                &[],
            )
            .await
            .context("Failed to create schema_migrations table")?;

        info!("✅ Migration tracking table ensured");
        Ok(())
    }

    /// Get list of applied migration versions
    async fn get_applied_versions(&self) -> Result<Vec<i32>> {
        let client = self
            .pool
            .get()
            .await
            .context("Failed to get database connection")?;

        let rows = client
            .query(
                "SELECT version FROM schema_migrations ORDER BY version",
                &[],
            )
            .await
            .context("Failed to query applied migrations")?;

        let versions: Vec<i32> = rows.iter().map(|row| row.get("version")).collect();
        Ok(versions)
    }

    /// Record a migration as applied
    async fn record_migration(&self, migration: &Migration, checksum: &str) -> Result<()> {
        let client = self
            .pool
            .get()
            .await
            .context("Failed to get database connection")?;

        client.execute(
            "INSERT INTO schema_migrations (version, name, checksum) VALUES ($1, $2, $3) ON CONFLICT (version) DO NOTHING",
            &[&migration.version, &migration.name, &checksum],
        ).await.context("Failed to record migration")?;

        Ok(())
    }

    /// Load migrations from the migrations directory
    async fn load_migrations(&self) -> Result<Vec<Migration>> {
        let migrations_path = Path::new(&self.migrations_dir);
        let mut migrations = Vec::new();

        if !migrations_path.exists() {
            warn!("Migrations directory not found: {}", self.migrations_dir);
            return Ok(migrations);
        }

        let mut entries = fs::read_dir(migrations_path)
            .await
            .context("Failed to read migrations directory")?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "sql") {
                let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                // Parse version from filename (supports both "008_name.sql" and "V009__name.sql" formats)
                let version = if filename.starts_with('V') {
                    // Format: V009__name.sql
                    filename
                        .trim_start_matches('V')
                        .split("__")
                        .next()
                        .and_then(|v| v.parse::<i32>().ok())
                } else {
                    // Format: 008_name.sql
                    filename
                        .split('_')
                        .next()
                        .and_then(|v| v.parse::<i32>().ok())
                };

                if let Some(version) = version {
                    let sql = fs::read_to_string(&path)
                        .await
                        .with_context(|| format!("Failed to read migration file: {:?}", path))?;

                    migrations.push(Migration {
                        version,
                        name: filename.to_string(),
                        sql,
                    });
                } else {
                    warn!("Skipping file with invalid name format: {}", filename);
                }
            }
        }

        // Sort by version
        migrations.sort_by_key(|m| m.version);
        Ok(migrations)
    }

    /// Calculate checksum for migration content
    fn calculate_checksum(sql: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        sql.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    /// Run all pending migrations
    pub async fn run(&self) -> Result<MigrationResult> {
        info!("🔄 Starting database migrations...");

        // Ensure tracking table exists
        self.ensure_migrations_table().await?;

        // Get applied versions
        let applied = self.get_applied_versions().await?;
        info!("📋 Already applied migrations: {:?}", applied);

        // Load migrations from disk
        let migrations = self.load_migrations().await?;
        info!("📂 Found {} migration files", migrations.len());

        let mut applied_count = 0;
        let mut skipped_count = 0;
        let mut errors = Vec::new();

        for migration in migrations {
            if applied.contains(&migration.version) {
                skipped_count += 1;
                continue;
            }

            info!(
                "⏳ Applying migration {} - {}...",
                migration.version, migration.name
            );

            let client = self
                .pool
                .get()
                .await
                .context("Failed to get database connection")?;

            // Execute migration in a transaction
            match client.batch_execute(&migration.sql).await {
                Ok(()) => {
                    let checksum = Self::calculate_checksum(&migration.sql);
                    self.record_migration(&migration, &checksum).await?;
                    info!(
                        "✅ Applied migration {} - {}",
                        migration.version, migration.name
                    );
                    applied_count += 1;
                }
                Err(e) => {
                    let error_msg = format!(
                        "Failed to apply migration {} - {}: {}",
                        migration.version, migration.name, e
                    );
                    error!("❌ {}", error_msg);
                    errors.push(error_msg);
                    // Continue with other migrations instead of failing completely
                }
            }
        }

        let result = MigrationResult {
            applied: applied_count,
            skipped: skipped_count,
            errors,
        };

        if result.errors.is_empty() {
            info!(
                "✅ Database migrations completed: {} applied, {} skipped",
                result.applied, result.skipped
            );
        } else {
            warn!(
                "⚠️ Database migrations completed with errors: {} applied, {} skipped, {} errors",
                result.applied,
                result.skipped,
                result.errors.len()
            );
        }

        Ok(result)
    }
}

/// Result of running migrations
#[derive(Debug)]
pub struct MigrationResult {
    /// Number of migrations successfully applied
    pub applied: usize,
    /// Number of migrations skipped (already applied)
    pub skipped: usize,
    /// List of error messages for failed migrations
    pub errors: Vec<String>,
}

impl MigrationResult {
    /// Check if all migrations were successful
    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Run database migrations using the provided connection pool
///
/// This is the main entry point for running migrations. It reads migration files
/// from the `migrations/` directory and applies any pending migrations to the database.
/// The migrations folder should contain all SQL files including:
/// - 001_initial_schema.sql (base schema)
/// - 008_*.sql onwards (incremental migrations)
pub async fn run_migrations(pool: Pool) -> Result<MigrationResult> {
    // Try multiple possible paths for migrations directory
    let migrations_paths = ["migrations", "/app/migrations", "./migrations"];

    let mut migrations_dir = "migrations".to_string();
    for path in &migrations_paths {
        if Path::new(path).exists() {
            migrations_dir = path.to_string();
            break;
        }
    }

    info!("📁 Using migrations directory: {}", migrations_dir);

    let runner = MigrationRunner::new(pool, migrations_dir);
    runner.run().await
}

/// Alias for run_migrations - kept for backward compatibility
///
/// Previously this would run schema.sql separately, but now all migrations
/// including the initial schema are in the migrations/ folder.
pub async fn run_migrations_with_schema(pool: Pool) -> Result<MigrationResult> {
    run_migrations(pool).await
}

/// Check migration status
pub async fn check_migration_status(pool: Pool) -> Result<MigrationStatus> {
    let client = pool
        .get()
        .await
        .context("Failed to get database connection")?;

    // Check if migrations table exists
    let table_exists = client
        .query_one(
            "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'schema_migrations')",
            &[],
        )
        .await
        .map(|row| row.get::<_, bool>(0))
        .unwrap_or(false);

    if !table_exists {
        return Ok(MigrationStatus {
            initialized: false,
            applied_count: 0,
            latest_version: None,
            pending_count: 0,
        });
    }

    let rows = client
        .query(
            "SELECT version, name, applied_at FROM schema_migrations ORDER BY version DESC LIMIT 1",
            &[],
        )
        .await
        .context("Failed to query migration status")?;

    let (latest_version, applied_count) = if rows.is_empty() {
        (None, 0)
    } else {
        let count: i64 = client
            .query_one("SELECT COUNT(*) FROM schema_migrations", &[])
            .await?
            .get(0);
        (Some(rows[0].get::<_, i32>("version")), count as usize)
    };

    Ok(MigrationStatus {
        initialized: true,
        applied_count,
        latest_version,
        pending_count: 0, // Would need to scan migrations directory to calculate
    })
}

/// Status of database migrations
#[derive(Debug)]
pub struct MigrationStatus {
    /// Whether the migrations table has been initialized
    pub initialized: bool,
    /// Number of applied migrations
    pub applied_count: usize,
    /// Latest applied migration version
    pub latest_version: Option<i32>,
    /// Number of pending migrations (not applied yet)
    pub pending_count: usize,
}

/// Get the current schema version
pub async fn get_schema_version(pool: Pool) -> Result<Option<i32>> {
    let status = check_migration_status(pool).await?;
    Ok(status.latest_version)
}
